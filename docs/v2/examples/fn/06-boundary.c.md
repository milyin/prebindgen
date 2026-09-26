<!-- spec: {"kind": "variant", "example": "fn", "stage": "06-boundary", "language": "c"} -->

[Stage chapter](../../stages/06-boundary.md) · [Common cell][fn_boundary] · [Element path][fn]
Owner: the registry, on the C frontend's function form

# Function taking an owned struct — Assemble the wrapper boundary — C

## Input

```text
node(input)  : produces an owned source Stamp, failures {}
node(output) : produces i64, failures {}

the form the C frontend recorded for `fn:stamp_sum`:
  symbol "stamp_sum", extern "C", no context parameters, inputs [stamp],
  one route: Binding -> abort
```

## Result

```text
FunctionPlan {
    abi, symbol: extern "C", "stamp_sum",
    params:      [ stamp: Stamp -> node(input) ],   // typed as node(input)'s wire type
    ret:         i64                                // node(output)'s wire type
    routes:      [ Binding -> abort ],              // unused: nothing here can fail
}
```

`stamp` is the first C argument. The [wrapper](../../stages/06-boundary.md#assemble-the-wrapper-boundary) uses it to construct the
source `Stamp`, then returns the source function's integer result directly.
These [conversions](../../stages/04-select.md#select-conversion-relations) raise nothing, so no error branch is written. An
**out-parameter**, by contrast, would be caller-provided storage that the wrapper
writes into; this example does not use one.

This plan determines the signature that [emission][fn_emit_c] renders:

```rust
#[no_mangle]
pub extern "C" fn stamp_sum(stamp: Stamp) -> i64
```

## Checks

- One aggregate in, one integer out: no synthetic parameters, no out-parameter.
  The symbol and calling convention come from the function form the C
  frontend recorded for this declaration, [as the request cell shows][fn_requests_c].
  The parameter's type is not the form's: it is the
  [wire type](../../stages/05-represent.md#describing-target-values-and-operations)
  its conversion resolved to, so the two cannot disagree.
- An out-parameter form would require an additional delivery implementation;
  the current V2 increment supports only a wrapper return or no value.
- The `Binding` route goes unused because both
  [nodes](../../stages/05-represent.md#represent-and-compose-values) are infallible,
  [as planned][fn_represent_c]. A later change that makes an input
  [conversion](../../stages/04-select.md#select-conversion-relations) fallible in another category must skip this function until
  the C form says where that failure goes.

[fn]: README.md
[fn_boundary]: 06-boundary.md
[fn_requests_c]: 03-requests.c.md
[fn_represent_c]: 05-represent.c.md
[fn_emit_c]: 08-emit.c.md
