<!-- spec: {"kind": "example", "example": "fn"} -->

[Project contents](../../README.md) · [Source crate](../../source.md)

# Function taking an owned struct

```rust
pub fn stamp_sum(stamp: Stamp) -> i64 {
    stamp.secs.wrapping_add(stamp.nanos)
}
```

Imagine a C or Kotlin caller wants to add the two fields of a `Stamp`. The Rust
function above already does the calculation. This walkthrough follows the work
needed to make that function callable without hand-writing its foreign entry
points.

The parameter is **owned**: Rust receives a `Stamp` value, not a reference to
one. The result is a single signed 64-bit integer. C supplies one C-compatible
struct; Kotlin supplies one JVM object. Each is one [wrapper](../../stages/06-boundary.md#assemble-the-wrapper-boundary) argument, but the
generated code must read its fields and reconstruct the source Rust struct.
Both targets then call `stamp_sum` exactly once and return the integer.

Read the numbered pages in order to see each intermediate result. The
[struct walkthrough][struct] explains the reusable `Stamp`
[conversion](../../stages/04-select.md#select-conversion-relations) in more detail.
This function walkthrough explains how that conversion fits into an exported
call: selecting the function, assigning wrapper arguments and results, and
handling failure. JNI getter calls can fail, so its [wrapper](../../stages/06-boundary.md#assemble-the-wrapper-boundary) needs an error path
that the C member reads do not need.

Deliberately not covered: a `Result` return, a borrowed parameter, a callback
argument and a non-scalar result. Each is a sub-variant of this path
(`fn_fallible`, `fn_borrowed_param`, `fn_callback`, `fn_complex_return`) and gets
its own cells when it is specified.

1. [Capture source items][fn_source]
2. [Build and inspect the source model][fn_flat]
3. [Record binding requests][fn_requests] · [C][fn_requests_c] · [Kotlin/JNI][fn_requests_jni]
4. [Select conversion relations][fn_select] · [C][fn_select_c] · [Kotlin/JNI][fn_select_jni]
5. [Represent and compose values][fn_represent] · [C][fn_represent_c] · [Kotlin/JNI][fn_represent_jni]
6. [Assemble the wrapper boundary][fn_boundary] · [C][fn_boundary_c] · [Kotlin/JNI][fn_boundary_jni]
7. [Retain supported output][fn_retain]
8. [Emit bindings][fn_emit] · [C][fn_emit_c] · [Kotlin/JNI][fn_emit_jni]

[struct]: ../struct/README.md
[fn_source]: 01-source.md
[fn_flat]: 02-flat.md
[fn_requests]: 03-requests.md
[fn_requests_c]: 03-requests.c.md
[fn_requests_jni]: 03-requests.jni.md
[fn_select]: 04-select.md
[fn_select_c]: 04-select.c.md
[fn_select_jni]: 04-select.jni.md
[fn_represent]: 05-represent.md
[fn_represent_c]: 05-represent.c.md
[fn_represent_jni]: 05-represent.jni.md
[fn_boundary]: 06-boundary.md
[fn_boundary_c]: 06-boundary.c.md
[fn_boundary_jni]: 06-boundary.jni.md
[fn_retain]: 07-retain.md
[fn_emit]: 08-emit.md
[fn_emit_c]: 08-emit.c.md
[fn_emit_jni]: 08-emit.jni.md
