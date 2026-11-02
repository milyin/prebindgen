# Isolated collection-output comparison

This follow-up runs exactly one benchmark case and collection size per JVM. It
reuses the frozen before/after native libraries and generated Kotlin classes from
the original comparison. An identical driver overlay adds calling-thread CPU time,
compilation and garbage-collection diagnostics around the unchanged operation loop.
The generated binding code and native libraries are not modified for measurement.

## Protocol

- Eighteen case/size/transport combinations, four fresh JVMs per revision per
  combination: 144 JVMs and 3,456 measured batches. Bulk handle round trips cover
  sizes 1, 64 and 512, including records and nested collections. Ordinary JNI
  round trips and primitive-record output cover size 512. Unchanged borrowed-handle
  input and scalar JNI calls serve as controls.
- Each case/revision pair runs consecutively. Before/after order alternates by
  fork, and case order uses a recorded deterministic shuffle per fork.
- Warmup makes 256 short sampling calls, then full-sized calls for at least three
  seconds and until compiler CPU time has remained unchanged for one second
  (20-second cap). Compilation logs independently record methods compiled during
  the measured phase; the quiet-period heuristic alone does not establish stability.
- The sampling loop cannot be inlined into the driver controller, and only that
  sampling method uses a reduced compilation threshold. The controller and GC
  diagnostic helpers remain interpreted. Generated wrappers, runtime helpers and
  operation implementations retain normal JVM compilation settings. This avoids
  warming an inlined loop and measuring a different, still-compiling loop.
- Each fork records 24 batches targeting 50 ms each. All batches remain included,
  including GC pauses. Checksums and callback counts are checked after each batch.
- Fixed 256 MiB G1 heap, two active processors, affinity CPUs 0–1, one JVM at a
  time. No builds or other benchmarks run concurrently. Compiler, native/JAR and
  source hashes are verified and preserved in the metadata.

The primary score is total timed wall nanoseconds divided by total operations in
one JVM, then the median of four JVM scores. Per-fork ranges and paired changes
are preserved. CPU time is measured on the calling thread and excludes other
threads and time stopped for GC or scheduling. Allocation is calling-thread JVM
allocation; native allocation is included in elapsed time but not byte counts.
These scores are means within a JVM, unlike the batch-median scores in the earlier
broad/focused tables. The older statistic is also retained in the JSON.

`-Xcheck:jni` is used only in separate smoke/validation runs, not timing runs.
Pilot runs were used to diagnose compilation of the sampling method during
measurement. They are excluded from the final comparison; timing outcomes were
not used to select individual samples or forks.

## Reproduce

Build the two revisions using the parent report's instructions, then prepare one
instrumented driver shared by both versions:

```sh
python3 examples/bench-bulk/output-cleanup-results/isolated/prepare.py \
  --before /tmp/cleanup-before --after /tmp/cleanup-after \
  --output /tmp/cleanup-driver --kotlinc /path/to/kotlinc
taskset -c 0,1 python3 examples/bench-bulk/output-cleanup-results/isolated/compare.py \
  --driver /tmp/cleanup-driver --output /tmp/cleanup-isolated
```

Use an empty output directory and the same Kotlin compiler as the frozen builds.
The runner stores stdout, complete HotSpot compilation XML, and GC/safepoint logs
under `raw/`, plus their hashes in `results.json`. `samples.csv` records every
batch. The committed metadata preserves all compilation events completed during
measurement, including JDK helpers, alongside the scores and per-fork diagnostics.
Raw JVM logs remain in the local measurement directory to avoid committing the
large XML files; no batches are removed based on those logs.

## Isolated results (2026-11-02)

Every JVM met its warmup conditions and had an active C2-compiled entry for the
sampling loop before measurement. The compilation/deoptimization logs show no
measured-phase compilation or deoptimization of the sampling loop, benchmark
operations or generated binding methods. JDK/Kotlin runtime helpers still changed:
234 runtime uncommon traps and 156 method-retirement events were logged during
measurement, mainly in `java.nio.Buffer` construction. This is a stronger isolation
check, not a guarantee that every JVM helper has reached a final state.

Positive changes below mean slower after. Ranges are four JVM scores, not confidence
intervals. CPU percentages use the same per-JVM aggregation as wall time.

| Mode | Case | Elements | Before ns [range] | After ns [range] | Wall change | CPU change | JVM B/call before / after |
|---|---|---:|---:|---:|---:|---:|---:|
| bulk | control | 1 | 13.7 [13.0–14.4] | 14.3 [13.1–15.0] | +4.5% | +4.6% | 0.0 / 0.0 |
| bulk | handles_input | 1 | 239.9 [238.7–244.8] | 222.2 [211.2–230.1] | -7.4% | -7.4% | 336.0 / 336.0 |
| bulk | handles_input | 64 | 2337.5 [2275.9–2489.6] | 2248.5 [2160.1–2386.5] | -3.8% | -3.7% | 2560.0 / 2552.0 |
| bulk | handles_input | 512 | 13174.7 [13093.4–14005.2] | 13954.0 [13070.2–14998.2] | +5.9% | +6.0% | 16476.0 / 16488.0 |
| bulk | nested_owned_handles_roundtrip | 1 | 1726.7 [1478.9–2418.2] | 1282.0 [1254.7–1676.9] | -25.8% | -25.6% | 1640.0 / 1216.0 |
| bulk | nested_owned_handles_roundtrip | 64 | 9562.9 [8921.4–10349.1] | 8998.1 [8738.3–10216.4] | -5.9% | -5.6% | 10832.0 / 7888.0 |
| bulk | nested_owned_handles_roundtrip | 512 | 86690.6 [81875.8–118652.6] | 87306.9 [86379.7–92547.4] | +0.7% | +0.7% | 74016.0 / 53160.0 |
| bulk | owned_handle_records_roundtrip | 1 | 920.2 [815.5–1048.1] | 954.7 [863.9–1209.7] | +3.7% | +3.7% | 800.0 / 768.0 |
| bulk | owned_handle_records_roundtrip | 64 | 12313.0 [10612.3–13254.9] | 12159.8 [10571.5–13089.9] | -1.2% | -1.2% | 12560.0 / 10008.0 |
| bulk | owned_handle_records_roundtrip | 512 | 86602.1 [84535.9–107743.6] | 82952.6 [77431.8–86797.4] | -4.2% | -3.7% | 93656.0 / 73208.0 |
| bulk | owned_handles_roundtrip | 1 | 882.7 [806.3–954.7] | 930.2 [832.4–972.0] | +5.4% | +4.7% | 768.0 / 736.0 |
| bulk | owned_handles_roundtrip | 64 | 8671.9 [8075.2–9317.7] | 8716.8 [7449.7–8872.9] | +0.5% | +0.6% | 10000.0 / 7460.0 |
| bulk | owned_handles_roundtrip | 512 | 73254.7 [70198.2–78648.5] | 70909.0 [68160.6–80460.1] | -3.2% | -2.7% | 73188.0 / 52728.0 |
| plain | control | 1 | 14.2 [13.2–14.3] | 14.5 [13.5–16.6] | +2.5% | +2.5% | 0.0 / 0.0 |
| plain | nested_owned_handles_roundtrip | 512 | 2271780.7 [2062928.4–2370106.0] | 2212669.6 [2037551.8–2648142.6] | -2.6% | -2.6% | 131848.0 / 106448.0 |
| plain | owned_handle_records_roundtrip | 512 | 3687040.2 [3260861.2–3837441.8] | 3947366.7 [3368572.9–4220923.6] | +7.1% | +7.2% | 223785.2 / 192614.4 |
| plain | owned_handles_roundtrip | 512 | 2198432.9 [1933254.5–2309375.2] | 2165317.5 [2011435.5–2306293.4] | -1.5% | -1.6% | 122696.0 / 97568.0 |
| plain | records_output | 512 | 6741115.7 [5795156.1–7326365.9] | 6561867.6 [5897239.6–7103659.0] | -2.7% | -2.0% | 179424.0 / 167992.0 |

Allocation is lower for every changed collection-output case. At 512 elements,
bulk flat/record/nested round trips save approximately 20.5 / 20.4 / 20.9 KB of
calling-thread JVM allocation. Small differences from the earlier profile reflect
different optimized driver/runtime allocation; compare revisions within a profile.

Bulk 512-element flat/record/nested wall scores change by -3.2% / -4.2% / +0.7%.
Those small changes are not conclusive: the unchanged 512-element bulk input
control is +5.9%, and individual paired forks overlap or disagree. The nested
one-element case improves in all four pairs (-14.7%, -16.6%, -11.4%, -48.1%);
its median score is 25.8% lower, with a wide before range.

Two slower cases deserve investigation. The one-element bulk flat-handle round
trip is +5.4% by the aggregate score; every paired fork is slower
(+1.8%, +6.1%, +4.6%, +3.2%). The ordinary JNI 512-element handle-record
round trip is +7.1% (+7.2% calling-thread CPU), with paired changes
+10.0%, +3.3%, +8.9%, +5.2%. These are observed regressions, not evidence
that every output path is slower. Four forks on one host and the remaining
runtime-helper changes do not establish the cause. The unchanged one-element
bulk input control also moves consistently (-7.4% by aggregate score), so
paired direction alone is not proof of a cleanup-code effect.

The supported outcome remains lower allocation with case-dependent latency. This
follow-up does not support a general latency-speedup claim or identify the cause
of the slower cases. All 3,456 batches, including GC pauses and slower forks, are
retained in [samples.csv](samples.csv). [results.json](results.json) records every
process, score, command and compilation event; [diagnostics.json](diagnostics.json)
records C2 availability and measured-phase runtime deoptimizations.

The final driver separately passed six fresh smoke JVMs with `-Xcheck:jni`
(three cases, both revisions, twelve checked batches). Its timings and the pilot
timings are excluded from the comparison.

For the additional deoptimization summary after a reproduced run:

```sh
python3 examples/bench-bulk/output-cleanup-results/isolated/diagnostics.py \
  /tmp/cleanup-isolated
```
