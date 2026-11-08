# C alias-checker storage evaluation

Production base: #824 after #890 (`ab57d2050`). These are benchmark-only prototypes;
no production generator, runtime or binding defaults change.

**Recommendation:** pursue one call-wide address collection and one sort for
checked C collection inputs. Preserve the existing omission of checks for
shared-only calls and the stack treatment of fixed inputs. Add an empty-input
shortcut before shipping. A single growing buffer generally beats the extra
nested-descriptor counting pass. Hash-only checking has important regressions;
its large shuffled-flat wins do not justify it as the general replacement.

## What was tested

The generated C fixture constructs Rust `Vec<u64>` / `Vec<Vec<u64>>` inputs from
opaque `Resource` pointers through `history_write(&mut Resource) -> u64`. The
shared case uses `history_read(&Resource)` plus a separate mutable `Resource`
parameter. The callback case returns owned `Vec<Vec<Resource>>` from C into Rust.
Every timed input has distinct, valid addresses. Failure probes run separately.
This measures complete native conversion calls, not packet throughput or JNI.

| Candidate | Operation |
|---|---|
| `current` | Current checked generation: pending vectors, sorted history, hash promotion and batch rollback. |
| `hash_grouped` | Existing validation/construction passes, with addresses inserted directly into one exact integer-hash table. No pending vectors or rollback. |
| `hash` | Existing unchecked conversion with an exact hash check immediately before each pointer acquisition. A late failure may consume a prefix. |
| `sort` | Collect all input addresses before construction, check them with one sort, discard the buffer, then use existing unchecked conversion. Reserve the already-known flat length; grow the buffer for nested inputs. |
| `sort_sized` | Same operation, with an additional nested-descriptor walk to reserve the exact total. Its flat native bodies are intentionally identical to `sort`. |

All candidates retain normal pointer/range/value validation. Sort preflight reads
only stable C descriptors and raw handles. It does not sort or copy payloads or
mutate the caller's rows. Callback result preflight runs after the existing
unread-owner and foreign row-release guards have been installed. The hash
prototype forwards vector capacity hints; nested inner hints overwrite outer
hints before acquisition in these fixtures. Consequently hash comparisons also
include their respective preparation/reservation code, not just pass count.

## Complete C-call results

Times are microseconds per call. The strongest borrowed-input comparison loads
all five native libraries into one C process and reuses exactly the same native
objects and C rows for each candidate. Four fresh process forks, 11 measured
batches targeting 20 ms, equal iteration counts, balanced per-batch order,
pinned CPU 1. Every call verifies its checksum. Native functions have verified
64-byte alignment. No compilation or allocation instrumentation runs during timing.

| Shape / total handles | Order | Current µs | Grouped hash µs | Direct hash µs | One-buffer sort µs | Sort/current time |
|---|---|---:|---:|---:|---:|---:|
| Flat / 8 | ascending | 0.102 | 0.175 | 0.162 | 0.075 | 0.732 |
| Flat / 8,192 | ascending | 49.108 | 80.806 | 83.211 | 26.006 | 0.532 |
| Flat / 8,192 | shuffled | 156.336 | 82.233 | 86.694 | 132.890 | 0.852 |
| Flat / 65,536 | ascending | 389.374 | 798.087 | 861.846 | 207.793 | 0.534 |
| Flat / 65,536 | shuffled | 1966.766 | 867.881 | 925.842 | 1795.202 | 0.912 |
| 8,192 singleton lists | shuffled | 1224.957 | 929.530 | 900.019 | 459.733 | 0.371 |
| 1,024 lists of eight | shuffled | 581.478 | 535.383 | 527.547 | 178.045 | 0.306 |
| 8,192 shared handles + mutable parameter | shuffled | 1237.271 | 933.196 | 905.471 | 479.335 | 0.387 |
| Shared-only control / 8,192 | ascending | 493.597 | 492.892 | 490.377 | 489.940 | 0.991 |

Ratios are medians of paired fork times (candidate/current, lower is better).
They need not equal the ratio of the displayed medians. The full
[same-address table](results/same-address-latency.csv) includes presizing and all
fork medians. The shared-only control omits alias checking in every candidate.

Owned callback results cannot reuse the same transferred objects between calls.
Their separate-process comparison uses the same compiled C caller, selecting
frozen native libraries with a loaded-library assertion, four balanced forks,
seven measured batches targeting 5 ms:

| Callback result / handles | Current µs | Grouped hash µs | Direct hash µs | One-buffer sort µs | Sort/current time |
|---|---:|---:|---:|---:|---:|
| 8,192 singleton lists / owned handles | 902.803 | 643.026 | 630.255 | 522.873 | 0.584 |

The broader [native sweep](results/native-latency.csv) covers sizes 0, 1, 8, 128,
512, 8,192 and 65,536; flat, singleton, groups-of-eight, mixed-access and callback
paths; ascending, shuffled and selected descending orders.

## Allocation and memory results

Complete-call Rust allocation counts at 8,192 shuffled handles, measured in
separate instrumented libraries. C fixture preparation is outside the counter.
The final Rust collections still need their own storage.

| Shape | Current allocations / reallocations | Sort allocations / reallocations | Current requested bytes | Sort requested bytes |
|---|---:|---:|---:|---:|
| Flat | 2 / 0 | 2 / 0 | 196,608 | 196,608 |
| Singleton lists | 16392 / 6 | 8194 / 11 | 950,064 | 524,224 |
| Groups of eight | 2056 / 4 | 1026 / 11 | 777,840 | 352,192 |
| Callback singleton lists | 16392 / 6 | 8194 / 11 | 950,064 | 524,224 |

The isolated checker at 8,192 shuffled singleton accesses changes from **8,199
allocations plus six reallocations to one allocation plus 11 reallocations**.
Peak live requested checker storage falls from 417,840 to 131,072 bytes. Exact
presizing keeps that peak and removes the reallocations. Thus one buffer does
not mean one heap operation. Requested bytes are cumulative allocation requests,
not peak memory or RSS. [All native counts](results/native-allocations.csv),
[checker counts/peaks](results/standalone-allocations.csv) and
[checker timings](results/standalone-latency.csv) are retained.

For the complete singleton-list call, exact presizing reduces requests from
524,224 to 393,216 bytes and removes 11 reallocations, but raises the confirmed
median from 459.733 to 495.967 µs. It also loses on groups of eight and the mixed
shared/mutable case. Lower allocation counts alone do not select the fastest path.

## Controls, regressions and limits

Separate-process identical-library controls exposed large flat-array variation;
those controls remain in [process-aa.csv](results/process-aa.csv). This motivated
the same-process/shared-address confirmation above. Its stronger control maps
five independent copies of the identical current library, verifies five distinct
function addresses and reuses the same input objects. Candidate/current ratios
are within approximately 0.5% of one across the tested flat, nested and shared-only
cells: [mapped-library controls](results/same-address-aa.csv). There are no
relocations against exported `bench_*` symbols in the measured native libraries.

The simpler designs have real costs. Direct hashing raises the 65,536-element
ascending flat median from 389.374 to 861.846 µs, and its eight-element flat case
is slower than current. The sorter adds work to empty nested and callback calls:
16.112 → 19.770 ns and 24.459 → 28.138 ns, respectively, in the broader sweep.
These empty cases need an early skip before a production rollout. Tiny differences
outside the stronger controls should not be generalized.

The native prototype intentionally covers this fixture's pointer-collection
recipes. It is not a universal conversion implementation. Production work still
needs an address-only traversal for all supported input shapes, including
selectors, inline owners and custom plans, plus recovering-error/ownership-receipt
coverage. Existing C row stability requirements apply. No additional lifetime or
reentry guarantee is introduced, and JNI's Kotlin preflight is outside this study.

## Correctness and reproduction

All candidates reject cross-group aliases and conflicts with another parameter.
Additional probes cover within-group aliases, repeated shared accesses, null
handles, null/nonempty and overflowing descriptors, malformed callback results
and exactly-once C row-release hooks. The isolated checker agrees on 5,000 unique
valid cases, 5,000 single-conflict cases, and shared repetitions across the
128-address promotion boundary up to 8,192 accesses. Checks stop after an error;
restoring checker state is deliberately unnecessary. CI builds the prototypes
and runs correctness probes, without timing gates.

```sh
python3 examples/bench-input-aliases/call-wide/build.py --output /tmp/alias-build
python3 examples/bench-input-aliases/call-wide/study.py \
  --build /tmp/alias-build --output /tmp/alias-study
python3 examples/bench-input-aliases/call-wide/confirm.py \
  --build /tmp/alias-build --output /tmp/alias-confirm
python3 examples/bench-input-aliases/call-wide/confirm.py \
  --build /tmp/alias-build --output /tmp/alias-control --aa
```

Use fresh output directories. This experiment targets Linux and needs cached
Cargo dependencies (the existing C fixture's `build.py --fetch` can populate them),
Rust, a C compiler, `nm`, `readelf` and `taskset`. Compilation finishes before
timing. The builder verifies checked/unchecked seed differences, access modes,
generated-source identity, distinct native artifacts and all aligned symbols.
The native shared library is asserted on every timed call process and correctness
probe. Source/artifact hashes, settings, compiler and profile information are in
[provenance.json](results/provenance.json). Measurement source commits are recorded;
subsequent changes add assertions, correctness/CI probes and this report. Raw
batches and logs remain outside Git and the PR.
