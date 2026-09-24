<!-- spec: {"kind": "variant", "example": "typedef", "stage": "04-select", "language": "jni"} -->

[Stage chapter](../../stages/04-select.md) · [Common cell][typedef_select] · [Element path][typedef]
Owner: the registry, from the rule the JNI frontend recorded for `ptr_class!(Ledger)`

# Type alias declaring an opaque handle — Select conversion relations — Kotlin/JNI

## Input

```text
Crossing { source: Ledger, direction: IntoRust }
Crossing { source: Ledger, direction: OutOfRust }
rules:    Type(Ledger) -> Terminal { in: jlong FromRaw, out: jlong IntoRaw, release }
```

## Result

```text
atomic, carried in a `jlong` whose metadata names `example.Ledger`
```

for both. A `ptr_class!` holds an address, not properties, so the JNI frontend
records a `Terminal` [representation](../../stages/05-represent.md#represent-and-compose-values)
naming the atomic [relation](../../stages/04-select.md#what-a-relation-is) —
the same answer for the same reason as C's, which is what makes the selection
stage target-independent in practice: the targets differ in what the address
is carried as.

## Checks

- Declared as `data_class!` instead, the extern would be refused here with
  `unsupported.type.not_a_struct`: a type with no fields has no struct relation.
- The `jlong` [wire type](../../stages/05-represent.md#describing-target-values-and-operations)
  is of kind `Handle`, not `Long`: the same Rust type as an `i64`'s, read by
  the JVM side as an address rather than a number, and so a different wire
  type.
- A `ptr_class!` with `.method(..)` members is still refused at those members:
  a method's receiver is a borrowed handle, which is the `fn_borrowed_param`
  path and not this one.

[typedef]: README.md
[typedef_select]: 04-select.md
