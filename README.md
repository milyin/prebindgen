# prebindgen: occurrence conversion slots

prebindgen generates C and JNI/Kotlin bindings from annotated Rust source.
The flat model describes source signatures and types. Build-script declarations
configure conversions for each parameter and result; the declaration compiler builds the
conversions, and language adapters emit wrappers and foreign declarations.

```text
#[prebindgen] source + build.rs declarations
        ↓ select declared functions, constructors, getters and referenced types
Flat<(), (), M> with source declarations and metadata
        ↓ resolve language representations, defaults and explicit choices
Flat<Input, Output, M> with resolved occurrence descriptions
        ↓ compile explicit operations with the language codec
Flat<I: Conversion, O: Conversion, M>
        (parameter slots: Option<I>; result slots: Option<O>)
        ↓ C or JNI wrapper writer
Rust wrappers + C header / Kotlin files
```

A conversion belongs to an occurrence, so two parameters of the same Rust type
can have different input recipes. `Flat<I, O>` stores the two payload types in
optional slots. Both can implement `Conversion` with distinct wire kinds and request enums; the model itself does
not require that trait. Record declarations preserve Rust source structure; typed
metadata on Flat owners records language representations and project choices.
Flat also preserves source names, locations and guards.
Wrapper writers use flat's generation API; recursive recipe resolution belongs to declarations.

## Examples migrated from main

This branch replaces the generator engine while preserving source capture and
source-reading support. Class/package declarations keep their syntax; conversion declarations now accept
direct `Input`/`Output` trees through `.input(...)` and `.output(...)`. Import shared declaration types and macros directly from
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
parsed `Flat` model can also include them. Unbound data-carrying enums, callbacks,
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

JniGen supports `source(...)` or `items(...)`, `package!`, `ptr_class!`, `data_class!`, `enum_class!`, `fun!`,
`.name()`, package prefix and native initialization. `build()` returns `JniGen`;
`write_rust` resolves relative paths under `OUT_DIR` and returns the written path;
`write_kotlin` returns the written paths. Settings may follow declarations.

The declaration layer also supports `expand_param!` constructor/identity variants,
`expand_return!` getter/identity outputs, `.view(fields!(...))` with per-field
output overrides, and per-function overrides. These build converters directly
in slots before generating wrappers. The migrated C and JNI examples exercise
Report constructor/handle input variants and `.view(fields!(report_to_struct)).field_self()` output delivery.
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
`.view(...)` projection still delivers separate field outputs. The covertest
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
.output(expand_return!(ResourceBundle).field_self_into())
.package(package!("resources").fun(fun!(resource_bundle_new).output(
    expand_return!(ResourceBundle).project_self(
        Projection::owned().field("details", Projection::borrowed())))))
```

`Projection` is available from `prebindgen_declarations` and `prebindgen_jni`.
Modes propagate recursively unless explicitly overridden; they never depend on
record contents. Legacy `field_self()` and unexpanded data-class outputs retain
a fixed borrowed policy. Use `field_self_into()` for non-cloneable handle fields.
Type-wide expansions of field types do not rewrite a data class's properties.

For separate field outputs, `.view(fields!(borrowed_getter))` remains available.
`.fields_into(fields!(consuming_accessor))` requires an accessor taking `T`
and moves its returned record fields. Both support existing per-field expansion
overrides through `.field_override("name", expand_return!(FieldType)...)` on `fields!(getter)`.
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
`record_output_owned` implement the respective access modes. `Option` fields are
supported; fixed-size arrays remain unsupported. The low-level `JniCodec` does not know data-class declarations: it binds records
as handles. Using it in `configure()` for a type declared only as a data class
therefore fails with a missing handle-class error unless an explicit expansion
replaces that slot. Use data-class declarations for automatic value-record conversion.

C uses `Cbindgen::builder().source(...)`, `.input(...)` / `.output(...)`, `.function(fun!(...))`,
`.prefix(...)` and `.opaque_ptr(name, type)`. Opaque declarations generate distinct
C pointer types and typed destructors. Its old naming and cbindgen postprocessing
settings remain commented out in the example manifests.

## Preparation and generation APIs

Builders prepare one binding model, then emit from that model:

```text
Source -> Flat<(), (), language metadata>
       -> Flat<Input, Output, language metadata>   (resolve descriptions)
       -> Flat<input conversion, output conversion, language metadata> (compile)
       -> generated Rust and C/Kotlin              (emit)
```

`Input` and `Output` describe requested conversions directly. Type defaults use
`Input::for_type` / `Output::for_type` (or `expand_param!` / `expand_return!`);
function `.input(name, tree)` / `.output(tree)` overrides replace them.
Resolution applies project choices and language representations, resolves source
constructor/getter references, and expands optional, sequence and record operations.
Compilation lowers those explicit operations into conversion slots. Sequence
returns carry an explicit `SequenceDelivery::Value` or `Callback` choice; codecs
receive that choice rather than inferring it from generated wire names. Both stages
read source facts from the same `Flat`; the instruction trees reference declarations
rather than copying records or signatures.

`Cbindgen::prepare()` and `JniGenBuilder::prepare()` return the complete prepared
`Flat`. The public `prebindgen_c::generate(&flat)` and
`prebindgen_jni::generate(&flat)` read all settings, type declarations, function
placement and conversions from that model. `build()` combines these steps.
Temporary indexes used during preparation are discarded; emission needs no builder
or separate configuration table.

`prebindgen-flat` provides the source model and minimal conversion interfaces:

- `Flat<I, O, M>` owns source declarations and binding metadata. `M: Metadata`
  selects distinct payload types for the module, functions, types, fields and
  enum variants; `M = ()` supports source-only inspection. Parameters hold
  `Option<I>` and returns hold `Option<O>` for occurrence-specific conversions.
  `map_slots` moves the source declarations and metadata into the next stage.
- `Conversion::code_fragment_slot(mode, expected_type)` accepts the generator's
  own requests, including any element or branch context.
- `InputConversion::input_node()` describes one input value.
  `OutputConversion::output_outcomes()` describes success and the optional source
  error and binding failure channels. Each channel contains its own value tree.
  Neither interface prescribes ABI packing or delivery.
- `WireNode<W, C, B>` describes structure: `Leaf(Vec<Wire<W>>)` is a terminal
  bundle, `Chain` has one boxed `child` and metadata `C`, and `Branch` has an
  ordered `children` vector and metadata `B`. Both parent shapes own
  `controls: Vec<Wire<W>>`. Adapter metadata determines meaning, such as an
  optional payload, repeated element, record fields or mutually exclusive arms.
  The shared tree does not prescribe runtime occurrence counts or delivery.
- `wire_definitions(std::iter::once(input.input_node()))` explicitly traverses
  all wire definitions, visiting node controls, a sequence's element schema
  and every alternative once. Select an output channel before traversing its tree.
  `wire_definitions_mut` edits definitions without cloning metadata.
  `Function::input_wire_definitions()` visits all input definitions.
  `wire_definitions_with_paths` includes structural `WirePath` addresses;
  `wire_at_path` resolves a definition. These operations accept filtered or
  chained root iterators. Paths are relative to that selection and must be
  rebuilt after structural edits or changing the selected roots.
- Each generator defines its request enum. Requests can borrow names, settings,
  paths and currently bound elements. There is no mandatory context, phase order
  or physical per-wire request protocol in the common trait.
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
C/JNI use the optional staged conversion protocol from `prebindgen-conversions`.
`InputPlan::node` stores one authoritative input tree.
`OutputPlan::outcomes` stores `Outcomes<WireNode<W, C, B>>`.
All definitions occur in terminal bundles or the `controls` of chain and branch
nodes. Metadata retains encoding decisions and references to
those definitions; it owns no wire descriptors or copies of child trees.

Each adapter has operations for actual ABI tasks: building a function's
parameters, laying out a successful result packet, defining one sequence row,
and choosing a visitor callback's arguments. Those operations inspect the tree
and their adapter's metadata. They can share private traversal code; the common
API imposes no physical parameter list or flattening policy.

`OutputPlan::values` holds `(binding_name, expression)` pairs. The adapter reads
its destination fields in ABI order and looks up each expression by name.
`match_output_fields` checks those assignments and rejects missing, extra or
duplicate bindings. Expression-list order therefore cannot silently change an
output packet or sequence row. The tree remains the source of field types and
placement; the expressions contain no second layout.

Sequence input hooks similarly provide named validation and deferred reads for
one element. Shared composition substitutes those names directly, without
flattening the element tree. Output hooks provide metadata and named outer
expressions, allowing one buffer value or separate pointer and length values.
The adapter validates each prepared child before embedding it in a sequence.

Option composition asks the codec to select a `NicheReservation`, test presence,
prepare named absent outputs, and retire niches from inactive payload values.
The reservation identifies a payload binding and its excluded representation.
The adapter chooses its candidates, including tree-owned controls, while
shared composition handles conditional validation, preparation and ownership.
`prefix_layout` lets a codec attach source paths without exposing its physical
values to the common compiler. `validate_input` and `validate_output` check the
finished conversion before installation.

C/JNI select `success` when building normal result packets. The `source_error`
and `binding_failure` payloads remain accessible through their named fields.
Callback/context parameters, acknowledgements and error packet/status details
belong to the adapters. The base `Conversion` trait requires only the logical
tree and the generator's slot requests.

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
during generation. The declaration compiler uses these shared conversion helpers and
adapter-supplied codecs.

`prebindgen-runtime` remains the execution-time support library. It acquires no
`syn` or code-generation dependencies from this separation.

### Preparation declarations and resolved field mapping

Conversion choices are direct `Input` and `Output` descriptions. Resolution
checks source fields, constructor/getter signatures and type defaults against
`Flat`, then stores explicit operations in occurrence slots. Language declaration
metadata belongs to the corresponding Flat type, function, field or variant.
Compilation does not consult a separate representation registry. Custom `Codec`
implementations select their metadata schema through `type Metadata = ...`; use
`()` when no declaration metadata is required.

A `Wire` exposes only a binding name and carrier description. It does not expose
fields, constructor arguments, alternatives or ownership meaning. `ComponentPath`
and `Condition` belong to declarations. A codec can retain logical mappings in
its private preparation metadata through `prefix_layout`. The final converter emits the corresponding
projections or constructions itself. Wrapper writers never interpret these paths.
`Function::input_conversions()` preserves slot grouping; the adapters match named prepared output expressions to their destination fields.

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
stores the adapter's description of that individual wire in `kind`. `Wire<W>` requires
`W: WireKind`, whose `rust_type()` returns a `syn::Type` for the boundary carrier.
Both writers use this API to declare Rust parameters and result fields. A plain
`syn::Type` implements `WireKind` when no additional metadata is needed.
C and JNI each use a single wire enum: `CWire` for scalars, enums, buffers and
pointers; `JniWire` for scalars, enums, handles, text, bytes and selectors.
Scalar and enum variants store `next_integer_niche`, as do JNI selectors.
C pointers and JNI handles, text and bytes store `null_niche_available`.
`CWire::{scalar, enumeration, pointer}` and
`JniWire::{scalar, handle, text, bytes, selector}` initialize that state;
`CWire::Buffer` is constructed directly. `available_niche()` reads the stored
state, `set_niche(...)` checks updates for variant compatibility, and
`clear_niche()` disables availability. Callers must not restore a consumed niche.
C buffers have no niche: a null data pointer with zero length is a valid empty
value. Reserved representations remain separate `CNiche` or `JniNiche` values on
optional-node metadata. Each adapter derives boundary types from its wire
variant; JNI also derives Kotlin types and JVM parameter width.
`JniWire` also exposes `jvm_descriptor()`, `is_reference()` and
`kotlin_default()` for its raw JVM representation. Its code-generation methods
`read_array_element()`, `output_object()` and `output_argument()` let ordinary
returns, collection rows and callback delivery share the same field conversion.
Reads produce native input values; output methods take prepared
`OutputPlan::values` (for example `Option<String>`, encoded wide-integer bytes,
or an allocation pointer), rather than JNI input references. They use the
`environment` template parameter and leave local-reference lifetimes and
ownership commit to their caller. `output_argument()` returns `JniArgument`:
emit its preparation statements before using its `jvalue` expression, in the
same local-reference frame.
All leaf information is accessible through
`wire.kind`; there is no second leaf-metadata type. C and JNI retain distinct
carrier types.

`Codec::Input` and `Codec::Output` select the final slot implementations. After
composition and finalization, preparation converts its plans into these types;
JNI keeps the logical specification and selected native plan in
`JniInput`/`JniOutput`; Rust and Kotlin emission share the native plan.
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

C functions without `Result` return their first boundary value directly; any
additional values use output pointers. Unit functions and destructors return
`void`. Text and bytes use pointer/length buffers; `&str` and `&[u8]` inputs borrow
that memory, while owned inputs copy it. Callers free returned buffers exactly once.

A non-Result function with a fallible binding conversion must be explicitly
declared with `.function(fun!(name)).panic()`. Without that declaration, generation
fails. Invalid inputs or conversion failures then abort at the C boundary; source
and destructor panics also abort. Use `Result<T, E>` when the caller needs to
recover from an error. Error messages use the explicit Result error output.

**JNI owns handle semantics.** As on `main`, a Kotlin object owns a boxed Rust
allocation whose address travels in a `Long`. A low pointer bit records closed
or consumed state. A JNI-specific wrapper guarantees the alignment needed for
that bit. Native objects require `Send` because Kotlin can transfer them between
threads; this restriction belongs to JNI.
Generated wrappers lock all active handle objects in a stable address order,
including handles nested in selected constructor arguments. Close uses the same
handle lock. Repeated close and close after consumption are harmless.

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
Further JNI object-array ABI optimization remains separate work.

Source callback parameters, data-carrying enums, maps/sets, mutable text/byte views, methods,
`convert!`, field renaming, local function declarations, split
overloads and shared selector argument storage remain outside this subset.

## Collections

`Vec<T>`, `&Vec<T>` and `&[T]` use the element type's ordinary input and
output declarations. They can contain scalars, strings, byte arrays, fieldless
enums, options, records and owned handles; owned vectors can be nested. Options
around a collection distinguish absence from an empty collection, and
`Result<Vec<T>, E>` uses the declared typed error projection or message mapper.
`Vec<u8>` and byte slices keep their existing byte-buffer transport.

Ownership remains explicit. An owned vector return moves its elements; a
borrowed vector or slice return uses borrowed element projections. An explicit
`expand_param!(Element)` or `expand_return!(Element)` on a collection occurrence
configures each element using the same constructor/getter rules as a scalar
occurrence. Borrowed collection inputs cannot consume handles: declare a
constructor taking borrowed inputs when the source elements need cloning.
Reference elements, mutable collection views, zero-wire elements, boxed slices,
maps, sets and nested `Result` values are rejected.

C receives one typed `{ data, len }` carrier. Each row contains the element's
ordinary boundary fields, with generated row definitions and option/field
comments in the header. Input storage remains caller-owned and must be readable
for the call. Output row storage belongs to the caller after success. The
matching `..._rows_free` releases **only the row allocation**: first free or
transfer owned handles, text buffers and nested row buffers using their matching
destructors. Do not pass stack storage or a different row schema to that function.
This permits moving element handles back into Rust before freeing their old row
storage. Failed output preparation reclaims unpublished elements automatically.
A structured error's generated destructor releases its collection fields recursively,
including owned elements and nested row allocations.

JNI exposes `List<T>` (nullable for optional collections) and transports rows
through object arrays. Each element
must expose exactly one Kotlin input/output value; declare a data class to wrap
multiple fields. General collection fields inside data classes are not yet
supported. Inputs snapshot list structure once before choosing locks, then read
handle addresses while all active handles are locked. Handle locking uses an
iterative acquisition loop so list length does not add JVM stack frames. JNI
validates row shapes and active field types, using local reference frames per
row. Native failure before ownership acknowledgement leaves input handles live;
after acknowledgement Kotlin marks every selected handle consumed. Prepared
outputs are reclaimed on failure. These row arrays and boxed JVM scalars are a
correctness-oriented transport, with no zero-copy or throughput claim.

C/JNI interpret a `Chain` according to adapter-owned metadata: `Optional`
means zero or one payload occurrence and `Sequence` means zero or more element
occurrences. Both have exactly one `child` description, irrespective of the
runtime number of values. Their `Branch` metadata distinguishes related
components (`Group`) from selected arms (`Alternatives`). For example:

```text
Chain(meta: Optional)
  child:
    Chain(meta: Sequence)
      child:
        Chain(meta: Optional)
          child: Leaf(integer)
```

`wire_definitions(nodes.iter())` visits the integer definition once, together with
all presence flags and sequence controls declared on the enclosing nodes. It describes neither a concrete
runtime collection nor a native parameter list. The C adapter can choose a row buffer
and presence controls; another adapter can deliver each integer through callback
arguments. The same logical structure supports either choice. Fields nested in
sequences have structural paths, rather than positions assigned to an enclosing
function signature.

C/JNI store their selected encoding in metadata on these same nodes. A sequence
node owns its element schema and its row-buffer or callback-completion controls.
An optional node similarly owns its presence controls and niche metadata and
any extra flag. Native parameter lists and row/callback field lists are borrowed
views of that tree, with explicit scope boundaries chosen by the adapter. These
adapters retain their existing ABI and conversion behavior.
Custom codecs can implement `sequence_input` and `sequence_output` to reuse the
shared validation, acquisition, construction and output-ownership phases.

### Callback delivery of sequence returns

Select `.callback_return()` on a function to keep its ordinary return and add
callback delivery. C emits `<function>_each`; Kotlin emits `<name>Fold`.
Use `.callback_return_only()` to expose only the callback variant. That explicit
choice also supports decomposed Kotlin elements with several values, without
requiring a data class. Both variants use the same declared element projections, including getters that
project a sequence from another source type. Presence follows the output tree. Callback variants are
compiled from declarations: a manually prebound output slot needs an explicit
`expand_return` override. Return projections that also expose unrelated values
outside the sequence are rejected.

```rust
// C: a null visitor is a binding error, so a non-Result function needs .panic().
.function(fun!(numbers).callback_return()).panic()
// JNI package declaration:
.fun(fun!(numbers).callback_return())
```

For `fn numbers() -> Vec<i64>`, C receives:

```c
uintptr_t pb_numbers_each(void (*callback)(void *context, int64_t v0), void *context);
```

The visitor runs once per element; the return value is the delivered count.
Each invocation transfers its owned element fields to the visitor. Free or
retain those fields explicitly. Earlier deliveries remain caller-owned if a
later conversion fails. Result functions still use explicit error outputs and
leave success-output storage unchanged on failure; consuming inputs retain their
ownership acknowledgement. During a C visitor, that acknowledgement output has
not yet been written: moved inputs already belong to Rust and must not be used.
A null visitor is rejected before source invocation
or input ownership transfer. Borrowed inputs must remain live throughout the
call, and C visitors must not unwind or longjmp.

Kotlin can fold into any accumulator:

```kotlin
val sum = numbersFold(0L) { acc, number -> acc + number }
```

Rust calls a generated JNI bridge once per element, passing primitive fields as
JVM primitives and object fields as references. The method is resolved once per
native invocation. The bridge retains raw rows in Kotlin; Rust keeps element
owners until the complete native delivery succeeds. It does not allocate a
JNI array containing all element rows or box primitive callback arguments in Rust. User folders run
after JNI returns, input ownership is acknowledged and handle locks are released.
This prevents user re-entry while Rust borrows are live. It deliberately buffers
the result before running user code; it does not provide constant-memory streaming
or early termination. The existing list-return variant remains available.

A folder receives the accumulator followed by each element's projected Kotlin
values. It owns handles from successfully completed steps. If a folder throws,
the current element and all remaining undelivered elements are cleaned up, and
the same exception is rethrown. Handles retained from earlier successful steps
remain caller-owned; `take()` can retain a handle from the failing step.
`Option<Vec<T>>` returns null without invoking the folder for None; Some(empty)
returns the initial accumulator. With a nullable accumulator type, a null final
accumulator and None both return null. Result errors use callbacks returning the same
accumulator type, and do not invoke the folder.

Sequence-valued arguments of source callbacks remain part of future general
callback support. Byte-buffer returns keep their existing transport and are not
selected as element-wise sequences by these declarations.

## Result returns

Declare how to project source errors explicitly, using typed output expansions
or a message mapper. A mapper takes exactly `E` or `&E` and returns `String`;
ownership follows that signature. No error fields or `Display` implementation
are inferred. For example, for a source function
`open_session(...) -> Result<Session, Error>` and helper
`error_message(error: &Error) -> String`:

```rust,ignore
// C builder: return a caller-owned message through char **error.
.opaque_error(syn::parse_quote!(Error), "error_message")
.function(fun!(open_session))

// JNI builder: project typed error fields, just as for ordinary outputs.
.output(expand_return!(Error).fields_into(fields!(error_into_parts)))
.package(package!().fun(fun!(open_session)))
```

A function's `.on_error("another_message")` overrides the type-wide mapper.
Mapper helpers are captured automatically but are not exported unless separately
declared. Return expansions and their defaults apply to `T`, the success type.
The success value can be an optional value, enum, record, or owned handle, using
the same conversions as an ordinary return. `Err` skips all success-output
preparation and delivery. Source errors and unpublished success values are
released through ordinary Rust ownership.

C follows the explicit error-output convention. `.opaque_error(type, mapper)`
(or `.error_mapper(type, mapper)`) delivers an allocated C string through
`char **error`; release it with `<prefix>_free_error`. Alternatively,
`.error("error_name", type)` declares a record error, whose fields use ordinary
output declarations. C receives `<prefix>_error_name *error`; release its owned
fields with `<prefix>_drop_error_name`. Structured source errors preserve their
fields, and binding failures construct the same error type through `From<String>`.
If converting a source error fails before delivery, C reports that failure through
`From<String>`. If constructing or converting this replacement also fails, C
initializes the error record to zero, with no owned allocations, so its generated
destructor remains safe. An error output can be NULL to discard details.
Success leaves error storage unchanged; release any prior error before reusing
that storage.

For Result calls, a success wire with an unused niche is returned directly, with
that niche indicating failure (for example NULL for a handle). If no success
wire has a niche, the function returns `bool` and writes success values through
output parameters. Those outputs remain unchanged on failure. For example:

```c
int64_t value;
char *error = NULL;
if (!demo_read(&value, &error)) {
    fprintf(stderr, "%s\n", error);
    demo_free_error(error);
}
```

Calls that consume handles additionally require `bool *ownership`: false means
validation failed before transfer; true means the active handles were consumed,
including when the source function or a later conversion fails. All success,
ownership and error output storage must be pairwise disjoint. This explicit
flag reports whether Rust accepted ownership before returning a Result failure.
Non-Result calls need no ownership flag because failures abort the process.

JNI follows the production adapter's callback model. `Err(E)` is converted
through the ordinary output declarations for `E`, including `expand_return!`,
data/enum classes and explicit borrowed/owned projections. The generated
`<Error>Handler<R>.run(...)` receives these typed values. A message mapper is an
explicit alternative that generates `<Error>MessageHandler<R>` with one String
parameter. Neither representation is selected from the error's field contents.

A fallible Kotlin function takes `onError` for domain errors and
`onBindingError: JniErrorHandler<ReturnType>` for conversion failures, closed
handles and caught Rust panics. An infallible function has just the binding
callback, named `onError`. A handler may return a fallback value of the function's
return type or throw. Both handlers execute after the native call returns and
after handle locks are released. Owned handles passed to a domain handler are
closed after it finishes, including when it throws; use `handle.take()` to retain
an owning handle beyond the callback. Native input ownership acknowledgements
are applied before invoking a handler.

Binding handlers default to throwing `IllegalArgumentException` in Kotlin.
Domain handlers are required unless `.error_class(E, "java.io.IOException")`
explicitly supplies a throwing default for a message-mapped error. That convenience
also emits `@Throws` for Java callers; it does not throw the source exception
inside JNI. Callers can replace either default with their own callback. Existing
pending JVM exceptions, such as allocation failures, propagate unchanged.
An error projection or mapper that fails is a binding failure.

`Result<(), E>` returns a C boolean without a dummy success output and Kotlin
`Unit`. Error routing supports only an outermost owned
`Result<T, E>` return. Result inputs, Result-valued fields, borrowed Results,
nested Results inside the success value, and fallible helpers used as expansion constructors or getters are
not yet supported. Spell both generic arguments explicitly using `Result`,
`std::result::Result`, or `core::result::Result`; one-argument aliases such as
`io::Result<T>` are not resolved. Named opaque aliases remain opaque.

Adapter authors can use the shared `Output::Result` recipe through `bind_output`.
The declaration compiler validates any selected mapper and unwraps the result
before preparing the success conversion. `Codec::result_error_value` receives
the owned error and an optional mapper expression; its default requires a mapper
and delegates to `Codec::result_error`. C can preserve declared error fields;
JNI preserves typed outputs or explicitly mapped messages for its domain callback. No Result-specific wire type, niche, or group metadata is introduced.

## Fieldless enums

Declare an enum explicitly in each adapter:

```rust,ignore
// JNI/Kotlin
.package(package!("model").class(enum_class!(Priority)))
// C
.enum_type("priority", syn::parse_quote!(Priority))
```

Both adapters currently use a signed 32-bit integer representation. They preserve
Rust discriminants, including negative values, gaps and implicit successors.
Kotlin emits an `enum class` with a `wireValue` property and checked `fromWire`
factory; ordinal positions never define the binding. Infrastructure members are
renamed with leading underscores if a source variant uses the same name.
Kotlin's built-in entry names `name`, `ordinal`, `entries`, and the generated
code's package reference `kotlin` require an explicit
override, for example `enum_class!(Code).variant_name("name", "Name")`.
Unknown overrides, duplicate overrides and duplicate foreign entry names fail
at generation time. Source variants are never renamed implicitly.
C emits an `int32_t` typedef and named constants, for example
`pb_priority_High`. The C compiler's native enum layout is not part of the ABI.

`prebindgen-flat` retains variant names and source discriminant expressions.
`prebindgen-declarations::EnumMapping` resolves the integer mapping and generates
exhaustive output matches and checked input matches for both adapters. Generated
Rust assertions compare the mapping with the actual source discriminants. No
integer is transmuted into an enum. Rust `repr` attributes do not determine the
foreign carrier. No `Copy` or `Clone` implementation is required, and shared
references are converted by value. Mutable enum references are unsupported.

Enums work as parameters, results, fields of Kotlin data classes, and fields in
explicit C constructor/projection expansions. `Option<Enum>` and nested options
reserve unused integer representations, starting below zero and skipping actual
variant values. For an enum with variants `-2`, `-1` and `0`, the first two
absence representations are `-3` and `-4`. Unknown active inputs are rejected;
reserved values are accepted only by the corresponding optional layers.

This increment supports exhaustive, nongeneric enums with unconditional
fieldless variants. Discriminants must be integer literals (optionally negative
or parenthesized) or implicit successors, all within `i32`; expressions involving
constants or arithmetic are diagnosed during generation. Enums with payloads,
`cfg`/`cfg_attr`-decorated enums or variants and `#[non_exhaustive]` remain unsupported. The existing
nonzero-size rule still applies; a single-variant enum can use `#[repr(i32)]`.
Enums implementing `Drop` are unsupported: Rust rejects the generated source
checks that cast variant discriminants to integers.

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
| Fieldless enum | Unused integer | Unused integer |
| `u8`, `u16`, `u32` | Separate presence flag | `-1` in the wider signed carrier |
| Text, bytes | Separate presence flag | Null Java reference |
| Selector tag | Separate presence flag | `-1` (valid arm indices are nonnegative) |
| Other scalars | Separate presence flag | Separate presence flag |

A presence flag is an `i32` with exactly `0` (absent) or `1` (present). Other
active flags are errors. C callers supply the documented sentinel for absence;
Kotlin callers supply `null`, and scalar sentinel values supplied as non-null
arguments are rejected. C buffers have no null niche: a null pointer with zero
length is a valid empty buffer. JNI strings and byte arrays use null for absence;
empty strings and arrays remain non-null present values. No valid scalar value
is reserved as a sentinel.

Only unconditional payload wires can donate a niche. Selector-arm wires cannot
supply niches. Optional plans can retain unused representations for outer layers:
C encodes `Option<Option<char>>` in one `uint32_t` (`0x110001` for outer `None`,
`0x110000` for `Some(None)`). JNI uses `-2` and `-1` for nested optional chars and
widened unsigned scalars. Once a domain is exhausted, additional layers use flags.
A JNI handle, string or byte array offers null once. Once an inner option uses
it, an enclosing option adds a flag. JNI selector tags and newly added presence
flags offer negative values in sequence, so further enclosing options can reuse
the tag instead of adding more flags. For example, `Option<Option<Option<String>>>`
uses one string reference and one tag: tag `-1` means outer `None`, tag `0` means
`Some(None)`, and tag `1` allows the reference to distinguish `Some(Some(None))`
from `Some(Some(Some(text)))`. Selector-arm payload wires remain ineligible;
the selector tag itself is unconditional. Other wires in an optional payload
cannot supply another niche, because an absent payload fills them with defaults.
C headers document the exact discriminator argument or result field for each
optional value, including enclosing optional values.

Adapters define `Codec::Niche: Clone`; the shared compiler never interprets its
representation. `niche` selects an excluded representation, `is_present` emits a
total, side-effect-free presence predicate, and `absent_value` emits its absent
value. `remaining_niche` reports the next unused representation after consuming
one niche, or `None` when exhausted. The shared compiler suppresses niches on
other payload wires and on inactive selector arms. Both input and output
composition use this protocol.

Each logical `Wire<W>` defines one typed value. `WireNode<W, C, B>` arranges
those definitions into leaves, parents with one child and parents with ordered
children. `C` and `B` are adapter metadata types for the two parent shapes;
the base API does not interpret them. C uses `CChain` and `CBranch`; JNI uses
`JniChain` and `JniBranch`. Optional and sequence policies belong to chain
metadata; grouping and alternative policies belong to branch metadata.
An empty leaf carries no wires. An empty branch can preserve a field collection
or zero-argument constructor; an adapter rejects it when its metadata instead
means a choice between alternatives. The shared tree imposes no such rule.
`Outcomes` surrounds output value trees and cannot be nested inside them.
For example, using the C/JNI metadata vocabulary:

```text
Outcomes — exactly one channel is selected
  success:
    Chain(meta: Optional)
      child:
        Chain(meta: Sequence)
          child: Leaf(integer, text)
  source_error:
    Leaf(code, details)
  binding_failure:
    Leaf(message)
```

The `source_error` field contains the declared projection of `E` in
`Result<T, E>`. Converting the error into a diagnostic message requires an
explicit mapper; string-valued fields retain their ordinary conversions.
`binding_failure` is a separate conversion/invocation failure channel. C and JNI
declare their diagnostic payload through `Codec::binding_failure_layout`.
Adapters choose how to deliver each channel: native outputs/status, typed error
callbacks, or another policy. JNI always uses a binding-error callback and requires
that channel in manually supplied schemas. C requires it for recoverable `Result`
wrappers; other C functions abort on binding failures and may omit the channel
in manual schemas. Standard C codec plans describe the diagnostic message in both
cases; the C writer chooses whether to deliver it or abort. Empty channels retain their identity, including
`Result<(), E>` and `Result<T, ()>`.

`Outcomes<N>` has a required `success: N` and two optional fields,
`source_error: Option<N>` and `binding_failure: Option<N>`. Here `N` is a
`WireNode<W, C, B>`. Read or update the channel fields directly.
`None` omits an error channel; `Some(WireNode::Leaf(vec![]))` declares an error
with no payload. The type cannot omit success, duplicate a channel or insert
completion channels into an input, sequence element or record field.

Choose a channel before traversing:
`wire_definitions_with_paths(std::iter::once(&output.output_outcomes().success))`.
Paths describe only the chosen value tree, including the terminal bundle index;
they contain no outcome discriminator. Equal wire names in different alternatives
are allowed. Runtime sequence elements and the selected output channel are
context supplied by the adapter.

Metadata types `C` and `B` describe chain and branch parents. Each adapter
chooses their semantic variants. `W` describes each individual value. Use `()` for metadata a generator does not need. The shared compiler
creates metadata through codec hooks and does not interpret it. `group_metadata`
describes ordered conversion components; `field_group` and `fields_group` retain
source field information. Constructor arguments and output combinations retain
their group with one component; constructors without arguments retain an empty
group. `optional_metadata` and `alternatives_metadata` borrow supporting wires
for validation; the compiler moves those wires into `controls`. Sequence hooks
return `SequenceInput<W, C>` or `SequenceOutput<W, C>`, with separate `controls`
and metadata fields. Metadata must not retain wire descriptors.

- `CSequence` / `JniSequence` are `Buffered` or `Callback` delivery policies.
  The sequence node's controls contain the actual row buffer, delivered-element
  count or JNI presence marker. Callback signatures, cleanup and Kotlin folds
  read the policy and controls from that node.
- `COptional` / `JniOptional` are `Flag(binding)` or `Niche`. A flag references
  one tree-owned integer control; a niche references a payload binding and its
  reserved absent representation. The wire kind retains remaining niche state.
- `CBranch::Alternatives` / `JniBranch::Alternatives` identify a choice between
  child arms. The selector is an ordinary control wire, validated against the
  adapter's tag representation.
  Its remaining niche state is stored on that same wire.

`wire_definitions` and `wire_definitions_mut` visit every terminal and control
wire once, including all alternative arms and one schema per repeated element.
No metadata-specific wire enumeration trait is needed. This exhaustive list is
not an extern signature: generators still select function parameters, result
fields, sequence row fields and callback arguments for their respective scopes.
Wrapper infrastructure such as JNI environment/class parameters is supplied by
the wrapper writer rather than modeled as a conversion value.

Supporting values are outside their own node's payload: a presence flag remains
available for an absent option, a selector is outside its arms, and a row buffer
or count is outside the repeated row schema. Enclosing conditions still apply.
`WirePath` starts with a root index. At either parent shape, `[0, i]` selects
control `i` and `[1, i]` selects child `i`; a chain has only child zero. One final
index selects a wire within a leaf bundle. Rebuild paths after structural edits.

Function sequence callbacks remain supported only for successful returns,
including supported optional wrapping. Parameters, error channels and nested
rows use buffered sequences. JNI's native callback fills an internal sink; user
folds run after native return and handle-lock release.

Optional and alternative metadata hooks return `Result`; errors leave installed
conversions unchanged. Other adapters can use several controls, such as separate pointer and
length values, and choose their own metadata types and policies.

For an option, the compiler chooses the discriminator and absent representation
before calling the hook. The hook receives the normalized presence binding used
by the prepared payload conditions; it can be a computed local rather than a
physical wire. The leaf's `W` records the niche still available to an enclosing
option. After C reserves `0x110000` for `None` in `Option<char>`, that leaf can
offer `0x110001` to the next option. `Codec::set_niche` updates this state;
`Codec::niche` reads it. `None` means no niche remains.

Record helpers check source field names while constructing input and
borrowed or owned output conversions. Native-plan finalization checks physical
wire names and success-expression counts. Adapter validation checks semantic
requirements such as nonempty alternatives, presence references, field grouping
and Kotlin binding names. All three shared tree shapes are structurally valid;
there is no metadata-blind semantic validator. Completion
channels returned prematurely by a nested leaf conversion are rejected.

Custom converters implement `Conversion` for code emission and the applicable
`InputConversion` or `OutputConversion` interface for their description.
Custom codecs return `CodecInput<Self>` and `CodecOutput<Self>`, declare the
`Chain` and `Branch` metadata types and implement their hooks. For the shared
plans, `Bindings<W, C, N, B>`, `InputPlan<W, C, N, B>` and
`OutputPlan<W, C, N, B>` default all parameters after `W` to `()`; `C` here is
the emission context, `N` is chain metadata and `B` is branch metadata. Shared value and boxed helpers create terminal
bundles without inventing parent metadata.

Niche metadata may describe enum tags, full-width unsigned values, bit patterns,
or structured predicates. A custom float domain may explicitly exclude a NaN
payload (test raw bits) or all NaNs (test `is_nan`); unrestricted `f32`/`f64` retain
all valid values and use flags. The C and JNI adapters currently advertise only
the representations in the table above. Foreign-language predicate rendering and
transport stability remain adapter responsibilities.

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

Adapter API migration for the logical schema:

- `WireNode<W, C, B>` separates structural shapes from adapter semantics.
  Rename terminal `Wire` to `Leaf`; map `Optional` and `Sequence` to `Chain`
  with `child`, and `Group` and `Alternatives` to `Branch` with `children`.
  Metadata moves to `meta` and has one type for chains and one for branches.
  Both parent shapes own `controls`; metadata must not own wire descriptors.
  C/JNI distinguish the old semantic roles in their own metadata enums.
- `Conversion` and `Codec` expose `Chain` and `Branch` metadata types.
  Optional/group hooks return those types; sequence transport hooks return
  chain metadata. Adapters validate semantic constraints and find their own
  sequence scopes. The shared `validate_wire_nodes` and `outer_sequences`
  operations have been removed.
- `Conversion` provides generator-defined emission requests. Inputs implement
  `InputConversion::input_node`; outputs implement
  `OutputConversion::output_outcomes`. The separate `Outcomes<N>` describes
  completion channels and cannot be a child node.
- Use explicit `wire_definitions` / `wire_definitions_mut` traversal over root
  iterators. Select a completion channel first. Paths include terminal bundle
  indices. At either parent shape, `[0, i]` addresses a control and `[1, i]`
  addresses a child; chains have only child zero. Rebuild paths after structural edits.
- Stored plans expose `InputPlan::node` and `OutputPlan::outcomes`.
  `OutputPlan::values` and sequence-hook reads remain keyed by binding name.
  `match_output_fields` matches expressions to selected destination fields.
- Codecs implement option niche reservation/presence, absent-output preparation,
  niche retirement, path prefixing and conversion validation as needed.
- `SequenceInput<W, C>`/`SequenceOutput<W, C>` return controls separately from metadata. Callback variant
  eligibility and sequence callback signatures belong to the adapters.
- `staged::emit_call` receives the generated function's parameter names for
  reserving locals; it never derives a parameter list from the logical tree.
- Codecs describe source errors through `source_error_layout` and binding
  failures through `binding_failure_layout`. C/JNI output signatures, typed
  error handlers and ownership rules are retained.
