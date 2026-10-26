# JNI callback benchmark

Run from any directory with Cargo, a JDK and Kotlin 2.1 available:

```sh
KOTLINC=/path/to/kotlinc python3 examples/bench-callbacks/run.py --output /tmp/callback-bench
```

`--checkout /path/to/prebindgen` selects a different source tree, including the
production `main` API. The script creates an isolated fixture project, builds
release Rust with offline Cargo, compiles the same Kotlin caller, and records raw
logs, environment details and medians in the output directory. Use separate output
directories for comparisons; a default run creates a new temporary directory.

Each of three fresh JVMs warms up ten batches per case and measures nine batches
of 200,000 callbacks. Rust times only repeated calls to an already-registered
callback. Cases deliver one scalar, a borrowed two-field record, and an owning
handle automatically closed by the Kotlin bridge. Conversion, the Kotlin callback
and automatic cleanup are timed; registration and the outer JNI wrapper are not.
These are callback-boundary costs, not network throughput measurements. The
reported `dirty` flag identifies measurements of uncommitted work.

## Reference measurement

Kotlin 2.1.0, OpenJDK 11.0.29, rustc 1.98.0, Linux, release Rust, fixed
256 MiB JVM heap; three JVMs per variant, with the defaults above. Variants were manually run
in rotating order after all builds completed; `run.py` measures one checkout at
a time. The main native artifact was built previously from the verified commit
below; the Kotlin driver was freshly compiled for all variants.

| Callback payload | `main` | #823 before | Updated #826 |
| --- | ---: | ---: | ---: |
| One `i64` | 183 ns | 270 ns | 200 ns |
| Borrowed two-field record | 184 ns | 272 ns | 212 ns |
| Owned handle, automatically closed | 220 ns | 508 ns | 231 ns |

The baseline trees are `main` at `5e149f869` and PR #823 at `94f9eab1b`
(the latter has the same sources as merged #825 at `02314df44`). The updated
column measures the runtime implementation committed in `4c3fcd44b`, tested
before committing the benchmark report.
The remaining gap to `main` is not attributed to an isolated component.
