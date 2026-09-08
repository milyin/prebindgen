<!-- spec: {"example": "function", "kind": "variant", "language": "c", "stage": "04-values"} -->

# Function: stamp_sum — Plan value conversions — c

[Pipeline chapter](../../stages/04-values.md) · [Common contract](04-values.md) · [Example path](README.md)

## Input

Stamp input conversion and i64 output conversion requested at the function sites.

## Owner

Registry uses C representation and scalar rules.

## Result

Use one StampC -> source Stamp record node and one source i64 -> native i64 identity node. Input record primitives are defined by the struct path at this same stage.

## Checks

No call to stamp_sum belongs in either conversion node. Both conversion failure sets are empty for this fixture.

## Related example

[struct: c at this stage](../struct/04-values.c.md)
