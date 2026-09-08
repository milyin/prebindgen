<!-- spec: {"example": "function", "kind": "cell", "stage": "03-requests"} -->

# Function: stamp_sum — Record binding requests

[Pipeline chapter](../../stages/03-requests.md) · [Example path](README.md)

## Input

The frontend is configured to expose `stamp_sum`, using the selected representation of its `Stamp` argument and a signed 64-bit scalar return.

## Owner

The language frontend records the public choices and creates internal binding requests.

## Result

One function OutputRequest names the checked FunctionView and an output identity. Its parameter site is `(this output, Param(0))`; its result site is `(this output, Return)`. Parameter 0 uses the struct rule. The result uses the target scalar rule. Language-specific names and boundary/error choices are recorded in the variant pages.

## Checks

The source function identity and requested public identity are distinct. The parameter name `stamp` does not select an alternate converter here. A duplicate contradictory declaration is invalid input. An unimplemented selected policy must remain visible, not be replaced by a default.

## Language variants

- [c](03-requests.c.md)
- [kotlin](03-requests.kotlin.md)

## Along this example

[Previous](02-flat.md) [Next](04-values.md)

## Related example

[struct at this stage](../struct/03-requests.md)
