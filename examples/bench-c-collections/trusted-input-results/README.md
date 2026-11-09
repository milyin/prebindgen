# Trusted C inputs

The baseline is #824 at `ab57d2050`; the measured implementation is
`64d77e68c`. Later source edits preserve that decoder: the borrowed-slice
experiment was reverted, and subsequent changes record generated files and results.
Both versions use the same current fixture and C caller. Inputs satisfy both
contracts. The baseline validates them; the implementation trusts the C caller.

C prepares source handles/rows outside timing. Measurements include Rust decoding,
construction, the source function and destruction. Allocation counting uses a
separate instrumented library. Counts cover two forks and three batches; latency
covers four forks and twelve calibrated 20 ms batches on CPU 0. There were no
other builds or tests during the main A/B measurements. These are synthetic
conversion costs, not packet-throughput measurements.

| Case, 512 rows | Baseline µs | Trusted µs | Change |
| --- | ---: | ---: | ---: |
| Owned handles | 15.154 | 11.844 | −21.8% |
| Two-handle records | 31.505 | 25.041 | −20.5% |
| Handle plus text/bytes | 49.819 | 45.399 | −8.9% |
| Callback-returned handles | 15.228 | 12.432 | −18.4% |
| Primitive-vector control | 0.255 | 0.341 | +33.8% |

All nonempty handle, record and callback-array cases remove one allocation: the
address-validation buffer. At 512 rows this saves 8 KiB for single-handle rows
and 16 KiB for two-handle records. Primitive arrays retain their one final vector
allocation. Text/byte ownership conversions retain their required per-row copies.
Fixed callback results and fixed borrowed parameters allocate zero times in both
versions. The fixed mutable/shared pair measured 11.23 → 2.68 ns; this is a tiny
microbenchmark and should not be extrapolated to complete application calls.

## Primitive control and investigation

Primitive timing is mixed and is a limitation of this report. The initial direct
row decoder reloaded the descriptor pointer within the copy loop. Disassembly
showed scalar copying where the baseline used SIMD. Capturing the pointer once
restores SIMD without adding checks, allocations or copies. A borrowed-slice
alternative emitted the same hot loop and was discarded.

The final A/B session still measures a slower primitive control: +33.8% at 512
rows and +14.4% at 8192. However, an identical-library control using the same
baseline-built C executable measures the candidate at 250/239 ns for 512 rows,
versus 341 ns in the A/B session. At 8192 rows that identical-library comparison
varies by +13.0%. These controls do not establish a universal latency improvement
or a resolved cause for the remaining primitive timing differences. No validation
or allocation was reintroduced to tune the control. The unchanged scalar call
measures 4.62 → 5.02 ns in A/B and 4.44 → 4.48 ns in the fixed-caller A/A run.

The input runner now holds the baseline-built C executable fixed even when
`--same-build after` selects the candidate library for both columns. Previously
that option also switched callers, weakening comparisons between A/B and A/A.
All A/B results above already use the baseline-built caller; the committed A/A
results use the corrected runner. No raw console logs are committed.

## Reproduce

Use clean checkouts of the two revisions and the same current fixture. Freeze
both uninstrumented libraries with `build.py --checkout <path> --output <dir>`;
repeat with `--count-allocations` into separate directories. Finish builds and
checks before timing.

```sh
taskset -c 0 python3 examples/bench-c-collections/input-compare.py \
  --before /tmp/before --after /tmp/after --output /tmp/comparison \
  --kind time --forks 4 --sizes 1,64,512,8192 \
  --cases handles,records,buffers,callback_many,numbers,control,fixed_borrow,callback_fixed
# Repeat with --same-build after and a different output directory.
```

`latency.csv`, `allocations.csv`, `identical-library.csv` and `provenance.json`
retain all measured sizes, per-fork changes, compiler/fixture identities and
frozen-library hashes. Native reallocations are zero in these counted cases.
