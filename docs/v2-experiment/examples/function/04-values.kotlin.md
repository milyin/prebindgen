<!-- spec: {"example": "function", "kind": "variant", "language": "kotlin", "stage": "04-values"} -->

# Function: stamp_sum — Plan value conversions — kotlin

[Pipeline chapter](../../stages/04-values.md) · [Common contract](04-values.md) · [Example path](README.md)

## Input

Stamp object input conversion and i64 output conversion requested at the function sites.

## Owner

Registry uses JNI representation and scalar rules.

## Result

Use an environment+object -> source Stamp record node and a source i64 -> jlong identity node. The record node can fail with jni::errors::Error; the result node is infallible.

## Checks

Propagate getter errors to the enclosing boundary plan; the value node must not throw or return a fallback value on its own.

## Related example

[struct: kotlin at this stage](../struct/04-values.kotlin.md)
