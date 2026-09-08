<!-- spec: {"kind": "variant", "example": "struct", "stage": "03-requests", "language": "c"} -->

# Record with scalar fields — Record binding requests — C

[Stage chapter](../../stages/03-requests.md) · [Common cell][struct_requests] · [Element path][struct]

## Input

A `build.rs` declaring `Stamp` as a data struct on the C frontend's builder.

## Owner

The C frontend, whose declarator choice is also its statement about the
representation: a data struct is the by-value aggregate, as opposed to an opaque
pointer handle or a value-opaque type.

## Result

The record's policy selects a C aggregate named `StampC` with two signed 64-bit
members, in the declaration's field order, passed and returned by value. Because
the aggregate's members are the record's fields, the members are read directly
and no runtime is involved.

## Checks

The C name comes from the frontend's mangler, not from the source; the source
name `Stamp` and the C name `StampC` stay distinct throughout, and so do the
source field identities and the target member identities. Declaring the type
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
  c_name:         "StampC"
  members:        secs: int64_t, nanos: int64_t   // declaration order
  passing:        by value
```

## Along this element

See the [common cell][struct_requests] for the request and the parts both targets share.

[struct]: README.md
[struct_requests]: 03-requests.md
