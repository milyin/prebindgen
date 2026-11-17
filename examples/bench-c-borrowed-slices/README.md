# Borrowed C numeric slice outputs

A Rust `fn values(owner: &Owner) -> &[i64]` previously allocated an owning
C row array, copied every element and required the caller to free it. The new
binding returns `{ const int64_t *data; size_t len; }` pointing into `Owner`.
This study compares the same fixture, compiled against the parent implementation
and this change. It measures generated C-to-Rust calls, not network throughput.

## Cases

- `view`: return the slice, check its length and endpoints, release the old owning
  row buffer on the parent. The caller intentionally does not traverse every row.
- `read`: the same call, then sum every element in C and verify the checksum.
- `callback`: one Rust call invokes a C callback **32 times**, borrowing the same
  slice each time. C verifies endpoints and callback count, and frees each old
  owning buffer on the parent. Times/counts below are per outer call, not per delivery.
- `control`: the unchanged primitive-only `bench_control(n)` call. It does not
  allocate or traverse a collection; its size parameter is only a label/control input.

Owner creation and destruction are outside measurements. All assertions remain
active. Every frozen library has its own compiled C caller and matching header;
ABI ownership and element access intentionally change between versions.

## Native allocations

At every nonzero tested length (1, 16, 128, 512, 8,192, 65,536), ordinary
`view`/`read` calls go from **1 allocation, 8 × length requested bytes** to **0**.
The callback case goes from **32 allocations, 256 × length requested bytes** to
**0**. At length 65,536 that removes 512 KiB per ordinary call, or 16 MiB per
32-delivery call. Reallocations are zero in both versions. Empty slices and the
primitive control allocate zero in both. Five counted observations agree for
every case. These are requested native bytes, not retained memory or JVM bytes.

Separate generated-boundary regression tests check original pointer identity,
reserved vector capacity, optional/error outputs, borrowed fixed arrays and an
inline Vec descriptor move. Repeated getters and callback deliveries allocate
zero; explicit collection copies still allocate and have matching destructors.
Strict-provenance Miri exercises those Rust-to-generated-C entry points.

## Complete-call timing

Linux x86_64, AMD EPYC 7713 VM with two available CPUs, glibc 2.39. Rust release
(opt-level 3, debug assertions off), C `-O3`; timings use an uninstrumented
library, allocation counts a separate instrumented build. Four fresh processes
per side, balanced before/after order, shuffled cases, CPU 0 affinity, three
calibration rounds, three warmup batches and nine measured batches targeting
10 ms each. A/A repeats the same baseline binary/library under both labels.
Compiler, revisions, source/binary hashes and settings are in `provenance.json`.
CPU governor/turbo settings are unavailable in this VM.

Each time is the median of per-process batch medians, in **nanoseconds per outer
call**. The ratio is the median of paired after/before process medians (lower is
better); it need not equal the ratio of the two displayed medians. A/A ratios
show substantial VM noise: do not interpret small timing changes as established
wins or regressions. All lengths and per-process medians are in `latency.csv`;
`aa-latency.csv` retains the identical-library controls, including slower labels.

| Case | Length | Before ns | After ns | Paired after/before | A/A ratio |
|---|---:|---:|---:|---:|---:|
| view | 0 | 10.45 | 8.21 | 0.5973 | 1.412 |
| view | 1 | 37.60 | 12.08 | 0.3454 | 0.991 |
| view | 512 | 176.96 | 9.80 | 0.0497 | 0.892 |
| view | 65,536 | 18584.30 | 6.04 | 0.0003 | 0.978 |
| read | 0 | 10.53 | 6.66 | 0.6296 | 0.794 |
| read | 1 | 22.11 | 11.15 | 0.5012 | 0.981 |
| read | 512 | 228.84 | 143.85 | 0.6778 | 1.021 |
| read | 65,536 | 38797.96 | 17646.54 | 0.4192 | 1.075 |
| callback | 0 | 320.19 | 122.00 | 0.3972 | 0.957 |
| callback | 1 | 945.57 | 140.15 | 0.1639 | 1.007 |
| callback | 512 | 4008.49 | 109.37 | 0.0259 | 1.061 |
| callback | 65,536 | 610125.69 | 158.71 | 0.0002 | 1.081 |
| control | 0 | 2.98 | 2.97 | 0.9980 | 0.986 |
| control | 1 | 2.97 | 2.96 | 0.9939 | 0.777 |
| control | 512 | 2.97 | 2.95 | 0.9910 | 0.994 |
| control | 65,536 | 2.97 | 3.03 | 1.0017 | 1.001 |

The allocation/copy elimination and original-address checks are deterministic.
The large `view` and callback differences reflect removing O(length) copying
when the consumer only inspects endpoints. `read` still does O(length) work in
C; its improvement is smaller and noisier. Small-call timing, and exact speedup
factors on this shared VM, are inconclusive. No universal throughput claim is made.

## Reproduce

Run on Linux from a clean committed checkout. The earlier checkout can be the
#824 parent or the pre-indexing worktree used for this study; their relevant C
output implementations are identical. The report identifies exact commits.

```sh
python3 examples/bench-c-borrowed-slices/run.py --before /path/to/parent \
  --output /tmp/c-borrowed-study --build-only
python3 examples/bench-c-borrowed-slices/run.py --before /path/to/parent \
  --output /tmp/c-borrowed-study --prepared
python3 examples/bench-c-borrowed-slices/run.py --before /path/to/parent \
  --output /tmp/c-borrowed-study --prepared --aa
```

The helper freezes all four libraries before timing, verifies loaded library
paths and hashes, and checks that the same fixture built both versions. Raw
samples, build logs and frozen binaries remain in the output directory outside
Git. Committed files contain only compact summaries and provenance.
