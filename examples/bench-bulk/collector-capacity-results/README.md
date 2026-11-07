# Capacity hints for Kotlin callback return collectors

Collection callback returns retain immutable raw rows in Kotlin until the native
call and handle locks finish. The collector previously grew a default ArrayList
as rows arrived. Rust already knows the collection length, so it now reserves
space once before delivery for collections of **64 rows or more**. Collections
of 0–63 rows keep the existing growth policy and add no JNI calls. The public
fold API, per-row method signature and per-row store code are unchanged.

The internal collector is an explicitly named class with its typed row list. A
shared `SequenceCapacity.reserve(Int)` interface supplies the reservation operation.
JNINative registers its actual Class once after loading the native library; Rust
pins that Class and caches the method ID. A large capture performs one cache read
and one reservation upcall, with no extra method-name/signature allocation or
lookup on each call. The existing per-collection run-method lookup remains.
This introduces one initialization call/class pin per binding module; cold startup
was not benchmarked. No public API or runtime-crate type was added.

[Java ArrayList](https://docs.oracle.com/en/java/javase/17/docs/api/java.base/java/util/ArrayList.html)
provides ensureCapacity to reduce incremental reallocation. The implementation
uses that public operation, with no JVM-internal field layout assumptions. The
HotSpot-only test probe reads the actual backing-array length to verify that
reservation completed before the first row.

A row transfers ownership only after successful delivery. If reservation throws,
the collector holds no rows and Rust drops the still-owned output resources.
Existing completed-prefix, failed-row and unread-row cleanup is preserved. Tests
cover a synthetic reservation exception, Some(empty)/None/source errors, borrowed
and owned values, nested/bulk rows, typed handlers, reentry and concurrency.
The synthetic failure is not a forced JVM heap OOM. New keep rules preserve the
internal capacity interface/collectors for JNI; Android R8 was not run.

## Source revisions and method

- Baseline artifacts: `9c0491cea` from #865. All nine generator/runtime source
  trees exactly match merged #824 at `39dd54e5b302fe6dd082483024647d812788bd93`.
- First prototype: `f8723d08d`, per-call reservation method lookup above ten rows.
- Cached eager prototype: `4da5c25f9`, shared method cache but still above ten rows.
- Final implementation: `2c8c1e809`, shared cache and the 64-row cutoff. Later
  commits contain benchmark snapshots and reports only.

[source-identity.json](source-identity.json) records tree IDs and shared driver
hashes. The runner verifies identical fixture, dependency lock, Rust release
profile/environment, driver JAR and compiler versions. Before/after libraries
are copied into distinct directories before build-cache reuse. Rust 1.98.0,
Kotlin 2.4.21, OpenJDK 11.0.29, Linux x86-64, CPU affinity 0,1, G1 and a fixed
256 MiB JVM heap. No builds or tests run during timing.

Final results use four fresh JVMs per version and case, alternating order. Each
warms for at least two seconds and a 500 ms quiet interval in JVM compilation,
then records twelve batches. Process scores are total measured time divided by
total calls; tables show their median and full min–max range. Lower time is better.
A size is the row/list length, not the number of fields. Allocation includes the
whole generated call, destination objects and user fold. Ratios compare the same
operation before/after. These are warmed microbenchmarks, not network throughput.

`primitive_fold` returns/folds records with eight i64 fields. `fold_flat` consumes
a handle list and returns handles through the callback. `fold_nested` does the
same for one-handle inner lists. Handle cases include unchanged input conversion
and locking, and are not isolated measurements of the collector. `control` is an
unchanged scalar JNI call.

## Final 64-row policy

| Case | Rows | Before µs [min–max] | After µs [min–max] | Time change | JVM bytes/call before → after |
|---|---:|---:|---:|---:|---:|
| `control` | 1 | 0.013 [0.013–0.014] | 0.013 [0.013–0.013] | -1.50% | 0.0 → 0.0 |
| `fold_flat` | 512 | 979.899 [970.391–1039.018] | 1024.674 [1012.977–1026.350] | +4.57% | 83,280.0 → 78,608.0 |
| `fold_nested` | 64 | 288.160 [275.742–340.305] | 296.286 [282.033–317.991] | +2.82% | 52,979.0 → 52,340.5 |
| `primitive_fold` | 0 | 0.891 [0.859–0.902] | 0.863 [0.840–1.034] | -3.14% | 80.0 → 80.0 |
| `primitive_fold` | 1 | 1.203 [1.181–1.233] | 1.183 [1.175–1.338] | -1.66% | 240.0 → 240.0 |
| `primitive_fold` | 11 | 3.440 [3.315–3.552] | 3.422 [3.400–3.501] | -0.51% | 1,360.0 → 1,360.0 |
| `primitive_fold` | 16 | 4.478 [4.394–8.043] | 4.455 [4.405–4.481] | -0.52% | 1,984.0 → 1,984.0 |
| `primitive_fold` | 32 | 7.865 [7.694–8.063] | 7.955 [7.834–8.204] | +1.15% | 3,800.0 → 3,800.0 |
| `primitive_fold` | 63 | 14.433 [14.133–15.775] | 14.801 [13.970–15.651] | +2.55% | 7,552.0 → 7,552.0 |
| `primitive_fold` | 64 | 15.115 [14.385–15.914] | 14.580 [14.174–15.509] | -3.54% | 7,656.0 → 7,008.0 |
| `primitive_fold` | 65 | 15.044 [14.745–15.637] | 15.165 [14.731–15.640] | +0.80% | 7,760.0 → 7,120.0 |
| `primitive_fold` | 512 | 111.065 [107.822–113.998] | 110.200 [104.870–115.104] | -0.78% | 60,088.0 → 55,416.0 |

At 512 primitive records, JVM allocation falls by 4,672 bytes/call, from 60,088
to 55,416 (7.8%). At 64 records it falls by 648 bytes, from 7,656 to 7,008 (8.5%).
These are eliminated backing-array allocations/reference copies, not removed
record allocations or fewer row callbacks. Sizes below 64 keep identical measured
allocation, including the 63-row boundary case.

Primitive-record timing is approximately unchanged: 512 rows move −0.8%, 64 rows
−3.5%, and 65 rows +0.8%, all with overlapping process ranges. The 11/16-row
regressions from the eager policy are absent in the final medians. The 32/63-row
shifts (+1.1%/+2.6%) also have overlapping ranges and no extra JNI calls. No
consistent throughput improvement is claimed for this allocation optimization.

The 512-handle case saves the same 4,672 JVM bytes/call (5.6%) and has a +4.6%
time median with overlapping ranges. Nested 64-row time moves +2.8%, also with
overlapping ranges; its roughly 0.6 KB allocation saving includes JVM/JIT variation.
These whole-call timing changes remain visible in the report and are not
silently discarded. The extra reservation is one call per qualifying collection,
not per handle.

All raw batches, process scores, CPU times, GC counts and commands are in
[results.json](results.json) and [samples.csv](samples.csv). Separate twelve-process
[CheckJNI diagnostics](diagnostic.json) exercise both sides of 63/64/65 and handle
and nested delivery without JNI warnings; those times are excluded.

## Why small collections skip reservation

The initial two-process [pilot](pilot-results.json) added a method lookup and
reservation call above ten rows. JVM allocation improved, but 16/32-row medians
were +8.8%/+6.4%. After caching the reservation method, the four-process
[eager policy](eager-results.json) still measured +9.7% at 11 rows and +7.3% at
16 rows. The default growth cost on small lists did not justify an added upcall.

The final policy uses no reservation at 0–63 rows, and reserves 64+ where the
fixed setup cost is amortized and the absolute allocation saving is larger. It
deliberately permits default growth on smaller lists. This cutoff is an internal
heuristic selected from this workload, not a guarantee of speed on every JVM.
The JNI test checks zero reservation calls at 63, one at 64/65, exactly one run
per row, and sufficient capacity before any run.

The pilot has two processes/version/case; eager and final policies have four.
All runs are retained, including slow processes (for example, one eager 512-row
baseline at 205.8 µs and one final 16-row baseline at 8.0 µs). Raw experimental
samples are [pilot-samples.csv](pilot-samples.csv) and
[eager-samples.csv](eager-samples.csv).

## A/A controls

[aa-results.json](aa-results.json) and [aa-samples.csv](aa-samples.csv) compare the
identical baseline artifacts with four fresh JVMs per side. Allocation is identical.
Apparent time shifts are +0.5% (empty), +0.4% (single), +2.1% (16 rows), and +0.6%
(512 rows); ranges overlap. These establish timing variability, not improvements.

Additional handle/nested identical-artifact controls are in
[handle-aa-results.json](handle-aa-results.json) and
[handle-aa-samples.csv](handle-aa-samples.csv), with four processes per side:

- `fold_flat` (512): 991.022 → 986.734 µs (-0.43%), identical JVM allocation.
- `fold_nested` (64): 301.373 → 278.412 µs (-7.62%), identical JVM allocation.

Process ranges overlap in these controls. Their variation reinforces treating
small whole-call timing shifts as inconclusive; the final handle +4.6% median
remains reported above, and no latency speedup is claimed.

## Verification and review

- Workspace Rust tests and Clippy with warnings denied pass.
- Actual C/Kotlin/JNI callers pass with C ASan/UBSan and JVM CheckJNI, including
  capacity failure, raw proxies, optional/error outputs and the cutoff boundary.
- All 281 example review snapshots match generated outputs. Benchmark snapshots
  record changed plain callback Rust/Kotlin; bulk outputs are unchanged.
- [Claude Opus 5.5 review notes](review.md) describe findings and fixes from four
  review passes. No blockers remained in the final cutoff/test review.

## Reproduction

Build clean checkouts at the two measured source commits with
`jni-output-results/prepare.py`, distinct `--output` directories and a shared
`--target` cache. Compile the unchanged `jni-output-results/Driver.kt` against the
baseline JAR and use that same driver for both versions. Run
`jni-output-results/measure.py` with `--forks 4`, CPU affinity 0,1 and the exact
`--cell` settings recorded in results metadata. Run `--check-jni --forks 1`
separately from timing. The JSON metadata records all artifact and sample hashes.
