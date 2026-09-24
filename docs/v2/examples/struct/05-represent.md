<!-- spec: {"kind": "cell", "example": "struct", "stage": "05-represent"} -->

[Stage chapter](../../stages/05-represent.md) · [Element path][struct] · [Source crate](../../source.md)
Owner: the registry, from the binding's rules · Previous: [Select conversion relations][struct_select] · Next: [Retain supported output][struct_retain]

# Struct with scalar fields — Represent and compose values

## Input

The [selection][struct_select] for the struct, with its leaves already planned:

```text
Stamp, IntoRust, relation Stamp.fields
  +-- secs  -> node(i64, IntoRust)     // finished: identity conversion
  +-- nanos -> node(i64, IntoRust)     // the same node
representation: the Type(Stamp) rule's — Product { via: Fields, wire_type, read }
```

## Result

```text
node(Stamp, IntoRust) {
    relation:       Stamp.fields
    representation: the Stamp rule's
    wire type:        this target's Stamp wire type
    children:       [ node(i64, IntoRust) as "secs",
                      node(i64, IntoRust) as "nanos" ]
    body:           apply read to wire type, for secs  -> convert -> local
                    apply read to wire type, for nanos -> convert -> local
                    construct source::Stamp { secs, nanos }
}
```

With both children finished, the registry composes the struct from its
[representation](../../stages/05-represent.md#represent-and-compose-values):
the [wire type](../../stages/05-represent.md#describing-target-values-and-operations)
that holds a `Stamp` on the target's side — a C aggregate or a JVM object
here — and the `read`
[primitive](../../stages/05-represent.md#represent-and-compose-values), applied
once per part. The [C][struct_represent_c] and [Kotlin/JNI][struct_represent_jni]
pages show those reads. Each read's result is passed through the child's
template, and the converted parts construct the source struct.

Both field positions resolve to the same cached `i64` input
[node](../../stages/05-represent.md#represent-and-compose-values); the
[wrapper](../../stages/06-boundary.md#assemble-the-wrapper-boundary) applies
that plan once per field. Each read already produces the Rust `i64` value, so
the child [conversion](../../stages/04-select.md#select-conversion-relations)
is **identity**: it passes the value through without generating another
operation. The owned result borrows neither input.

## Checks

- Each member's wire type is of a kind the `Stamp` wire type's kind can have
  as a part; a part resolving to any other kind refuses the struct where the
  member is.
- Construction follows declaration order, and a read that fails stops the body
  before the construction: no `Stamp` is built from a partial set of fields.
- The node is recorded only once its children exist, so its identity — type,
  direction, representation, children — is complete when it is cached and
  another struct with the same identity shares it.
- This is the struct entering Rust. Leaving Rust needs a construction
  operation a `Product` does not have, and is a reported skip.

## Language variants

- [C][struct_represent_c]
- [Kotlin/JNI][struct_represent_jni]

[struct]: README.md
[struct_select]: 04-select.md
[struct_retain]: 07-retain.md
[struct_represent_c]: 05-represent.c.md
[struct_represent_jni]: 05-represent.jni.md
