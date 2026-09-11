<!-- spec: {"kind": "variant", "example": "struct", "stage": "07-emit", "language": "c"} -->

[Stage chapter](../../stages/07-emit.md) · [Common cell][struct_emit] · [Element path][struct]
Owner: the common Rust writer, then `cbindgen`

# Record with scalar fields — Emit bindings — C

## Input

```text
SurfaceSpec(public Stamp) frozen, with
    payload: Aggregate { c_name: "Stamp",
                         members: [secs: int64_t, nanos: int64_t],
                         passing: by value }
```

## Result

In the generated C Rust module (`c.rs`):

```rust
#[repr(C)]
#[allow(non_camel_case_types)]
pub struct Stamp {
    pub secs: i64,
    pub nanos: i64,
}
```

The header `cbindgen` derives from it:

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
  layout [the wrapper][fn_emit_c] reads.
- The name is the frontend's: the specification's binding keeps the source name,
  and a real binding usually mangles it C-style — `stamp_t` — which is why the
  case lint is silenced on every aggregate.
- Member order and names match the aggregate description, since
  [the member reads][struct_values_c] refer to those identities.
- The header's formatting and include guards are `cbindgen`'s business.

[struct]: README.md
[struct_emit]: 07-emit.md
[struct_values_c]: 04-values.c.md
[fn_emit_c]: ../fn/07-emit.c.md
