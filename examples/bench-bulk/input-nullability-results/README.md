# Required Kotlin input nullability: before / after

Both generators bind the same benchmark fixture from commit `5f1e32e2a`.
`base` uses umbrella #824 at `cd2956dd3`; `candidate` uses `5f1e32e2a`.
The subsequent review cleanup removes an obsolete generator rewrite and tightens tests/documentation; it does not change the generated benchmark code.

Release Rust; Kotlin 2.4.21; Java 11; two available CPUs. No concurrent builds.
Run order: base, candidate, candidate, base. Each run uses four fresh JVMs per
package, alternating plain/bulk order: eight JVMs per side and package in total.
Each case has eight warmup batches and eleven measured batches targeting 30 ms.
Values below are medians of JVM medians. These are **before/after** comparisons;
plain and bulk are separate implementations, not the speedup denominator.

| Case | Plain base → candidate (ns/call) | Bulk base → candidate (ns/call) |
| --- | ---: | ---: |
| control/1 | 13.8 → 13.9 | 13.7 → 14.0 |
| handle_input/1 | 21.3 → 19.2 | 20.6 → 19.5 |
| buffer_input/1 | 714.1 → 709.0 | 734.9 → 716.7 |
| selected_handle_input/1 | 22.6 → 22.2 | 22.1 → 22.3 |
| inactive_handle_input/1 | 21.3 → 20.6 | 21.1 → 20.8 |
| handles_input/64 | 118,868.7 → 115,713.1 | 1,780.7 → 1,780.3 |
| handle_records_input/64 | 187,712.6 → 189,495.6 | 2,079.2 → 2,094.8 |

The direct handle case improves by about 1–2 ns (5–10%) in these runs. Other
median changes are small compared with JVM-to-JVM variation, including the
unchanged primitive control. These measurements show no broad latency regression;
they do not establish a general throughput gain.

Direct handle, selector and buffer cases allocate 0 JVM bytes/call on both sides.
Collection allocation medians change by −18 to +12 bytes/call for handle lists;
record-list medians are unchanged. No native allocation counter was enabled.
The complete JVM medians, ranges, allocation figures, source revisions, compiler
versions and artifact/sample hashes are in [summary.json](summary.json).

Reproduce each prepared build with the same fixture and the desired generator checkout:

```sh
KOTLINC=/path/to/kotlinc python3 examples/bench-bulk/run.py \
  --checkout /path/to/checkout --output /tmp/unique-output \
  --cases control,handle_input,selected_handle_input,inactive_handle_input,buffer_input,handles_input,handle_records_input \
  --sizes 64 --payloads 64 --jvms 4 --warmup 8 --batches 11 --target-ms 30
```

Build both sides before measuring, preserve each JAR/native library independently,
and use `--prepared` in the order above. Native dependency trees and fixture
sources must be committed and clean. Console logs and raw samples are omitted.
