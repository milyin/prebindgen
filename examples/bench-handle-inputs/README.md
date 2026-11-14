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

## Production measurements

Linux x86-64, OpenJDK 21, Kotlin 2.4.21, Rust 1.98 release builds. Four fresh JVMs
per variant/input order with the settings above. Baseline is #824 at
`45933039a858ad31f56f7e213d9dd6aa9d5cb2da`; candidate is
`b1c8f0f28` (the implementation commit). The later review fix only rejects missing
nested-group metadata at generation and leaves valid generated output unchanged.
These compare the production generator to #824, **without** the separate #886
native-validator optimization. No output source is patched for measurement.

Complete JNI calls at 8,192 array handles (plus the token where shown), in microseconds:

| Case | Creation order: before → after | Paired speedup | Shuffled: before → after | Paired speedup |
|---|---:|---:|---:|---:|
| One type, shared | 277.12 → 262.91 | 1.06× | 1553.16 → 1390.45 | 1.12× |
| One type, mutable | 316.65 → 273.05 | 1.18× | 1762.51 → 1400.60 | 1.26× |
| One type, singleton inner lists | 1424.34 → 788.90 | 1.78× | 6835.68 → 1927.49 | 3.54× |
| Four shared types + unrelated mutable token | 673.09 → 296.82 | 2.24× | 1871.42 → 1202.13 | 1.55× |
| Four mutable types | 661.17 → 307.41 | 2.14× | 1828.52 → 1211.99 | 1.50× |
| Owned input/output roundtrip | 979.52 → 788.86 | 1.24× | 1044.88 → 803.32 | 1.28× |

At 32,768 shuffled handles, mixed shared/mutable calls improve about 1.6×, while
singleton nested inputs improve 9.3× with no repeated native history
movement. One-type shared/mutable calls improve 1.15×/1.26×. The roundtrip improves
1.35×. Creation-order shared-only inputs at that size are effectively unchanged
(1.01×).

For 8,192 shuffled shared handles, whole-call JVM allocation falls from
2,586,560 to 274,240 bytes. Four shared types plus the token fall from 2,784,012
to 292,880 bytes. Primitive comparators remove boxed `Long` sorting keys; prepared
carriers supply type groups directly, and successful consumption receipts no
longer rescan aliases. Input snapshots, owner receipts and lock lists remain.

The one-element shared-list control is slightly slower in creation order:
0.211 → 0.220 µs (0.97× paired speedup); the shuffled repeat is faster. Single-type
carriers allocate 8 extra JVM bytes at size one, because group metadata adds fields
to their writer. This is a small-call tradeoff, not a universal speedup. The scalar
no-handle control allocates zero in both variants and measures about 0.02 µs;
its small timing differences are not attributable to handle validation.

Each cell reports separate variant medians. Paired speedups are the median of
four corresponding per-JVM ratios, so they need not equal the ratio of table
cells. [results.csv](results.csv) includes every size, ordering, latency and JVM
allocation result. Raw batch logs, hashes of generated libraries/JARs and exact
JVM commands remain in the external output directory. Owned roundtrips preserve
value order, but reboxing can change native address order between calls; their
shuffle describes the initial fixture only. Timing remains machine- and
workload-dependent.
