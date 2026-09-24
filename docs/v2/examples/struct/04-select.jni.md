<!-- spec: {"kind": "variant", "example": "struct", "stage": "04-select", "language": "jni"} -->

[Stage chapter](../../stages/04-select.md) · [Common cell][struct_select] · [Element path][struct]
Owner: the registry, from the rule the JNI frontend recorded for `data_class!(Stamp)`

# Struct with scalar fields — Select conversion relations — Kotlin/JNI

## Input

```text
Crossing { source: Stamp, direction: IntoRust }
position: wherever this Stamp sits
rules:    Type(Stamp) -> Product { via: Fields, wire_type: stamp_obj, read: JniOp::Getter (env, runtime failure) }
          Type(i64)   -> Terminal { wire_type: jlong, identity both ways }
wire types: stamp_obj = JniWireType::Object { kotlin_class: "example.Stamp" }
            jlong     = JniWireType::Long
relations of Stamp: [ Stamp.fields, atomic ]
```

## Result

```text
Stamp.fields, carried in stamp_obj
secs:  i64, IntoRust   Type(i64) rule -> atomic
nanos: i64, IntoRust   Type(i64) rule -> atomic
```

A data class [choice](../../stages/03-requests.md#what-a-choice-records) says the Kotlin side holds `Stamp` as a data class whose
properties mirror the fields, so the JNI frontend records a `Product` through
`Via::Fields` over a `JObject` [wire type](../../stages/05-represent.md#describing-target-values-and-operations) whose metadata names `example.Stamp`.
The registry resolves that to the struct
[relation](../../stages/04-select.md#what-a-relation-is): the
[conversion](../../stages/04-select.md#select-conversion-relations) into a
Rust `Stamp` is made of reading one property per field. No getter name or
descriptor exists yet; the JNI writer derives both when the registry feeds it
the part and the part's `jlong` wire type, at
[composition][struct_represent_jni].

`ptr_class!(Stamp)` — the struct kept in Rust and handed to Kotlin as an
opaque `jlong` handle — records a `Terminal` over a `jlong` wire type instead,
and no property would ever be read.

## Checks

- The relation is resolved by the registry from the rule; the JNI target never
  sees a relation id.
- The rule's `read` is fallible in the runtime category, since a property read
  is a JVM call, so this conversion is fallible — known now, before anything
  is written, and routed by the boundary.
- The scalar parts take the `i64` rule from JNI's scalar table: a `Terminal`
  over `jlong`, one JNI value.

[struct]: README.md
[struct_select]: 04-select.md
[struct_represent_jni]: 05-represent.jni.md
