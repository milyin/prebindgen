# Mixed inline-owner C callback results

Baseline: merged #824 at `0d5ae09f60e034fa0e73ba933f78a6c79ee936d2`.
The initial implementation is `c53633d94`; the final implementation also includes
the reservation fix in `3d79deb5c`. The same expanded fixture, dependency lock,
compiler, C caller and alias policy are used for both sides of each comparison.
Only aggregate results and build provenance are committed; raw batch/console logs
stay outside Git.

Rust calls a C callback that returns a stable array of rows. `callback_native_mixed`
rows contain two owning pointer handles and one inline native `NativePacket`.
The packet contains an id, a three-byte Vec and a three-byte String, each with
capacity 64. The checked decoder previously captured the packet carrier and a
construction closure in a temporary vector for every row. It now checks pointer
addresses first, rereads the original C rows, and constructs directly in the
final native vector. The final reservation fix counts only the two pointer
identities actually inserted into alias history. Borrowed/mutable by-value
carriers, callback fields and nested outer collections retain their previous path.

Source owners, handles, C rows and payload allocations are prepared outside
counting and timing. Measurements include callback invocation, conversion,
source function execution, result-storage release callback and native destruction.
C storage remains alive and unchanged throughout decoding. On success the release
hook frees only row storage while Rust owns the constructed values; on failure
Rust drops acquired values and reclaims unread owners before releasing the rows.
Native payloads retain their original allocations/capacity. These are complete-call
C microbenchmarks, not network-throughput measurements.

## Allocation requests at 512 rows

| Checks | Case | Allocation calls, before → after | Requested bytes, before → after |
|---|---|---:|---:|
| Enabled | callback_native_mixed | 3 → 2 | 102,400 → 53,248 |
| Disabled | callback_native_mixed | 1 → 1 | 36,864 → 36,864 |
| Enabled | callback_native | 1 → 1 | 28,672 → 28,672 |
| Enabled | callback_many | 2 → 2 | 12,288 → 12,288 |
| Enabled | native_mixed | 2 → 2 | 61,440 → 61,440 |
| Enabled | handles | 2 → 2 | 12,288 → 12,288 |

The final checked path saves 48 KiB per 512-row call: 40 KiB of prepared-row
storage (80 bytes/row) plus 8 KiB of unused alias reservation (16 bytes/row).
Remaining storage is the final native vector and enabled alias history. Counts
and byte sizes are constant across all samples at sizes 0, 1, 8, 64, 512 and 4096.
Every measured case has zero reallocations; empty arrays allocate zero times.
Requested bytes are allocator requests, not peak resident memory or copied bytes.

`callback_native` has only by-value inline owners and already constructed directly;
`callback_many` has only pointer handles and already replayed rows. The ordinary
`native_mixed` input and handle input are also controls. Their allocation budgets
are unchanged. The generated-boundary test independently checks exact allocation
counts and bytes, with alias checks on/off, at sizes 0–4096. C runtime tests also
verify original payload addresses and capacity 4096, aliases, invalid handles,
constructor panics, inactive optional fields, destruction and release ordering.
A strict-provenance Miri test exercises mixed by-value owner/pointer rows.

## Complete-call latency

Times are medians across six fresh processes per version, pinned to CPU 1, with
eight measured batches per process after calibration and warmup. Version order
alternates by process pair; cell order is shuffled. Local compilation and Kotlin
verification finished before timing. Allocation counting is disabled in timing
builds. The target is 20 ms/batch, bounded by 262,144 prepared rows to limit memory;
actual iteration counts and durations are recorded in the original run artifacts.

The final checked implementation gives these results. The paired column is the
median of each process pair's after/before change; it can differ from the ratio
of the two latency medians. The last column runs the identical baseline library
on both sides as a noise control (four fresh processes per side).

| Mixed rows | Before, µs | After, µs | Change | Paired median change | Identical-library change |
|---:|---:|---:|---:|---:|---:|
| 1 | 0.146 | 0.123 | -15.5% | -14.7% | -3.4% |
| 64 | 7.680 | 7.097 | -7.6% | -6.2% | — |
| 512 | 58.116 | 58.914 | +1.4% | -2.8% | -1.9% |
| 4096 | 465.734 | 460.209 | -1.2% | -3.5% | — |

Small batches improved in this run. Large-array timing remains inconclusive:
the 512-row final process ranges are 57.0–80.9 µs before and 55.1–76.8 µs after.
The paired changes include both signs, and the identical-library comparison also
moves by a few percent. The initial staging-only implementation measured -4.9%
at 512 rows and -7.3% at 4096 rows, before trimming alias reservation. Both runs
remain in the aggregate JSON; they do not establish a consistent large-array
speedup. The repeat establishes the memory saving independently of timing.

The final `callback_native` control, which receives no allocation optimization,
measured -1.9% at 512 rows and +3.8% at 64 rows. Initial checked 512-row controls
measured -0.6% for `callback_many`, +2.0% for ordinary `native_mixed` and +4.0%
for handles. These unfavorable and favorable control changes are retained.

Unchecked mixed rows measured +3.3%, +3.0%, -3.3% and -1.7% at sizes 1, 64, 512 and
4096 respectively. Their allocation count is unchanged and no consistent speedup
is claimed. The rebuilt final unchecked generated Rust and native library are
byte-for-byte identical to the measured unchecked build, so those measurements
apply to the final implementation. The final checked repeat uses the final
reservation code; provenance records every build and revision.

## Reproduction

Use this checkout's fixture to build both generator revisions into separate empty
directories. Commit sources before freezing a build. Add `--count-allocations`
for count builds or `--disable-alias-checks` for unchecked builds:

```sh
python3 examples/bench-c-collections/build.py --checkout /path/to/baseline --output /tmp/mixed-before
python3 examples/bench-c-collections/build.py --output /tmp/mixed-after
taskset -c 1 python3 examples/bench-c-collections/input-compare.py --before /tmp/mixed-before --after /tmp/mixed-after --output /tmp/mixed-time --kind time --cases callback_native_mixed,callback_native --sizes 1,64,512,4096 --forks 6 --batches 8
# Counts: --kind count --sizes 0,1,8,64,512,4096 --forks 1 --batches 3
# Identical-library control: append --same-build before, use --sizes 1,512 --forks 4.
python3 examples/bench-c-collections/inline-input-results/summarize.py checked-time=/tmp/mixed-time --output /tmp/mixed-summary
```

The shared exporter accepts multiple named runs. `summary.json` includes the
initial and final checked runs, unchecked results, allocation budgets and both
identical-library controls; it retains per-process aggregates, ranges, CPU times
and paired changes. `provenance.json` records source/toolchain/ABI/artifact hashes,
revisions, process settings and CPU information. Raw logs are not committed.
