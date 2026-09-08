<!-- spec: {"example": "function", "kind": "cell", "stage": "04-values"} -->

# Function: stamp_sum — Plan value conversions

[Pipeline chapter](../../stages/04-values.md) · [Example path](README.md)

## Input

The function request needs owned `Stamp` input and source `i64` encoded as a foreign signed 64-bit result.

## Owner

The registry requests and retains conversion nodes; the adapter supplies each target description.

## Result

The input node is the struct path's IntoRust record conversion. The output node is an OutOfRust `i64` identity conversion with the target's signed 64-bit carrier. Neither node contains the `stamp_sum` call. Their NodeIds feed the later function plan.

## Checks

Changing C to JNI does not reuse target-specific conversion nodes across generations. A second function using the same relation and effective policy in one generation can reuse the input node. An unsupported input node skips this function downstream without executing V1 for it.

## Language variants

- [c](04-values.c.md)
- [kotlin](04-values.kotlin.md)

## Along this example

[Previous](03-requests.md) [Next](05-boundary.md)

## Related example

[struct at this stage](../struct/04-values.md)
