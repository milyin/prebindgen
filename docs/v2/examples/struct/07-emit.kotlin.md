<!-- spec: {"example": "struct", "kind": "variant", "language": "kotlin", "stage": "07-emit"} -->

# Struct: Stamp — Emit bindings — kotlin

[Pipeline chapter](../../stages/07-emit.md) · [Common contract](07-emit.md) · [Example path](README.md)

## Input

Retained example.Stamp public declaration and the same getter metadata used by input primitives.

## Owner

JNI's Kotlin writer renders a data class; the Kotlin compiler emits its JVM getters.

## Result

Expected Kotlin data class appears below. The compiler must supply getSecs():long and getNanos():long, both descriptor ()J.

## Checks

A naming override must update the public metadata and primitive getter descriptions consistently. The class remains an independent requested type even if its consuming function is skipped.

## Concrete description or output

```kotlin
package example

data class Stamp(val secs: Long, val nanos: Long)
```

## Related example

[function: kotlin at this stage](../function/07-emit.kotlin.md)
