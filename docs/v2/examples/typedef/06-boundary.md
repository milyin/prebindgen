<!-- spec: {"kind": "cell", "example": "typedef", "stage": "06-boundary"} -->

[Stage chapter](../../stages/06-boundary.md) · [Element path][typedef] · [Source crate](../../source.md)
Owner: the registry, on the target's boundary description · Previous: [Represent and compose values][typedef_represent] · Next: [Retain supported output][typedef_retain]

# Type alias declaring an opaque handle — Assemble the wrapper boundary

## Input

The into-Rust [node](../../stages/05-represent.md#represent-and-compose-values)
that [representation](../../stages/05-represent.md#represent-and-compose-values)
produced ([the previous cell][typedef_represent]), and the release it
carries. There is no source signature: nothing in the source crate frees a
`Ledger`, Rust does.

```text
node(taken) : Ledger  IntoRust  -> an owned source Ledger, failures { Binding }
              release: apply Release to the carrier

source: none
```

## Result

```text
FunctionPlan {
    declaration: public Ledger in this target,          // the type's own, not a function's
    inputs:      [ node(taken) ],
    output:      none,
    boundary:    BoundarySpec { … },                    // per target, below
    body:        apply the release to the wrapper parameter
                 -> deliver nothing
}
```

The registry assembles the release
[wrapper](../../stages/06-boundary.md#assemble-the-wrapper-boundary) with the
code that assembles an exported function, with one difference: there is no
call, so the input is applied to the release operation instead of being
converted and passed on. The boundary is the target's answer to a
`SiteDescriptor` whose source function is absent and whose declaration is the
*type's*, so the target resolves the type's
[choice](../../stages/03-requests.md#what-a-choice-records) rather than a
function's. The registry checks it as it checks any other: the wrapper parameter must carry what the
release reads, and any failure the release could raise must have a route.

The [conversions](../../stages/04-select.md#select-conversion-relations) of
`ledger_open` and `ledger_close` meet their own boundaries in those functions'
plans. A handle parameter is placed as any input is, and a handle return is
delivered as any scalar result is; what this path adds to them is a failure
category, `Binding`, that the boundary must route.

## Checks

- A release is infallible, so its boundary needs no route; a release described
  as fallible would need one, and the type would be skipped without it.
- The plan is retained under the type's identity, so it survives exactly when
  the type does, and a report never lists it as a function the user asked for.
- A boundary that cannot place a release — a target with no destructor
  convention — refuses the type here, and everything requiring the type with
  it.

## Language variants

- [C][typedef_boundary_c]
- [Kotlin/JNI][typedef_boundary_jni]

[typedef]: README.md
[typedef_represent]: 05-represent.md
[typedef_retain]: 07-retain.md
[typedef_boundary_c]: 06-boundary.c.md
[typedef_boundary_jni]: 06-boundary.jni.md
