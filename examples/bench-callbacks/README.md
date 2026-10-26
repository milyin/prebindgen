# JNI callback benchmark

Run from any directory with Cargo, a JDK and Kotlin 2.1 available:

```sh
KOTLINC=/path/to/kotlinc python3 examples/bench-callbacks/run.py --output /tmp/callback-bench
```

`--checkout /path/to/prebindgen` selects a different source tree. The script
creates an isolated fixture project, builds release Rust with offline Cargo,
compiles the same Kotlin caller, and records raw
logs, environment details and medians in the output directory. Use separate output
directories for comparisons; a default run creates a new temporary directory.

Each of three fresh JVMs warms up ten batches per case and measures nine batches
of 200,000 callbacks. Rust times only repeated calls to an already-registered
callback. Cases deliver one scalar, a borrowed two-field record, and an owning
handle automatically closed by the Kotlin bridge. Conversion, the Kotlin callback
and automatic cleanup are timed; registration and the outer JNI wrapper are not.
These are callback-boundary costs, not network throughput measurements. The
reported `dirty` flag identifies measurements of uncommitted work.

## Comparing two prepared builds

Prepare each checkout in a separate output directory with `run.py`, then compare
those artifacts after all builds have completed:

```sh
python3 examples/bench-callbacks/compare.py \
  --variant before=/tmp/callback-before \
  --variant after=/tmp/callback-after \
  --output /tmp/callback-comparison
```

The comparison runs two complete rotations of the variant order: with two
variants, each runs in four fresh JVMs and goes first twice. An explicit `--jvms`
count must be a multiple of the variant count. Each JVM contributes its median;
the report gives the median and range of those JVM medians, plus every raw batch.
It checks callback result totals, saves failure logs, records artifact hashes,
and rejects files modified during measurement. The output directory must be new.

Prepare both artifacts with the current `run.py`. The comparison requires matching
Rust/Kotlin compilers, release profiles and recorded profile environment settings.
The `dirty` flag remains recorded so experiments on uncommitted code are visible.

## Reference measurement

Kotlin 2.1.0, OpenJDK 11.0.29, rustc 1.98.0, Linux, release Rust, fixed
256 MiB JVM heap; three JVMs per variant, with the defaults above. Variants were manually run
in rotating order after all builds completed; `run.py` measures one checkout at
a time. The native artifact for `5e149f869` was built previously; the Kotlin driver
was freshly compiled for all variants.

| Callback payload | `5e149f869` | `94f9eab1b` | `4c3fcd44b` |
| --- | ---: | ---: | ---: |
| One `i64` | 183 ns | 270 ns | 200 ns |
| Borrowed two-field record | 184 ns | 272 ns | 212 ns |
| Owned handle, automatically closed | 220 ns | 508 ns | 231 ns |

The columns identify the measured source commits. `94f9eab1b` has the same
source tree as `02314df44`. The `4c3fcd44b` implementation was measured before
committing the benchmark report. Differences between these measurements are not
attributed to an isolated component.

## Callback runtime inlining measurement

The source change adds inlining hints to callback invocation and local-frame
operations. The compiler can optimize these operations at their call sites;
callback ownership, exception handling and local-reference cleanup are unchanged.

Both prepared builds used rustc 1.98.0, Kotlin 2.1.0, OpenJDK 11.0.29, the release
profile and no profile environment overrides. The baseline is `51f9bfa29`;
`110537870` includes the inlining change. Both source trees were clean at build
time. No builds ran during the comparisons.

```sh
python3 examples/bench-callbacks/compare.py \
  --variant before=/tmp/callback-before \
  --variant after=/tmp/callback-after \
  --output /tmp/callback-inlining-comparison --jvms 8
```

The confirmation used eight fresh JVMs per variant, with ten warmup and nine
measured batches of 200,000 callbacks per case. Each JVM contributes its median;
the table reports the median of those eight JVM medians:

| Callback payload | `51f9bfa29` ns | `110537870` ns | Time reduction |
| --- | ---: | ---: | ---: |
| One `i64` | 204.4 | 191.3 | 6.4% |
| Borrowed two-field record | 213.4 | 187.2 | 12.3% |
| Owned handle, automatically closed | 234.9 | 220.4 | 6.1% |

[inlining-results.json](inlining-results.json) preserves the initial four-JVM run
and the eight-JVM confirmation, including every sample, JVM medians/ranges, build
metadata and artifact hashes. An unusually slow JVM occurred in the initial run;
the confirmation checks whether its result persists with more independent JVMs.
Scalar and record reductions were approximately 6–7% and 12% in both comparisons.
Handle estimates varied from 2% to 6%, with overlapping JVM ranges. These noisy
microbenchmarks do not establish a network throughput gain.
