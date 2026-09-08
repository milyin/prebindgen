<!-- spec: {"example": "function", "kind": "cell", "stage": "01-source"} -->

# Function: stamp_sum — Capture source items

[Pipeline chapter](../../stages/01-source.md) · [Example path](README.md)

## Input

The shared fixture declares `stamp_sum(stamp: Stamp) -> i64`. The body calls `wrapping_add` on the two source fields.

## Owner

The source capture mechanism records one function item.

## Result

One function capture retains the name `stamp_sum`, source module, parameter 0 named `stamp` of type `Stamp`, and return type `i64`. Parameter 0 belongs to the function capture; it is not a second top-level item.

## Checks

The signature order and exact type readings must survive capture. No C symbol, Kotlin name, target parameter or converter is assigned here. This function capture does not traverse field definitions; the captured body does not define a binding decomposition.


## Along this example

[Next](02-flat.md)

## Related example

[struct at this stage](../struct/01-source.md)
