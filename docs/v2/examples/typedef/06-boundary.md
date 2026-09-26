<!-- spec: {"kind": "cell", "example": "typedef", "stage": "06-boundary"} -->

[Stage chapter](../../stages/06-boundary.md) · [Element path][typedef] · [Source crate](../../source.md)
Owner: the registry, on the release form the binding stated · Previous: [Represent and compose values][typedef_represent] · Next: [Retain supported output][typedef_retain]

# Type alias declaring an opaque handle — Assemble the wrapper boundary

## Input

The into-Rust [node](../../stages/05-represent.md#represent-and-compose-values)
that [representation](../../stages/05-represent.md#represent-and-compose-values)
produced ([the previous cell][typedef_represent]), and the release it
carries. There is no source signature: nothing in the source crate frees a
`Ledger`, Rust does.

```text
node(taken) : Ledger  IntoRust  -> an owned source Ledger, failures { Binding }
              release: apply Release to the wire type

source: none
```

## Result

```text
FunctionPlan {
    declaration: public Ledger in this target,          // the type's own, not a function's
    symbol, abi: from the type output's release form    // per target, below
    params:      the form's context parameters, then the form's one input,
                 typed as node(taken)'s wire type
    ret:         none
    instrs:      apply the release to the wrapper parameter
}
```

The registry assembles the release
[wrapper](../../stages/06-boundary.md#assemble-the-wrapper-boundary) with the
code that assembles an exported function, with one difference: there is no
call, so the input is applied to the release operation instead of being
converted and passed on. The form is the one the binding stated on the type's
output, beside its representation, rather than a function's. The registry
checks it as it checks any other: the form names exactly one input, and any
failure the release could raise must have a route.

The [conversions](../../stages/04-select.md#select-conversion-relations) of
`ledger_open` and `ledger_close` meet their own boundaries in those functions'
plans. A handle parameter is placed as any input is, and a handle return is
delivered as any scalar result is; what this path adds to them is a failure
category, `Binding`, that their forms must route.

## Checks

- A release is infallible, so its wrapper needs no route; a release described
  as fallible would need one, and the type would be skipped without it. The
  `Binding` failure of `node(taken)` belongs to taking the handle, which the
  release wrapper does not do.
- The plan is retained under the type's identity, so it survives exactly when
  the type does, and a report never lists it as a function the user asked for.
- A representation with a release and a type output with no release form
  refuses the type with `unsupported.type.no_release`, and everything
  requiring the type with it.

## Language variants

- [C][typedef_boundary_c]
- [Kotlin/JNI][typedef_boundary_jni]

[typedef]: README.md
[typedef_represent]: 05-represent.md
[typedef_retain]: 07-retain.md
[typedef_boundary_c]: 06-boundary.c.md
[typedef_boundary_jni]: 06-boundary.jni.md
