<!-- spec: {"kind": "variant", "example": "fn_callback", "stage": "04-select", "language": "c"} -->

[Stage chapter](../../stages/04-select.md) · [Common cell][fn_callback_select] · [Element path][fn_callback]
Owner: the registry, from the rules the C frontend recorded

# Function taking a callback — Select conversion relations — C

## Input

The callback's [crossing](../../stages/03-requests.md#finding-an-existing-conversion-plan),
and the rules the C frontend recorded for it and its argument:

```text
Crossing { source: impl Fn(i64), direction: IntoRust }   Type(impl Fn(i64)), into Rust -> Callable over `closure_i64`
                                                         Type(i64), out of Rust        -> Whole over `i64`, identity
```

## Result

```text
param each    impl Fn(i64), IntoRust   -> callback.args, carried in `closure_i64`
  arg 0       i64,          OutOfRust  -> atomic,        carried in `i64`
```

The argument leaves Rust in the `i64`
[wire type](../../stages/05-represent.md#describing-target-values-and-operations)
it would leave in anywhere, which is what the closure struct's `call` will be
declared to take. `param stamp` selects exactly as on
[the function path][fn_select_c].

## Checks

- The closure struct holds scalars, addresses and enums as arguments, and `i64`
  resolves to a scalar, so the struct accepts it. An argument carried as
  anything else refuses the callback with `unsupported.c.arg.<class>`, at that
  argument.
- The `i64` leaving Rust here is the same
  [conversion](../../stages/04-select.md#select-conversion-relations) a
  function returning an `i64` uses: one rule, one [representation](../../stages/05-represent.md#represent-and-compose-values), one crossing.

[fn_callback]: README.md
[fn_callback_select]: 04-select.md
[fn_select_c]: ../fn/04-select.c.md
