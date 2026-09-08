<!-- spec: {"example": "struct", "kind": "cell", "stage": "02-flat"} -->

# Struct: Stamp — Build and inspect Flat

[Pipeline chapter](../../stages/02-flat.md) · [Example path](README.md)

## Input

Flat receives the named-field `Stamp` capture.

## Owner

Flat creates the typed declaration and resolves field types inside one snapshot.

## Result

`model.declared_type("Stamp").ty().as_record()` describes this record (schematic chained lookup). `fields()` yields `secs` at index 0 and `nanos` at index 1, both exact `i64` TypeViews. `shape()` is Named. The field owner is this record; an identical index on another record identifies a different field.

## Checks

The two views stay valid after the original Flat handle is dropped. `&Stamp` is a distinct type and requires explicit referent navigation. No wrapper stripping or parsing key text is allowed. Scalar fields are the only field-type variant specified by these initial example paths.


## Along this example

[Previous](01-source.md) [Next](03-requests.md)

## Related example

[function at this stage](../function/02-flat.md)
