# Scalar presence for collection callback returns

Collection callback returns previously delivered all raw rows to Kotlin and then
allocated an empty JVM Object[] only to report that the collection was present.
The generated JNI method now returns an Int: 0 for a present collection, including
Some(empty), and -1 for None. The Kotlin wrapper folds the retained rows only on
successful present returns, after JNI execution and handle locks finish.

The implementation reuses the existing selector wire, integer option niche and
direct-return machinery. It adds no public declaration or runtime-library type.
Row method signatures, collector capacity policy, per-row ownership transfers,
acknowledgement BooleanArray and failure cleanup are unchanged. Arrays inside
rows still use their configured transport. Direct source callbacks are unchanged.

Custom or wide signatures that require the existing outer result packet retain
that packet. Its collection status is a boxed 0 or -1 through Integer.valueOf,
which uses the JVM small-integer cache; no separate marker array is allocated.
Low-level code manually constructing a JniSequence::Callback wire tree must now
use a Selector control instead of Rows. Regenerate native and Kotlin artifacts
together, as for any generated JNI signature change.

A fold has one nullable accumulator, so nested outer Option<Option<Vec<T>>>
remains unsupported. Generation now diagnoses that shape explicitly: the new
integer niche must not silently flatten None and Some(None). Optional row fields
and nested collections keep their existing support.

Binding errors and source errors return their handlers before any presence read
or fold cleanup. Pending JVM exceptions discard accepted raw rows through the
unchanged acknowledgement path. Fold/constructor exceptions clean up from the
current row onward only in the success branch. Native conversion failure leaves
accepted rows to Kotlin and unaccepted rows to Rust. No new error-path restoration
or happy-path validation was introduced.

## Measurement method

Baseline artifacts were built from `2c8c1e809` for #866. All nine generator/runtime
source trees exactly match merged #824 at `d983606b7`; [source-identity.json](source-identity.json)
records the comparison. The new artifacts were built from clean `7f30f90fd`.
Later commits record generated benchmark bindings and results only.

Both use the unchanged `jni-output-results/Driver.kt` driver and the same fixture,
dependency lock, compiler profile and environment, checked by the existing runner.
Rust 1.98.0, Kotlin 2.4.21, OpenJDK 11.0.29, Linux x86-64, CPU affinity 0,1, G1 and
a fixed 256 MiB JVM heap. Timing runs have no concurrent builds/tests. Each case
uses four fresh JVMs per version with alternating before/after order. Each JVM
warms for at least two seconds and until JVM compilation has been quiet for
500 ms, then records twelve batches. A process score is total time or allocated
bytes divided by total calls; reported values are medians and full min–max ranges
of process scores. No runs or slow samples are discarded.

`primitive_fold` returns and folds records with eight i64 fields. `fold_flat`
consumes a handle list and returns handles through collection callbacks;
`fold_nested` does the same for one-handle inner lists. Handle cases include
unchanged input conversion/locking. `control` is an unchanged scalar JNI call;
`primitive_array` is unchanged ordinary object-array output. Row count is list
length. All comparisons are the same complete call before/after this change,
including destination allocation and user folds; they are not packet-throughput
measurements or isolated JNI-call timings.

## Before/after results

| Case | Rows | Before µs [min–max] | After µs [min–max] | Time change | JVM bytes/call before → after |
|---|---:|---:|---:|---:|---:|
| `control` | 1 | 0.013 [0.013–0.015] | 0.013 [0.013–0.013] | -3.79% | 0.0 → 0.0 |
| `fold_flat` | 0 | 1.015 [1.009–1.034] | 0.501 [0.490–0.513] | -50.66% | 232.0 → 216.0 |
| `fold_flat` | 1 | 3.241 [3.163–3.386] | 2.919 [2.837–3.022] | -9.94% | 584.0 → 568.0 |
| `fold_flat` | 512 | 1013.378 [959.718–1036.597] | 1018.838 [969.897–1054.167] | +0.54% | 78,608.0 → 78,592.0 |
| `fold_nested` | 64 | 289.023 [283.374–315.462] | 290.321 [280.877–297.008] | +0.45% | 52,323.0 → 52,140.0 |
| `primitive_array` | 512 | 176.104 [167.210–184.031] | 168.000 [161.645–177.215] | -4.60% | 86,096.0 → 86,096.0 |
| `primitive_fold` | 0 | 0.874 [0.862–0.929] | 0.343 [0.331–0.400] | -60.69% | 80.0 → 64.0 |
| `primitive_fold` | 1 | 1.766 [1.201–2.337] | 0.714 [0.677–0.909] | -59.57% | 240.0 → 224.0 |
| `primitive_fold` | 8 | 2.824 [2.784–3.394] | 2.279 [2.228–2.393] | -19.31% | 968.0 → 952.0 |
| `primitive_fold` | 64 | 14.761 [14.548–15.349] | 13.992 [13.578–14.582] | -5.20% | 7,008.0 → 6,992.0 |
| `primitive_fold` | 512 | 112.050 [108.595–117.921] | 112.344 [104.657–115.991] | +0.26% | 55,416.0 → 55,400.0 |

The empty primitive-record fold improves from 0.874 to 0.343 µs (60.7%), and eight
rows from 2.824 to 2.279 µs (19.3%), with disjoint process ranges. The one-row fold
also improves in every process comparison range; its baseline varies widely
(1.201–2.337 µs), so the 59.6% median change should be read with that range.
The benefit is a fixed per-collection cost, most visible on short calls.

At 512 primitive records and 512 handles, time is essentially unchanged (+0.3%
and +0.5%, overlapping ranges). The unchanged ordinary-array control itself moves
-4.6%, and the scalar control -3.8%; no general large-list speedup is claimed.
The 64-row primitive median improves 5.2%, with slightly overlapping ranges.
Nested-handle timing remains inconclusive (+0.4%, overlapping ranges).

Primitive and flat-handle cases consistently save 16 JVM bytes per call: one
empty Object[] on this JVM. For empty primitive output, allocation is 80 → 64
bytes (20%); at 512 records it is 55,416 → 55,400 bytes. Other row allocations,
collector capacity and input locking are unchanged. Nested-case allocated-byte
medians also vary with JVM/JIT behavior; their larger delta is not attributed
solely to this fixed marker removal.

[results.json](results.json) and [samples.csv](samples.csv) include all process
scores, raw batches, allocation/CPU/GC metrics, commands and artifact identities.

## Identical-artifact controls

[aa-results.json](aa-results.json) and [aa-samples.csv](aa-samples.csv) compare the
same baseline library/JAR on both sides, with four fresh JVMs per side and the
same procedure. Allocated bytes are identical in every control case.

| Case | Rows | First µs [min–max] | Second µs [min–max] | Apparent time change |
|---|---:|---:|---:|---:|
| `control` | 1 | 0.013 [0.013–0.013] | 0.013 [0.013–0.015] | +6.68% |
| `fold_flat` | 512 | 1017.557 [962.739–1080.809] | 1086.777 [1004.525–1353.462] | +6.80% |
| `primitive_fold` | 0 | 0.877 [0.841–0.928] | 0.851 [0.837–0.865] | -2.99% |
| `primitive_fold` | 1 | 1.246 [1.208–1.251] | 1.220 [1.208–1.225] | -2.08% |
| `primitive_fold` | 512 | 109.787 [107.839–114.914] | 109.615 [105.716–155.164] | -0.16% |

These controls show variation, not optimization: empty/one-row medians move
-3.0%/-2.1%, the scalar control +6.7%, 512 handles +6.8%, and 512 primitive rows
-0.2%. All non-scalar process ranges overlap; the scalar shift is under 1 ns.
The slow 1.353 ms handle-control process and all other outliers are retained.
The substantial short-call improvements above exceed these control shifts;
small large-list changes are treated as inconclusive.

## Verification and reproduction

- `cargo test --offline --workspace` and workspace/all-targets Clippy with warnings
  denied pass. JniGen has 170 unit and six documentation tests.
- `SANITIZE=1 python3 examples/check.py` passes actual C ASan/UBSan and JVM CheckJNI
  callers. The test-only JNI table hook counts zero NewObjectArray calls on these
  callback-return paths, including present-empty collections; row/frame/reservation
  counts and exact ownership cleanup remain checked.
- All 281 example generated files match. Benchmark snapshots contain the changed
  plain callback Rust/Kotlin files; bulk snapshots are byte-identical.
- Twelve separate release-artifact JVM diagnostics with CheckJNI pass without JNI
  warnings; [diagnostic.json](diagnostic.json) records them. Their timings are
  excluded from the performance table.
- [Claude Opus 5.5 review](review.md) reports no remaining blockers. Packet fallback
  has generated-plan coverage; the direct optional-return path has actual raw JNI
  absent/empty checks. No new runtime fixture forces optional packet fallback.

Build clean revisions with `jni-output-results/prepare.py`, distinct output
paths and a shared target cache. Compile `jni-output-results/Driver.kt` once
against the baseline JAR and use the same driver JAR for both versions. Run
`jni-output-results/measure.py --forks 4` with CPU affinity 0,1 and the exact
`--cell` arguments in results.json. For A/A controls pass the baseline directory
as both `--before` and `--after`. Run `--check-jni --forks 1` separately. The runner
records every command, artifact hash, fixture identity and process score.
