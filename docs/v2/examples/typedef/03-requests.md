<!-- spec: {"kind": "cell", "example": "typedef", "stage": "03-requests"} -->

[Stage chapter](../../stages/03-requests.md) · [Element path][typedef] · [Source crate](../../source.md)
Owner: the language frontend · Previous: [Build and inspect the source model][typedef_flat] · Next: [Select conversion relations][typedef_select]

# Type alias declaring an opaque handle — Record binding requests

## Input

The element Flat built for this alias:

```text
Element::Type(Type::Extern(Extern { name: Ledger, target: Some("crate :: ledger :: Ledger") }))
```

and a build script asking for it to be exposed as a handle:
[C][typedef_requests_c], [Kotlin/JNI][typedef_requests_jni].

## Result

```text
OutputRequest {
    id:     DeclarationId(public Ledger in this target),
    source: SourceItemId(crate::source::Ledger),
    policy: PolicyId(this target's handle policy),
}

conversion_rules.parts: {}       // an atomic value has no parts to record a rule for
```

The request is a root, as a struct's is. Its
[policy](../../stages/03-requests.md#what-policy-means) names two things a
struct's does not: the
[carrier](../../stages/05-represent.md#describing-target-values-and-operations)
the address crosses as, and where the **release** — the exported function that
frees a handle the caller does not give back — lands: a symbol for C, a native
method on the harness for Kotlin. Both are the frontend's choices, and the
release is not optional. A handle without one is a leak the foreign side cannot
avoid.

## Checks

- A handle request stands whether or not any exported function mentions the
  type. Unlike a struct's, honouring it emits a function — the release — so a
  target that cannot place one refuses the type here rather than at emission.
- A function returning or consuming a `Ledger` records nothing about the
  handle at its own
  [site](../../stages/03-requests.md#a-values-position-in-an-exported-function);
  it takes the type's policy, exactly as
  [a function taking a struct does][fn_requests].
- The same extern under a different carrier, or a struct under a handle policy,
  is a different [conversion](../../stages/04-select.md#select-conversion-relations):
  the policy is part of a conversion's identity.

## Language variants

- [C][typedef_requests_c]
- [Kotlin/JNI][typedef_requests_jni]

[typedef]: README.md
[typedef_flat]: 02-flat.md
[typedef_select]: 04-select.md
[typedef_requests_c]: 03-requests.c.md
[typedef_requests_jni]: 03-requests.jni.md
[fn_requests]: ../fn/03-requests.md
