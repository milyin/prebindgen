# Earlier native alias-history study after #887

The native-checked JNI results in this report describe the implementation before
JniGen removed that mode. Current JniGen always uses Kotlin handle preflight;
C native validation remains supported. Reproducing the old JNI comparison
requires the recorded revisions and their benchmark fixtures.

This study measured the original fixed-128 hybrid before the ascending append
fix. Its large ascending-group regression is addressed by the
[configurable-threshold update](thresholds.md), which preserves vector append and
compares six limits before retaining 128 as the default. The measurements below
remain as historical evidence of the problem and of the native paths affected.
Default JNI after #887 does not use this native history.

The baseline is #824 **after #887**, commit
`0d73b8e60eda49456d3c7d07407244435853793c`. Candidate runtime is #886's unchanged
hybrid implementation, merged onto that baseline; frozen JNI builds use
`0754d2bca`, and C builds with stronger rejection probes use `ab2395825`.
The added commits change fixtures/tests/provenance, not the runtime algorithm.

## C calls with checks enabled

Each row below contains 8,192 handles. Native C calls include input validation,
conversion, source work and destruction of Rust temporaries. Getters borrow
caller-owned opaque handles and construct primitive values. Callback results
instead transfer owned handles to Rust. Callback row storage remains with C
until decoding finishes. The shared case has a distinct mutable receiver of the
same handle type, so shared arrays still need native history. Read-only inputs
omit that receiver and use the shared-only implementation.

Order means the actual numeric pointer order, not allocation order. `groups8`
contains 1,024 inner lists of eight handles; singleton cases contain 8,192 inner
lists. Flat input and one-inner-list callback controls contribute one batch.

Microseconds/call before → after, followed by paired speedup:

| Case | Ascending | Descending | Shuffled |
|---|---:|---:|---:|
| Mutable getter, singleton lists | 1069.67 → 1157.43 (0.93×) | 9489.12 → 1174.90 (8.19×) | 5140.63 → 1197.93 (4.35×) |
| Mutable getter, groups of eight | 193.53 → 567.02 (0.34×) | 4651.17 → 578.07 (8.03×) | 4345.33 → 592.47 (7.32×) |
| Shared getter + mutable receiver | 1210.88 → 1198.33 (1.03×) | 9497.84 → 1154.07 (8.22×) | 5395.44 → 1213.67 (4.35×) |
| Owned callback, singleton lists | 867.25 → 819.70 (1.05×) | 9301.58 → 805.66 (11.40×) | 5079.49 → 875.89 (5.83×) |
| Flat first-batch control | 99.61 → 104.22 (0.96×) | 102.32 → 104.30 (0.98×) | 203.43 → 201.76 (1.01×) |
| Owned callback, one inner list | 200.16 → 202.45 (0.99×) | 204.08 → 205.85 (0.99×) | 381.93 → 386.66 (0.99×) |
| Shared-only control | 507.93 → 504.59 (1.01×) | 485.21 → 481.78 (1.01×) | 526.31 → 525.77 (1.00×) |

Shuffled singleton calls improve about 4.35× and owned callback results 5.83×.
Descending cases improve about 8–11×. These gains remove repeated movement of
accepted history when new addresses arrive before earlier entries.

The ascending groups-of-eight case is **2.92× slower** in the main run. A focused
repeat confirms 200.18 → 575.29 µs at 8,192 handles (2.87× slower) and
7.48 → 16.56 µs at 512 (2.21×). Its identical-library control differs by 0.42%
and 1.86%, respectively. The baseline already appends a disjoint ascending group
without scanning or shifting old entries. The hybrid promotes after 128 entries
before considering that append path, introducing hash allocation, insertion and
rehashing where the vector was cheap. This regression is not timing noise.

At 512 shuffled handles, singleton argument/callback cases improve about
20%/16%, and groups of eight about 42%. At 128 handles most shapes do not promote;
small changes remain visible in the CSV. A shared array followed by its separate
mutable receiver has 129 accesses and does cross the threshold.

C with checks disabled has identical allocation counts. Large-input timing
changes in those controls range approximately −3.2% to +3.0%; no benefit is
claimed. Shared-only calls also bypass `DynamicInputs`. A flat first batch keeps
its vector allocation but is not guaranteed a latency improvement: ascending flat
input measured about 4% slower.

## JNI with native checks explicitly enabled

Built using `--native-input-checks`, which sets
`set_kotlin_input_preflight(false)`. Kotlin retains locking but Rust performs
alias validation. Both libraries in each comparison use exactly the same Kotlin
JAR, and generated Rust glue also matches within the policy. Checked fixtures
contain nine dynamic native histories in the plain module and six in bulk;
default-JNI fixtures contain zero in both modules.

Bulk complete roundtrips at 8,192 handles:

| Case | Before µs | After µs | Paired speedup |
|---|---:|---:|---:|
| Ordinary singleton lists | 3647.46 | 1806.32 | 2.03× |
| Callback-returned singleton lists | 3643.94 | 1910.17 | 1.91× |
| One-inner-list control | 951.94 | 944.96 | 1.00× |
| Callback one-inner-list control | 906.57 | 902.92 | 1.00× |
| Flat control | 922.10 | 911.11 | 1.02× |

Default JNI instead measures about 1.01× for ordinary singleton-list arguments
and 1.00× for callback results at that size. It uses Kotlin preflight and does not
enter this history. No default-JNI speedup is claimed. Some small unchanged
controls vary more: the 128-row default callback control is about 11% slower in
the paired comparison. Its generated conversion code is unchanged; this study
does not establish an index-related cause. All controls are retained in the CSV.

The large JNI runs use `-Xss16m` on both sides for the existing recursive Kotlin
monitor helper. This PR neither changes that helper nor removes its stack limit.

## Native allocation cost

Separate instrumented libraries count allocations, reallocations and cumulative
requested native bytes. These are whole-call counts, not peak live memory or RSS.
JVM allocation is recorded separately by the timing driver. At 8,192 handles:

| Checked path | Allocations before → after | Reallocations before → after | Requested bytes before → after |
|---|---:|---:|---:|
| C singleton arguments | 16,385 → 16,392 | 12 → 6 | 655,296 → 950,064 |
| C singleton callback results | 16,385 → 16,392 | 12 → 6 | 655,296 → 950,064 |
| JNI singleton argument roundtrip | 24,579 → 24,586 | 14 → 8 | 1,310,736 → 1,605,504 |
| JNI singleton callback roundtrip | 24,582 → 24,589 | 16 → 10 | 1,310,818 → 1,605,586 |

Promotion adds seven allocations and 294,768 cumulative requested bytes in these
large cases, while removing six growth reallocations. First-batch, shared-only
and C checks-disabled controls retain their allocation counts. At 128 handles,
the singleton argument/callback counts are unchanged. The older validator-only
memory table in README.md remains a synthetic experiment, not whole-call memory.

## Method and reproduction

Linux x86-64, Rust 1.98 release, OpenJDK 21, Kotlin 2.4.21. All libraries/JARs are
frozen before timing. Timing libraries have no allocation instrumentation; counted
libraries verify instrumentation separately. Generated C Rust/ABI hashes and JNI
Kotlin/Rust glue match within each policy. Native libraries are verified by hash
and the C driver also checks the library actually loaded using `dladdr`.

C: four balanced process forks per cell, one CPU, three calibration and three
warmup batches, seven measured batches targeting 10 ms. The ordered regression
repeat uses nine batches targeting 15 ms. Callback owners are prepared outside
the timer; callback iteration counts are capped at 65,536 prepared handles per
batch. Borrowed fixtures stay live across calls. Counts use two process forks and
five observations each. Checked C pairs additionally reject deliberate aliases
in child processes, requiring the native alias diagnostic; malformed owners are
never sent to unchecked libraries.

JNI: four balanced fresh-JVM pairs, the same JAR for each library pair, fixed
256 MiB heaps/G1, two 256-call warmup passes over all cases, calibration, eight
warmup and nine measured batches targeting 20 ms. Counts use 1,000 warmup calls
and ten observations; observations agree exactly. Kotlin tests explicitly use a
native-checked binding and verify that a late alias fails after more than 128
native constructors, with consumed-prefix and unread callback cleanup checks.

Times are medians of process/JVM batch medians; paired speedups are medians of
corresponding per-fork ratios. Identical-library C/JNI controls use the same
procedure. On 8,192 singleton lists, C A/A changes are within 1% and JNI within
1.4%. Small differences should not be interpreted as universal gains.

[remaining-latency.csv](remaining-latency.csv) retains every timing cell, including
controls, slower paths, repeats and per-fork ranges.
[remaining-allocations.csv](remaining-allocations.csv) retains count summaries;
[remaining-provenance.json](remaining-provenance.json) records revisions, artifact
hashes, policies, fixture/glue hashes and settings. Raw batch logs stay outside Git.

Build C timing/counting pairs from clean checkouts using the same current fixture:

```sh
python3 examples/bench-c-collections/build.py --checkout "$BASE" --output /tmp/c-before
python3 examples/bench-c-collections/build.py --checkout "$CANDIDATE" --output /tmp/c-after
python3 examples/bench-input-aliases/c-paths.py --before /tmp/c-before --after /tmp/c-after \
  --output /tmp/c-comparison --kind time --order shuffled --forks 4 --batches 7 --target-ms 10
```

Repeat with `--order ascending`/`descending`; add `--disable-alias-checks` to both
builds for controls, or `--count-allocations` and `--kind count` for native counts.
For JNI, build both variants with `examples/bench-bulk/run.py --build-only
--native-input-checks`, then compare using `examples/bench-input-aliases/jni.py
--policy native`. Omit the build flag and use `--policy kotlin` for default-JNI
controls. Add `--native-allocations` and run `bench.allocations.AllocationBenchKt`
for JNI counts. Use `--aa` (JNI) or `--same-build before` (C) for A/A controls.
