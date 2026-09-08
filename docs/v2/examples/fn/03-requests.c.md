<!-- spec: {"kind": "variant", "example": "fn", "stage": "03-requests", "language": "c"} -->

# Function taking an owned record — Record binding requests — C

[Stage chapter](../../stages/03-requests.md) · [Common cell][fn_requests] · [Element path][fn]

## Input

A `build.rs` that declares `stamp_sum` on the C frontend's builder, alongside the
declaration of [`Stamp` as a data struct][struct_requests_c].

## Owner

The C frontend. It decides the exported symbol, which by default is the source
function's own name.

## Result

The function request carries a C function policy: the exported symbol
`stamp_sum`, the default calling convention, the argument delivered by value,
and the result delivered through the native return. C is opt-in, so a function
that is not declared produces no request and is reported as unselected rather
than skipped.

## Checks

The exported symbol is `stamp_sum` because nothing asked for anything else; a
naming hook configured on the builder would change it, and would be the only
thing that could. Either way the answer is the one the V1 pipeline gives for this
declaration. Declaring
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
  symbol:     "stamp_sum"
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
