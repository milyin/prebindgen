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
    .declare(
        decls!()
            .data_type(
                data_type!(Stamp).base_name("Stamp"),   // see the record path
            )
            .fun(fun!(stamp_sum)),
    )
    .build_with(prebindgen_c::pipeline::Pipeline::V2)
    .expect("generate the C binding");
```

Enable the `v2` Cargo feature on `prebindgen-c` for this build. The explicit
`build_with` call selects V2; plain `.build()` defaults to V1 unless
`PREBINDGEN_PIPELINE=v2` is set. Imports are omitted in this excerpt.

## Result

`.fun(fun!(stamp_sum))` creates the function request. The separate
`data_type!(Stamp)` declaration chooses the argument's public C type.
`source_module` names the Rust implementation path for a real source dependency;
the emitted fixture pages use a local module named `source` instead.

The following summarizes the requested calling interface. **ABI** means the
binary calling convention and types a compiled C caller must use.

```text
policy (C function):
    symbol:     "stamp_sum"      // a function keeps its Rust name; no hook renamed it
    convention: extern "C"
    input:      by value at its ABI position
    output:     native return
    failures:   none declared
```

## Checks

- A function keeps its Rust name as the exported symbol unless a builder naming
  hook or per-declaration `.base_name(...)` changes it. Neither does so here,
  which is why the symbol is `stamp_sum`.
- C is opt-in: a function nobody declares produces no request and has no
  [outcome](../../stages/06-retain.md#retain-supported-output). An
  *unselected* outcome is planned, not implemented.
- Without the `Stamp` declaration, the frontend records no record [policy](../../stages/03-requests.md#what-policy-means).
  Value planning then tries the default scalar policy and skips the function
  with `unsupported.c.carrier`. [Retention][fn_retain] preserves that skip;
  it does not first discover the missing [conversion](../../stages/04-values.md#plan-value-conversions).

[fn]: README.md
[fn_requests]: 03-requests.md
[fn_retain]: 06-retain.md
