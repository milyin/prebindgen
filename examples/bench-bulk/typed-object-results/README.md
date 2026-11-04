# Typed rows for ordinary JNI collection returns

Ordinary non-bulk collection returns previously allocated a JVM `Object[]` per
row, boxed primitive fields and populated slots individually through JNI. They
now reuse the immutable raw holder classes used by collection callback capture.
Rust borrows cached constructor metadata once per collection and constructs each
row with one JNI constructor invocation. Kotlin reconstructs public values after
the native call and handle locks finish. Raw primitive fields are unboxed.
Primitive-only rows use one automatically deleted local reference; rows containing
JVM references retain their existing local-reference frames.

Constructor and field IDs are registered once from actual Kotlin `Class` objects
after native-library loading, allowing Rust-created workers to construct the same
rows. Registration pins the classes in the native cache. The generated Android
keep rules must be included by shrinker-enabled consumers; R8 was not run here.

Ownership commits only after construction and insertion succeed. A completed
row prefix owns already transferred native handles; rollback reads the exact
registered class fields and reclaims nested typed, legacy and bulk rows. The
rejected row remains Rust-owned. No new per-call locking, alias checks or handle
cloning was added. Each row still has one transport object and the outer array;
this change does not turn the non-bulk path into a single bulk buffer.

## Compatibility

- Public Kotlin collection signatures and deferred user projections are unchanged.
- Bulk outputs and existing explicit callback delivery retain their protocols.
- The typed representation also applies to nested ordinary collections and
  explicitly non-bulk Rust-to-Kotlin callback arguments.
- More than 254 constructor parameter slots (including two per Long/Double)
  keeps legacy boxed array rows. Exactly 254 slots and mixed typed/legacy nesting
  are exercised through compiled JNI/Kotlin in the covertest.
- Existing hand-built `JniSequence::Buffered` output layouts keep their decoder.
  New hand-built typed schemas are checked for the constructor limit; custom
  typed writers must honor the exact registered row-class contract.

## Method

Before uses the immutable release artifacts built at
`7eacf4d1985df260a64bc3332c6f4473a4f3a255`. All nine generator/runtime dependency
trees are byte-identical to merged #824 at
`199e2da5d08eada169848bfe49a20539a21fd0ef`; the checked tree IDs are in
[source-identity.json](source-identity.json). After uses implementation commit
`9c0491cea` (later commits contain generated benchmark artifacts and reports).

Both sides use the same unchanged fixture and driver JAR, dependency lock,
release compiler/profile and environment, verified by the runner. Rust 1.98.0,
Kotlin 2.4.21, OpenJDK 11.0.29, Linux x86-64, two CPUs (`taskset -c 0,1`), G1
and a fixed 256 MiB JVM heap. Four fresh processes per case/version alternate
before/after order. Each warms for at least two seconds and waits for a 500 ms
quiet interval in JVM compilation before twelve measured batches. No builds or
tests run during timing. Process scores are total measured time divided by total
calls; tables show their median and full min–max range. Lower time is better.

`primitive_array` returns eight-i64 records through the ordinary non-bulk path.
`flat` consumes a list of handles and returns them as a List; `nested` does the
same for one-handle inner lists. Their timings include unchanged input conversion
and handle locking. `fold_nested` also benefits because the outer callback returns
ordinary inner lists. `primitive_fold`, `fold_flat` and scalar `control` retain
their existing hot paths and serve as controls. A size is a row/list length, not
a field count. Allocation covers the whole generated call, including destination
objects. Each ratio compares the same operation before/after.

## Complete-call results

| Case | Rows | Before µs [min–max] | After µs [min–max] | Time change | JVM bytes/call before → after |
|---|---:|---:|---:|---:|---:|
| `control` | 1 | 0.013 [0.013–0.016] | 0.014 [0.013–0.015] | +2.99% | 0.0 → 0.0 |
| `flat` | 1 | 5.436 [4.582–6.813] | 2.984 [2.880–3.715] | -45.11% | 592.0 → 568.0 |
| `flat` | 512 | 2094.013 [1958.426–2318.098] | 1055.072 [1016.067–1294.877] | -49.61% | 97,592.0 → 85,304.0 |
| `fold_flat` | 512 | 1055.704 [1013.932–2235.277] | 979.585 [970.610–1009.287] | -7.21% | 83,280.0 → 83,280.0 |
| `fold_nested` | 64 | 399.126 [389.371–555.646] | 285.197 [272.897–354.214] | -28.54% | 54,511.1 → 52,950.4 |
| `nested` | 1 | 7.129 [6.913–8.213] | 5.067 [4.869–5.368] | -28.92% | 976.0 → 944.0 |
| `nested` | 64 | 417.197 [399.186–531.387] | 280.188 [273.910–295.670] | -32.84% | 55,190.2 → 53,056.3 |
| `primitive_array` | 0 | 0.493 [0.448–0.531] | 0.449 [0.440–0.461] | -8.86% | 40.0 → 40.0 |
| `primitive_array` | 1 | 12.266 [11.702–14.270] | 0.984 [0.912–1.015] | -91.98% | 392.0 → 232.0 |
| `primitive_array` | 64 | 749.940 [718.181–872.612] | 23.884 [21.664–26.433] | -96.82% | 21,048.0 → 10,808.0 |
| `primitive_array` | 512 | 6070.637 [5760.017–7286.163] | 170.147 [165.568–182.075] | -97.20% | 168,016.0 → 86,096.0 |
| `primitive_fold` | 512 | 109.589 [106.986–113.139] | 115.162 [106.700–126.542] | +5.08% | 60,088.0 → 60,088.0 |

The 512-record ordinary return falls from 6070.6 to 170.1 µs (35.7× lower
whole-call latency; −97.2%) and 168,016 to 86,096 JVM bytes (−48.8%). The
allocation saving is 160 bytes per eight-field record: eight Long boxes plus
a generic row array are replaced by one typed holder. Single and 64-record
lists also improve substantially; all nonempty primitive before/after ranges
are separated. The empty-list case keeps the same 40-byte allocation and has
overlapping timing ranges.

The 512-handle list halves complete-call time (2094.0 → 1055.1 µs) and saves
12,288 JVM bytes; its unchanged input work limits the relative saving. Nested
64-row list time falls 32.8%, with JVM allocation reduced by about 2 KB. JVM
allocation for unchanged primitive/flat callback controls is identical; their
time medians move +5.1% and −7.2%, respectively. These control shifts must not be
attributed to changes in those hot paths. The baseline flat-callback process
range includes one slow process (2235 µs); all processes remain in the report.

These are warmed microbenchmarks, not network throughput measurements. No timing
run was discarded. [results.json](results.json) and [samples.csv](samples.csv)
preserve all process scores, raw batches, CPU times, allocation, GC counts,
commands and artifact hashes. [diagnostic.json](diagnostic.json) records twelve
separate before/after CheckJNI processes with no JNI warnings.

## A/A controls

The same baseline JAR/library was run as both versions, with the same four-process
protocol and no rebuilds. Allocation is identical in every A/A cell. These are
time-variation checks, not an alternative performance comparison.

| Case | Rows | A µs [min–max] | A µs [min–max] | Apparent change |
|---|---:|---:|---:|---:|
| `control` | 1 | 0.013 [0.013–0.014] | 0.013 [0.013–0.013] | -1.96% |
| `fold_flat` | 512 | 1067.102 [1001.104–1261.666] | 1000.784 [988.413–1018.883] | -6.21% |
| `primitive_array` | 512 | 5783.576 [5663.771–6027.898] | 6152.762 [5963.327–6587.940] | +6.38% |
| `primitive_fold` | 512 | 109.714 [105.028–110.992] | 118.383 [110.899–136.206] | +7.90% |

Raw [aa-results.json](aa-results.json) and [aa-samples.csv](aa-samples.csv)
include every process and batch. Ordinary primitive-record gains are much larger
than this variation. The unchanged callback/scalar timing shifts remain inconclusive;
no callback-control speedup or slowdown is claimed.

## Review and correctness

[review.md](review.md) records the Claude Opus 5.5 review findings and fixes.
The full C/Kotlin/JNI boundary suite passes with C ASan/UBSan and JVM CheckJNI,
including constructor/insertion failure, partial-prefix cleanup, mixed wide-row
fallback, native workers, destructor panic and pending-exception preservation.
Workspace tests, Clippy with warnings denied, formatting and all 281 generated
example-file snapshots pass locally. Committed benchmark snapshots show that
only the plain non-bulk generated Rust and Kotlin changed; bulk sources are
byte-identical.

## Reproduction

Use clean checkouts at the two measured source commits, build each with
`jni-output-results/prepare.py` (use `--checkout`, distinct `--output` directories
and a shared `--target` cache), and compile the unchanged
`jni-output-results/Driver.kt` against the before JAR. Run
`jni-output-results/measure.py` with the two build directories and the shared
driver, `--forks 4`, CPU affinity 0,1 and the exact `--cell` settings recorded in
results metadata. Run `--check-jni --forks 1` separately from timing. Native
libraries are copied into separate artifact directories before build-cache reuse.
