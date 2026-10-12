# prebindgen: occurrence conversion slots

This branch contains a small replacement pipeline for generating C and
JNI/Kotlin bindings from `#[prebindgen]` source. It is an experimental subset,
not a drop-in replacement for the full published adapters. The previous
registry engine, adapters, examples and documentation have been removed. The
`prebindgen-registry` crate now only re-exports the supported declarations so
existing build-script imports remain valid.

The source capture macro and its source-reading/configuration support remain
unchanged. The new pipeline is:

```text
#[prebindgen] source
        ↓
build.rs declarations → defaults and per-function overrides
        ↓
Flat<Slot>: syn::Type + a converter at each parameter/result occurrence
        ↓ prebindgen-tools: validate and compose the converters
Flat<Prepared>: wire roles, input construction, output delivery
        ↓ C or JNI wrapper writer
Rust wrappers + C header / Kotlin files
```

`Flat<S>` also preserves named record fields, source names and locations.
`TypeUse<S>` holds `syn::Type` and a slot; there is no parallel `TypeKind` tree
or global converter registry. Sharing an input recipe does not share runtime
values. A function is selected by filling its result slot, and all its parameter
slots must then have input conversions.

## The demonstrated subset

Both adapters use the declarations in [examples/declarations.rs](examples/declarations.rs).
Neither build script constructs a flat model or assigns conversion slots:

```rust
use prebindgen_registry::{expand_param, expand_return, fields, fun};

let key = expand_param!(KeyExpr)
    .variant_self()
    .variant(fun!(keyexpr_from_string));
let payload = expand_param!(Payload)
    .variant_self()
    .variant(fun!(payload_from_fields))
    .variant(fun!(payload_from_string))
    .variant(fun!(payload_from_slice));
let output = expand_return!(Payload)
    .field_self()
    .fields(fields!(payload_parts)
        .field("key", expand_return!(KeyExpr)
            .field_self().field(fun!(keyexpr_to_string)))
        .field("data", expand_return!(Vec<u8>).field_self()));
let function = fun!(foo)
    .expand_param("y", expand_param!(Payload).variant_self());
```

The generators accept these through `.expand(...)` and their function declarations.
`payload_from_fields` is an ordinary constructor taking the record's fields;
`payload_parts(&Payload) -> Payload` returns a cloned record for `.fields(...)`.
These use the existing JniGen declaration syntax. Shared declaration processing
in `prebindgen-tools` recursively fills occurrence slots before preparation.

The source function is `foo(k: KeyExpr, x: Payload, y: Payload) -> Payload`,
where `Payload` contains a `KeyExpr` and a `Vec<u8>`. This demonstrates:

- Different conversions for two occurrences of the same `Payload` type.
- Nested choices: a handle, record fields, or either of two constructors.
- Temporary storage retained through a constructor taking `&[u8]`.
- Four outputs from one result: the original payload handle, an independent
  cloned key handle, its text, and an independent cloned byte-vector handle.
- All 14 concrete input combinations through one native wrapper per operation.

Tools own recursive construction, field access, choice and output composition.
Adapters supply text/byte leaf decoding and the C/JNI wire carriers. Wrapper
writers only apply prepared slots and arrange boundary transport and errors.
An output combination evaluates observations before its consuming conversion,
while retaining the declared wire order. It rejects two moves of the same value
or a move from a borrowed field. `ClonedHandle` explicitly requires `Clone`.

The retained source grammar includes public nongeneric free functions, public
named records, opaque aliases, `String`, `Vec<u8>`, borrowed text/byte inputs and
unit. Types use declared flat names and the supported bare standard names;
arbitrary Rust type equivalence and arbitrary qualified paths are not resolved.
Unsupported source forms and incomplete conversions produce generation errors.
Constructors must return the required type directly; getters take one shared
borrow and return an owned value. Fallible source returns are not included yet.

## C and JniGen

[C's build script](examples/demo-c/build.rs) uses `Cbindgen::builder().source(...)`,
`.expand(...)`, `.function(...)` and typed handle destructor declarations, then
writes Rust plus a C header.
Each function returns status and fills an output struct. Strings and bytes use
pointer/length buffers. Active input buffers are borrowed during the call;
returned buffers must be freed exactly once with the generated buffer-release
function. A nonzero status leaves the output struct untouched. `last_error`
returns a thread-local message valid until the next boundary call on that thread.
C callers must supply valid readable input memory and writable output memory.

[JniGen's build script](examples/covertest-kotlin/build.rs) retains the existing
`source`, `package!`, `ptr_class!`, `fun!`, `.name()`, package-prefix and native
initialization declarations. The supported `expand_param!` constructor/identity
variants, `expand_return!` getter/identity fields, `.fields(fields!(...))` with
per-field overrides, and per-function overrides translate into the same tools
recipes. `build()` returns `JniGen`; `write_rust` resolves relative paths under
`OUT_DIR` and returns the written path; `write_kotlin` returns the written paths.
The supported declarations retain their production imports and accept owned
strings as before. Settings can follow declarations. `.configure(...)` and
`.flat(...)` remain optional low-level entry points for generator development.

Kotlin owns typed `AutoCloseable` handle classes. The current input surface
exposes a selector and arguments for all alternatives, including nullable dummy
arguments. Only the selected branch decodes or consumes them. Multi-value
outputs travel in a JNI object array and reach a typed Kotlin builder lambda.
If that lambda throws, its output handles are released. This transport is kept
simple to exercise the pipeline; it does not implement the full production
JniGen delivery surface or its performance optimizations.

Both adapters use an opaque numeric handle store. A wrong-type or stale handle
is an error, not a pointer dereference. Input handles transfer ownership when
that conversion executes. If a later input fails, earlier consumed handles stay
consumed and their Rust values are dropped; unvisited handles stay caller-owned.
Output handles are released if result encoding fails before handoff. Closing a
consumed or already closed handle is harmless. Concurrent attempts to consume
one handle allow exactly one successful take.

Rust panics are caught at each boundary when compiled with unwinding enabled;
they become a C error status or a JNI exception. Conversion errors use the same
boundary channels. This subset has no typed source-error delivery.

## What is deliberately absent

There is one pipeline and no legacy fallback. The following remain outside this
first subset: ordinary scalar conversions, enums and sums, options, general
collections, callbacks, borrowed/mutable handle inputs, methods, generated data
classes, `convert!`, consuming `.fields_self_into(...)`, field renaming, local
function declarations, domain errors, overload splitting, multiple specialized
native wrappers, and shared storage between selector alternatives. New support
should be added to this pipeline and exercised by both boundary tests as
applicable. Every JniGen feature belongs in the Kotlin covertest.

## Build and verify

Rust 1.85+, a C compiler, JDK 11+ and a Kotlin compiler are required for the full
boundary check. Build once online to fetch dependencies; the test script uses
Cargo's offline mode thereafter.

```sh
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all -- --check
python3 examples/check.py
```

`KOTLINC` and `CC` select the compilers. `SANITIZE=1 python3 examples/check.py`
enables AddressSanitizer and UndefinedBehaviorSanitizer on the C caller.
The script locates artifacts from Cargo's JSON build messages and runs JNI with
`-Xcheck:jni`; generated files stay under Cargo's output directory.

The C and Kotlin tests cover the 14 input combinations, output ownership,
inactive branches, invalid tags, stale and wrong-type handles, partial input
failure and source panics. Both examples exercise the declaration API throughout, including nested
expansions, record-field output overrides and per-function replacement of defaults.
Kotlin additionally exercises builder failure cleanup and concurrent consumption. Rust tests cover source
validation, occurrence independence, invalid compositions and output rollback.
