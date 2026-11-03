# Kotlin collection fold cleanup measurements

This compares #824 after merging #859/#860 (`57cbaa3d7`) with the callback-fold cleanup implementation (`c0750a412`). Later commits record benchmark tooling, generated snapshots and these results. The reused baseline artifact was built at `839a14f45`; its generator, runtime and benchmark fixture sources were checked against the #824 baseline and are identical. [Artifact identity](artifact-identity.json) records full revisions and generated-Rust hashes.

## Change and scope

Collection callback returns first collect raw rows during JNI execution. Kotlin then reconstructs each row and calls the user's fold after the native invocation and handle locks finish. Previously a per-row cleanup lambda retained partially reconstructed wrappers. Cleanup now uses a lexical row catch. Fixed fields use their local wrapper variables without a journal. Nested handle collections reuse one per-invocation `CollectionOwners` journal; successful delivery clears wrapper references while retaining its storage. Failure cleanup closes the current row and unread rows, preserving earlier delivered rows and the original exception.

Resource-free rows emit no cleanup lambda, discard traversal, processed-row counter or cleanup catch. The old empty lambda was already shared by the JVM in this fixture: deleting its bookkeeping does not reduce measured primitive-record allocation.

Both generated benchmark Rust files (`plain.rs`, `bulk.rs`) are byte-identical to the baseline. The native libraries have different hashes; no native-machine-code identity claim is made. Measurements use the plain module's object-array inputs and either object-array outputs or collection callback folds. Bulk-module snapshots are also recorded, but bulk delivery is not measured here.

## JVM allocation

`ThreadMXBean.getThreadAllocatedBytes` measures JVM bytes allocated on the calling thread across complete warmed calls, including existing input preparation/locks, JNI-created JVM objects, reconstruction and the user fold. It does not count Rust allocations or allocations on other threads. Handle creation and final close are outside the samples; handles circulate through consuming round trips. No native allocator instrumentation is enabled.

`fold_flat` delivers N individual handles. `fold_nested` delivers N one-handle inner lists. `fold_wide_nested` delivers one inner list with N handles, checking the tradeoff when the journal has only one row. `primitive_fold` constructs and reduces N primitive records. Ordinary `flat`, `nested` and `primitive_array` outputs plus scalar `control` provide comparisons that do not use the changed fold cleanup.

The following values are medians of per-JVM byte totals divided by call count, across four forks:

| Callback fold shape | N handles/records | JVM bytes/call before → after | Saved bytes/call |
|---|---:|---:|---:|
| `fold_flat` | 1 | 648.0 → 608.0 | 40.0 |
| `fold_flat` | 512 | 116,048.0 → 95,568.0 | 20,480.0 |
| `fold_nested` | 64 | 64,105.3 → 55,059.5 | 9,045.8 |
| `fold_wide_nested` | 1 | 1,136.0 → 1,096.0 | 40.0 |
| `fold_wide_nested` | 512 | 108,344.0 → 108,288.0 | 56.0 |
| `primitive_fold` | 512 | 142,008.0 → 142,008.0 | 0.0 |

Flat handle folds save exactly 40 bytes per row in every measured fork: 20,480 bytes for 512 handles (17.6%). The 64-small-list case saves about 9 KB per call (about 14%); JIT allocation differences make its exact byte totals vary across forks. One wide inner list saves only tens of bytes because there is no repeated nested journal to eliminate. Primitive records remain at 142,008 bytes/call. Ordinary 512-handle object arrays remain at 97,592 bytes/call, and primitive object arrays at 168,016 bytes/call. Full byte ranges and every fork score are in [latency/results.json](latency/results.json).

## Timing protocol and initial comparison

Linux x86-64 on AMD EPYC 7713, Rust 1.98.0, Kotlin 2.4.21 and OpenJDK 11.0.29. Separate release libraries use the same dependency lock, fixture, Cargo profile and common driver JAR. The runner checks artifact hashes and matching build settings. One case per fresh JVM, one JVM at a time, pinned to CPUs 0–1; two active processors, fixed 256 MiB G1 heap. Warmup lasts at least two seconds and waits for 500 ms without compilation-time changes, with a 15-second cap. All runs settled. Twelve measured batches follow. Case order is shuffled deterministically, revision order alternates by fork, and no builds or test suites ran alongside timing. Other users' host activity, CPU frequency and turbo are not controlled.

The initial comparison has 11 cells × four forks × two revisions = 88 JVMs and 1,056 batches. Each JVM score divides total time by total operations; tables report the median and full range of those scores. All samples, GC events and slow forks are retained. Negative changes mean faster. Every number is for a complete call, not an individual row.

| Case | N | Before ns/call [range] | After ns/call [range] | Change |
|---|---:|---:|---:|---:|
| `control` | 1 | 12.6 [12.5–12.6] | 12.7 [12.6–12.9] | +0.72% |
| `flat` | 1 | 4,742.3 [4,621.4–4,953.1] | 4,572.8 [4,550.5–4,673.6] | -3.58% |
| `flat` | 512 | 1,973,715.3 [1,953,323.3–2,033,851.8] | 1,993,713.5 [1,983,265.2–2,324,771.2] | +1.01% |
| `fold_flat` | 1 | 3,374.9 [3,274.0–3,512.9] | 3,351.0 [3,308.7–3,538.8] | -0.71% |
| `fold_flat` | 512 | 1,075,712.8 [1,035,548.6–1,129,662.9] | 1,060,150.9 [1,030,524.2–1,594,282.1] | -1.45% |
| `fold_nested` | 64 | 402,496.2 [392,783.8–418,188.8] | 384,769.4 [384,376.2–398,864.6] | -4.40% |
| `fold_wide_nested` | 1 | 7,344.9 [7,172.9–7,619.9] | 7,191.9 [7,086.3–7,385.0] | -2.08% |
| `fold_wide_nested` | 512 | 1,989,291.9 [1,945,428.0–2,068,889.0] | 1,967,258.6 [1,946,121.0–2,022,313.0] | -1.11% |
| `nested` | 64 | 409,553.1 [402,029.2–423,183.2] | 427,953.5 [414,633.4–447,081.6] | +4.49% |
| `primitive_array` | 512 | 5,889,156.1 [5,763,600.5–6,908,742.9] | 5,678,192.6 [5,621,334.7–5,817,276.8] | -3.58% |
| `primitive_fold` | 512 | 148,980.2 [145,234.0–155,056.5] | 157,806.9 [148,487.1–167,245.0] | +5.92% |

The initial timing results are mixed: nested folds have a lower median, while primitive folds have a 5.92% higher median. Controls also move in both directions. These results alone do not establish a general speedup or a primitive-path regression. [Raw samples](latency/samples.csv).

## Identical-build controls

The exact baseline JAR/library pair runs under both labels with the same driver: three cells × four forks × two labels = 24 JVMs and 288 batches.

| Case | N | Before ns/call [range] | After ns/call [range] | Change |
|---|---:|---:|---:|---:|
| `fold_nested` | 64 | 400,860.7 [383,405.8–797,127.4] | 398,490.2 [383,004.3–408,559.2] | -0.59% |
| `nested` | 64 | 415,429.0 [401,075.6–424,359.2] | 419,576.9 [401,772.1–438,254.9] | +1.00% |
| `primitive_fold` | 512 | 151,423.3 [144,032.1–160,684.0] | 161,111.1 [145,397.3–177,564.1] | +6.40% |

The identical primitive fold moves by +6.40%, comparable to the initial +5.92% difference. This demonstrates timing variation; it does not prove any specific implementation has zero effect. [Control data](aa/results.json), [samples](aa/samples.csv).

## Six-fork confirmation

A separate before/after comparison repeats those three cells with six fresh JVMs per revision: 36 JVMs and 432 batches.

| Case | N | Before ns/call [range] | After ns/call [range] | Change |
|---|---:|---:|---:|---:|
| `fold_nested` | 64 | 429,404.0 [395,154.8–685,817.4] | 391,644.5 [385,814.3–400,647.3] | -8.79% |
| `nested` | 64 | 416,900.9 [404,049.8–486,545.7] | 410,870.0 [398,008.3–664,141.8] | -1.45% |
| `primitive_fold` | 512 | 156,754.5 [144,760.3–167,575.6] | 148,443.0 [143,323.0–159,275.0] | -5.30% |

The primitive-fold difference changes direction in confirmation, and ordinary nested outputs also change direction. Nested folds have a lower median in both before/after comparisons, with overlapping fork ranges. The established benefit is lower JVM allocation for handle-bearing folds; no uniform throughput-speedup claim is made. The confirmation again saves about 9 KB per 64-small-list call. [Confirmation data](confirmation/results.json), [samples](confirmation/samples.csv).

## Verification and review

Workspace tests, Clippy with warnings denied and formatting passed. All 270 generated example snapshots match. The C boundary callers, C++ header checks and complete Kotlin/JNI callers pass, with C ASan/UBSan and `-Xcheck:jni` enabled. Coverage includes partial construction, failure at every source-record constructor position, user-fold failure, close/take, reentry, optional/empty rows, mixed bulk frames, exact native destruction counts and a native destructor panic suppressed onto the original fold exception. Earlier delivered rows remain open after a later failure. New generator tests check resource-free omission, fixed-field lexical cleanup and one reused nested journal. The wide-nested benchmark also passed a separate diagnostic JNI run, excluded from timing: [diagnostic results](diagnostic/results.json).

[Claude Opus 5.5 review](review.md) found no source blockers; follow-ups were checked through the generated JNI boundary. Committed generated Kotlin shows the cleanup change, and generated Rust records the additional test fixtures.

## Reproduction

Use clean committed source trees: baseline `57cbaa3d7` and implementation `c0750a412` (or this PR's final tree, whose later changes only add benchmark tooling/artifacts). Build both with the existing [prepare.py](../jni-output-results/prepare.py), identical `KOTLINC`, `JAVA_OPTS` and `CARGO_BUILD_JOBS=1`, and fresh output directories. Compile the common [Driver.kt](../jni-output-results/Driver.kt) against the before JAR using `-Xfriend-paths`, then use the shared [measure.py](../jni-output-results/measure.py):

```sh
python3 examples/bench-bulk/jni-output-results/prepare.py \
  --checkout /path/to/before --output /tmp/fold-before --target /tmp/fold-build-cache
python3 examples/bench-bulk/jni-output-results/prepare.py \
  --checkout /path/to/after --output /tmp/fold-after --target /tmp/fold-build-cache
"$KOTLINC" -cp /tmp/fold-before/bench.jar -Xfriend-paths=/tmp/fold-before/bench.jar \
  examples/bench-bulk/jni-output-results/Driver.kt -d /tmp/fold-driver.jar
taskset -c 0,1 python3 examples/bench-bulk/jni-output-results/measure.py \
  --before /tmp/fold-before --after /tmp/fold-after --driver /tmp/fold-driver.jar \
  --output /tmp/fold-latency --forks 4 \
  --cell control:1 --cell primitive_array:512 --cell primitive_fold:512 \
  --cell flat:1 --cell flat:512 --cell fold_flat:1 --cell fold_flat:512 \
  --cell nested:64 --cell fold_nested:64 \
  --cell fold_wide_nested:1 --cell fold_wide_nested:512
```

For A/A controls, pass the exact before directory under both labels, `--cases primitive_fold,nested,fold_nested --forks 4`, and a fresh result directory. Confirmation uses the before/after directories with that case subset and `--forks 6`. JSON build metadata records full input revisions, compiler/profile settings and JAR/library hashes; result metadata records the driver/runner hashes and every JVM command. Raw sample CSVs include CPU time, JVM bytes and GC counts as well as wall time.
