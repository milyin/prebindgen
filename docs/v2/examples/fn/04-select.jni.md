<!-- spec: {"kind": "variant", "example": "fn", "stage": "04-select", "language": "jni"} -->

[Stage chapter](../../stages/04-select.md) · [Common cell][fn_select] · [Element path][fn]
Owner: the registry, from the rules the JNI frontend recorded

# Function taking an owned struct — Select conversion relations — Kotlin/JNI

## Input

The two [crossings](../../stages/03-requests.md#finding-an-existing-conversion-plan), and the rules the JNI frontend recorded for their types:

```text
Crossing { source: Stamp, direction: IntoRust  }   Type(Stamp), into Rust -> Parts through Fields, a `JObject` of `example.Stamp`
Crossing { source: i64,   direction: OutOfRust }   Type(i64), out of Rust -> Whole over `jlong`, identity
```

## Result

```text
param stamp   Stamp, IntoRust   -> Stamp.fields, carried in a `JObject`
  field secs  i64,   IntoRust   -> atomic,       carried in `jlong`
  field nanos i64,   IntoRust   -> atomic,       carried in `jlong`
return        i64,   OutOfRust  -> atomic,       carried in `jlong`
```

`Stamp` is read through its fields because the build script declared it with
`data_class!`: a Kotlin `Stamp` object exposes its fields as properties, and
the [conversion](../../stages/04-select.md#select-conversion-relations) into a
Rust `Stamp` is made of reading each of them. `ptr_class!` would record a
`Whole` over a `jlong` address instead, which would leave the fields
unread. As in C, the rule decides the
[relation](../../stages/04-select.md#what-a-relation-is) before the registry
has read any field, and no JNI code runs to decide it.

The scalars take the `i64` rules, one per direction: a `Whole` over a `jlong`
[wire type](../../stages/05-represent.md#describing-target-values-and-operations),
one JNI scalar type, which needs no parts.

## Checks

- The relation is resolved by the registry from the rule.
- A JVM object can have only `Long` parts — what a getter returning a `long`
  reads — and both fields resolve to one. The rule's `read` is a getter,
  a JVM call that can fail — a consequence stated here and paid for
  [there][fn_represent_jni].
- The selection for `Stamp` is the same one [the struct's JNI page][struct_select_jni]
  shows; this function's parameter reuses it.

[fn]: README.md
[fn_select]: 04-select.md
[fn_represent_jni]: 05-represent.jni.md
[struct_select_jni]: ../struct/04-select.jni.md
