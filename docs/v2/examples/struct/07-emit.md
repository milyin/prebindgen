<!-- spec: {"kind": "cell", "example": "struct", "stage": "07-emit"} -->

[Stage chapter](../../stages/07-emit.md) · [Element path][struct] · [Source crate](../../source.md)
Owner: the common Rust writer, then `cbindgen` or the Kotlin writer · Previous: [Retain supported output][struct_retain]

# Record with scalar fields — Emit bindings

## Input

The frozen public declaration for this record:

```text
SurfaceSpec { declaration: "type:Stamp",
              rust: <C aggregate artifact, or empty for JNI>,
              payload: <None for C, class metadata for JNI> }
```

## Result

The generated declaration gives foreign callers a way to hold the two values.
It does not itself contain the Rust
[conversion](../../stages/04-values.md#plan-value-conversions). That work is
inlined into the [wrapper](../../stages/05-boundary.md#assemble-the-native-boundary)
of [the function that uses the record][fn_emit]. This distinction explains why
C needs a generated Rust ABI struct while Kotlin needs a JVM class:

| Contribution | C | Kotlin/JNI |
| --- | --- | --- |
| Rust declaration | `#[repr(C)] pub struct Stamp` | none; the value is a JVM object |
| Foreign declaration | header `typedef struct Stamp` | `data class Stamp(val secs: Long, val nanos: Long)` |
| Member access used by the conversions | Rust field reads | JVM getters `getSecs`/`getNanos` |

Rendered: [C][struct_emit_c], [Kotlin/JNI][struct_emit_jni].

## Checks

- Both fields are emitted, in declaration order, with types matching the chosen
  [representation](../../stages/04-values.md#plan-value-conversions).
- The member and property names match [the record's operations][struct_values]:
  the adapter derives both declarations and reads from the same source fields.
- A declaration is emitted only if it was retained; the writer adds nothing.

## Language variants

- [C][struct_emit_c]
- [Kotlin/JNI][struct_emit_jni]

[struct]: README.md
[struct_retain]: 06-retain.md
[struct_values]: 04-values.md
[struct_emit_c]: 07-emit.c.md
[struct_emit_jni]: 07-emit.jni.md
[fn_emit]: ../fn/07-emit.md
