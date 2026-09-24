<!-- spec: {"kind": "variant", "example": "typedef", "stage": "06-boundary", "language": "c"} -->

[Stage chapter](../../stages/06-boundary.md) · [Common cell][typedef_boundary] · [Element path][typedef]
Owner: the registry, on the C frontend's release form

# Type alias declaring an opaque handle — Assemble the wrapper boundary — C

## Input

```text
node(taken) : carrier *mut Ledger, release infallible

the release form the C frontend recorded on `type:Ledger`'s output:
  symbol "ledger_drop", extern "C", no context parameters, inputs [this_],
  Binding -> abort
```

## Result

```text
FunctionPlan {
    abi, symbol: extern "C", "ledger_drop",
    params:      [ this_: *mut Ledger -> node(taken) ],
    ret:         none,
    routes:      [ Binding -> abort ],
}
```

This plan fixes the signature that [emission][typedef_emit_c] renders:

```rust
#[no_mangle]
pub extern "C" fn ledger_drop(this_: *mut Ledger)
```

The C frontend names the release from the type, as v1's destructors are
named, and its one input `this_`, as v1's destructors do. The two functions
reaching the handle have ordinary function forms, whose one route now has a
failure to take:

```rust
#[no_mangle]
pub extern "C" fn ledger_open(stamp: Stamp) -> *mut Ledger
#[no_mangle]
pub extern "C" fn ledger_close(ledger: *mut Ledger) -> i64
```

## Checks

- The binding route has no reporting operation: C has no exception, and a
  function returning `int64_t` has no slot for a message. It terminates by
  aborting, which is v1's `.panic()` convention. A `Result` out-parameter would
  need a delivery this increment does not implement.
- `ledger_drop(NULL)` does not take the route: the release is infallible and a
  null address releases nothing. Only a *consuming*
  [conversion](../../stages/04-select.md#select-conversion-relations) —
  `ledger_close` — fails on null.
- No runtime route is declared, and none is needed: nothing on this target
  calls into a runtime that could fail.

[typedef]: README.md
[typedef_boundary]: 06-boundary.md
[typedef_emit_c]: 08-emit.c.md
