# Fixed JNI callback-result cleanup

A Kotlin callback returning a fixed record to Rust already supplies its owned
handle addresses in native locals. Successful decoding now avoids a separate
`CallbackReturnAllocations` vector and ownership scan. An existing stack guard
uses those locals only on failure/unwind, then releases unread owners after
skipping the consumed prefix. Optional fields retain their presence controls.
Ordinary JVM collections containing owned handles retain eager tracking because
their rows can change; independently cached bulk results retain their existing
failure-only cleanup. There is no public API change.

Baseline: #824 at `e8eb5f894f05802d9ab62c36395abd2dbdf9a7ce`.
Implementation: `970ac43a1` (later commits add generated benchmark output and this
report). The fixture and generated Kotlin source hashes match. Both timing
versions use the exact same baseline JAR. Generated `bulk.rs` is byte-identical;
its native machine-code layout can still change when the library is rebuilt.
Full revisions, artifact hashes, compiler versions and per-JVM medians are in
[results.json](results.json). Raw console logs remain outside Git.

## Native allocations

Each cell has two fresh JVMs, 1,000 warmup calls and ten measured calls per JVM.
All twenty observations per version agree. Separate allocation-instrumented
release libraries count calling-thread Rust allocations/reallocations throughout
the complete generated call. Initial handle creation and final close are outside
the samples; handles circulate through consuming calls. Requested bytes include
reallocation requests and do not represent peak live memory.

| Transport / callback result | Rows | Allocations before → after | Reallocations before → after | Requested bytes before → after |
| --- | ---: | ---: | ---: | ---: |
| bulk / one handle/tag record | 1 | 6 → 6 | 2 → 2 | 122 → 122 |
| bulk / handle/tag array | 64 | 70 → 70 | 3 → 3 | 3,714 → 3,714 |
| plain / one handle/tag record | 1 | 5 → 4 | 2 → 2 | 186 → 90 |
| plain / handle/tag array | 1 | 7 → 7 | 3 → 3 | 235 → 235 |
| plain / handle/tag array | 64 | 70 → 70 | 7 → 7 | 4,627 → 4,627 |
| plain / primitive control | 1 | 0 → 0 | 0 → 0 | 0 → 0 |

The affected ordinary fixed record saves **one allocation and 96 requested bytes
per call**. The bulk record, ordinary arrays and primitive control retain their
counts. Other native allocations in each complete call remain.

## Complete-call latency

Uninstrumented release libraries run six fresh JVMs per version, ten warmup
batches and eleven measured batches targeting 30 ms. Version order alternates.
The fixed record case (`callback_owned_record`) returns an existing handle/tag
record from Kotlin to Rust, then returns the reconstructed record to Kotlin for
reuse by the next call. JNI, locks, conversion, output construction, GC and
checksum work remain timed. The primitive record callback and primitive scalar
control contain no owned handles. Array callbacks exercise the retained path.

The table reports medians of per-JVM medians; change is
`after median / before median - 1`. This compares generator revisions with the
same transport. JVM allocation observations and paired ratios are also retained
in the JSON.

| Transport / case | Rows | Before µs/call | After µs/call | Change |
| --- | ---: | ---: | ---: | ---: |
| bulk / one handle/tag record | 1 | 2.019 | 2.058 | +1.9% |
| bulk / primitive record callback | 1 | 0.903 | 0.888 | -1.7% |
| bulk / primitive control | 1 | 0.014 | 0.015 | +1.9% |
| plain / one handle/tag record | 1 | 1.890 | 1.836 | -2.9% |
| plain / handle/tag array | 1 | 3.721 | 3.818 | +2.6% |
| plain / handle/tag array | 64 | 99.066 | 101.288 | +2.2% |
| plain / primitive record callback | 1 | 2.162 | 2.183 | +1.0% |
| plain / primitive control | 1 | 0.014 | 0.014 | -1.8% |

Identical-library controls load the same baseline native library and JAR under
both labels, with four fresh JVMs per label and the same batch settings:

| Transport / case | Before µs/call | After µs/call | Apparent change |
| --- | ---: | ---: | ---: |
| bulk / one handle/tag record | 2.270 | 2.254 | -0.7% |
| plain / one handle/tag record | 1.881 | 1.776 | -5.6% |
| plain / primitive record callback | 2.127 | 2.171 | +2.1% |
| plain / primitive control | 0.015 | 0.014 | -1.8% |

The affected case's 2.9% lower median is smaller than its 5.6% shift in the
identical-library control. Its median paired ratio shows only 1.2% improvement.
**Complete-call latency remains inconclusive.** The repeatable result is removal
of the successful-path native allocation, with a smaller cleanup predicate and
reuse of an existing guard.

## Correctness and reproduction

Generated Rust/JNI and Kotlin covertests exercise optional/multiple owners,
invalid Unicode and boxed scalar fields, invalid pointer encodings, source field
constructor panics, partial consumption, cleanup before retry and native worker
threads. Existing bulk and ordinary collection tests still pass. Workspace tests,
doctests, Clippy with warnings denied, C ASan/UBSan and Kotlin/Java `-Xcheck:jni`
checks pass. The separate instrumented benchmark also passes with `-Xcheck:jni`.
Claude Opus 5.5 reviewed the source/diff through Copilot with no blocking findings.

Use clean committed before/after checkouts and the fixture from this PR. Build
four frozen artifacts with `examples/bench-bulk/run.py --build-only`, specifying
`--checkout`, fresh `--output` directories and a shared `--target-dir` (build
sequentially). For each revision build once normally and once with
`--native-allocations`. Set `CARGO_BUILD_JOBS=1`, `KOTLINC` and `JAVA_OPTS` equally.
Run no builds during timing:

```sh
python3 examples/bench-bulk/jni-callback-cleanup-results/count.py \
  --before /tmp/before-counting --after /tmp/after-counting \
  --output /tmp/fixed-counts --forks 2 \
  --cell plain:callback_record:1 --cell bulk:callback_record:1 \
  --cell plain:callback_records:1 --cell plain:callback_records:64 \
  --cell bulk:callback_records:64 --cell plain:control:1
python3 examples/bench-bulk/jni-callback-cleanup-results/compare.py \
  --before /tmp/before-timing --after /tmp/after-timing \
  --output /tmp/fixed-latency --forks 6 \
  --group plain:1:callback_owned_record,callback_record_return,control \
  --group bulk:1:callback_owned_record,callback_record_return,control \
  --group plain:1,64:callback_owned_records
python3 examples/bench-bulk/jni-callback-cleanup-results/compare.py \
  --before /tmp/before-timing --after /tmp/after-timing \
  --output /tmp/fixed-aa --aa --forks 4 \
  --group plain:1:callback_owned_record,callback_record_return,control \
  --group bulk:1:callback_owned_record
```
