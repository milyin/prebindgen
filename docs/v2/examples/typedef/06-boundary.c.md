<!-- spec: {"kind": "variant", "example": "typedef", "stage": "06-boundary", "language": "c"} -->

[Stage chapter](../../stages/06-boundary.md) · [Common cell][typedef_boundary] · [Element path][typedef]
Owner: the registry, on the C adapter's `BoundarySpec`

# Type alias declaring an opaque handle — Assemble the wrapper boundary — C

## Input

```text
node(taken) : carrier *mut Ledger, release infallible

policy (C handle): release symbol "ledger_drop", extern "C"
                   Binding -> abort
```

## Result

```text
BoundarySpec {
    abi:      extern "C", symbol "ledger_drop",
    inputs:   [ InputPlacement { wrapper arg this_ (*mut Ledger) -> node(taken) } ],
    output:   OutputPlacement::Void,
    failures: { Binding: no report, abort },
}
```

Fixing the signature that [emission][typedef_emit_c] renders:

```rust
#[no_mangle]
pub extern "C" fn ledger_drop(this_: *mut Ledger)
```

The adapter answers a boundary with no source function by reading the release
symbol out of the type's
[policy](../../stages/03-requests.md#what-policy-means), and names the one
parameter `this_`, as v1's destructors do. The same policy fixes the
boundaries of the two functions reaching the handle, which are ordinary
function boundaries with the binding route added:

```rust
#[no_mangle]
pub extern "C" fn ledger_open(stamp: Stamp) -> *mut Ledger
#[no_mangle]
pub extern "C" fn ledger_close(ledger: *mut Ledger) -> i64
```

## Checks

- A function policy at a release boundary, or a handle policy at a function's,
  is contradictory input and fails the build.
- The binding route has no reporting operation: C has no exception, and a
  function returning `int64_t` has no slot for a message. It terminates by
  aborting, which is v1's `.panic()` convention. A `Result` out-parameter would
  be a different `OutputPlacement`, chosen by policy.
- `ledger_drop(NULL)` does not take the route: the release is infallible and a
  null address releases nothing. Only a *consuming*
  [conversion](../../stages/04-select.md#select-conversion-relations) —
  `ledger_close` — fails on null.
- No runtime route is declared, and none is needed: nothing on this target
  calls into a runtime that could fail.

[typedef]: README.md
[typedef_boundary]: 06-boundary.md
[typedef_emit_c]: 08-emit.c.md
