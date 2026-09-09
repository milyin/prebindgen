<!-- spec: {"kind": "cell", "example": "fn", "stage": "03-requests"} -->

# Function taking an owned record — Record binding requests

[Stage chapter](../../stages/03-requests.md) · [Element path][fn] · [Source crate](../../source.md)
Previous: [Build and inspect the source model][fn_flat] · Next: [Plan value conversions][fn_values]

## Input

The published source model, and the user's call on a language frontend asking for
`stamp_sum` to be part of the generated API.

## Owner

The language frontend. It resolves the name against the source model, records the
choice, and later translates it into `BindingRequests`.

## Result

One `OutputRequest` whose `source` is the captured `stamp_sum` and whose `policy`
points at the entry holding this target's choices for the function. It fixes two
sites, which is how every later decision about this function is addressed:
`Param(0)` for the argument and `Return` for the result. The configuration records no
per-site override, so both sites take the target's defaults; the record's own
representation is chosen by [the record's request][struct_requests] and applies
wherever that record is converted.

No conversion is planned here, and no support is claimed. The request says which
function, at which foreign placement, under which policy.

## Checks

Exposing the same Rust function at two foreign placements produces two
`OutputRequest`s with different `ElementId`s and independent outcomes. A setting
the frontend cannot translate — a naming or error convention it has not
implemented — becomes an `UnsupportedRequest` here, and survives into the report
rather than being dropped. A contradictory configuration, such as an override on
a parameter this function does not have, is invalid input and fails the build.

## Representation

For this path, schematically:

```text
OutputRequest {
  id:     ElementId(exported stamp_sum, in this target's placement),
  source: SourceItemId(crate::source::stamp_sum),
  policy: PolicyId(function policy for this target),
  requirements: [],   // no promised interface or convention beyond the defaults
}

sites:
  SiteId { owner: <that ElementId>, path: Param(0) }  -> owned Stamp input
  SiteId { owner: <that ElementId>, path: Return }    -> i64 result

conversion_rules.sites:  {}   // none recorded for this path
```

The two targets fill `policy` differently, and that difference is the whole of
the divergence at this stage.

## Language variants

- [C][fn_requests_c]
- [Kotlin/JNI][fn_requests_jni]

[fn]: README.md
[fn_flat]: 02-flat.md
[fn_values]: 04-values.md
[fn_requests_c]: 03-requests.c.md
[fn_requests_jni]: 03-requests.jni.md
[struct_requests]: ../struct/03-requests.md
