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

```text
OutputRequest {
    id:     ElementId(public Stamp in this target),
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
- Relation and representation are separate choices. A `stamp_from_millis`
  constructor would change the parts without changing the representation; an
  object-input override changes the representation without changing the parts.
  [Node identity][struct_values] distinguishes both.

## Language variants

- [C][struct_requests_c]
- [Kotlin/JNI][struct_requests_jni]

[struct]: README.md
[struct_flat]: 02-flat.md
[struct_values]: 04-values.md
[struct_requests_c]: 03-requests.c.md
[struct_requests_jni]: 03-requests.jni.md
