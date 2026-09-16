<!-- spec: {"kind": "example", "example": "fn"} -->

[Project contents](../../README.md) · [Source crate](../../source.md)

# Function taking an owned record

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
struct; Kotlin supplies one JVM object. Each is one native argument, but the
generated code must read its fields and reconstruct the source Rust struct.
Both targets then call `stamp_sum` exactly once and return the integer.

Read the numbered pages in order to see each intermediate result. The
[record walkthrough][struct] explains the reusable `Stamp`
[conversion](../../stages/04-values.md#plan-value-conversions) in more detail.
This function walkthrough explains how that conversion fits into an exported
call: selecting the function, assigning native arguments and results, and
handling failure. JNI getter calls can fail, so its [wrapper](../../stages/05-boundary.md#assemble-the-native-boundary) needs an error path
that the C member reads do not need.

Deliberately not covered: a `Result` return, a borrowed parameter, a callback
argument and a non-scalar result. Each is a sub-variant of this path
(`fn_fallible`, `fn_borrowed_param`, `fn_callback`, `fn_complex_return`) and gets
its own cells when it is specified.

1. [Capture source items][fn_source]
2. [Build and inspect the source model][fn_flat]
3. [Record binding requests][fn_requests] · [C][fn_requests_c] · [Kotlin/JNI][fn_requests_jni]
4. [Plan value conversions][fn_values] · [C][fn_values_c] · [Kotlin/JNI][fn_values_jni]
5. [Assemble the native boundary][fn_boundary] · [C][fn_boundary_c] · [Kotlin/JNI][fn_boundary_jni]
6. [Retain supported output][fn_retain]
7. [Emit bindings][fn_emit] · [C][fn_emit_c] · [Kotlin/JNI][fn_emit_jni]

[struct]: ../struct/README.md
[fn_source]: 01-source.md
[fn_flat]: 02-flat.md
[fn_requests]: 03-requests.md
[fn_requests_c]: 03-requests.c.md
[fn_requests_jni]: 03-requests.jni.md
[fn_values]: 04-values.md
[fn_values_c]: 04-values.c.md
[fn_values_jni]: 04-values.jni.md
[fn_boundary]: 05-boundary.md
[fn_boundary_c]: 05-boundary.c.md
[fn_boundary_jni]: 05-boundary.jni.md
[fn_retain]: 06-retain.md
[fn_emit]: 07-emit.md
[fn_emit_c]: 07-emit.c.md
[fn_emit_jni]: 07-emit.jni.md
