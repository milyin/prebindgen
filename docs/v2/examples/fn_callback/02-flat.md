<!-- spec: {"kind": "cell", "example": "fn_callback", "stage": "02-flat"} -->

[Stage chapter](../../stages/02-flat.md) · [Element path][fn_callback] · [Source crate](../../source.md)
Owner: Flat · Previous: [Capture source items][fn_callback_source] · Next: [Record binding requests][fn_callback_requests]

# Function taking a callback — Build and inspect the source model

## Input

From capture, the function's record and, from [the struct's path][struct_flat],
the `Stamp` it takes:

```json
{ "kind": "function", "name": "stamp_each",
  "content": "pub fn stamp_each(stamp: Stamp, each: impl Fn(i64) + Send + Sync + 'static) { /* placeholder */ }",
  "source_location": { "file": "src/source.rs", "line": 10, "column": 1 } }
```

## Result

```text
Element::Function(Function {
    name:   stamp_each,
    params: [ Param { name: stamp, ty: TypeRef { kind: Named { id: Stamp, args: [] } } },
              Param { name: each,  ty: TypeRef { kind: Callback { args: [ Scalar(I64) ] } } } ],
    ret:    TypeRef { kind: Unit },
    origin: <the captured syntax, src/source.rs:10>,
})
```

`Callback` is the model's word for one exact type shape:
`impl Fn(A, B, …) + Send + Sync + 'static`, with nothing returned. Its `args`
are the argument types in order, each read like any other type — `Scalar(I64)`
here. `Unit` is the function's own return: `stamp_each` returns nothing.

The type's key, which a binding names it by, is its canonical spelling:

```rust
let each = &model.function("stamp_each").unwrap().params[1].ty;
let TypeKind::Callback { args } = each.kind() else { … };
assert!(matches!(args[0].kind(), TypeKind::Scalar(ScalarKind::I64)));
assert_eq!(each.key(), TypeKey::parse("impl Fn(i64) + Send + Sync + 'static")?);
```

## Checks

- The shape is checked, not assumed. A callback returning a value, one missing
  any of the three bounds, and any other `impl Trait` are refused when the
  model is built, so the function is refused with them and no later stage
  meets a callable it cannot call from another thread.
- An argument's type resolves like a parameter's: an argument naming a type
  nothing declares refuses the function.
- Nothing here says a caller passes a C function pointer or a Kotlin lambda —
  the model states the Rust signature only.

[fn_callback]: README.md
[fn_callback_source]: 01-source.md
[fn_callback_requests]: 03-requests.md
[struct_flat]: ../struct/02-flat.md
