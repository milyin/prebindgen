<!-- spec: {"kind": "cell", "example": "fn", "stage": "06-boundary"} -->

[Stage chapter](../../stages/06-boundary.md) · [Element path][fn] · [Source crate](../../source.md)
Owner: the registry, on the function form the binding stated · Previous: [Represent and compose values][fn_represent] · Next: [Retain supported output][fn_retain]

# Function taking an owned struct — Assemble the wrapper boundary

## Input

The two [nodes](../../stages/05-represent.md#represent-and-compose-values) [representation](../../stages/05-represent.md#represent-and-compose-values) produced, and the source signature they belong to:

```text
node(input)  : Stamp   IntoRust   -> an owned source Stamp
node(output) : i64     OutOfRust  -> the target's signed 64-bit carrier

source: pub fn stamp_sum(stamp: Stamp) -> i64
```

## Result

The registry now combines those reusable [conversions](../../stages/04-select.md#select-conversion-relations) with this specific source
call. The function form the binding recorded for `fn:stamp_sum` supplies the
symbol, the calling convention, any parameter the convention adds and a route
per failure category; the plan records the parameters and the order in which
generated code operates.

```text
FunctionPlan {
    symbol, abi:  from the form                   // per target, below
    params:       the form's context parameters, then one per source parameter,
                  each typed as its conversion's carrier
    ret:          the output conversion's carrier
    instrs:       convert the input
                  -> call the source function once
                  -> convert the result
    result:       the converted result, returned
}
```

## Checks

- The source function is called once, and only after every input [conversion](../../stages/04-select.md#select-conversion-relations) has
  succeeded.
- Every failure the input node declares needs a route here; a category with no
  route is an unsupported boundary, not a default.
- A value of a wire type the form's parameters or return do not hold skips the
  function. Its ABI is never quietly changed to make it fit.
- There is one ordinary result. A future `Result` example will need separate
  success/error conversions and delivery; that branching behavior is not
  implemented in this increment.

## Language variants

- [C][fn_boundary_c]
- [Kotlin/JNI][fn_boundary_jni]

[fn]: README.md
[fn_represent]: 05-represent.md
[fn_retain]: 07-retain.md
[fn_boundary_c]: 06-boundary.c.md
[fn_boundary_jni]: 06-boundary.jni.md
