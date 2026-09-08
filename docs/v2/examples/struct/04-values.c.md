<!-- spec: {"kind": "variant", "example": "struct", "stage": "04-values", "language": "c"} -->

# Record with scalar fields — Plan value conversions — C

[Stage chapter](../../stages/04-values.md) · [Common cell][struct_values] · [Element path][struct]

## Input

The record crossing with the C policy: a by-value `Stamp` aggregate whose
members carry the two field values.

## Owner

The registry composes the conversion; the C adapter describes the aggregate's
Rust ABI type, its member identities and one member-read operation per member.

## Result

The representation is an aggregate layout over the `Stamp` carrier with two
members, and a product protocol whose projections are those member reads. Every
read is infallible, and its result is independent of the aggregate — a copied
integer owes nothing to the struct it came from — so the whole record conversion
is infallible too. That is what lets [the C function boundary][fn_boundary_c]
declare no failure routes at all.

## Checks

A **member identity** refers to one member of the described target aggregate. It
is different from the source field identity, because source and target names can
differ. The adapter's representation maps each source child to its target member,
and the registry validates that mapping.

The member expression is **not stored as a string containing `arg0.secs`**.
`PrimitiveSpec.implementation` stores the selected operation and member identity;
the registry's application supplies the particular aggregate value, and the
common writer assigns that value a Rust name.

## Representation

The stored specification for reading the target `secs` member:

```rust
// Specification sketch. These IDs refer to definitions in the adapter's
// returned description, imported and checked by the registry.
let read_secs = PrimitiveSpec {
    signature: PrimitiveSignature {
        operands: vec![OperandSpec {
            ty: OperationType::Carrier(stamp_aggregate), // The repr(C) Stamp, as an ABI carrier.
            access: Access::Shared,
        }],
        results: vec![OperationType::Carrier(c_i64_type)],
    },
    failure: PrimitiveFailure::Infallible,
    validity: ValidityContract {
        results: vec![ResultValidity::Independent],
    },
    resources: ResourceContract::none(),
    dependencies: vec![stamp_aggregate_decl],
    implementation: COperation::Rust(StandardRustOp::ReadMember {
        member: secs_member,
    }),
};
```

`stamp_aggregate` and `c_i64_type` are registered carrier type references;
`stamp_aggregate` is the generated `repr(C)` `Stamp`, not `source::Stamp`, which
the source model owns and no primitive names.
`stamp_aggregate_decl` identifies the generated type's artifact.
`secs_member` identifies that declaration's `secs` member. The `nanos`
specification has the same contracts and uses `nanos_member` instead.
`ResourceContract::none()` denotes no extra runtime ownership obligation.

`COperation` is the C adapter's rendering payload type for this example. Its
`Rust` variant selects a common Rust operation supplied by the registry library.
`StandardRustOp::ReadMember` means a typed Rust field read. The common writer
already knows how to render that operation, so C supplies no custom field-read
renderer.

With the allocated operand name `arg0` and the member spelling `secs`, rendering
this one primitive produces:

```rust
arg0.secs
```

The registry puts that expression into a local assignment and then constructs the
source record. Neither the local assignment nor the source construction lives
inside the member-read primitive.

## Along this element

See the [common cell][struct_values] for the record conversion both targets share,
and [the C emission][struct_emit_c] for the aggregate these operations read.

[struct]: README.md
[struct_values]: 04-values.md
[struct_emit_c]: 07-emit.c.md
[fn_boundary_c]: ../fn/05-boundary.c.md
