# C input row replay measurements

Baseline generator: `718c45d80` (#824 after #869). Updated generator: `1f5c7743c`.
Both revisions use the identical extended benchmark fixture. Release libraries are frozen copies; a shared Cargo cache cannot overwrite them. Later commits record documentation, tests and generated artifacts.

## What changes

Flat C rows containing only scalar fields and opaque pointers are borrowed as an immutable Rust slice. Resource-bearing rows validate their complete address group first, then reread values and construct the final Rust vector. The prepared-row closure vector is omitted. Resource-free scalar rows also benefit from the slice reader. Buffers, enums, inline owners, callbacks and nested collections retain their existing preparation strategy. Source constructors run once after validation; error cleanup and consumption-prefix receipts remain in place.

The caller must keep wire rows and descriptors alive and read-only for the whole call, including concurrent or reentrant code and source constructors. This is a stricter precondition than retaining prepared field snapshots. It applies to all collection inputs; pointee ownership and mutation still follow the handle contract. No runtime mutation check or additional lock is added. The shared row slice also forbids conflicting mutable access through aliases.

## Benchmark method

The C caller creates handles and wire arrays before the timer/allocation counter. One measured call decodes and consumes the entire collection and computes a checked sum. Box deallocation, alias validation, final Rust vector allocation, constructor work and source drops are included. Source-handle creation and C row allocation/free are excluded. The text case includes its required native String copies and uses the unchanged staged fallback. Plain numbers and a scalar call supply additional controls; the numbers reader itself changes in the final slice variant.

Cases: one owned handle per row; records with two handles and a tag; wide records with one handle and seven u64 fields; records with a handle and a three-byte string; plain u64 input rows; scalar control. Handles contain one u64 and have no custom destructor. These are binding microbenchmarks, not packet throughput measurements.

Both versions use one identical C executable per comparison. LD_LIBRARY_PATH selects the frozen library, and dladdr verifies the loaded path. Build records contain compiler/profile, fixture, lockfile, header and artifact hashes. Header declarations match after removing comments; no C ABI layout or signature change is hidden by that normalization.

Latency: four fresh processes per revision/cell, shuffled cell order and alternating revision order; 12 batches per process after calibration and three warmup batches. Target batch duration is 10 ms, bounded to 262,144 prepared rows (1,048,576 iterations for empty cases). Consequently large or very small cases can use shorter batches. A process score is total measured wall time divided by calls; the table reports the median of four scores. CPU time and all samples are preserved. No builds/tests ran during timing. Host affinity is recorded; CPU frequency and other host activity are not controlled.

Allocation counts use separate instrumented libraries: two processes per revision/cell, 10 measured calls after 10 warmups. Counts are native successful alloc/realloc operations and requested bytes, not live memory or JVM allocations.

## Allocation counts

| Case | Rows | Allocations before → after | Requested bytes before → after |
|---|---:|---:|---:|
| control | 1 | 0 → 0 | 0 → 0 |
| handles | 0 | 0 → 0 | 0 → 0 |
| handles | 1 | 3 → 2 | 40 → 24 |
| handles | 512 | 3 → 2 | 20480 → 12288 |
| numbers | 0 | 0 → 0 | 0 → 0 |
| numbers | 1 | 1 → 1 | 8 → 8 |
| numbers | 512 | 1 → 1 | 4096 → 4096 |
| records | 0 | 0 → 0 | 0 → 0 |
| records | 1 | 3 → 2 | 96 → 56 |
| records | 512 | 3 → 2 | 49152 → 28672 |
| text | 0 | 0 → 0 | 0 → 0 |
| text | 1 | 4 → 4 | 83 → 83 |
| text | 512 | 515 → 515 | 42496 → 42496 |
| wide | 0 | 0 → 0 | 0 → 0 |
| wide | 1 | 3 → 2 | 208 → 80 |
| wide | 512 | 3 → 2 | 106496 → 40960 |

All repeated count measurements agree, and reallocations are zero throughout. Nonempty eligible consuming collections save one allocation. At 512 rows, removed preparation storage saves 8 KiB for handles, 20 KiB for two-handle records and 64 KiB for wide records. The address-history buffer and final Rust vector remain. All count sizes, including 4,096, are in counts/results.json.

## Latency

| Case | Rows | Before ns/call | After ns/call | Change |
|---|---:|---:|---:|---:|
| control | 1 | 3.4 | 3.4 | -1.9% |
| handles | 0 | 19.7 | 19.2 | -2.4% |
| handles | 1 | 81.6 | 64.6 | -20.9% |
| handles | 8 | 190.1 | 174.3 | -8.3% |
| handles | 64 | 1095.4 | 991.4 | -9.5% |
| handles | 512 | 15674.7 | 14252.2 | -9.1% |
| handles | 4096 | 123825.3 | 114146.8 | -7.8% |
| numbers | 0 | 7.8 | 8.9 | +14.4% |
| numbers | 1 | 26.9 | 25.6 | -5.0% |
| numbers | 8 | 32.2 | 27.4 | -14.9% |
| numbers | 64 | 54.9 | 39.4 | -28.3% |
| numbers | 512 | 332.2 | 217.4 | -34.5% |
| numbers | 4096 | 2487.1 | 1512.2 | -39.2% |
| records | 0 | 19.9 | 18.8 | -5.8% |
| records | 1 | 103.7 | 85.3 | -17.7% |
| records | 8 | 332.1 | 272.6 | -17.9% |
| records | 64 | 4083.0 | 4201.6 | +2.9% |
| records | 512 | 31699.9 | 30463.8 | -3.9% |
| records | 4096 | 266954.2 | 249389.1 | -6.6% |
| text | 0 | 20.4 | 20.8 | +2.2% |
| text | 1 | 113.1 | 111.8 | -1.1% |
| text | 8 | 386.5 | 393.0 | +1.7% |
| text | 64 | 3569.9 | 3454.4 | -3.2% |
| text | 512 | 25736.7 | 25619.8 | -0.5% |
| text | 4096 | 206743.6 | 207388.6 | +0.3% |
| wide | 0 | 19.2 | 18.9 | -1.4% |
| wide | 1 | 95.2 | 70.6 | -25.9% |
| wide | 8 | 289.8 | 195.9 | -32.4% |
| wide | 64 | 2892.2 | 2097.5 | -27.5% |
| wide | 512 | 20909.1 | 16692.8 | -20.2% |
| wide | 4096 | 155444.3 | 129074.1 | -17.0% |

At 512 rows, handle latency fell 9.1%, wide-record latency 20.2%, and primitive-row latency 34.5%. Two-handle records improved only 3.9% at that size; ranges overlap, so a consistent speedup is not established there. The text fallback is approximately unchanged. The empty primitive case increased by about 1.1 ns (+14.4%), and the 64-row two-handle record median increased 2.9%; these regressions remain included.

## Identical-build controls

| Case | Rows | Same-library median shift |
|---|---:|---:|
| control | 1 | -0.9% |
| handles | 512 | +4.0% |
| wide | 512 | +0.9% |

The A/A run shows up to 4.0% median variation for the selected cases. Treat small latency differences cautiously; the allocation reductions are deterministic. No universal latency improvement is claimed.

## Experiment retained for comparison

The first raw-pointer reread variant removed the same preparation allocation but slowed the 4,096-handle median by 6.3%. The final variant uses a validated shared row slice. A focused four-process confirmation improved the handle medians at 1/64/512/4,096 rows and retained the wide-record improvement. Diagnostics JSON/CSV preserve both experiments; final latency/AA raw logs are committed separately.

## Reproduce

Build the same fixture against the baseline and updated generator with build.py --checkout, separate --output directories and optional --target-dir cache. Build once with --count-allocations and once without. Use input-compare.py --kind count or --kind time; add --same-build after for the A/A control. Exact commands, paths and hashes are recorded in each results.json. Native libraries and executables are not committed.

The existing output-collection smoke remains in CI. A separate input-driver smoke exercises every input case with the allocation instrumentation, and both source suites assert sums and complete ownership consumption through generated Rust/C code.
