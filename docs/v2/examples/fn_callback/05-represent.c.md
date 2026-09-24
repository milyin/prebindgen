<!-- spec: {"kind": "variant", "example": "fn_callback", "stage": "05-represent", "language": "c"} -->

[Stage chapter](../../stages/05-represent.md) · [Common cell][fn_callback_represent] · [Element path][fn_callback]
Owner: the registry; the C frontend states the closure struct and its call

# Function taking a callback — Represent and compose values — C

## Input

```text
impl Fn(i64), IntoRust, callback.args [arg 0: i64 OutOfRust, atomic]
representation: Callback { wire_type: `closure_i64`, capture: Identity, invoke: Call, routes: [] }
```

## Result

```text
node(each)  wire type:  closure_i64, members [arg 0: i64]
            capture:  none — the identity keeps the struct itself
            closure:  move |a0: i64| { Call(each, a0) }
            failures: {}
```

A C caller hands over a struct of three members: `context`, its own pointer;
`call`, the function to call with each argument and that context; and `drop`,
the function that frees the context. The struct is all a call needs, so there
is nothing to capture: the identity renders nothing, and the closure takes the
struct itself. `Call` is the one operation the C target writes. Applied to the
struct `each` and the argument's [wire type](../../stages/05-represent.md#describing-target-values-and-operations) `a0`, it renders:

```rust
{
    let closure = &each;
    if let ::core::option::Option::Some(call) = closure.call {
        unsafe { call(a0, closure.context) }
    }
}
```

The struct is borrowed whole before its members are read. A Rust closure
captures only the members it uses, and `context` alone is a raw pointer, which
is neither `Send` nor `Sync`; the struct is both, and the closure has to be.

## Checks

- The [node](../../stages/05-represent.md#represent-and-compose-values) is infallible: calling through a function pointer cannot fail, and
  an `i64` leaves Rust unchanged. So there is no route to state, and none is.
- A struct with no `call` is called and does nothing: the C caller asked to be
  told nothing.
- `drop` is not an operation: the struct is dropped when the closure is, and
  the declaration the C target writes for the
  [wire type](../../stages/05-represent.md#describing-target-values-and-operations)
  implements `Drop` by calling it. The C caller's context lives exactly as long
  as Rust keeps the callback — [the emitted struct][fn_callback_emit_c] shows it.

[fn_callback]: README.md
[fn_callback_represent]: 05-represent.md
[fn_callback_emit_c]: 08-emit.c.md
