# Dynamic input alias-history comparison

The validator checks numeric handle addresses for equality and remembers whether each access is shared or exclusive.

## Scope after Kotlin preflight (#887)

Default generated JNI wrappers now perform alias validation together with Kotlin
handle locking. Those calls do not instantiate the native `DynamicInputs`
history, including ordinary arguments and callback results covered by that
preparation. The JNI timings near the end of this document predate #887 and no
longer describe the default JNI path.

This runtime change applies to C with alias checks enabled. JniGen now always
uses Kotlin preflight and rejects custom conversions with incomplete handle
metadata. The former native-checked JNI mode has been removed; its measurements
below are historical. Its intended case is a history built incrementally by nested
collections or successive arguments. One first batch keeps sorted storage;
shared-only and fixed sets use other runtime implementations. C's check opt-out
also bypasses this history.

[The remaining-path report](remaining-paths.md) measures generated C calls,
native-checked JNI and default-JNI/checks-disabled controls against #824 **after**
#887. It separates whole-call timing from native allocation measurements and
retains slower controls and identical-library comparisons.

[The threshold investigation](thresholds.md) compares configurable limits using
complete C calls. The current implementation preserves disjoint ascending append
and retains 128 as the default; C supports project/local overrides. This fixes the ascending regression documented in the
earlier remaining-path study. The tables below record the earlier algorithm
comparison before that append fix, rather than the new threshold sweep.

The decision is a sorted-vector/integer-hash hybrid. The first batch retains its
sorted vector and allocation, including large batches. Disjoint ascending
additions also retain the vector. Other incremental additions beyond the
configured limit may promote to an exact `HashMap`. Repeated shared addresses
found by individual `access` do not trigger promotion. Fixed-size and shared-only
calls retain their existing implementations.

This favors predictable incremental scaling over the growing Bloom prototype's better unique-input speed and lower memory use. The growing filter is competitive on unique addresses, but confirmed shared repetitions still require exact searches through its history. The hybrid removes that linear fallback. This is a comparison of the supplied implementations, not proof that every possible Bloom-based design loses. In particular, separate shared/exclusive filters or specialized access to cached wire data could change the tradeoff, and are not implemented here.

The map reuses the runtime's existing SplitMix64 integer mixer and adds no dependency. Rust's standard map already uses SwissTable-style short hash fingerprints and exact key comparison, so it includes a compact filtering stage internally. See the [Rust HashMap documentation](https://doc.rust-lang.org/std/collections/struct.HashMap.html) and [Swiss tables design](https://abseil.io/about/design/swisstables). The hash fingerprints never decide alias validity without comparing full addresses.

## Validator timings

Microseconds per complete validator lifetime, including construction/destruction, group-buffer allocation and identity checks. `singletons` means N one-address groups. Addresses are synthetic aligned identities, never dereferenced. These are validator-only measurements, not JNI call or packet-throughput measurements.

| Addresses / grouping / order / use | Sorted array | Linear array | Growing Bloom | Default hash | Integer hash | B-tree | Selected hybrid |
|---|---:|---:|---:|---:|---:|---:|---:|
| 16/singletons/shuffled/unique | 0.73 | 0.76 | 0.86 | 1.09 | 0.92 | 0.69 | 0.69 |
| 64/singletons/shuffled/unique | 2.95 | 4.02 | 2.96 | 4.11 | 3.23 | 2.83 | 3.03 |
| 128/singletons/shuffled/unique | 6.47 | 10.16 | 5.68 | 7.99 | 6.13 | 6.37 | 6.72 |
| 256/singletons/shuffled/unique | 14.93 | 34.23 | 11.53 | 15.94 | 12.48 | 13.50 | 14.13 |
| 512/singletons/shuffled/unique | 40.91 | 109.85 | 23.64 | 32.69 | 24.44 | 29.23 | 26.36 |
| 2048/singletons/shuffled/unique | 304.45 | 1,590.01 | 94.36 | 129.07 | 95.49 | 210.22 | 98.97 |
| 8192/singletons/shuffled/unique | 4,915.72 | 24,643.37 | 424.98 | 807.07 | 650.71 | 966.42 | 702.20 |

Other shapes expose why no single implementation wins everywhere:

| Addresses / grouping / order / use | Sorted array | Linear array | Growing Bloom | Default hash | Integer hash | B-tree | Selected hybrid |
|---|---:|---:|---:|---:|---:|---:|---:|
| 8192/batch/ascending/unique | 34.06 | 44.89 | 32.41 | 155.86 | 95.62 | 636.91 | 30.40 |
| 8192/batch/shuffled/unique | 140.28 | 159.02 | 138.64 | 154.15 | 94.08 | 734.98 | 136.76 |
| 8192/singletons/descending/unique | 9,229.99 | 24,021.04 | 422.35 | 765.88 | 656.52 | 445.96 | 639.09 |
| 8192/history/repeated/shared | 509.29 | 24,425.61 | 13,005.66 | 502.79 | 367.04 | 1,250.05 | 378.39 |

The large-history shared-repeat case first accepts one exclusive prefix and 8,192 distinct shared addresses, then repeats all shared addresses as singleton groups in reverse order. The growing Bloom prototype performs 33,566,720 exact entry comparisons here despite zero filter false positives: these are true membership hits. A small repeated set is tested separately. Mixed direct/grouped calls, cross-group rejection and scattered aligned address values are also included.

Both fixed Bloom filters use three bit positions per address. The ordinary filters have 512 or 8,192 bits; the blocked variant puts three bits in one 64-bit word inside an 8,192-bit filter. All positive filter answers are checked exactly. The growing filter starts lazily after the first batch, allocates at least 512 bits and then roughly 16–32 bits per expected address, rebuilds below approximately 12 bits/address, and retains an append-only exact history. Unlike the fixed filters it adopts the first batch's allocation without building a filter. On 8,192 shuffled singleton inputs, the fixed 8,192-bit filter has 3,611 false positives and scans 20,446,987 exact entries; the growing filter has 31 false positives and scans 127,925 entries. These are measurements of these implementations, not ideal Bloom error estimates. [Parquet's filter description](https://parquet.apache.org/docs/file-format/bloomfilter/) explains the underlying space/selectivity tradeoff.

## Memory tradeoff

The selected index uses more memory after promotion. This is deliberate and limited to dynamic incremental histories. One-batch allocation reuse, empty-call laziness, fixed input storage and shared-only omission are preserved. Per-group buffers remain in every candidate.

| Case | Candidate | Allocations / reallocations | Peak live requested heap KiB | Validator state bytes |
|---|---|---:|---:|---:|
| 512/singletons/shuffled/unique | sorted | 512 / 8 | 8.02 | 24 |
| 512/singletons/shuffled/unique | bloom8192 | 513 / 7 | 8.02 | 1048 |
| 512/singletons/shuffled/unique | bloom_grow | 517 / 8 | 9.52 | 48 |
| 512/singletons/shuffled/unique | hash_fast | 521 / 0 | 25.55 | 40 |
| 512/singletons/shuffled/unique | btree | 580 / 0 | 13.52 | 32 |
| 512/singletons/shuffled/unique | selected | 515 / 6 | 25.55 | 40 |
| 8192/singletons/shuffled/unique | sorted | 8192 / 12 | 128.02 | 24 |
| 8192/singletons/shuffled/unique | bloom8192 | 8193 / 11 | 128.02 | 1048 |
| 8192/singletons/shuffled/unique | bloom_grow | 8201 / 12 | 152.02 | 48 |
| 8192/singletons/shuffled/unique | hash_fast | 8205 / 0 | 408.05 | 40 |
| 8192/singletons/shuffled/unique | btree | 9276 / 0 | 214.98 | 32 |
| 8192/singletons/shuffled/unique | selected | 8199 / 6 | 408.05 | 40 |
| 8192/batch/shuffled/unique | sorted | 1 / 0 | 128.00 | 24 |
| 8192/batch/shuffled/unique | bloom8192 | 2 / 11 | 256.00 | 1048 |
| 8192/batch/shuffled/unique | bloom_grow | 1 / 0 | 128.00 | 48 |
| 8192/batch/shuffled/unique | hash_fast | 2 / 0 | 400.02 | 40 |
| 8192/batch/shuffled/unique | btree | 1085 / 0 | 342.97 | 32 |
| 8192/batch/shuffled/unique | selected | 1 / 0 | 128.00 | 40 |

The allocator counter records allocations, reallocations, cumulative requested bytes and peak live requested bytes. It excludes allocator metadata, fragmentation, stack frames and the allocator's transient old/new buffers inside a moving `realloc`; it is not RSS. The separate validator-state size includes inline filter arrays but excludes the common group object (32 bytes on this host). All three counting observations per cell agree. Timing binaries contain neither allocator counters nor filter probe counters.

## Protocol and correctness

The main comparison snapshot is commit `41ad78e99`, based on merged #885 (`45933039a`). [comparison.csv](comparison.csv) records every candidate/case, four fork medians, memory counters and Bloom query/scan counters; [provenance.json](provenance.json) records source/binary hashes, compiler, CPU/libc and sample hash. Raw batch samples/logs remain outside Git. The early screening runs were exploratory and are not used in these tables.

Each candidate/case runs in its own pinned process on CPU 0, with two warmup batches and seven measured batches calibrated to approximately 3 ms. Four forks randomize candidate order, reversing each paired order. Fixtures are prepared outside timing; validator/group work and destruction remain inside. Four untimed probe calls select the iteration count. The machine has two available CPUs; governor/turbo, interrupts and sibling activity are not isolated. Differences of a few percent are inconclusive. The CSV retains fork spread, including slower cases. Synthetic allocation patterns do not reproduce the JVM heap.

Correctness checks compare all candidates with an exact model, including first-batch reuse, conflicts within and across groups, shared duplicates, direct accesses and continued use after rejected groups. The production runtime additionally tests long mixed histories across promotion, generation wrap, 127/128/129 boundaries and aligned hash distribution. Its map marks newly inserted entries with a group generation; a failed group removes only those entries, without an undo vector on success. Full-table rollback is confined to errors. The current group buffer and final Rust inputs retain their existing ownership roles.

The linear and Bloom candidates deduplicate shared addresses. Default/integer hash maps and the B-tree insert incrementally, using generation tags for error rollback instead of sorting a group. The hybrid prototypes use the baseline's exact sorted fast paths with thresholds 64 and 128. The selected column imports actual production runtime code rather than copying it. This measures each algorithm's full strategy, not identical instruction sequences with interchangeable containers.

Reproduce from the comparison snapshot, preserving the baseline input module outside the checkout:

```sh
git show 45933039a:prebindgen-runtime/src/input.rs > /tmp/input-alias-baseline.rs
python3 examples/bench-input-aliases/run.py --runtime /tmp/input-alias-baseline.rs --output /tmp/input-alias-comparison --quick --forks 4 --batches 7 --target-ms 3
```

The full mode also includes individual `access()` streams, groups of four and additional rejection cases. The current runner allows `--candidates selected,sorted` for focused repeats. Sources remain experiment-local; only the selected implementation is in `prebindgen-runtime`.

## Final-source confirmation

After source review cleanup, a separate focused repeat imports the final runtime at `4e4482ef5`. It uses four isolated-process forks, nine measured batches targeting 5 ms, and the same baseline. [confirmation.csv](confirmation.csv) and [confirmation-provenance.json](confirmation-provenance.json) retain all 126 cells, memory counts, fork spread and provenance.

| Case | Baseline µs | Final runtime µs |
|---|---:|---:|
| 16/singletons/shuffled/unique | 0.610 | 0.635 |
| 128/singletons/shuffled/unique | 6.115 | 6.365 |
| 512/singletons/shuffled/unique | 36.814 | 24.305 |
| 8192/singletons/shuffled/unique | 4,521.906 | 664.509 |
| 8192/history/repeated/shared | 480.391 | 323.788 |
| 128/mixed/shuffled/unique | 6.994 | 7.765 |
| 128/history/late/conflict | 0.571 | 1.804 |

The selected threshold preserves the vector through 128 distinct individual entries and avoids building another index for one first batch. Thresholds 64 and 128 are broadly competitive; 128 is a conservative choice for small histories, not a claim of a universal optimum. Transition costs remain visible: the 128-address mixed case adds 16 direct prefix accesses, crosses the threshold near the end, and is 11% slower in the final repeat. A late conflict after a 128-address first batch costs 0.571 → 1.804 µs because promotion happens before rejection. Error-path throughput is not the optimization goal. Small shuffled singleton cases remain within a few percent, but that statement does not apply to every small input or failure shape.

## Historical generated JNI calls before #887

Fresh frozen native libraries use the same baseline Kotlin JAR. The baseline library was built at `6e00520a7`; all runtime/generator/benchmark fixture sources match merged #824 at `45933039a`, verified by `git diff`. Candidate source is `4e4482ef5`. Native allocator instrumentation is verified on load, enabled only in separate counting libraries, and disabled for all latency runs. Builds, counting and timing run sequentially.

These measurements include Kotlin packing/locking, JNI, Rust input validation/construction, source work, output encoding and Kotlin reconstruction. The ordinary argument consumes lists passed by the wrapper; callback cases first ask Kotlin for lists that Rust decodes as callback results. They are complete roundtrips, not isolated input-decoder or packet-throughput benchmarks.

| Run | Transport | Case | Handles | µs/call before → after | Paired speedup |
|---|---|---|---:|---:|---:|
| latency | bulk | callback_many_nested_owned_handles | 512 | 186.21 → 171.41 | 1.15× |
| latency | bulk | callback_nested_owned_handles | 512 | 61.62 → 62.24 | 1.00× |
| latency | bulk | nested_many_owned_handles_roundtrip | 512 | 174.80 → 172.87 | 1.02× |
| latency | bulk | nested_owned_handles_roundtrip | 512 | 65.93 → 67.57 | 1.01× |
| latency | bulk | owned_handle_records_roundtrip | 512 | 73.45 → 76.08 | 0.97× |
| latency | bulk | owned_handles_roundtrip | 512 | 64.92 → 65.42 | 0.99× |
| latency | plain | callback_many_nested_owned_handles | 512 | 1,420.80 → 1,341.15 | 1.05× |
| latency | plain | callback_nested_owned_handles | 512 | 532.18 → 525.79 | 1.02× |
| latency | plain | nested_many_owned_handles_roundtrip | 512 | 1,081.32 → 1,076.78 | 1.00× |
| latency | plain | nested_owned_handles_roundtrip | 512 | 365.43 → 354.43 | 1.05× |
| large | bulk | callback_many_nested_owned_handles | 8192 | 4,743.61 → 2,816.80 | 1.69× |
| large | bulk | callback_nested_owned_handles | 8192 | 995.00 → 991.56 | 0.95× |
| large | bulk | nested_many_owned_handles_roundtrip | 8192 | 4,769.27 → 2,599.63 | 1.77× |
| large | bulk | nested_owned_handles_roundtrip | 8192 | 1,087.24 → 944.31 | 1.12× |
| large | bulk | owned_handle_records_roundtrip | 8192 | 1,169.27 → 1,192.48 | 0.98× |
| large | bulk | owned_handles_roundtrip | 8192 | 1,122.75 → 1,028.17 | 1.06× |

The default-stack run uses six balanced JVM pairs. The large run uses four balanced pairs; both use 10 warmup batches and 11 measured batches targeting 30 ms. Times are medians of fork medians; speedup is the reciprocal of the median paired after/before ratio, so it need not equal the ratio of the two displayed medians.

The first 8,192-handle attempt failed before Rust with `StackOverflowError` in existing recursive Kotlin lock acquisition. The successful large comparison sets **`-Xss16m` on both versions**, with the same heap/GC settings as the main run. This PR does not change Kotlin locking or fix that existing default-stack limit. Large-array measurements therefore require the documented stack setting.

Same-library A/A runs retain the entire procedure. At 512 handles, paired time drift is 2.4% for ordinary singleton-list arguments and 5.0% for callback singleton-list results in the main control run. At 8,192 handles the large-stack A/A drift reaches 4.1%. Small effects and slower controls remain in [jni-latency.csv](jni-latency.csv).

Counting uses two JVMs per cell, 1,000 warmup calls and ten measured calls each; all twenty observations per cell agree. At 512 singleton lists, native allocations rise from 1,539 to 1,542 for arguments and from 1,542 to 1,545 for callback results; reallocations fall by two in each. Cumulative requested native bytes rise by 18,224 per call. Flat and single-inner-list allocation counts remain unchanged. This is the cost of replacing sorted-history movement with the incremental hash index, separate from the prepared-row allocations already removed by #885.

[jni-allocations.csv](jni-allocations.csv) retains every count; [jni-provenance.json](jni-provenance.json) records verified instrumentation, binary/JAR/source-fixture hashes, JVM flags and sample hashes. Raw observations/logs stay outside Git. Reproduce the large comparison with [jni.py](jni.py) and the frozen libraries described above.
