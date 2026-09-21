<!-- spec: {"kind": "variant", "example": "fn", "stage": "05-represent", "language": "c"} -->

[Stage chapter](../../stages/05-represent.md) · [Common cell][fn_represent] · [Element path][fn]
Owner: the registry, on the C adapter's [representation](../../stages/05-represent.md#represent-and-compose-values)

# Function taking an owned struct — Represent and compose values — C

## Input

The two selected [conversions](../../stages/04-select.md#select-conversion-relations), each with the
[conversion key](../../stages/03-requests.md#finding-an-existing-conversion-plan)
`select` returned for it — which for this adapter is the C
[policy](../../stages/03-requests.md#what-policy-means) itself:

```text
Stamp, IntoRust,  Stamp.fields [secs: i64 atomic, nanos: i64 atomic]   conversion: data_struct, by value
i64,   OutOfRust, atomic                                               conversion: scalar carrier
```

## Result

```text
node(input)  representation: Aggregate { ty: Stamp (repr(C)), members: [secs, nanos] }
             operations:     ordinary member reads (see below)
             contract:       produces an owned source Stamp
                             validity Independent, failures {}

node(output) representation: Scalar(c_i64)
             contract:       produces the carrier, validity Independent, failures {}
```

`Aggregate` means the [wrapper](../../stages/06-boundary.md#assemble-the-wrapper-boundary)'s input is one struct containing the two members.
`Scalar(c_i64)` means the output uses one signed 64-bit value. The names here
summarize the plan rather than declare Rust types. `Independent` describes the
fact that copied integers and the reconstructed struct do not borrow the input;
the full validity-contract API is future work.

The registry combines the two field reads with Rust struct construction.
[The struct's C page][struct_represent_c] explains those reads individually.

## Checks

- Both [nodes](../../stages/05-represent.md#represent-and-compose-values) are infallible — reading a member of a by-value struct cannot fail,
  and the copied integer owes nothing to the aggregate — so this function has no
  failure route to plan, which is what [its C boundary][fn_boundary_c] relies on.
- The representation maps every source child to a member, and the registry
  validates that mapping; an unaccounted member is an error, not a dropped field.
- A field type that made its conversion fallible would add a failure here, and
  the C policy would have to route it or the function would be skipped.

[fn]: README.md
[fn_represent]: 05-represent.md
[fn_boundary_c]: 06-boundary.c.md
[struct_represent_c]: ../struct/05-represent.c.md
