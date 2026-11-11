# Canonical values versus explicit expansions

The comparison is between two configurations of the same Rust domain value:
`convert!(BenchValue)` with one input constructor and one shared output helper,
and explicit `expand_param!` / `expand_return!` trees calling those same helpers.
Both expose Kotlin `Long`, `List<Long>`, or a callback taking/returning `Long`.
The source functions perform identical work. This measures conversion machinery,
not the cost of a domain-specific transformation or application throughput.

The ordinary array path passes JVM objects. The bulk array path packs the same
values into C-representation rows. Scalar and scalar-callback cases use the normal
scalar ABI in both namespaces; their namespace does not select bulk transport.

## Method

Measured on Linux x86-64, AMD EPYC 7713, two assigned CPUs, Temurin OpenJDK
11.0.29 and Rust 1.98.0 with release optimization. The fixture pins dependencies
in `Cargo.lock.snapshot` and freezes each built JAR and native library before
measuring. No builds ran during timing.

The first campaign used eight fresh JVMs per transport, alternating transport
order and rotating/reversing case order, five warmup batches and nine measured
batches targeting 35 ms. The array-only follow-up used eight fresh JVMs per
transport, seven warmup batches and eleven measured batches targeting 40 ms.
Numbers below are medians of each JVM's batch median. JVM allocation bytes come
from `ThreadMXBean`; native allocations use a separate instrumented release build,
with 1,000 warmup calls and ten counted calls per case. Timing runs disable native
allocation instrumentation. The measured generator code was `ed8268b48`; subsequent
changes tighten generation-time diagnostics and record artifacts, without changing
the successful conversion code used by these cases.

## Latency and JVM allocations

| Case | Elements | Explicit expansion ns/call | Canonical ns/call | Change | JVM bytes/call, both |
|---|---:|---:|---:|---:|---:|
| Scalar | 1 | 14.0 | 14.1 | +0.7% | 0 |
| Scalar callback | 1 | 732.0 | 729.6 | −0.3% | 32 |
| Ordinary array round trip | 1 | 1,340.1 | 1,353.7 | +1.0% | 216 |
| Ordinary array round trip | 64 | 38,556.6 | 39,635.1 | +2.8% | 7,280 |
| Ordinary array round trip | 512 | 303,317.1 | 311,519.1 | +2.7% | 57,456 |
| Bulk array round trip | 1 | 450.0 | 454.1 | +0.9% | 384 |
| Bulk array round trip | 64 | 1,334.6 | 1,264.4 | −5.3% | 3,424 |
| Bulk array round trip | 512 | 7,135.3 | 6,610.7 | −7.4% | 24,928 |

Scalar rows use the first campaign; array rows use the longer-warmup follow-up.
The first ordinary single-element array measurement was 1,330.9 ns for explicit
expansion and 1,437.5 ns for canonical conversion (+8.0%); the follow-up reduced
that gap to +1.0%. These differences vary with JVM compilation and case mix.
The measurements do not establish an extra operation in the canonical path:

- LLVM merged the canonical and explicit scalar JNI entry points to the same
  address, separately in each namespace.
- Both ordinary array entry points call the same optimized native conversion
  body. Their normalized Kotlin bytecode is identical after replacing function
  names and constant-pool indices.
- Bulk Kotlin bytecode differs only in layout getter identifiers. Both layouts
  carry the same field types, sizes and offsets; the identifiers reflect the
  different declaration trees.

The comparison found matching allocation counts and shared conversion code, with
small end-to-end timing differences. It provides no basis for claiming a global
application performance change or an acceleration from the new API.

## Native allocations

The canonical and explicit configurations produced identical counts in every
counted call. Bytes are cumulative requested allocation/reallocation sizes,
not retained memory.

| Case | Elements | Allocations, both | Reallocations, both | Requested bytes, both |
|---|---:|---:|---:|---:|
| Scalar | 1 | 0 | 0 | 0 |
| Scalar callback | 1 | 3 | 2 | 48 |
| Ordinary array | 1 | 2 | 1 | 41 |
| Ordinary array | 64 | 2 | 1 | 545 |
| Ordinary array | 512 | 2 | 1 | 4,129 |
| Bulk array | 1 | 3 | 1 | 80 |
| Bulk array | 64 | 3 | 1 | 1,584 |
| Bulk array | 512 | 3 | 1 | 12,336 |

Scalar/callback counts also match in the second namespace. Resource creation
required by the selected representation remains visible; the canonical API adds
no allocations to these equivalent expansions.

## Reproduction

Use `examples/bench-bulk/run.py` with cases `canonical_value`, `expanded_value`,
`canonical_value_callback`, `expanded_value_callback`, `canonical_values` and
`expanded_values`. Build with `--build-only`, then reuse that output with
`--prepared`; use `--jvms 8 --warmup 7 --batches 11 --target-ms 40 --sizes 1,64,512`
for the array follow-up. Record/check generated files with the existing
`--record-generated` / `--check-generated` switches.

For native counts, make a separate `--native-allocations --build-only` output.
Run `bench.allocations.AllocationBenchKt <plain|bulk> <case> <size>` against that
output's JAR and library. Keep this instrumented build separate from latency runs.
