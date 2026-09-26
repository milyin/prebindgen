<!-- spec: {"kind": "variant", "example": "fn_callback", "stage": "04-select", "language": "jni"} -->

[Stage chapter](../../stages/04-select.md) · [Common cell][fn_callback_select] · [Element path][fn_callback]
Owner: the registry, from the rules the JNI frontend recorded

# Function taking a callback — Select conversion relations — Kotlin/JNI

## Input

The callback's [crossing](../../stages/03-requests.md#finding-an-existing-conversion-plan),
and the rules the JNI frontend recorded for it and its argument:

```text
Crossing { source: impl Fn(i64), direction: IntoRust }   Type(impl Fn(i64)), into Rust -> Callable over a `JObject` of example.LongCallback
                                                         Type(i64), out of Rust        -> Whole over `jlong`, identity
```

## Result

```text
param each    impl Fn(i64), IntoRust   -> callback.args, carried in a `JObject`
  arg 0       i64,          OutOfRust  -> atomic,        carried in `jlong`
```

The argument leaves Rust as a `jlong`, the
[wire type](../../stages/05-represent.md#describing-target-values-and-operations)
whose descriptor, `J`, is what the `run` method will be looked up with.
`param stamp` selects exactly as on [the function path][fn_select_jni].

## Checks

- A callable can have numbers, enums' numbers and addresses as arguments, and
  a `jlong` is a number. A `Stamp` argument would be refused
  before this check, since no JNI [conversion](../../stages/04-select.md#select-conversion-relations) builds one out of Rust.
- A handle argument would resolve to a `jlong` of the handle class, and an
  enum to a `jint`: both accepted, and both what makes the frontend add a raw
  interface for the callback.

[fn_callback]: README.md
[fn_callback_select]: 04-select.md
[fn_select_jni]: ../fn/04-select.jni.md
