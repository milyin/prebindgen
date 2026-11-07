# Slowdown investigation

`decode_probe.py` extracts the actual generated staged and reread input decoders, then runs both against controlled native frames without JNI, Kotlin, packing, output conversion or timed result destruction. Each frame contains fresh live native boxes; pointer ranks are sorted, reversed or deterministically shuffled before timing. Each decoder invocation uses live handles that it consumes exactly once; the allocator can reuse addresses between invocations. Returned values and their checksum are checked. Allocation/packing occur outside the timer. Native timings include decoding's temporary/final vector allocation, alias checking and source-box consumption.

Six decoder variants distinguish staging from rereading, removal of only the second pointer check, removal of duplicate pointer/bounds checks, and an alias-disabled diagnostic. The latter variants use only valid inputs and are never enabled in production. Nine balanced-order native batches target 20 ms per cell; discard batch zero and report the median of the remaining batch means. Runtime before is #824; runtime revised is the revised branch. Both runtime builds contain the same staged/reread generated source bodies. Four shapes hold 512 handles in one list, 256 pairs, 128 groups of four, or 512 singletons.

| Shape | Pointer order | Original runtime, reread µs | Revised runtime, reread µs | Speedup |
|---|---|---:|---:|---:|
| one | ascending | 14.23 | 14.13 | 1.01× |
| one | descending | 14.80 | 14.12 | 1.05× |
| one | shuffled | 16.65 | 16.50 | 1.01× |
| pairs | ascending | 129.93 | 21.32 | 6.09× |
| pairs | descending | 686.73 | 100.26 | 6.85× |
| pairs | shuffled | 721.60 | 78.17 | 9.23× |
| quads | ascending | 77.30 | 21.74 | 3.56× |
| quads | descending | 361.99 | 65.18 | 5.55× |
| quads | shuffled | 385.07 | 60.55 | 6.36× |
| many | ascending | 247.86 | 42.77 | 5.80× |
| many | descending | 1348.56 | 69.35 | 19.45× |
| many | shuffled | 1416.58 | 59.27 | 23.90× |

The old runtime repeatedly sorted/scanned all accumulated aliases. New allocation traffic can alter address order, making timing sensitive to heap layout even though logical input is unchanged. The large rank-dependent native costs and the alias-disabled diagnostic isolate this mechanism. This demonstrates a source of sensitivity rather than proving the exact allocator state of every earlier JVM fork. Duplicate field reads/checks had a much smaller effect in the valid-input perturbation.

`AddressBench.kt` controls ranks in complete JNI roundtrips. It does fixed invocation warmup (20,000 calls for each argument/callback × one-list/many-list combination, with eight handles), then seven per-case warmup batches and nine measured batches targeting 30 ms. Reordering/regrouping the supplied Kotlin lists occurs outside the timer; generated wire packing, locking, JNI, Rust work and output reconstruction remain inside. Four balanced JVM pairs use the same driver/JAR and frozen libraries. Time-based warmup alone gives slower variants fewer invocations; this experiment limits that possible compilation-state confound. JVM allocation and lock costs still remain in the measured wrapper call.

The initial staging-only comparison, under the same fixed-work protocol, is recorded in `controlled_original` in [summary.json](summary.json); it did not reproduce a consistent reread regression. The revised comparison against #824 is:

| Shape | Pointer order | Input | µs/call before → revised | Paired speedup |
|---|---|---|---:|---:|
| one | ascending | argument | 221.27 → 94.99 | 2.12× |
| one | ascending | callback | 63.36 → 68.24 | 0.91× |
| one | descending | argument | 209.99 → 73.91 | 1.64× |
| one | descending | callback | 67.65 → 63.95 | 1.02× |
| one | shuffled | argument | 99.59 → 95.05 | 1.06× |
| one | shuffled | callback | 89.84 → 94.06 | 0.96× |
| many | ascending | argument | 484.52 → 190.09 | 2.55× |
| many | ascending | callback | 616.69 → 201.71 | 2.75× |
| many | descending | argument | 1587.46 → 223.32 | 7.15× |
| many | descending | callback | 1787.84 → 232.29 | 7.71× |
| many | shuffled | argument | 1678.47 → 258.73 | 6.49× |
| many | shuffled | callback | 1687.41 → 263.33 | 6.30× |

The sorted-vector implementation still has worst-case linear insertion/merge moves per group, and can therefore perform quadratic memory movement for adversarial descending batches. The revision removes repeated full-history sorting and duplicate scans; it does not claim constant-time history access.

```sh
python3 examples/bench-bulk/nested-input-results/investigation/decode_probe.py --checkout /path/to/repo --runtime-checkout /path/to/before --output /tmp/native-old
python3 examples/bench-bulk/nested-input-results/investigation/decode_probe.py --checkout /path/to/repo --runtime-checkout /path/to/revised --output /tmp/native-revised
```

Compile `AddressBench.kt` against the baseline frozen benchmark JAR with the same Kotlin compiler and `-Xfriend-paths=<baseline-jar>` (for internal raw-address access). Pass that thin driver JAR to `address_compare.py --before <before-build> --after <revised-build> --driver-jar <driver-jar> --output <outside-git-directory>`. Samples remain outside Git. [summary.json](summary.json) contains fork/batch summaries, hashes and native-probe provenance.
