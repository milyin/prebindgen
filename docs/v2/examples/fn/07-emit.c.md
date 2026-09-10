<!-- spec: {"kind": "variant", "example": "fn", "stage": "07-emit", "language": "c"} -->

[Stage chapter](../../stages/07-emit.md) · [Common cell][fn_emit] · [Element path][fn]
Owner: the common Rust writer, then `cbindgen`

# Function taking an owned record — Emit bindings — C

## Input

```text
FunctionPlan(exported stamp_sum) frozen, with
    boundary: extern "C", symbol "stamp_sum", arg 0 by value, return int64_t
    node(input):  Aggregate { ty: Stamp, members: [secs, nanos] }, member reads
    node(output): Scalar(c_i64), identity
```

## Result

In the generated C Rust module (`c.rs`), after
[the aggregate declaration][struct_emit_c]:

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

The header `cbindgen` derives from it, after the aggregate's `typedef`:

```c
int64_t stamp_sum(struct Stamp arg0);
```

And a C caller:

```c
#include "bindings.h"

int main(void) {
    Stamp stamp = { .secs = 12, .nanos = 34 };
    return stamp_sum(stamp) == 46 ? 0 : 1;
}
```

## Checks

- The wrapper and the function it wraps are both `stamp_sum`, and never collide:
  the source one is only ever reached through its module path.
- `arg0.secs` and `arg0.nanos` are the target's operations; the order, the
  locals, the construction, the call and the return are the registry's.
- Compiling the module with the source crate must succeed, and the caller above
  must observe `46`.
- The C adapter writes no foreign source of its own: there is no generated C
  implementation file, because the body is the Rust wrapper.

[fn]: README.md
[fn_emit]: 07-emit.md
[struct_emit_c]: ../struct/07-emit.c.md
