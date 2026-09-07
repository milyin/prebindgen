<!-- spec: {"example": "struct", "kind": "cell", "stage": "07-emit"} -->

# Struct: Stamp — Emit bindings

[Pipeline chapter](../../stages/07-emit.md) · [Example path](README.md)

## Input

The frozen public type and target representation identify names, field order, carrier types and public type dependencies.

## Owner

The common Rust writer plus cbindgen emit the C representation; JNI's Kotlin writer emits the Kotlin class.

## Result

Emit the two-field public data type shown in the language pages. Source::Stamp itself is supplied by the source crate; writers do not duplicate that source definition. Primitive code that reads members/getters appears inside the function wrapper at each application.

## Checks

Public member names, types and getter descriptors must agree with the retained representation. A public type has no source-call side effect. Header layout and JVM signatures are part of the declared interface, while exact whitespace and temporary names are not.

## Language variants

- [c](07-emit.c.md)
- [kotlin](07-emit.kotlin.md)

## Along this example

[Previous](06-retain.md)

## Related example

[function at this stage](../function/07-emit.md)
