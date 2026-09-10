<!-- spec: {"kind": "variant", "example": "struct", "stage": "03-requests", "language": "c"} -->

[Stage chapter](../../stages/03-requests.md) · [Common cell][struct_requests] · [Element path][struct]
Owner: the C frontend

# Record with scalar fields — Record binding requests — C

## Input

```rust
// build.rs (schematic)
CbindgenBuilder::new()
    .data_struct("Stamp")
    .build();
```

## Result

```text
policy (C record):
    representation: data_struct
    c_name:         "Stamp"                       // the source name
    members:        secs: int64_t, nanos: int64_t // declaration order
    passing:        by value
```

## Checks

- The declarator is the representation: `data_struct` is the by-value aggregate,
  as opposed to an opaque pointer handle or a value-opaque type. Declaring the
  type under a different one is a different policy, and therefore a different
  conversion node.
- The C name defaults to the source name and a naming hook could change it
  without touching the source; the aggregate's member identities stay distinct
  from the record's source field identities even when both spell `secs`.

[struct]: README.md
[struct_requests]: 03-requests.md
