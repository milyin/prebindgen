<!-- spec: {"kind": "variant", "example": "fn", "stage": "04-select", "language": "c"} -->

[Stage chapter](../../stages/04-select.md) · [Common cell][fn_select] · [Element path][fn]
Owner: the registry, on the C adapter's selections

# Function taking an owned struct — Select conversion relations — C

## Input

The two [crossings](../../stages/03-requests.md#finding-an-existing-conversion-plan) and the [relations](../../stages/04-select.md#what-a-relation-is) the registry offers, beside the C [policy](../../stages/03-requests.md#what-policy-means) the `CTarget` holds for each — which
the registry does not pass in and cannot read:

```text
Crossing { source: Stamp, direction: IntoRust  }   offered: [ Stamp.fields, atomic ]
                                                    CTarget: data_struct named Stamp, by value
Crossing { source: i64,   direction: OutOfRust }   offered: [ atomic ]
                                                    CTarget: scalar carrier
```

## Result

```text
Param(0)  Stamp, IntoRust   -> Stamp.fields, conversion CPolicy::DataStruct { c_name: "Stamp" }
  secs    i64,   IntoRust   -> atomic,       conversion CPolicy::Scalar
  nanos   i64,   IntoRust   -> atomic,       conversion CPolicy::Scalar
Return    i64,   OutOfRust  -> atomic,       conversion CPolicy::Scalar
```

The C adapter selects `Stamp.fields` because the struct is declared as a
`data_struct`: a C struct passed by value is made of its members, so the
[conversion](../../stages/04-select.md#select-conversion-relations) has to be
made of the field conversions. Had the build script declared `Stamp` as an
opaque pointer type instead, the same query would answer `atomic` — the whole
value carried behind a pointer, its fields never read — and the registry would
descend no further. The adapter answers from the policy it looked up for this
position; it has not seen the fields and does not need to.

The two `i64` positions are answered the same way, `atomic`, and the return
position likewise: a scalar has one relation and the C policy for it is a
scalar [carrier](../../stages/05-represent.md#describing-target-values-and-operations).

`CPolicy` is both what the C frontend recorded and this target's
[conversion key](../../stages/03-requests.md#finding-an-existing-conversion-plan):
it is plain data, so the three scalar answers above are equal, and the registry
plans one `i64` conversion per direction rather than one per position.

## Checks

- The answer is one of the offered ids. `Stamp.fields` is the struct relation
  the registry registered from Flat's field list; the adapter did not construct
  it.
- The `data_struct` policy commits the adapter to a member per part in the next
  stage. A policy V2 cannot lower yet — one of the C declarators it does not
  implement — is answered as unsupported here, and the function is
  [skipped with that cause][fn_retain] rather than planned around.
- The selection for `Stamp` is the same one [the struct's C page][struct_select_c]
  shows; this function does not choose differently for its own parameter.

[fn]: README.md
[fn_select]: 04-select.md
[fn_retain]: 07-retain.md
[struct_select_c]: ../struct/04-select.c.md
