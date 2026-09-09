<!-- spec: {"kind": "variant", "example": "struct", "stage": "03-requests", "language": "c"} -->

[Stage chapter](../../stages/03-requests.md) · [Common cell][struct_requests] · [Element path][struct]

# Record with scalar fields — Record binding requests — C

## Input

A `build.rs` declaring `Stamp` as a data struct on the C frontend's builder.

## Owner

The C frontend, whose declarator choice is also its statement about the
representation: a data struct is the by-value aggregate, as opposed to an opaque
pointer handle or a value-opaque type.

## Result

The record's policy selects a C aggregate with two signed 64-bit members, in the
declaration's field order, passed and returned by value, and named `Stamp` — the
source name, since the configuration asked for no other. Because
the aggregate's members are the record's fields, the members are read directly
and no runtime is involved.

## Checks

The C name defaults to the source name, and here they coincide — which makes it
worth saying that they remain two different things. The policy carries a C name
that a naming hook could change without touching the source; the aggregate's
members are target member identities, distinct from the record's source field
identities even when both spell `secs`. Declaring the type
under a different declarator — an opaque handle — would be a different policy and
therefore a different conversion node, not a variation of this one.

## Representation

```rust
// build.rs, C frontend (schematic)
CbindgenBuilder::new()
    .data_struct("Stamp")
    .build();
```

```text
policy (C record):
  representation: data_struct
  c_name:         "Stamp"
  members:        secs: int64_t, nanos: int64_t   // declaration order
  passing:        by value
```

## Along this element

See the [common cell][struct_requests] for the request and the parts both targets share.

[struct]: README.md
[struct_requests]: 03-requests.md
