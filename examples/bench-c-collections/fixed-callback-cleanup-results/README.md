# Fixed C callback-result cleanup

Baseline: merged #824 at `54cf11e5785fe0f700ee7dd52b3e9c23008d680e`.
Measured implementation: `fbece75f8` (stack guard, static owner bound and 32-owner
cap). Later changes to the generated test fixture's public field declarations do
not affect benchmark code. Both builds use the same unchanged benchmark fixture,
pinned dependency lockfile, Rust/C compiler and C executable. Within each pair,
only the generator/runtime revision changes; alias checking stays on or off in
both libraries. These are synthetic C calls, not packet-throughput measurements.

## Target result

The C callback returns one record containing two owned handles and a scalar tag.
Rust decodes the record, sums its fields and destroys its resources. The old
cleanup vector allocated once even on successful decoding; its small initial
capacity requested 96 bytes. The new guard records addresses/drop functions in
stack storage, without copying native payloads. Inputs, handles and C callback
storage are prepared before timing/counting. Complete-call timing includes the
C loop, callback invocation, Rust decoding/constructor, ownership bookkeeping and
native destruction. No success-path cleanup work is excluded from the interval.

| Alias checks | Native allocations/call | Requested bytes/call | Median latency before → after | Change |
|---|---:|---:|---:|---:|
| Enabled | 1 → 0 | 96 → 0 | 69.42 → 43.17 ns | -37.8% |
| Disabled | 1 → 0 | 96 → 0 | 65.56 → 36.09 ns | -45.0% |

All eight paired processes measured lower fixed-record callback latency in each
alias mode. A separate generated-C-boundary test also requires zero allocations
for 1,000 retained callback invocations; creating the native handles and retained
callback is outside that budget.

## Controls and scope

Handle-bearing collections retain their existing dynamic cleanup storage. Their
allocation/reallocation counts are unchanged. Scalar functions, fixed borrows and
fixed moves do not use the modified cleanup guard; their allocation counts stay
unchanged too. Timing includes binary-layout/allocator/scheduling variation, so
these controls are reported rather than assumed identical.

| Alias checks | Case | Rows | Before | After | Change | Identical-library change |
|---|---|---:|---:|---:|---:|---:|
| Enabled | callback_fixed | 1 | 69.42 ns | 43.17 ns | -37.8% | +4.3% |
| Enabled | callback_many | 1 | 104.74 ns | 102.07 ns | -2.5% | +0.7% |
| Enabled | callback_many | 512 | 15117.92 ns | 14840.78 ns | -1.8% | +5.3% |
| Enabled | fixed_move | 1 | 25.97 ns | 27.12 ns | +4.4% | +3.9% |
| Enabled | control | 1 | 3.93 ns | 3.93 ns | -0.1% | +10.3% |
| Disabled | callback_fixed | 1 | 65.56 ns | 36.09 ns | -45.0% | -5.6% |
| Disabled | callback_many | 1 | 75.15 ns | 73.54 ns | -2.1% | -9.3% |
| Disabled | callback_many | 512 | 12709.62 ns | 13157.15 ns | +3.5% | +2.2% |
| Disabled | fixed_move | 1 | 22.14 ns | 21.71 ns | -2.0% | -0.8% |
| Disabled | control | 1 | 3.96 ns | 3.99 ns | +0.8% | +2.7% |

The checked fixed-move control measured 4.4% higher latency, while its identical-
library control differed by 3.9%. Dynamic-collection results and other small
control differences do not establish a consistent speedup or regression. The
fixed-record callback improvement (38–45%) is substantially larger than its
identical-library differences (+4.3% and −5.6%). No general throughput claim is
made for unrelated functions or collection returns.

## Protocol

Linux x86-64, Rust 1.98.0, GCC 13.3.0; optimized Rust/C libraries. Timing runs use
counter-free builds pinned to CPU 0. Allocation runs use separately compiled
counter-enabled builds. All builds/tests finished before timing began.

Each before/after timing comparison uses eight fresh processes per version/case,
ten measured batches per process, three warmup batches, and a calibration target
of 15 ms subject to the driver's memory cap. Versions alternate order between
processes; case order is shuffled. Reported latency is the median of process
wall-clock time/call; JSON summaries also include CPU time and every process score.
Cases include the fixed callback, collections of 0/1/64/512 handles, fixed moves,
fixed borrows, primitive collections and the scalar control. Identical-library
runs use the same baseline library for both labels, four fresh processes/version,
and the same batches/warmup/target settings. Counts use two processes/version and
ten samples each, separately for each alias mode.

The same C executable calls both versions. Loader-path selection and `dladdr`
verify the actual frozen library; hashes of libraries, drivers, fixture files,
header ABI, compiler and dependency lockfile are verified. Build provenance and
compact per-case/process summaries are adjacent. Raw console logs, CSV samples
and per-run stdout remain in temporary output directories and are not committed.

## Reproduce

Use clean, committed checkouts; replace `<base>` with a detached worktree at
`54cf11e57`. Choose an available CPU. Repeat the four builds for unchecked mode by
adding `--disable-alias-checks`, using separate output names. Finish all builds
before running either timing comparison.

```sh
CARGO_BUILD_JOBS=1 python3 examples/bench-c-collections/build.py --checkout <base> --output /tmp/before --target-dir /tmp/fixed-target
CARGO_BUILD_JOBS=1 python3 examples/bench-c-collections/build.py --output /tmp/after --target-dir /tmp/fixed-target
CARGO_BUILD_JOBS=1 python3 examples/bench-c-collections/build.py --checkout <base> --output /tmp/before-count --target-dir /tmp/fixed-target --count-allocations
CARGO_BUILD_JOBS=1 python3 examples/bench-c-collections/build.py --output /tmp/after-count --target-dir /tmp/fixed-target --count-allocations
python3 examples/bench-c-collections/input-compare.py --before /tmp/before-count --after /tmp/after-count --output /tmp/counts --kind count --forks 2 --cases callback_fixed,callback_many,fixed_borrow,fixed_move,handles,numbers,control --sizes 0,1,64,512
taskset -c 0 python3 examples/bench-c-collections/input-compare.py --before /tmp/before --after /tmp/after --output /tmp/times --kind time --forks 8 --cases callback_fixed,callback_many,fixed_borrow,fixed_move,handles,numbers,control --sizes 0,1,64,512 --batches 10 --target-ms 15
taskset -c 0 python3 examples/bench-c-collections/input-compare.py --before /tmp/before --after /tmp/after --output /tmp/aa --kind time --same-build before --forks 4 --cases callback_fixed,callback_many,fixed_move,control --sizes 1,512 --batches 10 --target-ms 15
```
