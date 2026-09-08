<!-- spec: {"kind": "variant", "example": "struct", "stage": "07-emit", "language": "c"} -->

# Record with scalar fields — Emit bindings — C

[Stage chapter](../../stages/07-emit.md) · [Common cell][struct_emit] · [Element path][struct]

## Input

The frozen aggregate representation: carrier type, member identities and their
signed 64-bit types.

## Owner

The common Rust writer renders the Rust type from the C adapter's aggregate
description; `cbindgen` derives the header declaration from that Rust.

## Result

A `repr(C)` struct in the generated C Rust module, and the matching `typedef` in
the header. The struct is the ABI type the wrapper takes by value, and the member
reads in [that wrapper][fn_emit_c] are ordinary Rust field accesses on it.

## Checks

The Rust declaration must be `repr(C)`: without it, the member layout the header
promises is not the layout the wrapper reads. Member order and names must match
the aggregate description, since [the member-read operations][struct_values_c]
refer to those identities. The header's shape is `cbindgen` output — formatting
and include guards can differ without the binding being wrong.

## Representation

In the generated C Rust module (`c.rs`):

```rust
#[repr(C)]
pub struct StampC {
    pub secs: i64,
    pub nanos: i64,
}
```

The header `cbindgen` derives:

```c
#include <stdint.h>

typedef struct StampC {
    int64_t secs;
    int64_t nanos;
} StampC;
```

## Along this element

See the [common cell][struct_emit] for what both targets emit for this record.

[struct]: README.md
[struct_emit]: 07-emit.md
[struct_values_c]: 04-values.c.md
[fn_emit_c]: ../fn/07-emit.c.md
