# JNI collection output ownership measurements

Measured on 2026-11-03, Linux x86-64, AMD EPYC 7713. This compares #824 at `de3617bea9f94d60aad3d5b531cc329b86a4dfb9` with the ownership implementation at `aa3c80058` (full revisions and build metadata are in the reports). Later commits record artifacts and results. These are generated-call measurements, not packet-throughput measurements.

## Change and scope

Ordinary JNI object-array outputs now retain accepted native owners through a stack guard over the completed array prefix. Collection-return callbacks transfer accepted rows to the generated Kotlin collector immediately. Both remove retained native per-row commit queues. Failure cleanup preserves nested ownership and the original JVM exception. Successful row delivery adds no JNI calls.

The former queue allocated one Vec per nonempty handle-bearing collection, not one allocation per closure. Nested one-handle lists expose many such queues. Primitive-only queues had zero-sized closures and did not allocate; their controls remain in the comparison. Kotlin row arrays, boxing, input locking/conversion and callback cleanup lambdas are unchanged.

## Native allocation counts

A benchmark-only global allocator counts allocations, reallocations and requested bytes on the calling thread. It excludes JVM heap allocations and JNI-internal allocator activity outside Rust’s global allocator. Totals include existing input conversion and Rust JNI-helper costs; they do not isolate output construction. Initial handle creation and final destruction are outside counting. Handles are reused through consuming round trips.

There are 19 case/size cells, two fresh JVMs per revision, 100 uncounted warmup operations and ten one-call count batches: 76 JVMs and 760 batches. Every count within each cell is identical. Empty, 1-, 64- and 512-handle shapes and primitive/scalar controls are recorded. Reallocation counts are unchanged in every cell. Requested bytes sum allocation/reallocation request sizes; they are not retained memory.

| Transport / shape | Handles | Native allocations before → after | Requested bytes before → after |
|---|---:|---:|---:|
| Object-array flat handles | 512 | 7,173 → 7,172 | 234,529 → 230,433 |
| Callback flat handles | 512 | 3,079 → 3,078 | 93,233 → 89,137 |
| Callback one-handle inner lists | 64 | 1,349 → 1,284 | 41,557 → 39,509 |
| Callback one-handle inner lists | 512 | 10,757 → 10,244 | 332,309 → 315,925 |
| Object-array one-handle inner lists | 64 | 1,411 → 1,346 | 43,617 → 41,569 |
| Object-array one-handle inner lists | 512 | 11,267 → 10,754 | 349,153 → 332,769 |

Flat 512-handle arrays and callbacks each remove one allocation and 4,096 requested bytes. Nested 512-handle shapes remove 513 allocations and 16,384 bytes; the 64-handle shapes remove 65 allocations and 2,048 bytes. Scalar and primitive-record allocation controls are unchanged. All sizes and raw counts are in [counts/results.json](counts/results.json) and [counts/samples.csv](counts/samples.csv).

## Uninstrumented latency

Separate release libraries disable allocation instrumentation. The same driver JAR is used throughout. All 3,535 JVM class/module entries are byte-identical across the four native builds, including the instrumented variants; [class-identity.json](class-identity.json) records the content-map and driver hashes.

One case/size runs per JVM, one JVM at a time, pinned to CPUs 0–1 with two active processors, a fixed 256 MiB G1 heap and no builds/tests alongside timing. Case order is deterministically shuffled and revision order alternates by fork. Warmup runs for at least two seconds and waits for 500 ms without compilation-time changes, with a 15-second cap; all runs settled. Twelve measured batches follow. Each JVM score divides total timed wall/CPU nanoseconds by total operations, and tables report the median of per-JVM scores with their full minimum–maximum range. GC events and every batch are retained. CPU frequency, turbo and other users’ host activity are not controlled.

The initial comparison has nine cells and four JVMs per revision: 72 JVMs and 864 batches. It measures complete calls, including normal input conversion/locking, Kotlin reconstruction or a fold collecting the same resulting list. Handles circulate through the calls and are closed after measurement. Primitive output cases include native source construction and a checksum.

| Case | N | Before ns/call [range] | After ns/call [range] | Wall change | CPU change |
|---|---:|---:|---:|---:|---:|
| Scalar control | 1 | 12.8 [12.5–13.2] | 13.2 [12.5–13.6] | +2.6% | +2.4% |
| Object-array flat handles | 1 | 4,686.4 [4,557.9–8,545.9] | 4,944.3 [4,569.5–6,324.2] | +5.5% | +6.7% |
| Object-array flat handles | 512 | 2,015,062.3 [1,910,570.5–2,202,282.0] | 1,937,178.4 [1,915,512.1–1,952,006.9] | -3.9% | -3.7% |
| Callback flat handles | 1 | 3,480.9 [3,253.6–3,547.2] | 3,473.0 [3,287.9–7,736.3] | -0.2% | +2.1% |
| Callback flat handles | 512 | 1,039,606.0 [1,003,939.1–1,232,124.4] | 1,074,114.1 [1,008,869.4–1,193,547.2] | +3.3% | +4.5% |
| Callback one-handle inner lists | 64 | 401,093.2 [385,686.5–476,820.8] | 398,408.4 [379,850.6–471,084.5] | -0.7% | -0.7% |
| Object-array one-handle inner lists | 64 | 412,744.0 [401,838.7–429,663.1] | 435,308.0 [432,801.3–440,963.6] | +5.5% | +4.5% |
| Primitive-record object array | 512 | 5,791,708.9 [5,693,071.6–6,213,316.6] | 5,725,727.3 [5,657,326.3–5,749,365.7] | -1.1% | -1.1% |
| Primitive-record callback fold | 512 | 157,229.8 [149,596.6–170,172.1] | 153,703.0 [149,404.8–180,774.7] | -2.2% | -3.9% |

Negative changes mean faster. Timings are mixed: flat 512-handle object arrays measured 3.9% faster, flat callbacks 3.3% slower, and 64 nested object-array rows 5.5% slower. These results do not show a general latency improvement. Raw samples and every fork score are in [latency/results.json](latency/results.json) and [latency/samples.csv](latency/samples.csv).

## Identical-build controls and nested confirmation

The A/A run uses the exact baseline JAR and library under both labels. Five cells with two forks give 20 JVMs and 240 batches. No sample, GC event or slow fork was discarded.

| Identical-build case | N | Before ns/call | After ns/call | Apparent wall change |
|---|---:|---:|---:|---:|
| Scalar control | 1 | 12.7 | 12.7 | -0.3% |
| Object-array flat handles | 1 | 4,881.9 | 4,618.2 | -5.4% |
| Object-array flat handles | 512 | 1,913,953.8 | 1,916,942.9 | +0.2% |
| Callback flat handles | 1 | 3,305.1 | 3,359.8 | +1.7% |
| Callback flat handles | 512 | 1,029,787.9 | 1,532,319.0 | +48.8% |

The unchanged 512-handle callback case includes one slow fork and moves by +48.8%, illustrating the limits of a two-fork aggregate. Its full range remains in [aa/results.json](aa/results.json).

The initial nested-array slowdown prompted a separate eight-fork comparison (16 JVMs, 192 batches), followed by four-fork nested A/A controls (8 JVMs, 96 batches), using the same artifacts and protocol.

| Nested arrays / 64 handles | Before ns/call [range] | After ns/call [range] | Wall change | CPU change |
|---|---:|---:|---:|---:|
| Before/after confirmation | 439,162.8 [402,387.8–782,722.6] | 450,883.4 [431,009.8–482,833.6] | +2.7% | +1.5% |
| Identical baseline under both labels | 454,603.5 [420,425.2–480,358.3] | 443,346.5 [426,366.9–485,137.2] | -2.5% | +0.7% |

The confirmation still has a slower after median (+2.7%), with overlapping fork ranges; its CPU median changes by +1.5%. The identical nested baseline control moves by −2.5% wall time and +0.7% CPU. The allocation reduction is established; these measurements do not establish a latency win or rule out a small nested-path slowdown. See [confirmation data](nested-confirmation/results.json) and [nested controls](nested-aa/results.json).

## Verification and reproduction

Workspace tests, Clippy with warnings denied, C ASan/UBSan and both complete Kotlin/JNI callers with `-Xcheck:jni` passed. Tests cover accepted-prefix conversion failures, rejected callbacks, optional/nested/multiple outputs, mixed bulk frames, exact destruction counts, original exception preservation, a destructor panic and a secondary cleanup exception. A separate diagnostic benchmark smoke run is excluded from timing. Generated example snapshots were checked, and benchmark snapshots match the compiled final fixture. [The ownership review](review.md) records the Claude Opus 5.5 findings and resolutions.

Build metadata records Rust/JVM/Kotlin versions, source revisions, dependency-lock and fixture hashes, release profile and native/JAR hashes. The measurement runner checks fixture/profile identity and artifact hashes before and after each comparison. Measurements used Kotlin 2.4.21, OpenJDK 11.0.29 and Rust 1.98.0. Keep the fixture checkout clean and committed; use a new output directory for every invocation. Set `KOTLINC`, `JAVA_OPTS` and `CARGO_BUILD_JOBS=1` identically for both revisions.

```sh
python3 examples/bench-bulk/jni-output-results/prepare.py \
  --checkout ../prebindgen-jni-output-baseline --output /tmp/jni-before \
  --target /tmp/jni-output-build-cache
python3 examples/bench-bulk/jni-output-results/prepare.py \
  --checkout . --output /tmp/jni-after --target /tmp/jni-output-build-cache
"$KOTLINC" -cp /tmp/jni-before/bench.jar \
  -Xfriend-paths=/tmp/jni-before/bench.jar \
  examples/bench-bulk/jni-output-results/Driver.kt -d /tmp/jni-output-driver.jar
taskset -c 0,1 python3 examples/bench-bulk/jni-output-results/measure.py \
  --before /tmp/jni-before --after /tmp/jni-after \
  --driver /tmp/jni-output-driver.jar --output /tmp/jni-comparison --forks 4
```

The baseline checkout must be at the recorded #824 revision. `prepare.py` snapshots the native library before shared build storage is reused. Prepare separate builds with `--counts` and run `measure.py --counts` for native allocations. Use `--check-jni --forks 1` for diagnostic validation, and use the same baseline path for `--before` and `--after` for A/A. `--cases nested` reproduces the focused confirmation. Native libraries and JARs are not committed.
