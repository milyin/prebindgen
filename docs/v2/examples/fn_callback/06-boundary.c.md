<!-- spec: {"kind": "variant", "example": "fn_callback", "stage": "06-boundary", "language": "c"} -->

[Stage chapter](../../stages/06-boundary.md) · [Common cell][fn_callback_boundary] · [Element path][fn_callback]
Owner: the registry, on the C frontend's function form

# Function taking a callback — Assemble the wrapper boundary — C

## Input

```text
node(stamp) : produces an owned source Stamp, failures {}
node(each)  : produces the closure, failures {}

the form the C frontend recorded for `fn:stamp_each`:
  symbol "stamp_each", extern "C", no context parameters, inputs [stamp, each],
  one route: Binding -> abort
```

## Result

```text
FunctionPlan {
    abi, symbol: extern "C", "stamp_each",
    params:      [ stamp: Stamp -> node(stamp), each: closure_i64 -> node(each) ],
    ret:         none,
    routes:      [ Binding -> abort ],             // unused: nothing here can fail
}
```

This plan determines the signature that [emission][fn_callback_emit_c]
renders:

```rust
#[no_mangle]
pub extern "C" fn stamp_each(stamp: Stamp, each: closure_i64)
```

The closure struct is passed by value, so its ownership moves into Rust with
the call: from here on, Rust decides when `drop` runs.

## Checks

- The closure struct is a parameter of class `Closure`, which the C form's
  parameters accept.
- Neither [node](../../stages/05-represent.md#represent-and-compose-values) can fail, so the route goes unused and no error branch is
  written.

[fn_callback]: README.md
[fn_callback_boundary]: 06-boundary.md
[fn_callback_emit_c]: 08-emit.c.md
