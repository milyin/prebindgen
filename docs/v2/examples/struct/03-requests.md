<!-- spec: {"kind": "cell", "example": "struct", "stage": "03-requests"} -->

# Record with scalar fields — Record binding requests

[Stage chapter](../../stages/03-requests.md) · [Element path][struct] · [Source fixture](../../source.md)

## Input

The published source model and the user's call asking for `Stamp` to be part of
the generated API.

## Owner

The language frontend.

## Result

One `OutputRequest` for the type, and the choices that decide how values of it
are converted wherever they appear: the source relationship — construct and read
`Stamp` through its two fields — and the target representation recorded in the
policy. A record request is a root in its own right: it is requested even if no
exported function mentions it, and it stays requested if the function that does
mention it is skipped.

The fields are addressable from here on. `(Stamp.fields, None, Field("secs"))`
and its `nanos` counterpart are the parts a per-field rule would attach to; this
fixture records none, so both take the default scalar treatment.

## Checks

Choosing a representation is not the same as choosing a source relationship: the
policy says "C aggregate" or "JVM object", while `Stamp.fields` says "built from
and read as these two fields". A different relationship — a `stamp_from_millis`
constructor, say — would change the parts without changing the representation,
and an object-input override would change the representation without changing the
parts. Both stay separately recorded so that the [conversion node
identity][struct_values] can distinguish them.

## Representation

```text
OutputRequest {
  id:     ElementId(public Stamp in this target),
  source: SourceItemId(crate::source::Stamp),
  policy: PolicyId(record policy for this target),
}

relation: Stamp.fields = Relation::Record(RecordRelation over the Stamp record view)
parts:
  PartId { owner: Stamp.fields, arm: None, position: Field("secs")  }
  PartId { owner: Stamp.fields, arm: None, position: Field("nanos") }

conversion_rules.parts: {}   // no per-field overrides in this fixture
```

## Language variants

- [C][struct_requests_c]
- [Kotlin/JNI][struct_requests_jni]

## Along this element

Previous: [Build and inspect the source model][struct_flat] · Next: [Plan value conversions][struct_values]

[struct]: README.md
[struct_flat]: 02-flat.md
[struct_values]: 04-values.md
[struct_requests_c]: 03-requests.c.md
[struct_requests_jni]: 03-requests.jni.md
