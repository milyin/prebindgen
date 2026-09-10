<!-- spec: {"kind": "variant", "example": "fn", "stage": "04-values", "language": "c"} -->

[Stage chapter](../../stages/04-values.md) · [Common cell][fn_values] · [Element path][fn]
Owner: the registry, on the C adapter's representation

# Function taking an owned record — Plan value conversions — C

## Input

The two crossings under the C policy: a by-value aggregate for the record, a
scalar carrier for the result.

## Result

```text
node(input)  representation: Aggregate { ty: Stamp (repr(C)), members: [secs, nanos] }
             operations:     ordinary member reads (see below)
             contract:       produces an owned source Stamp
                             validity Independent, failures {}

node(output) representation: Scalar(c_i64)
             contract:       produces the carrier, validity Independent, failures {}
```

The member reads are specified, with the fragment each renders, in
[the record's C value plan][struct_values_c].

## Checks

- Both nodes are infallible — reading a member of a by-value struct cannot fail,
  and the copied integer owes nothing to the aggregate — so this function has no
  failure route to plan, which is what [its C boundary][fn_boundary_c] relies on.
- The representation maps every source child to a member, and the registry
  validates that mapping; an unaccounted member is an error, not a dropped field.
- A field type that made its conversion fallible would add a failure here, and
  the C policy would have to route it or the function would be skipped.

[fn]: README.md
[fn_values]: 04-values.md
[fn_boundary_c]: 05-boundary.c.md
[struct_values_c]: ../struct/04-values.c.md
