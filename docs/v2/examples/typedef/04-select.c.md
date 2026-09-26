<!-- spec: {"kind": "variant", "example": "typedef", "stage": "04-select", "language": "c"} -->

[Stage chapter](../../stages/04-select.md) · [Common cell][typedef_select] · [Element path][typedef]
Owner: the registry, from the rule the C frontend recorded for `ptr_type!(Ledger)`

# Type alias declaring an opaque handle — Select conversion relations — C

## Input

```text
Crossing { source: Ledger, direction: IntoRust }
Crossing { source: Ledger, direction: OutOfRust }
rules:    Type(Ledger), into Rust   -> Whole { *mut Ledger, FromRaw }
          Type(Ledger), out of Rust -> Whole { *mut Ledger, IntoRaw, release }
```

## Result

```text
atomic, carried in `*mut Ledger`
```

for both. `ptr_type!` declares a pointer to a value C never looks into, so the
C frontend records a `Whole` [representation](../../stages/05-represent.md#represent-and-compose-values)
each way, which names the atomic [relation](../../stages/04-select.md#what-a-relation-is)
before anyone knows whether the type has fields. For an extern the atomic
relation is also the only one there is.

## Checks

- Declared as `data_type!` instead, the same extern would be refused here: the
  rule would name `Parts` through `Fields`, and the registry would find no
  fields. That refusal is `unsupported.type.not_a_struct`, the registry's.
- Declared as `ptr_type!`, a struct with private fields would get `atomic` too,
  and the registry would plan it as one whole value with no parts, its fields
  untouched.
- The two representations name one
  [wire type](../../stages/05-represent.md#describing-target-values-and-operations),
  and the out-of-Rust one a release, which the type's output exports as
  `ledger_drop`.

[typedef]: README.md
[typedef_select]: 04-select.md
