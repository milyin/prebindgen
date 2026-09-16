<!-- spec: {"kind": "cell", "example": "struct", "stage": "04-values"} -->

[Stage chapter](../../stages/04-values.md) · [Element path][struct] · [Source crate](../../source.md)
Owner: the registry, on descriptions from the target adapter · Previous: [Record binding requests][struct_requests] · Next: [Retain supported output][struct_retain]

# Record with scalar fields — Plan value conversions

## Input

```text
Crossing { source: Stamp, direction: IntoRust }
relation: Stamp.fields          // selected before any field is inspected
policy:   this target's record policy
```

## Result

```text
node(Stamp, IntoRust) {
    relation: Stamp.fields
    children: [ node(i64, IntoRust) as "secs",
                node(i64, IntoRust) as "nanos" ]
    body:     obtain secs carrier  -> convert -> local
              obtain nanos carrier -> convert -> local
              construct source::Stamp { secs, nanos }
    contract: produces an owned source Stamp, validity Independent
}
```

The registry resolves a child
[conversion](../../stages/04-values.md#plan-value-conversions) for each field,
then constructs the source record from the converted children. Each
[carrier](../../stages/04-values.md#describing-target-values-and-operations)
holds data during conversion: a C member value or a JNI getter result here.
The [C][struct_values_c] and [Kotlin/JNI][struct_values_jni] pages show how
those values are obtained.

Both field positions resolve to the same cached `i64` input [node](../../stages/04-values.md#plan-value-conversions) under this
[policy](../../stages/03-requests.md#what-policy-means); the [wrapper](../../stages/05-boundary.md#assemble-the-native-boundary) applies that plan once per field. Each carrier already
has the required Rust `i64` value, so the conversion is **identity**: it passes
the value through without generating another operation. `Independent`
describes the owned result, which borrows neither input; the full
validity-contract structure remains a proposed extension.

## Checks

- Selection precedes traversal. A future supported atomic-handle conversion
  could avoid inspecting unused fields; this increment selects field conversion.
- Parts come from the selected [relation](../../stages/04-values.md#what-a-relation-is), so a constructor relation would give
  one `millis` argument instead of two fields. That relation needs an additional
  implementation; the current engine offers atomic and record relations only.
- Construction follows declaration order. The registry checks the number and
  types of projections; the adapter is responsible for associating each read
  with the intended source field.
- An unsupported child makes this [node](../../stages/04-values.md#plan-value-conversions) unsupported, which propagates to
  [everything requiring the record][struct_retain] rather than producing a record
  missing a field.

## Language variants

- [C][struct_values_c]
- [Kotlin/JNI][struct_values_jni]

[struct]: README.md
[struct_requests]: 03-requests.md
[struct_retain]: 06-retain.md
[struct_values_c]: 04-values.c.md
[struct_values_jni]: 04-values.jni.md
