<!-- spec: {"kind": "variant", "example": "fn", "stage": "08-emit", "language": "c"} -->

[Stage chapter](../../stages/08-emit.md) · [Common cell][fn_emit] · [Element path][fn]
Owner: the common Rust writer, then `cbindgen`

# Function taking an owned struct — Emit bindings — C

## Input

```text
FunctionPlan(fn:stamp_sum) frozen, with
    abi, symbol:  extern "C", "stamp_sum"
    params:       [ stamp: Stamp ],  ret: i64
    node(input):  Parts over the Stamp aggregate, ReadMember per part
    node(output): Whole over i64, Identity
```

## Result

In the generated C Rust module (`c.rs`), after
[the aggregate declaration][struct_emit_c]:

```rust
#[no_mangle]
pub extern "C" fn stamp_sum(stamp: Stamp) -> i64 {
    let v0 = stamp.secs;
    let v1 = stamp.nanos;
    let v2 = source::Stamp { secs: v0, nanos: v1 };
    let v3 = source::stamp_sum(v2);
    v3
}
```

The argument `Stamp` is the generated C-compatible type. `source::Stamp` is the
original Rust type. `v0` and `v1` copy the input members; `v2` constructs the
original type; `v3` stores the result of the one source call. The qualified
`source::stamp_sum` path distinguishes that implementation from the exported
function with the same short name.

Here `source` is the fixture's local module. A real binding reaches its
[source items](../../stages/01-source.md#capture-source-items) through its
configured `.source_module(...)` path or the captured source crate's name.

The header `cbindgen` derives from it, after the aggregate's `typedef`:

```c
int64_t stamp_sum(struct Stamp stamp);
```

A C caller includes the generated header and links the binding library. This
small program checks a successful call by returning zero only when the sum is 46:

```c
#include "bindings.h"

int main(void) {
    Stamp stamp = { .secs = 12, .nanos = 34 };
    return stamp_sum(stamp) == 46 ? 0 : 1;
}
```

`v2check` compiles the generated Rust and calls its C-compatible entry point
from Rust. It does not run `cbindgen` or compile this C program; the header and
caller above illustrate the intended C use, rather than an additional executed
test.

## Checks

- The [wrapper](../../stages/06-boundary.md#assemble-the-wrapper-boundary) and the function it wraps are both `stamp_sum`, and never collide:
  the source one is only ever reached through its module path.
- The parameter keeps the source parameter's name, `stamp`, as the boundary
  stated it; the locals are the writer's, numbered so that none can shadow it.
- `stamp.secs` and `stamp.nanos` are the target's operations; the order, the
  locals, the construction, the call and the return are the registry's.
- Compiling the module with the source crate must succeed, and the caller above
  must observe `46`.
- The C adapter writes no foreign source of its own: there is no generated C
  implementation file, because the body is the Rust wrapper.

[fn]: README.md
[fn_emit]: 08-emit.md
[struct_emit_c]: ../struct/08-emit.c.md
