<!-- spec: {"kind": "variant", "example": "fn", "stage": "05-represent", "language": "c"} -->

[Stage chapter](../../stages/05-represent.md) · [Common cell][fn_represent] · [Element path][fn]
Owner: the registry, from the C frontend's [representations](../../stages/05-represent.md#represent-and-compose-values)

# Function taking an owned struct — Represent and compose values — C

## Input

The two selected [conversions](../../stages/04-select.md#select-conversion-relations), each with the
representation its rule named:

```text
Stamp, IntoRust,  Stamp.fields [secs: i64 atomic, nanos: i64 atomic]   Product over `Stamp`, read: ReadMember
i64,   OutOfRust, atomic                                               Terminal over `i64`, Identity
```

## Result

```text
node(input)  carrier:    Stamp (a repr(C) aggregate), members [secs: i64, nanos: i64]
             operations: a standard member read per part (see below)
             produces:   an owned source Stamp; failures {}

node(output) carrier:    i64
             operations: none — the identity renders nothing
             produces:   the carrier; failures {}
```

The [wrapper](../../stages/06-boundary.md#assemble-the-wrapper-boundary)'s input is one struct containing the two members, and
its output one signed 64-bit value. The copied integers and the reconstructed
struct do not borrow the input, which is why no validity contract is needed
yet.

The registry combines the two field reads with Rust struct construction.
[The struct's C page][struct_represent_c] explains those reads individually.

## Checks

- Both [nodes](../../stages/05-represent.md#represent-and-compose-values) are infallible — reading a member of a by-value struct cannot fail,
  and the copied integer owes nothing to the aggregate — so this function has no
  failure route to plan, which is what [its C boundary][fn_boundary_c] relies on.
- Every part is read as a member of the
  [carrier](../../stages/05-represent.md#describing-target-values-and-operations),
  and each member's carrier is
  one the aggregate holds; a member of another wire type refuses the struct,
  not a dropped field.
- A field type that made its conversion fallible would add a failure here, and
  the C function form would have to route it or the function would be skipped.

[fn]: README.md
[fn_represent]: 05-represent.md
[fn_boundary_c]: 06-boundary.c.md
[struct_represent_c]: ../struct/05-represent.c.md
