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

The comparison starts three fresh JVMs per variant, rotating variant order in
each round. It checks callback result totals, saves every timing sample, and
records the build metadata and artifact hashes. The output directory must be new.
Use the same Kotlin compiler, Rust toolchain and build profile for both builds.

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
