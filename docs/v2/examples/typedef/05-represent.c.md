<!-- spec: {"kind": "variant", "example": "typedef", "stage": "05-represent", "language": "c"} -->

[Stage chapter](../../stages/05-represent.md) · [Common cell][typedef_represent] · [Element path][typedef]
Owner: the registry; the C frontend states the [wire type](../../stages/05-represent.md#describing-target-values-and-operations)

# Type alias declaring an opaque handle — Represent and compose values — C

## Input

```text
Crossing { source: Ledger, direction: IntoRust }    relation: atomic   the Type(Ledger) rule
Crossing { source: Ledger, direction: OutOfRust }   relation: atomic   the Type(Ledger) rule
```

## Result

The wire type is a pointer to the incomplete C type the C target declares, and
the frontend's whole statement is three standard operations over it, recorded
when it reads `ptr_type!(Ledger)`:

```rust
let pointer = binding.wire_type(CWireType::Pointer {
    name: format_ident!("Ledger"),          // `*mut Ledger`: the C target's own type, not source::Ledger
});
Representation::Terminal {
    into_rust: Some(Codec { wire_type: pointer, operation: Operation::Standard(StandardOp::FromRaw) }),
    out_of_rust: Some(Codec { wire_type: pointer, operation: Operation::Standard(StandardOp::IntoRaw) }),
    release: Some(Operation::Standard(StandardOp::Release)),
}
```

A standard
[primitive](../../stages/05-represent.md#represent-and-compose-values)'s
failure is the registry's, fixed by the operation: taking a handle back fails in the binding category with a `String`,
and nothing else here can fail. A frontend cannot state the same operation
with a different failure, so a C route that aborts and a JNI route that throws
agree on what they are handed. Applied to the
[wrapper](../../stages/06-boundary.md#assemble-the-wrapper-boundary)'s value
`v3` and its parameters `ledger` and `this_`, the three render:

```rust
Box::into_raw(Box::new(v3)) as *mut Ledger
```

```rust
::core::ptr::NonNull::new(ledger as *mut source::Ledger)
    .map(|handle| unsafe { *Box::from_raw(handle.as_ptr()) })
    .ok_or_else(|| String::from("null `Ledger` handle"))
```

```rust
drop(::core::ptr::NonNull::new(this_ as *mut source::Ledger)
    .map(|handle| unsafe { Box::from_raw(handle.as_ptr()) }))
```

## Checks

- `*mut Ledger` names the adapter's incomplete type; `*mut source::Ledger`
  names the source type. The cast between them is the whole
  [conversion](../../stages/04-select.md#select-conversion-relations), and
  only the registry writes the second spelling.
- The into-Rust expression evaluates to `Result<source::Ledger, String>` and
  stops there; the `match` on it and the abort are
  [the boundary's][typedef_boundary_c].
- The release description has no result, which is what keeps it out of any
  conversion body: it can only be applied by the wrapper planned for it.

[typedef]: README.md
[typedef_represent]: 05-represent.md
[typedef_boundary_c]: 06-boundary.c.md
