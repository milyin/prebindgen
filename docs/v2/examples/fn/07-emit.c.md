<!-- spec: {"kind": "variant", "example": "fn", "stage": "07-emit", "language": "c"} -->

# Function taking an owned record — Emit bindings — C

[Stage chapter](../../stages/07-emit.md) · [Common cell][fn_emit] · [Element path][fn]

## Input

The frozen C function plan and the aggregate representation of the record.

## Owner

The common Rust writer renders the wrapper; `cbindgen` derives the header
declaration from it. The C adapter writes no foreign source of its own, and there
is no generated C implementation file — the function body is the Rust wrapper.

## Result

The generated Rust module contains the wrapper below, next to
[the aggregate declaration][struct_emit_c] it takes by value. The header declares
the same function; a C caller passing `{12, 34}` gets `46`.

## Checks

Compiling the module together with the source crate must succeed, and a C caller
must observe `46` for that input. The exported symbol and the by-value ABI must
match what [the boundary][fn_boundary_c] fixed. The two member reads are common
Rust operations: nothing in this output requires a C-specific renderer.

## Representation

In the generated C Rust module (`c.rs`), after the aggregate declaration:

```rust
use crate::source;

#[no_mangle]
pub extern "C" fn stamp_sum(arg0: Stamp) -> i64 {
    let v0 = arg0.secs;
    let v1 = arg0.nanos;
    let v2 = source::Stamp { secs: v0, nanos: v1 };
    let v3 = source::stamp_sum(v2);
    v3
}
```

The wrapper and the function it wraps are both called `stamp_sum`: the C name
defaults to the source name, and the two never collide because the source one is
only ever reached through its module path. The C adapter selects `repr(C)`, the
extern calling convention and the public names — here, the source names. The
common writer renders those declarations. The expressions `arg0.secs`
and `arg0.nanos` implement the selected target primitives. The registry supplies
their order, the local bindings, the source-record construction, the source call
and the return placement. All native local names are allocated by the common
writer.

`cbindgen` derives the declaration, following the aggregate's `typedef`:

```c
int64_t stamp_sum(struct Stamp arg0);
```

A caller can use it as follows:

```c
#include "bindings.h"

int main(void) {
    Stamp stamp = { .secs = 12, .nanos = 34 };
    return stamp_sum(stamp) == 46 ? 0 : 1;
}
```

## Along this element

See the [common cell][fn_emit] for the instruction-by-instruction mapping.

[fn]: README.md
[fn_emit]: 07-emit.md
[fn_boundary_c]: 05-boundary.c.md
[struct_emit_c]: ../struct/07-emit.c.md
