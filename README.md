# prebindgen: occurrence conversion slots

prebindgen generates C and JNI/Kotlin bindings from annotated Rust source.
The flat model describes source signatures and types. Build-script declarations
configure conversions for each parameter and result; shared tools prepare the
conversions, and language adapters emit wrappers and foreign declarations.

```text
#[prebindgen] source + build.rs declarations
        ↓ select declared functions, constructors, getters and referenced types
Flat with empty conversion slots
        ↓ prebindgen-tools builds and attaches converters
Bindings<W> = Flat<Option<InputConversion<W>>, Option<OutputConversion<W>>>
        ↓ C or JNI wrapper writer
Rust wrappers + C header / Kotlin files
```

A conversion belongs to an occurrence, so two parameters of the same Rust type
can have different input recipes. `Flat<I, O>` attaches consumer-defined `I` data
to parameters and `O` data to results. Record declarations contain only names and
source types; their fields have no conversion slots. Flat also preserves source
names, locations and guards. Wrapper writers use conversion slots; recursive
conversion decisions belong to tools.

## Examples migrated from main

This branch replaces the generator engine while preserving source capture and
source-reading support. The supported JniGen `build.rs` declarations keep their
existing syntax and import paths. `prebindgen-registry` re-exports declaration
types and macros from `prebindgen-tools`.

The artificial `demo-flat`, `demo-c`, and shared demonstration declarations have
been removed. All eight Rust example crates from `main` are in the workspace.
Unsupported declarations, record fields and matching constructor arguments are
commented in the flat sources and `build.rs` files as the checklist for future
work. The retained records demonstrate the supported shapes of the real examples.

| Example | Currently active |
|---|---|
| [example-flat](examples/example-flat) | Calculator/Error aliases, Caption and Foo with primitive fields |
| [example-cbindgen](examples/example-cbindgen/build.rs) | Calculator/Caption/Foo creation, Foo ID extraction and typed release |
| [perftest-flat](examples/perftest-flat) | Payload scalar fields, Stamp, Summary, Unsigned, BlobValue and Report projections |
| [perftest-c](examples/perftest-c/build.rs) | Storage, strings, BlobValue transfer, Report constructor selectors and label/pointer outputs |
| [perftest-kotlin](examples/perftest-kotlin/build.rs) | Payload construction/projection and Storage creation/release through JNI |
| [covertest-kotlin](examples/covertest-kotlin/build.rs) | The same BlobValue/Report conversions, strings, Storage, subpackages and the SummaryValue class rename |
| [covertest-helpers](examples/covertest-helpers) | Unsupported helper functions and multi-source bindings are commented out |
| [emitcheck](examples/emitcheck/build.rs) | Plain string/byte-vector fields compile; callback entry points remain commented, so it currently emits no API wrappers |

Source selection happens before subset validation. Automatic dependency discovery
follows unqualified declared type names. Types referenced only through qualified
paths need explicit selection by their unqualified declaration name, for example
with `ptr_class!(Foo)` or `.opaque_ptr("foo", syn::parse_quote!(Foo))`; an already
parsed `Flat` model can also include them. Unbound enums, callbacks,
collections and functions can stay in the source crate; selecting one still
produces a generation error when its shape is unsupported. Both annotated flat
examples also have tests that parse the entire captured surface, ensuring no
unsupported shapes remain active behind that selection.

The old foreign performance drivers require the commented bindings and have
been removed. Full coverage clients are replaced with focused C and Kotlin smoke
clients exercising the active declarations; this branch makes no performance or
full downstream compatibility claim. Generated artifacts remain under `OUT_DIR`.

Named flat objects must have nonzero Rust size, including opaque aliases and
records delivered as fields. The model emits compile-time size assertions for
all selected named types; adapters must emit these guards. This checks actual
Rust layout, including hidden or nested zero-sized types. Unit results remain
supported. The reduced `Archive` example is disabled because its remaining
fields give it zero size.

## Supported generator API

JniGen supports `source(...)` or `items(...)`, `package!`, `ptr_class!`, `fun!`,
`.name()`, package prefix and native initialization. `build()` returns `JniGen`;
`write_rust` resolves relative paths under `OUT_DIR` and returns the written path;
`write_kotlin` returns the written paths. Settings may follow declarations.

The tools layer also supports `expand_param!` constructor/identity variants,
`expand_return!` getter/identity outputs, `.fields(fields!(...))` with per-field
output overrides, and per-function overrides. These build converters directly
in slots before generating wrappers. The migrated C and JNI examples exercise
Report constructor/handle input variants and `.fields(fields!(report_to_struct)).field_self()` output delivery.
BlobValue and Unsigned use handle classes; data-class generation stays disabled.

C uses `Cbindgen::builder().source(...)`, `.expand(...)`, `.function(fun!(...))`,
`.prefix(...)` and `.opaque_ptr(name, type)`. Opaque declarations generate distinct
C pointer types and typed destructors. Its old naming and cbindgen postprocessing
settings remain commented out in the example manifests.

Manual conversion binding remains available for generator development, but none
of the migrated binding examples requires it. `Bindings<W>` has separate optional
`InputConversion<W>` and `OutputConversion<W>` slots. Each stores only wires,
conversion code and resource/ownership steps needed for wrapper emission.
`bind_input` and `bind_output` compile a recipe directly into one named occurrence.
`Declarations::fill` resolves defaults and overrides and compiles every needed
conversion before updating a function. A failed construction leaves its slots
unchanged. Existing converters survive default filling; explicit overrides replace
them using the codec input data supplied to that call.

`Input` and `Output` recipes and `Codec::Extra` are construction inputs. Neither is
stored in Flat, and there is no whole-model recipe-to-prepared transition or source
model clone. Nested fields, constructors, selectors, getters and combined outputs
use the enclosing call's extra data. Referenced helpers supply source signatures,
and record declarations supply field names and types. This subset does not provide
separate adapter data per nested recipe node. C and JNI use `()` for `Extra`.

Direction is enforced by the slot types. `validate_bindings` checks completeness
before emission: entirely unbound functions remain source-only helpers; partially
bound functions are rejected. Recipe/type compatibility is checked when attaching
a converter. Flat itself knows nothing about recipes, conversions or processing
stages. Names are scoped to parameter positions and the result, so independent
binding calls and overrides cannot collide with other occurrences in a wrapper.

The manual tools API now uses `Bindings`, `bind_input`/`bind_output` and
`Declarations::fill(flat, codec, declaration, extra)` instead of `Configured`,
`Slot` and `prepare`. C exposes `Bindings` and `CCodec`, and JNI exposes `Bindings`
and `JniCodec`; their manual model/callback APIs accept final conversion slots.
The supported declarative `build.rs` APIs retain their syntax.

A wire is one generated boundary parameter or result component. `Wire<W>`
stores its common role in `kind` and adapter data in `extra`. The adapter supplies
`extra` for scalar, text, byte and selector wires through `Codec::wire_extra`;
foreign conversion hooks supply it with their prepared wires. An adapter that
needs no per-wire data uses `W = ()`.
C and JNI use `Option<BoxedWire>`: foreign wires store the pointee type and access
mode in `Some(...)`, while other wires use `None`. Each adapter validates that
data when generating its wrappers.
`Bindings<W>` retains these wires. Tools compose primitives, text/bytes, adapter
leaves, fields, constructors, input selectors and combined outputs. The composition model
requires neither handles nor boxed storage. It has no object registry or universal
`Send` requirement.

The adapter implements foreign conversions for domain types and declares
whether an output consumes its value. Tools run observations before that move
and reject double consumption. For pointer/handle output from borrowed record
fields, these adapters make independent owned copies using Rust `Clone`.
A standalone owned text/byte result moves its buffer; an observed field is copied.

## Primitive conversions

All stable scalar types supported by Rust 1.85 have value converters, selected
by default in build-script declarations. Unit continues to use no wire values.
Field and constructor composition use the same converters as direct parameters
and results. The source model still stores `syn::Type`; primitive classification
only identifies the scalar leaf and leaves its representation to the adapter.

| Rust | C carrier | Kotlin surface / JNI carrier |
|---|---|---|
| `i8`, `i16`, `i32`, `i64` | matching `intN_t` | `Byte`, `Short`, `Int`, `Long` |
| `u8`, `u16` | matching `uintN_t` | `Int` / `jint`, checked on input |
| `u32` | `uint32_t` | `Long` / `jlong`, checked on input |
| `u64` | `uint64_t` | `ULong` / `jlong` bit pattern |
| `isize`, `usize` | `intptr_t`, `uintptr_t` | `Long`, `ULong` / `jlong`, checked against the native target width |
| `f32`, `f64` | `float`, `double` | `Float`, `Double` |
| `bool` | `uint8_t` | `Boolean` / `jboolean` |
| `char` | `uint32_t` Unicode scalar | `Int` Unicode scalar / `jint` |
| `i128`, `u128` | `<prefix>_int128 { uint64_t hi, lo; }` | `java.math.BigInteger` / `ByteArray` |

Boolean inputs normalize zero to false and any nonzero carrier to true; outputs
are 0 or 1. Character inputs reject surrogates and values outside Unicode's
scalar range, preserving supplementary characters. Floats retain infinities,
NaNs and signed zero. JNI unsigned mappings match the established JniGen API.

The C 128-bit pair carries the high and low halves of the value's bit pattern;
signed values use two's complement. JNI 128-bit inputs use BigInteger's big-endian
two's-complement bytes with range checks inside the selected converter. Outputs
use exactly 16 big-endian bytes; Kotlin reconstructs signed or unsigned BigInteger
values accordingly. No C compiler extension or narrowing to 64 bits is needed.
Native methods remain unmangled by Kotlin unsigned types because their carriers
use signed JVM types. Scalar `Long`/`Double` carriers count as two JVM parameter
slots when checking the method-size limit.

## Boundary contracts and limits

The call data flow has an explicit ownership boundary:

```text
select active alternatives
    → validate/decode all active leaves and check access conflicts
    → acquire all consuming inputs
    → construct arguments and invoke the source function
    → prepare outputs under Rust ownership
    → publish foreign outputs and transfer their ownership
```

Before acquisition, failure leaves every input owned by its caller. Acquisition
is infallible after validation; all consuming leaves become Rust-owned before
any constructor runs. A constructor/source panic or later delivery failure does
not restore consumed inputs. Rust guards reclaim acquired inputs and unpublished
outputs. Borrows last through the call, including constructors. Multiple shared
borrows of one allocation are allowed; an exclusive borrow or move conflicts
with every other active use of that allocation. Inactive selector arms neither
validate nor acquire their inputs.

**C owns pointer semantics.** Each declared opaque type has its own incomplete
C struct and `T *` / `const T *` inputs. The caller must supply live pointers of
the declared type, valid for the whole call, and serialize conflicting access.
Consumed pointers become invalid. Destruction accepts null, but destroying a
non-null pointer twice or using a consumed pointer is undefined behavior, as
with ordinary owned C pointers. There is no runtime type/liveness registry.
C imposes no `Send` bound: the LocalCounter example wraps `Rc<Cell<u64>>`.
Both current adapters opt into `prebindgen_tools::boxed`, which shares pointer
validation, borrow/move acquisition, cloned observations and output commit.
C stores objects directly in `Box<T>`; its header exposes typed opaque pointers.
Other adapters can choose different storage through their foreign conversion
hooks; there is no universal `Object` operation or required allocation policy.
Thread-affine source values retain their source-side restrictions.

C wrappers still return status and write output structs only on success. Text
and bytes use pointer/length buffers; `&str` and `&[u8]` inputs borrow that
memory, while owned inputs copy it. Callers free returned buffers exactly once.
`last_error` is thread-local; clearing it on success allocates nothing.

**JNI owns handle semantics.** As on `main`, a Kotlin object owns a boxed Rust
allocation whose address travels in a `Long`. A low pointer bit records closed
or consumed state. A JNI-specific wrapper guarantees the alignment needed for
that bit. Native objects require `Send` because Kotlin can transfer them between
threads; this restriction belongs to JNI.
Generated wrappers lock all active handle objects in a stable address order,
including handles nested in selected constructor arguments. Close uses the same
object monitor. Repeated close and close after consumption are harmless.

For calls with consuming inputs, a one-element boolean array acknowledges when
native validation has succeeded and ownership transfers. Kotlin marks the
selected objects consumed in `finally` only after this acknowledgement, including
on a caught native panic. This keeps preflight failures from closing caller
objects. It adds one allocation to those calls; it is not a performance claim.
Native entry points are internal Kotlin plumbing and require the generated
wrappers' pointer provenance and locking contract; arbitrary `Long` values are
not safe native inputs.

Rust panics are caught when unwinding is enabled. JNI still boxes outputs into
an object array. Multi-output delivery uses a builder lambda, outside input
locks; a throwing builder closes output handles, including escaped references.
C status/out-struct and JNI object-array ABI optimization remain separate work.

Callbacks, enums, options, general collections, borrowed pointer/handle outputs,
mutable text/byte views, methods, data classes, `convert!`, consuming record
decomposition, field renaming, local function declarations, domain errors, split
overloads and shared selector argument storage remain outside this subset.

## API documentation

Run `cargo doc --workspace --no-deps --open` to browse the API. Start with
`prebindgen_flat::Flat` for occurrence slots, `prebindgen_tools::Declarations`
for build-script expansions, and `prebindgen_tools::Codec` for adapter authors.
The codec and prepared-slot contracts explain validation, ownership acquisition
and output delivery; `Cbindgen` and `JniGen` document configuration and writing files.

Runnable rustdoc examples cover occurrence configuration, selector/combined-output
declarations, a codec with `()` as its wire data, C and JNI generation, boxed borrows,
and JNI allocation release. `cargo test --workspace --doc` builds and runs them.
The existing C and Kotlin smoke clients exercise the generated foreign boundaries.
New pipeline crates enable `missing_docs` warnings so the warnings-as-errors
Clippy check also detects undocumented public additions.

## Build and verify

Rust 1.85+, a C compiler, JDK 11+ and a Kotlin compiler are needed for all checks.
Build once online to fetch dependencies; the boundary script then uses offline
Cargo builds.

```sh
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all -- --check
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
python3 examples/check.py
```

The script builds every example and runs both C and both JNI smoke clients,
including every primitive at boundary values, invalid unsigned/Unicode/128-bit
inputs, restored Payload fields, string round trips, byte-vector constructors, consumed handles,
selector arms, field projections, builder rollback, output preservation and
concurrent close. Ownership coverage also exercises mutable/shared Storage
borrows, repeated shared arguments, opposite lock order, close/borrow/consume
races, later-selector/text failures, duplicate consumption, native panic after
transfer, C pointer type checking and a non-`Send` C object. `KOTLINC` and `CC` select the compilers.
`SANITIZE=1` instruments the C callers with AddressSanitizer and
UndefinedBehaviorSanitizer; JNI always runs with `-Xcheck:jni`.

The native Rust benchmark is commented out because it depends on disabled
optional-field and callback APIs. Its driver reports that limitation.
