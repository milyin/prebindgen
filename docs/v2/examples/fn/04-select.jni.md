<!-- spec: {"kind": "variant", "example": "fn", "stage": "04-select", "language": "jni"} -->

[Stage chapter](../../stages/04-select.md) · [Common cell][fn_select] · [Element path][fn]
Owner: the registry, on the JNI adapter's selections

# Function taking an owned struct — Select conversion relations — Kotlin/JNI

## Input

The two [crossings](../../stages/03-requests.md#finding-an-existing-conversion-plan), with the JNI [policy](../../stages/03-requests.md#what-policy-means) recorded for them and the [relations](../../stages/04-select.md#what-a-relation-is) the registry offers:

```text
Crossing { source: Stamp, direction: IntoRust  }   policy: DataClass { class: "example.Stamp" }
                                                    offered: [ Stamp.fields, atomic ]
Crossing { source: i64,   direction: OutOfRust }   policy: jlong carrier
                                                    offered: [ atomic ]
```

## Result

```text
Param(0)  Stamp, IntoRust   -> Stamp.fields
  secs    i64,   IntoRust   -> atomic
  nanos   i64,   IntoRust   -> atomic
Return    i64,   OutOfRust  -> atomic
```

The JNI adapter selects `Stamp.fields` because the struct is declared as a
data class: a Kotlin `Stamp` object exposes its fields as properties, and the
[conversion](../../stages/04-select.md#select-conversion-relations) into a
Rust `Stamp` is made of reading each of them. The same policy family has a
different answer for a type declared as a pointer class — an opaque handle in a
`jlong` — which would select `atomic` and leave the fields unread. As in C, the
adapter answers from the policy, before the registry has shown it any field.

The scalars select `atomic`; the JNI policy for an `i64` is a `jlong`
[carrier](../../stages/05-represent.md#describing-target-values-and-operations),
one JNI scalar type, which needs no parts.

## Checks

- The answer is one of the offered ids; the adapter chooses, it does not
  invent.
- The `DataClass` policy commits the adapter to a property read per part in the
  next stage, and property reads are JNI calls that can fail — a consequence
  chosen here and paid for [there][fn_represent_jni].
- The selection for `Stamp` is the same one [the struct's JNI page][struct_select_jni]
  shows; this function's parameter reuses it.

[fn]: README.md
[fn_select]: 04-select.md
[fn_represent_jni]: 05-represent.jni.md
[struct_select_jni]: ../struct/04-select.jni.md
