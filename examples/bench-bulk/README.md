# Bulk JNI transport comparison

This fixture binds the same Rust functions twice in one release native library:
`bench.plain` uses ordinary JNI conversion and `bench.bulk` adds `.bulk()` to the
relevant expansion. Both modes disable automatic bulk-return selection with
`set_bulk_collection_outputs(false)` so future defaults cannot change the ordinary
baseline. The comparison still uses explicit bulk overrides.

```sh
KOTLINC=/path/to/kotlinc taskset -c 0,1 python3 examples/bench-bulk/run.py \
  --output /tmp/bulk-bench --check-generated --warmup 10
```

The command above uses Linux `taskset` to match the recorded CPU affinity; choose
a permitted CPU pair if CPUs 0–1 are unavailable, or omit `taskset` on other systems.
Use an empty output directory. Add `--fetch` if the pinned dependencies are not
cached; downloading finishes before the native build and measurement. Cargo builds the isolated fixture offline, then
Kotlin compiles both generated surfaces into one JAR. All builds finish before
measurement. Four fresh JVMs per mode run in alternating plain/bulk order, with
case order rotated/reversed across rounds. An untimed pass over all cases trains
dispatch profiles before any per-case calibration. Each case then calibrates its
batch size, warms up ten batches with the command above (five by default), and measures nine batches targeting 30 ms each. Each JVM
contributes its median; the report summarizes the median and range of those JVM
medians and preserves every sample. There are no performance pass/fail thresholds.

The driver uses an unboxed primitive SAM interface (`run(Int): Long`), consumes every
record field, and checks closed-form batch checksums. New single-record return cases
use field weights to detect swapped fields as well as incorrect sums. Callback cases additionally
check invocation counts. Inputs and callback registration are prepared before
timing; layout lookup/class loading occur during untimed calibration. Native
conversion, source function work, Kotlin reconstruction/consumption and normal
wrapper/handle bookkeeping remain timed. Native builds require clean, committed repositories for all path dependencies
discovered by Cargo, including this fixture, and use the committed dependency lock. Rust is built in release mode; timing
uses ordinary JNI checks, without `-Xcheck:jni`. Absolute timings include SAM dispatch, which can be megamorphic. Actual batch durations are recorded; calibration is capped at five million calls.

The matrix includes batches of 0, 1, 8, 64 and 512 elements:

| Case | Timed work | Reason to compare |
| --- | --- | --- |
| `control` | Identical ordinary `i64` argument/result in both packages | Indicates process-to-process timing variation. |
| `record_return` | One eight-field record returned from Rust, then a Kotlin checksum | Expanded output uses a primitive-parameter capture callback; bulk returns a frame directly. |
| `wide_record_input` / `wide_record_return` | One record with 32 named `i64` fields | Tests wider primitive argument and return expansion without source collections. |
| `callback_record_return` / `callback_wide_record_return` | A pre-registered Kotlin callback returns one eight-/32-field record; Rust checksums it | Compares boxed-field callback results against packing the result fields. |
| `scalar` | One `i64` argument/result | Ordinary primitive parameters should beat allocating/copying frames. |
| `record_input` | One eight-`i64` record to a Rust sum | Flattened primitives already avoid object access; bulk can add unnecessary work. |
| `packet_input` | Eight-field header plus 64 B, 4 KiB or 64 KiB byte payload, to a Rust checksum | Bulk adds Kotlin-frame and native decoding copies; ordinary transport can be better for payload-heavy packets. |
| `numbers_input` | `Vec<i64>` to a Rust sum | Bulk can avoid object-array/boxed-number access for each input element. |
| `records_input` | Shared record slice to a Rust sum | Bulk can combine many expanded fields in one byte frame. |
| `handles_input` | Borrow existing handles and read their values in Rust | Exercises collection input preparation and handle locks; handles are created outside timing and reused. |
| `handle_records_input` | Records containing a borrowed handle and a scalar tag | Exercises record preparation and locks. An explicit Rust constructor copies only the scalar payload from each borrowed handle into the owned record; that construction remains timed in both modes. |
| `owned_handles_roundtrip` | Consume a handle list and return its owners, then read the first value | Exercises ownership acknowledgement and both input/output conversion. Each call reuses the owners returned by the preceding call; one extra scalar JNI call checks the result. Initial handle creation and final cleanup are outside timing. |
| `records_output` | Rust constructs records; Kotlin materializes a list and sums fields | Bulk can avoid native-side per-row object arrays and primitive boxing. |
| `records_roundtrip` | Kotlin list to Rust `Vec<Row>` and back, then checksum | Measures conversion in both directions. |
| `callback_records` | Fire a pre-registered Rust callback with a record batch; Kotlin sums it | Both registration and handler destruction are outside the timed path. |
| `records_reduce` | Same Rust result reduced to a checksum | Plain uses a dedicated per-row `callback_return_only()` fold; bulk materializes its list then reduces it. Both update a reusable primitive accumulator; `Unit` prevents generic accumulator boxing. A separate Rust entry point forwards to the same row-construction function; the ordinary list-return binding is unchanged. This deliberately compares different delivery APIs for an aggregate-only use case. |

`records_output` measures materialized-list delivery; `records_reduce` also checks
whether streaming is a better alternative when a list is unnecessary. Empty
batches expose fixed costs rather than per-element savings. Packet inputs use the separate `--payloads` byte-size list. Record values exceed
the JVM boxed-number cache range. Plain packet input copies the byte array once
into the source `Vec<u8>`; bulk input also packs it into a Kotlin frame and
copies/decodes that frame. Frame growth is part of the measured generated API.

Latency is elapsed nanoseconds per call, not packet throughput. JVM allocated
bytes per call come from `ThreadMXBean` on the calling thread, including JNI-created
objects and synchronous callbacks. They exclude Rust/native allocations, JIT/GC
worker allocations and any retained input objects. Native allocations and copies
still contribute to elapsed time. The allocation counter is read outside each
timed loop. A fixed 256 MiB G1 heap and two active JVM processors include normal GC effects.

A short correctness run uses the real generated boundary with JNI diagnostics;
its timings are excluded from the summary and must not be used for performance claims:

```sh
KOTLINC=/path/to/kotlinc python3 examples/bench-bulk/run.py \
  --output /tmp/bulk-check --jvms 2 --warmup 1 --batches 1 \
  --target-ms 2 --check-jni
```

Reuse prepared artifacts without rebuilding:

```sh
python3 examples/bench-bulk/run.py --prepared /tmp/bulk-bench \
  --output /tmp/bulk-confirmation
```

The report records source/tool/profile metadata, compiler versions from the fixture build directory, the dependency-lock hash, Cargo config hashes, the current runner hash, generated fixture source hashes,
JAR/native-library hashes, raw batch samples in `samples.csv` and per-JVM medians. Artifact hashes
are checked before and after measurement. `generated/` records the Rust/Kotlin
outputs of `--record-generated` for review.

A build-only generated-file check (no performance measurement) is available:

```sh
KOTLINC=/path/to/kotlinc python3 examples/bench-bulk/run.py \
  --output /tmp/bulk-artifact-check --build-only --check-generated
```

## Original 35-case comparison

Measured on 2026-10-30 (Europe/Paris), Linux x86-64, AMD EPYC 7713, with CPU
affinity restricted to CPUs 0–1, Temurin OpenJDK 11.0.29, Kotlin 2.4.21 and Rust
1.98.0. The measured source is commit `24d19c9185bc58a6c4eee0e882090b2641dc4731`;
later commits add results/documentation and fix snapshot recording, without
changing the timed Kotlin/native fixture or generated wrappers. The cdylib was verified at optimization level 3 with debug
assertions disabled. Four JVMs per mode each used ten warmup and nine measured
batches. No extra JVM option environment variables were set.

The [full report](results.json) contains all 35 cases and every per-JVM median.
[Raw samples](samples.csv) contain all 2,520 measured batches; the report records
their SHA-256. Build and measurement inherited that affinity from the execution environment.
For the exact original fixture, check out its recorded source commit and use the
first command above with `--warmup 10`. The single-record extension added six
cases (41 total at that revision); the focused run below measures those cases and
two controls. The current fixture also has 15 handle-collection cases (56 total).
`ActiveProcessorCount=2` alone does not restrict CPU placement.
The recorded reports describe their pinned source revisions. Current generation
also preallocates final collection storage and directly constructs every input
element, including custom constructors and handles; the recorded timings do
not measure that optimization.
These are single-host measurements, not a
platform-independent speedup guarantee or end-to-end network throughput.

Times below are median **[minimum–maximum JVM median]**. Ratios divide ordinary
time by bulk time: **above 1 favors bulk**. Allocation columns show calling-thread
JVM bytes per call as **ordinary / bulk**, excluding native allocations.

### Where bulk helps

For these nonempty collection paths, ordinary JNI uses object arrays and boxed
values. The generated ordinary Rust output, for example, calls `Long.valueOf`
and sets an object-array slot for each field of every row. Bulk moves the
expanded fields in one frame and builds the destination objects in Kotlin.
Those calls use the checked JNI API with class/method lookup, rather than cached
method IDs. The ordinary converter costs roughly 9–11 µs per row in this run;
boxing alone should not be read as the explanation for that cost. The large ratios
below compare bulk against this specific list converter. Caching JNI lookups or
adding other optimized transports could change the comparison.

| Case / elements | Ordinary µs [range] | Bulk µs [range] | Ordinary / bulk | JVM B/call |
| --- | ---: | ---: | ---: | ---: |
| `numbers_input` / 64 | 109.27 [106.69–112.36] | 0.83 [0.75–0.89] | 130.9× | 7328 / 1000 |
| `records_input` / 64 | 594.23 [587.33–618.65] | 2.78 [2.46–2.93] | 213.6× | 19104 / 4584 |
| `records_output` / 64 | 688.10 [682.27–708.41] | 2.42 [2.35–2.50] | 284.6× | 22640 / 9592 |
| `records_roundtrip` / 64 | 1295.26 [1281.22–1379.50] | 5.73 [5.36–6.28] | 226.2× | 41744 / 14176 |
| `callback_records` / 64 | 691.23 [685.06–693.28] | 2.79 [2.67–2.81] | 248.1× | 22640 / 9592 |
| `numbers_input` / 512 | 865.84 [850.29–906.40] | 3.97 [3.38–4.24] | 218.2× | 57504 / 6376 |
| `records_input` / 512 | 4801.81 [4662.47–4944.51] | 19.77 [16.31–21.42] | 242.8× | 151712 / 35048 |
| `records_output` / 512 | 5502.55 [5434.71–5918.58] | 17.49 [17.21–17.89] | 314.6× | 179424 / 75896 |
| `records_roundtrip` / 512 | 10383.65 [10269.85–10486.66] | 39.48 [37.20–40.68] | 263.0× | 331136 / 110944 |
| `callback_records` / 512 | 5558.35 [5499.33–5724.96] | 18.20 [16.83–18.66] | 305.4× | 179424 / 75896 |

Bulk already wins for one-element record collections in this fixture: input is
9.67 µs ordinary versus 0.275 µs bulk; output is 11.73 µs versus 0.292 µs.
That is a different boundary shape from the single flattened `Row` below.

### Where ordinary parameters are better

Flattened scalar/record arguments already pass native primitives without
per-field object access. Packing those into a frame adds work. Packet arguments
also have an efficient byte-array path: bulk adds frame allocation and copying
without removing per-payload-element JNI calls, since the ordinary path has none.

| Case | Ordinary ns [range] | Bulk ns [range] | Ordinary / bulk | JVM B/call |
| --- | ---: | ---: | ---: | ---: |
| `scalar/1` | 13.8 [13.7–13.9] | 275.7 [265.9–565.8] | 0.05× | 0 / 152 |
| `record_input/1` | 30.3 [29.5–31.1] | 150.9 [147.9–178.8] | 0.20× | 0 / 128 |
| `numbers_input/0` | 106.0 [97.0–109.5] | 161.4 [157.5–178.0] | 0.66× | 40 / 80 |
| `records_input/0` | 114.1 [110.1–117.1] | 177.8 [169.4–194.7] | 0.64× | 40 / 80 |
| `packet_input/64` | 142.6 [141.8–145.0] | 331.9 [327.2–341.0] | 0.43× | 0 / 368 |
| `packet_input/4096` | 963.3 [949.2–992.9] | 1820.0 [1697.0–3087.2] | 0.53× | 0 / 4384 |
| `packet_input/65536` | 13385.8 [13269.3–13507.4] | 22668.0 [21945.8–24573.6] | 0.59× | 0 / 65824 |

The scalar bulk path is about 20× slower and the single-record path about 5×
slower. Bulk packet input is 1.7–2.3× slower across the measured payload sizes.
Keep those ordinary paths unless a surrounding collection provides a reason to
pack them together. Empty input collections also expose the frame's fixed cost;
empty output/callback cases have different costs and favor bulk in this run.

### Aggregate-only results

This comparison changes the delivery contract: `callback_return_only()` streams
rows to a fold; bulk returns a materialized list. The ordinary materialized-list
result (from `records_output`) is included to show how much the fold already
avoids. That case calls `checksumPlain`; the bulk fold case uses an inline loop.

| Elements | Ordinary list µs | Ordinary fold µs | Bulk list µs | Fold / bulk list | JVM B/call: list / fold / bulk |
| --- | ---: | ---: | ---: | ---: | ---: |
| 64 | 688.10 | 20.42 | 2.40 | 8.52× | 22640 / 16360 / 9592 |
| 512 | 5502.55 | 155.85 | 17.27 | 9.02× | 179424 / 129696 / 75896 |

Bulk still wins for this small-row checksum, even after materialization. The fold
is much faster than the ordinary list path and can avoid retaining a whole list;
this test does not establish which API is best for large rows, a different
consumer, or memory-pressure constraints.

Some bulk JVM medians varied substantially: scalar spans 266–566 ns and 4 KiB
packet input spans 1.70–3.09 µs. Their ranges remain slower than ordinary, but the
exact slowdown should not be treated as a constant. Large ordinary collection
calls hit the minimum batch size of 16 operations: the 512-row round trip had a
median batch duration of 166 ms rather than the 30 ms target. The report preserves
actual durations and ranges; no sample was discarded for being slow. Some bulk
JVMs also overshot the target after warmup (scalar 49.7 ms; single-record input
38.1 ms), so warmup did not eliminate every timing phase change.

## Single records without source collections

The [focused report](single-record-results/results.json) and
[576 raw batches](single-record-results/samples.csv) were measured separately from
the original run, using source commit
`82416465ed8784b218e7f20498ef2d1a014b352c`. The build generated and compiled wrappers from that committed source;
the resulting snapshots were committed separately.
The source library, generator, Kotlin and dependency lock were clean and committed
at build time, and the native library reports optimization level 3. Hardware/JVM
settings match the original run. Eight cases ran in four fresh JVMs per mode,
with ten warmup and nine measured batches, targeting 30 ms, pinned to CPUs 0–1.

The measured value payloads are scalars or single records; there are no source
arrays, lists or byte payloads in these cases. Callback registration and
handle destruction remain outside timing. Callback record creation, transport,
Rust weighted checksums and normal wrapper/handle work remain timed. For direct
returns, Kotlin consumes a field-weighted checksum. Inputs reuse prepared records;
return cases vary the seed over 1,024 values. The first/second JVM trials use
forward/reverse case order; the third/fourth also rotate by one position. Each
plain/bulk pair uses identical case order, and mode order alternates per trial.
A separate four-JVM `-Xcheck:jni` run
passed the same cases before timing.

Times are the median of four per-JVM medians, with
**[minimum–maximum JVM median]**, in ns. The ratio divides the
ordinary median by the bulk median; its brackets show the minimum/maximum of the
four paired per-trial ratios, rather than a confidence interval. JVM B/call
excludes native allocations, as in the original report.

| Case | Ordinary ns [range] | Bulk ns [range] | Ratio [paired range] | JVM B/call: ordinary / bulk |
| --- | ---: | ---: | ---: | ---: |
| `record_return` | 253.8 [220.5–285.2] | 181.4 [170.4–187.4] | 1.40× [1.18–1.67] | 80 / 208 |
| `wide_record_return` | 486.7 [468.4–503.3] | 360.3 [342.4–384.8] | 1.35× [1.30–1.39] | 272 / 592 |
| `callback_record_return` | 9335.2 [8937.0–9704.3] | 2060.4 [1996.2–2196.1] | 4.53× [4.28–4.61] | 240 / 152 |
| `callback_wide_record_return` | 35915.7 [34621.1–36330.8] | 2374.6 [2187.9–2540.8] | 15.13× [13.63–16.24] | 912 / 360 |
| `record_input` | 28.8 [28.5–30.2] | 155.0 [147.3–160.0] | 0.19× [0.18–0.19] | 0 / 128 |
| `wide_record_input` | 94.8 [93.5–95.1] | 327.2 [325.9–332.4] | 0.29× [0.28–0.29] | 0 / 320 |
| `scalar` | 13.5 [13.0–14.0] | 273.5 [249.5–285.8] | 0.05× [0.05–0.06] | 0 / 152 |
| `control` | 14.5 [13.2–14.7] | 14.1 [13.7–14.9] | 1.03× [0.95–1.07] | 0 / 0 |

Bulk wins here for single-record returns in both directions. Direct Rust returns
avoid a native-to-JVM success capture callback, but allocate more JVM bytes:
80 → 208 B for eight fields, and 272 → 592 B for 32 fields. Returning a record
from Kotlin callbacks gives larger wins in this run. The ordinary adapter returns
eight or 32 boxed fields, which Rust reads through checked JNI readers. The
benchmark measures that complete path, including lookup and local-reference work;
it does not isolate which part dominates its cost. The current bulk callback
adapter still returns a two-slot object array containing the frame and its length;
the frame replaces the eight or 32 individually boxed fields. These measurements
include that remaining wrapper cost.

Expanded primitive arguments stay faster, including the 32-field record. The
identical ordinary `control` path is 14.5 versus 14.1 ns (1.03×); its per-JVM ranges
overlap. The return improvements in this run have separated timing ranges and
are larger than that control difference. These observations describe this host
and current generated code; they do not establish a universal crossover point.

Run only this comparison with:

```sh
KOTLINC=/path/to/kotlinc taskset -c 0,1 python3 examples/bench-bulk/run.py \
  --output /tmp/bulk-single-records --check-generated --warmup 10 \
  --jvms 4 --batches 9 --target-ms 30 \
  --cases control,scalar,record_input,record_return,wide_record_input,wide_record_return,callback_record_return,callback_wide_record_return
```

`--cases` accepts any comma-separated subset of case names from the matrix.
Both modes call the same Rust source functions. Plain uses the generator defaults;
bulk uses the explicit overrides in `bindings.rs`. The generated native
callback-fire and control wrappers are identical after normalizing package names. Generated function returns and callback adapters were
also inspected to verify that every intended bulk override changed transport.

## Collection input helper removal

[The before/after report](collection-input-results/REPORT.md) measures #824 after
#854 against the Kotlin input-helper removal, using identical benchmark fixtures.
It includes primitive and record inputs, borrowed handles, records containing
handles and consuming handle round trips in both JNI transport modes. It records
allocation reductions, bulk handle-input speedups, and the cases where latency
was unchanged or slower. The comparison runner, all process medians and raw
samples are included.
