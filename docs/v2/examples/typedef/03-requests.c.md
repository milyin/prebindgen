<!-- spec: {"kind": "variant", "example": "typedef", "stage": "03-requests", "language": "c"} -->

[Stage chapter](../../stages/03-requests.md) · [Common cell][typedef_requests] · [Element path][typedef]
Owner: the C frontend

# Type alias declaring an opaque handle — Record binding requests — C

## Input

```rust
// build.rs
Cbindgen::builder()
    .source(source_crate::PREBINDGEN_OUT_DIR)
    .source_module(parse_quote!(source_crate))
    .declare(
        decls!()
            .ptr_type(ptr_type!(Ledger))
            .fun(fun!(ledger_open))
            .fun(fun!(ledger_close)),
    )
    .build_with(prebindgen_c::pipeline::Pipeline::V2)
    .expect("generate the C binding");
```

## Result

```text
policy (C handle):
    representation: opaque_ptr
    c_name:         "Ledger"          // the incomplete C type; the frontend's default is `ledger`
    carrier:        Ledger *          // the address of a Rust-owned value
    release:        "ledger_drop"     // <base>_drop, exported beside the functions
    null handle:    abort             // C has no exception; the process stops
```

`ptr_type!` is the declarator for an opaque pointer, as `data_type!` is for a
by-value aggregate. The frontend's manglers name everything: the type through
`mangle_type_name`, the release through `mangle_destructor`, both over the base
`mangle_rust_type` derives from the Rust name. The specification's example
keeps the source name; a real binding usually reads `ledger_t` and
`ledger_drop`.

## Checks

- The declarator is the
  [representation](../../stages/05-represent.md#represent-and-compose-values).
  A struct may be declared with `ptr_type!` too, and then crosses whole
  without its fields being read.
- The release symbol derives from the same base as the type name, so renaming
  the base moves the type, the pointer and the destructor together.
- A null handle passed where a value is consumed is a binding failure. C offers
  nowhere to report one from a function returning `int64_t`, so the frontend's
  convention is the one v1 calls `.panic()`: abort. Routing it to a `Result`
  out-parameter is the `fn_fallible` path's business.

[typedef]: README.md
[typedef_requests]: 03-requests.md
