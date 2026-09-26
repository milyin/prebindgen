<!-- spec: {"kind": "cell", "example": "fn_callback", "stage": "04-select"} -->

[Stage chapter](../../stages/04-select.md) · [Element path][fn_callback] · [Source crate](../../source.md)
Owner: the registry, from the binding's rules · Previous: [Record binding requests][fn_callback_requests] · Next: [Represent and compose values][fn_callback_represent]

# Function taking a callback — Select conversion relations

## Input

```text
Crossing { source: Stamp,                                  direction: IntoRust }   // at `param stamp`
Crossing { source: impl Fn(i64) + Send + Sync + 'static,  direction: IntoRust }   // at `param each`
rules:    Type(Stamp)        -> this target's Stamp representation, read through its fields
          Type(impl Fn(i64)) -> this target's callback representation
          Type(i64)          -> this target's i64 representation, whole
```

## Result

```text
   param stamp --> Stamp, IntoRust, relation Stamp.fields    // as in the function path
                     +-- field secs  --> i64, IntoRust, atomic
                     +-- field nanos --> i64, IntoRust, atomic

   param each  --> impl Fn(i64), IntoRust
                     relation: callback.args                 // what a `Callable` names
                     +-- arg 0 --> i64, OutOfRust, atomic
```

The callback's [representation](../../stages/05-represent.md#represent-and-compose-values)
is a `Callable`, and the
[relation](../../stages/04-select.md#what-a-relation-is) a `Callable` names is
the callback's arguments: one positional part per argument, `arg 0` here. The
registry descends into it as it descends into a struct's fields, with one
difference. The callable enters Rust, and Rust hands the argument to it, so the
argument's [crossing](../../stages/03-requests.md#finding-an-existing-conversion-plan)
is the other way: an `i64` leaving Rust, which is exactly the crossing
[the function path's result][fn_select] takes, and plans as the same leaf.

## Checks

- The direction of `arg 0` follows from the relation, and the rule it takes is
  an out-of-Rust one. A rule at `param each.arg 0` naming an into-Rust
  representation fails the build.
- The argument is looked up at its own position first, `param each.arg 0`: a
  rule there would give this callback's argument a representation of its own
  without touching any other `i64`.
- An argument that cannot leave Rust refuses the callback, and the function
  with it: an `impl Fn(Stamp)` would need a `Stamp` built out of Rust, and is
  refused with `unsupported.struct.out_of_rust` at `param emit.arg 0`.
- A `Callable` rule on a type that is not an `impl Fn(..)` finds no relation,
  and is refused as `unsupported.type.not_a_callback`.

## Language variants

- [C][fn_callback_select_c]
- [Kotlin/JNI][fn_callback_select_jni]

[fn_callback]: README.md
[fn_callback_requests]: 03-requests.md
[fn_callback_represent]: 05-represent.md
[fn_callback_select_c]: 04-select.c.md
[fn_callback_select_jni]: 04-select.jni.md
[fn_select]: ../fn/04-select.md
