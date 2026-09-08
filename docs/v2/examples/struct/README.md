<!-- spec: {"kind": "example", "example": "struct"} -->

# Record with scalar fields

[Project contents](../../README.md) · [Source fixture](../../source.md)

```rust
pub struct Stamp {
    pub secs: i64,
    pub nanos: i64,
}
```

A record is not something a foreign caller can hold directly. Both targets have
to answer two questions about it — what carries its data, and through which
operations are the parts of that carrier read — and the answers differ sharply:
C copies an aggregate by value and reads its members, Kotlin/JNI passes an object
and calls its property getters through the JVM. What does not differ is the
source-side work: the same two fields, converted by the same child conversions,
assembled into the same `Stamp { secs, nanos }`.

This path is where that split is visible in the smallest possible form. It is
also where the target's operation descriptions are worked out in full — the C
member reads and the JNI getters, with their operands, failures, validity and the
one fragment each renders — because those descriptions are what makes the
registry's record traversal target-independent.

The record has no native boundary of its own: it crosses inside
[the function that takes it][fn], and that is the only cell it is missing.

Deliberately not covered: a field whose type is itself a record, an optional
field and a sequence field (`struct_nested`, `struct_option_field`,
`struct_vec_field`), each of which changes what a child conversion is allowed to
produce and would get its own cells.

1. [Capture source items][struct_source]
2. [Build and inspect the source model][struct_flat]
3. [Record binding requests][struct_requests] · [C][struct_requests_c] · [Kotlin/JNI][struct_requests_jni]
4. [Plan value conversions][struct_values] · [C][struct_values_c] · [Kotlin/JNI][struct_values_jni]
5. [Retain supported output][struct_retain]
6. [Emit bindings][struct_emit] · [C][struct_emit_c] · [Kotlin/JNI][struct_emit_jni]

[fn]: ../fn/README.md
[struct_source]: 01-source.md
[struct_flat]: 02-flat.md
[struct_requests]: 03-requests.md
[struct_requests_c]: 03-requests.c.md
[struct_requests_jni]: 03-requests.jni.md
[struct_values]: 04-values.md
[struct_values_c]: 04-values.c.md
[struct_values_jni]: 04-values.jni.md
[struct_retain]: 06-retain.md
[struct_emit]: 07-emit.md
[struct_emit_c]: 07-emit.c.md
[struct_emit_jni]: 07-emit.jni.md
