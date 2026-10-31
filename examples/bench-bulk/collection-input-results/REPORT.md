# Kotlin collection input helper removal

Compared with #824 after #854, using identical new handle-collection fixtures: baseline `6cda6fe42da16bbc7c9257f7eeca56838c7824d9` versus updated `d45e560aadc9b35f059c07998a6f3db4eb191db1`. Later commits record generated artifacts, documentation and these results, plus a generation-time string assembly adjustment for Rust 1.85 Clippy. They do not change the generated runtime or measured fixture.

Measured 2026-10-31 on Linux x86-64, AMD EPYC 7713, CPUs 0–1, Rust 1.98.0 release (optimization level 3, debug assertions disabled), Kotlin 2.4.21 and OpenJDK 11.0.29. No local builds or other benchmark processes ran during timing. Both revisions first passed all 56 fixture cases in four diagnostic JVMs each with `-Xcheck:jni`; diagnostic timings are excluded here.

The table reports the median of four JVM medians for each revision and mode. Each JVM runs six warmup batches and nine measured batches targeting 20 ms. Four balanced orders rotate before/after and ordinary/bulk treatments; each process also reverses/rotates case order. The fixed 256 MiB G1 heap uses two active JVM processors. Calibration, source list setup and initial handle creation are outside timing. Normal conversion, locks, ownership acknowledgement, Rust source work and output reconstruction remain timed. Large ordinary cases may exceed the batch target because calibration has a minimum of 16 calls; all samples are retained.

`plain` means ordinary JNI object-array collection transport; `bulk` means an explicit packed byte-buffer expansion. Ratios divide **before by after within the same mode**: above 1 means this PR is faster. They do not compare ordinary with bulk. Allocation is calling-thread JVM bytes per call, excluding Rust/native allocations, worker threads and retained input values. Timings include the cost of those native allocations. These are single-host boundary microbenchmarks, not network throughput.

Borrowed handles are reused. The handle-record fixture explicitly constructs owned Rust records by copying their scalar payload from borrowed handles; it does not allocate an extra native object per record. The consuming round trip passes the previous call’s returned handle owners into the next call. Its timer also includes one extra scalar JNI call to check the first handle. The empty case has no such read. `control` and single `record_input` help expose timing variation in paths without collection input preparation.

## Interpretation

- Bulk borrowed-handle inputs improve from 4,087 to 2,244 ns at 64 elements (1.82×) and from 28,421 to 14,853 ns at 512 elements (1.91×). JVM allocation falls from 12,400 to 6,184 B/call and from 96,200 to 45,232 B/call respectively.
- Bulk handle-record inputs improve 1.66× at 64 elements and 1.72× at 512. The consuming bulk handle round trip improves 1.20× and 1.22×, including output reconstruction and the extra scalar read.
- Ordinary JNI primitive inputs halve JVM allocation at 64/512 elements, but latency is nearly unchanged or slightly worse: native per-field JNI work still dominates. Ordinary handle input allocation falls by about 43–44% at those sizes. Removing helpers does not remove the row arrays or boxed wire values required by ordinary transport.
- Latency is not uniformly better. Ordinary 64-handle input is 5.3% slower by the aggregate medians, with overlapping process ranges (108,510–113,234 ns before; 106,766–150,176 ns after). Ordinary 512-number input is 2.8% slower, ordinary 64-record input is 2.4% slower, and bulk 64-record input is 2.1% slower. All samples, including outlying process medians, remain in the report. These results support allocation reductions and bulk handle-input speedups, not an across-the-board latency improvement.
- Bulk primitive/record allocation is unchanged: those paths already wrote directly without per-element helper objects. Their timing changes also include moving packing into setup; primitive-only paths gain no handle metadata. Empty ordinary primitive/record input allocation falls from 40 to 16 B/call; empty bulk handle inputs fall from 192 to 144 B/call.
- Unchanged scalar `control` timings are effectively flat for ordinary JNI (0.0% change) and 1.4% slower for bulk's identical scalar control, with overlapping ranges. Calling-thread allocation in consuming round trips varies across JVMs as output reconstruction is optimized by the JIT; the report retains process timing medians and raw allocation samples.

## Full results

Latency is nanoseconds per call; brackets show the range of the four JVM medians. JVM bytes are before / after.

| Mode | Case | Elements | Before ns [range] | After ns [range] | Before / after | JVM B/call |
|---|---|---:|---:|---:|---:|---:|
| bulk | control | 1 | 14.3 [14.1–19.1] | 14.5 [13.8–16.2] | 0.986× | 0.0 / 0.0 |
| bulk | handle_records_input | 0 | 211.4 [204.9–232.9] | 187.5 [183.0–201.7] | 1.127× | 192.0 / 144.0 |
| bulk | handle_records_input | 1 | 399.9 [377.2–418.7] | 288.1 [279.8–290.9] | 1.388× | 552.0 / 440.0 |
| bulk | handle_records_input | 8 | 879.0 [862.4–1048.7] | 612.6 [605.5–630.4] | 1.435× | 1808.0 / 1032.0 |
| bulk | handle_records_input | 64 | 4274.6 [4052.0–4296.4] | 2582.3 [2555.0–2628.2] | 1.655× | 13024.0 / 6696.0 |
| bulk | handle_records_input | 512 | 30596.1 [29228.6–31044.4] | 17773.9 [17324.2–17869.3] | 1.721× | 100408.0 / 49328.0 |
| bulk | handles_input | 0 | 212.5 [203.2–240.4] | 185.2 [174.2–197.8] | 1.147× | 192.0 / 144.0 |
| bulk | handles_input | 1 | 367.7 [350.8–452.3] | 287.6 [266.9–307.6] | 1.279× | 552.0 / 440.0 |
| bulk | handles_input | 8 | 781.3 [734.2–797.4] | 520.0 [509.5–528.4] | 1.503× | 1728.0 / 968.0 |
| bulk | handles_input | 64 | 4086.7 [3863.7–4354.5] | 2244.4 [2175.8–2337.0] | 1.821× | 12400.0 / 6184.0 |
| bulk | handles_input | 512 | 28421.1 [26464.0–29185.8] | 14853.0 [14494.4–15137.6] | 1.913× | 96200.0 / 45232.0 |
| bulk | numbers_input | 0 | 149.6 [130.8–161.2] | 137.4 [133.8–148.8] | 1.089× | 80.0 / 80.0 |
| bulk | numbers_input | 1 | 197.0 [184.2–224.3] | 197.3 [193.9–199.4] | 0.998× | 224.0 / 224.0 |
| bulk | numbers_input | 8 | 243.3 [226.9–250.7] | 248.9 [228.5–289.0] | 0.977× | 328.0 / 328.0 |
| bulk | numbers_input | 64 | 582.8 [559.9–591.6] | 520.9 [511.6–533.7] | 1.119× | 1000.0 / 1000.0 |
| bulk | numbers_input | 512 | 2984.4 [2946.1–3050.6] | 2512.9 [2486.8–2718.3] | 1.188× | 6376.0 / 6376.0 |
| bulk | owned_handles_roundtrip | 0 | 537.0 [506.1–618.8] | 494.8 [485.3–498.6] | 1.085× | 416.0 / 360.0 |
| bulk | owned_handles_roundtrip | 1 | 1003.1 [911.5–1051.1] | 918.9 [908.9–985.5] | 1.092× | 928.0 / 904.0 |
| bulk | owned_handles_roundtrip | 8 | 1968.0 [1770.7–2123.4] | 1691.1 [1608.0–1726.1] | 1.164× | 3104.0 / 2280.0 |
| bulk | owned_handles_roundtrip | 64 | 10888.1 [10472.9–13186.9] | 9064.7 [8455.5–10824.9] | 1.201× | 24849.7 / 19838.3 |
| bulk | owned_handles_roundtrip | 512 | 83620.8 [82222.2–306385.9] | 68539.0 [66604.3–105752.2] | 1.220× | 175865.0 / 125876.3 |
| bulk | record_input | 1 | 142.8 [134.2–230.9] | 124.2 [122.0–126.5] | 1.149× | 128.0 / 128.0 |
| bulk | records_input | 0 | 148.6 [143.3–160.3] | 139.7 [130.9–148.6] | 1.064× | 80.0 / 80.0 |
| bulk | records_input | 1 | 247.0 [219.0–253.3] | 247.1 [213.6–273.2] | 1.000× | 272.0 / 272.0 |
| bulk | records_input | 8 | 476.6 [434.4–505.4] | 465.9 [453.0–492.4] | 1.023× | 776.0 / 776.0 |
| bulk | records_input | 64 | 1923.4 [1691.6–1976.0] | 1964.2 [1859.5–2048.5] | 0.979× | 4584.0 / 4584.0 |
| bulk | records_input | 512 | 13754.8 [11779.1–15028.9] | 13461.8 [13359.7–14645.2] | 1.022× | 35048.0 / 35048.0 |
| plain | control | 1 | 14.8 [13.8–15.9] | 14.8 [13.7–18.2] | 1.000× | 0.0 / 0.0 |
| plain | handle_records_input | 0 | 126.1 [116.2–143.1] | 70.5 [67.7–71.9] | 1.789× | 128.0 / 96.0 |
| plain | handle_records_input | 1 | 3113.2 [3067.3–3181.3] | 3092.2 [2952.3–3724.4] | 1.007× | 488.0 / 280.0 |
| plain | handle_records_input | 8 | 22776.6 [22552.8–25810.7] | 23103.8 [22176.5–23814.1] | 0.986× | 1988.0 / 1112.0 |
| plain | handle_records_input | 64 | 179827.4 [177346.1–185843.0] | 180347.6 [176343.4–272186.7] | 0.997× | 14548.0 / 8296.0 |
| plain | handle_records_input | 512 | 1440063.1 [1418005.9–1482072.8] | 1426281.6 [1408295.9–1673124.6] | 1.010× | 121900.0 / 72640.0 |
| plain | handles_input | 0 | 121.2 [114.3–174.0] | 64.9 [56.2–83.3] | 1.867× | 128.0 / 96.0 |
| plain | handles_input | 1 | 2004.9 [1975.3–2048.8] | 1930.1 [1860.6–1948.9] | 1.039× | 488.0 / 280.0 |
| plain | handles_input | 8 | 14138.3 [13812.8–14275.5] | 13888.9 [13702.6–14074.8] | 1.018× | 2000.0 / 1112.0 |
| plain | handles_input | 64 | 109152.1 [108510.1–113233.7] | 114914.3 [106765.6–150176.2] | 0.950× | 14560.0 / 8296.0 |
| plain | handles_input | 512 | 933208.2 [870465.8–1003146.2] | 866378.8 [850906.3–1580733.3] | 1.077× | 112696.0 / 63424.0 |
| plain | numbers_input | 0 | 103.6 [98.3–105.3] | 44.0 [41.5–53.7] | 2.355× | 40.0 / 16.0 |
| plain | numbers_input | 1 | 1956.0 [1858.1–3117.7] | 1782.1 [1757.2–1825.5] | 1.098× | 256.0 / 96.0 |
| plain | numbers_input | 8 | 13470.4 [13429.5–14670.5] | 13596.0 [13279.0–13873.7] | 0.991× | 1056.0 / 504.0 |
| plain | numbers_input | 64 | 106562.9 [104933.7–107962.4] | 106479.0 [104745.1–109478.2] | 1.001× | 7328.0 / 3640.0 |
| plain | numbers_input | 512 | 851254.1 [844096.4–878138.9] | 874966.6 [832714.5–940542.5] | 0.973× | 57504.0 / 28728.0 |
| plain | owned_handles_roundtrip | 0 | 695.3 [685.9–711.7] | 652.2 [637.3–676.0] | 1.066× | 192.0 / 184.0 |
| plain | owned_handles_roundtrip | 1 | 4825.9 [4761.8–5393.7] | 4836.3 [4789.1–5007.5] | 0.998× | 796.0 / 696.0 |
| plain | owned_handles_roundtrip | 8 | 31050.1 [30383.2–31666.7] | 31324.5 [30218.8–32964.3] | 0.991× | 3426.0 / 2821.2 |
| plain | owned_handles_roundtrip | 64 | 241975.1 [237751.6–246122.2] | 241068.2 [235481.2–255410.1] | 1.004× | 31281.2 / 28684.9 |
| plain | owned_handles_roundtrip | 512 | 1924077.2 [1873502.5–2113865.4] | 1892650.9 [1866431.2–1928996.2] | 1.017× | 214898.0 / 168870.0 |
| plain | record_input | 1 | 32.1 [29.6–33.2] | 31.2 [29.2–32.3] | 1.029× | 0.0 / 0.0 |
| plain | records_input | 0 | 97.7 [91.0–175.6] | 42.8 [38.6–51.3] | 2.286× | 40.0 / 16.0 |
| plain | records_input | 1 | 9439.1 [9385.2–23026.6] | 9125.3 [9090.5–9308.4] | 1.034× | 440.0 / 288.0 |
| plain | records_input | 8 | 74197.2 [73603.7–75387.1] | 73721.9 [71743.7–73806.8] | 1.006× | 2528.0 / 2040.0 |
| plain | records_input | 64 | 598417.7 [588850.3–606398.7] | 612961.2 [572732.6–682151.4] | 0.976× | 19104.0 / 15928.0 |
| plain | records_input | 512 | 4716663.0 [4713325.0–4822072.4] | 4673815.5 [4561658.8–4689631.9] | 1.009× | 151712.0 / 127032.0 |

## Reproduce

Build each recorded revision into a separate empty directory using `examples/bench-bulk/run.py --build-only` and the same Kotlin compiler. Those revisions have identical fixture sources. Use the prepared directories below; `compare.py` verifies fixture, compiler, lock, build-profile and artifact hashes before timing, and rechecks JAR/native hashes afterwards.

```sh
taskset -c 0,1 python3 examples/bench-bulk/collection-input-results/compare.py \
  --before /tmp/collection-before --after /tmp/collection-after \
  --output /tmp/collection-comparison
```

Choose a permitted CPU pair if CPUs 0–1 are unavailable. On systems without
Linux `taskset`, use the platform's affinity control or omit it and record the
different placement. `ActiveProcessorCount=2` does not pin CPU placement.

[results.json](results.json) records all process medians, exact build revisions, compiler/profile information, affinity, process commands and artifact hashes. [samples.csv](samples.csv) preserves all 3,888 measured batches. The compared builds used clean, committed source.
