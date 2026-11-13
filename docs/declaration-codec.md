# Implementing a language conversion compiler

A language adapter owns its conversion operations and deferred choices. There is
no shared declaration operation vocabulary and no translation into adapter operations.
The shared crate provides tree structure, typed configuration containers, source-aware
resolution helpers and compilation helpers.

## Trees and names

`InputNode<InOps, OutOps, InRequest, OutRequest>` describes foreign-to-Rust
conversion; `OutputNode` describes Rust-to-foreign conversion. `InputOps` and
`OutputOps` are separate traits. Both node directions keep the same parameter
order, including inside callable domains.

The structural variants are `None`, `Request`, `Leaf`, `Unary`, `Branch` and
`Arrow`. Each adapter selects the operation types carried by the four latter
variants. An arrow's domain uses the opposite conversion direction; its
codomain uses the enclosing direction. Its argument and result conversions are
ordinary visible subtrees.

| Role | C | JNI |
| --- | --- | --- |
| Conversion trees | `CInput` / `COutput` | `JniInput` / `JniOutput` |
| Operation sets | `CInputOps` / `COutputOps` | `JniInputOps` / `JniOutputOps` |
| Deferred choices | `CInputRequest` / `COutputRequest` | `JniInputRequest` / `JniOutputRequest` |
| Function declaration | `DeclCFunction` | `DeclJniFunction` |
| Callback declaration | `DeclCCallback` | `DeclJniCallback` |
| Prepared converters | `CInputConverter` / `COutputConverter` | `JniInputConverter` / `JniOutputConverter` |
| Binding model | `CBindings` / `CSchema` | `JniBindings` / `JniSchema` |

Both phases use the same adapter operations:

```rust,ignore
// Configuration and source-dependent requests:
JniInput<JniInputRequest, JniOutputRequest>
JniOutput<JniInputRequest, JniOutputRequest>

// Resolved, request-free trees:
JniInput   // both request types default to Infallible
JniOutput
```

C follows the same scheme. C has no bulk or JVM-record operation variants and
its builders do not expose `.bulk()`. Import a language adapter's macros for
configuration (`fun!`, `callback!`, `expand_param!`, `expand_return!`, `fields!`).
Direct tree construction uses its `prelude` extension traits, for example
`use prebindgen_jni::prelude::*`.

## Configuration uses concrete adapter trees

`DeclFunction<InTree, OutTree>`, `DeclCallback<InTree, OutTree>` and
`DeclDefaults<InTree, OutTree>` are generic data containers. They do not select
an operation vocabulary. `DeclJniFunction`, for example, fixes its tree types to
JNI input/output nodes carrying JNI requests. Callbacks store output trees for
their arguments and input trees for their results.

Defaults and function-local overrides remain order-independent. Resolution
checks source signatures and applies local choices over type-wide defaults.
Callback overrides replace the corresponding argument/result tree and retain
the existing scoped-versus-transferred ownership rules.

## Shared helpers require only their operation's dependencies

Each adapter matches its own request and operation enums and calls shared
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

The same rule applies to the user-facing builders. `DeclInputAlternatives<T>`
handles alternative order and explicit temporary reconstruction, including the
empty expansion's fallback to the owned type's default. `getter_view` checks
accessor references and preserves view names. The C and JNI facades supply the
few node constructors needed by these operations and keep their existing fluent
methods. Return views use an ordinary `Vec`; field builders keep their small
accessor/override maps locally. `DeclError<I, O>` holds an error type and its mapper.
The independent test adapter uses `DeclInputAlternatives`, `getter_view` and the
shared source-resolution helpers too.

`resolve_container` shares Option/sequence fallback where payloads and elements
use the same child rule; adapters choose whether a collection is a native leaf
and construct their own wrappers. The direct-value paths that select different
payload/element defaults or check collection projections keep that logic local.
`input_value_conversion` and `output_value_conversion` share canonical conversion
lookup and direction checks. `override_output_field` edits named fields through
adapter-identified wrappers in place, without cloning the tree or substituting a
placeholder node.

Generalization stops where it adds more machinery than it removes. Small enum
constructors, request dispatch and source-reference matching stay in each adapter.
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

`resolve_bindings` installs request-free adapter trees in
`Flat<ResolvedSchema<C>>`, where `C` is the compilation codec. The model owns
source declarations, adapter metadata, compiler settings and resolved standalone
callback/error recipes. `compile_resolved(flat)` needs no external defaults table.
Explicitly supplied converters remain authoritative unless explicitly overridden.

The adapter implements `Codec::compile_input_tree` and `compile_output_tree` to
interpret its operations. Compilation produces `Flat<C::Schema>` for wrapper
emission. Shared `bind_input` and `bind_output` accept request-free adapter trees;
the adapters also expose convenience functions of those names that first resolve
their own requests with an empty defaults table.

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
