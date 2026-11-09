# Kotlin handle-lock stack usage

Large JNI inputs and callback results hold one JVM monitor per native handle.
Previously the helper entered one monitor per recursive call. A cold JVM could
therefore overflow its stack before an 8,192-handle call reached Rust.

The runtime now enters four consecutive monitors per recursive call, sharing
this helper with the sorted-list path. Monitor identity, acquisition order,
reentry, close protection, Kotlin alias preflight and Cleaner reachability
fences are unchanged. There are no additional wrappers, payload copies or allocation
sites. Calls using the fixed zero/one/two/three-handle overloads are
unchanged.

This reduces call-frame overhead, but JVM monitor records still occupy stack
space. It does **not** provide an unlimited batch size. Limits depend on the JVM,
thread stack, compilation state and the work executed inside the locked body.
Attached native workers keep their native thread stack size; `-Xss` applies to
JVM-created threads.

## Choosing the group size

`groups.json` contains independent JVM observations for groups of 2/4/8/16/32
and the previous helper. Handles escape through a volatile reference, and the
body calls a method explicitly excluded from inlining. Each timing cell uses a
fresh JVM, 12 warmup batches, 11 measured batches, and six forks on Java 11
(four on Java 21). The table reports median nanoseconds per call across fork
medians. This isolates monitor acquisition; it is not a complete JNI call.
Timings use `-Xss16m` for both variants, so the previous helper can finish large
calls. Stack-limit checks separately use `-Xss1m` and a fresh JVM for each cell.

| Java 11 handles | Previous | Group 2 | Group 4 | Group 8 | Group 16 | Group 32 |
|---:|---:|---:|---:|---:|---:|---:|
| 8 | 26.7 | 27.1 | 26.9 | 48.8 | 29.3 | 29.7 |
| 64 | 213 | 175 | 161 | 327 | 318 | 321 |
| 512 | 1,662 | 1,533 | 1,268 | 2,616 | 2,371 | 2,350 |
| 8,192 | 33,993 | 31,812 | 26,562 | 45,289 | 41,844 | 41,503 |

Every helper measured zero steady-state JVM bytes per call. Four gives the best
latency among these groups on Java 11. Larger groups improve cold stack limits
further but regress latency, so the production helper uses four.

On Java 21, previous → group four: 64 handles 399 → 412 ns; 512 handles
5,036 → 4,149 ns; 8,192 handles 95,023 → 82,691 ns. These are observations on
this host, not a promised JVM-independent speedup.

Largest successful **tested grid point** with a 1 MiB stack (not an exact limit):

| Runtime/state | Previous | Group 2 | Group 4 | Group 8 | Group 16 | Group 32 |
|---|---:|---:|---:|---:|---:|---:|
| Java 11 cold | 4,096 | 8,192 | 8,192 | 16,384 | 16,384 | 16,384 |
| Java 11 warmed | 16,384 | 16,384 | 16,384 | 16,384 | 16,384 | 32,768 |
| Java 21 cold | 4,096 | — | 16,384 | — | — | — |
| Java 21 warmed | 8,192 | — | 16,384 | — | — | — |

The focused covertest first makes an actual JNI call borrowing 8,192 managed
handles in a cold `-Xss1m` JVM. It then consumes those owners, exercises 4,096
callback result rows with two owned handles per row on the calling thread and an
attached worker, checks constructor-failure cleanup and retry, and tests monitor
boundaries, partial acquisition, reentry, reversed-order threads and concurrent
close. The regular boundary runner launches this test in a separate JVM. The original
4,096-batch `callback_mixed_batches_many` benchmark also reproduces
`StackOverflowError` with the previous jar and passes with group four in fresh
`-Xss1m -Xcheck:jni` JVMs.

## Complete JNI calls

The six-fork Java 11 comparison changes only the Kotlin jar. Every generated
Rust/Kotlin source in the frozen native build matches the #824 base; both jars
use that exact native image. Settings: 10 warmup batches, 11 measured batches,
20 ms calibration target, 256 MiB heap and `-Xss16m` for both versions. A separate
four-fork identical-code control repeats the previous jar under both labels.
`results.json` retains fork medians, allocation observations and source/image
hashes. No timing run overlaps compilation or another benchmark.

Times below are medians of fork medians, in microseconds. Percentages are the
median of paired after/before ratios; they can differ from the ratio of the two
medians. Negative means faster. The control column uses identical code, so its
changes indicate run variation rather than this fix's effect.

| Path/case | Rows | Previous µs | Group 4 µs | Paired change | Identical-code change |
|---|---:|---:|---:|---:|---:|
| bulk `callback_batches_many` | 1 | 2.336 | 2.361 | -0.2% | +7.6% |
| bulk `callback_batches_many` | 512 | 198.323 | 194.568 | -1.3% | +2.7% |
| bulk `callback_batches_many` | 4,096 | 1642.056 | 1696.208 | +2.6% | +1.0% |
| bulk `callback_mixed_batches_many` | 1 | 2.965 | 2.985 | +2.0% | -0.3% |
| bulk `callback_mixed_batches_many` | 512 | 724.339 | 718.722 | +2.0% | +0.9% |
| bulk `callback_mixed_batches_many` | 4,096 | 5810.100 | 5786.304 | -1.6% | -3.1% |
| bulk `callback_owned_records` | 1 | 2.036 | 2.060 | +3.4% | +2.2% |
| bulk `callback_owned_records` | 512 | 58.510 | 61.596 | +3.9% | +0.7% |
| bulk `callback_owned_records` | 4,096 | 434.129 | 431.697 | -1.0% | -2.6% |
| bulk `control` | 1 | 0.014 | 0.014 | +0.6% | -3.0% |
| bulk `handles_input` | 1 | 0.270 | 0.383 | +1.2% | +18.1% |
| bulk `handles_input` | 512 | 18.214 | 12.154 | -8.8% | -4.7% |
| bulk `nested_many_owned_handles_roundtrip` | 1 | 1.021 | 1.020 | -1.5% | +2.5% |
| bulk `nested_many_owned_handles_roundtrip` | 512 | 119.286 | 116.549 | -4.6% | -6.9% |
| bulk `nested_many_owned_handles_roundtrip` | 4,096 | 928.247 | 891.278 | -6.4% | -0.4% |
| plain `callback_mixed_batches_many` | 1 | 2.886 | 2.887 | -0.5% | -5.5% |
| plain `callback_mixed_batches_many` | 4,096 | 5664.026 | 5599.358 | -1.5% | -2.8% |

These measurements do not establish a consistent complete-call speedup. They
include slower cases and variation in the identical-code control. The fix's
confirmed benefit is the larger supported cold-stack batch; the isolated helper
comparison supports choosing group four over larger, slower groups.

JVM allocation observations are mostly unchanged. Some large cases vary with
JVM compilation state, including identical-code runs; the JSON preserves all
fork medians rather than presenting those fluctuations as allocation savings.
The helper-only experiment measures zero steady-state bytes/call. Native
allocation behavior is unchanged because both versions execute the exact same
native image. Grouping adds no allocation sites.

For case interpretation: `callback_batches_many` returns batches with an outer
owned handle and a singleton inner handle/tag record (8,192 handles at 4,096
rows). `callback_mixed_batches_many` returns the same values through ordinary
outer records and packed inner lists.
`callback_owned_records` returns a flat list of handle/tag records.
`nested_many_owned_handles_roundtrip` is an ordinary Kotlin-to-Rust call with
singleton inner lists; `handles_input` borrows a handle list. `control` is the
unchanged primitive call. `bulk`/`plain` select the two fixture configurations.
Both batch functions return packed output to Kotlin; the mixed callback input
uses the same ordinary/packed combination in both modes.

Repeat the complete-call command with `--aa --forks 4` and a fresh output path to
obtain the identical-code control. Add `--target-ms 20` to reproduce this run's
calibration target. All other group selections are the same.

## Reproduction

Run from a clean, committed repository. Keep logs and jars outside Git. Do not
run compilers or other benchmarks during measurements.

```sh
KOTLINC=/path/to/kotlinc python3 examples/bench-bulk/handle-lock-stack-results/experiment.py \
  --output /tmp/monitor-groups
KOTLINC=/path/to/kotlinc python3 examples/bench-bulk/handle-lock-stack-results/experiment.py \
  --output /tmp/monitor-groups-java21 --java /path/to/java21 \
  --variants base,4 --sizes 1,8,64,512,8192 --forks 4
```

For complete JNI comparisons, first build the previous checkout using
`examples/bench-bulk/run.py --build-only`. `prepare.py` recompiles the committed
Kotlin benchmark sources against that frozen native image. It checks fixture
hashes, unchanged generated Rust, Kotlin compiler version and all Kotlin files
except the two changed `NativeHandle.kt` files. `compare.py --kotlin-only` uses
that exact native image for both jars. This avoids native code placement noise.
The build manifests explicitly distinguish the Kotlin revision from the reused
native build revision.

```sh
KOTLINC=/path/to/kotlinc python3 examples/bench-bulk/handle-lock-stack-results/prepare.py \
  --before /tmp/before --output /tmp/after-kotlin
python3 examples/bench-bulk/jni-callback-cleanup-results/compare.py \
  --before /tmp/before --after /tmp/after-kotlin --kotlin-only \
  --java-flag=-Xss16m --output /tmp/complete-calls \
  --group bulk:1,512,4096:callback_mixed_batches_many,callback_batches_many \
  --group plain:1,4096:callback_mixed_batches_many \
  --group bulk:1,512,4096:nested_many_owned_handles_roundtrip,callback_owned_records \
  --group bulk:1,512:handles_input --group bulk:1:control
```
