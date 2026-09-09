<!-- spec: {"kind": "variant", "example": "fn", "stage": "04-values", "language": "c"} -->

[Stage chapter](../../stages/04-values.md) · [Common cell][fn_values] · [Element path][fn]

# Function taking an owned record — Plan value conversions — C

## Input

The two crossings with the C policy: an aggregate representation for the record
and a scalar carrier for the result.

## Owner

The registry, using the C adapter's representation and its member-read
operations. The reads themselves are common Rust operations, so C supplies
member identities rather than a renderer.

## Result

The input node carries the `Stamp` aggregate and is infallible: reading a member
of a by-value struct cannot fail, and the copied integers are independent of the
aggregate afterwards. Its operations are [the member reads specified in the
record's C plan][struct_values_c].

The output node maps source `i64` onto the C signed 64-bit carrier. The two are
the same Rust value, so the node's body is empty and its contract is infallible.

With both nodes infallible, this function has no failure route to plan — a fact
[the C boundary][fn_boundary_c] depends on.

## Checks

The aggregate's members are matched to the record's fields by the representation,
and the registry validates that mapping; a member the representation does not
account for is an error, not a silently dropped field. If a future field type
made its conversion fallible, this node would gain a failure and the C policy
would have to declare a route for it or the function would be skipped.

## Representation

```text
node(input)  contract: produced = source Stamp (owned)
                       access   = Owned
                       validity = Independent
                       failures = {}
             representation: Aggregate { ty: Stamp, members: [secs, nanos] }

node(output) contract: produced = carrier c_i64
                       access   = Owned
                       validity = Independent
                       failures = {}
             representation: Scalar(c_i64), identity conversion
```

## Along this element

The record's member-read operations and their rendered fragments are in
[the record's C value plan][struct_values_c].

[fn]: README.md
[fn_values]: 04-values.md
[fn_boundary_c]: 05-boundary.c.md
[struct_values_c]: ../struct/04-values.c.md
