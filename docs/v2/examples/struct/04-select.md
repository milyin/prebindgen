<!-- spec: {"kind": "cell", "example": "struct", "stage": "04-select"} -->

[Stage chapter](../../stages/04-select.md) · [Element path][struct] · [Source crate](../../source.md)
Owner: the registry, from the binding's rules · Previous: [Record binding requests][struct_requests] · Next: [Represent and compose values][struct_represent]

# Struct with scalar fields — Select conversion relations

## Input

```text
Crossing { source: Stamp, direction: IntoRust }
position: wherever this Stamp sits           // what a rule at a position is looked up by
rules:    Type(Stamp) -> Conversion { via: Fields, choice: <what this target recorded> }
relations of Stamp: [ Stamp.fields, atomic ] // registered by the registry from Flat
```

## Result

```text
Stamp, IntoRust
  relation: Stamp.fields                // what the rule's `Fields` resolves to,
  conversion: <what this target recorded for Stamp>   // with the rule's choice
  +-- part 0  secs:  i64, IntoRust  ->  atomic   // no rule: the target's default
  +-- part 1  nanos: i64, IntoRust  ->  atomic
```

The registry registers the struct's two [relations](../../stages/04-select.md#what-a-relation-is)
the first time `Stamp` is planned: the atomic one every type has, and
`Stamp.fields`, built from the fields Flat reports. No rule is recorded at this
`Stamp`'s position, so the registry takes the one recorded for the type. Under
the struct [choices](../../stages/03-requests.md#what-a-choice-records) these
pages use — a C `data_struct`, a Kotlin data class — that rule says `Fields`,
which resolves to `Stamp.fields`, and only then does the registry read the
relation's parts and plan each. Nothing records a rule for `i64`, so each part
takes `Target::default_conversion(i64)`: `Whole`, the atomic relation, with
nothing under it, and the tree ends there.

Selection is the whole of this stage for a struct, and the registry does all
of it. No
[carrier](../../stages/05-represent.md#describing-target-values-and-operations)
has been named and no read described; what is settled is that the
[conversion](../../stages/04-select.md#select-conversion-relations) will be
made of two field conversions, in declaration order, each of them an `i64`
converted whole. The rule's choice is the
[conversion key](../../stages/03-requests.md#finding-an-existing-conversion-plan)
— which is what later stages compare, and never read. The
[C][struct_select_c] and [Kotlin/JNI][struct_select_jni] pages show each
frontend's rule.

## Checks

- Selection precedes traversal. The lookup reads the type and the position,
  not the fields; a rule saying `Whole` — an opaque handle — would leave
  private and unsupported fields uninspected.
- Parts come from the selected relation, so a constructor relation, when it
  exists, would give one `millis` argument instead of two fields. The current
  engine resolves `Whole` and `Fields` only.
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
