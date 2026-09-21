<!-- spec: {"kind": "cell", "example": "typedef", "stage": "04-select"} -->

[Stage chapter](../../stages/04-select.md) · [Element path][typedef] · [Source crate](../../source.md)
Owner: the registry, on the target adapter's selections · Previous: [Record binding requests][typedef_requests] · Next: [Represent and compose values][typedef_represent]

# Type alias declaring an opaque handle — Select conversion relations

## Input

```text
Crossing { source: Ledger, direction: IntoRust  }   // the type's own request, and ledger_close's parameter
Crossing { source: Ledger, direction: OutOfRust }   // the type's own request, and ledger_open's return
offered:  [ atomic ]                                // the only relation an extern has

held by the target, not passed in:
choice:   what this target recorded for Ledger
```

## Result

```text
Ledger, IntoRust
  relation: atomic                  // no parts; the tree ends here
  conversion: <what this target recorded for Ledger>

Ledger, OutOfRust
  relation: atomic
  conversion: <what this target recorded for Ledger>
```

The registry registers one
[relation](../../stages/04-select.md#what-a-relation-is) for an extern — the
atomic one every type has — and offers it with each
[crossing](../../stages/03-requests.md#finding-an-existing-conversion-plan) and
position. The target names it, and there is nothing under it to plan: the whole
[conversion](../../stages/04-select.md#select-conversion-relations) will be
one operation, chosen at the next stage. Beside the relation the target returns
the [conversion key](../../stages/03-requests.md#finding-an-existing-conversion-plan)
standing for the [choice](../../stages/03-requests.md#what-a-choice-records) it
holds for `Ledger`. Both directions are selected because
the type's own request asks for both: a handle is a promise that what one
function returns can be given to another.

Selection is trivial here, and that is the point of the page. A struct under
a handle choice gets the same answer from the same adapter, and its fields go
uninspected — which is how a type with private or unsupported fields can still
cross, as [the C][typedef_select_c] and [Kotlin/JNI][typedef_select_jni] pages
say.

## Checks

- The answer is the one id the registry offered; a handle choice on a struct
  selects `atomic` from the two offered, and a data choice on an extern finds
  no struct relation and is refused with `no_relation`.
- No part is planned, so nothing under the type can be unsupported: the only
  refusals left are the target's own, at the next stage.
- The same two selections serve the type's request, `ledger_open` and
  `ledger_close`: the target resolves the same choice at all three positions,
  so the key it returns is equal at all three, and one conversion serves them
  all.

## Language variants

- [C][typedef_select_c]
- [Kotlin/JNI][typedef_select_jni]

[typedef]: README.md
[typedef_requests]: 03-requests.md
[typedef_represent]: 05-represent.md
[typedef_select_c]: 04-select.c.md
[typedef_select_jni]: 04-select.jni.md
