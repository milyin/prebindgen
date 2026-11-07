# Revised nested input measurements

Compare #824 at `b179b9888d85` with revised #885 at `6e00520a7726`. The earlier staging-only attempt and its slower cases are retained in [initial-README.md](initial-README.md) and [initial-summary.json](initial-summary.json).

The revised implementation keeps removal of prepared constructor vectors and fixes dynamic alias bookkeeping: skip empty groups, check singleton addresses with binary search/insertion, sort/check only a new multi-access group, then merge its distinct addresses into the already sorted history. It neither re-sorts nor revalidates previous history. The first batch's allocation is reused, and the merge uses the existing history allocation. Null/type/pointer/value checks, shared/exclusive conflict policy, consumed-input receipts and failure cleanup remain enabled.

The original history algorithm sorted and scanned the accumulated history for every inner list. Its cost depended strongly on address order, which temporary allocations can perturb. Extracted generated Rust decoders with fixed pointer ranks showed this cost dominating; rereading itself was slightly faster than staging in the initial isolated comparison. Merely skipping repeated pointer/bounds checks did not remove the dominant cost. Fixed-rank JVM reruns did not show a consistent staging-removal slowdown. See [the investigation](investigation/README.md) for the controlled experiments and the limits of this attribution.

## Complete-call results

All list rows below have 512 handles. A large-inner-list case surrounds the 512-element list with two empty lists; singleton cases have 512 one-element lists. The scalar has one value. `bulk` uses packed C-layout bytes; `plain` uses ordinary JNI rows. These are complete successful roundtrips, including Kotlin packing/locks, Rust decoding, source work, output encoding and Kotlin reconstruction. Callback cases first have Rust invoke Kotlin for its input list. They do not isolate the input decoder.

| Transport | Case | µs/call before → revised | Paired speedup | Native allocations before → revised |
|---|---|---:|---:|---:|
| bulk | One large inner list, ordinary argument | 66.623 → 69.236 | 0.95× | 518 → 517 |
| bulk | Singleton lists, ordinary argument | 1,499.004 → 188.490 | 7.89× | 2,051 → 1,539 |
| bulk | One inner list returned by Kotlin callback to Rust | 68.796 → 66.363 | 1.02× | 521 → 520 |
| bulk | Singleton lists returned by Kotlin callback to Rust | 1,533.054 → 200.677 | 7.68× | 2,054 → 1,542 |
| bulk | Flat handles | 69.011 → 70.568 | 0.98× | 517 → 516 |
| bulk | Flat handle/tag records | 84.489 → 79.236 | 1.07× | 516 → 516 |
| plain | One large inner list, ordinary argument | 402.946 → 388.502 | 1.03× | 520 → 520 |
| plain | Singleton lists, ordinary argument | 2,440.279 → 1,127.392 | 2.13× | 2,562 → 2,562 |
| plain | One inner list returned by Kotlin callback to Rust | 600.060 → 583.515 | 1.00× | 524 → 524 |
| plain | Singleton lists returned by Kotlin callback to Rust | 2,767.861 → 1,456.057 | 1.88× | 2,566 → 2,566 |
| plain | Scalar | 0.014 → 0.014 | 0.99× | 0 → 0 |

Native allocation counts match all 20 observations per cell. Bulk singleton-list arguments and callback results still save 512 allocations and 8 KiB of cumulative requested Rust bytes per call. A single 512-handle inner list saves one allocation and 8 KiB. Reallocations and requested-byte totals for every case are in [summary.json](summary.json). Counts cover the calling-thread Rust allocator, including final collections and native output handles, rather than JVM allocations or peak/live memory.

Latency uses six balanced JVM pairs, 10 warmup batches and 11 measured batches targeting 30 ms, separate uninstrumented release libraries and the exact same baseline JAR. Times are medians of per-fork medians; speedup is the reciprocal of the median paired after/before ratio. Unchanged-library A/A drift reaches 6.8% in this run. The separate fixed-work JVM investigation warms the two entry points with both nested shapes (80,000 calls total) before its time-based case warmup; use that experiment to assess sensitivity to address order and compilation state. Controls and all slower cases are retained.

Builds run sequentially, libraries are frozen before Kotlin compilation and actual instrumentation mode is checked by a JVM probe. Kotlin class bytecode is identical. No compilers/counting runs overlap latency. Two instrumented JVMs per cell use 1,000 warmup calls and 10 measured calls. Raw samples/logs stay outside Git; summaries record hashes and build provenance.

Reproduce with [count.py](count.py), [compare.py](compare.py) and the investigation tools. Build before/after separately with `examples/bench-bulk/run.py --checkout ... --output ... --build-only`; use `--native-allocations` only for allocation-count builds, and `--check-generated` for the candidate. Original focused-repeat scripts remain available for auditing the superseded result.
