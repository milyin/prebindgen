# Native bulk-output ownership measurements

Measured on 2026-11-03, Linux x86-64. This compares the native ownership change
against #824 after #856. It measures complete generated JNI calls, rather than
isolating native row conversion or claiming packet-processing throughput.

## Change being measured

Previously, Rust retained a commit closure for every converted element in a Vec
until output delivery succeeded. Each nonempty handle collection allocated that
Vec; nested collections allocated another Vec for each nonempty inner list.

Bulk frame writers now reclaim transferred handles by walking their completed
rows on conversion failure before final transfer. Completing a parent row retires its child registrations because
the parent can reach those children through their frame descriptors. Successful
final delivery consumes the writer and transfers ownership to the recipient.
Two pending registrations fit inline. Deeper/wider shapes can allocate a fallback
stack bounded by the schema's pending nesting/siblings, rather than row count.
Those deeper/wider shapes are not benchmarked here.

This removes native deferred commit queues for bulk outputs. Ordinary JNI object
rows and C collection output queues remain separate work under #824. Kotlin input
locking and conversion are included in round-trip timings.

## Native allocation counts

A benchmark-only Rust global allocator counts successful allocations,
reallocations and requested bytes on the calling thread. JVM allocations and
JNI allocations made inside the JVM are outside this counter. Counts start
immediately before a call and stop immediately after; source-handle creation and
final handle destruction are outside the counted interval. These are allocation
counts, not a timing profile or retained-memory measurement. Requested bytes sum
allocation/reallocation request sizes.

Each case has two fresh JVMs per revision, 1,000 uncounted warmup operations and
ten one-call count batches: 112 JVMs and 1,120 batches over 28 cells. Sizes are
0, 1, 8, 64, 512 and 4,096. Every repeated count within a cell is identical.
The separate six-JVM smoke run enables `-Xcheck:jni`.

Shapes use `InputHandle` and `HandleRecord { handle: InputHandle, tag: i64 }`:

- Flat: `Vec<InputHandle>` with N handles.
- Records: N records, one handle each.
- Nested: exactly three inner lists `[[], N handles, []]`.
- Many: N inner lists, each containing one handle; N is the total handle count.

| Output shape | N | Native allocations before → after | Requested native bytes before → after |
|---|---:|---:|---:|
| Flat handles | 0 | 2 → 2 | 32 → 32 |
| Flat handles | 1 | 7 → 6 | 128 → 120 |
| Flat handles | 512 | 518 → 517 | 36,912 → 32,816 |
| Handle records | 512 | 518 → 517 | 57,392 → 53,296 |
| One nonempty inner list | 0 | 4 → 3 | 288 → 216 |
| One nonempty inner list | 512 | 520 → 518 | 37,216 → 33,048 |
| Many one-handle inner lists | 1 | 9 → 7 | 256 → 224 |
| Many one-handle inner lists | 512 | 2,564 → 2,051 | 106,512 → 90,128 |

At N=512, flat and record outputs remove one allocation and 4,096 requested
bytes. The three-list nested shape removes two allocations and 4,168 bytes.
Many one-handle inner lists remove 513 allocations and 16,384 bytes. Native
reallocation counts are unchanged in every cell. Ordinary JNI flat/record
controls and both scalar controls have identical before/after counts.
These calls still allocate their native output handles and frame storage.
All 28 cells and every sample are in [counts/results.json](counts/results.json)
and [counts/samples.csv](counts/samples.csv).

## Uninstrumented latency

Timing uses separately built release libraries with the allocation hook disabled.
All 3,531 JVM class/module entries are byte-identical across the two revisions;
[class-identity.json](class-identity.json) records the content-map hash and
benchmark-class hashes. The same instrumented driver surrounds the unchanged
operation loop with CPU, compilation, GC and JVM allocation diagnostics.

Ten case/size/transport cells, four fresh JVMs per revision: 80 JVMs and 1,920
batches. Each case/size gets its own JVM. Before/after order alternates by fork;
case order uses a recorded deterministic shuffle. Timing covers input packing,
native conversion/source execution, output decoding and a result checksum.
Handle round trips reuse returned handles on the next iteration and check one
handle's value through a second JNI call. Final destruction is outside timing.
No latency measurement was added for the many-one-handle-lists count case.

Warmup makes 256 short sampling calls, then full-sized calls until at least three
seconds have elapsed and compilation time has been quiet for one second, with a
20-second cap. All 80 JVMs met that condition and had an active C2 sampling method
before measurement. Each fork records 24 batches targeting 50 ms. All batches,
GC pauses and slow forks are retained. Scores are total timed nanoseconds divided
by operations within each JVM, then the median of four JVM scores. Ranges are
fork minima/maxima, not confidence intervals. Positive changes mean slower.
Calling-thread CPU scores exclude other JVM threads.

Fixed 256 MiB G1 heap, two active processors, affinity CPUs 0–1, one JVM at a time.
Builds, tests and allocation probes finished before timing. JIT/GC logging remains
enabled in both revisions. CPU frequency, turbo and other host activity are not
controlled. `-Xcheck:jni` is used only in separate validation runs.

| Transport / case | N | Before ns/call [fork range] | After ns/call [fork range] | Wall change | CPU change |
|---|---:|---:|---:|---:|---:|
| bulk / Scalar control | 1 | 14.7 [13.2–17.6] | 13.4 [12.9–14.9] | -8.4% | -5.9% |
| bulk / Borrowed handle input | 512 | 12,340.8 [12,120.2–12,608.6] | 12,380.5 [12,272.8–12,497.5] | +0.3% | +0.3% |
| bulk / Nested handle round trip | 1 | 1,294.2 [1,198.5–1,316.4] | 1,215.7 [1,204.7–1,265.3] | -6.1% | -6.3% |
| bulk / Nested handle round trip | 512 | 75,192.4 [66,112.9–82,625.1] | 78,223.2 [74,342.8–90,902.2] | +4.0% | +4.0% |
| bulk / Handle record round trip | 512 | 75,168.8 [73,928.8–76,269.5] | 73,166.5 [72,262.5–75,201.7] | -2.7% | -1.7% |
| bulk / Flat handle round trip | 1 | 796.0 [783.0–821.3] | 775.6 [748.0–1,216.3] | -2.6% | -2.1% |
| bulk / Flat handle round trip | 64 | 7,675.5 [7,558.8–7,792.2] | 7,599.1 [7,402.3–7,792.7] | -1.0% | -1.1% |
| bulk / Flat handle round trip | 512 | 64,794.7 [63,898.9–66,407.7] | 63,154.1 [62,179.5–66,214.8] | -2.5% | -2.6% |
| bulk / Primitive record output | 512 | 15,433.5 [15,264.0–15,948.6] | 15,000.1 [14,685.4–15,308.5] | -2.8% | -3.0% |
| plain / Flat handle round trip | 512 | 2,093,574.5 [2,012,393.2–2,637,287.3] | 2,102,020.7 [2,042,906.8–2,124,245.1] | +0.4% | -0.1% |

Flat and record 512-element round trips measured 2.5% and 2.7% faster. The nested
512-element aggregate measured 4.0% slower; its paired changes were +37.5%, +8.3%,
−2.0% and −7.7%. This is not a consistent per-fork regression. The scalar control
also moved by −8.4%. Primitive record output has no handle commit-queue allocation
saving but measured −2.8%. These scores do not isolate the cause of small changes.

## A/A control

The same baseline JAR and native library are used under both labels, with the
same protocol: 16 fresh JVMs and 384 batches. All warmup conditions were met.

| Transport / case | N | Before ns/call [fork range] | After ns/call [fork range] | Wall change | CPU change |
|---|---:|---:|---:|---:|---:|
| bulk / Scalar control | 1 | 14.2 [13.1–15.4] | 13.4 [13.2–14.1] | -5.6% | -5.6% |
| bulk / Flat handle round trip | 512 | 65,178.7 [64,433.0–67,135.0] | 66,029.7 [62,928.9–68,061.4] | +1.3% | +1.4% |

The identical-build scalar aggregate moved by −5.6%; the 512-handle round trip
moved by +1.3%, with paired changes +0.5%, +0.2%, +4.8%, −3.8%. Small timing
differences therefore need caution. Lower native allocation is the supported
outcome; these measurements do not establish a general latency speedup.

JIT diagnostics show no measured-phase deoptimization belonging to the sampling
loop, benchmark operations or generated bindings. Two level-0 native JNI drop
stubs were generated in measurement windows, one baseline and one updated nested
512-element fork. There were 129 runtime-helper uncommon traps and 82 method
retirements in A/B, and 12/8 in A/A. These helper changes remain a confound.
The complete events, CPU/GC/compiler data, per-fork scores and JVM allocation
samples are retained in the latency and A/A JSON/CSV files; no outliers are removed.

## Provenance and reproduction

Measured implementation: `7b358a8fc`; baseline: `dea0fbcea` (#824 at `c4dba3d79`
plus identical allocation-probe fixtures). The one-slot ownership pilot is excluded.
Compiler, dependency lock, Cargo configuration, source and artifact hashes are
in each results file. The allocation build reuses the uninstrumented JAR after
verifying all 39 generated Rust/Kotlin source files match byte-for-byte; its
manifest preserves those hashes and the instrumented native artifact/profile.
Within each revision, the allocation build enables only the benchmark allocator
feature relative to the timing build. Allocation
instrumentation remains disabled in all latency builds.

The committed launcher now names `BenchKt` explicitly: adding a second benchmark
entry point means kotlinc need not put a Main-Class in the JAR manifest. This
launcher-only correction followed the frozen builds; it changes no generated
classes, native source or timed operation.

From a clean checkout, keep the current fixture identical for both implementations:

```sh
git worktree add --detach /tmp/native-before dea0fbcea
python3 examples/bench-bulk/run.py --checkout /tmp/native-before --build-only --output /tmp/native-before-build
python3 examples/bench-bulk/run.py --build-only --output /tmp/native-after-build
python3 examples/bench-bulk/run.py --checkout /tmp/native-before --build-only --native-allocations --output /tmp/native-before-counts
python3 examples/bench-bulk/run.py --build-only --native-allocations --output /tmp/native-after-counts
python3 examples/bench-bulk/native-output-results/count.py --before /tmp/native-before-counts --after /tmp/native-after-counts --output /tmp/native-counts
python3 examples/bench-bulk/output-cleanup-results/isolated/prepare.py --before /tmp/native-before-build --after /tmp/native-after-build --output /tmp/native-driver --kotlinc /path/to/kotlinc
taskset -c 0,1 python3 examples/bench-bulk/output-cleanup-results/isolated/compare.py --driver /tmp/native-driver --output /tmp/native-times --cells bulk:owned_handles_roundtrip:1,bulk:owned_handles_roundtrip:64,bulk:owned_handles_roundtrip:512,bulk:owned_handle_records_roundtrip:512,bulk:nested_owned_handles_roundtrip:1,bulk:nested_owned_handles_roundtrip:512,bulk:handles_input:512,bulk:control:1,bulk:records_output:512,plain:owned_handles_roundtrip:512
taskset -c 0,1 python3 examples/bench-bulk/output-cleanup-results/isolated/compare.py --driver /tmp/native-driver --output /tmp/native-aa --same-build before --cells bulk:control:1,bulk:owned_handles_roundtrip:512
python3 examples/bench-bulk/output-cleanup-results/isolated/diagnostics.py /tmp/native-times
python3 examples/bench-bulk/output-cleanup-results/isolated/diagnostics.py /tmp/native-aa
```

Use the same compiler/environment for both builds, build sequentially, and finish
all compilation/tests before timing. Set `KOTLINC` for builds if needed. Compiler
memory limits can be set through `JAVA_OPTS`; the timed JVM uses explicit flags.
The runners verify frozen artifact hashes before and after measurement. Full
stdout, compilation XML and GC logs remain under the local measurement directories
`/tmp/native-output-final-{latency,aa}/raw/`; their hashes and diagnostic summaries
are committed. Generated Rust review snapshots are also committed.

Both frozen implementations pass the complete benchmark validation matrix in
eight fresh JVMs with `-Xcheck:jni`. The real C/Kotlin/Java covertests cover partial
rows, nested outputs, independent frames, JNI allocation failure and callbacks.
Nine runtime bulk tests pass under Miri, including exact per-allocation destruction
counts, partial nested ownership, successful transfer and destructor panic during
unwinding. See [review.md](review.md) for the Claude/Copilot Opus 5.5 review summary.
