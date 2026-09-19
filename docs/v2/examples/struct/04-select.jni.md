<!-- spec: {"kind": "variant", "example": "struct", "stage": "04-select", "language": "jni"} -->

[Stage chapter](../../stages/04-select.md) · [Common cell][struct_select] · [Element path][struct]
Owner: the registry; the JNI adapter selects from the offered [relations](../../stages/04-select.md#what-a-relation-is)

# Struct with scalar fields — Select conversion relations — Kotlin/JNI

## Input

```text
Crossing { source: Stamp, direction: IntoRust }
policy:   DataClass { class: "example.Stamp" }
offered:  [ Stamp.fields, atomic ]
```

## Result

```text
Stamp.fields
```

and, for each part the registry then plans:

```text
secs:  i64, IntoRust   policy: jlong carrier   offered: [ atomic ]   -> atomic
nanos: i64, IntoRust   policy: jlong carrier   offered: [ atomic ]   -> atomic
```

A `DataClass` [policy](../../stages/03-requests.md#what-policy-means) says the Kotlin side holds `Stamp` as a data class whose
properties mirror the fields, so the JNI adapter answers with the struct
[relation](../../stages/04-select.md#what-a-relation-is): the
[conversion](../../stages/04-select.md#select-conversion-relations) into a
Rust `Stamp` is made of reading one property per field. The answer follows from
the policy alone. The adapter
has not yet derived a getter name or a descriptor; those belong to the
[representation](../../stages/05-represent.md#represent-and-compose-values), and the representation is asked for only after the parts are
planned.

A pointer-class policy — the struct kept in Rust and handed to Kotlin as an
opaque `jlong` handle — would make the same adapter answer `atomic`, and no
property would ever be read.

## Checks

- The answer is one of the ids the registry offered.
- Under `DataClass` the adapter has committed to one property read per part in
  the next stage, and since a property read is a JVM call, it has also
  committed this conversion to being fallible — a fact
  [the getters][struct_represent_jni] make explicit and the boundary routes.
- The scalar parts select `atomic`: a `jlong` is one JNI value, and an `i64`
  has no other relation to offer.

[struct]: README.md
[struct_select]: 04-select.md
[struct_represent_jni]: 05-represent.jni.md
