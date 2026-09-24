<!-- spec: {"kind": "variant", "example": "struct", "stage": "08-emit", "language": "c"} -->

[Stage chapter](../../stages/08-emit.md) · [Common cell][struct_emit] · [Element path][struct]
Owner: the common Rust writer, then `cbindgen`

# Struct with scalar fields — Emit bindings — C

## Input

```text
Retained(type:Stamp), whose root node's carrier is fed to the C writer:
    CarrierFeed { carrier: Stamp (Aggregate, c_name "Stamp"),
                  members: [ secs: i64, nanos: i64 ] }
```

## Result

What the C writer returns for that
[carrier](../../stages/05-represent.md#describing-target-values-and-operations), in the generated C Rust module
(`c.rs`):

```rust
#[repr(C)]
#[allow(non_camel_case_types)]
pub struct Stamp {
    pub secs: i64,
    pub nanos: i64,
}
```

`#[repr(C)]` tells Rust to lay out this generated struct using the C-compatible
rules. That promise applies to this boundary type, not to the original
`source::Stamp`. The [conversion](../../stages/04-select.md#select-conversion-relations) reads the generated struct and constructs the
original one, so the source crate does not need to adopt a C layout.

`cbindgen` reads the generated Rust declaration and produces the corresponding
C header declaration. `<stdint.h>` supplies the exact-width `int64_t` name:

```c
#include <stdint.h>

typedef struct Stamp {
    int64_t secs;
    int64_t nanos;
} Stamp;
```

## Checks

- This `Stamp` is a different type from `source::Stamp` — same spelling,
  different module: one is the ABI type a C caller fills in, the other the Rust
  value the source function takes.
- `repr(C)` is required: without it the layout the header promises is not the
  layout the [wrapper](../../stages/06-boundary.md#assemble-the-wrapper-boundary)
  reads, [as the function path shows][fn_emit_c].
- The binding explicitly chooses `Stamp`. The default type base is `stamp`,
  and a naming hook could choose another convention such as `stamp_t`.
  The generated lint allowance permits such non-CamelCase Rust type names.
- Member order and names match the aggregate description, since
  [the member reads][struct_represent_c] refer to those identities.
- The header's formatting and include guards are `cbindgen`'s business.

[struct]: README.md
[struct_emit]: 08-emit.md
[struct_represent_c]: 05-represent.c.md
[fn_emit_c]: ../fn/08-emit.c.md
