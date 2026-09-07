<!-- spec: {"kind": "stage", "stage": "04-values"} -->

# Plan value conversions

[Experiment contents](../README.md)

## Input

The registry receives a requested type, direction, selected source relationship and effective target policy. Direction says whether a conversion produces a Rust value or encodes one for foreign code.

## Operation and owner

The registry selects the source operation before traversing its children. It obtains fields/helper arguments from Flat, recursively resolves child conversions, and asks the target for local representation and primitive descriptions. A primitive is one typed target operation; PrimitiveSpec records operands, results, failure, validity, resources, generated dependencies and rendering payload. The registry validates descriptions and assembles a ValuePlan.

## Output and failure contract

The result is a reusable conversion plan with a retained TypeView, child plans, representation, instructions and explicit failure/validity contract. It contains no enclosing-function return. Cache identity includes exact source type, direction, source relationship and effective policy. A missing required child capability makes this conversion unsupported; it must not generate a partial record.

## Apply this stage

- [Function: stamp_sum](../examples/function/04-values.md) · [c](../examples/function/04-values.c.md) · [kotlin](../examples/function/04-values.kotlin.md)
- [Struct: Stamp](../examples/struct/04-values.md) · [c](../examples/struct/04-values.c.md) · [kotlin](../examples/struct/04-values.kotlin.md)

## Pipeline navigation

[Previous](03-requests.md) [Next](05-boundary.md)
