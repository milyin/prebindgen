# C inline-owner collection inputs

Baseline: merged #824 at `4aa79f6c51842995e6a0cd1d98f170ac5d81140b`.
Implementation: `32ca5b650`; follow-up `0ebafd6c8` generates byte-for-byte
identical benchmark Rust and an identical native timing library (SHA-256 in
`provenance.json`). Both versions use the same new fixture, dependency lock,
compiler and C caller. Alias policies are matched between versions.

Checked C input arrays containing pointers to inline opaque owners previously
stored one native empty replacement and a construction closure per row. Rust now
checks the complete address group, rereads the stable C rows, prepares each empty
replacement immediately before taking that owner, and fills the final Rust Vec.
Each move updates its consumption receipt after acquisition. A replacement-factory
panic can leave a partially consumed row; Rust destroys its acquired values and
C retains the unconsumed suffix. No rollback is promised. By-value callback-return
owners, callback fields and nested outer rows keep their existing fallback.

`native` transfers one inline `NativePacket` per row. That native struct contains
an id, a three-byte Vec and a three-byte String, each with capacity 64. `native_mixed`
adds two owning opaque pointer handles around that packet. Native owners, handles,
C row storage and payload buffers are created outside timing/counting. Successful
conversion, source function, native destruction and enabled alias checks are
included. This is a complete-call C microbenchmark, not network throughput.
Separate C boundary tests cover exact original String/Vec buffer addresses and
capacity 4096, replacement/constructor panics, precise receipts, aliases, optional
poisoned fields and borrowed receiver conflicts. Allocation budgets cover sizes
0, 1, 8, 64, 512 and 4096 with both alias policies.

## Allocation requests at 512 rows

| Checks | Case | Allocation calls, before → after | Requested bytes, before → after |
|---|---|---:|---:|
| Enabled | native | 3 → 2 | 73,728 → 36,864 |
| Enabled | native_mixed | 3 → 2 | 106,496 → 61,440 |
| Disabled | native | 1 → 1 | 28,672 → 28,672 |
| Disabled | native_mixed | 1 → 1 | 36,864 → 36,864 |

Checked inputs remove one preparation allocation and save 72 bytes/row for native
owners or 88 bytes/row for mixed rows. Unchecked inputs already construct directly.
The primitive, handle and handle-plus-enum controls retain their allocation counts
and sizes. Every measured case has zero reallocations; empty arrays allocate zero
times. Payload allocations are retained without cloning or compaction. Requested
bytes count allocator requests, not peak memory or copied bytes.

## Timing

Each table entry is the median complete-call latency across six fresh processes
per version, pinned to CPU 1. Each process runs eight measured batches after
calibration and warmup. Process order alternates and cell order is shuffled.
The repeat ran after local compilation finished. `summary.json` retains both runs,
process ranges, CPU times, individual process aggregates and paired changes. The
paired column is the median of per-process after/before changes, which can differ
from the ratio of the two latency medians.

| Checked case | Rows | Before, µs | After, µs | Change | Paired median change |
|---|---:|---:|---:|---:|---:|
| native | 1 | 0.094 | 0.083 | -11.6% | -12.0% |
| native | 512 | 43.614 | 34.440 | -21.0% | -19.1% |
| native | 4096 | 336.269 | 271.437 | -19.3% | -19.5% |
| native_mixed | 1 | 0.123 | 0.105 | -14.7% | -14.5% |
| native_mixed | 512 | 79.482 | 73.213 | -7.9% | -8.7% |
| native_mixed | 4096 | 712.111 | 641.589 | -9.9% | -7.1% |

The initial checked run at 512 rows measured -15.9% for native and -10.2% for mixed
rows; the repeat supports an improvement without identifying a single fixed
speedup. Unchecked 512-row native inputs measured +0.1%, and mixed inputs +3.1%.
They have no allocation saving and no consistent latency improvement.

Controls at 512 rows include unfavorable results: checked handles measured +3.1%
initially and +2.5% in the repeat; checked handle-plus-enum rows measured +3.6%.
Unchecked handles measured +1.8% and enum rows +2.3%. A four-process identical
baseline-library control measured -0.1% for handles, +0.6% for native owners and
-2.4% for mixed rows. These small changes remain in the data rather than being
reported as optimization wins.

The primitive control rose from 208.4 to 254.9 ns (+22.3%) in the repeat, despite
unchanged allocation counts and conversion operations. As in the preceding enum
optimization, native code placement is a dominant cause. The identical 152-byte
native sum implementation starts at `0x29bd0` before and `0x28d30` after; its hot
loop at function +0x40 changes from offset 16 to offset 48 within a 64-byte line.
The latter straddles the line. The isolated same-process placement diagnostic
validates the instruction shape and allocator forwarding, reuses matching source
and destination addresses, and tests original and cloned placements. At 512 rows
with a streaming source pool, original placement measured +26.5%, whereas matching
both sum implementations at offset 0 measured -4.0%; at 4096 rows these are +22.0%
and -0.7%. Matching separate clones at offset 0 also removes the large gap.
`placement-summary.json` retains every tested offset and fork. These tests alter
only private diagnostic-process memory and add no production alignment policy.
The original driver caps total prepared rows, so its primitive batches can also
be much shorter than the requested 20 ms; the placement test uses 16,777,216
elements per batch to avoid that limitation.

## Reproduction

Commit the implementation and fixture before freezing. Use this checkout's
`build.py` for both generator checkouts. Produce checked/unchecked timing and
allocation builds with separate empty output directories:

```sh
python3 examples/bench-c-collections/build.py --checkout /path/to/baseline --output /tmp/inline-before-time
python3 examples/bench-c-collections/build.py --output /tmp/inline-after-time
# Add --count-allocations for count builds; add --disable-alias-checks for unchecked builds.
taskset -c 1 python3 examples/bench-c-collections/input-compare.py --before /tmp/inline-before-time --after /tmp/inline-after-time --output /tmp/inline-time --kind time --cases native,native_mixed,handles,numbers --sizes 1,512,4096 --forks 6 --batches 8
# Counts: --kind count --sizes 0,1,8,64,512,4096 --forks 1 --batches 3
# Identical-library control: append --same-build before, use --forks 4.
python3 examples/bench-c-collections/inline-input-results/summarize.py checked-time=/tmp/inline-time --output /tmp/inline-summary
python3 examples/bench-c-collections/enum-input-results/placement.py --before /tmp/inline-before-time --after /tmp/inline-after-time --output /tmp/inline-placement --cpu 1 --forks 4 --batches 6 --elements 16777216 --probe-allocations
python3 examples/bench-c-collections/enum-input-results/summarize-placement.py /tmp/inline-placement --output /tmp/inline-summary/placement-summary.json
```

`provenance.json` records build revisions, source/toolchain/ABI/artifact hashes,
process settings and CPU information. Only compact aggregates and provenance are
committed; raw samples and console logs stay outside Git. The exporter accepts
multiple named runs so the initial run, repeat, counts and identical-library
controls can be exported together without editing results by hand.
