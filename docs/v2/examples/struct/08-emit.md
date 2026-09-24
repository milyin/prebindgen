<!-- spec: {"kind": "cell", "example": "struct", "stage": "08-emit"} -->

[Stage chapter](../../stages/08-emit.md) · [Element path][struct] · [Source crate](../../source.md)
Owner: the common Rust writer, then `cbindgen` or the Kotlin writer · Previous: [Retain supported output][struct_retain]

# Struct with scalar fields — Emit bindings

## Input

The retained output for this struct:

```text
Retained { output: type:Stamp, output_value: node(Stamp, IntoRust) }
  carrier:  the one node(Stamp, IntoRust) resolved to, with its members
  metadata: <none for C, the data class for JNI>
```

## Result

The generated declaration gives foreign callers a way to hold the two values.
It does not itself contain the Rust
[conversion](../../stages/04-select.md#select-conversion-relations). That work is
inlined into the [wrapper](../../stages/06-boundary.md#assemble-the-wrapper-boundary)
of [the function that uses the struct][fn_emit]. This distinction explains why
C needs a generated Rust ABI struct while Kotlin needs a JVM class:

| Contribution | C | Kotlin/JNI |
| --- | --- | --- |
| Rust declaration | `#[repr(C)] pub struct Stamp` | none; the value is a JVM object |
| Foreign declaration | header `typedef struct Stamp` | `data class Stamp(val secs: Long, val nanos: Long)` |
| Member access used by the conversions | Rust field reads | JVM getters `getSecs`/`getNanos` |

Rendered: [C][struct_emit_c], [Kotlin/JNI][struct_emit_jni].

## Checks

- Both fields are emitted, in declaration order, with types matching the chosen
  [representation](../../stages/05-represent.md#represent-and-compose-values).
- The member and property names match [the struct's operations][struct_represent]:
  the adapter derives both declarations and reads from the same source fields.
- A declaration is emitted only if it was retained; the writer adds nothing.

## Language variants

- [C][struct_emit_c]
- [Kotlin/JNI][struct_emit_jni]

[struct]: README.md
[struct_retain]: 07-retain.md
[struct_represent]: 05-represent.md
[struct_emit_c]: 08-emit.c.md
[struct_emit_jni]: 08-emit.jni.md
[fn_emit]: ../fn/08-emit.md
