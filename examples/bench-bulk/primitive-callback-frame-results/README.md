# JNI frames for primitive collection callback rows

Collection-return callbacks previously pushed and popped a JNI local-reference frame for every output row. The generator now omits both operations when all encoded fields are JVM primitives, including raw native handles and enum tags. Rows containing a JVM reference still have frames. A Rust `i128` or a bulk-packed `i64` has a byte-array reference on this boundary and retains its frame.

The comparison uses the existing output-ownership fixture and driver. No benchmark workload or Kotlin wrapper changed. All 3,534 JVM class files are byte-identical, verified by [class-identity.json](class-identity.json). The native runtime has no new condition or configuration option; the generator makes the choice.

## Revisions and method

- Before: `0a0cd1fbe34e6bbda6dc4de64065d93a0844dbc7`, whose generator and benchmark sources are identical to #824 at `3014203c2aef28a575d8f8d27b873f7a8b2eb1c0`. `git diff --exit-code 0a0cd1fbe 3014203c2 -- 'prebindgen*' examples/bench-bulk` passed.
- After: `7c3f8988d`, containing the runtime change and initial tests. Later changes strengthen test assertions and add benchmark snapshots/reports; they do not change generator/runtime code or benchmark inputs.
- Release native builds use the same Rust compiler, dependency lock, profile, fixture and compiler settings, checked by the runner. Build metadata and artifact hashes are in [results.json](results.json).
- Four fresh JVMs per cell and version, alternating old/new order; both CPUs enabled with `taskset -c 0,1`, fixed 256 MiB heap, G1 and two active processors. Each JVM warms for at least two seconds and waits for a 500 ms pause in compilation, then records twelve batches. No builds or tests ran during timing. These measurements use Kotlin 2.4.21 and OpenJDK 11.0.29 on this Linux x86-64 host; complete toolchain metadata is recorded.
- Each fork score is total batch elapsed time divided by total calls. Tables show the median and full range of those four scores, in microseconds per complete function call. Lower is better; change is `(after / before - 1) × 100`. These are microbenchmarks, not packet-throughput measurements.

## Main comparison

`primitive_fold` returns records of eight `i64` fields and folds them after the native call. `fold_flat` consumes a list of native handles and returns those handles through the collection callback, including the existing input conversion and locking. `fold_nested` returns one-handle inner lists, so its callback argument is a JVM array reference and its frames remain. `primitive_array` uses ordinary object-array output, and `control` is an unchanged scalar native call. Row counts are collection lengths, not the number of fields per record.

| Case | Rows | Before µs [min–max] | After µs [min–max] | Time change |
|---|---:|---:|---:|---:|
| `control` | 1 | 0.013 [0.013–0.014] | 0.013 [0.013–0.014] | +1.18% |
| `fold_flat` | 1 | 3.328 [3.302–3.458] | 3.292 [3.189–3.554] | -1.06% |
| `fold_flat` | 512 | 1058.710 [1021.474–1096.202] | 1022.439 [985.826–1269.478] | -3.43% |
| `fold_nested` | 64 | 386.821 [377.879–397.430] | 413.450 [400.111–425.060] | +6.88% |
| `primitive_array` | 512 | 5853.635 [5579.193–8674.900] | 5817.804 [5768.859–6100.837] | -0.61% |
| `primitive_fold` | 0 | 0.879 [0.859–0.888] | 0.853 [0.844–0.921] | -2.97% |
| `primitive_fold` | 1 | 1.368 [1.323–1.508] | 1.313 [1.226–1.509] | -4.04% |
| `primitive_fold` | 64 | 20.265 [19.982–20.704] | 16.968 [15.983–17.478] | -16.27% |
| `primitive_fold` | 512 | 148.749 [144.156–164.266] | 123.370 [118.452–154.553] | -17.06% |

The main result is lower median primitive-record callback latency: 16.3% at 64 rows and 17.1% at 512 rows. The 512-row ranges overlap and one of four forks was slower after the change. Handle results are inconclusive: the 512-row median decreased 3.4%, but ranges overlap substantially. Do not infer a universal speedup from these medians.

JVM allocation is effectively unchanged for the primitive and flat-handle cases: 17,896 bytes/call at 64 primitive rows, 142,008 at 512 primitive rows, and 95,568 at 512 handles. The optimization removes native JNI frame operations, not Kotlin row arrays or boxing. Tiny fractional differences in batch accounting and nested-row allocation are retained in the raw data; no native allocation reduction is claimed.

## Timing controls and confirmation

The main comparison's unchanged nested-reference control was 6.88% slower, despite retaining the same generated Rust. A focused repetition used the same two libraries and four fresh JVMs per version:

| Case | Rows | Before µs [min–max] | After µs [min–max] | Time change |
|---|---:|---:|---:|---:|
| `fold_nested` | 64 | 390.915 [378.567–400.832] | 393.054 [383.439–432.238] | +0.55% |

That control regression did not repeat: the focused comparison is +0.55% with overlapping ranges. The original result remains in the report; the focused run does not replace it.

An A/A comparison used the exact same after library and JAR on both sides, again with four fresh JVMs per side. “Before” and “after” below are only alternating labels:

| Case | Rows | A µs [min–max] | A µs [min–max] | Apparent change |
|---|---:|---:|---:|---:|
| `fold_nested` | 64 | 401.055 [387.334–413.221] | 394.324 [392.215–434.948] | -1.68% |
| `primitive_fold` | 512 | 133.367 [122.974–239.763] | 125.805 [117.292–148.703] | -5.67% |

A/A median shifts of −5.67% for primitives and −1.68% for nested references show the sensitivity of this host, including a large primitive-fork outlier. The primitive comparison's 16–17% reductions are promising; the small handle changes are not evidence of acceleration. [Raw main samples](samples.csv), [A/A samples](aa-samples.csv) and [confirmation samples](reference-confirmation.csv) preserve every batch. Their JSON companions contain per-fork scores, CPU time, JVM allocation, GC counts, commands, checksums and build metadata.

## Correctness and review

The covertest uses a test-only JNI function-table probe. It identifies the exact generated collector interface, counts frames enclosing collector calls and checks that all frames are popped. Primitive, handle and optional-primitive rows have zero row frames at sizes 0, 1 and 4,096; strings, byte arrays and bulk-packed scalar rows retain one frame pair per row. Rejecting the second handle row after an accepted prefix preserves the exact exception object and destroys all three inputs exactly once without running user fold code. The hook is confined to the test crate and absent from the benchmark libraries.

Workspace tests, Clippy with warnings denied, formatting, all 270 generated example snapshots and the actual C/Kotlin boundary suite passed. C callers used ASan/UBSan; Kotlin used `-Xcheck:jni`. Final probe assertions and parsing checks were revalidated with focused tests, Clippy, snapshots and the actual covertest. [diagnostic.json](diagnostic.json) records a separate twelve-JVM `-Xcheck:jni` benchmark validation; its timings are diagnostic only and are excluded from the tables. [review.md](review.md) records both Claude Opus 5.5 review passes and addressed suggestions.

## Reproduce

Use clean committed checkouts at the revisions above. Run each checkout's own `prepare.py` so fixture provenance refers to the intended checkout. Keep these settings identical for both builds:

```sh
export CARGO_BUILD_JOBS=1
export KOTLINC=/path/to/kotlinc
export JAVA_OPTS='-Xmx768m -XX:ActiveProcessorCount=2 -XX:-UsePerfData'
python3 examples/bench-bulk/jni-output-results/prepare.py \
  --checkout . --output /tmp/frames-before --target /tmp/frames-build-cache
# In the after checkout:
python3 examples/bench-bulk/jni-output-results/prepare.py \
  --checkout . --output /tmp/frames-after --target /tmp/frames-build-cache
"$KOTLINC" -cp /tmp/frames-before/bench.jar \
  -Xfriend-paths=/tmp/frames-before/bench.jar \
  examples/bench-bulk/jni-output-results/Driver.kt -d /tmp/frames-driver.jar
taskset -c 0,1 python3 examples/bench-bulk/jni-output-results/measure.py \
  --before /tmp/frames-before --after /tmp/frames-after \
  --driver /tmp/frames-driver.jar --output /tmp/frames-timing --forks 4 \
  --cell control:1 --cell primitive_fold:0 --cell primitive_fold:1 \
  --cell primitive_fold:64 --cell primitive_fold:512 \
  --cell fold_flat:1 --cell fold_flat:512 \
  --cell fold_nested:64 --cell primitive_array:512
```

Use `--check-jni --forks 1` for diagnostic validation. For A/A, pass `/tmp/frames-after` on both sides and select `primitive_fold:512` and `fold_nested:64`. Select only `fold_nested:64` for the focused before/after confirmation. Use a new output directory per run. Build failures caused by concurrent compiler memory pressure must be resolved before timing; native libraries and JARs are not committed.
