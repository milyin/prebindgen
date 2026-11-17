# JNI bulk record input replay

The candidate removes the native vector of per-row construction closures for
eligible bulk record inputs. It first checks handle addresses for conflicting
access against the cached native frame, then rereads that frame and constructs
the final Rust vector. Text/byte decoding and source constructors run once.
Normal JVM object rows keep decoding once; nested collection shapes retain their
existing conversion path. Kotlin handle locking and required marshalling copies
are unchanged.

Allocation savings are repeatable. Complete-call latency is inconclusive: the
before/after medians include both improvements and regressions, and identical
library comparisons also vary substantially. This report makes no throughput or
latency speedup claim.

## Native allocations

Both revisions bind the same benchmark fixture. These calling-thread Rust counts
include conversion, the source call, returned owner export and cleanup. Each
cell has two fresh JVMs with 1,000 warmup calls and ten measured calls per JVM;
all twenty observations are identical. Allocation instrumentation is built
separately and is never used for latency.

512-row calls, alias checks enabled:

| Bulk case | Allocations before → after | Reallocations before → after | Requested native bytes before → after | Bytes saved |
| --- | ---: | ---: | ---: | ---: |
| Handle + scalar | 517 → 516 | 1 → 1 | 53,296 → 36,912 | 16,384 |
| Handle + 64 B text + 64 B bytes | 1,541 → 1,540 | 4 → 4 | 524,560 → 499,984 | 24,576 |
| Callback-returned handle + scalar | 526 → 525 | 14 → 14 | 77,954 → 61,570 | 16,384 |
| Raw handle list | 517 → 517 | 1 → 1 | 32,816 → 32,816 | 0 |
| One-handle inner lists | 2,051 → 2,051 | 10 → 10 | 90,128 → 90,128 | 0 |

Each nonempty changed sequence saves one allocation, holding 32 bytes per
handle/scalar row or 48 bytes per text/byte row. At 4,096 rows this saves
131,072 and 196,608 requested bytes respectively. Remaining per-handle output
allocations and required payload copies are included in the totals. Empty lists,
ordinary object-row controls, raw-handle lists and nested-list controls retain
their allocation counts. Sizes 0, 1, 8, 64, 512 and 4,096 are recorded; the
many-small-inner-lists control stops at 512.

## Complete-call latency

Each table value is the median of six fresh-JVM medians; each JVM median covers
nine calibrated batches targeting 20 ms after six warmup batches. Before/after
order alternates and case order rotates/reverses. Both libraries use the exact
same baseline JAR after verifying identical generated Kotlin source hashes.
Timing includes Kotlin packing, locks, JNI, native conversion, the source call,
output reconstruction and a checksum/read of the result. Each call reuses the
owners returned by the previous call. Text/byte rows contain 64 bytes of each.

The percentage below is `(after median / before median - 1) * 100`; negative is
faster. It is not a bulk-versus-object comparison: both changed sides use bulk.
The JSON also retains per-process medians, ranges and the median of paired ratios,
which is a different statistic.

| Bulk case | Rows | Before µs/call | After µs/call | Change |
| --- | ---: | ---: | ---: | ---: |
| Handle + scalar | 1 | 0.944 | 0.896 | -5.1% |
| Handle + scalar | 8 | 1.655 | 1.664 | +0.6% |
| Handle + scalar | 512 | 68.059 | 69.385 | +1.9% |
| Handle + scalar | 4,096 | 555.337 | 542.768 | -2.3% |
| Handle + text + bytes | 1 | 1.737 | 1.762 | +1.5% |
| Handle + text + bytes | 8 | 7.492 | 7.603 | +1.5% |
| Handle + text + bytes | 512 | 451.269 | 425.942 | -5.6% |
| Handle + text + bytes | 4,096 | 3952.294 | 4233.868 | +7.1% |
| Callback-returned handle + scalar | 1 | 3.596 | 3.481 | -3.2% |
| Callback-returned handle + scalar | 8 | 4.437 | 4.442 | +0.1% |
| Callback-returned handle + scalar | 512 | 80.884 | 82.206 | +1.6% |
| Callback-returned handle + scalar | 4,096 | 589.064 | 583.788 | -0.9% |

512-row controls, measured by the same method:

| Control | Before µs/call | After µs/call | Change |
| --- | ---: | ---: | ---: |
| Bulk raw handle list | 59.470 | 59.480 | +0.0% |
| Bulk primitive input | 2.038 | 1.899 | -6.8% |
| Bulk shared handle input | 12.314 | 13.374 | +8.6% |
| Ordinary object-row handle/scalar list | 1705.048 | 1665.852 | -2.3% |

An additional four-JVM-per-label A/A run loads the **same baseline native library
and JAR** under both labels. Its 512-row handle/scalar medians are
68.236 versus 81.299 µs, and callback-record medians are 76.592 versus 95.401 µs.
This variation is larger than the changed-path differences. No stable latency
improvement or regression can be isolated from these runs. The full aggregates
retain slower cases and all controls.

Kotlin/JVM allocation is unchanged in the main 1–512-row changed cases: at
512 rows, handle/scalar, text/byte and callback records respectively allocate
73,240, 876,472 and 73,264 JVM bytes/call in both revisions. Large-case readings
and shared-handle controls vary slightly with JVM optimization. The generated
Kotlin is identical; the intended saving is native temporary row storage.

## Provenance and reproduction

Generator baseline: `634b548ad5c5957b9a3e4bb399a27bf9297ce4c4` (#881 merged into
#824). Candidate: `910120e9b` (including the separate-bulk-field lifetime fix).
Both use the fixture source hashes in [summary.json](summary.json). Native
libraries are frozen into separate output directories and hashed; later shared
Cargo-cache builds cannot replace them. The build records release optimization
level 3, disabled debug assertions, clean dependency repositories, the pinned
lock, toolchain/configuration identity and generated Kotlin hashes.

The host has two allowed CPUs (0 and 1). JVMs use a fixed 256 MiB G1 heap and two
active processors. This is a local elapsed-time microbenchmark, with ordinary GC
and scheduler variation; it does not measure network packet throughput.

From this checkout, prepare a detached worktree at the baseline as `../replay-base`.
Use new empty output directories for all commands, and a Kotlin compiler on PATH
or set `KOTLINC`. Each version must be committed and clean when built.

```sh
python3 examples/bench-bulk/run.py --checkout ../replay-base --output /tmp/replay-before --build-only
python3 examples/bench-bulk/run.py --output /tmp/replay-after --build-only
python3 examples/bench-bulk/jni-input-replay-results/compare.py --before /tmp/replay-before --after /tmp/replay-after --output /tmp/replay-latency --forks 6
python3 examples/bench-bulk/jni-input-replay-results/compare.py --before /tmp/replay-before --after /tmp/replay-after --output /tmp/replay-aa --forks 4 --aa
python3 examples/bench-bulk/run.py --checkout ../replay-base --output /tmp/replay-before-count --build-only --native-allocations
python3 examples/bench-bulk/run.py --output /tmp/replay-after-count --build-only --native-allocations
python3 examples/bench-bulk/jni-input-replay-results/count.py --before /tmp/replay-before-count --after /tmp/replay-after-count --output /tmp/replay-counts --forks 2
```

Finish all compilation before timing. `--target-dir` can share a Cargo cache.
The result artifact contains compact aggregates and provenance; raw console logs
and batch samples remain local. [review.md](review.md) records the source review
outcome. The full C ASan/UBSan and Kotlin/JNI boundary suite, workspace tests,
Clippy and committed generated-file checks pass. A separate four-JVM smoke run
of the changed benchmark cases passes with `-Xcheck:jni`; those timings are
excluded from this report.
