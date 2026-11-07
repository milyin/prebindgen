# Typed collection callback capture

When a Rust function returns a collection through generated JNI callbacks, Kotlin retains each element until the native call and handle locks finish. Capture previously allocated an `Array<Any?>` per row and boxed its primitive fields. The final implementation uses small immutable generated classes whose raw fields have the JVM wire types. Each row still has one holder object; primitive boxes and generic row arrays are eliminated from capture. Domain objects, user constructors, folds and handlers remain deferred until after JNI and locks finish.

Callbacks with matching JVM descriptors and Kotlin field types reuse the generated holder class, with a distinct immutable instance per row. Classes have stable readable hash names and expose raw fields with `@JvmField`. The collector keeps the existing `ArrayList` capacity policy, which copies only holder references during growth. Native ownership transfers after a successful append. If holder allocation or append fails, Rust retains the rejected row; Kotlin cleanup releases the accepted prefix. This covers the callback portion of #824's intermediate-row/boxing item. Ordinary JNI object-array outputs and their native row boxing remain tracked there.

## Revisions and method

- Before: release artifacts from `7c3f8988d9e22957dd252ea403048e39bce0ce48`. Their generator/runtime and benchmark sources are identical to merged #824 at `5e5d0909fb2adf982213c8a119c7cb1f0e708afe`, excluding `cfg(test)` assertions. The source diff and original library/JAR hashes were checked before reuse; metadata records this explicitly.
- Final typed holders: `7eacf4d1985df260a64bc3332c6f4473a4f3a255`. Later commits contain benchmark snapshots and reports only.
- Column prototype: `883bc6b440b37ef91853f32b7e2d1b72c9472c00`, measured separately below.
- Generated `plain.rs` and `bulk.rs` are byte-identical in all variants. Only the two callback-bearing Kotlin benchmark files change. [source-identity.json](source-identity.json) records native-source and shared-driver hashes.
- Both variants use the same fixture, unchanged driver JAR, compiler versions, dependency lock and release settings, checked by the runner. Kotlin 2.4.21, OpenJDK 11.0.29, Rust 1.98.0, Linux x86-64, two CPUs (`taskset -c 0,1`), fixed 256 MiB heap and G1. Full metadata/artifact hashes are in [results.json](results.json).
- Four fresh JVM processes per case/version, alternating before/after order. Each warms for at least two seconds and a 500 ms pause in compilation, then records twelve batches. No builds or tests run alongside timing. Each process's score is total measured time divided by total calls; tables show the median and full range of four scores. Lower time is better; change is `(after / before − 1) × 100`.

## Final typed holders versus #824

`primitive_fold` returns and folds records of eight `i64` fields. `fold_flat` consumes a handle list and returns its handles through the callback; its whole-call cost includes existing input conversion and locking. `fold_nested` returns one-handle inner lists through reference-valued callback arguments. `primitive_array` is an unchanged ordinary object-array return; `control` is an unchanged scalar native call. Rows are collection lengths, not field counts. JVM bytes include the entire generated call, not just capture. The same primitive/handle data and ownership operations are used on both sides.

| Case | Rows | Before µs [min–max] | After µs [min–max] | Time change | JVM bytes/call before → after |
|---|---:|---:|---:|---:|---:|
| `control` | 1 | 0.013 [0.013–0.013] | 0.013 [0.013–0.013] | +1.02% | 0.0 → 0.0 |
| `fold_flat` | 1 | 3.206 [3.185–3.726] | 3.304 [3.251–3.400] | +3.05% | 608.0 → 584.0 |
| `fold_flat` | 512 | 1061.337 [994.118–1213.355] | 1050.780 [1003.440–1083.586] | -0.99% | 95,568.0 → 83,280.0 |
| `fold_nested` | 64 | 423.279 [387.886–460.486] | 396.312 [386.792–413.130] | -6.37% | 54,988.6 → 54,518.4 |
| `primitive_array` | 512 | 6012.943 [5778.774–6067.224] | 5888.890 [5794.502–6114.374] | -2.06% | 168,016.0 → 168,016.0 |
| `primitive_fold` | 0 | 0.873 [0.854–0.999] | 0.894 [0.876–0.906] | +2.42% | 80.0 → 80.0 |
| `primitive_fold` | 1 | 1.261 [1.241–1.318] | 1.207 [1.185–1.234] | -4.21% | 400.0 → 240.0 |
| `primitive_fold` | 64 | 16.803 [16.382–18.096] | 14.895 [14.594–15.473] | -11.36% | 17,896.0 → 7,656.0 |
| `primitive_fold` | 65 | 17.001 [16.357–17.551] | 15.031 [14.734–15.555] | -11.59% | 18,160.0 → 7,760.0 |
| `primitive_fold` | 512 | 126.012 [120.295–127.201] | 107.216 [106.194–109.303] | -14.92% | 142,008.0 → 60,088.0 |
| `primitive_fold` | 513 | 129.190 [127.584–132.566] | 107.882 [106.537–113.466] | -16.49% | 142,272.0 → 60,192.0 |

Primitive-record allocation falls from 142,008 to 60,088 bytes/call at 512 rows (57.7%). At 64, 65 and 513 rows the reductions are also about 57–58%, without a capacity-boundary spike. The eight primitive boxes plus generic row array are replaced by one typed holder, saving 160 JVM bytes per primitive record in these workloads. The empty case stays at 80 bytes; the single-record case falls from 400 to 240 bytes.

Primitive-record median complete-call latency improves 4.2% at one row and 11.4–16.5% at 64–513 rows; before/after ranges for those nonempty primitive cases do not overlap in this run. These are warmed microbenchmarks, not packet-throughput measurements or a universal speedup guarantee. The empty case's +2.4% timing shift has overlapping ranges.

At 512 handles, JVM allocation falls from 95,568 to 83,280 bytes/call (12.9%). Handle timings are inconclusive: the single-handle median is +3.1%, the 512-handle median is −1.0%, and ranges overlap. Nested-reference allocation/timing shifts include JIT variation; do not attribute a general throughput improvement to their −6.4% median. The unchanged ordinary array control shifts −2.1% with overlapping ranges. All raw samples, per-process scores, CPU time, GC counts and commands are preserved in [samples.csv](samples.csv) and [results.json](results.json).

## Why the column prototype was replaced

The first implementation used one growing primitive/reference array per wire field. It avoided per-row holders and boxes, but growth copied every field's storage, added multiple array allocations for small wide results, and produced spikes just above powers of two. The final holders reuse shallow `ArrayList` growth and need no custom column-growth algorithm. Each comparison below has its own paired baseline JVM runs; do not compare latency columns across separate runs as a controlled head-to-head experiment.

| Case | Rows | Before µs [min–max] | After µs [min–max] | Time change | JVM bytes/call before → after |
|---|---:|---:|---:|---:|---:|
| `control` | 1 | 0.013 [0.013–0.013] | 0.013 [0.013–0.013] | +0.38% | 0.0 → 0.0 |
| `fold_flat` | 1 | 3.498 [3.212–7.417] | 3.256 [3.181–3.798] | -6.92% | 608.0 → 528.0 |
| `fold_flat` | 512 | 1027.394 [1016.652–1064.665] | 1074.447 [998.590–1184.699] | +4.58% | 95,568.0 → 72,600.0 |
| `fold_nested` | 64 | 393.529 [387.272–402.145] | 403.825 [396.280–422.462] | +2.62% | 54,961.2 → 53,122.7 |
| `primitive_array` | 512 | 5967.488 [5848.748–6099.881] | 5847.412 [5832.859–6158.568] | -2.01% | 168,016.0 → 168,016.0 |
| `primitive_fold` | 0 | 0.855 [0.842–0.877] | 0.865 [0.853–0.873] | +1.21% | 80.0 → 112.0 |
| `primitive_fold` | 1 | 1.244 [1.236–1.260] | 1.364 [1.230–1.596] | +9.60% | 400.0 → 328.0 |
| `primitive_fold` | 64 | 16.739 [16.165–17.063] | 16.408 [15.597–18.779] | -1.98% | 17,896.0 → 10,672.0 |
| `primitive_fold` | 65 | 17.121 [16.869–18.061] | 18.570 [17.324–19.133] | +8.46% | 18,160.0 → 19,016.0 |
| `primitive_fold` | 512 | 120.239 [116.449–128.735] | 117.243 [114.890–119.749] | -2.49% | 142,008.0 → 79,176.0 |
| `primitive_fold` | 513 | 126.706 [117.432–170.933] | 126.946 [124.345–143.545] | +0.19% | 142,272.0 → 144,864.0 |

The prototype's 64/512-record allocation reductions (40.4%/44.2%) came with 4.7%/1.8% allocation increases at 65/513 rows and a 32-byte increase for empty wide captures. Its median latency regressed 9.6% at one record and 8.5% at 65. These results motivated the final typed-holder design. [columns-samples.csv](columns-samples.csv) and [columns-results.json](columns-results.json) retain the complete prototype comparison; the prototype code is available at its recorded commit.

## A/A timing control

The same final library and JAR were run on both sides with four fresh JVM processes per side. The labels represent alternating runs, not different implementations.

| Case | Rows | A median µs [min–max] | A median µs [min–max] | Apparent time change |
|---|---:|---:|---:|---:|
| `fold_flat` | 512 | 1092.322 [1022.141–1272.666] | 1022.989 [985.729–1082.355] | -6.35% |
| `primitive_fold` | 512 | 114.071 [108.727–117.198] | 107.902 [106.502–124.133] | -5.41% |

These controls quantify process/JIT timing variation. Their −5.4%/−6.3% median shifts reinforce treating small latency changes, including the single-record and handle cases, as inconclusive; they do not support a universal speedup claim. Allocation accounting, all batches and per-process scores are preserved in [aa-samples.csv](aa-samples.csv) and [aa-results.json](aa-results.json).

Use the final build on both sides of `measure.py`, selecting `primitive_fold:512` and `fold_flat:512`, to reproduce this control.

## Verification

The final design passes all 162 `prebindgen-jni` unit tests and six doc tests, targeted Clippy with warnings denied, formatting, all 270 generated example snapshots, and the full actual C/Kotlin boundary suite. Workspace tests and Clippy also passed during development; CI runs the full workspace on the final revision. C used ASan/UBSan and Kotlin used `-Xcheck:jni`.

Boundary coverage includes all primitive wire kinds, strings/bytes, wide integers, options, nested/bulk rows, constructor/fold failure, native projection failure after an accepted prefix, native collector rejection with exact exception preservation, handle close/take and unread-owner cleanup, size/growth boundaries, reentry and concurrent calls. [diagnostic.json](diagnostic.json) records a separate twelve-JVM `-Xcheck:jni` benchmark validation. Diagnostic timings are excluded from performance conclusions. [review.md](review.md) records Claude Opus 5.5's final-design review; [columns-review.md](columns-review.md) preserves the superseded prototype reviews.

## Reproduce

Use clean committed checkouts at the before and final revisions above, or rebuild #824 at the recorded equivalent revision for a fresh baseline. Run each checkout's own `prepare.py` so the fixture provenance refers to the intended checkout. The script snapshots each native library before reusing the build cache.

```sh
export CARGO_BUILD_JOBS=1
export KOTLINC=/path/to/kotlinc
export JAVA_OPTS='-Xmx768m -XX:ActiveProcessorCount=2 -XX:-UsePerfData'
python3 examples/bench-bulk/jni-output-results/prepare.py \
  --checkout . --output /tmp/rows-before --target /tmp/rows-cache
# In the final checkout:
python3 examples/bench-bulk/jni-output-results/prepare.py \
  --checkout . --output /tmp/rows-after --target /tmp/rows-cache
"$KOTLINC" -cp /tmp/rows-before/bench.jar \
  -Xfriend-paths=/tmp/rows-before/bench.jar \
  examples/bench-bulk/jni-output-results/Driver.kt -d /tmp/rows-driver.jar
taskset -c 0,1 python3 examples/bench-bulk/jni-output-results/measure.py \
  --before /tmp/rows-before --after /tmp/rows-after \
  --driver /tmp/rows-driver.jar --output /tmp/rows-timing --forks 4 \
  --cell control:1 --cell primitive_array:512 \
  --cell primitive_fold:0 --cell primitive_fold:1 \
  --cell primitive_fold:64 --cell primitive_fold:65 \
  --cell primitive_fold:512 --cell primitive_fold:513 \
  --cell fold_flat:1 --cell fold_flat:512 --cell fold_nested:64
```

Use `--check-jni --forks 1` for diagnostic runs. Rebuild at the column-prototype commit to reproduce that comparison. Keep build settings identical and use a new output directory per invocation. Native libraries and JARs are not committed.
