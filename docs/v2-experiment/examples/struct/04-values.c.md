<!-- spec: {"example": "struct", "kind": "variant", "language": "c", "stage": "04-values"} -->

# Struct: Stamp — Plan value conversions — c

[Pipeline chapter](../../stages/04-values.md) · [Common contract](04-values.md) · [Example path](README.md)

## Input

RecordRelation with two i64 children, selected StampC carrier and its two member identities.

## Owner

C adapter selects common member-read operations; registry builds the record body.

## Result

Each PrimitiveSpec takes Shared StampC and returns native i64; failure Infallible, result validity Independent, resources none, dependency StampC declaration. Payload selects `COperation::Rust(StandardRustOp::ReadMember { member })`. For secs, its emitted expression is `arg0.secs`; for nanos, `arg0.nanos`.

## Checks

The payload holds an operation/member identity, not a caller variable name or complete converter. The common writer renders member reads; registry adds assignments and source construction. See the complete specification below.

## Concrete description or output

```rust
// Specification sketch. These IDs refer to definitions in the adapter's
// returned description, imported and checked by the registry.
let read_secs = PrimitiveSpec {
    signature: PrimitiveSignature {
        operands: vec![OperandSpec {
            ty: OperationType::Carrier(stamp_c_type), // Rust StampC ABI carrier.
            access: Access::Shared,
        }],
        results: vec![OperationType::Carrier(c_i64_type)],
    },
    failure: PrimitiveFailure::Infallible,
    validity: ValidityContract {
        results: vec![ResultValidity::Independent],
    },
    resources: ResourceContract::none(),
    dependencies: vec![stamp_c_declaration],
    implementation: COperation::Rust(StandardRustOp::ReadMember {
        member: secs_member,
    }),
};
```

The names `stamp_c_type`, `c_i64_type`, `stamp_c_declaration` and `secs_member` refer to checked definitions returned by the adapter. The nanos primitive differs only in member identity. `ResourceContract::none()` records no extra ownership obligation.

## Related example

[function: c at this stage](../function/04-values.c.md)
