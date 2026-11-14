# Implementing a language conversion compiler

A language adapter owns its conversion operations and deferred choices. There is
no shared declaration operation vocabulary and no translation into adapter operations.
The shared crate provides tree structure, typed configuration containers, source-aware
resolution helpers and compilation helpers.

## Trees and names

`InputNode<InOps, OutOps>` describes foreign-to-Rust
conversion; `OutputNode` describes Rust-to-foreign conversion. `InputOps` and
`OutputOps` are separate traits. Both node directions keep the same parameter
order, including inside callable domains.

The structural variants are `None`, `Leaf`, `Unary`, `Branch` and
`Arrow`. Each adapter selects the operation types carried by the four latter
variants. An arrow's domain uses the opposite conversion direction; its
codomain uses the enclosing direction. Its argument and result conversions are
ordinary visible subtrees.

| Role | C | JNI |
| --- | --- | --- |
| Final trees | `CInput` / `COutput` | `JniInput` / `JniOutput` |
| Configuration trees | `CInputConfig` / `COutputConfig` | `JniInputConfig` / `JniOutputConfig` |
| Operation sets | `CInputOps` / `COutputOps` | `JniInputOps` / `JniOutputOps` |
| Configuration operation sets | `CInputConfigOps` / `COutputConfigOps` | `JniInputConfigOps` / `JniOutputConfigOps` |
| Function declaration | `DeclCFunction` | `DeclJniFunction` |
| Callback declaration | `DeclCCallback` | `DeclJniCallback` |
| Prepared converters | `CInputConverter` / `COutputConverter` | `JniInputConverter` / `JniOutputConverter` |
| Binding model | `CBindings` / `CSchema` | `JniBindings` / `JniSchema` |

Unary, branch and arrow fields hold `InOps::Unary`, `InOps::Branch` and
`InOps::Arrow` directly (and the corresponding output associated types).
Each adapter defines its complete metadata enums. Configuration-only variants
have a `Config` prefix and a generic payload, whose default is `Infallible`.
The configuration operation sets select concrete payloads; final operation sets
use the uninhabited defaults. This applies to leaf choices as well as structural
operations. The shared tree has no `Request` variant or request parameters.

For example, the same branch enum serves both phases:

```rust,ignore
pub enum CInputBranch<EnumExpansion = Infallible, VariantExpansion = Infallible> {
    ConfigEnumFields(EnumExpansion),
    ConfigVariantFields(VariantExpansion),
    Fields,
    Selector,
    Constructor { function: String },
    Enum { value: bool },
    Variant,
}
// Only configuration can construct the first two variants.
pub type CInputConfigBranch = CInputBranch<(), CVariantFields>;
```

`ConfigDefault(())`, `ConfigValue(...)` and `ConfigConstructor(name)` are leaf
operations. Type scoping, partial fields and callback configuration use their
corresponding unary, branch and arrow types. Their final payloads are directly
`Infallible`, not an optional wrapper that could still contain `None`.

Explicit recipes always use `child`, `children`, `domain` and `codomain`.
Configuration metadata contains names, indices, types and settings, never nested
conversion trees. For example, a type-specific override is a unary node:

```rust,ignore
CInputConfig::Unary {
    operation: CInputUnary::ConfigForType { ty },
    child: Box::new(recipe),
}
```

Named record fields are branch children with parallel field names in metadata.
Partial enums contain variant branches, each naming its explicit field children.
A pending callback is already an arrow: its domain contains explicit argument
recipes and its codomain contains the result recipe. Argument indices and an
explicit-result flag describe which source defaults these children override.
Callback declarations and field builders can keep maps while being configured;
they place their recipes on structural edges when converted into a tree.

The shared child walkers traverse every supplied conversion, including the
children of configuration-only operations. Adapters inspect their own metadata
during source resolution and source-reference discovery.

Resolution interprets configuration payloads, fills missing source defaults and
returns the final operation sets. Compilation takes only these final types, so
configuration-only operations cannot reach it, including inside nested fields
or opposite-direction callback domains. C and JNI share their complete-operation
helper implementations between phases; metadata definitions are not duplicated.

```rust,ignore
// Configuration:
JniInputConfig   // InputNode<JniInputConfigOps, JniOutputConfigOps>
JniOutputConfig  // OutputNode<JniInputConfigOps, JniOutputConfigOps>

// After resolution:
JniInput         // InputNode<JniInputOps, JniOutputOps>
JniOutput        // OutputNode<JniInputOps, JniOutputOps>
```

`map_into()` converts each metadata value with `From`, preserving every child
and both arrow directions. C and JNI provide widening conversions from final
metadata to configuration metadata, allowing explicit resolved trees to be
embedded in configuration. The opposite direction requires source resolution;
there is no infallible conversion that could discard incomplete configuration.

C follows the same scheme. C has no bulk or JVM-record operation variants and
its builders do not expose `.bulk()`. Import a language adapter's macros for
configuration (`fun!`, `callback!`, `expand_param!`, `expand_return!`, `fields!`).
Direct tree construction uses its `prelude` extension traits, for example
`use prebindgen_jni::prelude::*`.

## Configuration uses concrete adapter trees

`DeclFunction<InTree, OutTree>`, `DeclCallback<InTree, OutTree>` and
`DeclDefaults<InTree, OutTree>` are generic data containers. They do not select
an operation vocabulary. `DeclJniFunction`, for example, fixes its tree types to
JNI configuration input/output nodes. Callbacks store output trees for
their arguments and input trees for their results.

Defaults and function-local overrides remain order-independent. Resolution
checks source signatures and applies local choices over type-wide defaults.
Callback overrides replace the corresponding argument/result tree and retain
the existing scoped-versus-transferred ownership rules.

## Shared helpers require only their operation's dependencies

Each adapter matches its own operation enums and calls shared
source-aware helpers. Helpers take concrete trees, source metadata and small
callbacks instead of a trait describing every supported operation.

| Helper | Shared responsibility | Adapter supplies |
| --- | --- | --- |
| `resolve_function` | Validate occurrence overrides and preserve prebound slots | Parameter/result resolution |
| `resolve_constructor` | Check constructor signature and resolve arguments | Child resolution and constructor operation |
| `resolve_getter` | Check borrowed or consuming receiver and select result type | Child resolution |
| `resolve_record_fields` / `resolve_named_fields` | Check fields and select local/default recipes | Field resolution |
| `resolve_enum_fields` / `resolve_enum_variants` | Check variant/member names and counts, preserve source order | Field resolution |
| `resolve_type_default` | Look up defaults and reject recursion | Resolution of the selected recipe |
| `resolve_typed_expansion` | Apply a typed override through Option/collection layers | Resolution and container construction |
| `resolve_conversion` | Follow canonical representations and reject cycles | Representation resolution |
| `resolve_callback` | Merge callback overrides, determine ownership and reverse directions | Argument and result resolution |
| `input_children` / `output_children` | Traverse structural children across arrow directions | Adapter-specific source-reference handling |

The declaration family is generic over the adapter's tree types and extension
settings. `DeclExpandParam<In, FunctionDecl, Extension>`,
`DeclExpandReturn<Out, FunctionDecl, Extension>` and `DeclFields<Recipe, Extension>` own
the complete fluent API. `DeclExpand<ParamDecl, ReturnDecl, ErrorDecl>` is their common
registration enum. C and JNI provide aliases and small factory functions that
supply node constructors. An adapter should also expose its own `fun!` macro
using its concrete function alias, including that alias's extension type, as C
and JNI already do. The shared `fun!` starts with no extension type;
`with_extension(options)` can attach settings explicitly. Expansion factories
are also available as `expand_param`, `expand_return` and `fields`, alongside the familiar macros of the same names.
An adapter's aliases cannot add inherent methods to shared Rust types; JNI adds
`bulk()` with `JniBulkExpansion` and record/enum/container policies with
`JniInputExpansion` and `JniOutputExpansion`, all exported by its `prelude`.
`DeclFields` stores named field recipes and an optional accessor. `param_fields!()`
selects incoming source fields; `fields!()` selects outgoing source fields and
`fields!(getter)` selects accessor-result fields. `variant_fields` shares the
named-field builder with C; JNI's `variant_record` selects its grouped Kotlin
representation and explicit owned-field consumption.

Each declaration carries an optional adapter-defined `extension`. The default
extension type is `Infallible`, so adapters without additional settings have no
extension values to configure. The same extension mechanism exists on
`DeclFunction<In, Out, Extension>`, `DeclCallback<In, Out, Extension>`,
`DeclError<FunctionDecl, Extension>` and `DeclConvert<Extension>`. Common fluent
methods retain these options. An adapter adds its own fluent methods through a
local extension trait and interprets the settings when resolving or constructing
its tree. It can put the resulting policy in its own operations or Flat metadata;
compilation still reads the complete resolved Flat.

Callback defaults inherit the whole extension value unless an occurrence supplies
one. `resolve_callback` retains that value in `CallbackOperation<Extension>`, so
it survives in the resolved arrow. `resolve_callback_arrow` shares signature and
arity validation and resolves arguments outward and the result inward.
`CallbackInput` is the purpose-specific construction bridge for converting a
shared `DeclCallback` into an adapter's complete input tree. The adapter delegates
to `callback_input`, supplying its typed-scope, argument-group and arrow
constructors. It does not return metadata containing the declaration's trees.
`resolve_callback_tree` borrows the arrow's structural children; declaration
objects and structural arrows use one default/ownership resolution algorithm.
`callback_variant` preserves a function's extension settings. The compiler's
`InputArrow::operation` uses the adapter's `InputOps::Arrow` type, so callback
settings can reach its compilation hook. Constructor helpers accept named parameter overrides, placed on structural branch
children; missing parameters inherit their defaults during source resolution.
Getter helpers accept a result override, placed on their structural unary child.
`constructor_input` and `getter_view` share this lowering between C and JNI.
Export-only settings and overrides in unsupported positions remain rejected;
binding options must not silently disappear from a getter, constructor or error mapper.

`LocalConversion` is the corresponding construction bridge for `DeclConvert`: it
keeps a local conversion inside a structural type-scope node, so appended
selectors and getters still apply to the declared type inside containers.

`prepare_value_conversions` borrows declarations. Its shared job is Rust
conversion validation and registration; the adapter can still read its extension
settings and apply them to its own metadata. `DeclDefaults` stores only error
mapper names. An adapter must apply error extensions to its own Flat metadata
before registering these mapper defaults.
Canonical conversion declarations may select opposite directions for one source
type; the adapter defines compatibility checks for their extension settings.
Extension data is not interpreted by the shared compiler. The independent
adapter tests extend input, return, field,
function, callback, error and conversion declarations without C/JNI operations.

`resolve_container` shares Option/sequence fallback where payloads and elements
use the same child rule; adapters choose whether a collection is a native leaf
and construct their own wrappers. The direct-value paths that select different
payload/element defaults or check collection projections keep that logic local.
`input_value_conversion` and `output_value_conversion` share canonical conversion
lookup and direction checks. `override_output_field` edits named fields through
adapter-identified wrappers in place, without cloning the tree or substituting a
placeholder node.

Small enum constructors, configuration dispatch and source-reference matching stay in
each adapter.
C ownership decisions and JNI record/bulk policies stay there as well. Sharing
those matches would require another common operation dictionary or a set of
callbacks larger than the local operation itself. The shared helpers instead own
the multi-step behavior that must remain consistent between adapters.

For example, constructor resolution does not require a complete `Codec` or
support for optionals, collections, callbacks or bulk transport:

```rust,ignore
resolve_constructor(
    flat,
    expected_type,
    "make_item",
    arguments,
    |ty, child| resolve_jni_input(ty, child),
    |function| JniInputBranch::Constructor { function },
)
```

The executable tests in `prebindgen-declarations/src/source.rs` exercise this
helper with an input vocabulary containing only scalar leaves and constructor
branches, and an uninhabited output vocabulary. No codec is implemented there.
The compiler conformance fixture has its own test-only operation vocabulary and
uses the same source helpers as C and JNI.

## Resolved Flat and compilation

`resolve_bindings` installs resolved adapter trees in
`Flat<ResolvedSchema<C>>`, where `C` is the compilation codec. The model owns
source declarations, adapter metadata, compiler settings and resolved standalone
callback/error recipes. `compile_resolved(flat)` needs no external defaults table.
Explicitly supplied converters remain authoritative unless explicitly overridden.

The adapter implements `Codec::compile_input_tree` and `compile_output_tree` to
interpret its operations. Compilation produces `Flat<C::Schema>` for wrapper
emission. Shared `bind_input` and `bind_output` accept resolved adapter trees;
the adapters also expose convenience functions of those names that first resolve
their own configuration with an empty defaults table.

`Codec` still selects compiled converter types, physical representations and
compilation hooks. Narrowing that compilation interface is separate from removing
the declaration vocabulary. Generation and all helper dispatch happen at build
time; generated calls gain no dispatch or allocations from this refactoring.

## Leaf and wire helpers

Besides its operation vocabularies and compilation state, a codec selects
`Schema`, `Wire`, `Context`, `Extra`, `Chain` and `Branch`. `Schema` fixes the
converters and declaration metadata; `Wire` describes one boundary value.
`Context` supplies Rust emission bindings, and `Extra` supplies occurrence
settings. `Chain` and `Branch` describe wire parents without duplicating children.

The existing physical-conversion hooks remain available to the reusable helpers:

| Operation | Purpose |
| --- | --- |
| `input_leaf(InputLeaf)` | Construct one Rust input from boundary values. |
| `output_leaf(OutputLeaf)` | Prepare boundary values from a Rust source expression. |
| `output_consumes(flat, ty, extra)` | Report whether the leaf moves its source. |
| `group(GroupKind)` | Describe values, fields or named output views. |

Optional features reject unsupported work by default. All interpretation and
composition happens during generation, not during an FFI call.

## Requests describe the actual conversion

`InputLeaf` carries `flat`, `ty`, `extra`, `wire`, `target`, `active`, and two
independent decisions made by the adapter:

- `representation`: `Ordinary` or native fields inside a `Bulk` frame.
- `role`: a function `Parameter` or a retained source callback's `CallbackResult`.
  The role propagates through nested fields and constructors.

`InputLeaf::new(...)` requests an ordinary enclosing parameter conversion.
Set the public fields when manually preparing another context. These request
values replace four old leaf hooks; they add no runtime dispatch.

`OutputLeaf` carries `flat`, `ty`, `extra`, `wire`, `value`, `access`, and
`representation`. `ValueAccess::Owned` means the expression has the declared
Rust type `T`; `Shared` means it observes `T` through `&T`. If `T` itself is a
reference, `Owned` does not grant ownership of its pointee.

`output_projected(request, projection)` remains separate. An explicit borrowed
projection can authorize a copy; an ordinary shared observation cannot. The
default rejects field overrides and owned projections of reference payloads,
including references inside options. C and JNI keep their existing specialized
projection checks. Source-return references, owner-backed getters and borrowed
sequences retain their separate operations and ownership rules.

`supports_bulk(Direction)` defaults to false. JNI checks it before bulk leaves and carriers; the sequence helpers check it
before using bulk transport. An implementation may support only one direction and
must still reject unsupported types. Whole-expansion packing (`bulk_input` /
`bulk_output`) remains distinct from element transport.

## Complete optional, alternative and sequence operations

`optional(Optional)` receives the mutable payload tree, requested normalized
presence binding, occurrence settings and direction. It returns one
`OptionalEncoding` containing:

- a flag wire named by that presence binding, or a niche with an input test;
- chain metadata describing that encoding;
- expressions for all inactive output values, keyed by binding name.

Input niches require a total, side-effect-free boolean test; output niches have
no input test. Input has no absent output expressions. Output's donor expression
must produce its reserved representation. The shared compiler validates flag
names, test direction and expression bindings; the codec validates its flag
representation and private niche metadata.

Niches are optional adapter implementation details. `Codec::Niche` is removed.
`prepare_optional_payload` and the smaller reservation helpers accept explicit
native-scope wires and supplied wire operations. C and JNI use these helpers;
the domain conformance tests use structured predicates unrelated to either ABI.
An adapter must select its own scope: a sequence carrier and its element fields
belong to different native scopes.

`selector(extra)` returns `Result<WireKind>`, so an unsupported selector can be
rejected. The compiler names and prefixes it, compiles and prefixes each arm,
then calls `alternatives(Alternatives)`. This operation validates the selector
and retires inactive arm offers. Runtime invalid-selector handling remains the
compiler's policy. All wires remain owned by the tree.

`sequence_input(SequenceInputRequest)` and
`sequence_output(SequenceOutputRequest)` choose ordinary or bulk transport using
`representation`. The requests retain element schema, length/index bindings,
input activation or output values and delivery. Transport results retain their
existing construction, cleanup and commit protocol. Callback delivery and bulk
packing within each callback element remain separate choices.

## Source errors

`source_error(SourceError)` receives the original error type, owned value and
optional explicit message-mapper expression. It returns a `SourceErrorPlan`
containing both the payload tree and its propagation expression. Layout and
routing are prepared together rather than selected by two independent hooks.

The default requires an explicit mapper and calls `mapped_error`; a codec that
only supports mapped String errors can override that convenience operation.
`prepare_mapped_source_error` provides the same fallback to codecs that also
handle typed errors. No implicit stringification is introduced. C keeps its
structured error outputs; JNI keeps typed domain-error callbacks and the
separate binding-error callback, with user handlers invoked after native return.

## Migration

The adapter factories replace direct expansion-builder constructors:

| Previous call | Replacement |
| --- | --- |
| `DeclCExpandParam::new(ty)` / JNI equivalent | `prebindgen_c::expand_param(ty)` / JNI equivalent |
| `DeclCExpandReturn::new(ty)` / JNI equivalent | `prebindgen_c::expand_return(ty)` / JNI equivalent |
| `DeclCFields::new(function)` / JNI equivalent | `prebindgen_c::fields(function)` / JNI equivalent |

Literal unary, branch and arrow constructors take their metadata directly:
`operation: CInputUnary::Optional`. Patterns use the same adapter variant without
an extra wrapper. A type-scoping operation is `CInputUnary::ConfigForType`; named output fields
belong to `COutputBranch`; enum field groups belong to `CInputBranch` or
`COutputBranch` according to direction; callbacks use `CInputArrow` (JNI
uses the corresponding `Jni*` names). Leaf metadata includes childless `Config*` choices with uninhabited final payloads.

The declaration macros retain their spelling. JNI expansion `.bulk()` now
requires `use prebindgen_jni::prelude::*` (or `JniBulkExpansion` directly).
`prepare_value_conversions` now accepts a borrowed declaration slice. Data-only
function/callback/error/conversion declarations can use `with_extension` to select
a new extension type; expansion factories fix their extension type because their
node constructors interpret it. `with_extension` replaces the whole previous
option value, including any options already selected by adapter methods.


| Former API | Replacement |
| --- | --- |
| `input_leaf`, `input_row_leaf`, `input_bulk_leaf`, `input_bulk_row_leaf` | `input_leaf(InputLeaf)` with representation/construction context. |
| `output_leaf`, `output_bulk_leaf` | `output_leaf(OutputLeaf)`. |
| `output_projected`, `output_bulk_projected` | `output_projected(OutputLeaf, Projection)`. |
| `consumes` | `output_consumes`. |
| `group_metadata`, `fields_group`, `field_group`, `named_output_group` | `group(GroupKind)`, returning `Result`. |
| Required optional metadata/reservation hooks | One optional `optional` operation; opt-in wire helpers. |
| `alternatives_metadata` plus niche retirement | `alternatives` receives the prefixed selector and mutable arms. |
| `sequence_*` and `sequence_*_bulk` | Request-based `sequence_input` / `sequence_output`. |
| `result_error_value` plus `source_error_layout` | `source_error`, returning both parts. |
| `result_error` | `mapped_error`. |
| Unit values `CCodec` / `JniCodec` | `CCodec::raw()` / `JniCodec::raw()` (also `Default`). |

Each language now implements `Codec` once. `CCodec::declared()` and
`JniCodec::declared()` apply enum, record, error and callback
metadata from Flat; builders choose this policy. Callback argument ownership
is stored on each argument's output subtree. `input_arrow(InputArrow)` receives
the callback metadata, opposite-direction domain and same-direction codomain;
no external callback output set is needed. Raw policy
preserves the former low-level conversion behavior; it still respects existing
C opaque-pointer and inline-owner declarations. There is no legacy forwarding
codec or second declaration implementation.
