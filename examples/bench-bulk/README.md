# Bulk JNI transport comparison

This fixture binds the same Rust functions twice in one release native library:
`bench.plain` uses ordinary JNI conversion and `bench.bulk` adds `.bulk()` to the
relevant expansion. There are no generator/runtime changes in this benchmark.

```sh
KOTLINC=/path/to/kotlinc python3 examples/bench-bulk/run.py \
  --output /tmp/bulk-bench --record-generated
```

Use an empty output directory. Cargo builds the isolated fixture offline, then
Kotlin compiles both generated surfaces into one JAR. All builds finish before
measurement. Four fresh JVMs per mode run in alternating plain/bulk order, with
case order rotated/reversed across rounds. An untimed pass over all cases trains
dispatch profiles before any per-case calibration. Each case then calibrates its
batch size, warms up five batches, and measures nine batches targeting 30 ms each. Each JVM
contributes its median; the report summarizes the median and range of those JVM
medians and preserves every sample. There are no performance pass/fail thresholds.

The driver uses an unboxed primitive SAM interface (`run(Int): Long`), consumes all eight
record fields, and checks closed-form batch checksums. Callback cases additionally
check invocation counts. Inputs and callback registration are prepared before
timing; layout lookup/class loading occur during untimed calibration. Native
conversion, source function work, Kotlin reconstruction/consumption and normal
wrapper/handle bookkeeping remain timed. Native builds require clean, committed repositories for all path dependencies
discovered by Cargo, including this fixture, and use the committed dependency lock. Rust is built in release mode; timing
uses ordinary JNI checks, without `-Xcheck:jni`. Absolute timings include SAM dispatch, which can be megamorphic. Actual batch durations are recorded; calibration is capped at five million calls.

The matrix includes batches of 0, 1, 8, 64 and 512 elements:

| Case | Timed work | Reason to compare |
| --- | --- | --- |
| `scalar` | One `i64` argument/result | Ordinary primitive parameters should beat allocating/copying frames. |
| `record_input` | One eight-`i64` record to a Rust sum | Flattened primitives already avoid object access; bulk can add unnecessary work. |
| `packet_input` | Eight-field header plus 64 B, 4 KiB or 64 KiB byte payload, to a Rust checksum | Bulk adds Kotlin-frame and native decoding copies; ordinary transport can be better for payload-heavy packets. |
| `numbers_input` | `Vec<i64>` to a Rust sum | Bulk can avoid object-array/boxed-number access for each input element. |
| `records_input` | Shared record slice to a Rust sum | Bulk can combine many expanded fields in one byte frame. |
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
