# prebindgen: occurrence conversion slots

This branch replaces the generator engine with a small pipeline. The source
capture macro and source-reading support remain unchanged. The supported JniGen
`build.rs` declarations keep their existing syntax and import paths;
`prebindgen-registry` is a declaration re-export facade, with no old registry
engine behind it.

```text
#[prebindgen] source + build.rs declarations
        ↓ select declared functions, constructors, getters and referenced types
Flat<Slot<Extra>>: source type + conversion recipe + adapter data per occurrence
        ↓ prebindgen-tools: defaults, overrides, validation and composition
Flat<Prepared<AdapterWire>>: input preparation, access requirements, output delivery
        ↓ C or JNI wrapper writer
Rust wrappers + C header / Kotlin files
```

A conversion belongs to an occurrence, so two parameters of the same Rust type
can have different input recipes. `Flat<S>` also preserves named record fields,
source names and locations. There is no parallel `TypeKind` tree. Wrapper
writers use prepared slots; recursive conversion decisions belong to tools.

## Examples migrated from main

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

Source selection happens before subset validation. Unbound enums, callbacks,
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
output overrides, and per-function overrides. These fill slots before preparing
wrappers. The migrated C and JNI examples exercise Report constructor/handle input
variants and `.fields(fields!(report_to_struct)).field_self()` output delivery.
BlobValue and Unsigned use handle classes; data-class generation stays disabled.

C uses `Cbindgen::builder().source(...)`, `.expand(...)`, `.function(fun!(...))`,
`.prefix(...)` and `.opaque_ptr(name, type)`. Opaque declarations generate distinct
C pointer types and typed destructors. Its old naming and cbindgen postprocessing
settings remain commented out in the example manifests.

Manual slot configuration remains available for generator development, but none
of the migrated binding examples requires it. `Input`, `Output` and `Recipe`
are non-generic conversion recipes. `Slot<E>` stores a recipe beside the
adapter's extra data; the source type remains on the flat occurrence. Preparation
receives all three separately. Declaration filling sets or overrides only the
recipe and preserves the extra data, without consulting a codec.

The same occurrence data is passed through its entire recipe, including nested
fields, constructors, selectors, getters and combined outputs. It is not replaced
with the extra data of a referenced field or function declaration. This subset
does not provide separate metadata per nested recipe node. C and JNI currently
use `()` as their extra data.

Prepared wires follow the same separation: `WireKind` is non-generic, while
`Wire<W>` stores the common kind and adapter data in separate `kind` and `extra`
fields. The adapter supplies data for common conversions through `wire_extra`
and for foreign conversions in their prepared wires. The framework does not
interpret this data or require foreign metadata; an adapter can use `W = ()`.
C and JNI use optional pointer metadata and check their own requirements.
`Prepared<W>` retains these wires. Tools compose primitives, text/bytes, adapter
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
declarations, a codec without wire metadata, C and JNI generation, boxed borrows,
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
