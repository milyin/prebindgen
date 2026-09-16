<!-- spec: {"kind": "variant", "example": "struct", "stage": "04-values", "language": "c"} -->

[Stage chapter](../../stages/04-values.md) · [Common cell][struct_values] · [Element path][struct]
Owner: the registry; the C adapter describes the aggregate and its member reads

# Record with scalar fields — Plan value conversions — C

## Input

```text
Crossing { source: Stamp, direction: IntoRust }
relation: Stamp.fields, parts [secs, nanos]
policy:   data_struct named Stamp, passed by value
```

## Result

The representation, and one operation per member:

```text
ReprSpec {
    layout:   Aggregate { ty: <the repr(C) Stamp>, members: [secs, nanos] },
    protocol: Product { projections: [read_secs, read_nanos], … },
}
```

```rust
// read_secs; read_nanos differs only in the member it names.
PrimitiveSpec {
    signature: PrimitiveSignature {
        operands: vec![OperandSpec {
            ty: OperationType::Carrier(stamp_aggregate),   // the repr(C) Stamp
            access: Access::Shared,
        }],
        results: vec![OperationType::Carrier(c_i64_type)],
    },
    failure:      PrimitiveFailure::Infallible,
    validity:     ValidityContract { results: vec![ResultValidity::Independent] },
    resources:    ResourceContract::none(),
    dependencies: vec![stamp_aggregate_decl],              // the generated type
    implementation: COperation::Rust(StandardRustOp::ReadMember { member: secs_member }),
}
```

Applied to an aggregate the wrapper holds under the name `stamp` — the source
parameter's name, which the boundary keeps — that description renders:

```rust
stamp.secs
```

## Checks

- The whole record conversion is infallible: reading a member cannot fail, and
  the copied integer is independent of the aggregate afterwards.
- `implementation` stores the operation and the member identity, not the string
  `stamp.secs`. The caller's value comes from the application, and the name from
  the boundary.
- `ReadMember` is a common Rust operation, so C ships no field-read renderer.
- A member identity is not a source field identity: the adapter's representation
  maps one to the other, and the registry validates that mapping. The aggregate
  those members belong to is [emitted here][struct_emit_c].

[struct]: README.md
[struct_values]: 04-values.md
[struct_emit_c]: 07-emit.c.md
