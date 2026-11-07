# Nested bulk handle input measurements

Compare #824 at `b179b9888d85` with the optimization at `054b0d5c5778`. Both use the same benchmark fixture and identical compiled JVM classes. `bulk` packs fields into C-layout byte buffers; `plain` uses ordinary JNI row conversion.

The generator now recognizes redundant bulk modifiers when element construction is deferred for grouped handle validation inside an existing native bulk frame. Eligible inner lists validate their handles and alias groups, then reread native fields and construct directly in the final Rust vector. This removes the temporary vector of deferred constructors, including in flat handle declarations with redundant bulk modifiers. It adds no JNI row reads or payload copies. Ordinary JNI rows and flat handle/tag records with no redundant modifier retain their conversion paths.

Each operation is a **complete successful roundtrip**: Kotlin supplies handles, Rust decodes and returns them, and Kotlin reconstructs the returned wrappers. Callback cases have Rust invoke Kotlin for its nested handle list before decoding and returning it to Kotlin. These timings and counts do not isolate input decoding. Native counts cover calling-thread Rust allocations, including final Rust vectors and native handle allocations for the output; JVM allocations and JVM/native internals outside Rust's allocator are excluded.

"One inner list" surrounds a list of N handles with two empty lists. "Many singleton lists" has N lists, each holding one handle. Empty inner lists allocate no temporary constructor vector. Native allocations fall by one per nonempty inner list: one allocation for a 512-element inner list, or 512 allocations for 512 singleton lists. Both arrangements save 8,192 cumulative requested native bytes at N=512. Final Rust collections and alias-check storage remain.

| Transport | Case | Handles | µs/call before → after | Paired speedup | Native allocations / reallocations before → after |
|---|---|---:|---:|---:|---:|
| bulk | One inner list, ordinary input | 1 | 1.712 → 1.638 | 1.09× | 7 / 2 → 6 / 2 |
| bulk | One inner list, ordinary input | 64 | 9.249 → 9.231 | 1.01× | 70 / 2 → 69 / 2 |
| bulk | One inner list, ordinary input | 512 | 72.190 → 74.836 | 0.97× | 518 / 2 → 517 / 2 |
| bulk | Many singleton lists, ordinary input | 1 | 1.308 → 1.360 | 0.96× | 7 / 2 → 6 / 2 |
| bulk | Many singleton lists, ordinary input | 64 | 42.278 → 40.021 | 1.05× | 259 / 7 → 195 / 7 |
| bulk | Many singleton lists, ordinary input | 512 | 1,494.552 → 1,452.299 | 1.02× | 2,051 / 10 → 1,539 / 10 |
| bulk | One inner list returned by Kotlin callback to Rust | 1 | 3.150 → 3.034 | 0.99× | 10 / 4 → 9 / 4 |
| bulk | One inner list returned by Kotlin callback to Rust | 64 | 10.696 → 10.561 | 1.01× | 73 / 4 → 72 / 4 |
| bulk | One inner list returned by Kotlin callback to Rust | 512 | 70.628 → 72.094 | 1.06× | 521 / 4 → 520 / 4 |
| bulk | Many singleton lists returned by Kotlin callback to Rust | 1 | 2.625 → 2.583 | 1.02× | 10 / 4 → 9 / 4 |
| bulk | Many singleton lists returned by Kotlin callback to Rust | 64 | 44.084 → 42.837 | 1.03× | 262 / 9 → 198 / 9 |
| bulk | Many singleton lists returned by Kotlin callback to Rust | 512 | 1,549.345 → 1,462.485 | 1.10× | 2,054 / 12 → 1,542 / 12 |
| plain | One inner list, ordinary input | 512 | 397.362 → 383.689 | 1.04× | 520 / 4 → 520 / 4 |
| plain | Many singleton lists, ordinary input | 512 | 2,525.480 → 2,528.781 | 1.01× | 2,562 / 521 → 2,562 / 521 |
| plain | One inner list returned by Kotlin callback to Rust | 512 | 574.844 → 561.695 | 1.02× | 524 / 13 → 524 / 13 |
| plain | Many singleton lists returned by Kotlin callback to Rust | 512 | 2,742.469 → 2,785.875 | 0.98× | 2,566 / 530 → 2,566 / 530 |
| bulk | Flat handle list | 512 | 71.134 → 76.124 | 0.92× | 517 / 1 → 516 / 1 |
| bulk | Flat handle/tag records (control) | 512 | 83.287 → 79.379 | 1.07× | 516 / 1 → 516 / 1 |
| plain | Scalar call (control) | 1 | 0.015 → 0.015 | 0.93× | 0 / 0 → 0 / 0 |

The unchanged-code A/A run has paired timing drift up to 3.2%. Treat timing differences near this scale as inconclusive. See [summary.json](summary.json) for all fork medians, allocation counts, requested-byte totals, JVM bytes and control results. Requested bytes sum allocation sizes and new reallocation sizes; they are not live or peak memory.

Native counts are identical across all 20 observations per cell: two JVMs, 1,000 warmup calls and 10 counted calls each. Latency uses separate uninstrumented release builds, the same baseline JAR for both libraries, six balanced before/after JVM pairs, 10 warmup batches and 11 measured batches targeting 30 ms. Reported times are medians of per-fork medians; speedup is the reciprocal of the median paired after/before ratio. The A/A run loads the unchanged baseline under both labels. JVM settings: fixed 256 MiB heap, G1 and two active processors. No compiler or allocation-counting run competes with timing.

The four native builds run sequentially. Each library is frozen before Kotlin compilation and loaded in a separate JVM to verify its instrumentation mode. Generated benchmark bindings are committed. Raw logs/samples remain outside Git; the summary records hashes and build provenance.

```sh
python3 examples/bench-bulk/run.py --checkout /path/to/before --output /tmp/nested-before --build-only
python3 examples/bench-bulk/run.py --checkout /path/to/after --output /tmp/nested-after --build-only --check-generated
python3 examples/bench-bulk/nested-input-results/compare.py --before /tmp/nested-before --after /tmp/nested-after --output /tmp/nested-time
python3 examples/bench-bulk/nested-input-results/compare.py --before /tmp/nested-before --after /tmp/nested-after --output /tmp/nested-aa --aa
```

Build two further directories with `--native-allocations`, then pass them to [count.py](count.py). Its `--smoke` option uses `-Xcheck:jni` to exercise ordinary/bulk nested arguments and callback results; smoke timings are not performance results.

The longer focused repeat uses four balanced JVM pairs, 15 warmup batches, and 17 measured batches targeting 60 ms. Positive time changes mean slower calls.

| Focused case | Handles | µs/call before → after | Paired time change |
|---|---:|---:|---:|
| Many singleton lists returned by Kotlin callback to Rust | 512 | 1,412.929 → 1,523.561 | +8.11% |
| Many singleton lists, ordinary input | 512 | 1,492.489 → 1,509.971 | +2.39% |
| One inner list, ordinary input | 512 | 69.135 → 73.672 | +5.72% |
| Flat handle/tag records (control) | 512 | 79.839 → 79.329 | -1.32% |
| Flat handle list | 512 | 68.683 → 68.410 | +0.05% |
| Scalar call (control) | 1 | 0.014 → 0.014 | -0.15% |

The callback-many-list gain in the main run did not reproduce: paired time changed by -8.9% initially and +8.1% in the repeat. One large inner list was slower in both runs (+2.7% and +5.7%); the flat-list initial +9.1% increase did not repeat (+0.05%). Allocation reductions are established; a consistent latency benefit is not. The stable-field reread trades intermediate storage for a second native validation/construction traversal. All slower and control results are retained.

Use [focused.py](focused.py) with the same `--before`, `--after` and `--output` arguments to repeat this comparison.
