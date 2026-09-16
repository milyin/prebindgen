<!-- spec: {"kind": "cell", "example": "fn", "stage": "02-flat"} -->

[Stage chapter](../../stages/02-flat.md) · [Element path][fn] · [Source crate](../../source.md)
Owner: Flat · Previous: [Capture source items][fn_source] · Next: [Record binding requests][fn_requests]

# Function taking an owned record — Build and inspect the source model

## Input

From capture, one line per marked item, parsed back into the items they hold:

```json
{ "kind": "function", "name": "stamp_sum",
  "content": "pub fn stamp_sum(stamp: Stamp) -> i64 { /* placeholder */ }",
  "source_location": { "file": "src/source.rs", "line": 6, "column": 1 } }
{ "kind": "struct", "name": "Stamp",
  "content": "pub struct Stamp { pub secs: i64, pub nanos: i64 }",
  "source_location": { "file": "src/source.rs", "line": 1, "column": 1 } }
```

## Result

Flat turns the function syntax into a structured entry indexed by `stamp_sum`.
The following is a readable summary of that entry, not a serialized format:

```text
Element::Function(Function {
    name:   stamp_sum,
    params: [ Param { name: stamp, ty: TypeRef { kind: Named { id: Stamp, args: [] } } } ],
    ret:    TypeRef { kind: Scalar(I64) },
    origin: <the captured syntax, src/source.rs:6>,
})
```

`params` preserves parameter order. `Named` means the parameter refers to a
declared type; `Scalar(I64)` identifies the built-in signed 64-bit return type.
`origin` retains source information for diagnostics and final Rust emission.

To inspect the parameter's fields, first find the function, then resolve the
name stored in its parameter type. The existing borrowed Flat API expresses
that navigation as follows (with uninteresting branches abbreviated):

```rust
let function = model.function("stamp_sum").unwrap();
let TypeKind::Named { id, .. } = function.params[0].ty.kind() else { … };
let Type::Struct(stamp) = model.resolve(id).unwrap() else { … };  // the record element
assert_eq!(function.ret.kind(), &TypeKind::Scalar(ScalarKind::I64));
```

## Checks

- The parameter's type holds the name `Stamp`; the declaration is
  [a separate element][struct_flat], reached by lookup.
- `Stamp` resolves, so this element survives; had nothing declared it, the
  function would have been refused with it.
- Nothing here says the function is exported, or that anything constructs a
  `Stamp` — the model states source facts only.

[fn]: README.md
[fn_source]: 01-source.md
[fn_requests]: 03-requests.md
[struct_flat]: ../struct/02-flat.md
