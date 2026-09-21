<!-- spec: {"kind": "cell", "example": "struct", "stage": "04-select"} -->

[Stage chapter](../../stages/04-select.md) · [Element path][struct] · [Source crate](../../source.md)
Owner: the registry, on the target adapter's selections · Previous: [Record binding requests][struct_requests] · Next: [Represent and compose values][struct_represent]

# Struct with scalar fields — Select conversion relations

## Input

```text
Crossing { source: Stamp, direction: IntoRust }
position: wherever this Stamp sits           // what the target looks its policy up by
offered:  [ Stamp.fields, atomic ]           // registered by the registry from Flat

held by the target, not passed in:
policy:   this target's struct policy
```

## Result

```text
Stamp, IntoRust
  relation: Stamp.fields                // the target's answer,
  conversion: <this target's struct policy>   // with the key that policy makes
  +-- part 0  secs:  i64, IntoRust  ->  atomic
  +-- part 1  nanos: i64, IntoRust  ->  atomic
```

The registry registers the struct's two [relations](../../stages/04-select.md#what-a-relation-is)
the first time `Stamp` is planned: the atomic one every type has, and
`Stamp.fields`, built from the fields Flat reports. It offers both to the
target with the [crossing](../../stages/03-requests.md#finding-an-existing-conversion-plan) and the position, and the target names one. Under the
struct [policies](../../stages/03-requests.md#what-policy-means) these pages use — a C `data_struct`, a Kotlin data class — the
answer is `Stamp.fields`, and only then does the registry read the relation's
parts and plan each: an `i64` has the atomic relation and nothing under it, so
the tree ends there.

Selection is the whole of this stage for a struct. No
[carrier](../../stages/05-represent.md#describing-target-values-and-operations)
has been named and no read described; what is settled is that the
[conversion](../../stages/04-select.md#select-conversion-relations) will be
made of two field conversions, in declaration order, each of them an `i64`
converted whole. Beside the relation the target returns the
[conversion key](../../stages/03-requests.md#finding-an-existing-conversion-plan)
standing for the policy it found at that position — which is what later stages
compare, and never read. The [C][struct_select_c] and [Kotlin/JNI][struct_select_jni]
pages show each target's reason for that answer.

## Checks

- Selection precedes traversal. The target sees the type and the position it
  resolves its policy from, not the fields; a target answering `atomic` — an opaque handle — would leave
  private and unsupported fields uninspected.
- Parts come from the selected relation, so a constructor relation, when it
  exists, would give one `millis` argument instead of two fields. The current
  engine offers atomic and struct relations only.
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
