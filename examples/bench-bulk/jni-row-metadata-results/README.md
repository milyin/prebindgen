# JNI row metadata cache measurements

Compare #824 at `eab81c99d993` with the metadata-cache branch at `ece53f5b23d4`.
The same generated wrappers decode ordinary Kotlin collection inputs and records returned by Kotlin callbacks to Rust. The change caches built-in Java classes and primitive unboxing methods; null/type checks and ownership handling stay in place. Warm JNI tracing verifies zero `FindClass`, `GetObjectClass`, `GetMethodID` and `NewGlobalRef` calls in these readers.

In the table, `plain` uses ordinary JNI conversion; `bulk` packs expanded fields into C-layout byte buffers.

Times are **complete successful calls**, including Kotlin packing, generated ownership handling, Rust work and output reconstruction where applicable. A callback return case has Rust invoke Kotlin, Kotlin return the record(s) to Rust, and the outer Rust function return the resulting record(s) to Kotlin. It does not isolate the decoder. Native counts include all calling-thread Rust allocations across that call, including handle objects created for its output. They exclude JVM allocations and JVM/native internals outside Rust's allocator.

| Transport | Case | Elements | µs/call, before → after | Paired speedup | Native allocations / reallocations, before → after |
|---|---|---:|---:|---:|---:|
| plain | Integer list input | 1 | 1.836 → 0.484 | 3.68× | 6 / 4 → 1 / 0 |
| plain | Integer list input | 512 | 878.572 → 179.833 | 4.87× | 2,561 / 2,048 → 1 / 0 |
| plain | Eight-field record list input | 1 | 9.590 → 1.935 | 4.98× | 34 / 25 → 1 / 0 |
| plain | Eight-field record list input | 512 | 4,865.011 → 915.786 | 5.32× | 16,897 / 12,800 → 1 / 0 |
| plain | Owned handle/tag record roundtrip | 1 | 4.293 → 1.950 | 2.18× | 14 / 8 → 5 / 1 |
| plain | Owned handle/tag record roundtrip | 512 | 1,652.143 → 495.679 | 3.31× | 5,124 / 3,585 → 516 / 1 |
| plain | Kotlin callback returns record list to Rust | 1 | 7.032 → 3.207 | 2.19× | 23 / 14 → 9 / 3 |
| plain | Kotlin callback returns record list to Rust | 512 | 2,542.225 → 708.566 | 3.56× | 7,688 / 5,642 → 520 / 10 |
| plain | Kotlin callback returns one record to Rust | 1 | 3.488 → 1.669 | 2.12× | 13 / 8 → 5 / 2 |
| bulk | Integer list input | 512 | 2.726 → 2.880 | 1.01× | 2 / 0 → 2 / 0 |
| bulk | Eight-field record list input | 512 | 14.271 → 14.533 | 0.99× | 2 / 0 → 2 / 0 |
| bulk | Owned handle/tag record roundtrip | 512 | 75.617 → 74.571 | 1.01× | 516 / 1 → 516 / 1 |
| bulk | Kotlin callback returns record list to Rust | 512 | 73.031 → 70.213 | 1.04× | 524 / 7 → 519 / 3 |
| bulk | Kotlin callback returns one record to Rust | 1 | 3.380 → 2.048 | 1.65× | 11 / 6 → 6 / 2 |
| plain | Scalar call | 1 | 0.013 → 0.013 | 1.02× | 0 / 0 → 0 / 0 |

The record input has eight `i64` fields. The handle/tag record has one owned native handle and one `i64`. The bulk callback cases still decode an outer byte-array/length pair through ordinary row readers, so caching saves a fixed amount there too. Bulk ordinary inputs and the scalar call serve as controls.

The A/A run shows paired timing drift up to 7.7% with unchanged code. Treat the small bulk/scalar timing differences, including the 1.04× bulk callback-array result, as noise; the native allocation reductions are exact. The ordinary 512-record input improves in every fork (4.7–5.4×), as does the ordinary callback-list result (3.4–3.8×).

Native counts reproduced exactly in all 20 observations per cell. Cumulative requested native bytes, all latency cells, fork medians and A/A results are in [summary.json](summary.json). Requested bytes add allocation sizes and new reallocation sizes; they are neither live memory nor peak memory.

Measurements use release Rust, identical compiled JVM classes, a fixed 256 MiB JVM heap, G1 and two active processors. Latency uses the same baseline JAR for both frozen libraries, six balanced before/after JVM pairs, 10 warmup batches and 11 measured batches targeting 30 ms each. The table reports the median of per-fork medians; speedup is the reciprocal of the median paired after/before ratio. The A/A run loads the unchanged baseline under both labels. Allocation counting uses separate instrumented builds, two JVMs per cell, 1,000 warmup calls and 10 counted calls per JVM. Compilers and allocation counting do not run concurrently with timing.

Builds sharing the Cargo target directory run sequentially. The driver freezes each library before Kotlin compilation and loads that frozen library in a separate JVM to verify its actual instrumentation mode. Only uninstrumented builds are timed. Generated-source checks pass for all four builds. Raw logs and samples remain outside Git; the committed summary includes their hashes and build provenance.

Reproduce from the benchmark fixture with the same pinned lockfile and toolchains:

```sh
python3 examples/bench-bulk/run.py --checkout /path/to/before --output /tmp/row-before --build-only --check-generated
python3 examples/bench-bulk/run.py --checkout /path/to/after --output /tmp/row-after --build-only --check-generated
python3 examples/bench-bulk/jni-row-metadata-results/compare.py --before /tmp/row-before --after /tmp/row-after --output /tmp/row-timing
python3 examples/bench-bulk/jni-row-metadata-results/compare.py --before /tmp/row-before --after /tmp/row-after --output /tmp/row-aa --aa
```

Build two more directories with `--native-allocations`, then pass those directories to [count.py](count.py). Its optional `--smoke` adds `-Xcheck:jni` and is for correctness rather than timing.
