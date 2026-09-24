<!-- spec: {"kind": "cell", "example": "struct", "stage": "04-select"} -->

[Stage chapter](../../stages/04-select.md) · [Element path][struct] · [Source crate](../../source.md)
Owner: the registry, from the binding's rules · Previous: [Record binding requests][struct_requests] · Next: [Represent and compose values][struct_represent]

# Struct with scalar fields — Select conversion relations

## Input

```text
Crossing { source: Stamp, direction: IntoRust }
position: wherever this Stamp sits           // what a rule at a position is looked up by
rules:    Type(Stamp) -> Product { via: Fields, wire_type: <this target's Stamp wire type>, read, build }
          Type(i64)   -> Terminal { wire_type: <this target's i64 wire type>, identity }
relations of Stamp: [ Stamp.fields, atomic ] // registered by the registry from Flat
```

## Result

```text
Stamp, IntoRust
  representation: the Type(Stamp) rule's     // relation Stamp.fields
  +-- part 0  secs:  i64, IntoRust  ->  the Type(i64) rule's   // relation atomic
  +-- part 1  nanos: i64, IntoRust  ->  the Type(i64) rule's
```

The registry registers the struct's two [relations](../../stages/04-select.md#what-a-relation-is)
the first time `Stamp` is planned: the atomic one every type has, and
`Stamp.fields`, built from the fields Flat reports. No rule is recorded at this
`Stamp`'s position, so the registry takes the one recorded for the type. Under
the struct [choices](../../stages/03-requests.md#what-a-choice-records) these
pages use — a C `data_struct`, a Kotlin data class — that rule says `Fields`,
which resolves to `Stamp.fields`, and only then does the registry read the
relation's parts and plan each. Each part takes the `i64` rule from the
adapter's scalar table: a `Terminal`, the atomic relation, with nothing under
it, and the tree ends there.

Selection is the whole of this stage for a struct, and the registry does all
of it without calling the target. Nothing has been written, but everything a
writer will need is now known: the
[conversion](../../stages/04-select.md#select-conversion-relations) is made of
two field conversions, in declaration order, and the
[wire type](../../stages/05-represent.md#describing-target-values-and-operations)
of `Stamp` has two members, each carried as the `i64` rule says. The
[C][struct_select_c] and [Kotlin/JNI][struct_select_jni] pages show each
frontend's rules.

## Checks

- Selection precedes traversal. The lookup reads the type and the position,
  not the fields; a rule saying `Whole` — an opaque handle — would leave
  private and unsupported fields uninspected.
- Parts come from the selected relation, so a constructor relation, when it
  exists, would give one `millis` argument instead of two fields.
- Part order is declaration order, and the next stage's reads are paired with
  parts in that order.
- An unsupported part makes this conversion unsupported, which propagates to
  [everything requiring the struct][struct_retain] rather than producing a
  struct missing a field.

## Language variants

- [C][struct_select_c]
- [Kotlin/JNI][struct_select_jni]

[struct]: README.md
[struct_requests]: 03-requests.md
[struct_represent]: 05-represent.md
[struct_retain]: 07-retain.md
[struct_select_c]: 04-select.c.md
[struct_select_jni]: 04-select.jni.md
