<!-- spec: {"kind": "variant", "example": "fn_callback", "stage": "08-emit", "language": "c"} -->

[Stage chapter](../../stages/08-emit.md) · [Common cell][fn_callback_emit] · [Element path][fn_callback]
Owner: the common Rust writer, then `cbindgen`

# Function taking a callback — Emit bindings — C

## Input

```text
Retained(callback:impl Fn(i64)+Send+Sync+'static), whose wire type is fed to the C writer:
    WireTypeFeed { wire_type: closure_i64 (Closure, c_name "closure_i64"), members: [ arg 0: i64 ] }
FunctionPlan(fn:stamp_each) frozen, with
    abi, symbol: extern "C", "stamp_each"
    params:      [ stamp: Stamp, each: closure_i64 ], no return
```

## Result

What the C writer returns for the
[wire type](../../stages/05-represent.md#describing-target-values-and-operations),
in the generated C Rust module (`c.rs`):

```rust
#[repr(C)]
#[allow(non_camel_case_types)]
pub struct closure_i64 {
    /// The caller's own state, handed to `call` and to `drop`.
    pub context: *mut ::core::ffi::c_void,
    /// Called on every call of the callback, with its arguments
    /// and `context`, from whichever thread Rust calls it on. When
    /// null, a call does nothing.
    pub call: ::core::option::Option<
        unsafe extern "C" fn(i64, *mut ::core::ffi::c_void),
    >,
    /// Called once with `context` when Rust lets go of the
    /// callback. When null, nothing frees `context`.
    pub drop: ::core::option::Option<unsafe extern "C" fn(*mut ::core::ffi::c_void)>,
}
unsafe impl ::core::marker::Send for closure_i64 {}
unsafe impl ::core::marker::Sync for closure_i64 {}
impl ::core::ops::Drop for closure_i64 {
    fn drop(&mut self) {
        if let ::core::option::Option::Some(drop) = self.drop {
            unsafe { drop(self.context) }
        }
    }
}
```

`call` takes each argument's wire type, then the context. Each member says what
it is for, and what a null one means, in its own comment — which is where
`cbindgen` carries it into the header, since a C caller reads that and not
this Rust. A null `call` is a callback that is told nothing: the source
function calls it, and nothing happens, as in v1. The two `unsafe impl`s
are the promise a C caller makes by passing one: that its context may be used
and freed from any thread. `Drop` is how the context is freed — when Rust drops
the closure that owns the struct, whether the source function called it
never, once or many times.

The [wrapper](../../stages/06-boundary.md#assemble-the-wrapper-boundary):

```rust
#[no_mangle]
pub extern "C" fn stamp_each(stamp: Stamp, each: closure_i64) {
    let v0 = stamp.secs;
    let v1 = stamp.nanos;
    let v2 = source::Stamp {
        secs: v0,
        nanos: v1,
    };
    let v4 = move |v3: i64| {
        {
            let closure = &each;
            if let ::core::option::Option::Some(call) = closure.call {
                unsafe { call(v3, closure.context) }
            }
        };
    };
    source::stamp_each(v2, v4);
}
```

The header `cbindgen` derives from them:

```c
typedef struct closure_i64 {
  /**
   * The caller's own state, handed to `call` and to `drop`.
   */
  void *context;
  /**
   * Called on every call of the callback, with its arguments
   * and `context`, from whichever thread Rust calls it on. When
   * null, a call does nothing.
   */
  void (*call)(int64_t, void*);
  /**
   * Called once with `context` when Rust lets go of the
   * callback. When null, nothing frees `context`.
   */
  void (*drop)(void*);
} closure_i64;

void stamp_each(struct Stamp stamp, struct closure_i64 each);
```

And a C caller, collecting the values into an array it owns:

```c
typedef struct { int64_t values[2]; int count; } seen_t;

static void remember(int64_t value, void *context) {
    seen_t *seen = context;
    seen->values[seen->count++] = value;
}

seen_t seen = { .count = 0 };
Stamp stamp = { .secs = 12, .nanos = 34 };
stamp_each(stamp, (closure_i64){ .context = &seen, .call = remember, .drop = NULL });
/* seen.values is { 12, 34 } */
```

## Checks

- `v2check` calls this wrapper from Rust as C would: the call runs once per
  field, in field order, with the context the caller gave, and `drop` runs
  once, after the source function is done with the callback.
- A caller whose context outlives the call — a subscriber, say — frees it in
  `drop`, which Rust calls whenever it lets the callback go; one whose context
  is on the stack, as above, passes no `drop` and relies on the source
  function not keeping the callback past the call.
- The header names the struct and its three members exactly as the Rust
  declaration lays them out, because `repr(C)` is what `cbindgen` reads.

[fn_callback]: README.md
[fn_callback_emit]: 08-emit.md
