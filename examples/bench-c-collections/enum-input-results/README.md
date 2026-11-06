# C enum collection input preparation

Baseline: merged #824 at `477201c2df2635b3ff92892d128c42777e70925d`.
Measured implementation: `9104db603` / `6b762c9d7` (the latter only records
generated benchmark artifacts). Both versions use the same expanded fixture,
dependency snapshot, compiler and C caller. Alias checking stays enabled in
both checked builds and disabled in both unchecked builds.

The checked path previously captured a construction closure for every row
containing an enum. Eligible rows now check the handle address group, reread
the caller's stable C rows and construct directly into the final Rust vector.
Each integer tag is decoded once during construction, including optional and
borrowed enum parameters. This removes one preparation allocation and its
per-row storage. Inline owners and other complex rows keep their fallback.
Unchecked inputs already constructed directly, so they are controls for this
allocation optimization. No native handle, enum or payload cloning is added.

`tagged` contains one owning handle and a sparse, non-Copy enum. `tagged_buffers`
also contains a three-byte String and a three-byte Vec. C handle allocations
and source row/buffer preparation occur outside timing and counting; successful
Rust conversion, source function execution and destruction are included. The
String/Vec input copies are required by those Rust parameter types and remain.
These are complete-call C microbenchmarks, not network-throughput measurements.

## Native allocation requests at 512 rows

| Checks | Case | Allocation calls, before → after | Requested bytes, before → after |
|---|---|---:|---:|
| Enabled | tagged | 3 → 2 | 28,672 → 16,384 |
| Enabled | tagged_buffers | 1,027 → 1,026 | 72,704 → 44,032 |
| Disabled | tagged | 1 → 1 | 8,192 → 8,192 |
| Disabled | tagged_buffers | 1,025 → 1,025 | 35,840 → 35,840 |

Checked `tagged` saves 12,288 requested bytes (24 bytes/row); checked
`tagged_buffers` saves 28,672 bytes (56 bytes/row). Both remove one allocation.
Unchecked counts are unchanged. Empty inputs allocate zero times; every case
has zero reallocations. Counts are constant across all measured samples at
sizes 0, 1, 8, 64, 512 and 4,096. Required final vectors, checked alias storage
and owned String/Vec payload allocations remain. Requested bytes count allocator
requests; they are not peak resident memory or copied payload bytes.

The generated C-boundary allocation-budget test independently checks sizes
0–4,096 under both alias policies: a handle-plus-enum constructor producing
an i64 row needs only its final vector plus enabled alias storage, with no
growth allocations.

## Median complete-call latency

Times below are nanoseconds per call. Each median uses eight fresh processes
per version; the adjacent JSON retains paired process changes, ranges and CPU times.
The last column runs the same baseline library on both sides as a noise control
(four fresh processes per side). A negative change means lower latency.

| Checks | Case | Rows | Before, ns | After, ns | Change | Same-library change |
|---|---|---:|---:|---:|---:|---:|
| Enabled | tagged | 0 | 20.67 | 19.19 | -7.1% | — |
| Enabled | tagged | 1 | 80.17 | 68.88 | -14.1% | +0.3% |
| Enabled | tagged | 8 | 202.06 | 189.10 | -6.4% | — |
| Enabled | tagged | 64 | 2003.41 | 1107.28 | -44.7% | -0.1% |
| Enabled | tagged | 512 | 16708.15 | 16115.87 | -3.5% | +2.5% |
| Enabled | tagged | 4,096 | 126818.98 | 124363.51 | -1.9% | +5.4% |
| Enabled | tagged_buffers | 0 | 21.77 | 20.82 | -4.4% | — |
| Enabled | tagged_buffers | 1 | 130.33 | 112.65 | -13.6% | -6.5% |
| Enabled | tagged_buffers | 8 | 622.92 | 538.96 | -13.5% | — |
| Enabled | tagged_buffers | 64 | 6903.59 | 6477.92 | -6.2% | +1.2% |
| Enabled | tagged_buffers | 512 | 51496.05 | 50041.08 | -2.8% | -1.4% |
| Enabled | tagged_buffers | 4,096 | 421764.83 | 404326.86 | -4.1% | -1.0% |
| Disabled | tagged | 0 | 9.11 | 9.45 | +3.8% | — |
| Disabled | tagged | 1 | 37.18 | 37.17 | -0.0% | +3.8% |
| Disabled | tagged | 8 | 113.99 | 118.09 | +3.6% | — |
| Disabled | tagged | 64 | 755.65 | 732.34 | -3.1% | -0.6% |
| Disabled | tagged | 512 | 12783.46 | 12977.00 | +1.5% | +1.3% |
| Disabled | tagged | 4,096 | 103587.10 | 103137.97 | -0.4% | +4.1% |
| Disabled | tagged_buffers | 0 | 12.52 | 12.69 | +1.4% | — |
| Disabled | tagged_buffers | 1 | 77.75 | 78.52 | +1.0% | +0.3% |
| Disabled | tagged_buffers | 8 | 469.45 | 465.26 | -0.9% | — |
| Disabled | tagged_buffers | 64 | 5819.72 | 5728.91 | -1.6% | -2.0% |
| Disabled | tagged_buffers | 512 | 47577.94 | 46587.22 | -2.1% | +2.0% |
| Disabled | tagged_buffers | 4,096 | 363864.34 | 367011.94 | +0.9% | +0.7% |

## Unaffected controls and a focused repeat

The following controls do not contain enum fields. Their allocation counts
remain identical. Their timings can vary with scheduling and binary layout;
they must not be read as benefits of removing enum preparation storage.

| Checks | Case | Rows | Before, ns | After, ns | Change | Same-library change |
|---|---|---:|---:|---:|---:|---:|
| Enabled | handles | 512 | 15071.27 | 15149.31 | +0.5% | -0.8% |
| Enabled | buffers | 512 | 49883.32 | 49650.71 | -0.5% | +5.4% |
| Enabled | numbers | 512 | 231.49 | 250.52 | +8.2% | -6.7% |
| Enabled | control | 1 | 4.62 | 4.53 | -2.0% | -0.1% |
| Disabled | handles | 512 | 12403.13 | 12537.30 | +1.1% | -0.3% |
| Disabled | buffers | 512 | 46523.47 | 46118.90 | -0.9% | +1.2% |
| Disabled | numbers | 512 | 215.24 | 214.46 | -0.4% | -5.1% |
| Disabled | control | 1 | 4.58 | 4.55 | -0.6% | +3.4% |

The broad checked run measured an 8.2% latency increase for 512 primitive-only
rows and a 13.5% increase for 4,096 rows. Both increases are retained above and
in the JSON. The committed generated `bench_take_numbers` body is unchanged
between the baseline and implementation (SHA-256
`ee3c29b4b050959bdf64ae3de7d81fd87b1cd5ace4d28467d6ca28f22f56274f`).
This establishes that the generator did not add conversion operations to that
function; it does not explain the timing difference or rule out a binary-level
regression. A focused repeat used eight fresh processes/version, twenty batches
and a 20 ms calibration target; its same-library control uses four processes:

| Case | Rows | Repeat before, ns | Repeat after, ns | Repeat change | Same-library change |
|---|---:|---:|---:|---:|---:|
| numbers | 64 | 43.57 | 42.65 | -2.1% | +3.2% |
| numbers | 512 | 220.20 | 254.23 | +15.5% | +1.1% |
| numbers | 4096 | 1673.31 | 1849.05 | +10.5% | +2.4% |
| control | 1 | 4.89 | 4.66 | -4.7% | -0.5% |

The large primitive-control increase reproduced: +15.5% at 512 rows and +10.5%
at 4,096, compared with +1.1% and +2.4% in its focused same-library control.
This is an unresolved timing regression for these benchmark binaries; it is
not dismissed as ordinary run-to-run noise. Inspection of the primitive
conversion boundary found the same 211 disassembled instructions after
normalizing relocated addresses and Rust symbol hashes; the code and data
addresses differ. Binary-layout effects are a possible explanation, not an
established cause. No extra converter operations or allocations were found.

The supported guarantee is less preparation storage on successful checked enum
inputs. Latency results are case-specific; neither unchecked inputs nor unrelated
functions are promised a speedup. The report retains all measured control changes.

## Environment and reproduction

Linux x86-64, Rust 1.98.0, GCC 13.3.0, release Rust/C builds. Timing is pinned to
CPU 0. Counter-enabled libraries are separate from counter-free timing builds.
Measured-library builds, runtime tests and Clippy finished before timing. A cached
generated-file check overlapped the start of the checked run; it finished in
0.67 seconds without compilation. Later runs had no overlapping checks.
Each timing process calibrates to 20 ms, subject to the driver's memory cap,
performs three warmup batches and records ten batches (twenty for the focused
repeat). Versions alternate order between processes; cases are shuffled. Count
runs use two fresh processes/version and ten samples after ten warmup calls.
One executable calls both versions through loader-path selection. Library and
driver hashes, `dladdr` and matching fixture/header/compiler/dependency metadata
verify the libraries being compared. Choose an available CPU for reproduction.

Use clean committed checkouts. `<base>` is a detached worktree at `477201c2d`.
Run these four builds, then repeat with `--disable-alias-checks` and distinct
output paths. Complete all builds before timing either pair.

```sh
CARGO_BUILD_JOBS=1 python3 examples/bench-c-collections/build.py --checkout <base> --output /tmp/before --target-dir /tmp/enum-target
CARGO_BUILD_JOBS=1 python3 examples/bench-c-collections/build.py --output /tmp/after --target-dir /tmp/enum-target
CARGO_BUILD_JOBS=1 python3 examples/bench-c-collections/build.py --checkout <base> --output /tmp/before-count --target-dir /tmp/enum-target --count-allocations
CARGO_BUILD_JOBS=1 python3 examples/bench-c-collections/build.py --output /tmp/after-count --target-dir /tmp/enum-target --count-allocations
python3 examples/bench-c-collections/input-compare.py --before /tmp/before-count --after /tmp/after-count --output /tmp/counts --kind count --forks 2 --cases tagged,tagged_buffers,handles,buffers,numbers,control --sizes 0,1,8,64,512,4096
taskset -c 0 python3 examples/bench-c-collections/input-compare.py --before /tmp/before --after /tmp/after --output /tmp/times --kind time --forks 8 --cases tagged,tagged_buffers,handles,buffers,numbers,control --sizes 0,1,8,64,512,4096 --batches 10 --target-ms 20
taskset -c 0 python3 examples/bench-c-collections/input-compare.py --before /tmp/before --after /tmp/after --output /tmp/aa --kind time --same-build before --forks 4 --cases tagged,tagged_buffers,handles,buffers,numbers,control --sizes 1,64,512,4096 --batches 10 --target-ms 20
taskset -c 0 python3 examples/bench-c-collections/input-compare.py --before /tmp/before --after /tmp/after --output /tmp/repeat --kind time --forks 8 --cases numbers,control --sizes 64,512,4096 --batches 20 --target-ms 20
taskset -c 0 python3 examples/bench-c-collections/input-compare.py --before /tmp/before --after /tmp/after --output /tmp/repeat-aa --kind time --same-build before --forks 4 --cases numbers,control --sizes 64,512,4096 --batches 20 --target-ms 20
```

`summary.json` retains compact measurements, paired process changes, timing ranges, settings
and sample hashes. `provenance.json` retains build and executable metadata.
Raw console output, per-process stdout and raw sample CSV files are omitted.
