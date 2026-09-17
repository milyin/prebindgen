<!-- spec: {"kind": "cell", "example": "fn", "stage": "06-boundary"} -->

[Stage chapter](../../stages/06-boundary.md) · [Element path][fn] · [Source crate](../../source.md)
Owner: the registry, on the target's native interface · Previous: [Represent and compose values][fn_represent] · Next: [Retain supported output][fn_retain]

# Function taking an owned record — Assemble the native boundary

## Input

The two [nodes](../../stages/05-represent.md#represent-and-compose-values) [representation](../../stages/05-represent.md#represent-and-compose-values) produced, and the source signature they belong to:

```text
node(input)  : Stamp   IntoRust   -> an owned source Stamp
node(output) : i64     OutOfRust  -> the target's signed 64-bit carrier

source: pub fn stamp_sum(stamp: Stamp) -> i64
```

## Result

The registry now combines those reusable [conversions](../../stages/04-select.md#select-conversion-relations) with this specific source
call. The following is a conceptual summary, not the exact Rust fields of
`FunctionPlan`. The boundary supplies the target's symbol, calling convention
and destinations; the body records the order in which generated code operates.

```text
FunctionPlan {
    source:   CalleeId(crate::source::stamp_sum),
    inputs:   [ node(input) ],                    // in source parameter order
    output:   FunctionOutput::Single(node(output)),
    boundary: BoundarySpec { … },                 // per target, below
    body:     convert the input
              -> call the source function once
              -> convert the result
              -> deliver through the configured destination
}
```

## Checks

- The source function is called once, and only after every input [conversion](../../stages/04-select.md#select-conversion-relations) has
  succeeded.
- Every failure the input node declares needs a route here; a category with no
  route is an unsupported boundary, not a default.
- An unsupported destination skips the function. Its ABI is never quietly
  changed to make it fit.
- `Single` in this sketch means one ordinary result. A future `Result` example
  will need separate success/error conversions and delivery; that branching
  behavior is not implemented in this increment.

## Language variants

- [C][fn_boundary_c]
- [Kotlin/JNI][fn_boundary_jni]

[fn]: README.md
[fn_represent]: 05-represent.md
[fn_retain]: 07-retain.md
[fn_boundary_c]: 06-boundary.c.md
[fn_boundary_jni]: 06-boundary.jni.md
