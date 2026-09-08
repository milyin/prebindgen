<!-- spec: {"example": "struct", "kind": "cell", "stage": "01-source"} -->

# Struct: Stamp — Capture source items

[Pipeline chapter](../../stages/01-source.md) · [Example path](README.md)

## Input

The shared fixture declares `Stamp` with public named fields `secs: i64` followed by `nanos: i64`.

## Owner

The source capture mechanism records one struct item.

## Result

One type capture retains the name `Stamp`, named-field form, both fields in declaration order and their types. The fields belong to that capture and are not top-level export requests.

## Checks

Preserve both names, source order, accessibility and source location. This item declares fields, not function parameters or a result. No C aggregate or Kotlin class is selected by capture.


## Along this example

[Next](02-flat.md)

## Related example

[function at this stage](../function/01-source.md)
