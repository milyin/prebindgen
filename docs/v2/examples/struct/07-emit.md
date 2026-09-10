<!-- spec: {"kind": "cell", "example": "struct", "stage": "07-emit"} -->

[Stage chapter](../../stages/07-emit.md) · [Element path][struct] · [Source crate](../../source.md)
Owner: the common Rust writer, then `cbindgen` or the Kotlin writer · Previous: [Retain supported output][struct_retain]

# Record with scalar fields — Emit bindings

## Input

The frozen public declaration for the record and the representation payload the
target attached to it.

## Result

One public type per target, carrying no conversion code — the conversions live
in the wrapper of [the function that uses the record][fn_emit]:

| Contribution | C | Kotlin/JNI |
| --- | --- | --- |
| Rust declaration | `#[repr(C)] pub struct Stamp` | none; the value is a JVM object |
| Foreign declaration | header `typedef struct Stamp` | `data class Stamp(val secs: Long, val nanos: Long)` |
| Member access used by the conversions | Rust field reads | JVM getters `getSecs`/`getNanos` |

Rendered: [C][struct_emit_c], [Kotlin/JNI][struct_emit_jni].

## Checks

- Both fields are emitted, in declaration order, with types matching the chosen
  representation.
- The member and property names are the ones [the record's operations
  read][struct_values], because both come from the same retained metadata.
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
