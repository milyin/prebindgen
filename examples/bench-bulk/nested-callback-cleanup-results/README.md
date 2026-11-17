# Nested ordinary JNI callback cleanup

Kotlin callbacks returning ordinary collections of owned handles previously caused a JNI ownership pre-scan and temporary handle vector before normal decoding. The decoder now handles nested collections and records containing several such collections in one successful-call traversal. Failure or unwind triggers a depth-first cleanup scan through the private JVM arrays retained by the callback frame. Packed mixed layouts retain their existing guard.

Cleanup visits moved owners in decoder order, skips the consumed prefix, excludes borrowed handles and releases accessible unread owners in partial rows and sibling fields. Partial Rust values drop before cleanup. Null roots and absent/malformed columns are covered; per-read local frames bound JNI references. The existing rollback helper preserves pending JVM exceptions and retains its abort policy for unrecoverable JVM failures reading trusted storage. No new Kotlin copies or native alias checks are introduced.

Kotlin data-class declarations now accept `Vec<T>` fields as lists, including nested/optional elements and declared handles. They reuse existing sequence converters. Handle-bearing records still require explicit input expansions describing ownership; byte vectors retain `ByteArray`, and lists retain standard Kotlin list equality. The covertest exercises an explicit record input/output round trip as well as callback results.

## Compared artifacts

Base: #824 at `ba356ba67b37874bbd86447c7340b235b4c020ef`. Implementation: `cd11cf800a5c0b9fbe9d3dc48c020dab06c90eec`. Later commits record the standalone benchmark snapshot and measurements. The frozen baseline was built at `3782ddc26377907e5531611eab8cacc2e9cbe618`; its generator/runtime crates, root manifest and seven benchmark fixture files are byte-identical to the umbrella base. [results.json](results.json) records the path inventory and tree digest, compiler/fixture/artifact hashes, native counts and per-JVM timing medians. Raw console logs stay outside Git.

Both timing libraries use the exact same baseline JAR after verifying identical generated Kotlin sources and fixture hashes. Generated `bulk.rs` is unchanged. Native allocation counts use separate instrumented release libraries; latency builds have instrumentation disabled. No builds or tests run during timing.

## Native allocations

Counts cover complete calls on the calling Rust thread, with two fresh JVMs per version, 1,000 warmup calls and ten measured calls per JVM. All twenty observations agree. Handle creation and final disposal are outside the samples; returned handles circulate through consuming calls. Requested bytes include growth reallocations and are not peak memory.

| Transport / result shape | Handles | Allocations before → after | Reallocations before → after | Requested bytes before → after |
| --- | ---: | ---: | ---: | ---: |
| bulk / one large inner list | 512 | 519.0 → 519.0 | 4.0 → 4.0 | 16,746.0 → 16,746.0 |
| bulk / singleton inner lists | 512 | 1,030.0 → 1,030.0 | 4.0 → 4.0 | 57,506.0 → 57,506.0 |
| plain / one large inner list | 0 | 8.0 → 8.0 | 6.0 → 6.0 | 286.0 → 286.0 |
| plain / one large inner list | 1 | 11.0 → 10.0 | 6.0 → 6.0 | 398.0 → 302.0 |
| plain / one large inner list | 8 | 18.0 → 17.0 | 7.0 → 6.0 | 702.0 → 414.0 |
| plain / one large inner list | 64 | 74.0 → 73.0 | 10.0 → 6.0 | 4,286.0 → 1,310.0 |
| plain / one large inner list | 512 | 522.0 → 521.0 | 13.0 → 6.0 | 32,958.0 → 8,478.0 |
| plain / one large inner list | 4,096 | 4,106.0 → 4,105.0 | 16.0 → 6.0 | 262,334.0 → 65,822.0 |
| plain / singleton inner lists | 0 | 4.0 → 4.0 | 3.0 → 3.0 | 115.0 → 115.0 |
| plain / singleton inner lists | 1 | 9.0 → 8.0 | 4.0 → 4.0 | 284.0 → 188.0 |
| plain / singleton inner lists | 8 | 30.0 → 29.0 | 12.0 → 11.0 | 987.0 → 699.0 |
| plain / singleton inner lists | 64 | 198.0 → 197.0 | 71.0 → 67.0 | 7,763.0 → 4,787.0 |
| plain / singleton inner lists | 512 | 1,542.0 → 1,541.0 | 522.0 → 515.0 | 61,971.0 → 37,491.0 |
| plain / singleton inner lists | 4,096 | 12,294.0 → 12,293.0 | 4,109.0 → 4,099.0 | 495,635.0 → 299,123.0 |
| plain / fixed callback record control | 1 | 4.0 → 4.0 | 2.0 → 2.0 | 90.0 → 90.0 |
| plain / flat callback array control | 512 | 517.0 → 517.0 | 3.0 → 3.0 | 12,403.0 → 12,403.0 |
| plain / primitive scalar control | 1 | 0.0 → 0.0 | 0.0 → 0.0 | 0.0 → 0.0 |

Final Rust vectors, inner-list allocations and returned native handles remain; this change removes the auxiliary ownership-tracking storage. Empty results and unchanged controls retain their allocation counts.

## Complete-call latency

The existing benchmarks return a Kotlin list of existing handles to Rust through a callback, then return the reconstructed vector to Kotlin for reuse. One shape has a large inner list between two empty lists; the other has one singleton list per handle. Preparation and locks, JNI, conversion, output reconstruction, GC and checksum/read calls remain timed. These are binding microbenchmarks, not packet-throughput measurements.

Each entry is the median of six fresh-JVM medians per version, with ten warmup batches and eleven measured batches targeting 30 ms. Version order alternates. Change is `after median / before median - 1`, comparing generator revisions using the same transport. The JSON also retains paired ratios and JVM allocation observations.

| Transport / case | Handles | Before µs/call | After µs/call | Change |
| --- | ---: | ---: | ---: | ---: |
| bulk / singleton inner lists | 1 | 2.327 | 2.406 | +3.4% |
| bulk / singleton inner lists | 512 | 148.967 | 141.133 | -5.3% |
| bulk / one large inner list | 1 | 2.649 | 2.756 | +4.0% |
| bulk / one large inner list | 512 | 60.751 | 58.757 | -3.3% |
| plain / singleton inner lists | 0 | 1.476 | 1.474 | -0.1% |
| plain / singleton inner lists | 1 | 4.791 | 3.818 | -20.3% |
| plain / singleton inner lists | 8 | 23.418 | 18.705 | -20.1% |
| plain / singleton inner lists | 64 | 164.782 | 130.444 | -20.8% |
| plain / singleton inner lists | 512 | 1263.891 | 990.288 | -21.6% |
| plain / singleton inner lists | 4,096 | 10064.715 | 7962.094 | -20.9% |
| plain / one large inner list | 0 | 6.103 | 5.117 | -16.2% |
| plain / one large inner list | 1 | 7.337 | 6.146 | -16.2% |
| plain / one large inner list | 8 | 15.345 | 11.038 | -28.1% |
| plain / one large inner list | 64 | 76.086 | 52.165 | -31.4% |
| plain / one large inner list | 512 | 563.533 | 358.674 | -36.4% |
| plain / one large inner list | 4,096 | 4390.893 | 2874.131 | -34.5% |
| plain / flat callback array control | 1 | 2.855 | 2.682 | -6.0% |
| plain / flat callback array control | 512 | 503.792 | 477.031 | -5.3% |
| plain / primitive callback record control | 1 | 2.112 | 2.096 | -0.7% |
| plain / primitive scalar control | 1 | 0.014 | 0.014 | -0.8% |
| plain / ordinary singleton-list control | 1 | 2.986 | 2.987 | +0.1% |
| plain / ordinary singleton-list control | 512 | 1008.136 | 1005.307 | -0.3% |
| plain / ordinary nested-call control | 1 | 5.313 | 5.474 | +3.0% |
| plain / ordinary nested-call control | 512 | 362.725 | 371.971 | +2.5% |

Identical-library controls load the same baseline native library and JAR under both labels, with four JVMs per label and the same warmup/batch settings:

| Transport / case | Handles | Before µs/call | After µs/call | Apparent change |
| --- | ---: | ---: | ---: | ---: |
| bulk / singleton inner lists | 512 | 150.996 | 146.780 | -2.8% |
| bulk / one large inner list | 512 | 59.424 | 65.093 | +9.5% |
| plain / singleton inner lists | 1 | 4.735 | 4.574 | -3.4% |
| plain / singleton inner lists | 512 | 1311.638 | 1328.852 | +1.3% |
| plain / singleton inner lists | 4,096 | 10325.552 | 10485.955 | +1.6% |
| plain / one large inner list | 1 | 7.315 | 7.476 | +2.2% |
| plain / one large inner list | 512 | 571.547 | 556.049 | -2.7% |
| plain / one large inner list | 4,096 | 4463.365 | 4592.692 | +2.9% |
| plain / primitive scalar control | 1 | 0.014 | 0.014 | -2.3% |

For 512 handles, the ordinary single-large-list and singleton-list callback shapes improve 36.4% and 21.6%; their paired improvements are 35.8% and 21.7%. Identical-library apparent changes for the same cells are −2.7% and +1.3%. At 4,096 handles, ordinary improvements are 34.5% and 20.9%, versus identical-library changes of +2.9% and +1.5%. The one-handle cases improve 16.2% and 20.3%, versus +2.2% and −3.4% in their identical-library controls.

Unchanged controls vary: flat callback results measure about 5–6% faster, ordinary nested functions range from 0.3% faster to 3.0% slower, and packed callback controls range from 5.3% faster to 4.0% slower. The packed single-large-list identical-library control varies by 9.5% in the ratio of medians (2.4% paired), so those control differences do not establish a separate optimization or regression. Only ordinary nested callback ownership scanning changed. Empty singleton-list output is unchanged; the other zero-handle case still contains three empty inner lists, whose eager traversal is also removed.

## Verification and reproduction

Generated Rust/JNI and Kotlin tests cover nested optional collections, absent versus empty inner lists, records mixing moved/borrowed handles with several collections, explicit record inputs, partial inner rows, invalid scalars/pointers, wrong or short arrays, null roots, source-constructor panics, cleanup before retry and native workers. Workspace tests/doctests, all-target Clippy with warnings denied, C ASan/UBSan and Kotlin/Java boundary checks, formatting and generated artifacts pass. Final changed JNI code also passes the full JVM covertest with `-Xcheck:jni`; instrumented benchmark smoke runs use the same check. Claude Opus 5.5 reviewed the implementation and final JNI reference handling without blocking findings.

Build clean committed checkouts sequentially with `examples/bench-bulk/run.py --build-only`, passing `--checkout`, fresh `--output` directories and a shared `--target-dir`. Use identical `CARGO_BUILD_JOBS=1`, `KOTLINC` and `JAVA_OPTS`. Build each revision normally and again with `--native-allocations`. Then run without concurrent builds:

```sh
python3 examples/bench-bulk/jni-callback-cleanup-results/count.py \
  --before /tmp/before-counting --after /tmp/after-counting \
  --output /tmp/nested-counts --forks 2 \
  --cell plain:callback_nested:512 --cell plain:callback_nested_many:512 \
  --cell plain:callback_nested:4096 --cell plain:callback_nested_many:4096 \
  --cell bulk:callback_nested:512 --cell bulk:callback_nested_many:512 \
  --cell plain:callback_records:512 --cell plain:callback_record:1 --cell plain:control:1
python3 examples/bench-bulk/jni-callback-cleanup-results/compare.py \
  --before /tmp/before-timing --after /tmp/after-timing \
  --output /tmp/nested-latency --forks 6 \
  --group plain:0,1,8,64,512,4096:callback_nested_owned_handles,callback_many_nested_owned_handles \
  --group bulk:1,512:callback_nested_owned_handles,callback_many_nested_owned_handles \
  --group plain:1,512:callback_owned_records,nested_owned_handles_roundtrip,nested_many_owned_handles_roundtrip \
  --group plain:1:control,callback_record_return
python3 examples/bench-bulk/jni-callback-cleanup-results/compare.py \
  --before /tmp/before-timing --after /tmp/after-timing \
  --output /tmp/nested-aa --aa --forks 4 \
  --group plain:1,512,4096:callback_nested_owned_handles,callback_many_nested_owned_handles \
  --group bulk:512:callback_nested_owned_handles,callback_many_nested_owned_handles \
  --group plain:1:control
```
