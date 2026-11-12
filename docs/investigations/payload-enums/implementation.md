# Payload enums: implementation and verification

A source enum now keeps its variants and payload fields through the shared
conversion compiler. C receives a tag and a union of converted fields. Kotlin
receives a sealed value, or the selected expanded fields when the binding asks
for expansion. Bulk arrays use the same selected-field tree, with a target
measured tag/union layout. No native allocation is introduced for the enum itself.

## Configuring a value or an expansion

For `enum Event { Idle, Count(u64), Packet { id: u64, bytes: Vec<u8> }`:

```rust,ignore
// Ordinary Kotlin values: Event.Idle, Event.Count(v0), Event.Packet(id, bytes).
.package(package!()
    .class(sealed_class!(Event))
    .fun(fun!(exchange)))

// Ordinary C values: event.tag and event.payload.Packet.{id, bytes}.
.tagged_union("event", syn::parse_quote!(Event))
.function(fun!(exchange))
```

Ordinary payload properties use their canonical value conversions. For example,
a configured Duration-to-milliseconds conversion also applies inside an enum.
An explicit expansion instead inherits configured boundary expansions:

```rust,ignore
.expand(expand_return!(ReplyResult).view(Output::enum_fields([])))
.expand(expand_return!(Reply).fields_into(fields!(reply_into_struct)))
```

This callback can deliver a tag, expanded Sample or ReplyError fields, and a
sibling replier id without creating a foreign ReplyResult class. To replace one
field's expansion locally, supply source variant/field keys:

```rust,ignore
fun!(exchange).expand_return(Output::enum_fields([
    ("Packet", vec![("bytes", Output::value())]),
]))
```

Tuple field keys are `"0"`, `"1"`, etc.; ordinary Kotlin tuple properties are
`v0`, `v1`, etc. Unknown keys and incompatible grouped properties are rejected.
`Input::enum_fields` supplies the matching input operation. The lower-level
`Input::Enum`/`Output::Enum` trees allow grouped per-field overrides as well.
The existing `.bulk()` modifier applies to these recipes, including sequences.

A callback's `Result<T, E>`, a record's Result property and a nested Result are
ordinary values. The outer Result of an exported function remains its typed
success/error channel. `.result_as_value()` opts that whole function result into
data conversion; it cannot also specify `.on_error(...)`. A concrete Kotlin
Result declaration needs an explicit name, such as
`sealed_class!(Result<i64, String>).name("Decision")`.

## Ownership and generated code

* [Flat variants](../../../prebindgen-flat/src/payload.rs) retain source
  constructor syntax and register concrete standard Results.
* [Shared compilation](../../../prebindgen-declarations/src/variants.rs)
  constructs or matches only the active variant. Output preparation retains one
  statically typed selected commit guard; it adds no heap owner or per-row plan.
* [C carriers](../../../prebindgen-c/src/tagged.rs) contain converted fields.
  String/Vec owners stay inline and retain their original allocations. An owning
  input empties the caller's carrier, then replaces consumed owner slots with
  valid empty values or null pointers. Error cleanup releases the remaining
  active fields; already constructed Rust values drop normally. C still trusts
  valid input tags, pointers and ownership, with no native alias checks.
* [Sealed Kotlin values](../../../prebindgen-jni/src/sealed.rs) use primitive
  and existing buffer/handle encodings at JNI. Kotlin performs the usual handle
  preflight. Ordinary values do not require JNI property traversal. Owning sealed
  values implement `AutoCloseable`; closing visits only their owning fields.
* [Target layouts](../../../prebindgen-reprc/src/tagged.rs) are shared by C and
  [JNI bulk](../../../prebindgen-jni/src/bulk.rs). Native `size_of`, `align_of`
  and component `offset_of` measurements initialize Kotlin's layout table.
  Readers inspect tags/discriminators before active fields; they never read
  inactive union bytes. Optional discriminators retain their actual wire widths.

Tags are source-order indices starting at zero, independently of Rust enum
numeric discriminants. C reserves -1 for an empty owning carrier. Named tag
constants are emitted even for field-only expansions, and JNI initialization
checks the complete payload tag mapping against the loaded library. Fieldless
unit-syntax enums keep their existing numeric representation.

## Boundary coverage

The committed generated artifacts accompany executable callers:

* [C callers](../../../examples/perftest-c/c/payload_enums.h): scalar and empty
  variants, owner take/drop, buffer pointer identity across a callback, nested
  Result data, arrays of owning variants and failure after a partial move.
  An absent optional vector contains deliberately invalid owner bytes; neither
  input conversion nor destruction reads them.
* [Kotlin callers](../../../examples/covertest-kotlin/kotlin/io/prebindgen/covertest/PayloadEnums.kt):
  optional RecoveryMode and independent retention, mixed instrumentation rows,
  consuming Reply-style expansion without reply classes, handle-bearing optional
  records with local overrides, nested sealed values, callback arguments/returns,
  nested Results, typed enum errors and failure cleanup.
* [Generation probe](probe/src/main.rs): both adapters, concrete callback Results
  and selected real Zenoh declarations. Its optional snapshot check compares 16
  normalized declaration shapes with the pinned upstream Git object.

The source audit's independent prerequisites remain separate: binding-local
helper signatures, declared scalar domains, fixed-array record properties and
scoped borrowed C callbacks. Source generic enums, recursive value definitions
and non-exhaustive/conditional payload declarations remain outside the supported
Flat subset. Moving fields from a source enum implementing Drop requires a
suitable explicit conversion, as for owned record decomposition.

## Reproducing validation

```sh
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
python3 examples/record-generated.py --check
SANITIZE=1 python3 examples/check.py
cargo run --manifest-path docs/investigations/payload-enums/probe/Cargo.toml -- \
  --zenoh-flat ../zenoh-flat
```

`examples/check.py` compiles C/C++ callers and Kotlin/Java callers, runs the JNI
suite with `-Xcheck:jni`, and checks nullability/ownership rejection cases.
The generated-source check compares the exact Cargo output directories.

## Transport measurements

The optional [benchmark](../../../examples/covertest-kotlin/kotlin/io/prebindgen/covertest/PayloadBench.kt)
compares a reused sealed scalar with equivalent explicit tag/field arguments,
the same scalar packed into bulk, two owning callback arguments with their
sealed equivalent, and 256 mixed scalar variants through ordinary object rows
or bulk rows. It reports whole-wrapper latency and JVM allocated bytes per call.
Native owner allocations in the two callback cases are the same source operation.

Build `covertest-kotlin` in release mode, compile its matching generated and
handwritten Kotlin sources as in `examples/check.py`, and run that jar with
`payload-bench` and the release native-library directory. Keep native and Kotlin
artifacts from the same build: JNI rejects mismatched payload mappings.

### Measured result

Linux x86-64, Rust 1.98.0 release optimization, Temurin OpenJDK 11.0.29, 512 MiB
JVM heap, one CPU affinity. Three fresh JVMs; each warms all seven cases, then
rotates their order over nine measurement rounds. The table reports the median
of the three JVM medians. Native allocation accounting is a separate instrumented
build; its timings are not used.

| Case | ns/call | JVM bytes/call | Native allocations/reallocations per call |
| --- | ---: | ---: | ---: |
| Explicit scalar tag/fields | 23.45 | 24 | 0 / 0 |
| Reused sealed scalar | 24.18 | 24 | 0 / 0 |
| Bulk scalar | 125.52 | 112 | 1 / 0 |
| Callback with two explicit owners | 833.40 | 72 | 5 / 2 |
| Callback with owning sealed variant | 950.34 | 104 | 5 / 2 |
| Ordinary object rows, 256 variants | 261,400 | 13,392 | 1 / 0 |
| Bulk rows, 256 variants | 2,765 | 7,368 | 2 / 0 |

The lambda-based harness boxes its ULong result: the common 24 JVM bytes are not
an enum allocation. Reusing the sealed scalar adds about 0.73 ns (3.1%) over the
manual tag/fields control; bulk packing is about 5.2 times slower for this small
scalar. The owning sealed callback allocates one additional 32-byte Kotlin
variant object and takes about 14% more time than the explicit two-owner callback.
Field-only expansion avoids that object when the consumer wants direct fields.
The native callback allocation counts are equal; these include existing owner
and callback bookkeeping, not just resource construction.

For 256 mixed variants, bulk is about 94.5 times faster than ordinary JNI object
rows. Bulk uses a native copy of the packed frame plus the destination Rust Vec,
versus the Vec alone for object rows: 12,304 versus 6,144 native allocated bytes
per call. Avoiding per-row JNI decoding dominates that additional frame storage.
The three fork medians ranged from 256.5–264.9 microseconds for object rows and
2.72–2.87 microseconds for bulk. These are transport microbenchmarks, not packet
throughput results or a claimed global speedup.

Target-measured layouts on this host are 24 bytes for the bulk scalar enum row
(alignment 8; flattened field offsets 0, 8, 8, 16), 40 bytes for C WireEvent
(which contains an inline vector owner), 32 bytes for C WireOwnedEvent and 4 bytes
for the all-empty C WireEmpty. Fields of different union arms deliberately share
offsets. A larger variant makes every row that large; bulk is not always the
best choice for a highly skewed enum with one very large arm.

The compact runner builds the same fixture functions and the public generator
API without compiling unrelated covertest wrappers:

```sh
taskset -c 1 python3 examples/covertest-kotlin/bench-payload.py \
  --output /tmp/payload-latency --forks 3
python3 examples/covertest-kotlin/bench-payload.py \
  --output /tmp/payload-allocations --count-native --forks 1
```

Set `KOTLINC` if the compiler is not on PATH. `--target-dir` can reuse a release
Cargo cache. Generated sources, matching native library/JAR and per-fork logs
stay under the requested output directory. The C pointer-identity tests check
that forwarding a vector payload preserves its allocation.
