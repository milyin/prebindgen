# prebindgen: occurrence conversion slots

prebindgen generates C and JNI/Kotlin bindings from annotated Rust source.
The flat model describes source signatures and types. Build-script declarations
configure conversions for each parameter and result; the declaration compiler builds the
conversions, and language adapters emit wrappers and foreign declarations.

```text
#[prebindgen] source + build.rs declarations
        ↓ select declared functions, constructors, getters and referenced types
Flat with empty conversion slots
        ↓ prebindgen-declarations builds and attaches converters
Flat<I: Conversion, O: Conversion>
        (parameter slots: Option<I>; result slots: Option<O>)
        ↓ C or JNI wrapper writer
Rust wrappers + C header / Kotlin files
```

A conversion belongs to an occurrence, so two parameters of the same Rust type
can have different input recipes. `Flat<I, O>` stores the two payload types in
optional slots. Both can implement `Conversion` with distinct wire kinds and request enums; the model itself does
not require that trait. Record declarations preserve Rust
source structure. Named representation schemas describe logical components,
source-field associations and adapter-specific type data `R`; wires refer to
components by stable paths. Flat also preserves source names, locations and guards.
Wrapper writers use flat's generation API; recursive recipe resolution belongs to declarations.

## Examples migrated from main

This branch replaces the generator engine while preserving source capture and
source-reading support. The supported JniGen `build.rs` declarations keep their
existing syntax. Import shared declaration types and macros directly from
`prebindgen-declarations` (`prebindgen_declarations` in Rust imports).

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
| [covertest-kotlin](examples/covertest-kotlin/build.rs) | Data-class round trips, Report projections, strings, Storage, selectors, subpackages and class renames |
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

JniGen supports `source(...)` or `items(...)`, `package!`, `ptr_class!`, `data_class!`, `fun!`,
`.name()`, package prefix and native initialization. `build()` returns `JniGen`;
`write_rust` resolves relative paths under `OUT_DIR` and returns the written path;
`write_kotlin` returns the written paths. Settings may follow declarations.

The declaration layer also supports `expand_param!` constructor/identity variants,
`expand_return!` getter/identity outputs, `.fields(fields!(...))` with per-field
output overrides, and per-function overrides. These build converters directly
in slots before generating wrappers. The migrated C and JNI examples exercise
Report constructor/handle input variants and `.fields(fields!(report_to_struct)).field_self()` output delivery.
Kotlin `data_class!(T)` emits a data class for a public named-field Rust record.
Supported fields are scalars, `String`, `Vec<u8>`, other declared data classes,
and declared opaque handles. Records without handle fields support owned inputs
and outputs as ordinary Kotlin values, without `close()`. Nested classes
may be declared in any order and in different packages; `.name(...)` changes the
Kotlin class name. Byte-array properties use content-based equality, hashing,
and display. Arrays remain mutable, and Kotlin's generated `copy()` is shallow.

```rust,ignore
.package(package!("model")
    .class(data_class!(Stamp))
    .class(data_class!(BlobValue))
    .fun(fun!(blob_value_echo)))
```

Explicit input/output expansions still override the default representation.
A data-class leaf inside an expansion remains one Kotlin value; an explicit
`.fields(...)` projection still delivers separate field outputs. The covertest
uses data classes for `BlobValue`, `Unsigned`, `Stamp`, and additional test records, including nested values,
selector alternatives, combined outputs, and package/class renaming.

Data classes currently require owned occurrences and at least one field.
Records containing handles, including nested ones, support returns only. Declare
member handle types with `ptr_class!`. Kotlin callers close each handle member;
the data class has no collective `close()`, and `copy()` shares handle objects.

Output ownership is explicit in the existing expansion hierarchy:

- `.field_self_ref()` borrows the value and all nested fields. Handle fields are
  cloned into independent owning handles and require Rust `Clone`.
- `.field_self_into()` moves the value and all nested fields without cloning.
- `.project_self(Projection::owned().field("details", Projection::borrowed()))`
  moves fields except the explicitly borrowed `details` subtree. Overrides can nest.

The same declaration works as a type-wide default or a per-function replacement:

```rust,ignore
.expand(expand_return!(ResourceBundle).field_self_into())
.package(package!("resources").fun(fun!(resource_bundle_new).expand_return(
    expand_return!(ResourceBundle).project_self(
        Projection::owned().field("details", Projection::borrowed())))))
```

`Projection` is available from `prebindgen_declarations` and `prebindgen_jni`.
Modes propagate recursively unless explicitly overridden; they never depend on
record contents. Legacy `field_self()` and unexpanded data-class outputs retain
a fixed borrowed policy. Use `field_self_into()` for non-cloneable handle fields.
Type-wide expansions of field types do not rewrite a data class's properties.

For separate field outputs, `.fields(fields!(borrowed_getter))` remains available.
`.fields_into(fields!(consuming_accessor))` requires an accessor taking `T`
and moves its returned record fields. Both support existing per-field expansion
overrides through `.field("name", expand_return!(FieldType)...)` on `FieldsDecl`.
These fields use their type-wide defaults when not overridden.
`fields_self_into(...)` remains available as a deprecated alias for `fields_into(...)`.

A borrowed occurrence cannot satisfy an owned projection. Unknown field overrides
and duplicate consuming views are rejected. Observing getters run before consuming
views regardless of declaration order. Rust checks `Clone` requirements and rejects
field moves from records implementing `Drop`. Native conversion or delivery failure
drops unpublished resources; Kotlin delivery failure closes transferred handles,
including nested members. Handle-containing records remain unsupported as inputs.

Reference fields, data-class aliases, recursive records and invalid Kotlin field
names are rejected with generation errors. Methods and direct
`jobject` input are not part of this increment. Field layouts use their declared
value types; type-wide expansions apply to boundary occurrences, not to the
properties inside a data class. Borrowed projections clone strings and byte
vectors; owned projections move them. The shared helpers `record_output` and
`record_output_owned` implement the respective access modes. Fixed-size arrays and
`Option` fields are supported. The low-level `JniCodec` does not know data-class declarations: it binds records
as handles. Using it in `configure()` for a type declared only as a data class
therefore fails with a missing handle-class error unless an explicit expansion
replaces that slot. Use data-class declarations for automatic value-record conversion.

C uses `Cbindgen::builder().source(...)`, `.expand(...)`, `.function(fun!(...))`,
`.prefix(...)` and `.opaque_ptr(name, type)`. Opaque declarations generate distinct
C pointer types and typed destructors. Its old naming and cbindgen postprocessing
settings remain commented out in the example manifests.

## Preparation and generation APIs

`prebindgen-flat` provides the source model and minimal conversion interfaces:

- `Flat<I, O>` stores `Option<I>` on parameters and `Option<O>` on results.
  Payloads belong to the consumer; source-only inspection can use `Flat<(), ()>`.
- `Conversion` exposes `wires()`,
  `code_fragment_slot(mode, expected_type)` and
  `code_fragment_wire(pos, mode, expected_type)`.
- `wires()` borrows a stable, ordered slice. Conditional wires stay in the list;
  `pos` indexes that list and an invalid position is an error.
- Each generator defines its slot and wire request enums. Variants carry only the
  data that operation needs. Associated request types have a lifetime parameter
  so they can borrow names, contexts or other configuration without cloning it.
  There is no mandatory context type or universal set of conversion phases.
- Successful requests return a Rust expression, including `()` for a no-op and
  block expressions for operations with statements. `expected_type` is the
  caller's expectation; implementations may use it to configure or validate the
  request. Only compiling the generated Rust establishes type correctness.
- `bind_fragment(target, expected_type, expression)` validates expression syntax
  and emits a typed `let` binding. It rejects empty or statement-only fragments.
- `WireKind` supplies each carrier's Rust type. The generator defines its carrier
  enum and request semantics, including activity or access questions when needed.
  Flat does not interpret resource identities, access modes or activity.

`prebindgen-conversions` provides optional `InputPlan<W, C>` / `OutputPlan<W, C>`
storage, template rendering, and a staged request protocol used by C and JNI.
The staged protocol is one implementation of the minimal traits, not a requirement
on other generators. Its requests cover validation, acquisition, construction and
output delivery. Requests introducing locals accept a continuation expression so
borrowed storage and acquired owners stay alive through the call.

The C and JNI adapters opt into `staged::emit_call`, which orders those requests,
checks allocation aliases and shared acknowledgement, and returns a fallible
expression. The protocol and its assembly helper live outside Flat; generators
with different requirements can compose their own request enums directly.
`StagedFunction::outputs()` requests each output wire's value with its carrier type.

Stored plans use `TemplateContext::render`. `template_parameter(name)` marks a
binding supplied by the generator, and `source_result_expr()` marks the source
return-value binding carried by output requests. Custom converters need neither
stored plans nor template support.

`prebindgen-declarations` prepares slots. Its `Bindings<W, C>` alias is
`Flat<InputPlan<W, C>, OutputPlan<W, C>>`. `Codec::Context` selects the context
type for the prepared slots; it defaults to `()` in the alias for
conversions that need no generator-specific settings. `bind_input` and `bind_output` compile one
occurrence; `Declarations::fill` compiles defaults and explicit overrides for a
function. `Codec::finish_input` and `finish_output` attach glue after recursive
composition, during preparation only; the latter also runs for unit results.
Failed compilation preserves existing slots. Existing converters survive
default filling; explicit overrides replace them. Recipes and `Codec::Extra` are
construction inputs, never stored in Flat. Nested recipes use the enclosing
occurrence's extra data.

`prebindgen-conversions` owns reusable leaf implementations: `value`, `boxed`,
`boxed::Storage` and `boxed::BoxedWire`. It depends on flat and does not depend
on declarations. Adapters use its builders during preparation and its metadata
during generation. Declarations uses conversions only in tests and examples;
its production compiler accepts adapter-supplied codecs.

`prebindgen-runtime` remains the execution-time support library. It acquires no
`syn` or code-generation dependencies from this separation.

### Preparation declarations and resolved field mapping

Representation choices belong to `prebindgen-declarations`. Its optional
`Representations<R>` registry holds shape declarations, nested references and
canonical defaults. Preparation clients validate these with `validate(&flat)`
and can check an occurrence's wires with `validate_mapping`. They copy any
selected adapter metadata into their own slot implementations. The standard
recipe compiler resolves fields, constructors, selectors, getters and combined
outputs directly, without creating a registry.

`Flat<I, O>` contains no representation registry, and conversion traits contain
no representation IDs. Generation never looks up declarations or defaults.
A `Wire` exposes only a binding name and carrier description. It does not expose
fields, constructor arguments, alternatives or ownership meaning. `ComponentPath`
and `Condition` belong to declarations. A codec can retain logical mappings in
its private preparation metadata through `prefix_component`; `validate_mapping`
accepts explicit name/path pairs. The final converter emits the corresponding
projections or constructions itself. Wrapper writers never interpret these paths.
`Function::input_conversions()` preserves slot grouping; the optional staged
protocol pairs output carriers with requested expressions.

The optional staged plans describe allocation participation using emitted Rust
boolean expressions. Their assembly helper checks allocation aliases. JNI slots
also emit Kotlin conversion, conditional locking, ownership updates and output
rollback through `KotlinInputConversion` and `KotlinOutputConversion`. The Kotlin
writer routes carriers and orders these operations without inspecting their meaning.

The preparation registry can be changed or dropped after slots are filled without
affecting generation. C/JNI public wrapper surfaces remain the same.

Unconfigured helpers may retain empty slots. Selected exported functions must have
all slots filled, including unit results. The current adapters additionally use
`validate_bindings` to reject partially configured functions.

A leaf is a source type the adapter converts itself rather than composing it
from parts. Recipes mark where the leaves are (`Input::Leaf`, `Output::Leaf`);
the codec chooses each leaf's conversion from its type. Its leaf methods are `input_leaf`, `output_leaf`, `consumes` and `selector`. `prebindgen-conversions` provides two
helpers for leaves:

- `prebindgen_conversions::value` copies values across the boundary. `Value::of`
  classifies primitives, text (`String`, `&String`, `&str`) and bytes
  (`Vec<u8>`, `&Vec<u8>`, `&[u8]`). The adapter supplies only the decode and
  encode expressions; the helper adapts them to owned and borrowed targets, so a
  `&str` target can borrow the caller's buffer without copying.
- `prebindgen_conversions::boxed` keeps objects in Rust-owned allocations.

Both adapters treat values as copies and every other leaf type as a boxed object,
so a type alias such as `type Key = String` crosses as an object.

A wire is one generated boundary parameter or result component. `Wire<W>`
stores the adapter's transport description in `kind`. `Wire<W>` requires
`W: WireKind`, whose `rust_type()` returns a `syn::Type` for the boundary carrier.
Both writers use this API to declare Rust parameters and result fields. A plain
`syn::Type` implements `WireKind` when no additional metadata is needed.
C uses `CWire` for scalars,
buffers and typed pointers. JNI's `JniWire` describes Rust/Kotlin carrier types and
JVM parameter width. Ownership and selector semantics remain private to the slots.
C and JNI retain distinct carrier types.

`Codec::Input` and `Codec::Output` select the final slot implementations. After
composition and finalization, preparation converts its plans into these types;
JNI uses this step to hide its preparation metadata inside `JniInput`/`JniOutput`.
Simple codecs may use `InputPlan`/`OutputPlan` directly. Declarations compose leaves, fields, constructors,
input selectors and combined outputs. The composition model requires neither
handles nor boxed storage. It has no object registry or universal `Send`
requirement.

The adapter declares whether a leaf output consumes its value; values do not.
Declarations run observations before that move and reject double consumption. For
pointer/handle output from borrowed record fields, these adapters make
independent owned copies using Rust `Clone`. A standalone owned text/byte result
moves its buffer; an observed field is copied.

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
Both current adapters opt into `prebindgen_conversions::boxed`, which shares pointer
validation, borrow/move acquisition, cloned observations and output commit.
C stores objects directly in `Box<T>`; its header exposes typed opaque pointers.
Other adapters can choose different storage in their leaf conversions; there is
no universal `Object` operation or required allocation policy.
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

Callbacks, enums, general collections, mutable text/byte views, methods,
`convert!`, field renaming, local function declarations, domain errors, split
overloads and shared selector argument storage remain outside this subset.

## Optional values and niches

Both adapters compose `Option<T>` around the existing conversions for `T`,
including type-wide expansions and explicit borrowed/owned output projections.
Occurrence overrides may name either the whole `Option<T>` or its payload `T`;
payload overrides are lifted through the optional layers.
Absent inputs skip payload validation, resource checks, acquisition and construction.
Absent outputs skip payload conversion. Prepared output allocations remain owned
by Rust until delivery succeeds, and Kotlin rollback closes only present handles.
Borrowed text, byte slices and handle inputs are supported inside `Option`;
shared-reference outputs copy values or clone handles into independently owned outputs.

A **niche** is a wire representation unavailable to a present payload. The shared
compiler uses an eligible niche instead of adding a presence wire:

| Payload | C absence | JNI absence |
|---|---|---|
| Opaque pointer / handle | Null pointer | Zero handle |
| `char` | `0x110000` | `-1` |
| `u8`, `u16`, `u32` | Separate presence flag | `-1` in the wider signed carrier |
| Other scalars, text, bytes | Separate presence flag | Separate presence flag |

A presence flag is an `i32` with exactly `0` (absent) or `1` (present). Other
active flags are errors. C callers supply the documented sentinel for absence;
Kotlin callers supply `null`, and scalar sentinel values supplied as non-null
arguments are rejected. Buffers have no null niche: a null pointer with zero
length is a valid empty buffer. No valid scalar value is reserved as a sentinel.

Only unconditional payload wires can donate a niche. Selector-arm wires and
already optional wires cannot donate theirs to an outer option. Additional
optional layers therefore use presence flags, preserving every nested state.
C headers document the exact discriminator argument or result field for each
optional group, including enclosing optional groups.

Adapter authors expose valid niches through `Codec::niche`; `absent_value` supplies
inert carriers, and `mark_optional` retains foreign grouping while consuming the
available niches. Custom codecs must suppress conditional niches during composition.
The default absence/grouping hooks reject options until the codec implements them.

Kotlin exposes a single converted value as `T?`, including nullable data classes
and their optional fields. Expanded payloads with multiple parameters retain an
explicit Boolean presence parameter; nested options and unit similarly retain
explicit presence where Kotlin nullability alone would lose states. Multiple
output views use the existing builder lambda, with nullable payload values and
an outer Boolean for nested options. For example, `Option<Option<i64>>` returns
`(present: Boolean, value: Long?)`: `(false, null)`, `(true, null)` and `(true, n)`
are distinct. Data-class declarations reject directly nested options; expose
those through explicit field expansions instead. Recursive records and
handle-containing data-class inputs remain unsupported. The C ABI exposes all selected wires directly.

## API documentation

Run `cargo doc --workspace --no-deps --open` to browse the API. Start with
`prebindgen_flat::Flat` for occurrence slots, `prebindgen_declarations::Declarations`
for build-script expansions, and `prebindgen_declarations::Codec` for adapter authors.
The Flat traits describe expression requests; the optional staged protocol in
`prebindgen-conversions` explains validation, ownership acquisition and output delivery; `Cbindgen` and `JniGen` document configuration and writing files.

Runnable rustdoc examples cover occurrence configuration, selector/combined-output
declarations, a codec with Rust types as its wire data, C and JNI generation, boxed borrows,
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
