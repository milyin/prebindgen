<!-- spec: {"kind": "variant", "example": "struct", "stage": "04-select", "language": "c"} -->

[Stage chapter](../../stages/04-select.md) · [Common cell][struct_select] · [Element path][struct]
Owner: the registry, from the rule the C frontend recorded for `data_struct!(Stamp)`

# Struct with scalar fields — Select conversion relations — C

## Input

```text
Crossing { source: Stamp, direction: IntoRust }
position: wherever this Stamp sits
rules:    Type(Stamp) -> Product { via: Fields, carrier: stamp_c, read: ReadMember, build: BuildAggregate }
          Type(i64)   -> Terminal { carrier: i64 carrier, identity both ways }
carriers: stamp_c = CName("Stamp")
          i64 carrier, CName(builtin)
relations of Stamp: [ Stamp.fields, atomic ]
```

## Result

```text
Stamp.fields, carried in stamp_c
secs:  i64, IntoRust   Type(i64) rule -> atomic
nanos: i64, IntoRust   Type(i64) rule -> atomic
```

A `data_struct` is a C struct passed by value, and a by-value struct is nothing
but its members, so the C frontend records a `Product` through `Via::Fields`
over the `Stamp` [carrier](../../stages/05-represent.md#describing-target-values-and-operations) when it reads `data_struct!(Stamp)`. The registry resolves that to the struct
[relation](../../stages/04-select.md#what-a-relation-is): the
[conversion](../../stages/04-select.md#select-conversion-relations) into a
Rust `Stamp` will be made of one conversion per field. No C code runs at this
stage: the fields' `i64`s take the rule from C's scalar table.

The alternative is real, not hypothetical: `ptr_type!(Stamp)` records a
`Terminal` over a `*mut stamp_t` carrier instead, and the registry would then
plan `Stamp` as one whole value with no parts, its fields untouched.

## Checks

- The relation is resolved by the registry from the rule; the C target never
  sees a relation id.
- The `Stamp` carrier's members are now known — two, each an `i64` — so the
  C writer can be fed them to write the `repr(C)` declaration, and the
  registry can write the [member reads][struct_represent_c] itself.
- A `data_struct` rule on a struct with a field V2 cannot carry is not refused
  here — the registry still resolves `Stamp.fields` — but at the part, when that
  field's own conversion is unsupported, and the refusal climbs back to this
  conversion.

[struct]: README.md
[struct_select]: 04-select.md
[struct_represent_c]: 05-represent.c.md
