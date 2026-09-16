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
data class whose JVM getters the native wrapper can call.

The key question is how either foreign representation becomes the original Rust
`Stamp`. For each field, the target describes a read operation, and the registry
combines those reads with field conversions and Rust construction. C member
reads cannot fail; JNI getter calls can. The value-planning pages explain both
operations, including the additional contracts planned for more complex values.

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
