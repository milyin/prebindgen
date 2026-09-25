<!-- spec: {"kind": "variant", "example": "fn_callback", "stage": "05-represent", "language": "jni"} -->

[Stage chapter](../../stages/05-represent.md) · [Common cell][fn_callback_represent] · [Element path][fn_callback]
Owner: the registry; the JNI frontend states the callable's capture, call and route

# Function taking a callback — Represent and compose values — Kotlin/JNI

## Input

```text
impl Fn(i64), IntoRust, callback.args [arg 0: i64 OutOfRust, atomic]
representation: Callable { wire_type: a `JObject` of example.LongCallback,
                           capture: CaptureCallback (jni.env; Runtime),
                           invoke:  CallCallback (Runtime),
                           routes:  [Runtime -> ReportCallbackError, then return] }
```

## Result

```text
node(each)  wire type:  a JObject of example.LongCallback, members [arg 0: jlong]
            capture:  CaptureCallback(each, env) -> captured
            closure:  move |a0: i64| { CallCallback(captured, a0), routed }
            failures: { Runtime: jni::errors::Error }      // capture's only
```

A local JVM reference is valid only until the [wrapper](../../stages/06-boundary.md#assemble-the-wrapper-boundary) returns, so the
object cannot be kept as it arrived. `CaptureCallback` keeps what a call from
any thread needs — the JVM, a global reference to the object, and its `run`
method, looked up once with the descriptor built from the arguments'
[wire types](../../stages/05-represent.md#describing-target-values-and-operations),
`(J)V`:

```rust
(|| -> jni::errors::Result<_> {
    let class = env.get_object_class(&each)?;
    let method = env.get_method_id(&class, "run", "(J)V")?;
    Ok((env.get_java_vm()?, env.new_global_ref(&each)?, method))
})()
```

`CallCallback` is applied on every call to that triple and the argument. It
attaches the calling thread to the JVM — a no-op on a thread already attached —
and calls `run`. If `run` throws, the exception is described and cleared: it
has nowhere to propagate to, and a pending exception would poison the next JNI
call on that thread.

```rust
(|| -> jni::errors::Result<()> {
    let (vm, callable, method) = &captured;
    let mut env = vm.attach_current_thread_as_daemon()?;
    let called = unsafe {
        env.call_method_unchecked(
            callable,
            *method,
            jni::signature::ReturnType::Primitive(jni::signature::Primitive::Void),
            &[jni::sys::jvalue { j: a0 }],
        )
    };
    if called.is_err() {
        let _ = env.exception_describe();
        let _ = env.exception_clear();
    }
    called.map(|_| ())
})()
```

The global reference is released when the closure is dropped: it is an owned
Rust value inside it, so ordinary destruction discharges it.

## Checks

- `run` is found on the object's own class, which is the lambda's: a Kotlin
  `fun interface` compiles `run` with exactly the descriptor the arguments'
  wire types spell.
- A call's failure takes the callback's route — it is written to standard
  error and the call returns — and never the wrapper's, which reported
  `capture`'s failure to the JVM while it still had a caller.
- A handle argument handed out by a call that then fails is not taken back:
  the call cannot tell whether Kotlin received it before throwing. The
  [resource contract](../../extensions.md#runtime-resources) is what would say
  who owns it on that path.

[fn_callback]: README.md
[fn_callback_represent]: 05-represent.md
