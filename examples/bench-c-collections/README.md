# Buffered C collection outputs

This fixture measures native allocation and complete C produce/read/free cycles
for converted collection outputs. It exercises opaque handles, two-handle
records, nested lists, primitive arrays and an unchanged scalar control.

The optimization replaces per-row ownership-transfer closures with a runtime
`OutputRows<T>` guard around the final row allocation. Each row is written before
its resources transfer and its completed count advances. On conversion failure,
the guard reclaims the completed prefix using generated typed cleanup. Nested
buffers follow the same ownership rule. Successful delivery relinquishes the
guard. C callers still destroy owned fields and row storage separately.

Final storage is allocated as an uninitialized boxed slice at its known length.
There is no intermediate row Vec or Vec-to-box finalization. The old path's
preallocated row Vec did not reallocate in the measured cases; the observed
allocation savings come from removed ownership queues.

See [results/README.md](results/README.md) for every result, protocol, limitations,
and provenance; [results/review.md](results/review.md) records the Opus review.
The measurements are synthetic and do not establish network-packet throughput.

## Reproduce

Start with committed, clean generator checkouts. Use the same fixture/compiler
for both revisions. The recorded baseline is `3b9acfffa`: #824 after #857 plus
the benchmark fixture, before the C optimization. The implementation measured
is `1c2e5ecf2`.

```sh
git worktree add --detach /tmp/c-rows-before 3b9acfffa
CARGO_BUILD_JOBS=2 python3 examples/bench-c-collections/build.py --checkout /tmp/c-rows-before --output /tmp/c-before --fetch
CARGO_BUILD_JOBS=2 python3 examples/bench-c-collections/build.py --output /tmp/c-after --fetch
CARGO_BUILD_JOBS=2 python3 examples/bench-c-collections/build.py --checkout /tmp/c-rows-before --output /tmp/c-before-counts --count-allocations --fetch
CARGO_BUILD_JOBS=2 python3 examples/bench-c-collections/build.py --output /tmp/c-after-counts --count-allocations --fetch
python3 examples/bench-c-collections/compare.py --before /tmp/c-before-counts --after /tmp/c-after-counts --output /tmp/c-counts --kind count --forks 2
taskset -c 0,1 python3 examples/bench-c-collections/compare.py --before /tmp/c-before --after /tmp/c-after --output /tmp/c-times --kind time
taskset -c 0,1 python3 examples/bench-c-collections/compare.py --before /tmp/c-before --after /tmp/c-after --output /tmp/c-aa --kind time --same-build before
```

Use empty output directories and finish all builds/tests/counting before timing.
Choose available CPUs for `taskset`. The current driver targets Linux and uses
GCC/Clang's GNU C11 extensions. `CC` selects the C compiler. Build parallelism is
limited in the example commands to avoid host memory pressure.

The counter is a feature of the standalone benchmark crate, enabled only by
`--count-allocations`. Timing rejects libraries with the counter enabled.
The same C executable is used for both libraries; `dladdr` verifies the selected
native library, and the runner verifies hashes before and after measurement.
Output includes raw stdout, all samples, per-process scores and build metadata.

Regenerate the committed Rust/header snapshots with a clean checkout:

```sh
python3 examples/bench-c-collections/build.py --output /tmp/c-record --record-generated --fetch
```

CI builds the counting fixture with `--check-generated`, then executes all shapes
and the scalar control using `compare.py --kind count --smoke --forks 1`.

Input comparisons use `input-compare.py`. In addition to handles and scalar rows,
the input driver covers a handle plus text, bytes, or both. Short payloads contain
three bytes; `_large` cases use a 4 KiB UTF-8 string (Euro signs plus one ASCII
byte) or a 4 KiB byte payload. Payloads, handles and C rows are prepared before
timing. Native decoding, required owned copies, alias checks, construction and
destruction remain timed. `--same-build` uses the selected `--cases` and `--sizes`
for an identical-library control. Per-run console logs stay in the selected
output directory; only compact summaries need to be committed for PR review.
