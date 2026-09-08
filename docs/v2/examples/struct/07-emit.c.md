<!-- spec: {"example": "struct", "kind": "variant", "language": "c", "stage": "07-emit"} -->

# Struct: Stamp — Emit bindings — c

[Pipeline chapter](../../stages/07-emit.md) · [Common contract](07-emit.md) · [Example path](README.md)

## Input

Retained StampC target declaration and source-field mapping.

## Owner

Common Rust writer renders the target Rust struct; cbindgen emits the C header type.

## Result

Expected repr(C) Rust and C header declarations appear below; the two members occur in source order with signed 64-bit types.

## Checks

No custom C foreign writer or C implementation file is generated. Validate aggregate layout and by-value call behavior on supported target platforms during implementation.

## Concrete description or output

```rust
#[repr(C)]
pub struct StampC {
    pub secs: i64,
    pub nanos: i64,
}
```

```c
#include <stdint.h>
typedef struct StampC {
    int64_t secs;
    int64_t nanos;
} StampC;
```

## Related example

[function: c at this stage](../function/07-emit.c.md)
