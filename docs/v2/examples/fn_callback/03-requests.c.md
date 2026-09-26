<!-- spec: {"kind": "variant", "example": "fn_callback", "stage": "03-requests", "language": "c"} -->

[Stage chapter](../../stages/03-requests.md) · [Common cell][fn_callback_requests] · [Element path][fn_callback]
Owner: the C frontend

# Function taking a callback — Record binding requests — C

## Input

```rust
// build.rs
Cbindgen::builder()
    .source(source_crate::PREBINDGEN_OUT_DIR)
    .source_module(parse_quote!(source_crate))
    .declare(
        decls!()
            .data_type(data_type!(Stamp).base_name("Stamp"))   // see the struct path
            .callback(callback!(impl Fn(i64) + Send + Sync + 'static))
            .fun(fun!(stamp_each)),
    )
    .build_with(prebindgen_c::pipeline::Pipeline::V2)
    .expect("generate the C binding");
```

## Result

`callback!` declares the signature, and the frontend states it as data: a
[carrier](../../stages/05-represent.md#describing-target-values-and-operations)
for the closure struct a C caller fills in, the
[representation](../../stages/05-represent.md#represent-and-compose-values)
over it, the rule, and the output.

```rust
let closure = binding.carrier(WireType {
    rust: parse_quote!(closure_i64),
    class: CClass::Closure,
    // What an argument may be: the scalar, an address, an enum.
    members: Some(Accepts::of([CClass::I64, CClass::Pointer, CClass::Enum])),
    meta: CCarrier::Closure { c_name: "closure_i64".into() },
});
let callback = binding.representation(Representation::Callback {
    carrier: closure,
    capture: Operation::standard(StandardOp::Identity),   // the struct itself is kept
    invoke: Operation::target(COp::Call),                 // calls its `call`
    routes: Vec::new(),                                   // nothing a call does can fail
});
binding.rule(Scope::Type(key), callback);
binding.output(Declaration::Callback(key), OutputForm::Type { representation: callback, release: None, meta: () });
```

`key` is the type's key, `impl Fn(i64) + Send + Sync + 'static`, built from
the declared signature. `closure_i64` is the frontend's default name for it,
`closure_` followed by each argument's base; a `mangle_callback` hook, or
`.base_name(..)` on the declaration, names it otherwise. `stamp_each`'s form
is [the function path's][fn_requests_c], with a closure among what a
parameter may be.

## Checks

- C is opt-in: without the `callback!` declaration, no rule covers the
  callback type, and `stamp_each` is skipped with
  `unsupported.conversion.no_rule`.
- A struct is not among what an argument may be: a callback taking a `Stamp`
  would need it built out of Rust, which is refused before the closure's
  members are checked.
- No route is stated because the C call cannot fail, and neither can handing
  out an `i64`, an address or an enum. A C argument whose [conversion](../../stages/04-select.md#select-conversion-relations) could fail
  would refuse the callback with `unsupported.callback.unrouted_failure`.

[fn_callback]: README.md
[fn_callback_requests]: 03-requests.md
[fn_requests_c]: ../fn/03-requests.c.md
