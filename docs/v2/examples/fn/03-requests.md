<!-- spec: {"kind": "cell", "example": "fn", "stage": "03-requests"} -->

[Stage chapter](../../stages/03-requests.md) · [Element path][fn] · [Source crate](../../source.md)
Owner: the language frontend · Previous: [Build and inspect the source model][fn_flat] · Next: [Select conversion relations][fn_select]

# Function taking an owned struct — Record binding requests

## Input

The element Flat built for this function:

```text
Element::Function(Function {
    name:   stamp_sum,
    params: [ Param { name: stamp, ty: Named { id: Stamp } } ],
    ret:    Scalar(I64),
})
```

The binding build script additionally asks to expose the function. The
[C configuration][fn_requests_c] and [Kotlin/JNI configuration][fn_requests_jni]
make that choice through their respective frontend APIs. Knowing that the
function exists and asking to export it are separate inputs.

## Result

The request associates the source function with a public declaration and the
adapter's function settings. This sketch names the two positions whose values
need [conversion](../../stages/04-select.md#select-conversion-relations): the first parameter and the return value.
The block uses design notation, not exact current fields. Current
`OutputRequest` contains a `Declaration` — which names its origin, here
`Origin::Function(stamp_sum)` — and a `PolicyId`; the engine uses
`Position { declaration, path }` rather than the proposed `SiteId` below.

```text
OutputRequest {
    id:     DeclarationId("fn:stamp_sum"),
    source: SourceItemId(crate::source::stamp_sum),
    policy: PolicyId(this target's function policy),
}

sites:
    SiteId { owner: <that DeclarationId>, path: Param(0) }   // the owned Stamp
    SiteId { owner: <that DeclarationId>, path: Return }     // the i64

site_policies: {}             // no position-specific overrides
type_policies: { Stamp -> <the struct's policy> }
```

The struct's [representation](../../stages/05-represent.md#represent-and-compose-values) is not decided here: it comes from
[the struct's request][struct_requests] and applies wherever a `Stamp` is
converted.

## Checks

- Current ids combine the kind and Rust name. Requesting two placements of the
  same function is not yet supported; the stage chapter describes that extension.
- A frontend setting V2 cannot translate becomes a request under a [policy](../../stages/03-requests.md#what-policy-means)
  that reports the missing capability. It must not disappear silently.
- Nothing here claims the function can be generated.

## Language variants

- [C][fn_requests_c]
- [Kotlin/JNI][fn_requests_jni]

[fn]: README.md
[fn_flat]: 02-flat.md
[fn_select]: 04-select.md
[fn_requests_c]: 03-requests.c.md
[fn_requests_jni]: 03-requests.jni.md
[struct_requests]: ../struct/03-requests.md
