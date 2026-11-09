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
- Warmup makes 256 short sampling calls, then full-sized calls until total warmup
  has lasted at least three seconds and cumulative compilation time reported by
  `CompilationMXBean` has remained unchanged for one second (20-second cap).
  Compilation logs independently record methods compiled during
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
- Compilation and GC/safepoint logging remain enabled during timing in both
  revisions. CPU frequency/turbo and core isolation are not controlled.

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
git show 7169e3405:examples/bench-bulk/output-cleanup-results/isolated/compare.py \
  > /tmp/cleanup-original-runner.py
taskset -c 0,1 python3 /tmp/cleanup-original-runner.py \
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

Inspection of the stored data verified that all 144 JVMs met their warmup conditions and had an active entry compiled by
HotSpot's optimizing compiler (C2) for the
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

Two cells show unresolved slowdowns. The one-element bulk flat-handle round
trip is +5.4% by the aggregate score; every paired fork is slower
(+1.8%, +6.1%, +4.6%, +3.2%). The ordinary JNI 512-element handle-record
round trip is +7.1% (+7.2% calling-thread CPU), with paired changes
+10.0%, +3.3%, +8.9%, +5.2%. These changes are comparable to control drift;
four paired forks across 18 cells do not establish a cleanup-code regression.
Four forks on one host and the remaining
runtime-helper changes do not establish the cause. The unchanged one-element
bulk input control also moves consistently (-7.4% by aggregate score), so
paired direction alone is not proof of a cleanup-code effect. Its initial paired
changes are -4.2%, -13.7%, -7.6%, -6.6%.

The supported outcome is lower allocation; latency attribution remains unresolved.
This follow-up does not support a general latency-speedup claim or identify the cause
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

## Diagnostic attribution

The `compiled_method` owning each runtime trap/retirement event is used for
attribution, rather than its innermost inline frame. None belongs to the
benchmark or generated binding methods. All generated classes and runtime
helpers, including handle monitors, live in `bench.plain` or `bench.bulk`, which
the method-prefix filter includes. Kotlin/JDK library helpers remain separate.
Counts below include all four forks per cell. They can differ by revision and
therefore remain a latency confound.

| Mode | Case | Size | Before traps / retirements | After traps / retirements |
|---|---|---:|---:|---:|
| plain | control | 1 | 0 / 0 | 0 / 0 |
| bulk | control | 1 | 0 / 4 | 0 / 1 |
| bulk | handles_input | 1 | 16 / 4 | 16 / 4 |
| bulk | handles_input | 64 | 4 / 4 | 0 / 5 |
| bulk | handles_input | 512 | 8 / 7 | 8 / 3 |
| bulk | nested_owned_handles_roundtrip | 1 | 16 / 7 | 16 / 5 |
| bulk | nested_owned_handles_roundtrip | 64 | 16 / 9 | 16 / 7 |
| bulk | nested_owned_handles_roundtrip | 512 | 0 / 7 | 0 / 5 |
| bulk | owned_handle_records_roundtrip | 1 | 16 / 6 | 16 / 4 |
| bulk | owned_handle_records_roundtrip | 64 | 4 / 4 | 4 / 5 |
| bulk | owned_handle_records_roundtrip | 512 | 4 / 4 | 4 / 6 |
| bulk | owned_handles_roundtrip | 1 | 16 / 4 | 16 / 5 |
| bulk | owned_handles_roundtrip | 64 | 16 / 5 | 12 / 5 |
| bulk | owned_handles_roundtrip | 512 | 4 / 2 | 5 / 4 |
| plain | nested_owned_handles_roundtrip | 512 | 0 / 3 | 0 / 6 |
| plain | owned_handle_records_roundtrip | 512 | 0 / 5 | 0 / 7 |
| plain | owned_handles_roundtrip | 512 | 0 / 1 | 1 / 7 |
| plain | records_output | 512 | 0 / 0 | 0 / 1 |

The initial 144-JVM result uses the runner committed at `7169e3405`; its recorded
SHA-256 matches that source. The runner was subsequently extended for explicit
A/A and shared-native-library controls. The driver is unchanged.

A separate compilation against the after binding JAR produced byte-identical
classes and Kotlin metadata to compilation against the before JAR (all five
entries). [driver-identity.json](driver-identity.json) records the command and
class hashes. This checks that the shared driver does not silently inline
different binding code or constants from one revision.

## A/A and identical-native-library follow-ups

The additional profiles use the same driver, JVM flags, warmup, 24 measured
batches per process and four paired forks. An A/A comparison labels the same
baseline JAR/native image as both A and B (24 JVMs); the second comparison
loads the exact same baseline native image with each Kotlin revision (40 JVMs).
The generated JNI interfaces match. Each process command and its effective
artifacts are recorded separately from the original build provenance.

Together with the initial profile this is **208 timing JVMs and 4,992 measured
batches**. No timing samples are dropped. The A/A data demonstrates variation;
it does not define a universal confidence bound or a threshold for significance.

The shared driver was recompiled against both binding JARs and its classes
and Kotlin metadata are identical. This excludes different inlined binding
functions or constants in the driver as an explanation. The native library
files and their `.text` sections differ across the original builds despite
identical generated Rust source; the shared-library profile removes that
particular build difference.

### Same code against itself (A/A)

Positive changes below mean B is slower. Both labels execute the same code.

| Mode | Case | Size | A ns | B ns | Wall change | JVM B/call A / B | Paired changes |
|---|---|---:|---:|---:|---:|---:|---|
| bulk | control | 1 | 14.3 | 15.0 | +5.2% | 0.0 / 0.0 | -4.7%, +10.7%, +4.9%, +11.7% |
| bulk | owned_handles_roundtrip | 1 | 916.8 | 935.2 | +2.0% | 768.0 / 768.0 | -7.5%, +19.5%, -2.1%, +6.2% |
| plain | owned_handle_records_roundtrip | 512 | 3711250.1 | 3602170.9 | -2.9% | 228200.6 / 224605.9 | -40.8%, -2.6%, -1.0%, +12.3% |

The aggregate scores move by 2–5% even with identical code. Individual paired
changes reach -40.8% / +19.5%, so four-fork aggregates must not be treated as
precise causal estimates. In particular, the initial +5.4% / +7.1% cells do
not establish cleanup-code regressions.

### Both Kotlin revisions with one native image

Positive changes mean slower with the new Kotlin decoder. Controls are unchanged
Kotlin input methods. The ordinary JNI handle-record comparison changes sign, but the identical
baseline configuration also shifts between sessions; this does not attribute
the change to native build layout. Flat one-element handles are slower in 7 of
8 paired comparisons across the two profiles; nested one-element handles are
faster in 8 of 8. Both are latency candidates, one slower and one faster; neither
is established as a cleanup-code effect. Only the flat case has a same-cell A/A
comparison. Neither profile establishes a universal speedup or a slowdown cause.

| Mode | Case | Size | Before ns | After ns | Wall change | CPU change | Paired changes |
|---|---|---:|---:|---:|---:|---:|---|
| bulk | handles_input | 1 | 229.6 | 236.0 | +2.8% | +2.7% | +4.6%, -13.9%, +1.1%, +12.9% |
| bulk | handles_input | 512 | 13891.9 | 13993.5 | +0.7% | +0.7% | -9.0%, +1.4%, +5.6%, +3.8% |
| bulk | nested_owned_handles_roundtrip | 1 | 1631.9 | 1448.8 | -11.2% | -10.9% | -7.4%, -15.1%, -1.8%, -11.5% |
| bulk | owned_handles_roundtrip | 1 | 889.0 | 964.7 | +8.5% | +8.0% | +8.6%, +35.1%, +7.2%, -8.9% |
| plain | owned_handle_records_roundtrip | 512 | 4072635.4 | 3761047.7 | -7.7% | -8.1% | -4.9%, +8.7%, -10.6%, -13.3% |

Allocation reductions remain the supported finding. The shared-native profile
also confirms 768 → 736 B for one flat handle and 1,640 → 1,216 B for one
nested handle. The initial 512-element output reductions remain about 20–21 KB
per call. A/A allocation can also vary with JVM optimization, especially in the
ordinary JNI row path; allocation comparisons must stay within each profile.

[All A/A metadata and scores](aa/results.json), [A/A batches](aa/samples.csv),
[all shared-native metadata and scores](common-native/results.json), and
[shared-native batches](common-native/samples.csv) retain each process and sample.

Across sessions, the byte-identical baseline ordinary JNI record configuration
moves from 3.687 ms initially to 3.711 / 3.602 ms in A/A and 4.073 ms in the
shared-native profile (+10.5% versus the initial baseline). The after score moves
from 3.947 to 3.761 ms. The unchanged 512-element input baseline also moves
13,175 → 13,892 ns (+5.4%). Thus the ordinary-record sign change cannot be
attributed to removing native-build differences.

For nested one-element handles, the initial ratio-of-median scores improves
25.8%, while the median paired change is -15.6%; one pair is -48.1%. With one
native image, the corresponding scores are -11.2% and a -9.4% median paired
change. These statistics differ; the outlier must not inflate the headline.

The bulk A/A allocation values are identical. Ordinary JNI A/A allocation
varies by approximately 3.6 KB, while the ordinary handle-record comparisons
show approximately 31 KB lower allocation in both profiles. This is observed
variation, not a bound on all JVM optimization effects.

The shared-native configuration separately passed six smoke JVMs with
`-Xcheck:jni` (twelve checked batches). No builds ran during timing.

To reproduce the additional profiles with the current runner:

```sh
taskset -c 0,1 python3 examples/bench-bulk/output-cleanup-results/isolated/compare.py \
  --driver /tmp/cleanup-driver --output /tmp/cleanup-aa --same-build before \
  --cells bulk:control:1,bulk:owned_handles_roundtrip:1,plain:owned_handle_records_roundtrip:512
taskset -c 0,1 python3 examples/bench-bulk/output-cleanup-results/isolated/compare.py \
  --driver /tmp/cleanup-driver --output /tmp/cleanup-common-native --native-version before \
  --cells bulk:handles_input:1,bulk:handles_input:512,bulk:owned_handles_roundtrip:1,plain:owned_handle_records_roundtrip:512,bulk:nested_owned_handles_roundtrip:1
```

## Diagnostics for the additional 64 JVMs

All 24 A/A and 40 shared-native JVMs had an active C2 sampling entry before
measurement. No benchmark or generated binding compilation/deoptimization
occurred during measurement. The complete additional logs are summarized in
[A/A diagnostics](aa/diagnostics.json) and
[shared-native diagnostics](common-native/diagnostics.json). The tables below
include all four forks per cell and attribute events by the compiled method
that owned the event, including any inlined frames.

aa:

| Mode | Case | Size | Before traps / retirements | After traps / retirements |
|---|---|---:|---:|---:|
| bulk | control | 1 | 0 / 2 | 0 / 3 |
| bulk | owned_handles_roundtrip | 1 | 16 / 5 | 15 / 4 |
| plain | owned_handle_records_roundtrip | 512 | 2 / 12 | 0 / 5 |

common-native:

| Mode | Case | Size | Before traps / retirements | After traps / retirements |
|---|---|---:|---:|---:|
| bulk | handles_input | 1 | 16 / 4 | 15 / 7 |
| bulk | handles_input | 512 | 8 / 4 | 4 / 4 |
| bulk | owned_handles_roundtrip | 1 | 16 / 6 | 12 / 5 |
| plain | owned_handle_records_roundtrip | 512 | 0 / 5 | 1 / 9 |
| bulk | nested_owned_handles_roundtrip | 1 | 16 / 7 | 16 / 6 |

Two additional diagnostic JVMs, one per Kotlin revision, were inspected through
`/proc/<pid>/maps` while running the shared-native configuration. Both loaded
`/tmp/output-cleanup-before/target/release/libbulk_bench.so` with the recorded
baseline SHA-256. [native-load-check.json](native-load-check.json) preserves
commands and loaded mappings. Neither JAR contains a bundled native library;
the generated `JNINative` initializer calls `System.loadLibrary("bulk_bench")`,
and both commands supply the
same single native-library directory. These diagnostic timings are excluded.
