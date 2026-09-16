<!-- spec: {"kind": "cell", "example": "struct", "stage": "03-requests"} -->

[Stage chapter](../../stages/03-requests.md) · [Element path][struct] · [Source crate](../../source.md)
Owner: the language frontend · Previous: [Build and inspect the source model][struct_flat] · Next: [Plan value conversions][struct_values]

# Record with scalar fields — Record binding requests

## Input

The element Flat built for this record:

```text
Element::Type(Type::Struct(Struct {
    name:   Stamp,
    shape:  Named,
    fields: [ Field { name: secs,  index: 0, ty: Scalar(I64) },
              Field { name: nanos, index: 1, ty: Scalar(I64) } ],
}))
```

and a build script asking for it to be exposed: [C][struct_requests_c],
[Kotlin/JNI][struct_requests_jni].

## Result

The type request asks for a public foreign [representation](../../stages/04-values.md#plan-value-conversions) of `Stamp`; it is
independent of the request for `stamp_sum`. This sketch also shows the fields
that the registry will use when it discovers the record [relation](../../stages/04-values.md#what-a-relation-is) during value
planning. All structures in the block are design notation, not exact current
request fields. Current `OutputRequest` contains a `Declaration` and `PolicyId`.
The frontend also records a type [policy](../../stages/03-requests.md#what-policy-means) for `Stamp`, which is how a later
function parameter finds this representation without repeating the configuration.

```text
OutputRequest {
    id:     DeclarationId("type:Stamp"),
    source: SourceItemId(crate::source::Stamp),
    policy: PolicyId(this target's record policy),
}

relation: Stamp.fields = Relation::Record(over the Stamp record element)

parts:
    PartId { owner: Stamp.fields, arm: None, position: Field("secs")  }
    PartId { owner: Stamp.fields, arm: None, position: Field("nanos") }

conversion_rules.parts: {}    // none recorded for this path
```

## Checks

- A record request is a root: it stands whether or not any exported function
  mentions the type, and survives a function that is skipped.
- A [relation](../../stages/04-values.md#what-a-relation-is) describes how Rust
  constructs or reads the value; a
  [representation](../../stages/04-values.md#plan-value-conversions) describes
  the foreign values carrying it. This example selects the field relation. A
  future constructor relation would instead have the constructor's arguments
  as parts and would need a compatible target representation.
  [Node](../../stages/04-values.md#plan-value-conversions) identity must
  distinguish these choices, [as the value-planning page explains][struct_values].

## Language variants

- [C][struct_requests_c]
- [Kotlin/JNI][struct_requests_jni]

[struct]: README.md
[struct_flat]: 02-flat.md
[struct_values]: 04-values.md
[struct_requests_c]: 03-requests.c.md
[struct_requests_jni]: 03-requests.jni.md
