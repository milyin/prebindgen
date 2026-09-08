<!-- spec: {"example": "function", "kind": "variant", "language": "c", "stage": "07-emit"} -->

# Function: stamp_sum — Emit bindings — c

[Pipeline chapter](../../stages/07-emit.md) · [Common contract](07-emit.md) · [Example path](README.md)

## Input

Retained C wrapper and aggregate declaration.

## Owner

Common Rust writer renders registry body; cbindgen derives the callable header declaration.

## Result

Expected native and header code appears below. The header's StampC definition is specified on the struct emission path.

## Checks

Compile the complete fixture module with its source and struct definitions. C caller `(StampC){12,34}` must obtain 46. Native function symbol and by-value ABI agree with the request.

## Concrete description or output

In the generated C Rust module, import the source crate and place the struct-path StampC definition alongside this wrapper:

```rust
use crate::source;

#[no_mangle]
pub extern "C" fn stamp_sum_c(arg0: StampC) -> i64 {
    let v0 = arg0.secs;
    let v1 = arg0.nanos;
    let v2 = source::Stamp { secs: v0, nanos: v1 };
    let v3 = source::stamp_sum(v2);
    v3
}
```

Following the struct definition, the C header declares:

```c
int64_t stamp_sum_c(struct StampC arg0);
```

The registry plans `let` bindings, construction, source call and return. The selected common member primitives render `arg0.secs` and `arg0.nanos`; the C adapter selects the exported name and calling convention.

## Related example

[struct: c at this stage](../struct/07-emit.c.md)
