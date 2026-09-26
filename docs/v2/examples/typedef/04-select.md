<!-- spec: {"kind": "cell", "example": "typedef", "stage": "04-select"} -->

[Stage chapter](../../stages/04-select.md) · [Element path][typedef] · [Source crate](../../source.md)
Owner: the registry, from the binding's rules · Previous: [Record binding requests][typedef_requests] · Next: [Represent and compose values][typedef_represent]

# Type alias declaring an opaque handle — Select conversion relations

## Input

```text
Crossing { source: Ledger, direction: IntoRust  }   // the type's own value, and ledger_close's parameter
Crossing { source: Ledger, direction: OutOfRust }   // the type's own value, and ledger_open's return
rules:    Type(Ledger) -> this target's handle: a Terminal over an address, with a release
relations of Ledger: [ atomic ]                     // the only relation an extern has
```

## Result

```text
Ledger, IntoRust
  relation: atomic                  // no parts; the tree ends here
Ledger, OutOfRust
  relation: atomic
```

A `Terminal` [representation](../../stages/05-represent.md#represent-and-compose-values)
names the atomic [relation](../../stages/04-select.md#what-a-relation-is), the
one every type has, and there is nothing under it to plan: the whole
[conversion](../../stages/04-select.md#select-conversion-relations) is one
operation each way, which the representation states. Both directions are
planned because the representation names a release: a handle is a promise
that what one function returns can be given to another.

Selection is trivial here, and that is the point of the page. A struct under a
handle rule gets the same answer, and its fields go uninspected — which is how a
type with private or unsupported fields can still cross, as
[the C][typedef_select_c] and [Kotlin/JNI][typedef_select_jni] pages say.

## Checks

- A handle rule on a struct names the atomic relation too; a `Product` rule on
  an extern finds no struct relation and is refused with
  `unsupported.type.not_a_struct`, in the same words for both targets.
- No part is planned, so nothing under the type can be unsupported.
- The same two conversions serve the type's own value, `ledger_open` and
  `ledger_close`: all three positions take the one `Type(Ledger)` rule, so one
  representation, and one conversion per direction.

## Language variants

- [C][typedef_select_c]
- [Kotlin/JNI][typedef_select_jni]

[typedef]: README.md
[typedef_requests]: 03-requests.md
[typedef_represent]: 05-represent.md
[typedef_select_c]: 04-select.c.md
[typedef_select_jni]: 04-select.jni.md
