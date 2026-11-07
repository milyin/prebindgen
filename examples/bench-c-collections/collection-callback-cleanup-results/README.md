# Failure-only cleanup for C callback collections

Baseline: merged #824 at `44090750efa9a4b958f28a449b9b3c8dfad049cc`.
Measured implementation: `c8da3fc43` (including the diagnostic follow-up). Both use
the same unchanged fixture, pinned dependencies, compiler and C caller. Checked
pairs keep alias checking enabled in both versions; unchecked pairs keep it
disabled in both. This isolates the cleanup optimization, not the alias opt-out.
These synthetic C-call measurements do not establish packet throughput.

## What is timed

The C callback returns a descriptor for an array of owning handles. Rust decodes
those handles into its final `Vec<Resource>`, reads the values, and destroys them.
Previously an ownership scan built another native vector so failure cleanup could
find unread owners. Successful decoding now commits a stack-held guard without
running that scan. Failure cleanup may still rescan, allocate and deduplicate.
The final Rust vector and enabled alias checks retain their required storage.

C rows, native handles and callback storage are prepared outside timing/counting.
The measured complete-call interval includes the C loop, callback invocation,
Rust conversion, alias checking when enabled, constructors and native destruction.
No cleanup work that runs on successful decoding is excluded from the interval.

## Native allocation counts

Counts include allocator/reallocator requests, not peak or retained bytes.
For 512 handles, the removed cleanup vector had one initial allocation and seven
growth reallocations. At 4,096 handles it had ten growth reallocations.

| Alias checks | Handles | Allocations before → after | Reallocations before → after | Requested bytes before → after |
|---|---:|---:|---:|---:|
| Enabled | 0 | 0 → 0 | 0 → 0 | 0 → 0 |
| Enabled | 1 | 3 → 2 | 0 → 0 | 120 → 24 |
| Enabled | 64 | 3 → 2 | 4 → 0 | 4512 → 1536 |
| Enabled | 512 | 3 → 2 | 7 → 0 | 36768 → 12288 |
| Enabled | 4096 | 3 → 2 | 10 → 0 | 294816 → 98304 |
| Disabled | 0 | 0 → 0 | 0 → 0 | 0 → 0 |
| Disabled | 1 | 2 → 1 | 0 → 0 | 104 → 8 |
| Disabled | 64 | 2 → 1 | 4 → 0 | 3488 → 512 |
| Disabled | 512 | 2 → 1 | 7 → 0 | 28576 → 4096 |
| Disabled | 4096 | 2 → 1 | 10 → 0 | 229280 → 32768 |

The unchecked path now allocates only the final Rust vector. The checked path
also allocates its existing alias-check group. A generated-boundary budget test
requires exactly one final-vector allocation per unchecked 512-handle callback,
zero reallocations, and no metadata bytes. Fixed-record callback allocation
remains zero; ordinary input and scalar allocation counts are unchanged.

## Median complete-call latency

Eight fresh processes per version/case, ten measured batches/process, three
warmup batches and a 15 ms calibration target subject to the driver's memory cap.
Values are medians of each process's total wall time divided by its call count;
negative changes mean lower latency. The identical-library controls use the same
baseline library for both labels, four processes/version and the same batch and
calibration settings. A dash means that size was outside the identical-library
control subset; it is not a zero difference.

| Alias checks | Handles | Before | After | Change | Identical-library change |
|---|---:|---:|---:|---:|---:|
| Enabled | 0 | 0.0421 µs | 0.0357 µs | -15.1% | — |
| Enabled | 1 | 0.1128 µs | 0.0812 µs | -28.0% | -0.4% |
| Enabled | 8 | 0.3009 µs | 0.1893 µs | -37.1% | — |
| Enabled | 64 | 1.3944 µs | 1.0281 µs | -26.3% | +1.9% |
| Enabled | 512 | 15.8381 µs | 15.2014 µs | -4.0% | +0.2% |
| Enabled | 4096 | 131.4402 µs | 119.2087 µs | -9.3% | -0.4% |
| Disabled | 0 | 0.0285 µs | 0.0214 µs | -24.9% | — |
| Disabled | 1 | 0.0777 µs | 0.0465 µs | -40.2% | +2.0% |
| Disabled | 8 | 0.2232 µs | 0.1150 µs | -48.5% | — |
| Disabled | 64 | 1.0065 µs | 0.6459 µs | -35.8% | +2.9% |
| Disabled | 512 | 13.5649 µs | 12.2029 µs | -10.0% | +0.1% |
| Disabled | 4096 | 108.4222 µs | 95.5677 µs | -11.9% | +1.1% |

All eight checked processes improved at sizes 0–512; seven of eight improved
at 4,096 handles. The unchecked 512-handle case improved in seven of eight
processes; all other unchecked collection sizes improved in all eight. The small
collection gains are large relative to their identical-library controls. The
512-handle checked gain is smaller (4.0%), with a 0.2% identical-library difference.
Per-process scores, CPU time and ranges are retained in the JSON summaries.

## Unrelated controls and a focused repeat

Fixed-record callbacks, fixed borrows/moves, scalar calls and ordinary collection
inputs do not use the modified collection cleanup. Their complete-call timings
can still vary with allocation history, scheduling and binary layout. The broad
run includes these increases; no general improvement is claimed for other paths.

| Alias checks | Case | Before | After | Change |
|---|---|---:|---:|---:|
| Enabled | callback_fixed | 47.14 ns | 47.12 ns | -0.0% |
| Enabled | fixed_borrow | 12.32 ns | 11.96 ns | -3.0% |
| Enabled | fixed_move | 28.38 ns | 29.94 ns | +5.5% |
| Enabled | control | 4.35 ns | 4.46 ns | +2.6% |
| Disabled | callback_fixed | 37.04 ns | 38.27 ns | +3.3% |
| Disabled | fixed_borrow | 6.23 ns | 6.88 ns | +10.4% |
| Disabled | fixed_move | 23.13 ns | 23.31 ns | +0.8% |
| Disabled | control | 4.23 ns | 4.22 ns | -0.1% |

The initial unchecked fixed-borrow median increased 10.4% (about 0.65 ns), while
the checked fixed-move control increased 5.5%. A focused repeat of fixed borrows
and the scalar control used eight fresh processes/version, twenty batches and a
20 ms calibration target. Its separate identical-library run used the same
settings. The fixed-borrow slowdown did not reproduce:

| Alias checks | Case | Before | After | Repeat change | Identical-library change |
|---|---|---:|---:|---:|---:|
| Enabled | control | 4.44 ns | 4.55 ns | +2.4% | +0.3% |
| Enabled | fixed_borrow | 13.18 ns | 13.22 ns | +0.3% | -1.5% |
| Disabled | control | 4.49 ns | 4.57 ns | +1.8% | +3.0% |
| Disabled | fixed_borrow | 7.12 ns | 7.12 ns | +0.0% | +2.5% |

The original broad results are retained, including the fixed-move increase.
These controls do not establish a consistent cross-function speedup or absence of
regressions. The supported claim is reduced successful callback-collection work,
verified allocation savings, and the measured callback-collection latency changes.

## Environment and reproduction

Linux x86-64, Rust 1.98.0 and GCC 13.3.0, optimized Rust/C builds, CPU 0 for timing.
Counter-enabled libraries are separate from counter-free timing libraries. All
builds and tests finished before timing. Count runs use two fresh processes per
version and ten samples for each cell. Versions alternate order between processes;
cases are shuffled. One C executable calls both versions through loader-path
selection; `dladdr`, library/driver hashes and matching fixture/header/compiler/
lockfile metadata verify the actual libraries. Choose an available CPU.

Use clean committed checkouts. `<base>` is a detached worktree at `44090750e`.
Repeat all four builds with `--disable-alias-checks` and distinct output paths for
the unchecked comparison. Finish every build before timing either mode.

```sh
CARGO_BUILD_JOBS=1 python3 examples/bench-c-collections/build.py --checkout <base> --output /tmp/before --target-dir /tmp/cold-target
CARGO_BUILD_JOBS=1 python3 examples/bench-c-collections/build.py --output /tmp/after --target-dir /tmp/cold-target
CARGO_BUILD_JOBS=1 python3 examples/bench-c-collections/build.py --checkout <base> --output /tmp/before-count --target-dir /tmp/cold-target --count-allocations
CARGO_BUILD_JOBS=1 python3 examples/bench-c-collections/build.py --output /tmp/after-count --target-dir /tmp/cold-target --count-allocations
python3 examples/bench-c-collections/input-compare.py --before /tmp/before-count --after /tmp/after-count --output /tmp/counts --kind count --forks 2 --cases callback_many,callback_fixed,handles,numbers,fixed_move,fixed_borrow,control --sizes 0,1,8,64,512,4096
taskset -c 0 python3 examples/bench-c-collections/input-compare.py --before /tmp/before --after /tmp/after --output /tmp/times --kind time --forks 8 --cases callback_many,callback_fixed,handles,numbers,fixed_move,fixed_borrow,control --sizes 0,1,8,64,512,4096 --batches 10 --target-ms 15
taskset -c 0 python3 examples/bench-c-collections/input-compare.py --before /tmp/before --after /tmp/after --output /tmp/aa --kind time --same-build before --forks 4 --cases callback_many,callback_fixed,control --sizes 1,64,512,4096 --batches 10 --target-ms 15
taskset -c 0 python3 examples/bench-c-collections/input-compare.py --before /tmp/before --after /tmp/after --output /tmp/control-repeat --kind time --forks 8 --cases fixed_borrow,control --sizes 1 --batches 20 --target-ms 20
taskset -c 0 python3 examples/bench-c-collections/input-compare.py --before /tmp/before --after /tmp/after --output /tmp/control-aa --kind time --same-build before --forks 8 --cases fixed_borrow,control --sizes 1 --batches 20 --target-ms 20
```

Adjacent files retain compact summaries and build provenance. Raw console logs,
per-process stdout and raw sample CSV files are intentionally omitted from the PR.
