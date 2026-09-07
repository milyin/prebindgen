<!-- spec: {"example": "struct", "kind": "cell", "stage": "03-requests"} -->

# Struct: Stamp — Record binding requests

[Pipeline chapter](../../stages/03-requests.md) · [Example path](README.md)

## Input

The frontend is configured to expose `Stamp` as a public data type and to use its named fields for owned input construction.

## Owner

The language frontend creates a type OutputRequest plus a conversion rule using a checked RecordRelation.

## Result

The type output is an independent root. The input rule selects `Stamp` IntoRust with its record relation and the language-specific representation. Rule parts identify field 0 `secs` and field 1 `nanos`; their arm is None because the record has no alternatives. The function parameter refers to this rule.

## Checks

A public type request alone does not invent a callable or require an output conversion. This fixture needs IntoRust and two scalar children. OutOfRust for Stamp is deferred in this fixture. Constructor helper arguments are not inferred from field names.

## Language variants

- [c](03-requests.c.md)
- [kotlin](03-requests.kotlin.md)

## Along this example

[Previous](02-flat.md) [Next](04-values.md)

## Related example

[function at this stage](../function/03-requests.md)
