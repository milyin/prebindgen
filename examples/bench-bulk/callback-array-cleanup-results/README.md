# Ordinary JNI callback collection cleanup

Kotlin callbacks returning collections of owned records to Rust previously caused
two JNI traversals: one collected handle addresses in a cleanup vector, then the
normal decoder read those fields again to construct Rust records. Eligible
results now decode once on success. The existing consumed-owner counter remains;
only an error or panic triggers a rescan and cleanup vector.

Kotlin already creates private wire row arrays under its input preparation and
locking protocol. The callback's JNI frame retains these arrays. The change adds
no marshalling copy, public API, or Kotlin code to the benchmark. It initially
covers one object-array carrier with flat row fields, including optional and
borrowed handles. Nested collections, mixed bulk layouts and multiple carriers
retain their existing paths. Bulk results and fixed records keep their existing
failure-only cleanup.

Decoding errors and unwinds drop partial Rust values before rescanning. Cleanup
skips handles already consumed by Rust, excludes borrowed fields and reclaims
valid owners in the partial row and later rows. The existing JNI rollback helper
suspends/restores a pending JVM exception. Invalid fields and malformed short,
extra or wrong-class rows do not prevent cleanup of accessible valid siblings.
An unrecoverable JVM failure while reading trusted cleanup storage uses the
existing rollback helper's abort policy.

## Compared artifacts

Base: #824 at `c7b5f8feebbe0a2119ebb08083609291f372b629`.
Implementation: `3782ddc26377907e5531611eab8cacc2e9cbe618`.
Later commits add benchmark snapshots and measurements. The frozen baseline was
built at `970ac43a1`; its generator, runtime, root manifest and benchmark fixture
are byte-identical to the #824 base. [results.json](results.json) records both
revisions, the compared Git paths and their tree digest, artifact hashes,
compilers, fixture hashes and per-JVM medians. Raw console logs stay outside Git.

Both timing revisions run the exact same baseline JAR after verifying matching
generated Kotlin source hashes. Generated `bulk.rs` is byte-identical; rebuilding
the complete native library can still affect machine-code placement. Timing
libraries have native allocation instrumentation disabled; counts use separate
instrumented release libraries. No builds or tests run during measurement.

## Native allocations

The Rust allocator is counted on the calling thread for complete calls. Each cell
has two fresh JVMs, 1,000 warmup calls and ten measured calls per JVM. All twenty
observations per version agree. Initial handle creation and final disposal are
outside the samples; the returned handles circulate through consuming calls.
Requested bytes include growth reallocation requests and are not peak memory.

| Transport / callback result | Rows | Allocations before → after | Reallocations before → after | Requested bytes before → after |
| --- | ---: | ---: | ---: | ---: |
| bulk / handle/tag array | 512 | 518 → 518 | 3 → 3 | 28,802 → 28,802 |
| plain / fixed handle/tag record | 1 | 4 → 4 | 2 → 2 | 90 → 90 |
| plain / handle/tag array | 0 | 4 → 4 | 3 → 3 | 115 → 115 |
| plain / handle/tag array | 1 | 7 → 6 | 3 → 3 | 235 → 139 |
| plain / handle/tag array | 8 | 14 → 13 | 4 → 3 | 595 → 307 |
| plain / handle/tag array | 64 | 70 → 69 | 7 → 3 | 4,627 → 1,651 |
| plain / handle/tag array | 512 | 518 → 517 | 10 → 3 | 36,883 → 12,403 |
| plain / handle/tag array | 4096 | 4,102 → 4,101 | 13 → 3 | 294,931 → 98,419 |
| plain / primitive scalar control | 1 | 0 → 0 | 0 → 0 | 0 → 0 |

For 512 ordinary rows this removes one allocation, seven growth reallocations
and 24,480 cumulative requested bytes per call. At 4,096 rows the saving is one
allocation, ten reallocations and 196,512 requested bytes. The final Rust vector,
returned native handle allocations and other required conversion storage remain.
Empty arrays, bulk arrays and fixed-record/scalar controls retain their counts.

## Complete-call latency

The unchanged `callback_owned_records` benchmark returns an existing handle/tag
list from Kotlin to Rust, then returns the reconstructed Rust vector to Kotlin
for reuse by the next call. Input preparation/locks, JNI, native conversion,
output reconstruction, GC and checksum/read calls remain timed. The ordinary
input/output round trip, fixed handle record, primitive callback record and
scalar control do not use the changed ownership traversal.

Each timing entry is the median of six fresh-JVM medians per version, with ten
warmup batches and eleven measured batches targeting 30 ms. Version order
alternates. Change is `after median / before median - 1`; this compares generator
revisions using the same transport, rather than plain against bulk. The JSON
also retains paired ratios and calling-thread JVM allocation observations.

| Transport / case | Rows | Before µs/call | After µs/call | Change |
| --- | ---: | ---: | ---: | ---: |
| bulk / handle/tag callback array | 1 | 2.341 | 2.295 | -2.0% |
| bulk / handle/tag callback array | 512 | 61.889 | 61.756 | -0.2% |
| plain / fixed handle/tag record | 1 | 1.616 | 1.570 | -2.8% |
| plain / handle/tag callback array | 0 | 1.512 | 1.448 | -4.2% |
| plain / handle/tag callback array | 1 | 3.209 | 2.763 | -13.9% |
| plain / handle/tag callback array | 8 | 12.800 | 9.482 | -25.9% |
| plain / handle/tag callback array | 64 | 84.798 | 63.869 | -24.7% |
| plain / handle/tag callback array | 512 | 663.255 | 503.712 | -24.1% |
| plain / handle/tag callback array | 4096 | 5388.717 | 4060.126 | -24.7% |
| plain / primitive callback record | 1 | 2.046 | 2.030 | -0.8% |
| plain / primitive scalar control | 1 | 0.014 | 0.013 | -3.6% |
| plain / ordinary handle/tag round trip | 1 | 2.206 | 2.200 | -0.2% |
| plain / ordinary handle/tag round trip | 512 | 527.614 | 530.810 | +0.6% |

Identical-library controls load the same baseline native library and JAR under
both labels, with four JVMs per label and the same warmup/batch settings:

| Transport / case | Rows | Before µs/call | After µs/call | Apparent change |
| --- | ---: | ---: | ---: | ---: |
| bulk / handle/tag callback array | 512 | 63.296 | 64.570 | +2.0% |
| plain / handle/tag callback array | 1 | 3.145 | 3.145 | -0.0% |
| plain / handle/tag callback array | 512 | 655.592 | 659.596 | +0.6% |
| plain / handle/tag callback array | 4096 | 5303.861 | 5326.336 | +0.4% |
| plain / primitive scalar control | 1 | 0.013 | 0.013 | +0.4% |

Nonempty ordinary callback arrays improve 13.9–25.9% in this synthetic complete-call
benchmark. At 512 rows, the paired improvement is 23.8%, compared with a paired
0.3% shift in the identical-library control. At 4,096 rows those figures are
24.5% and 1.3%. The unchanged 512-row ordinary-function control measures 0.6%
slower, and the bulk callback control 0.2% faster; these are small variations.
The empty-array 4.2% change is not evidence of a separate optimization: empty
results already allocated no cleanup vector. These are binding microbenchmarks,
not network-packet throughput measurements.

## Correctness and reproduction

Actual generated Rust/JNI and Kotlin tests cover empty/optional arrays, multiple
and optional owned fields, explicit borrowed-field conversion, malformed boxed
scalars/handles, misaligned pointers, short/extra/wrong row shapes, missing optional
controls, partial construction, constructor panic, cleanup before retry and
native workers. Kotlin rejects the same moved handle appearing in different
rows before ownership transfer. The full C/Kotlin/Java boundary suites, workspace
tests/doctests, Clippy with warnings denied, formatting and generated artifacts
pass. Separate instrumented benchmark correctness runs pass with `-Xcheck:jni`.
Claude Opus 5.5 reviewed the source/diff and final error handling through Copilot
with no blocking findings.

Build clean committed checkouts with `examples/bench-bulk/run.py --build-only`,
passing `--checkout`, fresh `--output` directories and a shared `--target-dir`.
Build sequentially with identical `CARGO_BUILD_JOBS=1`, `KOTLINC` and `JAVA_OPTS`.
Build each revision once normally and once with `--native-allocations`. Use the
same fixture. Run no builds during timing:

```sh
python3 examples/bench-bulk/jni-callback-cleanup-results/count.py \
  --before /tmp/before-counting --after /tmp/after-counting \
  --output /tmp/array-counts --forks 2 \
  --cell plain:callback_records:0 --cell plain:callback_records:1 \
  --cell plain:callback_records:8 --cell plain:callback_records:64 \
  --cell plain:callback_records:512 --cell plain:callback_records:4096 \
  --cell bulk:callback_records:512 --cell plain:callback_record:1 --cell plain:control:1
python3 examples/bench-bulk/jni-callback-cleanup-results/compare.py \
  --before /tmp/before-timing --after /tmp/after-timing \
  --output /tmp/array-latency --forks 6 \
  --group plain:0,1,8,64,512,4096:callback_owned_records \
  --group bulk:1,512:callback_owned_records \
  --group plain:1:callback_owned_record,callback_record_return,control \
  --group plain:1,512:owned_handle_records_roundtrip
python3 examples/bench-bulk/jni-callback-cleanup-results/compare.py \
  --before /tmp/before-timing --after /tmp/after-timing \
  --output /tmp/array-aa --aa --forks 4 \
  --group plain:1,512,4096:callback_owned_records \
  --group bulk:512:callback_owned_records --group plain:1:control
```
