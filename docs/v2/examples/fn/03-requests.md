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

and a build script asking for it to be exposed. What that call looks like is the
language's business: [C][fn_requests_c], [Kotlin/JNI][fn_requests_jni].

## Result

```text
OutputRequest {
    id:     ElementId(exported stamp_sum, at this target's placement),
    source: SourceItemId(crate::source::stamp_sum),
    policy: PolicyId(this target's function policy),
}

sites:
    SiteId { owner: <that ElementId>, path: Param(0) }   // the owned Stamp
    SiteId { owner: <that ElementId>, path: Return }     // the i64

conversion_rules.sites: {}    // none recorded for this path
```

The record's representation is not decided here: it comes from
[the record's request][struct_requests] and applies wherever a `Stamp` is
converted.

## Checks

- Exposing the same function at two placements gives two `ElementId`s with
  independent outcomes.
- A setting the frontend cannot translate is recorded as an
  `UnsupportedRequest` and reaches the report; an override naming a parameter
  this function does not have fails the build.
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
