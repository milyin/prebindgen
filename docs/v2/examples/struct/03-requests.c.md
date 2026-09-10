<!-- spec: {"kind": "variant", "example": "struct", "stage": "03-requests", "language": "c"} -->

[Stage chapter](../../stages/03-requests.md) · [Common cell][struct_requests] · [Element path][struct]
Owner: the C frontend

# Record with scalar fields — Record binding requests — C

## Input

```rust
// build.rs
Cbindgen::builder()
    .source(source_crate::PREBINDGEN_OUT_DIR)
    .source_module(parse_quote!(source_crate))
    .module(module!().data_type(data_type!(Stamp).base_name("Stamp")))
    .build();
```

## Result

```text
policy (C record):
    representation: data_struct
    c_name:         "Stamp"                       // this example's name; the default base is `stamp`
    members:        secs: int64_t, nanos: int64_t // declaration order
    passing:        by value
```

## Checks

- The declarator is the representation: `data_struct` is the by-value aggregate,
  as opposed to an opaque pointer handle or a value-opaque type. Declaring the
  type under a different one is a different policy, and therefore a different
  conversion node.
- The C name is the frontend's, not the source's: a type's default base is the
  snake_case of its short name, so `Stamp` would reach C as `stamp` — which is
  why this declaration names it, and why the two names stay two things. The
  aggregate's member identities are likewise distinct from the record's source
  field identities, even when both spell `secs`.

[struct]: README.md
[struct_requests]: 03-requests.md
