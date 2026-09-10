<!-- spec: {"kind": "cell", "example": "fn", "stage": "02-flat"} -->

[Stage chapter](../../stages/02-flat.md) · [Element path][fn] · [Source crate](../../source.md)
Owner: Flat · Previous: [Capture source items][fn_source] · Next: [Record binding requests][fn_requests]

# Function taking an owned record — Build and inspect the source model

## Input

The capture entry above, parsed back into the item it was, in a build that also
captured [`Stamp`][struct_flat].

## Result

One element in the namespace, under the name `stamp_sum`:

```text
Element::Function(Function {
    name:   stamp_sum,
    params: [ Param { name: stamp, ty: TypeRef { kind: Named { id: Stamp, args: [] } } } ],
    ret:    TypeRef { kind: Scalar(I64) },
    origin: <the captured syntax, src/source.rs:6>,
})
```

Reached and walked as:

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
