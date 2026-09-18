<!-- spec: {"kind": "variant", "example": "struct", "stage": "04-select", "language": "c"} -->

[Stage chapter](../../stages/04-select.md) · [Common cell][struct_select] · [Element path][struct]
Owner: the registry; the C adapter selects from the offered [relations](../../stages/04-select.md#what-a-relation-is)

# Struct with scalar fields — Select conversion relations — C

## Input

```text
Crossing { source: Stamp, direction: IntoRust }
policy:   data_struct named Stamp, passed by value
offered:  [ Stamp.fields, atomic ]
```

## Result

```text
Stamp.fields
```

and, for each part the registry then plans:

```text
secs:  i64, IntoRust   policy: scalar carrier   offered: [ atomic ]   -> atomic
nanos: i64, IntoRust   policy: scalar carrier   offered: [ atomic ]   -> atomic
```

A `data_struct` is a C struct passed by value, and a by-value struct is nothing
but its members, so the C adapter answers with the struct
[relation](../../stages/04-select.md#what-a-relation-is): the
[conversion](../../stages/04-select.md#select-conversion-relations) into a
Rust `Stamp` will be made of one conversion per field. The adapter reads the
answer off the [policy](../../stages/03-requests.md#what-policy-means); it does
not look at the fields, and it does not need to know how many there are.

The alternative is real, not hypothetical: a type declared to C as an opaque
pointer would get `atomic` from the same adapter, and the registry would then
plan `Stamp` as one whole value with no parts, its fields untouched.

## Checks

- The answer is one of the ids the registry offered; the adapter cannot return
  a relation of its own making.
- Under `data_struct` the adapter has committed to describing, in the next
  stage, one member of a `repr(C)` aggregate per part, in part order — the
  [member reads][struct_represent_c] that stage shows.
- A `data_struct` policy on a struct with a field V2 cannot carry is not
  refused here — the adapter still answers `Stamp.fields` — but at the part,
  when that field's own selection is unsupported, and the refusal climbs back
  to this conversion.

[struct]: README.md
[struct_select]: 04-select.md
[struct_represent_c]: 05-represent.c.md
