<!-- spec: {"kind": "cell", "example": "fn_callback", "stage": "06-boundary"} -->

[Stage chapter](../../stages/06-boundary.md) · [Element path][fn_callback] · [Source crate](../../source.md)
Owner: the registry, on the function form the binding stated · Previous: [Represent and compose values][fn_callback_represent] · Next: [Retain supported output][fn_callback_retain]

# Function taking a callback — Assemble the wrapper boundary

## Input

The two [nodes](../../stages/05-represent.md#represent-and-compose-values) the
function's parameters take, and the source signature:

```text
node(stamp) : Stamp        IntoRust  -> an owned source Stamp           // the function path's
node(each)  : impl Fn(i64) IntoRust  -> the closure, failures: capture's

source: pub fn stamp_each(stamp: Stamp, each: impl Fn(i64) + Send + Sync + 'static)
```

## Result

```text
FunctionPlan {
    symbol, abi:  from the form                     // per target, below
    params:       the form's context parameters, then `stamp` and `each`,
                  each typed as its conversion's carrier
    ret:          none                              // stamp_each returns nothing
    instrs:       convert stamp
                  -> capture each, build the closure
                  -> call the source function once, with both
    result:       none
}
```

The callback is placed like any input: a [wrapper](../../stages/06-boundary.md#assemble-the-wrapper-boundary) parameter typed as its
[carrier](../../stages/05-represent.md#describing-target-values-and-operations),
feeding its [conversion](../../stages/04-select.md#select-conversion-relations).
The conversion ends with a closure rather than a converted value, and the
source function receives the closure. Everything before the source call
happens once, in the
[wrapper](../../stages/06-boundary.md#assemble-the-wrapper-boundary); the calls
of the closure happen during the source call, or after it, if the source keeps
the callback.

The form routes what the wrapper can meet: the `Stamp` conversion's failures
and `capture`'s. A failure inside a call is routed by the callback's own
routes, inside the closure, and the form is never asked about it.

## Checks

- The source function is called once, after both inputs are converted; a
  failed capture stops the wrapper before the call, as a failed getter does.
- Nothing a call does is the wrapper's to route, so a form with no route for
  a call's failure category is not skipped for it: the callback's
  [representation](../../stages/05-represent.md#represent-and-compose-values) is where that route is stated, and checked.
- A function returning nothing delivers nothing: the wrapper has no return
  value, and a route that terminates it returns `()`.

## Language variants

- [C][fn_callback_boundary_c]
- [Kotlin/JNI][fn_callback_boundary_jni]

[fn_callback]: README.md
[fn_callback_represent]: 05-represent.md
[fn_callback_retain]: 07-retain.md
[fn_callback_boundary_c]: 06-boundary.c.md
[fn_callback_boundary_jni]: 06-boundary.jni.md
