<!-- spec: {"example": "struct", "kind": "variant", "language": "kotlin", "stage": "03-requests"} -->

# Struct: Stamp — Record binding requests — kotlin

[Pipeline chapter](../../stages/03-requests.md) · [Common contract](03-requests.md) · [Example path](README.md)

## Input

Public Stamp declaration and owned record-input rule.

## Owner

JNI frontend records choices interpreted by JNI adapter.

## Result

Choose `example.Stamp`, Kotlin data class with `secs: Long` and `nanos: Long`, and JVM-object native input. Retain getter metadata getSecs/getNanos, descriptor `()J`.

## Checks

Separate-argument input is a different representation and is deferred in this experiment. It must not silently replace object input.

## Related example

[function: kotlin at this stage](../function/03-requests.kotlin.md)
