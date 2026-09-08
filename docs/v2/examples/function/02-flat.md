<!-- spec: {"example": "function", "kind": "cell", "stage": "02-flat"} -->

# Function: stamp_sum — Build and inspect Flat

[Pipeline chapter](../../stages/02-flat.md) · [Example path](README.md)

## Input

Flat receives the `stamp_sum` function capture and the `Stamp` declaration from the same source model.

## Owner

Flat lowers and checks the signature, then returns immutable inspection views.

## Result

`model.function("stamp_sum")` succeeds with a FunctionView. `parameters()` yields exactly one ParameterView, with index 0, name `stamp`, and a TypeView of `Stamp`. `return_type()` yields a TypeView of `i64`. Following `parameter.ty().as_record()` reaches the struct view described by the struct path at this stage.

## Checks

All returned views retain the same snapshot. A reference named `Stamp` in another snapshot is not interchangeable. Function lookup and enumeration must agree; neither changes the source model. Reading a function does not assign it a constructor or projector conversion role.


## Along this example

[Previous](01-source.md) [Next](03-requests.md)

## Related example

[struct at this stage](../struct/02-flat.md)
