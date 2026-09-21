<!-- spec: {"kind": "variant", "example": "typedef", "stage": "04-select", "language": "jni"} -->

[Stage chapter](../../stages/04-select.md) · [Common cell][typedef_select] · [Element path][typedef]
Owner: the registry; the JNI adapter selects from the offered [relations](../../stages/04-select.md#what-a-relation-is)

# Type alias declaring an opaque handle — Select conversion relations — Kotlin/JNI

## Input

```text
Crossing { source: Ledger, direction: IntoRust }
Crossing { source: Ledger, direction: OutOfRust }
offered:  [ atomic ]

held by the JniTarget, not passed in:
choice:   ptr_class example.Ledger
```

## Result

```text
atomic, conversion JniChoice::PtrClass { class: "example.Ledger", .. }
```

for both. A `ptr_class` holds an address, not properties, so the JNI adapter
wants the value whole and answers `atomic` off the
[choice](../../stages/03-requests.md#what-a-choice-records) — the same answer for
the same reason as C's, which is what makes the selection stage
target-independent in practice: the adapters differ at the next stage, in what
the address is carried as.

## Checks

- Declared as `data_class!` instead, the extern would be refused here with
  `unsupported.jni.no_relation`: no struct relation is offered for a type with
  no fields.
- Under `ptr_class` the adapter has committed to describing a `jlong`
  [carrier](../../stages/05-represent.md#describing-target-values-and-operations)
  in each direction, and a release for the consuming one.
- A `ptr_class` with `.method(..)` members is still refused at those members:
  a method's receiver is a borrowed handle, which is the `fn_borrowed_param`
  path and not this one.

[typedef]: README.md
[typedef_select]: 04-select.md
