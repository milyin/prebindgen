<!-- spec: {"kind": "variant", "example": "typedef", "stage": "08-emit", "language": "c"} -->

[Stage chapter](../../stages/08-emit.md) · [Common cell][typedef_emit] · [Element path][typedef]
Owner: the common Rust writer, then `cbindgen`

# Type alias declaring an opaque handle — Emit bindings — C

## Input

```text
SurfaceSpec(public Ledger) frozen, with
    rust: [ the incomplete type Ledger ]
FunctionPlan(release of Ledger) frozen, with
    boundary: extern "C", symbol "ledger_drop", arg this_ *mut Ledger, no return
    body:     Release(this_)
```

## Result

In the generated C Rust module (`c.rs`), the declaration among the
[artifacts](../../stages/05-represent.md#individual-target-operations) and
the release among the
[wrappers](../../stages/06-boundary.md#assemble-the-native-boundary):

```rust
#[repr(C)]
#[allow(non_camel_case_types)]
pub struct Ledger {
    _private: [u8; 0],
}

#[no_mangle]
pub extern "C" fn ledger_drop(this_: *mut Ledger) {
    drop(
        ::core::ptr::NonNull::new(this_ as *mut source::Ledger)
            .map(|handle| unsafe { Box::from_raw(handle.as_ptr()) }),
    );
}
```

The two wrappers the handle's
[conversions](../../stages/04-select.md#select-conversion-relations) render
into, beside [the struct's][fn_emit_c]:

```rust
#[no_mangle]
pub extern "C" fn ledger_open(stamp: Stamp) -> *mut Ledger {
    let v0 = stamp.secs;
    let v1 = stamp.nanos;
    let v2 = source::Stamp {
        secs: v0,
        nanos: v1,
    };
    let v3 = source::ledger_open(v2);
    let v4 = Box::into_raw(Box::new(v3)) as *mut Ledger;
    v4
}

#[no_mangle]
pub extern "C" fn ledger_close(ledger: *mut Ledger) -> i64 {
    let v0 = match ::core::ptr::NonNull::new(ledger as *mut source::Ledger)
        .map(|handle| unsafe { *Box::from_raw(handle.as_ptr()) })
        .ok_or_else(|| String::from("null `Ledger` handle"))
    {
        Ok(value) => value,
        Err(_) => {
            std::process::abort();
        }
    };
    let v1 = source::ledger_close(v0);
    v1
}
```

The header `cbindgen` derives from them:

```c
typedef struct Ledger Ledger;

void ledger_drop(Ledger *this_);
Ledger *ledger_open(struct Stamp stamp);
int64_t ledger_close(Ledger *ledger);
```

And a C caller:

```c
Stamp stamp = { .secs = 12, .nanos = 34 };
Ledger *ledger = ledger_open(stamp);
int64_t total = ledger_close(ledger);   /* 46; `ledger` is dead now */
ledger_drop(ledger_open(stamp));        /* released without being read */
ledger_drop(NULL);                      /* nothing to release */
```

A struct whose only member is a zero-length array is what `cbindgen` renders
as an incomplete type: C can name `Ledger` and hold a `Ledger *`, and cannot
declare one or read into it. This `Ledger` and `source::Ledger` are different
types, as the aggregate and the struct are: the cast in each expression is
where one becomes the other. The `Err` arm binds nothing, because the route
reports nothing; a route with a reporter binds the error for it.

## Checks

- Compiling the module with the source crate must succeed; the caller above
  must observe `46`, and both `ledger_drop` calls must return. `v2check`
  executes exactly that.
- Passing `ledger` to `ledger_close` a second time is undefined, as freeing
  twice is in C. The handle carries no closed flag on this target.
- `ledger_close(NULL)` aborts the process, and the C header says nothing about
  it: that is the cost of the `.panic()` convention, and the reason a
  binding that wants an error instead declares a `Result` return.

[typedef]: README.md
[typedef_emit]: 08-emit.md
[fn_emit_c]: ../fn/08-emit.c.md
