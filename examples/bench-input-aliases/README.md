# Dynamic input alias-history comparison

The decision is a sorted-vector/integer-hash hybrid. One first batch retains the existing sorted vector and reuses its allocation, including large batches. Small incremental histories retain the same sorted-vector algorithm. A subsequent group whose potential total exceeds 128 entries, or a new individual address beyond 128 stored entries, promotes history into an exact `HashMap`. Existing shared addresses found by `access` do not trigger promotion. Fixed-size and shared-only calls keep their existing implementations.

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
