<!-- spec: {"kind": "cell", "example": "struct", "stage": "02-flat"} -->

# Record with scalar fields — Build and inspect the source model

[Stage chapter](../../stages/02-flat.md) · [Element path][struct] · [Source fixture](../../source.md)

## Input

The captured `Stamp` record.

## Owner

Flat, which lowers the declaration into an element in the namespace and
publishes the views over it.

## Result

`model.declared_type("Stamp")` returns a `TypeDeclView`, whose `ty()` is a
`TypeView` of `Stamp`, whose `as_record()` is a `RecordView` with a named field
shape and two `FieldView`s: index 0 named `secs` and index 1 named `nanos`, each
with a `TypeView` of `i64`. The same record view is what [the function
path][fn_flat] reaches by following its parameter's type.

## Checks

The views report source facts and nothing more: field order and names are the
declaration's, `i64` is the exact type as written, and no view says how a field
should cross a boundary. Field identity includes its owner, so `secs` here is not
interchangeable with a field of the same name and index in another record. A view
obtained from a different snapshot is rejected even when its type key is equal.
An opaque declaration stays opaque — `as_record()` returns nothing for a type
whose fields Flat does not model, rather than an empty record.

## Representation

```rust
let decl = model.declared_type("Stamp").expect("captured");
let record = decl.ty().as_record().expect("named-field record");

let fields: Vec<_> = record.fields().collect();
assert_eq!(fields.len(), 2);
assert_eq!(fields[0].name(), Some("secs"));
assert_eq!(fields[0].index(), 0);
assert_eq!(fields[1].name(), Some("nanos"));
assert_eq!(fields[1].index(), 1);

for field in record.fields() {
    let ty = field.ty();                 // TypeView of i64, same snapshot
    assert!(ty.as_record().is_none());
}
```

What leaves this stage is one record view and two field type views. Everything
the later stages do to this record — selecting a relationship, converting the
children, mapping them onto members or getters — is derived from exactly these.

## Along this element

Previous: [Capture source items][struct_source] · Next: [Record binding requests][struct_requests]

[struct]: README.md
[struct_source]: 01-source.md
[struct_requests]: 03-requests.md
[fn_flat]: ../fn/02-flat.md
