# Opt-in Kotlin Cleaner costs

This fixture compares explicitly managed handles with `.gc_managed()` handles.
Both aliases contain the same Rust value, run the same destructor, and expose
matching JNI signatures. The difference is the generated Kotlin ownership policy.

```sh
KOTLINC=/path/to/kotlinc python3 examples/bench-gc-handles/run.py
```

The script builds the release covertest native library and compiles its generated
Kotlin with the benchmark driver. Each case runs in six fresh JVMs, with a
deterministic shuffled case order. Ten warmup batches precede ten measured batches.
Read batches contain 500,000 calls; creation/close batches contain 20,000 calls.
The report takes the median of six process scores, each computed from total batch
time divided by total calls. Identical plain cases run under separate labels as
A/A controls. `--wait-for-file PATH` can keep timing from overlapping other builds.

- `plain-read` / `gc-read`: repeatedly borrow one existing wrapper. Includes the
  generated Kotlin wrapper, handle locking, JNI entry, Rust read and result
  delivery. Managed calls also retain the wrapper through a JVM reachability fence.
- `plain-close` / `gc-close`: create a native value and wrapper, then explicitly
  close it. Includes allocation, Cleaner registration/cancellation where enabled,
  both JNI calls and native destruction. This does not measure asynchronous GC
  cleanup latency.

JVM allocation comes from `ThreadMXBean` on the calling thread. It excludes any
Cleaner-thread or native allocation. Timings are binding microbenchmarks, not
packet throughput. A shared, memory-constrained host can introduce timing noise;
the JSON retains every batch and process range. Per-run console logs are not
committed.

## Measurements

Measured implementation: `7f6b6d995bf0f800b06ee9fdfee77fad1be6a377`, with a
clean source tree. Host: Linux x86-64, Temurin OpenJDK 11.0.29. The prepared
benchmark's generated Rust and Kotlin matched the current generated files
byte-for-byte. The JSON records artifact hashes, process ranges and all samples.
Only one measurement controller ran, after builds and tests finished.

| Operation | Median ns/call | Calling-thread JVM bytes/call |
|---|---:|---:|
| Plain borrowed read | 21.86 | 0 |
| Managed borrowed read | 21.43 | 0 |
| Plain create + close | 67.73 | 32 |
| Managed create + close | 197.26 | 136 |

The identical-code plain controls measured 21.44 ns/read and 69.57 ns/create-close.
The read difference is comparable to the A/A variation; this run establishes no
read-call speedup or slowdown. Opting into Cleaner registration adds 104 calling-
thread JVM bytes and approximately 130 ns to creation plus explicit close in this
fixture (2.91 times the plain lifecycle latency). These are the optional lifecycle
costs; reused wrappers allocate no JVM objects in either read path.

To reuse an existing matching fixture build without recompilation:

```sh
python3 examples/bench-gc-handles/run.py --prepared target
```

The prepared mode assumes the JAR and release library came from the same fixture
revision; rebuild after changing bindings. The measured JAR’s generated Rust, generated Kotlin and `Bench.kt` code match
the recorded implementation; handwritten lifecycle tests are not executed by the
benchmark.

Claude Opus 5.5 reviewed the ownership implementation through Copilot. Review
follow-ups added targeted reachability fences and a diagnostic for borrowed
managed-handle callback-return constructors. Existing single-handle construction
rollback and `Send` requirements were verified; a final review reported no
remaining blockers. Review console logs stay local.
