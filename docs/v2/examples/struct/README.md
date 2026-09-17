<!-- spec: {"kind": "example", "example": "struct"} -->

[Project contents](../../README.md) · [Source crate](../../source.md)

# Record with scalar fields

```rust
pub struct Stamp {
    pub secs: i64,
    pub nanos: i64,
}
```

This walkthrough follows the type that the function example accepts. A
**record** here is a struct whose fields Flat can inspect. `Stamp` has two
signed 64-bit fields; a foreign caller needs its own way to hold those values.
C uses a generated C-compatible struct passed by value. Kotlin uses a generated
data class whose JVM getters the native
[wrapper](../../stages/06-boundary.md#assemble-the-native-boundary) can call.

The key question is how either foreign [representation](../../stages/05-represent.md#represent-and-compose-values) becomes the original Rust
`Stamp`. For each field, the target describes a read operation, and the registry
combines those reads with field [conversions](../../stages/04-select.md#select-conversion-relations) and Rust construction. C member
reads cannot fail; JNI getter calls can. The representation pages explain both
operations; the contracts planned for more complex values are collected on
[the extensions page](../../extensions.md).

Follow the pages below from capture to the emitted type. There is no separate
native-boundary page for a record: a type is not a callable entry point. The
record crosses the native boundary as an argument of
[the function that takes it][fn].

Deliberately not covered: a field whose type is itself a record, an optional
field and a sequence field (`struct_nested`, `struct_option_field`,
`struct_vec_field`), each of which changes what a child conversion is allowed to
produce and would get its own cells.

1. [Capture source items][struct_source]
2. [Build and inspect the source model][struct_flat]
3. [Record binding requests][struct_requests] · [C][struct_requests_c] · [Kotlin/JNI][struct_requests_jni]
4. [Select conversion relations][struct_select] · [C][struct_select_c] · [Kotlin/JNI][struct_select_jni]
5. [Represent and compose values][struct_represent] · [C][struct_represent_c] · [Kotlin/JNI][struct_represent_jni]
6. [Retain supported output][struct_retain]
7. [Emit bindings][struct_emit] · [C][struct_emit_c] · [Kotlin/JNI][struct_emit_jni]

[fn]: ../fn/README.md
[struct_source]: 01-source.md
[struct_flat]: 02-flat.md
[struct_requests]: 03-requests.md
[struct_requests_c]: 03-requests.c.md
[struct_requests_jni]: 03-requests.jni.md
[struct_select]: 04-select.md
[struct_select_c]: 04-select.c.md
[struct_select_jni]: 04-select.jni.md
[struct_represent]: 05-represent.md
[struct_represent_c]: 05-represent.c.md
[struct_represent_jni]: 05-represent.jni.md
[struct_retain]: 07-retain.md
[struct_emit]: 08-emit.md
[struct_emit_c]: 08-emit.c.md
[struct_emit_jni]: 08-emit.jni.md
