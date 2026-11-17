# C input alias checks: enabled versus disabled

Both builds use revision `46f75616c97faf5718d55719518bae79d52e4af2` and the
same fixture, dependencies, compiler and C executable. `before` means the default
checked mode; `after` means `set_emit_alias_checks(false)`. This isolates the
additional opt-out above #875, including removal of the collection alias prepass.
Required validation and ownership cleanup remain in both builds. This measures
synthetic C calls, not network throughput.

## Complete-call latency

Linux x86-64, Rust 1.98.0, GCC 13.3.0, optimized Rust/C builds without allocation
counters. One CPU (0), four independent processes per case/version, eight timed
batches each. Versions alternate order between processes and cases are shuffled.
Each process calibrates its iteration count toward 10 ms subject to the driver's
memory cap, then runs three warmup batches. Values below are medians of each
process's total elapsed time divided by its call count. Smaller is better.

Handles and source payloads are created outside timing. Rust conversion,
construction and destruction, plus the C calling loop, are timed. The fixed
borrow case reuses two distinct live handles. Fixed moves and record callbacks
consume two newly prepared handles per call. A record callback returns one
three-field record; a collection callback returns the requested number of handles.

| Case | Rows | Checked | Unchecked | Latency change |
|---|---:|---:|---:|---:|
| fixed_borrow | 1 | 0.0112 µs | 0.0059 µs | -47.0% |
| fixed_move | 1 | 0.0267 µs | 0.0218 µs | -18.5% |
| callback_fixed | 1 | 0.0698 µs | 0.0675 µs | -3.4% |
| handles | 1 | 0.0631 µs | 0.0326 µs | -48.3% |
| handles | 64 | 1.0439 µs | 0.5805 µs | -44.4% |
| handles | 512 | 14.4824 µs | 11.5802 µs | -20.0% |
| records | 512 | 29.7717 µs | 23.7464 µs | -20.2% |
| text | 512 | 24.7121 µs | 21.9698 µs | -11.1% |
| buffers | 512 | 45.4626 µs | 42.2034 µs | -7.2% |
| callback_many | 512 | 15.0490 µs | 12.3673 µs | -17.8% |
| numbers | 512 | 0.2072 µs | 0.2104 µs | +1.5% |
| control | 1 | 0.0042 µs | 0.0042 µs | -0.2% |

The identical checked-library control measured −0.3% for 512 handles, −0.4% for
512 records, −1.4% for collection callbacks, −2.4% for fixed borrows, and 0.0% for
fixed moves. The record callback's −3.4% difference is inconclusive against its
+4.4% identical-library control. Primitive/control calls have no alias work to
remove; their small differences are noise or binary-layout effects.

## Large payloads can regress

Short text/byte payloads contain three bytes. `_large` uses 4 KiB of valid UTF-8
(Euro signs plus one ASCII byte) and/or a 4 KiB byte buffer. The initial broad run
showed +16.6% latency for 512 text rows and +2.0% for combined text/byte rows.
A focused repeat used six processes/version, twelve batches, a 20 ms calibration
target, and the same frozen libraries. Its separate identical-library control used
four processes/version and the same batch/target settings:

| Case | Rows | Checked | Unchecked | Change | Identical-library change |
|---|---:|---:|---:|---:|---:|
| buffers_large | 64 | 436.65 µs | 478.19 µs | +9.5% | -0.8% |
| buffers_large | 512 | 4317.88 µs | 4646.08 µs | +7.6% | -1.9% |
| bytes_large | 64 | 12.77 µs | 11.25 µs | -11.9% | +1.5% |
| bytes_large | 512 | 1309.30 µs | 1288.36 µs | -1.6% | +0.5% |
| text_large | 64 | 188.35 µs | 245.90 µs | +30.6% | +7.7% |
| text_large | 512 | 2892.40 µs | 3299.84 µs | +14.1% | +2.9% |

Disabling checks is not universally faster: the focused run also shows slower
text and combined-buffer calls despite less bookkeeping. The cause has not been
isolated; allocator state, memory access and compiled code layout can affect these
copy/UTF-8-heavy loops. The 64-row text result includes one large outlier; all six
paired text results were slower, including at 512 rows. Per-function
`.alias_checks(true)` can retain checked generation for bindings where it measures
better under an otherwise unchecked project default. Keep these regressions when
assessing the change; do not infer packet throughput from the smaller cases.

## Native allocation counts

Separate counter-enabled builds, two processes/version, ten samples each, sizes
0, 1, 64, 512 and 4096. Input handle creation and C memory allocation/free are
outside the counter. Counts below are per call for 512 rows (fixed cases use one
call with two handles). Requested bytes are allocator requests, not retained or
peak memory. Callback ownership tracking still grows its vector; its reallocations
are unchanged (seven in the 512-handle callback case).

| Case | Allocations checked → unchecked | Requested bytes checked → unchecked |
|---|---:|---:|
| buffers | 1026 → 1025 | 39936 → 31744 |
| callback_fixed | 1 → 1 | 96 → 96 |
| callback_many | 3 → 2 | 36768 → 28576 |
| fixed_borrow | 0 → 0 | 0 → 0 |
| fixed_move | 0 → 0 | 0 → 0 |
| handles | 2 → 1 | 12288 → 4096 |
| numbers | 1 → 1 | 4096 → 4096 |
| records | 2 → 1 | 28672 → 12288 |
| text | 514 → 513 | 26112 → 17920 |

Empty inputs allocate nothing in both modes. Nonempty handle-bearing collections
save one alias-group allocation. Record callbacks retain their 96-byte failure
cleanup allocation; fixed arguments already use allocation-free checks. Alias
opt-out removes neither required owned payload copies nor the final Rust vector.

## Reproduce

Start from a clean committed checkout. Build before timing and run no builds or
tests during the measurements. Use the same `--target-dir` to share build cache;
each output still contains its own frozen library. `dladdr` and library/fixture/
header/compiler hashes are checked by the runner. Choose an available CPU.

```sh
CARGO_BUILD_JOBS=1 python3 examples/bench-c-collections/build.py --output /tmp/checked --target-dir /tmp/alias-target
CARGO_BUILD_JOBS=1 python3 examples/bench-c-collections/build.py --output /tmp/unchecked --target-dir /tmp/alias-target --disable-alias-checks
CARGO_BUILD_JOBS=1 python3 examples/bench-c-collections/build.py --output /tmp/checked-count --target-dir /tmp/alias-target --count-allocations
CARGO_BUILD_JOBS=1 python3 examples/bench-c-collections/build.py --output /tmp/unchecked-count --target-dir /tmp/alias-target --count-allocations --disable-alias-checks
python3 examples/bench-c-collections/input-compare.py --before /tmp/checked-count --after /tmp/unchecked-count --output /tmp/counts --kind count --forks 2 --sizes 0,1,64,512,4096
taskset -c 0 python3 examples/bench-c-collections/input-compare.py --before /tmp/checked --after /tmp/unchecked --output /tmp/times --kind time --forks 4 --sizes 0,1,64,512 --batches 8 --target-ms 10
taskset -c 0 python3 examples/bench-c-collections/input-compare.py --before /tmp/checked --after /tmp/unchecked --output /tmp/aa --kind time --same-build before --forks 4 --cases handles,records,buffers,callback_many,fixed_borrow,fixed_move,callback_fixed,numbers,control --sizes 1,512 --batches 8 --target-ms 10
taskset -c 0 python3 examples/bench-c-collections/input-compare.py --before /tmp/checked --after /tmp/unchecked --output /tmp/large --kind time --forks 6 --cases text_large,bytes_large,buffers_large --sizes 64,512 --batches 12 --target-ms 20
taskset -c 0 python3 examples/bench-c-collections/input-compare.py --before /tmp/checked --after /tmp/unchecked --output /tmp/large-aa --kind time --same-build before --forks 4 --cases text_large,bytes_large,buffers_large --sizes 64,512 --batches 12 --target-ms 20
```

The adjacent JSON files retain compact per-case/process summaries and build
provenance. Raw samples and per-run stdout remain in the selected temporary output
directories and are intentionally omitted from this PR.
