<!-- spec: {"example": "function", "kind": "cell", "stage": "06-retain"} -->

# Function: stamp_sum — Retain supported output

[Pipeline chapter](../../stages/06-retain.md) · [Example path](README.md)

## Input

The candidate wrapper requires the Stamp input conversion, scalar result conversion, public parameter type and any helper artifacts its failure route invokes.

## Owner

The registry computes dependency closure and records the function outcome.

## Result

Retain the wrapper only if all required nodes, public declarations and helper artifacts are supported. C requires the StampC declaration used by its native parameter. Kotlin requires example.Stamp, Bindings.sum and the JNI error-reporting helper. The function and these dependencies are frozen together.

## Checks

Temporarily mark the record conversion unsupported: the function must be skipped with a path through Param(0) to that cause. A missing JNI reporting operation skips the JNI function even if both getter operations are otherwise supported. No dangling extern/native declaration may remain; unrelated supported roots survive.


## Along this example

[Previous](05-boundary.md) [Next](07-emit.md)

## Related example

[struct at this stage](../struct/06-retain.md)
