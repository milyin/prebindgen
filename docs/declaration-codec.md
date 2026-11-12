# Implementing a language conversion compiler

`InputNode<I, O>` and `OutputNode<I, O>` describe composition: a leaf, a unary
operation, a branch, or an arrow. `None` represents unit. An arrow's domain uses
the opposite conversion direction and its codomain uses the enclosing direction.
The adapter supplies the meanings of all operations through the separate `I`
and `O` implementations of `Operations`.

C and JNI define their own operation enums in their `operations` modules. For
example, JNI has bulk packing and JVM object-record operations; C has neither.
An adapter can introduce a different operation without extending a common enum
or teaching a shared interpreter its meaning.

## Resolving and compiling

A `Flat<Resolved<C>>` stores the adapter's request-free conversion trees, source
metadata, codec and occurrence settings. `compile_resolved(flat)` consumes that
model and produces `Flat<C::Schema>` containing conversion plans for wrapper
emission. It needs no configuration registry outside the model. Standalone
callback and typed error recipes also live in Flat; `compile_declarations`
compiles them before the function conversions.

The adapter implements `Codec::compile_input_tree` and `compile_output_tree` to
interpret its own nodes. `compile_output_root` and `compile_output_return` allow
the adapter to preserve source-error routing and references returned by Rust.
The `Compiler` context provides source metadata, deterministic names, nested
compilation and reusable operations for constructors, fields, optional values,
sequences, enum payloads, conversions and getters. These helpers accept adapter
trees or compiled children; they do not match a shared operation enum.
`CompileState` belongs to the adapter. JNI uses it for nested bulk and ordinary
collection scopes; C has no such state.

An adapter can construct the resolved model directly using
`ResolvedModule::new`. The executable test
[`independent_operation_vocabularies_compile_without_declaration_frontend`](../prebindgen-declarations/src/operation_tests.rs)
builds and runs a wrapper with custom arithmetic input/output operations and
never implements `DeclarationCodec`.

## The optional declaration frontend

`Declarations` implements the existing expansion, conversion and callback
configuration language. Its `DeclarationInput` / `DeclarationOutput` trees
are configuration syntax, distinct from the adapter trees stored in resolved
Flat. The syntax enums are in `prebindgen_declarations::frontend`.

Adapters using that syntax also implement `DeclarationCodec`. Resolution applies
defaults and local overrides recursively, interpreting each syntax operation
into the adapter's vocabulary. `expand_input` and `expand_output` can return
adapter-specific trees directly. Their `Resolve` context resolves nested values
with the same defaults, recursion stack and occurrence policy. For example, a
JNI record expansion resolves each field before constructing its JVM-record
node. A callback's arguments and result use this same process in opposite
directions.

`finish_input_tree` / `finish_output_tree` normalize the resolved adapter tree.
JNI owns bulk promotion here. Per-element callback delivery is selected before
this promotion, preserving the callback element boundary.

Application authors normally use `Cbindgen` or `JniGen` declarations.
`Declarations::fill` remains available for individual functions and uses the
same resolution and compilation pipeline. Direct compiler callers supply
`prebindgen_c::operations::{Input, Output}` or the corresponding JNI types;
declaration syntax cannot accidentally be passed as a resolved adapter tree.

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
