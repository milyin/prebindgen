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
output   type:Ledger   type, representation <the handle>, release <a function form>
rule     Type(Ledger) -> <the handle>   // every Ledger, wherever it turns up
    parts: none                         // an atomic value has no parts to record a rule for
```

The request is a root, as a struct's is. Its
[choice](../../stages/03-requests.md#what-a-choice-records) names two things a
struct's does not: through the
[representation](../../stages/05-represent.md#represent-and-compose-values), the
[carrier](../../stages/05-represent.md#describing-target-values-and-operations)
the address crosses as, and, through the release form, where the **release** —
the exported function that frees a handle the caller does not give back —
lands: a symbol for C, an `external` method on the harness for Kotlin. Both are the frontend's choices,
and the
release is not optional. A handle without one is a leak the foreign side cannot
avoid.

## Checks

- A handle request stands whether or not any exported function mentions the
  type. Unlike a struct's, honouring it emits a function — the release — so a
  type output with no release form refuses the type with
  `unsupported.type.no_release` rather than failing at emission.
- A function returning or consuming a `Ledger` records nothing about the
  handle at its own
  [site](../../stages/03-requests.md#a-values-position-in-an-exported-function);
  it takes the type's choice, exactly as
  [a function taking a struct does][fn_requests].
- The same extern under a different carrier, or a struct under a handle
  representation, is a different
  [conversion](../../stages/04-select.md#select-conversion-relations): the
  representation a [conversion rule](../../stages/03-requests.md#conversion-rules)
  names is part of a conversion's identity.

## Language variants

- [C][typedef_requests_c]
- [Kotlin/JNI][typedef_requests_jni]

[typedef]: README.md
[typedef_flat]: 02-flat.md
[typedef_select]: 04-select.md
[typedef_requests_c]: 03-requests.c.md
[typedef_requests_jni]: 03-requests.jni.md
[fn_requests]: ../fn/03-requests.md
