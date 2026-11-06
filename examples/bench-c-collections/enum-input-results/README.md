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
This is a real timing regression for the original benchmark binaries. The
placement investigation below identifies a dominant cause; the original
measurements are retained. Inspection of the primitive
conversion boundary found the same 211 disassembled instructions after
normalizing relocated addresses and Rust symbol hashes; the code and data
addresses differ. That comparison alone did not explain the slowdown. No extra
converter operations or allocations were found.

The supported guarantee is less preparation storage on successful checked enum
inputs. Latency results are case-specific; neither unchecked inputs nor unrelated
functions are promised a speedup. The report retains all measured control changes.

## Follow-up: native sum-function placement

The dominant cause is the placement of the benchmark's native Rust summation
function, `c_collection_flat::take_numbers`. This function performs the workload
**after** generated C input conversion. Its machine instructions are unchanged,
but the shorter generated code moves the function from library offset `0x27580`
to `0x27030`. Its 28-byte vector-sum loop starts 64 bytes into the function:

| Binary | Loop address in library | Offset within a 64-byte instruction-cache line |
|---|---:|---:|
| Before | `0x275c0` | 0 |
| After | `0x27070` | 48 |

In the second placement, the tight loop crosses a cache-line boundary. This
observation suggested an explanation; the interventions below test causality.
They identify a code-placement cost on this AMD EPYC 7713 VM, rather than an
extra operation in the primitive converter. They do not identify the precise
processor front-end mechanism or promise the same effect on another CPU.

The original driver also capped primitive tests at 512 calls/batch for 512-row
inputs: median measured duration was 0.116 ms despite a 20 ms target. The
follow-up uses a fixed, reusable input pool and much longer batches. Primitive
C rows are read-only and can be reused while every call still constructs,
consumes and destroys its own final Rust vector. No handles are reused.

Both frozen libraries are loaded into one process with local symbol scopes and
receive the same C input addresses. A separate untimed allocator probe confirms
one native allocation per 512-row call and the same destination address for
both versions, in both single-array and streaming modes. Source-offset sweeps
also reproduced the slowdown. Thus different input/allocator addresses and
short batches do not explain it.

To control placement directly, the isolated diagnostic copies the verified
152-byte native sum function to executable pages at chosen offsets. It repairs
its sole relative deallocation call, keeping that target fixed across the cloned
variants, and redirects only the native sum call through the process's function
pointer table. The generated converters, native data, instructions and allocation
policy remain fixed. Separate function copies avoid requiring both versions to
share a call target. Library files are never changed. This experiment is not
part of generated bindings or the runtime library.

Eight fresh processes/configuration use eight paired batches each, alternate
library load and measurement order, and randomize configuration order. Each
batch processes 33,554,432 elements (65,536 calls at 512 rows; 8,192 at 4,096),
rather than increasing input storage with the number of calls. Both a reused
single array and a fixed 2 MiB streaming pool are measured. The table below uses
the streaming pool; all configurations and paired process scores are retained
in `placement-summary.json`.

| Native function offsets modulo 64, before / after | Rows | Before, ns/call | After, ns/call | Change |
|---|---:|---:|---:|---:|
| Original binaries: 0 / 48 | 512 | 189.72 | 222.65 | +17.4% |
| Original binaries: 0 / 48 | 4,096 | 1288.57 | 1600.43 | +24.2% |
| Both copied: 0 / 0 | 512 | 198.48 | 189.25 | -4.7% |
| Both copied: 0 / 0 | 4,096 | 1284.90 | 1271.31 | -1.1% |
| Both copied: 48 / 48 | 512 | 242.26 | 231.00 | -4.6% |
| Both copied: 48 / 48 | 4,096 | 1590.41 | 1611.13 | +1.3% |
| Reproduce placement: 0 / 48 | 512 | 194.66 | 226.72 | +16.5% |
| Reproduce placement: 0 / 48 | 4,096 | 1255.27 | 1533.12 | +22.1% |
| Swap placement: 48 / 0 | 512 | 235.09 | 186.17 | -20.8% |
| Swap placement: 48 / 0 | 4,096 | 1514.85 | 1244.12 | -17.9% |

Matching the function placement removes the slowdown: the 512-row case measures
4.7% lower latency after the change with both copies aligned to offset zero.
Recreating the original offset difference reproduces a 16.5% slowdown; swapping
the offsets reverses it to a 20.8% improvement. The 4,096-row results show the
same direction. Moving the same instructions is enough to produce and reverse
the large timing gap. The extra enum support does not introduce primitive-row
conversion work; the compiled program's layout changes its execution time.

The original binaries still have their measured slowdown. This investigation
explains a dominant cause, rather than erasing the result or guaranteeing that
unrelated functions retain their timing after compilation. The confirmed enum
preparation allocation savings stand independently. No generator padding,
mandatory alignment policy or global compiler flags were added.

Reproduce using the existing checked, counter-free frozen build directories:

```sh
python3 examples/bench-c-collections/enum-input-results/placement.py --before /tmp/before --after /tmp/after --output /tmp/placement --probe-allocations
```

`placement.py` verifies library hashes, discovers function/table offsets from ELF
metadata, checks the exact supported native instruction shape and read-only
relocation pages, then builds `placement.c`. It rejects unsupported code before
loading it. This diagnostic currently requires Linux x86-64, 4 KiB pages, GNU
libc for the optional allocator probe, and the verified Rust/LLVM sum-function
shape. It needs `cc`, `nm`, `readelf`, `objdump` and `taskset`; use `--cpu` to select
an available core. `--smoke` runs checksum/protocol checks without useful latency
statistics. New compiler code shapes require a separate audit rather than a
silent change to the relocation assumptions.

The full run passed all checksum assertions. The smoke run and GNU libc address
probe also pass; a deliberately modified native function is rejected during
inspection before loading/execution. The committed summary omits raw logs.

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
