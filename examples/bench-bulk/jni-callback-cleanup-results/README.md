# JNI bulk callback cleanup tracking

This change removes the temporary list of owned handle addresses and their
cleanup functions from successful Kotlin callback-result decoding. A stack guard
keeps access to the native bytes already copied from Kotlin. Only a conversion
error or unwind triggers a scan that releases unread handles. The guard is
cancelled after the final Rust value has been constructed. Normal pointer/alias
checks and Kotlin locking remain.

Only callback results with all owned handles in independently cached bulk byte
buffers qualify. Non-bulk and mixed noncached results keep their current guards.
Nested/optional handle arrays and fixed records qualify. The cleanup scan skips
borrowed fields, deduplicates aliases of consumed handles, bounds-checks each
native read and continues to valid siblings after a malformed nested range.
No new public configuration or extra marshalling copies are introduced.

**Allocation savings are repeatable; complete-call latency is inconclusive.**
The first timing run favored large arrays, but a longer focused repeat did not
reproduce that result. Identical-library controls also vary substantially.

## Native allocation operations

These counts cover the calling-thread Rust allocator during conversion, the
source call, returned handle creation and cleanup. Each cell has two fresh JVMs,
1,000 warmup calls and ten measured calls per JVM. All twenty observations agree.
The allocation-instrumented builds are separate from ordinary release timing.

| Bulk callback result | Rows | Allocations before → after | Reallocations before → after | Requested native bytes before → after |
| --- | ---: | ---: | ---: | ---: |
| One record | 1 | 12 → 11 | 6 → 6 | 314 → 218 |
| Record array | 0 | 10 → 10 | 6 → 6 | 210 → 210 |
| Record array | 1 | 14 → 13 | 7 → 7 | 394 → 298 |
| Record array | 8 | 21 → 20 | 8 → 7 | 1,090 → 802 |
| Record array | 64 | 77 → 76 | 11 → 7 | 7,810 → 4,834 |
| Record array | 512 | 525 → 524 | 14 → 7 | 61,570 → 37,090 |
| Record array | 4,096 | 4,109 → 4,108 | 17 → 7 | 491,650 → 295,138 |

For 512 rows, the removed tracking vector saves one allocation, seven growth
reallocations and 24,480 cumulative requested native bytes. One fixed record
saves one allocation and 96 requested bytes. Requested bytes include the sizes
requested by growth reallocations; they are not peak live memory. The byte frame,
final Rust collection, alias bookkeeping and returned native handle allocations
remain.

Controls retain their allocation counts: a normal 512-row bulk input/output
round trip uses 516 allocations/one reallocation/36,912 requested bytes in both
versions. Non-bulk callback results retain 13 allocations/eight reallocations
for one record and 7,688 allocations/5,642 reallocations for 512 records. The
unchanged per-row JNI work is included in that latter total. Empty bulk results
also retain their counts.

## Complete-call latency

Both revisions run the exact same baseline JAR, after checking identical
generated Kotlin source hashes. Each call returns an existing owned handle/tag
record or list from Kotlin to Rust, then returns the reconstructed Rust result
to Kotlin. The next call reuses those returned owners. Initial allocation and
final disposal are outside timing; packing, locks, JNI, native conversion,
output construction, GC and checksum/read calls remain timed.

Each entry below is the median of fresh-JVM medians. The main run has six JVMs
per version, ten warmup batches and eleven measured batches targeting 30 ms.
The focused repeat has four JVMs per version and fifteen measured batches
at 50 ms after ten warmup batches. Version order alternates and case order
rotates/reverses. Negative percentages mean lower elapsed time, calculated as
`after median / before median - 1`. These compare generator revisions using the
same transport, not bulk against ordinary object conversion.

| Run | Bulk callback result | Rows | Before µs/call | After µs/call | Change |
| --- | --- | ---: | ---: | ---: | ---: |
| Main | One record | 1 | 4.076 | 4.262 | +4.6% |
| Main | Record array | 0 | 3.565 | 3.537 | -0.8% |
| Main | Record array | 1 | 4.196 | 3.810 | -9.2% |
| Main | Record array | 8 | 5.247 | 5.149 | -1.9% |
| Main | Record array | 64 | 17.355 | 15.482 | -10.8% |
| Main | Record array | 512 | 91.585 | 84.354 | -7.9% |
| Main | Record array | 4,096 | 710.392 | 634.752 | -10.6% |
| Focused | One record | 1 | 3.973 | 3.851 | -3.1% |
| Focused | Record array | 512 | 83.788 | 84.239 | +0.5% |
| Focused | Record array | 4,096 | 613.173 | 616.003 | +0.5% |

The focused repeat changes the initial 512-/4,096-row gains to approximately
0.5% slower medians. The single-record result changes from 4.6% slower to 3.1%
faster. Neither direction is a stable complete-call result.

Controls and A/A comparisons retain the same protocol. A/A loads the **same
baseline library and JAR** under both labels: four JVMs per label in the main
control, and two per label in the longer focused control.

| Run/control | Case | Rows | Before µs/call | After µs/call | Change |
| --- | --- | ---: | ---: | ---: | ---: |
| Main / non-bulk callback | Record array | 512 | 3028.883 | 3096.138 | +2.2% |
| Main / normal bulk call | Record array | 512 | 95.175 | 94.490 | -0.7% |
| Focused / normal bulk call | Record array | 512 | 101.754 | 85.947 | -15.5% |
| Main A/A | One record | 1 | 3.542 | 4.009 | +13.2% |
| Main A/A | Record array | 512 | 84.477 | 81.677 | -3.3% |
| Main A/A | Record array | 4,096 | 617.933 | 614.909 | -0.5% |
| Focused A/A | Record array | 512 | 99.885 | 79.742 | -20.2% |
| Focused A/A | Record array | 4,096 | 590.322 | 582.598 | -1.3% |

The focused A/A 512-row shift is about 20% despite identical artifacts. The
unchanged normal bulk-call control shifts about 15% in the focused comparison.
All aggregates, per-JVM medians and ranges are retained, including slower cases.
The JSON also reports medians of paired ratios, a different statistic from the
ratio of medians shown here. No latency or network-throughput speedup is claimed.

Generated Kotlin is unchanged between revisions. Main-run JVM allocation for
bulk arrays is unchanged at 0, 1 and 8 rows, with small JVM optimization variation
at larger sizes. For the fixed record it is unchanged. The intended reduction
is Rust cleanup-tracking storage, not JVM allocation.

## Provenance and reproduction

The generator baseline is `cafa3eae0551df793e5cf5834864b6344eadffb2` (#882 merged
into #824). The candidate is `bfffa33ed`, including preservation of ordinary
callback generation. Later snapshot/documentation commits do not change the
implementation. Both use the same benchmark fixture, pinned lock, toolchains
and build configuration recorded in [summary.json](summary.json). Frozen native
libraries have separate allocation and timing hashes; timing rejects allocator
instrumentation and all runners check artifact hashes before and after use.

The host permits CPUs 0 and 1. JVMs use a fixed 256 MiB G1 heap with two active
processors. These synthetic elapsed-time measurements include normal GC and
scheduler variation. They do not measure network packet throughput.

From this checkout, create a committed, clean detached baseline worktree at
`../cleanup-base`. Use empty output directories, set `KOTLINC` or put the Kotlin
compiler on PATH, and finish all compilation before timing.

```sh
python3 examples/bench-bulk/run.py --checkout ../cleanup-base --output /tmp/cleanup-before --build-only
python3 examples/bench-bulk/run.py --output /tmp/cleanup-after --build-only
python3 examples/bench-bulk/jni-callback-cleanup-results/compare.py --before /tmp/cleanup-before --after /tmp/cleanup-after --output /tmp/cleanup-latency --forks 6
python3 examples/bench-bulk/jni-callback-cleanup-results/compare.py --before /tmp/cleanup-before --after /tmp/cleanup-after --output /tmp/cleanup-aa --forks 4 --aa
python3 examples/bench-bulk/jni-callback-cleanup-results/focused.py --before /tmp/cleanup-before --after /tmp/cleanup-after --output /tmp/cleanup-focused --forks 4
python3 examples/bench-bulk/jni-callback-cleanup-results/focused.py --before /tmp/cleanup-before --after /tmp/cleanup-after --output /tmp/cleanup-focused-aa --forks 2 --aa
python3 examples/bench-bulk/run.py --checkout ../cleanup-base --output /tmp/cleanup-before-count --build-only --native-allocations
python3 examples/bench-bulk/run.py --output /tmp/cleanup-after-count --build-only --native-allocations
python3 examples/bench-bulk/jni-callback-cleanup-results/count.py --before /tmp/cleanup-before-count --after /tmp/cleanup-after-count --output /tmp/cleanup-counts --forks 2
```

`--target-dir` can share a Cargo cache: each build copies the native library into
its own output before hashing, so later builds cannot replace measured binaries.
Raw console logs and individual batch samples remain local; this directory
contains only runners, aggregate results, provenance and the
[review outcome](review.md).

Workspace tests, final JNI unit tests, workspace Clippy with warnings denied,
formatting, C ASan/UBSan and the full Kotlin/JNI boundary suite pass. The new
single-record and array benchmarks pass four fresh JVMs with `-Xcheck:jni`;
those correctness timings are excluded from the performance results.
