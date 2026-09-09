<!-- spec: {"kind": "cell", "example": "fn", "stage": "02-flat"} -->

# Function taking an owned record — Build and inspect the source model

[Stage chapter](../../stages/02-flat.md) · [Element path][fn] · [Source crate](../../source.md)
Previous: [Capture source items][fn_source] · Next: [Record binding requests][fn_requests]

## Input

The captured `stamp_sum` record, in a build that also captured `Stamp`.

## Owner

Flat. It lowers the signature into an element, checks that every type the
signature names is declared somewhere in the namespace, and publishes read-only
views over the result.

## Result

`model.function("stamp_sum")` returns a `FunctionView`. Its `parameters()` yields
exactly one `ParameterView`, at index 0, named `stamp`, whose `ty()` is a
`TypeView` of `Stamp`. Its `return_type()` is a `TypeView` of `i64`.

That parameter type holds the name `Stamp` and nothing else — there is no stored
link to the declaration. `parameter.ty().as_record()` performs the lookup and
reaches [the record view][struct_flat], the same view the record path describes,
in the same snapshot.

Reading a function assigns it nothing. `stamp_sum` is a function that takes a
`Stamp`; whether it is exported, and whether some other function is a constructor
for `Stamp`, are decisions no view expresses.

## Checks

All views reached from this function retain the same snapshot, and stay usable
after the original `Flat` handle is dropped. A `Stamp` view from a different
snapshot is not interchangeable with this one, even though the name and the type
key match. Lookup and enumeration agree: `model.functions()` yields this same
function view. Had the parameter named a type nothing declares, this function
would not be here to look up at all: building the model refuses an element whose
names do not resolve, so a `FunctionView` that exists is one whose types can be
followed.

## Representation

```rust
let function = model.function("stamp_sum").expect("captured");
assert_eq!(function.name(), "stamp_sum");

let parameters: Vec<_> = function.parameters().collect();
assert_eq!(parameters.len(), 1);
assert_eq!(parameters[0].index(), 0);
assert_eq!(parameters[0].name(), "stamp");

let stamp = parameters[0].ty();               // TypeView of Stamp
let record = stamp.as_record().expect("record");   // RecordView, same snapshot
assert_eq!(record.fields().count(), 2);

let result = function.return_type();          // TypeView of i64
assert!(result.as_record().is_none());
```

Two type views leave this stage, and they are what the later stages plan
against: `Stamp` used as an owned parameter type, and `i64` used as the result
type. Neither is a name or a key — each retains its reading and its snapshot, so
`Stamp`, `&Stamp` and `Option<Stamp>` remain three different inputs to planning.

[fn]: README.md
[fn_source]: 01-source.md
[fn_requests]: 03-requests.md
[struct_flat]: ../struct/02-flat.md
