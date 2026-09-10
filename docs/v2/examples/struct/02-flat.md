<!-- spec: {"kind": "cell", "example": "struct", "stage": "02-flat"} -->

[Stage chapter](../../stages/02-flat.md) · [Element path][struct] · [Source crate](../../source.md)
Owner: Flat · Previous: [Capture source items][struct_source] · Next: [Record binding requests][struct_requests]

# Record with scalar fields — Build and inspect the source model

## Input

From capture, the line this record produced, parsed back into the item it holds:

```json
{ "kind": "struct", "name": "Stamp",
  "content": "pub struct Stamp { pub secs: i64, pub nanos: i64 }",
  "source_location": { "file": "src/source.rs", "line": 1, "column": 1 } }
```

## Result

One element in the namespace, under the name `Stamp`:

```text
Element::Type(Type::Struct(Struct {
    name:   Stamp,
    shape:  FieldShape::Named,
    fields: [ Field { name: Some(secs),  index: 0, ty: TypeRef { kind: Scalar(I64) } },
              Field { name: Some(nanos), index: 1, ty: TypeRef { kind: Scalar(I64) } } ],
    origin: <the captured syntax, src/source.rs:1>,
}))
```

This is the element [the function's parameter][fn_flat] resolves to, and the one
every later stage takes apart.

## Checks

- Field order and names are the declaration's; `i64` is the exact type as
  written; no view says how a field crosses a boundary.
- Field identity includes its owner, so `secs` here is not interchangeable with
  a `secs` at index 0 of another record.
- A type whose fields Flat does not model is an `Extern` instead — a name with
  nothing behind it — not a `Struct` with an empty field list.

[struct]: README.md
[struct_source]: 01-source.md
[struct_requests]: 03-requests.md
[fn_flat]: ../fn/02-flat.md
