# Implementing a declaration codec

A `Codec` translates resolved `Input` and `Output` recipes into conversion
plans. The shared declaration compiler traverses constructors, fields, optional
values, sequences, getters and source results; a codec supplies the language's
physical values and conversion code. All codec operations run during generation.
The completed plans and language declarations remain in `Flat`; generation reads
that model without consulting the original recipes or another binding registry.

The wrapper generator drives Rust emission. An adapter author implements
conversion operations and invokes preparation; an application author normally
uses `Cbindgen` or `JniGen` declarations instead of implementing a codec.

## Four required operations

A codec selects six associated types: `Schema`, `Wire`, `Context`, `Extra`,
`Chain` and `Branch`. `Schema` fixes the converters and declaration metadata;
its input/output converters accept the codec's compiled plans. `Wire` describes
one boundary value, `Context` supplies Rust emission bindings, and `Extra` is
borrowed configuration for one conversion occurrence. `Chain` and `Branch`
describe parent nodes without owning duplicate children or wires.

Only these operations are required:

| Operation | Purpose |
| --- | --- |
| `input_leaf(InputLeaf)` | Construct one Rust input from boundary values. |
| `output_leaf(OutputLeaf)` | Prepare boundary values from a Rust source expression. |
| `output_consumes(flat, ty, extra)` | Tell the compiler whether the leaf moves its source, before it orders observations and moves. |
| `group(GroupKind)` | Describe values, fields, a named field, or a named output view; reject unsupported kinds. |

A scalar codec need not provide placeholder implementations for options,
selectors, collections, callbacks or source errors. Those operations reject
unsupported work by default. Validation and finalization defaults retain their
existing purposes. The executable
[four-method codec example](../prebindgen-declarations/src/operation_tests.rs)
compiles and runs a complete generated function call.

## Requests describe the actual conversion

`InputLeaf` carries `flat`, `ty`, `extra`, `wire`, `target`, `active`, and three
independent decisions made by the compiler:

- `representation`: `Ordinary` or native fields inside a `Bulk` frame.
- `construction`: `EnclosingConversion` or `GroupedRow`, whose decoded values
  must be combined while row storage remains alive.
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

`supports_bulk(Direction)` defaults to false. The shared compiler checks it
before every bulk leaf, carrier or sequence operation, including implicitly
selected bulk output. An implementation may support only one direction and
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
`JniCodec::declared(callback_outputs)` apply enum, record, error and callback
metadata from Flat; builders choose this policy. JNI's optional callback output
set supplies preparation context for retained callback parameters. Raw policy
preserves the former low-level conversion behavior; it still respects existing
C opaque-pointer and inline-owner declarations. There is no legacy forwarding
codec or second declaration implementation.
