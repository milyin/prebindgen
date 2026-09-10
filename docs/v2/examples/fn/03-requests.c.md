<!-- spec: {"kind": "variant", "example": "fn", "stage": "03-requests", "language": "c"} -->

[Stage chapter](../../stages/03-requests.md) · [Common cell][fn_requests] · [Element path][fn]
Owner: the C frontend

# Function taking an owned record — Record binding requests — C

## Input

```rust
// build.rs
Cbindgen::builder()
    .source(source_crate::PREBINDGEN_OUT_DIR)
    .source_module(parse_quote!(source_crate))
    .module(
        module!().data_type(
            data_type!(Stamp)              // see the record path
                .method(fun!(stamp_sum)),
        ),
    )
    .build();
```

## Result

```text
policy (C function):
    symbol:     "stamp_sum"      // the source name; no hook renamed it
    convention: extern "C"
    input:      by value at its ABI position
    output:     native return
    failures:   none declared
```

## Checks

- The symbol is the source name unless a naming hook on the builder changes it.
- C is opt-in: a function nobody declares produces no request, and is reported
  as unselected rather than skipped.
- Declaring the function without declaring `Stamp` is a valid request that fails
  later, at [retention][fn_retain].

[fn]: README.md
[fn_requests]: 03-requests.md
[fn_retain]: 06-retain.md
