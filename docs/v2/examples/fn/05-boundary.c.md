<!-- spec: {"kind": "variant", "example": "fn", "stage": "05-boundary", "language": "c"} -->

[Stage chapter](../../stages/05-boundary.md) · [Common cell][fn_boundary] · [Element path][fn]
Owner: the registry, on the C adapter's `BoundarySpec`

# Function taking an owned record — Assemble the native boundary — C

## Input

```text
node(input)  : produces an owned source Stamp, failures {}
node(output) : produces c_i64, failures {}

policy (C function): symbol "stamp_sum", extern "C",
                     input by value, output through the native return
```

## Result

```text
BoundarySpec {
    abi:      extern "C", symbol "stamp_sum",
    inputs:   [ InputPlacement { native arg 0 (Stamp, by value) -> node(input) } ],
    output:   OutputPlacement::Return(node(output) -> int64_t),
    failures: {},
}
```

`native arg 0` is the first C argument. The [wrapper](../../stages/05-boundary.md#assemble-the-native-boundary) uses it to construct the
source `Stamp`, then returns the source function's integer result directly.
The empty failure set means these [conversions](../../stages/04-values.md#plan-value-conversions) need no error branch. An
**out-parameter**, by contrast, would be caller-provided storage that the wrapper
writes into; this example does not use one.

These choices determine the signature that [emission][fn_emit_c] renders:

```rust
#[no_mangle]
pub extern "C" fn stamp_sum(stamp: Stamp) -> i64
```

## Checks

- One aggregate in, one integer out: no synthetic parameters, no out-parameter.
  The symbol and calling convention come from the recorded
  [policy](../../stages/03-requests.md#what-policy-means),
  [as the request cell shows][fn_requests_c].
- An out-parameter form would require an additional delivery implementation;
  the current V2 increment supports only a native return or no value.
- No route is declared because both
  [nodes](../../stages/04-values.md#plan-value-conversions) are infallible,
  [as planned][fn_values_c]. A later change that makes an input
  [conversion](../../stages/04-values.md#plan-value-conversions) fallible must skip this function until
  the C policy says where that failure goes.

[fn]: README.md
[fn_boundary]: 05-boundary.md
[fn_requests_c]: 03-requests.c.md
[fn_values_c]: 04-values.c.md
[fn_emit_c]: 07-emit.c.md
