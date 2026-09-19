<!-- spec: {"kind": "variant", "example": "struct", "stage": "05-represent", "language": "c"} -->

[Stage chapter](../../stages/05-represent.md) · [Common cell][struct_represent] · [Element path][struct]
Owner: the registry; the C adapter describes the aggregate and its member reads

# Struct with scalar fields — Represent and compose values — C

## Input

```text
Crossing { source: Stamp, direction: IntoRust }
relation: Stamp.fields, parts [secs, nanos]
policy:   data_struct named Stamp, passed by value
```

## Result

The [representation](../../stages/05-represent.md#represent-and-compose-values) contains
one C-compatible struct and one read operation for each member. `Product`
means the registry obtains the selected [relation](../../stages/04-select.md#what-a-relation-is)'s parts separately, then
combines their converted values. The operation below uses the current fields,
with descriptive variables for the [carrier](../../stages/05-represent.md#describing-target-values-and-operations) types and member.

```text
ReprSpec {
    layout:   Aggregate { ty: <the repr(C) Stamp>, members: [secs, nanos] },
    protocol: Product { projections: [read_secs, read_nanos] },
}
```

```rust
// read_secs; read_nanos differs only in the member it names.
PrimitiveSpec {
    operands: vec![OperandSpec::value(
        OperationType::Carrier(stamp_aggregate), // the repr(C) Stamp
        Access::Shared,
    )],
    result: Some(OperationType::Carrier(c_i64_type)),
    failure:      PrimitiveFailure::Infallible,
    dependencies: vec![], // the public type's SurfaceSpec contributes the struct
    implementation: Operation::Standard(StandardOp::ReadMember { member: secs_member }),
}
```

`Access::Shared` means the read borrows its input rather than consuming the
whole struct before the next member can be read. The resulting integer is a
copy. `ReadMember` is a common operation rendered by the engine, so the C adapter
does not need to supply Rust text for it. The aggregate declaration is retained
through the public type's description, not as a dependency on this read.

Applied to the [wrapper](../../stages/06-boundary.md#assemble-the-wrapper-boundary)'s
input named `stamp`, the member-read operation renders:

```rust
stamp.secs
```

## Checks

- The whole struct [conversion](../../stages/04-select.md#select-conversion-relations) is infallible: reading a member cannot fail, and
  the copied integer is independent of the aggregate afterwards.
- `implementation` stores the operation and the member identity, not the string
  `stamp.secs`. The caller's value comes from the application, and the name from
  the boundary.
- `ReadMember` is a common Rust operation, so C ships no field-read renderer.
- A member identity is not a source field identity. The adapter pairs members
  with source fields in declaration order. The registry checks the member/part
  count and rejects reads naming undeclared members; it does not independently
  prove that the adapter paired each member with the intended source field.
  The aggregate those members belong to is [emitted here][struct_emit_c].

[struct]: README.md
[struct_represent]: 05-represent.md
[struct_emit_c]: 08-emit.c.md
