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
