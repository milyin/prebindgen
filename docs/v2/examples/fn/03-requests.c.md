<!-- spec: {"kind": "variant", "example": "fn", "stage": "03-requests", "language": "c"} -->

# Function taking an owned record — Record binding requests — C

[Stage chapter](../../stages/03-requests.md) · [Common cell][fn_requests] · [Element path][fn]

## Input

A `build.rs` that declares `stamp_sum` on the C frontend's builder, alongside the
declaration of [`Stamp` as a data struct][struct_requests_c].

## Owner

The C frontend. Its manglers decide the exported symbol; nothing in the source
crate names it.

## Result

The function request carries a C function policy: the exported symbol
`stamp_sum_c`, the default calling convention, the argument delivered by value,
and the result delivered through the native return. C is opt-in, so a function
that is not declared produces no request and is reported as unselected rather
than skipped.

## Checks

The exported name comes from the frontend's mangler applied to the source name,
which is the same answer the V1 pipeline gives for this declaration. Declaring
the function without declaring `Stamp` is a valid request that fails later, at
[retention][fn_retain] — the frontend does not silently add the missing type
declaration.

## Representation

```rust
// build.rs, C frontend (schematic)
CbindgenBuilder::new()
    .data_struct("Stamp")     // see the record path
    .function("stamp_sum")    // this request
    .build();
```

```text
policy (C function):
  symbol:     "stamp_sum_c"
  convention: extern "C"
  input:      by value at its ABI position
  output:     native return
  failures:   none declared; this function has no fallible route
```

## Along this element

See the [common cell][fn_requests] for the request identities both targets share.

[fn]: README.md
[fn_requests]: 03-requests.md
[fn_retain]: 06-retain.md
[struct_requests_c]: ../struct/03-requests.c.md
