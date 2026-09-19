<!-- spec: {"kind": "variant", "example": "typedef", "stage": "04-select", "language": "c"} -->

[Stage chapter](../../stages/04-select.md) · [Common cell][typedef_select] · [Element path][typedef]
Owner: the registry; the C adapter selects from the offered [relations](../../stages/04-select.md#what-a-relation-is)

# Type alias declaring an opaque handle — Select conversion relations — C

## Input

```text
Crossing { source: Ledger, direction: IntoRust }    policy: opaque_ptr Ledger
Crossing { source: Ledger, direction: OutOfRust }   policy: opaque_ptr Ledger
offered:  [ atomic ]
```

## Result

```text
atomic
```

for both. An `opaque_ptr` is a pointer to a value C never looks into, so the C
adapter wants the value whole and answers `atomic` — off the
[policy](../../stages/03-requests.md#what-policy-means), before it knows
whether the type has fields. For an extern the answer is also the only one on
offer.

## Checks

- Declared as `data_type!` instead, the same extern would be refused here:
  the adapter would ask for the struct relation and find none offered. That
  refusal is `unsupported.c.no_relation`, and it is the adapter's, not the
  registry's.
- Declared as `ptr_type!`, a struct with private fields would get `atomic` too,
  and the registry would plan it as one whole value with no parts, its fields
  untouched.
- Under `opaque_ptr` the adapter has committed to describing, at the next
  stage, one [carrier](../../stages/05-represent.md#describing-target-values-and-operations)
  for the whole value in each direction, and a release for the consuming one.

[typedef]: README.md
[typedef_select]: 04-select.md
