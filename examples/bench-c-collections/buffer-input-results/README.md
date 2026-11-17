# C buffer input rows: before / after

The same fixture from `931d25d2b` is built against #824 at `e75d23177` and
the revised generator at `931d25d2b`. Later hygiene changes add only generator
debug assertions and unused-variable attributes. Normalized C header declarations
match exactly. Each library is frozen independently; one identical C executable
loads both versions and verifies the loaded library path.

Flat rows containing scalars, opaque handle pointers and deferred text/byte
buffers validate the complete handle group, then construct directly in the final
Rust vector. Buffer decoding, UTF-8 checks, required owned copies and source
constructors run once. The prepared-row closure vector is removed. Native alias
history, the final Rust vector and required owned String/Vec payloads remain.

Handles and C rows/payloads are prepared before timing/counting. Measured work
includes native conversion, alias checks, construction, source computation,
Box deallocation and final Rust drops. Short payloads contain three bytes.
`_large` uses 4096-byte text (1365 Euro signs and an ASCII byte) and/or bytes.
These are binding microbenchmarks, not packet-throughput measurements.

## Native allocation counts

Two processes per revision/cell; ten counted calls per process after ten warmups.
All repeated measurements agree exactly; all reallocations are zero.

| Case | Rows | Allocations before → after | Requested bytes before → after |
| --- | ---: | ---: | ---: |
| buffers | 1 | 5 → 4 | 126 → 78 |
| buffers | 512 | 1027 → 1026 | 64,512 → 39,936 |
| buffers_large | 1 | 5 → 4 | 8,312 → 8,264 |
| buffers_large | 512 | 1027 → 1026 | 4,255,744 → 4,231,168 |
| bytes | 1 | 4 → 3 | 83 → 51 |
| bytes | 512 | 515 → 514 | 42,496 → 26,112 |
| bytes_large | 1 | 4 → 3 | 4,176 → 4,144 |
| bytes_large | 512 | 515 → 514 | 2,138,112 → 2,121,728 |
| control | 1 | 0 → 0 | 0 → 0 |
| text | 1 | 4 → 3 | 83 → 51 |
| text | 512 | 515 → 514 | 42,496 → 26,112 |
| text_large | 1 | 4 → 3 | 4,176 → 4,144 |
| text_large | 512 | 515 → 514 | 2,138,112 → 2,121,728 |

Each nonempty eligible buffer collection saves one allocation. At 512 rows,
temporary storage falls by 16 KiB for text or bytes and 24 KiB for both. Counts
also cover 0, 64 and 4096 rows; empty inputs and handle/numeric controls keep
their previous allocation counts.

## Complete-call latency

Release Rust, GCC C11 `-O3 -fno-plt`, two available CPUs, no concurrent builds/tests.
Four fresh processes per revision/cell; shuffled cases and alternating revision
order. Three warmup batches follow calibration; twelve measurement batches
target 10 ms, bounded to 262144 prepared rows. Each process score is total wall
time / calls; the table shows the median of four process scores. Ranges and all
process scores, CPU times, hashes and tool versions are in `summary.json`.

| Case | Rows | Before ns/call | After ns/call | Change |
| --- | ---: | ---: | ---: | ---: |
| buffers | 1 | 125.2 | 106.6 | -14.9% |
| buffers | 512 | 52,507.1 | 46,223.5 | -12.0% |
| buffers_large | 1 | 3,202.6 | 3,255.4 | +1.6% |
| buffers_large | 512 | 4,224,306.3 | 4,380,726.3 | +3.7% |
| bytes | 1 | 98.6 | 88.0 | -10.7% |
| bytes | 512 | 23,258.3 | 21,987.5 | -5.5% |
| bytes_large | 1 | 222.7 | 196.2 | -11.9% |
| bytes_large | 512 | 1,383,404.1 | 1,337,956.1 | -3.3% |
| control | 1 | 4.3 | 4.4 | +0.6% |
| text | 1 | 105.4 | 92.9 | -11.9% |
| text | 512 | 25,231.6 | 25,117.3 | -0.5% |
| text_large | 1 | 3,318.3 | 3,026.7 | -8.8% |
| text_large | 512 | 2,983,358.4 | 3,051,940.0 | +2.3% |

Short combined-buffer rows improve 12.0% at 512 rows and 14.9% at one row.
The first large-payload runs are mixed: text and combined payload medians rise
2.3% and 3.7% at 512 rows, while bytes fall 3.3%. These regressions remain in
the results; no universal speedup is claimed.

## Identical-library control and large-payload confirmation

The identical-library run uses the updated library on both sides, with four
processes and the same 12 × 10 ms policy. At 512 rows, median shifts are −0.1%
for short text, +0.1% for combined short buffers, −1.2% for large text and −1.7%
for large combined buffers. The tiny scalar control shifts +5.2% (about 0.2 ns).

A focused confirmation uses four processes per side, 16 batches targeting 20 ms:

| Case | Rows | Before ns/call | After ns/call | Change |
| --- | ---: | ---: | ---: | ---: |
| buffers_large | 512 | 4,312,279.8 | 4,252,627.6 | -1.4% |
| bytes_large | 512 | 1,349,906.8 | 1,344,205.2 | -0.4% |
| control | 1 | 4.3 | 4.3 | -0.2% |
| text_large | 512 | 2,832,571.8 | 2,826,086.0 | -0.2% |

The initial large-payload regressions are not consistent across runs. Treat large
payload timing as inconclusive. The allocation reduction is deterministic.

## Reproduce

Use the identical fixture to build separate before/after libraries with `build.py`
`--checkout`, `--output` and `--target-dir`. Build both counting variants using
`--count-allocations` and both release timing variants without it. Repositories
must be committed and clean. Finish all builds/tests before timing.

```sh
python3 examples/bench-c-collections/input-compare.py \
  --before /tmp/before-counts --after /tmp/after-counts --output /tmp/counts \
  --kind count --forks 2 --sizes 0,1,64,512,4096
python3 examples/bench-c-collections/input-compare.py \
  --before /tmp/before-time --after /tmp/after-time --output /tmp/time \
  --kind time --forks 4 --sizes 0,1,64,512 --batches 12 --target-ms 10
```

Measurements selected `text,bytes,buffers,text_large,bytes_large,buffers_large,handles,numbers,control`.
The identical-build run adds `--same-build after --cases text,text_large,buffers,buffers_large,control --sizes 512`.
Confirmation selects `text_large,bytes_large,buffers_large,control --sizes 512 --batches 16 --target-ms 20`.
Console logs and raw sample files remain outside Git.
