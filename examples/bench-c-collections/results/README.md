# Buffered C collection output measurements

Measured 2026-11-03, Linux x86-64. Baseline `3b9acfffa` is #824 at `6eabb97c7`
plus the identical benchmark fixture. Updated implementation: `1c2e5ecf2`.
Libraries are release builds. Compiler, Cargo lock/configuration, source and
artifact hashes are preserved in each results file. This is a synthetic C binding
benchmark, not a network-packet throughput measurement.

## What each cycle does

The C caller asks Rust to produce a collection, reads a checksum, destroys every
owned field, and frees each row buffer through its generated function. Handle
values are read through generated native getters. There is no input collection,
JVM, JNI, or JIT. Resource objects contain one u64 and have no custom destructor;
real application objects may have substantially different costs.

- Handle list: N opaque Resource pointers.
- Records: N records, each with two independently owned Resource pointers and a tag.
- Three inner lists: `[[], N handles, []]`, so N is total handles, not outer length.
- One-handle inner lists: N lists containing one handle each.
- Numbers: N primitive u64 rows; no owned fields or per-element heap allocations.
- Scalar: one unchanged native scalar call.

The same compiled C executable serves both revisions. Its loader path selects
one frozen library per process, and `dladdr` verifies which library answered.
Generated headers match byte-for-byte, and their hashes are checked before the
comparison. All result sizes and checksums are asserted by the executable.

## Allocation counts

Separate libraries enable a benchmark-only thread-local Rust allocation counter.
The timed libraries have no global allocator hook. The counter brackets the
entire produce/read/free cycle; it counts successful alloc/realloc requests and
requested bytes, not C-library allocations, deallocations, or retained memory.
Each process warms 100 cycles, then records ten individual counted cycles.

There are 31 case/size cells: sizes 0, 1, 8, 64, 512 and 4096 for the five
collection shapes, plus the scalar control. Two processes per revision per cell:
124 processes and 1,240 samples. Every repeated metric within a cell is identical.

| Case | N | Native allocations before → after | Requested native bytes before → after |
|---|---:|---:|---:|
| Handle list | 0 | 0 → 0 | 0 → 0 |
| Handle list | 1 | 4 → 3 | 32 → 24 |
| Handle list | 512 | 515 → 514 | 16,384 → 12,288 |
| Two-handle records | 512 | 1,027 → 1,026 | 40,960 → 32,768 |
| Three inner lists | 0 | 3 → 2 | 240 → 120 |
| Three inner lists | 512 | 518 → 516 | 16,624 → 12,408 |
| One-handle inner lists | 1 | 7 → 5 | 112 → 64 |
| One-handle inner lists | 512 | 2,051 → 1,538 | 57,344 → 32,768 |
| Primitive numbers | 512 | 2 → 2 | 8,192 → 8,192 |
| Scalar control | 1 | 0 → 0 | 0 → 0 |

At N=512, removing the ownership queue saves 4 KiB for a flat handle list,
8 KiB for two-handle records, and 24 KiB plus 513 allocations for one-handle inner
lists. The three-list shape saves two allocations and 4,216 requested bytes.
Primitive and scalar allocation counts are unchanged.

**Reallocations are zero in both revisions in every measured cell.** The old code
preallocated its row Vec to the known length; its `into_boxed_slice()` did not
reallocate in these cases. The new code allocates the final boxed storage at its
exact length and removes that finalization step by construction. The observed
allocation savings come from removed commit queues, not an asserted old compaction
copy. Full counts: [JSON](counts/results.json), [CSV](counts/samples.csv).

## Uninstrumented latency

Sixteen case/size cells: sizes 1, 64 and 512 for the five collection shapes plus
the scalar control. Four fresh processes per revision per cell, 128 processes and
2,560 batches. Before/after order alternates by fork; cases use a recorded
shuffle. Each process calibrates its batch iteration count to at least 20 ms,
warms for at least 200 ms, then records 20 batches at that iteration count.
The exact counts and durations remain in the samples.

One process runs at a time, pinned to CPUs 0–1. Builds, tests and allocation
probes finished before timing. CPU frequency/turbo and other host activity are
not controlled. Fixed C flags: `-std=gnu11 -O3 -fno-plt -Wall -Wextra -Werror`.
The score is total measured wall time divided by total cycles in one process,
then the median of four process scores. Ranges are minima/maxima across those
four processes, not confidence intervals. Calling-thread CPU time is also
recorded. All samples remain included; no outliers are removed.

| Case | N | Before ns/cycle [fork range] | After ns/cycle [fork range] | Wall change | CPU change |
|---|---:|---:|---:|---:|---:|
| Scalar control | 1 | 3.1 [2.9–3.3] | 3.0 [2.9–3.0] | -3.0% | -3.0% |
| Handle list | 1 | 68.5 [65.6–68.9] | 47.7 [47.3–52.2] | -30.3% | -30.2% |
| Handle list | 64 | 1,533.8 [1,476.3–1,653.7] | 1,285.7 [1,257.6–1,327.8] | -16.2% | -16.2% |
| Handle list | 512 | 16,949.8 [16,734.0–17,064.5] | 16,469.5 [16,145.7–16,866.0] | -2.8% | -2.8% |
| One-handle inner lists | 1 | 109.8 [109.3–114.2] | 75.9 [74.2–84.6] | -30.9% | -30.9% |
| One-handle inner lists | 64 | 7,820.2 [7,412.2–8,356.2] | 5,426.3 [5,303.1–5,697.9] | -30.6% | -30.6% |
| One-handle inner lists | 512 | 59,470.2 [57,744.1–63,750.7] | 43,641.7 [41,662.1–49,957.5] | -26.6% | -26.4% |
| Three inner lists | 1 | 141.4 [138.2–144.6] | 87.9 [84.9–91.1] | -37.8% | -37.9% |
| Three inner lists | 64 | 1,594.7 [1,558.3–1,635.0] | 1,287.7 [1,279.1–1,496.8] | -19.3% | -19.3% |
| Three inner lists | 512 | 17,230.0 [16,970.2–17,450.4] | 15,680.0 [15,354.7–16,217.5] | -9.0% | -8.9% |
| Primitive numbers | 1 | 37.3 [36.4–40.0] | 33.8 [33.3–34.7] | -9.3% | -9.2% |
| Primitive numbers | 64 | 128.4 [125.6–157.4] | 64.6 [64.0–66.9] | -49.7% | -49.7% |
| Primitive numbers | 512 | 799.1 [789.1–801.1] | 350.2 [343.3–356.3] | -56.2% | -56.1% |
| Two-handle records | 1 | 84.4 [83.3–152.7] | 66.7 [64.2–134.7] | -20.9% | -20.7% |
| Two-handle records | 64 | 4,303.1 [4,089.2–4,565.1] | 3,964.9 [3,887.7–4,443.6] | -7.9% | -7.9% |
| Two-handle records | 512 | 34,137.1 [33,241.8–34,846.9] | 31,898.9 [30,947.4–32,288.8] | -6.6% | -6.6% |

The strongest owned-output result is many short inner lists: N=512 changes from
59.5 to 43.6 microseconds/cycle (−26.6%), with all four paired changes negative
(−34.3%, −23.0%, −27.9%, −16.7%). The corresponding savings are 513 allocations
and 24 KiB per cycle. Flat N=512 moves only −2.8%, within the scale of the A/A
variation below; a reliable latency improvement is not established for that case.
Two-handle N=512 records change −6.6%, and the three-list shape changes −9.0%.

The primitive case also improves despite identical allocation counts. Its new
copy loop uses SIMD loads/stores for four u64 values per iteration; the baseline
retains scalar Vec push/capacity checks, including empty-closure bookkeeping,
and an out-of-line boundary helper. The saved [disassembly](assembly/) documents
those code-generation differences. This supports an explanation beyond allocation
counts; it is not an exclusive causal attribution of the −56.2% aggregate change.

## A/A control

The same baseline library/executable is used under both labels, with the same
protocol: 16 fresh processes and 320 batches.

| Case | N | Before ns/cycle [fork range] | After ns/cycle [fork range] | Wall change | CPU change |
|---|---:|---:|---:|---:|---:|
| Scalar control | 1 | 3.1 [3.0–3.3] | 3.0 [3.0–3.3] | -2.4% | -2.4% |
| Handle list | 512 | 17,380.9 [17,157.1–17,745.8] | 17,206.6 [16,639.7–18,125.2] | -1.0% | -0.9% |

The identical-library scalar and 512-handle aggregates move −2.4% and −1.0%.
The latter's paired changes are −2.5%, +3.4%, +2.7%, −6.2%. Small A/B changes
need caution. These results support lower native allocation and observed speedups
for several synthetic cases, not a uniform application-throughput guarantee.

## Evidence and limits

All commands, selected native library paths, source/build/executable hashes,
per-fork scores, checksums and all batch samples are in the three JSON/CSV pairs.
Raw stdout remains under `/tmp/c-output-{counts,latency,aa}/raw/`; its hashes are
recorded. Before/after headers are identical. Counts and timing libraries use
identical generated code; only the benchmark allocation feature differs within a
revision. The one C executable used in each comparison is identified by hash.

Generated Rust/header review snapshots are committed under `../generated/`.
See [../README.md](../README.md) for reproduction commands and
[review.md](review.md) for the Opus 5.5 review summary. Correctness verification
includes the C/C++ ASan/UBSan and Kotlin/Java JNI suite, five runtime Miri tests,
and actual generated C calls under Miri for partial rows, nested outputs, multiple
independent outputs and callback argument conversion. Strings in the generated
failure fixtures exercise by-value opaque ownership; deliberate clones exist
only in those test projections, not the benchmark or default conversion.
