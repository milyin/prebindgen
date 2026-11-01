# Kotlin collection output cleanup

Compared #824 after #855 with identical added benchmark fixtures: baseline `7956e5e058be6fc28273540b35366498156ace85` versus cleanup change `86991f627d211415770a33710331b8fb77283ab1`. Later commits record generated artifacts, benchmark results and documentation; they do not change the measured generator or fixtures. Both revisions retain the existing sorted handle monitors and recompute bulk pointer positions under those monitors.

Generated native JNI source (`plain.rs` and `bulk.rs`) is byte-for-byte identical between these builds. The Kotlin changes remove per-row cleanup closures/closure-captured mutable cells, share one list of constructed native-owner wrappers across nested collections, omit resource-free cleanup state, and preallocate ordinary result-list storage. Normal handle wrappers, marshalling carriers, locking, input ownership receipts, native work and output reconstruction remain timed.

## Results and limits

- The longer-warmup bulk run saves **20,480 calling-thread JVM bytes per call** at 512 elements for both flat handles and records containing one handle: 73,240 → 52,760 B and 93,720 → 73,240 B. Nested handles save **20,864 B**, 74,024 → 53,160 B. At 64 elements the corresponding savings are 2,560 / 2,560 / 2,944 B. Required wrappers and result lists still allocate.

- Latency is mixed. In the longer-warmup run, 512-element flat/record/nested round trips have 3.7% / 4.9% / 10.5% lower medians. The 64-element record case is effectively flat (+0.2%); the one-handle flat round trip is 4.8% slower. In the broad run, 512-element medians are instead 1.5% / 5.4% / 13.1% slower. These observations do not establish an across-the-board latency improvement.

- Process/JIT variation is material. The unchanged 512-element borrowed-input control is 7.8% slower in the broad run and 14.3% faster in the focused run. Its JVM allocation falls from roughly 41 KiB in the broad run to roughly 16 KiB with longer warmup in both revisions. Allocation differences across these two profiles must not be presented as a cleanup optimization; compare before/after **within** a profile.

- The broad run retains all outliers, including its slower ordinary JNI cases. At 512 elements, ordinary flat-handle round trips are 5.0% slower, record round trips 2.3% slower and primitive-record output 3.5% slower by aggregate medians; nested handles are 1.5% faster. Native row decoding/encoding dominates that transport.

- A diagnostic JIT check found smaller wrapper bytecode after the change (bulk nested-owner wrapper: 2,733 → 2,120 bytes) and confirmed `CollectionOwners.own` is inlined on hot paths. No oversized-method explanation was found for the slower cases. Diagnostic JVM timings are excluded.

The supported conclusion is lower temporary JVM allocation, with case-dependent latency. These single-host boundary microbenchmarks do not measure network packet throughput.

## Method

Measured 2026-11-01 on Linux x86-64, AMD EPYC 7713, affinity CPUs 0–1, Rust 1.98.0 release optimization level 3, Kotlin 2.4.21 and OpenJDK 11.0.29. JVMs use a fixed 256 MiB G1 heap and two active processors. No local builds or other benchmarks ran during timing.

The **broad** comparison runs ordinary JNI object rows (`plain`) and packed byte frames (`bulk`), four fresh JVMs per revision/mode: 16 JVMs, six warmup and nine measured batches targeting 20 ms, 37 cases per process, **5,328 measured batches**. Four treatment orders balance revisions/modes, and case order rotates/reverses between trials. Large ordinary cases can exceed the target due to the minimum batch size; every sample remains included.

The **focused** follow-up investigates the broad run’s latency variation using bulk only, four fresh JVMs per revision: eight JVMs, 20 warmup and 15 measured batches targeting 30 ms, 13 cases per process, **1,560 measured batches**. It includes the same unchanged borrowed-handle input control. This is an additional profile; it does not replace the broad results.

Both compiled revisions first passed all **66 benchmark cases in four diagnostic JVMs each with `-Xcheck:jni`**. Diagnostic timings are excluded. The generated Kotlin/Rust boundary tests separately cover partial construction, optional/nested rows, callback arguments, take/close before failure, allocation failure while registering an owner and destructor exceptions.

Latency is the median of four JVM medians. Brackets are the range of those medians, not confidence intervals. Ratios divide **before by after in the same transport/profile**, so above 1 means faster after. JVM bytes count calling-thread allocation, excluding native allocations, other threads and retained input objects; native allocations remain part of elapsed time. Initial handles/source collections and final cleanup are outside timing. Consuming round trips use the preceding result as the next input and include one extra scalar JNI read to check the first handle. The record case also carries a scalar tag; the nested case has empty lists before/after a handle list.

## Longer-warmup bulk results

| Case | Elements | Before ns [range] | After ns [range] | Before / after | JVM B/call before / after |
|---|---:|---:|---:|---:|---:|
| control | 1 | 14.0 [13.6–14.9] | 15.3 [13.4–16.5] | 0.917× | 0.0 / 0.0 |
| handles_input | 1 | 257.9 [231.7–299.0] | 275.3 [242.4–289.1] | 0.937× | 384.0 / 384.0 |
| handles_input | 64 | 1875.3 [1841.5–2106.0] | 1911.7 [1840.1–1981.9] | 0.981× | 2616.0 / 2608.0 |
| handles_input | 512 | 14815.4 [12058.8–19680.1] | 12700.2 [12488.9–14050.6] | 1.167× | 16528.0 / 16520.0 |
| nested_owned_handles_roundtrip | 1 | 1705.8 [1477.3–1733.7] | 1497.9 [1374.5–1641.9] | 1.139× | 1640.0 / 1216.0 |
| nested_owned_handles_roundtrip | 64 | 9581.7 [8707.3–10169.1] | 8365.3 [7824.7–8505.2] | 1.145× | 10840.0 / 7896.0 |
| nested_owned_handles_roundtrip | 512 | 73570.1 [63340.4–92397.9] | 65844.6 [58595.0–85237.9] | 1.117× | 74024.0 / 53160.0 |
| owned_handle_records_roundtrip | 1 | 930.4 [908.8–964.1] | 901.6 [846.2–914.2] | 1.032× | 848.0 / 816.0 |
| owned_handle_records_roundtrip | 64 | 10916.2 [10560.3–11186.5] | 10943.5 [10444.1–11444.7] | 0.998× | 12624.0 / 10064.0 |
| owned_handle_records_roundtrip | 512 | 78433.9 [73100.9–82410.9] | 74596.3 [69906.7–80460.0] | 1.051× | 93720.0 / 73240.0 |
| owned_handles_roundtrip | 1 | 868.4 [834.8–934.8] | 910.1 [841.4–985.2] | 0.954× | 816.0 / 784.0 |
| owned_handles_roundtrip | 64 | 8227.2 [7710.6–8689.7] | 7983.8 [7707.7–8242.0] | 1.030× | 10064.0 / 7504.0 |
| owned_handles_roundtrip | 512 | 64091.6 [63682.3–67105.9] | 61737.5 [60907.5–63002.6] | 1.038× | 73240.0 / 52760.0 |

## Broad results

| Mode | Case | Elements | Before ns [range] | After ns [range] | Before / after | JVM B/call before / after |
|---|---|---:|---:|---:|---:|---:|
| bulk | callback_records | 0 | 481.4 [476.8–492.4] | 477.8 [474.9–491.7] | 1.008× | 104.0 / 104.0 |
| bulk | callback_records | 1 | 599.3 [586.3–605.7] | 592.5 [577.6–640.0] | 1.012× | 272.0 / 272.0 |
| bulk | callback_records | 8 | 832.1 [813.2–851.2] | 819.3 [806.4–882.3] | 1.016× | 1304.0 / 1304.0 |
| bulk | callback_records | 64 | 2654.7 [2591.4–2741.7] | 2609.3 [2525.5–2676.6] | 1.017× | 9592.0 / 9592.0 |
| bulk | callback_records | 512 | 16865.7 [16116.7–17531.8] | 16497.1 [16234.2–17292.4] | 1.022× | 75896.0 / 75896.0 |
| bulk | control | 1 | 14.3 [12.7–28.3] | 14.1 [13.6–36.2] | 1.012× | 0.0 / 0.0 |
| bulk | handles_input | 0 | 160.8 [148.8–191.1] | 155.8 [144.7–170.7] | 1.032× | 136.0 / 136.0 |
| bulk | handles_input | 1 | 269.5 [259.2–295.3] | 272.0 [248.9–285.1] | 0.991× | 384.0 / 384.0 |
| bulk | handles_input | 8 | 528.9 [465.5–554.5] | 493.1 [472.8–533.1] | 1.073× | 912.0 / 920.0 |
| bulk | handles_input | 64 | 2179.0 [1887.7–2193.3] | 2148.3 [2059.7–2303.6] | 1.014× | 5632.0 / 5640.0 |
| bulk | handles_input | 512 | 14096.5 [13336.1–15139.0] | 15196.9 [14953.3–15470.3] | 0.928× | 41048.0 / 41056.0 |
| bulk | nested_owned_handles_roundtrip | 0 | 1243.4 [1036.0–1336.2] | 1178.3 [974.1–1349.9] | 1.055× | 1080.0 / 691.1 |
| bulk | nested_owned_handles_roundtrip | 1 | 1693.5 [1518.4–1748.7] | 1617.9 [1451.5–1686.4] | 1.047× | 1672.0 / 1248.0 |
| bulk | nested_owned_handles_roundtrip | 8 | 2389.3 [2278.8–2502.6] | 2299.3 [2058.2–2362.2] | 1.039× | 2880.0 / 2220.0 |
| bulk | nested_owned_handles_roundtrip | 64 | 10604.3 [9488.1–13161.6] | 9532.0 [8780.0–14122.1] | 1.112× | 20032.1 / 15596.0 |
| bulk | nested_owned_handles_roundtrip | 512 | 77509.8 [66326.5–107776.5] | 87634.8 [67401.8–101743.7] | 0.884× | 124472.8 / 116030.9 |
| bulk | owned_handle_records_roundtrip | 0 | 486.9 [464.6–519.3] | 480.3 [439.8–494.6] | 1.014× | 352.0 / 336.0 |
| bulk | owned_handle_records_roundtrip | 1 | 956.4 [916.5–973.2] | 1035.8 [1015.4–1082.5] | 0.923× | 880.0 / 848.0 |
| bulk | owned_handle_records_roundtrip | 8 | 1889.0 [1744.0–2073.3] | 1892.4 [1726.5–2049.6] | 0.998× | 2520.0 / 2240.0 |
| bulk | owned_handle_records_roundtrip | 64 | 11774.7 [11505.2–12406.9] | 12019.1 [11679.0–12584.9] | 0.980× | 23506.0 / 22797.0 |
| bulk | owned_handle_records_roundtrip | 512 | 81911.1 [78198.9–112382.0] | 86346.3 [75854.5–119582.3] | 0.949× | 134679.2 / 122440.0 |
| bulk | owned_handles_roundtrip | 0 | 497.7 [465.9–528.7] | 477.2 [464.1–500.3] | 1.043× | 352.0 / 336.0 |
| bulk | owned_handles_roundtrip | 1 | 936.2 [891.1–965.5] | 924.2 [891.8–936.7] | 1.013× | 848.0 / 816.0 |
| bulk | owned_handles_roundtrip | 8 | 1641.8 [1530.7–1843.4] | 1584.6 [1470.6–1639.2] | 1.036× | 2260.0 / 1860.0 |
| bulk | owned_handles_roundtrip | 64 | 8461.5 [8111.8–8505.4] | 8194.4 [7578.7–8480.6] | 1.033× | 16963.2 / 14444.7 |
| bulk | owned_handles_roundtrip | 512 | 68837.9 [67673.6–69834.4] | 69864.8 [64989.4–74233.1] | 0.985× | 124056.2 / 106461.7 |
| bulk | record_input | 1 | 137.4 [129.9–258.6] | 140.3 [135.4–222.0] | 0.980× | 128.0 / 128.0 |
| bulk | records_output | 0 | 168.2 [165.1–178.2] | 187.8 [180.0–239.3] | 0.896× | 104.0 / 104.0 |
| bulk | records_output | 1 | 324.3 [271.2–381.8] | 314.3 [266.5–371.6] | 1.032× | 272.0 / 272.0 |
| bulk | records_output | 8 | 528.0 [516.7–562.5] | 538.2 [512.2–563.5] | 0.981× | 1304.0 / 1304.0 |
| bulk | records_output | 64 | 2391.7 [2347.4–2714.3] | 2267.0 [2212.1–2470.8] | 1.055× | 9592.0 / 9592.0 |
| bulk | records_output | 512 | 16483.4 [16404.9–16562.4] | 16499.4 [16137.1–18280.3] | 0.999× | 75896.0 / 75896.0 |
| bulk | records_roundtrip | 0 | 316.5 [296.6–375.3] | 317.2 [307.6–343.3] | 0.998× | 184.0 / 184.0 |
| bulk | records_roundtrip | 1 | 530.9 [500.6–623.0] | 559.0 [514.7–680.4] | 0.950× | 544.0 / 544.0 |
| bulk | records_roundtrip | 8 | 1038.6 [987.7–1091.0] | 1037.6 [998.8–1253.9] | 1.001× | 2080.0 / 2080.0 |
| bulk | records_roundtrip | 64 | 4454.4 [4242.0–4719.8] | 4597.9 [4465.6–4627.5] | 0.969× | 14176.0 / 14176.0 |
| bulk | records_roundtrip | 512 | 31191.5 [30726.0–32286.4] | 31669.8 [31323.7–32980.1] | 0.985× | 110944.0 / 110944.0 |
| plain | callback_records | 0 | 811.3 [804.7–848.1] | 825.4 [807.6–843.1] | 0.983× | 64.0 / 40.0 |
| plain | callback_records | 1 | 12687.5 [12080.7–15153.6] | 12571.2 [12480.5–12695.6] | 1.009× | 504.0 / 392.0 |
| plain | callback_records | 8 | 99060.5 [92143.6–175640.1] | 95309.0 [94591.5–107985.3] | 1.039× | 2768.0 / 2680.0 |
| plain | callback_records | 64 | 746562.4 [729723.1–757272.8] | 776016.1 [744958.1–787819.6] | 0.962× | 22640.0 / 21048.0 |
| plain | callback_records | 512 | 5962521.0 [5863568.2–6042685.6] | 6105393.1 [5985468.2–6787615.9] | 0.977× | 179424.0 / 167992.0 |
| plain | control | 1 | 13.6 [13.4–13.8] | 13.7 [13.3–14.5] | 0.994× | 0.0 / 0.0 |
| plain | handles_input | 0 | 68.2 [65.4–72.6] | 67.2 [66.2–68.9] | 1.014× | 96.0 / 96.0 |
| plain | handles_input | 1 | 1980.1 [1973.3–2044.7] | 1986.3 [1948.0–2100.8] | 0.997× | 280.0 / 280.0 |
| plain | handles_input | 8 | 14416.9 [14265.9–14536.5] | 14689.5 [14425.7–14809.3] | 0.981× | 1112.0 / 1112.0 |
| plain | handles_input | 64 | 115313.8 [112841.0–125267.3] | 115707.9 [113446.3–123131.5] | 0.997× | 8296.0 / 8296.0 |
| plain | handles_input | 512 | 934765.0 [902694.5–978932.4] | 932936.0 [925297.7–973771.6] | 1.002× | 63424.0 / 63424.0 |
| plain | nested_owned_handles_roundtrip | 0 | 7612.7 [7598.7–7624.1] | 7851.9 [7603.9–8546.4] | 0.970× | 1032.0 / 760.0 |
| plain | nested_owned_handles_roundtrip | 1 | 12521.3 [12093.3–24679.3] | 12385.2 [12259.6–13938.1] | 1.011× | 1752.0 / 1432.0 |
| plain | nested_owned_handles_roundtrip | 8 | 40504.5 [39655.7–41424.0] | 40079.2 [39631.7–40285.6] | 1.011× | 3772.0 / 3184.0 |
| plain | nested_owned_handles_roundtrip | 64 | 264174.4 [260425.5–280262.1] | 264703.1 [263516.7–271687.3] | 0.998× | 30513.0 / 27265.3 |
| plain | nested_owned_handles_roundtrip | 512 | 2072488.9 [2043916.4–2107975.2] | 2042053.0 [2023620.2–2130778.9] | 1.015× | 171016.0 / 142252.5 |
| plain | owned_handle_records_roundtrip | 0 | 688.4 [683.1–747.9] | 712.0 [696.7–733.8] | 0.967× | 184.0 / 184.0 |
| plain | owned_handle_records_roundtrip | 1 | 7582.1 [7490.3–8463.5] | 7680.5 [7612.4–7918.7] | 0.987× | 720.0 / 672.0 |
| plain | owned_handle_records_roundtrip | 8 | 55682.1 [52768.2–77095.6] | 54004.9 [53370.1–57360.8] | 1.031× | 2876.0 / 2520.0 |
| plain | owned_handle_records_roundtrip | 64 | 431902.1 [416331.1–447419.5] | 423783.4 [419988.1–435653.8] | 1.019× | 30017.0 / 26848.5 |
| plain | owned_handle_records_roundtrip | 512 | 3317646.1 [3302842.7–3527197.6] | 3392386.7 [3338755.8–3464131.4] | 0.978× | 200560.5 / 169404.0 |
| plain | owned_handles_roundtrip | 0 | 689.9 [675.8–711.4] | 707.4 [678.6–739.0] | 0.975× | 184.0 / 184.0 |
| plain | owned_handles_roundtrip | 1 | 4960.0 [4896.9–5045.4] | 5087.5 [4929.7–5222.4] | 0.975× | 696.0 / 648.0 |
| plain | owned_handles_roundtrip | 8 | 32323.1 [31930.0–32978.5] | 32907.5 [32533.3–33327.7] | 0.982× | 2712.0 / 2380.0 |
| plain | owned_handles_roundtrip | 64 | 252770.6 [251014.7–261090.4] | 261993.2 [259620.4–262634.1] | 0.965× | 28678.9 / 25728.0 |
| plain | owned_handles_roundtrip | 512 | 1987437.1 [1970978.4–2023743.4] | 2085863.0 [2017682.7–2254314.5] | 0.953× | 166785.0 / 142911.0 |
| plain | record_input | 1 | 30.6 [29.5–31.4] | 30.3 [29.0–31.6] | 1.009× | 0.0 / 0.0 |
| plain | records_output | 0 | 530.0 [515.4–546.5] | 508.7 [500.8–619.1] | 1.042× | 64.0 / 40.0 |
| plain | records_output | 1 | 12163.9 [11878.6–12278.4] | 12189.4 [12105.2–12564.8] | 0.998× | 504.0 / 392.0 |
| plain | records_output | 8 | 93122.8 [90273.1–94915.8] | 96091.7 [94557.6–96971.3] | 0.969× | 2768.0 / 2680.0 |
| plain | records_output | 64 | 760595.2 [747077.7–782483.0] | 770845.3 [765999.2–790972.5] | 0.987× | 22640.0 / 21048.0 |
| plain | records_output | 512 | 5866215.8 [5848953.2–6119016.2] | 6073915.0 [5991637.6–6293014.9] | 0.966× | 179424.0 / 167992.0 |
| plain | records_roundtrip | 0 | 551.9 [527.2–769.0] | 570.5 [544.4–596.9] | 0.967× | 80.0 / 56.0 |
| plain | records_roundtrip | 1 | 22333.2 [21685.8–24132.8] | 22416.3 [22382.9–22489.6] | 0.996× | 792.0 / 680.0 |
| plain | records_roundtrip | 8 | 180520.5 [170877.8–298969.8] | 177774.9 [173383.6–178451.8] | 1.015× | 4808.0 / 4720.0 |
| plain | records_roundtrip | 64 | 1374887.8 [1347594.9–1435460.5] | 1452045.2 [1370753.1–1467509.9] | 0.947× | 38568.0 / 36976.0 |
| plain | records_roundtrip | 512 | 11080527.4 [10736635.8–11708120.2] | 11130993.7 [10950470.6–11443861.6] | 0.995× | 306456.0 / 295024.0 |

## Reproduce

Build each recorded revision in a clean checkout into a separate empty output directory using `examples/bench-bulk/run.py --build-only` and the same Kotlin compiler. The two revisions have identical fixture hashes. The comparison runners validate compiler, dependency lock, profile and artifact hashes, and recheck JAR/native hashes after timing.

```sh
taskset -c 0,1 python3 examples/bench-bulk/output-cleanup-results/compare.py \
  --before /tmp/cleanup-before --after /tmp/cleanup-after \
  --output /tmp/cleanup-broad
taskset -c 0,1 python3 examples/bench-bulk/output-cleanup-results/focused/compare.py \
  --before /tmp/cleanup-before --after /tmp/cleanup-after \
  --output /tmp/cleanup-focused
```

Choose permitted CPUs or record a different placement on another host. [Broad metadata and process medians](results.json), [broad raw samples](samples.csv), [focused metadata and medians](focused/results.json) and [focused raw samples](focused/samples.csv) preserve every measured batch and exact build revisions.
