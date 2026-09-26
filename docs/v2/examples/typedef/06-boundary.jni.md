<!-- spec: {"kind": "variant", "example": "typedef", "stage": "06-boundary", "language": "jni"} -->

[Stage chapter](../../stages/06-boundary.md) · [Common cell][typedef_boundary] · [Element path][typedef]
Owner: the registry, on the JNI frontend's release form

# Type alias declaring an opaque handle — Assemble the wrapper boundary — Kotlin/JNI

## Input

```text
node(taken) : wire type jlong, release infallible

the release form the JNI frontend recorded on `type:Ledger`'s output:
  symbol "Java_example_JNINative_freeLedger", extern "system",
  context [_env: JNIEnv supplies "jni.env", _this: JObject unused], inputs [ptr],
  Binding -> throw the message, then return a default
  Runtime -> report to the JVM, then return a default
```

## Result

```text
FunctionPlan {
    abi, symbol: extern "system", "Java_example_JNINative_freeLedger",
    params:      [ _env: JNIEnv (supplies jni.env), _this: JObject (unused),
                   ptr: jlong -> node(taken) ],
    ret:         none,
    routes:      { Binding: throw IllegalStateException(message), then return ();
                            if throwing fails -> abort
                   Runtime: report through report_jni_error, then return ();
                            if reporting fails -> abort },
}
```

This plan fixes the signature that [emission][typedef_emit_jni] renders:

```rust
#[no_mangle]
pub extern "system" fn Java_example_JNINative_freeLedger(
    _env: jni::JNIEnv<'_>,
    _this: jni::objects::JObject<'_>,
    ptr: jni::sys::jlong,
)
```

The binding route's reporting operation takes an exclusive environment operand
and the owned `String`, produces nothing, and can fail with a runtime error:

```rust
env.throw_new("java/lang/IllegalStateException", error)
```

The environment is still a parameter — the JVM passes it whether or not the
[wrapper](../../stages/06-boundary.md#assemble-the-wrapper-boundary) wants
it — but nothing in a release uses it, and the frontend names it `_env` so the
generated code says so rather than warning.

## Checks

- Two routes, two error types: the binding route is handed the `String` the
  handle [conversion](../../stages/04-select.md#select-conversion-relations)
  raises, the runtime route the `jni::errors::Error` a JVM call raises. The
  registry checks that each reporter is handed the type its operations produce.
- Both routes are declared on every JNI form, whether or not the
  conversions at hand can take them; a route that is never taken renders
  nothing.
- `freeLedger(0L)` takes no route: the release is infallible. `ledgerClose(0L)`
  throws, and the Kotlin caller sees `IllegalStateException` carrying
  ``null `Ledger` handle`` rather than a returned zero.

[typedef]: README.md
[typedef_boundary]: 06-boundary.md
[typedef_emit_jni]: 08-emit.jni.md
