<!-- spec: {"example": "struct", "kind": "cell", "stage": "04-values"} -->

# Struct: Stamp — Plan value conversions

[Pipeline chapter](../../stages/04-values.md) · [Example path](README.md)

## Input

The registry has the owned Stamp IntoRust crossing, checked RecordRelation, and target policy.

## Owner

The registry discovers the two children; the adapter describes their foreign member/getter operations; the registry composes the body.

## Result

The ValuePlan obtains `secs`, converts its carrier to source `i64`, obtains `nanos`, converts it, and constructs `source::Stamp { secs, nanos }`. Both scalar conversions are identities for the selected carriers. The resulting source value is owned. Successful copied integers have no validity dependency on the foreign aggregate after reading.

## Checks

Read fields in source order and construct only after both reads succeed. No source function is called in this node. The adapter must not emit recursive field conversion or a complete wrapper. C reads are infallible; JNI getter failures remain explicit in the language variant. No escaping resource is acquired by these integer reads.

## Language variants

- [c](04-values.c.md)
- [kotlin](04-values.kotlin.md)

## Along this example

[Previous](03-requests.md) [Next](06-retain.md)

## Related example

[function at this stage](../function/04-values.md)
