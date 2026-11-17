# Mixed ordinary/bulk JNI callback results

An ordinary JVM object-array result can contain records whose selected fields use bulk packing. The previous generator emitted per-row bulk caches before their row bindings existed and hid row field operands in template markers. Generated Rust failed to compile. This PR decodes each buffer in its row scope and makes the operands available to row composition. The same fix covers ordinary function inputs.

Callback success also avoids the eager ownership pre-scan and temporary cleanup vector. Failure cleanup traverses the retained private JVM arrays and delegates packed children to the existing native bulk scanner. The scan runs after partial Rust values drop, skips the consumed-owner prefix, excludes borrowed fields and continues through accessible unread siblings. A malformed declared used length cannot hide readable owners: failure cleanup reads the actual byte array. Generated packers allocate fresh zeroed carriers per occurrence and never pool/reuse them. Normal decoding still validates and reads the declared length. Unrecoverable JVM cleanup failures retain the existing abort policy. Obsolete private eager scan helpers are removed.

## Experiment

An owned `HandleBatch` contains an owned guard handle and `Vec<HandleRecord>`; each inner record contains an owned handle and an integer tag. The callback returns the current Kotlin batches to Rust, and Rust returns the same logical value to Kotlin for reuse. All three input transports use the same fully packed Rust-to-Kotlin output, so output transport stays constant:

- **ordinary:** outer and inner lists use JVM object arrays.
- **mixed:** outer rows use JVM object arrays; each inner list has its own packed byte array.
- **packed:** the complete outer tree uses one packed byte array.

The large-inner-list shape has one batch with N records (N+1 handles). The singleton-list shape has N batches of one record (2N handles). At N=0, the former still contains one guard-owning empty batch; the latter has no batches. The table row count is N, not the total number of handles. Input preparation/locks, callback JNI, conversion, fully packed output reconstruction, GC and checksum/read calls remain timed. These are complete binding calls rather than packet-throughput measurements.

The old mixed path does not compile, so the primary tables compare supported transports within the new generator, not before/after versions of mixed handling. Counts use a separate instrumented release library; latency uses an uninstrumented library. Each timing cell pools eight fresh-JVM medians (four per identical-library label), with ten warmup and eleven measured batches targeting 30 ms. Ordinary and mixed cases run in the same JVM, with case order rotated; fully packed cases run in separate JVMs. All measured JVMs use `-Xss16m` as an explicit command-line option: 4,096 singleton batches contain 8,192 handles and exceed the existing recursive Kotlin lock helper’s default stack. The partial default-stack and environment-only stack count attempts are excluded. No locking code changes are included. No builds or tests run during measurement.

Base: #824 at `d97072f2a38b600e8e43e254a652a4854c069b35`. Measured implementation: `6c71de8926f4e911ebfeb7c9b3968e9c3e365ca6`. A later commit records generated benchmark wrappers. [results.json](results.json) retains compiler, fixture, artifact and source-tree hashes, native counts, per-JVM medians, paired transport ratios and same-library variation. Raw logs remain outside Git.

## Complete-call latency

Change compares mixed with ordinary: `mixed median / ordinary median - 1`. A positive value means mixed takes longer.

| Shape | Inner records | Ordinary µs | Mixed µs | Fully packed µs | Mixed vs ordinary |
| --- | ---: | ---: | ---: | ---: | ---: |
| large | 0 | 2.216 | 2.710 | 2.262 | +22.3% |
| large | 1 | 3.099 | 3.035 | 2.520 | -2.1% |
| large | 8 | 7.978 | 4.010 | 3.255 | -49.7% |
| large | 64 | 49.440 | 12.514 | 11.303 | -74.7% |
| large | 512 | 370.266 | 68.009 | 58.526 | -81.6% |
| large | 4,096 | 2815.890 | 497.712 | 432.058 | -82.3% |
| many | 0 | 1.197 | 1.199 | 1.803 | +0.1% |
| many | 1 | 3.106 | 3.110 | 2.461 | +0.1% |
| many | 8 | 12.948 | 12.211 | 4.737 | -5.7% |
| many | 64 | 98.677 | 90.166 | 27.539 | -8.6% |
| many | 512 | 764.261 | 713.258 | 202.851 | -6.7% |
| many | 4,096 | 6177.680 | 5819.694 | 1748.798 | -5.8% |

For one large inner list, mixed packing lowers median complete-call latency by 81.6% at 512 records (5.44× ordinary/mixed) and 82.3% at 4,096 records. The paired changes are −81.1% and −82.6%; identical-label variation in the same cells is much smaller. Fully packed input remains faster: 58.5 versus 68.0 µs at 512 records.

Singleton-list gains are modest: 6.7% at 512 batches and 5.8% at 4,096, with paired changes of −5.9% and −4.7%. The 512-batch mixed identical-label change is −5.4%, close to the size of this gain; treat it as workload-dependent rather than a broad acceleration claim. Fully packed input is substantially faster for these shapes: 202.9 versus 713.3 µs at 512 batches.

The empty inner list provides the expected counterexample: mixed takes 2.710 versus 2.216 µs, 22.3% longer, and allocates more. A single inner record is roughly equal between ordinary and mixed. Bulk packing is useful for grouping enough expanded fields; a separate byte-array carrier per tiny inner list has a real cost.

## Allocations

Calling-thread Rust counts use two fresh JVMs per cell, 1,000 warmup calls and ten measured calls per JVM; all twenty observations agree. Initial handle creation and final disposal are outside the samples. Allocation counts include necessary output handle allocations and collection/buffer storage. Requested bytes include growth reallocations and are not peak memory. JVM values below are medians of calling-thread allocated bytes during the timing runs.

| Shape | Inner records | Rust allocations ordinary / mixed / packed | Rust reallocations ordinary / mixed / packed | Requested Rust bytes ordinary / mixed / packed | JVM bytes ordinary / mixed / packed |
| --- | ---: | ---: | ---: | ---: | ---: |
| large | 0 | 6 / 7 / 7 | 3 / 3 / 3 | 178 / 194 / 218 | 848 / 904 / 904 |
| large | 1 | 8 / 9 / 9 | 4 / 4 / 4 | 282 / 314 / 338 | 1,240 / 1,360 / 1,192 |
| large | 8 | 15 / 16 / 16 | 4 / 4 / 4 | 538 / 682 / 706 | 2,280 / 2,176 / 1,960 |
| large | 64 | 71 / 72 / 72 | 4 / 4 / 4 | 2,778 / 3,818 / 3,842 | 14,128 / 12,008 / 10,424 |
| large | 512 | 519 / 520 / 520 | 4 / 4 / 4 | 20,698 / 28,906 / 28,930 | 112,048 / 84,584 / 73,600 |
| large | 4,096 | 4,103 / 4,104 / 4,104 | 4 / 4 / 4 | 164,058 / 229,610 / 229,634 | 902,318 / 659,040 / 575,888 |
| many | 0 | 4 / 4 / 5 | 2 / 2 / 2 | 98 / 98 / 114 | 288 / 296 / 352 |
| many | 1 | 8 / 9 / 9 | 4 / 4 / 4 | 282 / 314 / 338 | 1,240 / 1,360 / 1,192 |
| many | 8 | 29 / 37 / 30 | 4 / 4 / 4 | 1,234 / 1,490 / 1,570 | 6,360 / 7,264 / 4,184 |
| many | 64 | 197 / 261 / 198 | 4 / 4 / 4 | 8,850 / 10,898 / 11,426 | 48,280 / 55,456 / 28,856 |
| many | 512 | 1,541 / 2,053 / 1,542 | 4 / 4 / 4 | 69,778 / 86,162 / 90,274 | 378,480 / 435,832 / 221,072 |
| many | 4,096 | 12,293 / 16,389 / 12,294 | 4 / 4 / 4 | 557,202 / 688,274 / 721,058 | 3,008,544 / 3,467,304 / 1,747,264 |

Each mixed inner-list carrier requires its own JNI byte-array access and native byte buffer. That required marshalling cost grows with the number of outer batches. The fully packed case accesses one whole-tree carrier. Neither mixed nor ordinary successful callback decoding allocates a separate ownership-cleanup vector.

## Existing-path regression controls

The unchanged prior fixture is built against both generator versions and run with the exact same baseline JAR. Fixture hashes, generated Kotlin source hashes, compiler/profile and dependency lock match. The frozen baseline implementation is source-identical to the #824 base for the generator/runtime crates and root manifest; the new control build is source-identical to the measured candidate for those paths. Rebuilding the unchanged fixture uses clean generated Kotlin output. The report records both equivalence checks. These controls assess existing fully packed callbacks whose native cache expression changed.

Each before/after timing and identical-library control has four fresh JVMs per label and the same warmup/batch settings.

| Existing bulk case | Rows | Before µs | After µs | Change |
| --- | ---: | ---: | ---: | ---: |
| callback_many_nested_owned_handles | 1 | 2.329 | 2.320 | -0.4% |
| callback_many_nested_owned_handles | 512 | 139.704 | 144.884 | +3.7% |
| callback_nested_owned_handles | 1 | 2.595 | 2.604 | +0.3% |
| callback_nested_owned_handles | 512 | 55.208 | 53.907 | -2.4% |
| callback_owned_record | 1 | 2.064 | 2.084 | +1.0% |
| callback_owned_records | 1 | 1.997 | 2.000 | +0.1% |
| callback_owned_records | 512 | 62.709 | 61.889 | -1.3% |
| control | 1 | 0.013 | 0.014 | +2.0% |

Identical-library controls load the same baseline JAR and native library under both labels:

| Existing bulk case | Rows | Before µs | After µs | Apparent change |
| --- | ---: | ---: | ---: | ---: |
| callback_many_nested_owned_handles | 512 | 137.347 | 136.372 | -0.7% |
| callback_nested_owned_handles | 512 | 61.413 | 57.024 | -7.1% |
| callback_owned_record | 1 | 2.054 | 2.059 | +0.2% |
| callback_owned_records | 512 | 62.308 | 60.723 | -2.5% |
| control | 1 | 0.013 | 0.014 | +2.2% |

Existing packed controls range from 2.4% lower to 3.7% higher median latency. The 512-singleton-list callback control measures 3.7% higher (4.4% paired); that result is retained. Other controls move in both directions, and identical-library controls range from −7.1% to +2.2%. These runs do not establish the cause of the small changes or a general acceleration of existing paths. The primary result is correct mixed composition with the measured transport trade-offs above.

## Correctness and reproduction

Workspace tests/doctests, all-target Clippy with warnings denied, C ASan/UBSan and C++ header checks, actual Kotlin/JNI with `-Xcheck:jni`, Java boundary tests and generated-artifact verification pass. Audited native allocation identities verify no missing or duplicate drop across moved/borrowed fields, optional ordinary owners, nested optional lists, malformed packed ranges/tags/scalars/pointers/lengths, partial rows, source-constructor panics, current/unread siblings, retries and worker threads. Ordinary function failures report the accepted prefix and leave the unread suffix owned by Kotlin. Scalar/text-only mixed input and existing selector-arity diagnostics are covered. The identity audit exists only in the correctness fixture, not in the benchmark or generated runtime. Claude Opus 5.5 reviewed the source and resolved the fresh-carrier and owner-order questions against the passing tests.

Build clean committed checkouts with `examples/bench-bulk/run.py --build-only`, once normally and once with `--native-allocations`, using identical `CARGO_BUILD_JOBS=1`, `KOTLINC` and `JAVA_OPTS`. Use distinct target caches for changed and unchanged fixtures (or clear stale generated Kotlin output before switching fixtures). For regression controls, invoke the unchanged base fixture’s run.py with `--checkout` selecting each generator revision. Then run without concurrent builds:

```sh
python3 examples/bench-bulk/jni-callback-cleanup-results/count.py \
  --before /tmp/new-counting --after /tmp/new-counting --output /tmp/mixed-counts --forks 1 --java-flag=-Xss16m \
  --cell plain:callback_batches:512 --cell plain:callback_mixed_batches:512 \
  --cell bulk:callback_batches:512 --cell plain:callback_batches_many:512 \
  --cell plain:callback_mixed_batches_many:512 --cell bulk:callback_batches_many:512
python3 examples/bench-bulk/jni-callback-cleanup-results/compare.py \
  --before /tmp/new-timing --after /tmp/new-timing --output /tmp/mixed-latency --aa --forks 4 --java-flag=-Xss16m \
  --group plain:0,1,8,64,512,4096:callback_batches_large,callback_mixed_batches_large,callback_batches_many,callback_mixed_batches_many \
  --group bulk:0,1,8,64,512,4096:callback_batches_large,callback_batches_many
python3 examples/bench-bulk/jni-callback-cleanup-results/compare.py \
  --before /tmp/base-timing --after /tmp/new-generator-base-fixture --output /tmp/mixed-controls --forks 4 --java-flag=-Xss16m \
  --group bulk:1,512:callback_owned_records,callback_nested_owned_handles,callback_many_nested_owned_handles \
  --group bulk:1:callback_owned_record,control
```
