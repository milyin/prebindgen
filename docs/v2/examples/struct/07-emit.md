<!-- spec: {"kind": "cell", "example": "struct", "stage": "07-emit"} -->

# Record with scalar fields — Emit bindings

[Stage chapter](../../stages/07-emit.md) · [Element path][struct] · [Source fixture](../../source.md)

## Input

The frozen public declaration for the record and the representation payload the
target attached to it.

## Owner

The common Rust writer renders whatever Rust the representation needs; the
foreign declaration comes from `cbindgen` for C and from the Kotlin writer for
JNI.

## Result

One public type per target: a `repr(C)` Rust struct that `cbindgen` turns into a
header `typedef`, or a Kotlin data class. Neither carries any conversion code —
the conversions live in the wrapper of [the function that uses the record][fn_emit],
which is the only place a `Stamp` is actually built.

## Checks

The emitted type has both fields, in declaration order, with types matching the
representation the policy chose. The member and property names are the ones
[the record's operations read][struct_values], because both come from the same
retained metadata. A declaration is emitted only if it was retained; the writer
adds nothing of its own.

## Representation

| Contribution | C | Kotlin/JNI |
| --- | --- | --- |
| Rust declaration | `#[repr(C)] pub struct Stamp` | none needed; the object is a JVM value |
| Foreign declaration | header `typedef struct Stamp` | `data class Stamp(val secs: Long, val nanos: Long)` |
| Member access used by conversions | Rust field reads | JVM getters `getSecs`/`getNanos` |

## Language variants

- [C][struct_emit_c]
- [Kotlin/JNI][struct_emit_jni]

## Along this element

Previous: [Retain supported output][struct_retain]

[struct]: README.md
[struct_retain]: 06-retain.md
[struct_values]: 04-values.md
[struct_emit_c]: 07-emit.c.md
[struct_emit_jni]: 07-emit.jni.md
[fn_emit]: ../fn/07-emit.md
