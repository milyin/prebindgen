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

The carriers and the operations that obtain them are the target's:
[C][struct_values_c], [Kotlin/JNI][struct_values_jni]. The two `i64` children are
identity conversions in both, and render no code of their own.

## Checks

- Selection precedes traversal: an opaque representation would never inspect the
  fields, which is what makes a record with unreadable private fields
  representable as a handle.
- Parts come from the selected relation, so a constructor relation would give
  one `millis` argument instead of two fields, with no change to this stage.
- Construction follows declaration order, and the mapping from parts to carriers
  is validated.
- An unsupported child makes this node unsupported, which propagates to
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
