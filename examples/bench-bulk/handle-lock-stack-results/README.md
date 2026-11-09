# Kotlin handle-lock stack usage

Large JNI inputs and callback results hold one JVM monitor per native handle.
Previously the helper entered one monitor per recursive call. A cold JVM could
therefore overflow its stack before an 8,192-handle call reached Rust.

The runtime now enters four consecutive monitors per recursive call, sharing
this helper with the sorted-list path. Monitor identity, acquisition order,
reentry, close protection, Kotlin alias preflight and Cleaner reachability
fences are unchanged. There are no additional wrappers, payload copies or heap
allocations. Calls using the fixed zero/one/two/three-handle overloads are
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
close. The regular boundary runner launches this test in a separate JVM.

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
