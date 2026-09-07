<!-- spec: {"example": "function", "kind": "variant", "language": "kotlin", "stage": "03-requests"} -->

# Function: stamp_sum — Record binding requests — kotlin

[Pipeline chapter](../../stages/03-requests.md) · [Common contract](03-requests.md) · [Example path](README.md)

## Input

Function request for stamp_sum; the struct rule selects a JVM object input.

## Owner

JNI frontend.

## Result

Record package `example`, object `Bindings`, static native method `sum(Stamp): Long`. Select JNI extern system convention. Runtime failures preserve an existing pending exception; otherwise request RuntimeException. Failure to report an error aborts.

## Checks

This is an explicit fixture policy, not a change to existing frontend defaults. Native-library loading is harness work. No Java writer is introduced.

## Related example

[struct: kotlin at this stage](../struct/03-requests.kotlin.md)
