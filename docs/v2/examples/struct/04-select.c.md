<!-- spec: {"kind": "variant", "example": "struct", "stage": "04-select", "language": "c"} -->

[Stage chapter](../../stages/04-select.md) · [Common cell][struct_select] · [Element path][struct]
Owner: the registry, from the rule the C frontend recorded for `data_struct!(Stamp)`

# Struct with scalar fields — Select conversion relations — C

## Input

```text
Crossing { source: Stamp, direction: IntoRust }
position: wherever this Stamp sits
rules:    Type(Stamp) -> Conversion { via: Fields, choice: CChoice::DataStruct { c_name: "Stamp" } }
relations of Stamp: [ Stamp.fields, atomic ]
```

## Result

```text
Stamp.fields, conversion CChoice::DataStruct { c_name: "Stamp" }
```

and, for each part the registry then plans:

```text
secs:  i64, IntoRust   Default rule: Whole, CChoice::Scalar   -> atomic
nanos: i64, IntoRust   Default rule: Whole, CChoice::Scalar   -> atomic
```

A `data_struct` is a C struct passed by value, and a by-value struct is nothing
but its members, so the C frontend records its rule with `Via::Fields` when it
reads `data_struct!(Stamp)`. The registry resolves that to the struct
[relation](../../stages/04-select.md#what-a-relation-is): the
[conversion](../../stages/04-select.md#select-conversion-relations) into a
Rust `Stamp` will be made of one conversion per field. No C code runs at this
stage: the fields' `i64`s take the `Default` rule the C frontend recorded.

The alternative is real, not hypothetical: `ptr_type!(Stamp)` records the same
rule with `Via::Whole`, and the registry would then plan `Stamp` as one whole
value with no parts, its fields untouched.

## Checks

- The relation is resolved by the registry from the rule; the C target never
  sees a relation id.
- Under `data_struct` the target has committed to describing, in the next
  stage, one member of a `repr(C)` aggregate per part, in part order — the
  [member reads][struct_represent_c] that stage shows.
- A `data_struct` rule on a struct with a field V2 cannot carry is not refused
  here — the registry still resolves `Stamp.fields` — but at the part, when that
  field's own conversion is unsupported, and the refusal climbs back to this
  conversion.

[struct]: README.md
[struct_select]: 04-select.md
[struct_represent_c]: 05-represent.c.md
