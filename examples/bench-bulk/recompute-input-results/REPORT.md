# Bulk input: saved offsets versus recomputation

This compares the generated saved-offset implementation at `d45e560aadc9b35f059c07998a6f3db4eb191db1` with generated schema-based recomputation at `596e433a994310a5592adabd4c4c8668e61abcf3`. Both already remove per-row Kotlin helper objects. This is **not** a comparison with #824 or `main`. Subsequent commits record snapshots, correct a boundary-test expectation for consumed inputs, and add this report; the measured generator and benchmark sources are unchanged.

Recomputation replaces the growing per-handle offset array with generated traversal of the packed frame. Kotlin retains the wrappers, locks them using the existing mechanism, then reads current native addresses and fills the recomputed positions. Null markers and absent nested collections preserve the handle order. No strategy switch or per-call liveness check was added.

Measured 2026-11-01 on Linux x86-64, AMD EPYC 7713, affinity CPUs 0–1, Rust 1.98.0 release optimization level 3, Kotlin 2.4.21, OpenJDK 11.0.29. Four balanced rounds run each revision in ordinary JNI object-array (`plain`) and packed byte-buffer (`bulk`) modes: 16 fresh JVMs, 6 warmup batches and 9 measured batches per case, targeting 20 ms per batch. JVMs use a fixed 256 MiB G1 heap and two active processors. No local builds or other benchmarks ran during timing. All 3,888 measured batches are retained. The updated build also passed all 56 benchmark cases in four separate diagnostic JVMs with `-Xcheck:jni`; diagnostic timings are excluded.

Latency is the median of four JVM medians. Ratios divide **saved-offset latency by recompute latency within the same mode**; above 1 means recompute is faster. Brackets show the range of JVM medians, not confidence intervals. Allocation is calling-thread JVM bytes per call, excluding Rust/native allocations, other threads and retained source values. Timings include native work and allocations. Source list setup and initial handle creation are outside timing; packing, locking, conversion, ownership acknowledgement and output reconstruction remain timed. Consuming round trips include an extra scalar JNI read to verify the first result. These are single-host boundary microbenchmarks, not network throughput.

## Findings

- Borrowed bulk handle inputs at 64/512 elements improve from 2,438/16,752 ns to 2,196/14,672 ns (10.0%/12.4% lower latency). Allocation falls by 552/4,184 B per call, from 6,184/45,232 to 5,632/41,048 B.
- Bulk handle-record inputs at 64/512 elements improve 6.8%/7.9%, with the same 552/4,184 B savings. One-handle inputs save 56 B; empty handle frames save 8 B and skip the patch traversal.
- Consuming bulk round trips are not uniformly faster: 8 elements are 10.2% slower, 64 are 4.5% faster, and 512 are effectively flat (0.4% faster). Output allocation varies with JVM optimization and is not a direct measure of removed offset storage.
- Primitive bulk input allocation is unchanged. Its 64/512-element medians are 6.7%/8.3% slower despite having no handle patch traversal. Ordinary transport also has no recompute path; its scalar control is 13.2% slower in this run, while bulk's identical scalar control is 0.4% faster. Process variation/JIT effects remain material; these results establish the metadata-allocation saving and favorable borrowed-handle medians, not a universal latency improvement.
- Nullable/nested correctness is covered by generated Kotlin boundary tests, not by these flat-array timing fixtures. No concurrency or tail-latency claim follows from this benchmark.

## Full results

| Mode | Case | Elements | Saved offsets ns [range] | Recompute ns [range] | Saved / recompute | JVM B/call saved / recompute |
|---|---|---:|---:|---:|---:|---:|
| bulk | control | 1 | 13.7 [13.2–19.9] | 13.6 [13.2–14.4] | 1.004× | 0.0 / 0.0 |
| bulk | handle_records_input | 0 | 190.0 [179.8–193.7] | 157.1 [150.4–173.1] | 1.209× | 144.0 / 136.0 |
| bulk | handle_records_input | 1 | 283.0 [268.2–288.7] | 277.6 [261.9–279.9] | 1.020× | 440.0 / 384.0 |
| bulk | handle_records_input | 8 | 625.1 [608.7–702.5] | 611.7 [592.2–641.1] | 1.022× | 1032.0 / 976.0 |
| bulk | handle_records_input | 64 | 2635.8 [2473.6–2924.8] | 2457.3 [2234.7–2698.2] | 1.073× | 6696.0 / 6144.0 |
| bulk | handle_records_input | 512 | 18967.5 [17678.8–20580.5] | 17467.5 [15661.2–18593.1] | 1.086× | 49328.0 / 45144.0 |
| bulk | handles_input | 0 | 203.3 [179.8–211.5] | 172.0 [146.9–195.6] | 1.182× | 144.0 / 136.0 |
| bulk | handles_input | 1 | 277.7 [268.6–300.7] | 276.6 [265.9–290.2] | 1.004× | 440.0 / 384.0 |
| bulk | handles_input | 8 | 531.8 [505.1–589.6] | 496.9 [468.6–520.7] | 1.070× | 968.0 / 912.0 |
| bulk | handles_input | 64 | 2438.3 [2162.1–2599.9] | 2195.6 [1909.9–2215.5] | 1.111× | 6184.0 / 5632.0 |
| bulk | handles_input | 512 | 16752.2 [14138.9–17844.8] | 14672.1 [13014.0–15468.9] | 1.142× | 45232.0 / 41048.0 |
| bulk | numbers_input | 0 | 148.1 [131.9–175.5] | 134.4 [128.7–157.7] | 1.102× | 80.0 / 80.0 |
| bulk | numbers_input | 1 | 212.5 [181.0–230.0] | 203.9 [195.0–224.4] | 1.042× | 224.0 / 224.0 |
| bulk | numbers_input | 8 | 251.1 [239.2–266.9] | 246.4 [221.2–265.5] | 1.019× | 328.0 / 328.0 |
| bulk | numbers_input | 64 | 527.4 [485.8–620.7] | 562.6 [543.7–618.5] | 0.937× | 1000.0 / 1000.0 |
| bulk | numbers_input | 512 | 2687.6 [2319.0–2976.1] | 2910.3 [2774.2–3005.2] | 0.923× | 6376.0 / 6376.0 |
| bulk | owned_handles_roundtrip | 0 | 504.8 [491.1–515.3] | 471.0 [469.1–485.6] | 1.072× | 360.0 / 352.0 |
| bulk | owned_handles_roundtrip | 1 | 928.9 [849.4–975.3] | 941.0 [915.1–1068.3] | 0.987× | 904.0 / 848.0 |
| bulk | owned_handles_roundtrip | 8 | 1640.5 [1518.5–2155.7] | 1808.2 [1692.4–1912.6] | 0.907× | 2280.0 / 2380.0 |
| bulk | owned_handles_roundtrip | 64 | 9249.3 [9020.8–14992.4] | 8836.0 [8556.1–13545.5] | 1.047× | 18933.2 / 18307.4 |
| bulk | owned_handles_roundtrip | 512 | 71307.1 [69338.9–95608.3] | 71017.3 [66307.8–102795.7] | 1.004× | 135245.3 / 125777.4 |
| bulk | record_input | 1 | 145.9 [128.8–195.4] | 142.4 [131.5–159.1] | 1.024× | 128.0 / 128.0 |
| bulk | records_input | 0 | 158.8 [140.6–166.2] | 146.6 [134.9–148.1] | 1.083× | 80.0 / 80.0 |
| bulk | records_input | 1 | 246.2 [220.6–260.6] | 239.5 [222.2–250.1] | 1.028× | 272.0 / 272.0 |
| bulk | records_input | 8 | 465.5 [454.7–494.4] | 455.7 [451.9–519.2] | 1.021× | 776.0 / 776.0 |
| bulk | records_input | 64 | 1929.5 [1872.8–1982.2] | 1886.2 [1860.0–1905.9] | 1.023× | 4584.0 / 4584.0 |
| bulk | records_input | 512 | 13696.2 [13499.0–13826.0] | 13367.4 [13199.4–13971.0] | 1.025× | 35048.0 / 35048.0 |
| plain | control | 1 | 15.7 [14.0–16.9] | 17.8 [13.2–22.8] | 0.883× | 0.0 / 0.0 |
| plain | handle_records_input | 0 | 65.5 [61.7–73.9] | 67.4 [66.2–70.8] | 0.972× | 96.0 / 96.0 |
| plain | handle_records_input | 1 | 2981.3 [2915.6–3049.1] | 3147.1 [2938.2–3449.2] | 0.947× | 280.0 / 280.0 |
| plain | handle_records_input | 8 | 22762.4 [22229.6–23708.1] | 22455.8 [22376.9–23007.2] | 1.014× | 1112.0 / 1112.0 |
| plain | handle_records_input | 64 | 181236.2 [177299.1–183392.7] | 178375.4 [176775.3–185943.7] | 1.016× | 8296.0 / 8296.0 |
| plain | handle_records_input | 512 | 1440275.4 [1407988.9–2409172.3] | 1454415.8 [1418475.8–1494862.6] | 0.990× | 72640.0 / 72640.0 |
| plain | handles_input | 0 | 64.4 [62.5–67.2] | 64.5 [61.8–68.4] | 0.999× | 96.0 / 96.0 |
| plain | handles_input | 1 | 1910.6 [1869.2–1981.0] | 2023.5 [1922.5–2089.7] | 0.944× | 280.0 / 280.0 |
| plain | handles_input | 8 | 13996.5 [13514.7–14309.6] | 13740.6 [13566.4–15406.7] | 1.019× | 1112.0 / 1112.0 |
| plain | handles_input | 64 | 110004.0 [108233.1–112725.6] | 107463.0 [107011.2–110445.7] | 1.024× | 8296.0 / 8296.0 |
| plain | handles_input | 512 | 866257.2 [848698.4–877729.3] | 856971.7 [855843.2–893178.3] | 1.011× | 63424.0 / 63424.0 |
| plain | numbers_input | 0 | 44.7 [41.2–53.9] | 47.1 [42.5–48.9] | 0.949× | 16.0 / 16.0 |
| plain | numbers_input | 1 | 1802.0 [1751.8–1845.4] | 1766.2 [1765.4–1848.4] | 1.020× | 96.0 / 96.0 |
| plain | numbers_input | 8 | 13614.7 [13213.9–13787.4] | 13641.9 [13242.8–15291.7] | 0.998× | 504.0 / 504.0 |
| plain | numbers_input | 64 | 107152.2 [103929.2–109533.6] | 108578.2 [105179.2–115800.1] | 0.987× | 3640.0 / 3640.0 |
| plain | numbers_input | 512 | 848176.9 [833215.4–859945.3] | 872561.5 [839997.1–950647.3] | 0.972× | 28728.0 / 28728.0 |
| plain | owned_handles_roundtrip | 0 | 639.3 [630.9–649.4] | 627.3 [616.5–628.2] | 1.019× | 184.0 / 184.0 |
| plain | owned_handles_roundtrip | 1 | 4701.7 [4639.9–4738.2] | 4713.8 [4654.1–5183.9] | 0.997× | 696.0 / 696.0 |
| plain | owned_handles_roundtrip | 8 | 31046.8 [30013.3–32436.2] | 30947.5 [30197.0–32046.6] | 1.003× | 2584.0 / 2562.0 |
| plain | owned_handles_roundtrip | 64 | 249458.9 [238129.1–267508.3] | 242025.9 [238099.2–253858.8] | 1.031× | 24591.3 / 24452.7 |
| plain | owned_handles_roundtrip | 512 | 1897716.8 [1847555.7–1930694.4] | 1942054.7 [1884636.3–4220411.6] | 0.977× | 162316.5 / 162333.5 |
| plain | record_input | 1 | 31.1 [30.4–31.6] | 31.7 [29.5–34.1] | 0.982× | 0.0 / 0.0 |
| plain | records_input | 0 | 41.5 [40.1–43.0] | 41.3 [39.2–44.0] | 1.004× | 16.0 / 16.0 |
| plain | records_input | 1 | 9217.7 [9056.0–9545.5] | 9261.8 [9185.4–9819.5] | 0.995× | 288.0 / 288.0 |
| plain | records_input | 8 | 73255.8 [71652.4–75252.9] | 73906.6 [72530.7–81274.0] | 0.991× | 2040.0 / 2040.0 |
| plain | records_input | 64 | 587014.9 [578444.8–591941.9] | 600437.0 [580622.2–638968.8] | 0.978× | 15928.0 / 15928.0 |
| plain | records_input | 512 | 4773263.1 [4688628.9–4808930.2] | 4659503.3 [4634187.9–4878714.7] | 1.024× | 127032.0 / 127032.0 |

## Reproduce

Build the two recorded revisions into separate empty directories with `examples/bench-bulk/run.py --build-only` and the same Kotlin compiler. Their fixture hashes are identical. Then use the existing comparison runner:

```sh
taskset -c 0,1 python3 examples/bench-bulk/collection-input-results/compare.py \
  --before /tmp/saved-offsets --after /tmp/recompute \
  --output /tmp/recompute-comparison
```

Choose permitted CPUs or record different placement on another machine. The runner verifies fixture, compiler, lock, profile and artifact hashes before timing and rechecks artifacts afterwards. [results.json](results.json) records exact revisions, commands, process medians, paired ratios and environment; [samples.csv](samples.csv) retains every measured batch.
