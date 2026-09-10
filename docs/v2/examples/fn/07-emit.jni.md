<!-- spec: {"kind": "variant", "example": "fn", "stage": "07-emit", "language": "jni"} -->

[Stage chapter](../../stages/07-emit.md) · [Common cell][fn_emit] · [Element path][fn]
Owner: the common Rust writer and the JNI adapter's Kotlin writer

# Function taking an owned record — Emit bindings — Kotlin/JNI

## Input

```text
FunctionPlan(exported stamp_sum) frozen, with
    boundary: extern "C", symbol "Java_example_JNINative_stampSum",
              (JNIEnv, JClass, JObject, __error_sink) -> jlong,
              Runtime -> signal __error_sink, then return 0
    node(input):  object carrier, property getters "getSecs" / "getNanos"
    node(output): Scalar(jlong), identity
```

## Result

The Kotlin declarations, beside [the data class][struct_emit_jni]:

```kotlin
package example

public fun stampSum(stamp: Stamp, onError: JniErrorHandler<Long>): Long {
    val __bcap = JniErrorHandlerCapture.acquire()
    val __ret = JNINative.stampSum(stamp, __bcap)
    if (__bcap.failed) return onError.run(__bcap.ze0)
    return __ret
}

internal object JNINative {
    @JvmSynthetic
    external fun stampSum(stamp: Stamp, errorSink: Any): Long
}
```

`JniErrorHandler` and the capture the wrapper hands the native side are generated
into the package too, once for all of its functions.

The native wrapper it calls (`kotlin.rs`):

```rust
use crate::source;

#[no_mangle]
pub unsafe extern "C" fn Java_example_JNINative_stampSum<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    arg0: jni::objects::JObject<'a>,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::sys::jlong {
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod =
        ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "example/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";

    let v0 = match env.call_method(&arg0, "getSecs", "()J", &[])
        .and_then(|value| value.j())
    {
        Ok(value) => value,
        Err(error) => {
            signal_binding_error(&mut env, &__error_sink, &__SINK_MID,
                                 __SINK_FQN, __SINK_DESCR, &error.to_string());
            return 0;
        }
    };
    let v1 = match env.call_method(&arg0, "getNanos", "()J", &[])
        .and_then(|value| value.j())
    {
        Ok(value) => value,
        Err(error) => {
            signal_binding_error(&mut env, &__error_sink, &__SINK_MID,
                                 __SINK_FQN, __SINK_DESCR, &error.to_string());
            return 0;
        }
    };
    let v2 = source::Stamp { secs: v0, nanos: v1 };
    let v3 = source::stamp_sum(v2);
    v3
}
```

`stampSum(Stamp(12, 34)) { … }` returns `46L` once the native library is loaded,
which the harness does.

## Checks

- The symbol is built from the package, the `JNINative` object and the method
  name, so renaming the Kotlin function moves both.
- If `getSecs` fails, `getNanos` and `stamp_sum` do not run; if `getNanos` fails,
  the source `Stamp` is never constructed. Either way the Kotlin caller's
  `onError` runs, and the zero this wrapper returned is never observed.
- The adapter supplies the two getter expressions and the signalling helper,
  which is generated into this module; every `match`, the reporting call, the
  terminal return, the construction and the call are the registry's.
- Source `i64` and `jlong` are the same Rust value here, so the scalar
  conversions render nothing. A child type needing real work would insert its
  own conversion between a getter and the construction.

[fn]: README.md
[fn_emit]: 07-emit.md
[struct_emit_jni]: ../struct/07-emit.jni.md
