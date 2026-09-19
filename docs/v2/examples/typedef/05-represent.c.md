<!-- spec: {"kind": "variant", "example": "typedef", "stage": "05-represent", "language": "c"} -->

[Stage chapter](../../stages/05-represent.md) · [Common cell][typedef_represent] · [Element path][typedef]
Owner: the registry; the C adapter states the [carrier](../../stages/05-represent.md#describing-target-values-and-operations)

# Type alias declaring an opaque handle — Represent and compose values — C

## Input

```text
Crossing { source: Ledger, direction: IntoRust }    relation: atomic   policy: opaque_ptr Ledger
Crossing { source: Ledger, direction: OutOfRust }   relation: atomic   policy: opaque_ptr Ledger
```

## Result

The carrier is a pointer to the incomplete C type the adapter declares, and the
adapter's whole answer is three standard descriptions over it:

```text
node(Ledger, IntoRust)   ReprSpec { layout: Scalar(*mut Ledger),
                                    protocol: Terminal(from_raw(*mut Ledger -> Ledger)),
                                    release: Some(release(*mut Ledger)) }
node(Ledger, OutOfRust)  ReprSpec { layout: Scalar(*mut Ledger),
                                    protocol: Terminal(into_raw(Ledger -> *mut Ledger)),
                                    release: None }
```

```rust
// The adapter's `represent`, for the atomic relation under an opaque_ptr policy.
let carrier = WireType::abi(parse_quote!(*mut Ledger));   // the adapter's own type, not source::Ledger
match direction {
    Direction::IntoRust => ReprSpec {
        layout: Layout::Scalar(carrier.clone()),
        protocol: Protocol::terminal(PrimitiveSpec::from_raw(carrier.clone(), ty.clone())),
        release: Some(PrimitiveSpec::release(carrier, ty)),
    },
    Direction::OutOfRust => ReprSpec {
        layout: Layout::Scalar(carrier.clone()),
        protocol: Protocol::terminal(PrimitiveSpec::into_raw(ty, carrier)),
        release: None,
    },
}
```

`PrimitiveSpec::from_raw`, `into_raw` and `release` build the three standard
[primitives](../../stages/05-represent.md#represent-and-compose-values) with
their operand types, results and failures fixed: an adapter cannot describe the
same operation with a different failure, so a C route that aborts and a JNI
route that throws agree on what they are handed. Applied to the
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
