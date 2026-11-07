# JNI handle-input benchmark

Compare two generator checkouts using the same source library, declarations and
Kotlin caller. Both variants compile their own generated Rust and Kotlin without
patching either output. Builds and raw logs stay outside Git.

```sh
python3 examples/bench-handle-inputs/run.py \
  --baseline /path/to/base-checkout --candidate "$PWD" \
  --output /tmp/handle-input-comparison \
  --java /path/to/jdk/bin/java --kotlinc /path/to/kotlinc
```

Requires Linux `taskset`, a cached offline Cargo dependency set and Kotlin/JDK.
The runner uses four fresh JVMs for each variant and input order, alternating
variant order, one CPU, fixed heaps, 20,000 warmup calls per case at size 8,
five warmup batches, and seven measured batches of approximately 12 ms. Each
result is the median of the per-JVM batch medians. Speed ratios use paired
per-JVM results. JVM allocation counters cover the whole call. Checksums and
native live-owner counts are verified; an error aborts the run.

Sizes 1, 128, 8,192 and 32,768 cover the single-element control and larger arrays.
Inputs use creation order or a fixed shuffle. Four-type arrays are allocated
round-robin across types so the allocator does not give each type its own address
run. The mixed cases contain that many handles in total, distributed over four
arrays. Nested inputs contain singleton inner lists. The owned roundtrip measures
input consumption plus bulk output reconstruction, preserving and reusing the
returned owners. A scalar JNI call is the no-handle control.

The enlarged JVM stack accommodates the existing recursive monitor helper at
32,768 handles. The benchmark does not change that locking implementation.
