<!-- spec: {"kind": "variant", "example": "fn", "stage": "05-boundary", "language": "c"} -->

[Stage chapter](../../stages/05-boundary.md) · [Common cell][fn_boundary] · [Element path][fn]

# Function taking an owned record — Assemble the native boundary — C

## Input

The function plan's two nodes and the C function policy from
[the C request][fn_requests_c].

## Owner

The registry, using the C adapter's `BoundarySpec`.

## Result

One exported symbol, `stamp_sum`, with the C calling convention. The aggregate
is passed by value in the first ABI position and feeds the input conversion
whole; the converted result is delivered through the native return. There are no
synthetic parameters, no out-parameters, and no failure routes, because
[both nodes are infallible][fn_values_c].

## Checks

The native signature has exactly the arguments the placement describes: one
aggregate in, one integer out. An out-parameter form would be a different
`OutputPlacement`, chosen by policy, not an implementation detail of the writer.
Since no route is declared, any later change that makes an input conversion
fallible must skip this function until the C policy declares where that failure
goes.

## Representation

```text
BoundarySpec {
  abi:    extern "C", symbol "stamp_sum",
  inputs: [ InputPlacement { native arg 0 (Stamp, by value) -> node(input) } ],
  output: OutputPlacement::Return(node(output) -> int64_t),
  failures: {},
}
```

The signature this fixes, which [emission][fn_emit_c] renders:

```rust
pub extern "C" fn stamp_sum(arg0: Stamp) -> i64
```

## Along this element

See the [common cell][fn_boundary] for the plan both targets share.

[fn]: README.md
[fn_boundary]: 05-boundary.md
[fn_requests_c]: 03-requests.c.md
[fn_values_c]: 04-values.c.md
[fn_emit_c]: 07-emit.c.md
