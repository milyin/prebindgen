<!-- spec: {"kind": "cell", "example": "fn_callback", "stage": "05-represent"} -->

[Stage chapter](../../stages/05-represent.md) · [Element path][fn_callback] · [Source crate](../../source.md)
Owner: the registry, from the binding's rules · Previous: [Select conversion relations][fn_callback_select] · Next: [Assemble the wrapper boundary][fn_callback_boundary]

# Function taking a callback — Represent and compose values

## Input

The [selection][fn_callback_select] for the callback, with its argument's leaf
already planned:

```text
impl Fn(i64), IntoRust, relation callback.args
  +-- arg 0 -> node(i64, OutOfRust)    // finished: the leaf a function's i64 result uses
representation: the callback rule's — Callback { wire type, capture, invoke, routes }
```

## Result

```text
node(each) {
    crossing:       impl Fn(i64), IntoRust
    relation:       callback.args
    representation: the callback rule's
    wire type:        this target's callable wire type
    children:       [ node(i64, OutOfRust) as "arg 0" ]
    body:           apply capture to wire type -> captured
                    closure, taking captured, over (a0: i64) {
                        convert a0 -> wire          // the child's template
                        apply invoke to captured, wire
                    }
    failures:       what capture raises
}
```

The [node](../../stages/05-represent.md#represent-and-compose-values)'s body
does two things at two different times. `capture` runs once, where the callable
enters Rust, on the [wire type](../../stages/05-represent.md#describing-target-values-and-operations)
it arrived in. The closure is what the source function receives, and its body
runs on every call: the child's template turns the argument into its wire form,
and `invoke` hands that to the foreign callable together with what `capture`
kept. The closure takes the captured value with it — a `move` closure — so it
owns everything a call needs and borrows nothing from the [wrapper](../../stages/06-boundary.md#assemble-the-wrapper-boundary), which has
returned by the time a stored callback is called.

A failure has two places to happen, and the node keeps them apart. What
`capture` raises is the node's own, and the
[wrapper](../../stages/06-boundary.md#assemble-the-wrapper-boundary) routes it
like a getter's. What the child's [conversion](../../stages/04-select.md#select-conversion-relations) or `invoke` raises happens inside
a call, and takes the callback's own routes there; it is not among the node's
failures, because no wrapper is running to route it.

## Checks

- Before the closure is built, the registry checks that each argument's wire type
  is of a kind the callable's kind can have as a part, that every failure a call can raise has
  a route of the callback's own, and that nothing in a call asks for a runtime
  context. Each failed check refuses the callback, not the call.
- The closure's identity is the node's: a second function taking
  `impl Fn(i64)` under the same rule reuses this node and inlines the same
  body.
- The argument conversion is inside the closure, not before it: a handle
  argument is handed out on each call, never while the wrapper runs.

## Language variants

- [C][fn_callback_represent_c]
- [Kotlin/JNI][fn_callback_represent_jni]

[fn_callback]: README.md
[fn_callback_select]: 04-select.md
[fn_callback_boundary]: 06-boundary.md
[fn_callback_represent_c]: 05-represent.c.md
[fn_callback_represent_jni]: 05-represent.jni.md
