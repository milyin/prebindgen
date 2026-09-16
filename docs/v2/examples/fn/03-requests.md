<!-- spec: {"kind": "cell", "example": "fn", "stage": "03-requests"} -->

[Stage chapter](../../stages/03-requests.md) · [Element path][fn] · [Source crate](../../source.md)
Owner: the language frontend · Previous: [Build and inspect the source model][fn_flat] · Next: [Plan value conversions][fn_values]

# Function taking an owned record — Record binding requests

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
need conversion: the first parameter and the return value.

```text
OutputRequest {
    id:     DeclarationId("fn:stamp_sum"),
    source: SourceItemId(crate::source::stamp_sum),
    policy: PolicyId(this target's function policy),
}

sites:
    SiteId { owner: <that DeclarationId>, path: Param(0) }   // the owned Stamp
    SiteId { owner: <that DeclarationId>, path: Return }     // the i64

conversion_rules.sites: {}    // none recorded for this path
```

The record's representation is not decided here: it comes from
[the record's request][struct_requests] and applies wherever a `Stamp` is
converted.

## Checks

- Current ids combine the kind and Rust name. Requesting two placements of the
  same function is not yet supported; the intended richer identity is described
  in the stage chapter.
- A frontend setting V2 cannot translate becomes a request under a policy
  that reports the missing capability. It must not disappear silently.
- Nothing here claims the function can be generated.

## Language variants

- [C][fn_requests_c]
- [Kotlin/JNI][fn_requests_jni]

[fn]: README.md
[fn_flat]: 02-flat.md
[fn_values]: 04-values.md
[fn_requests_c]: 03-requests.c.md
[fn_requests_jni]: 03-requests.jni.md
[struct_requests]: ../struct/03-requests.md
