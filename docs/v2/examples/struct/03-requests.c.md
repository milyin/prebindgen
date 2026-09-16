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
    .declare(decls!().data_type(data_type!(Stamp).base_name("Stamp")))
    .build();
```

## Result

`data_type!(Stamp)` requests a data representation rather than an opaque handle.
The generated C struct will expose both fields and be passed by value.
`.base_name("Stamp")` chooses the public type name explicitly; it is not the
default snake-case name. The following summarizes the intended representation;
the planner obtains the member types from Flat in the next stage.

```text
policy (C record):
    representation: data_struct
    c_name:         "Stamp"                       // this example's name; the default base is `stamp`
    members:        secs: int64_t, nanos: int64_t // declaration order
    passing:        by value
```

## Checks

- The `data_type!` declaration selects the by-value aggregate,
  as opposed to an opaque pointer handle or a value-opaque type. Declaring the
  type under a different one is a different policy, and therefore a different
  conversion node if that representation is supported. This increment does not
  implement the opaque alternatives.
- The C name is the frontend's, not the source's: a type's default base is the
  snake_case of its short name, so `Stamp` would reach C as `stamp` — which is
  why this declaration names it, and why the two names stay two things. The
  aggregate's member identities are likewise distinct from the record's source
  field identities, even when both spell `secs`.

[struct]: README.md
[struct_requests]: 03-requests.md
