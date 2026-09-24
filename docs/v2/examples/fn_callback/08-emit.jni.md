<!-- spec: {"kind": "variant", "example": "fn_callback", "stage": "08-emit", "language": "jni"} -->

[Stage chapter](../../stages/08-emit.md) · [Common cell][fn_callback_emit] · [Element path][fn_callback]
Owner: the common Rust writer and the JNI adapter's Kotlin writer

# Function taking a callback — Emit bindings — Kotlin/JNI

## Input

```text
Retained(callback:impl Fn(i64)+Send+Sync+'static), read by the Kotlin writer, with
    metadata: Callback example.LongCallback, no raw interface
    args:     [ arg 0 through a jlong wire type of Kotlin type Long ]
FunctionPlan(fn:stamp_each) frozen, with
    abi, symbol: extern "system", "Java_example_JNINative_stampEach"
    params:      [ mut env: JNIEnv, _this: JObject, stamp: JObject, each: JObject ], no return
    routes:      Runtime -> ReportError then return (), reporting failure -> abort
```

## Result

The Kotlin declarations, beside [the data class][struct_emit_jni]:

```kotlin
package example

public fun interface LongCallback {
    public fun run(value: Long)
}

public fun stampEach(stamp: Stamp, each: LongCallback) = JNINative.stampEach(stamp, each)

internal object JNINative {
    @JvmSynthetic
    external fun stampEach(stamp: Stamp, each: LongCallback)
}
```

A caller passes a lambda: `stampEach(Stamp(12, 34)) { value -> println(value) }`
prints `12`, then `34`. `run` is public and takes a plain `Long`, so its JVM
name and descriptor are `run` and `(J)V`, which is what the Rust side looks up.

The [wrapper](../../stages/06-boundary.md#assemble-the-wrapper-boundary) the
JVM binds `stampEach` to (`kotlin.rs`):

```rust
#[no_mangle]
pub extern "system" fn Java_example_JNINative_stampEach(
    mut env: jni::JNIEnv<'_>,
    _this: jni::objects::JObject<'_>,
    stamp: jni::objects::JObject<'_>,
    each: jni::objects::JObject<'_>,
) {
    let v0 = match env
        .call_method(&stamp, "getSecs", "()J", &[])
        .and_then(|value| value.j())
    {
        Ok(value) => value,
        Err(error) => {
            if report_jni_error(&mut env, error).is_err() {
                std::process::abort();
            }
            return ();
        }
    };
    let v1 = match env
        .call_method(&stamp, "getNanos", "()J", &[])
        .and_then(|value| value.j())
    {
        Ok(value) => value,
        Err(error) => {
            if report_jni_error(&mut env, error).is_err() {
                std::process::abort();
            }
            return ();
        }
    };
    let v2 = source::Stamp {
        secs: v0,
        nanos: v1,
    };
    let v3 = match (|| -> jni::errors::Result<_> {
        let class = env.get_object_class(&each)?;
        let method = env.get_method_id(&class, "run", "(J)V")?;
        Ok((env.get_java_vm()?, env.new_global_ref(&each)?, method))
    })() {
        Ok(value) => value,
        Err(error) => {
            if report_jni_error(&mut env, error).is_err() {
                std::process::abort();
            }
            return ();
        }
    };
    let v5 = move |v4: i64| {
        if let Err(error) = (|| -> jni::errors::Result<()> {
            let (vm, callable, method) = &v3;
            let mut env = vm.attach_current_thread_as_daemon()?;
            let called = unsafe {
                env.call_method_unchecked(
                    callable,
                    *method,
                    jni::signature::ReturnType::Primitive(
                        jni::signature::Primitive::Void,
                    ),
                    &[jni::sys::jvalue { j: v4 }],
                )
            };
            if called.is_err() {
                let _ = env.exception_describe();
                let _ = env.exception_clear();
            }
            called.map(|_| ())
        })() {
            ::std::eprintln!("a callback failed: {}", error);
            return ();
        }
    };
    source::stamp_each(v2, v5);
}
```

Inside the closure, `env` is the call's own, attached for the calling thread;
the wrapper's `env` is not captured. A callback whose argument is a handle or
an enum is declared twice: the public interface over `Ledger` or `Operation`,
and an internal `…Raw` one over `Long` or `Int` that the `external` method takes,
with an `asRaw()` the public function calls to adapt one to the other —
`Ledger(ledger)`, `Operation.fromInt(operation)` for each argument.

## Checks

- The lambda may be called on another thread, after `stampEach` has returned,
  or both: the global reference keeps it alive until Rust drops the closure,
  and each call attaches the thread it runs on.
- A lambda that throws does not unwind into Rust: the exception is described
  on standard error and cleared, the failure is written out, and
  `stamp_each`'s call of it returns normally.
- Kotlin's `Unit`-returning `run` is what `Primitive::Void` calls; a callback
  returning a value is refused when the model is built.

[fn_callback]: README.md
[fn_callback_emit]: 08-emit.md
[struct_emit_jni]: ../struct/08-emit.jni.md
