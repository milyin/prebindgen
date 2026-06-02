# prebindgen-opaque-types

Build-time generator of opaque `#[repr(C, align(_))]` counterpart structs for
prebindgen's inline-by-value FFI (`lang::Cbindgen::value_opaque`).

For each Rust type it probes the **target** size/alignment (by compiling a tiny
probe crate for `$TARGET` and reading `#[no_mangle] static` values back from the
artifact with the `object` crate — no target-code execution, so it works under
cross-compilation) and emits an opaque struct of identical layout, e.g.

```rust
#[repr(C, align(8))]
pub struct z_zbytes_t { _0: [u8; 32] }
```

`lang::Cbindgen` transmutes the real Rust value to/from this opaque type by value
(no `Box`) and emits `const _` size/align equality asserts, so a wrong probe fails
the consumer's build (fail-closed) rather than corrupting memory.

Call `generate(&Config)` from a consumer `build.rs`; `include!` the result into the
crate and feed it to cbindgen so the opaque struct also appears in the C header.
