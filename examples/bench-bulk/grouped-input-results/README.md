# PR 854: grouped validation versus per-value hashing

Compared with 9fe03d086 (previous PR revision). Identical fixtures and release compiler settings. Median of four process medians, each with nine measured batches. JNI uses six warmup batches, balanced revision/mode order, and no -Xcheck:jni. C handle creation is outside the timer; consuming decode, deallocation and Rust source drops remain timed.

| Boundary | Mode | Case | Elements/bytes | Previous ns/call | Updated ns/call | Previous / updated | Updated latency |
|---|---|---|---:|---:|---:|---:|---:|
| C | native | constructor | 0 | 14.9 | 8.3 | 1.795x | -44.3% |
| C | native | constructor | 1 | 30.4 | 25.5 | 1.189x | -15.9% |
| C | native | constructor | 8 | 47.3 | 42.2 | 1.122x | -10.9% |
| C | native | constructor | 64 | 206.0 | 166.8 | 1.235x | -19.0% |
| C | native | constructor | 512 | 2310.2 | 1235.6 | 1.870x | -46.5% |
| C | native | constructor | 4096 | 20800.9 | 14162.6 | 1.469x | -31.9% |
| C | native | consuming_handles | 0 | 19.5 | 24.4 | 0.797x | +25.5% |
| C | native | consuming_handles | 1 | 73.2 | 88.5 | 0.827x | +20.9% |
| C | native | consuming_handles | 8 | 263.7 | 229.7 | 1.148x | -12.9% |
| C | native | consuming_handles | 64 | 2636.9 | 1235.0 | 2.135x | -53.2% |
| C | native | consuming_handles | 512 | 19541.3 | 16501.4 | 1.184x | -15.6% |
| C | native | consuming_handles | 4096 | 165627.4 | 128206.1 | 1.292x | -22.6% |
| C | native | primitive | 0 | 15.0 | 8.9 | 1.685x | -40.7% |
| C | native | primitive | 1 | 30.3 | 24.8 | 1.220x | -18.0% |
| C | native | primitive | 8 | 35.9 | 28.2 | 1.272x | -21.4% |
| C | native | primitive | 64 | 94.8 | 53.5 | 1.771x | -43.5% |
| C | native | primitive | 512 | 539.0 | 308.6 | 1.746x | -42.7% |
| C | native | primitive | 4096 | 4185.0 | 2473.8 | 1.692x | -40.9% |
| JNI | bulk | callback_record_return | 1 | 1985.5 | 2000.4 | 0.993x | +0.8% |
| JNI | bulk | control | 1 | 13.7 | 13.6 | 1.008x | -0.8% |
| JNI | bulk | numbers_input | 0 | 152.8 | 162.3 | 0.941x | +6.2% |
| JNI | bulk | numbers_input | 1 | 209.6 | 208.7 | 1.004x | -0.4% |
| JNI | bulk | numbers_input | 8 | 264.2 | 257.2 | 1.027x | -2.6% |
| JNI | bulk | numbers_input | 64 | 613.4 | 549.4 | 1.116x | -10.4% |
| JNI | bulk | numbers_input | 512 | 3328.2 | 2625.8 | 1.267x | -21.1% |
| JNI | bulk | record_input | 1 | 179.8 | 175.5 | 1.024x | -2.4% |
| JNI | bulk | records_input | 0 | 165.3 | 146.3 | 1.130x | -11.5% |
| JNI | bulk | records_input | 1 | 239.8 | 227.6 | 1.054x | -5.1% |
| JNI | bulk | records_input | 8 | 452.6 | 445.9 | 1.015x | -1.5% |
| JNI | bulk | records_input | 64 | 1995.8 | 1850.4 | 1.079x | -7.3% |
| JNI | bulk | records_input | 512 | 15136.5 | 12425.7 | 1.218x | -17.9% |
| JNI | plain | callback_record_return | 1 | 9241.1 | 9048.1 | 1.021x | -2.1% |
| JNI | plain | control | 1 | 13.9 | 13.7 | 1.015x | -1.5% |
| JNI | plain | numbers_input | 0 | 99.1 | 104.2 | 0.951x | +5.2% |
| JNI | plain | numbers_input | 1 | 2044.7 | 2046.5 | 0.999x | +0.1% |
| JNI | plain | numbers_input | 8 | 14987.4 | 14527.8 | 1.032x | -3.1% |
| JNI | plain | numbers_input | 64 | 117699.8 | 114142.5 | 1.031x | -3.0% |
| JNI | plain | numbers_input | 512 | 917143.5 | 929264.1 | 0.987x | +1.3% |
| JNI | plain | record_input | 1 | 30.3 | 30.3 | 0.998x | +0.2% |
| JNI | plain | records_input | 0 | 94.0 | 92.0 | 1.023x | -2.2% |
| JNI | plain | records_input | 1 | 9924.5 | 9596.4 | 1.034x | -3.3% |
| JNI | plain | records_input | 8 | 78898.9 | 77007.5 | 1.025x | -2.4% |
| JNI | plain | records_input | 64 | 610691.4 | 614099.4 | 0.994x | +0.6% |
| JNI | plain | records_input | 512 | 4850468.5 | 4855157.5 | 0.999x | +0.1% |

Native C also remeasures ed9faf756, before the per-value hash-history change:

| Case | Rows | Original staged ns | Per-value hash ns | Grouped ns | Staged / grouped |
|---|---:|---:|---:|---:|---:|
| constructor | 0 | 14.4 | 14.9 | 8.3 | 1.737x |
| constructor | 1 | 31.5 | 30.4 | 25.5 | 1.233x |
| constructor | 8 | 52.7 | 47.3 | 42.2 | 1.249x |
| constructor | 64 | 219.8 | 206.0 | 166.8 | 1.318x |
| constructor | 512 | 1644.4 | 2310.2 | 1235.6 | 1.331x |
| constructor | 4096 | 16753.1 | 20800.9 | 14162.6 | 1.183x |
| consuming_handles | 0 | 30.9 | 19.5 | 24.4 | 1.267x |
| consuming_handles | 1 | 90.1 | 73.2 | 88.5 | 1.017x |
| consuming_handles | 8 | 309.3 | 263.7 | 229.7 | 1.347x |
| consuming_handles | 64 | 1481.3 | 2636.9 | 1235.0 | 1.199x |
| consuming_handles | 512 | 16516.5 | 19541.3 | 16501.4 | 1.001x |
| consuming_handles | 4096 | 127394.3 | 165627.4 | 128206.1 | 0.994x |
| primitive | 0 | 9.8 | 15.0 | 8.9 | 1.101x |
| primitive | 1 | 26.1 | 30.3 | 24.8 | 1.053x |
| primitive | 8 | 32.6 | 35.9 | 28.2 | 1.156x |
| primitive | 64 | 85.7 | 94.8 | 53.5 | 1.601x |
| primitive | 512 | 528.3 | 539.0 | 308.6 | 1.712x |
| primitive | 4096 | 4266.7 | 4185.0 | 2473.8 | 1.725x |

The grouped build is `f57fd50a1`; subsequent commits refresh generated review
artifacts only. JNI's previous prepared build records `34c33a5b2`, whose generator
matches `9fe03d086` (the latter only refreshed snapshots). All JNI fixture hashes,
Rust/Kotlin versions and dependency lock hashes match. The C collection benchmark
source is unchanged. These are local measurements, not universal speed guarantees.

The empty and one-handle C cases regress against the per-value implementation by
about 5 ns and 15 ns respectively. The grouped implementation keeps a prepared-row
buffer for consuming collections. Large C handle inputs return to approximately
the older staged implementation's throughput; ordinary object-row JNI remains
approximately unchanged. The full tables above include these cases.

`results.json` contains process medians and build metadata; `samples.csv` contains
all measured batches. `native.c` and `measure.py` are the exact runner sources used
for this comparison. Their absolute paths describe this run's prepared artifacts;
reproducing it requires rebuilding those revisions and adjusting the artifact
paths. Handle creation is outside the C timer; native validation, construction,
Box release, source-object destruction and ownership receipts are inside it.

Correctness runs are separate: the release JNI fixture passed all 41 cases in four
fresh JVMs with `-Xcheck:jni`. Those timings are excluded from this report.
