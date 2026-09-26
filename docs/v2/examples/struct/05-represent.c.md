<!-- spec: {"kind": "variant", "example": "struct", "stage": "05-represent", "language": "c"} -->

[Stage chapter](../../stages/05-represent.md) · [Common cell][struct_represent] · [Element path][struct]
Owner: the registry; the C frontend states the aggregate and its member read

# Struct with scalar fields — Represent and compose values — C

## Input

```text
Crossing { source: Stamp, direction: IntoRust }
relation: Stamp.fields, parts [secs, nanos]
representation: Parts { via: Fields, wire_type: `Stamp`, read: ReadMember }
```

## Result

The [representation](../../stages/05-represent.md#represent-and-compose-values)
is one C-compatible struct and one read, applied to each member. The C
frontend states it when it reads `data_type!(Stamp)`:

```rust
let stamp_c = binding.wire_type(CWireType::Aggregate {
    name: format_ident!("Stamp"),                  // the repr(C) `Stamp` C declares
});
InRepresentation::Parts {
    via: Via::Fields,
    wire_type: stamp_c,
    read: Operation::Standard(StandardOp::ReadMember), // infallible, needs no context
}
```

`ReadMember` names no member: the registry applies it once per part of the
selected [relation](../../stages/04-select.md#what-a-relation-is), and the
part says which. It is a standard operation the registry writes itself, so the
C target writes no Rust for it. Applied to the
[wrapper](../../stages/06-boundary.md#assemble-the-wrapper-boundary)'s input
named `stamp`, for the part `secs`, it renders:

```rust
stamp.secs
```

The [wire type](../../stages/05-represent.md#describing-target-values-and-operations)'s
own declaration — the `repr(C)` struct — is what the C target writes, when the
registry feeds it the wire type and its two resolved members at emission.

## Checks

- The whole struct [conversion](../../stages/04-select.md#select-conversion-relations) is infallible: reading a member cannot fail, and
  the copied integer is independent of the aggregate afterwards.
- The member read is written from the part, not stored as text: the caller's
  value comes from the application, and its name from the form.
- The aggregate holds `I64` members only; a field resolving to anything else —
  another aggregate, a handle, an enum — refuses the struct with
  `unsupported.c.member.<class>`, where the member is.
- The aggregate those members belong to is [emitted here][struct_emit_c].

[struct]: README.md
[struct_represent]: 05-represent.md
[struct_emit_c]: 08-emit.c.md
