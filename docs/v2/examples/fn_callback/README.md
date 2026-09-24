<!-- spec: {"kind": "example", "example": "fn_callback"} -->

[Project contents](../../README.md) · [Source crate](../../source.md)

# Function taking a callback

```rust
pub fn stamp_each(stamp: Stamp, each: impl Fn(i64) + Send + Sync + 'static) {
    each(stamp.secs);
    each(stamp.nanos);
}
```

Imagine a C or Kotlin caller wants to be told each field of a `Stamp`, one at a
time. The Rust function above calls whatever it is given once per field, in
field order. The caller passes code, not data: a C function pointer with a
context, or a Kotlin lambda.

The parameter `each` is an `impl Fn(i64)`, a **callback**: a callable Rust
receives and may call any number of times, from any thread, for as long as it
keeps it — the bounds `Send + Sync + 'static` say exactly that. Rust cannot call
C or Kotlin code directly, so the generated
[wrapper](../../stages/06-boundary.md#assemble-the-wrapper-boundary) builds a
Rust closure around the foreign callable and passes the closure to
`stamp_each`. Every call of the closure converts its argument out of Rust and
calls the foreign code with it.

This path is a sub-variant of [the function path][fn]: `stamp` crosses exactly
as it does there, and the pages here concentrate on `each`. What is new is a
[conversion](../../stages/04-select.md#select-conversion-relations) whose
parts cross the other way from the value, a body that builds a closure, and
failures that happen after the wrapper has returned.

A callback argument may be any value that leaves Rust — here an `i64`; a
handle or a fieldless enum crosses the same way. A struct argument would have
to be built out of Rust, which no target does yet, so a callback taking one is
refused. Not covered: a callback that returns a value, and a callback held by
reference rather than moved into the source function.

1. [Capture source items][fn_callback_source]
2. [Build and inspect the source model][fn_callback_flat]
3. [Record binding requests][fn_callback_requests] · [C][fn_callback_requests_c] · [Kotlin/JNI][fn_callback_requests_jni]
4. [Select conversion relations][fn_callback_select] · [C][fn_callback_select_c] · [Kotlin/JNI][fn_callback_select_jni]
5. [Represent and compose values][fn_callback_represent] · [C][fn_callback_represent_c] · [Kotlin/JNI][fn_callback_represent_jni]
6. [Assemble the wrapper boundary][fn_callback_boundary] · [C][fn_callback_boundary_c] · [Kotlin/JNI][fn_callback_boundary_jni]
7. [Retain supported output][fn_callback_retain]
8. [Emit bindings][fn_callback_emit] · [C][fn_callback_emit_c] · [Kotlin/JNI][fn_callback_emit_jni]

[fn]: ../fn/README.md
[fn_callback_source]: 01-source.md
[fn_callback_flat]: 02-flat.md
[fn_callback_requests]: 03-requests.md
[fn_callback_requests_c]: 03-requests.c.md
[fn_callback_requests_jni]: 03-requests.jni.md
[fn_callback_select]: 04-select.md
[fn_callback_select_c]: 04-select.c.md
[fn_callback_select_jni]: 04-select.jni.md
[fn_callback_represent]: 05-represent.md
[fn_callback_represent_c]: 05-represent.c.md
[fn_callback_represent_jni]: 05-represent.jni.md
[fn_callback_boundary]: 06-boundary.md
[fn_callback_boundary_c]: 06-boundary.c.md
[fn_callback_boundary_jni]: 06-boundary.jni.md
[fn_callback_retain]: 07-retain.md
[fn_callback_emit]: 08-emit.md
[fn_callback_emit_c]: 08-emit.c.md
[fn_callback_emit_jni]: 08-emit.jni.md
