<!-- spec: {"kind": "example", "example": "fn"} -->

[Project contents](../../README.md) · [Source crate](../../source.md)

# Function taking an owned record

```rust
pub fn stamp_sum(stamp: Stamp) -> i64 {
    stamp.secs.wrapping_add(stamp.nanos)
}
```

This is the smallest exported callable that is not trivial: one parameter that is
an owned record, and a scalar result. It is enough to exercise the pipeline's
central claim — that the registry owns the recursive work and the target only
answers local questions — because the argument cannot cross the boundary as one
value in either target. C flattens it into an aggregate whose members are read;
Kotlin/JNI passes an object whose properties are read through the JVM. Both routes
run the same source-side plan: obtain two field values, construct `Stamp`, call
`stamp_sum` once, deliver its `i64`.

The record half of that work belongs to [the record path][struct], which this path
depends on at value planning. What is specific here is everything around the
conversion: the request that names the exported function, the boundary that maps
native arguments and the return, and the failure routes — which is where the two
targets diverge most, because the JNI property reads can fail and the C member
reads cannot.

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
