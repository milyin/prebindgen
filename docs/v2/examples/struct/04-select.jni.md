<!-- spec: {"kind": "variant", "example": "struct", "stage": "04-select", "language": "jni"} -->

[Stage chapter](../../stages/04-select.md) · [Common cell][struct_select] · [Element path][struct]
Owner: the registry, from the rule the JNI frontend recorded for `data_class!(Stamp)`

# Struct with scalar fields — Select conversion relations — Kotlin/JNI

## Input

```text
Crossing { source: Stamp, direction: IntoRust }
position: wherever this Stamp sits
rules:    Type(Stamp) -> Conversion { via: Fields, choice: JniChoice::DataClass { class: "example.Stamp" } }
relations of Stamp: [ Stamp.fields, atomic ]
```

## Result

```text
Stamp.fields, conversion JniChoice::DataClass { class: "example.Stamp" }
```

and, for each part the registry then plans:

```text
secs:  i64, IntoRust   Default rule: Whole, JniChoice::Scalar   -> atomic
nanos: i64, IntoRust   Default rule: Whole, JniChoice::Scalar   -> atomic
```

A `DataClass` [choice](../../stages/03-requests.md#what-a-choice-records) says the Kotlin side holds `Stamp` as a data class whose
properties mirror the fields, so the JNI frontend records its rule with
`Via::Fields`, and the registry resolves that to the struct
[relation](../../stages/04-select.md#what-a-relation-is): the
[conversion](../../stages/04-select.md#select-conversion-relations) into a
Rust `Stamp` is made of reading one property per field. Nothing has derived a
getter name or a descriptor yet; those belong to the
[representation](../../stages/05-represent.md#represent-and-compose-values), and the representation is asked for only after the parts are
planned.

`ptr_class!(Stamp)` — the struct kept in Rust and handed to Kotlin as an
opaque `jlong` handle — records the same rule with `Via::Whole`, and no
property would ever be read.

## Checks

- The relation is resolved by the registry from the rule; the JNI target never
  sees a relation id.
- Under `DataClass` the target has committed to one property read per part in
  the next stage, and since a property read is a JVM call, it has also
  committed this conversion to being fallible — a fact
  [the getters][struct_represent_jni] make explicit and the boundary routes.
- The scalar parts take the binding's `Default` rule, `Whole`: a `jlong` is
  one JNI value.

[struct]: README.md
[struct_select]: 04-select.md
[struct_represent_jni]: 05-represent.jni.md
