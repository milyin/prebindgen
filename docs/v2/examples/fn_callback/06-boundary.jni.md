<!-- spec: {"kind": "variant", "example": "fn_callback", "stage": "06-boundary", "language": "jni"} -->

[Stage chapter](../../stages/06-boundary.md) · [Common cell][fn_callback_boundary] · [Element path][fn_callback]
Owner: the registry, on the JNI frontend's function form

# Function taking a callback — Assemble the wrapper boundary — Kotlin/JNI

## Input

```text
node(stamp) : produces an owned source Stamp, failures { Runtime: jni::errors::Error }
node(each)  : produces the closure,          failures { Runtime: jni::errors::Error }   // capture's

the form the JNI frontend recorded for `fn:stamp_each`:
  symbol "Java_example_JNINative_stampEach", extern "system",
  context [mut env: JNIEnv supplies "jni.env", _this: JObject unused], inputs [stamp, each],
  Runtime -> report to the JVM, then return a default
  Binding -> throw the message, then return a default
```

## Result

```text
FunctionPlan {
    abi, symbol: extern "system", "Java_example_JNINative_stampEach",
    params:      [ mut env: JNIEnv (supplies jni.env), _this: JObject (unused),
                   stamp: JObject -> node(stamp), each: JObject -> node(each) ],
    ret:         none,
    routes:      { Runtime: report through report_jni_error, then return ();
                            if reporting fails -> abort,
                   Binding: throw the message, then return () },
}
```

This plan determines the signature that [emission][fn_callback_emit_jni]
renders:

```rust
#[no_mangle]
pub extern "system" fn Java_example_JNINative_stampEach(
    mut env: jni::JNIEnv<'_>,
    _this: jni::objects::JObject<'_>,
    stamp: jni::objects::JObject<'_>,
    each: jni::objects::JObject<'_>,
)
```

`capture` asks for `jni.env`, which `env` supplies, as a getter does. The
[wrapper](../../stages/06-boundary.md#assemble-the-wrapper-boundary) returns nothing, so the terminal action of each route is `return ()`.

## Checks

- A failed capture — a class, a method or a global reference the JVM refused —
  is reported to the JVM like a failed getter, and `stamp_each` is not called.
- The wrapper's `env` is not what a call uses: a call attaches its own
  thread, since the wrapper's environment belongs to the thread and the frame
  that called it.

[fn_callback]: README.md
[fn_callback_boundary]: 06-boundary.md
[fn_callback_emit_jni]: 08-emit.jni.md
