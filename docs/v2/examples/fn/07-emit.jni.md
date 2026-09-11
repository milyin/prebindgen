<!-- spec: {"kind": "variant", "example": "fn", "stage": "07-emit", "language": "jni"} -->

[Stage chapter](../../stages/07-emit.md) · [Common cell][fn_emit] · [Element path][fn]
Owner: the common Rust writer and the JNI adapter's Kotlin writer

# Function taking an owned record — Emit bindings — Kotlin/JNI

## Input

```text
FunctionPlan(exported stamp_sum) frozen, with
    boundary: extern "system", symbol "Java_example_JNINative_stampSum",
              (JNIEnv, JClass, JObject) -> jlong,
              Runtime -> report_jni_error then return 0, reporting failure -> abort
    node(input):  object carrier, property getters "getSecs" / "getNanos"
    node(output): Scalar(jlong), identity
```

## Result

The Kotlin declarations, beside [the data class][struct_emit_jni]: the function
a caller uses, and the native method it delegates to on the harness object
every native call routes through — `JNINative`, the same object v1's bindings
use.

```kotlin
package example

public fun stampSum(stamp: Stamp): Long = JNINative.stampSum(stamp)

internal object JNINative {
    @JvmSynthetic
    external fun stampSum(stamp: Stamp): Long
}
```

The native wrapper the JVM binds that method to (`kotlin.rs`), after the
reporting helper the adapter contributes once per file:

```rust
pub fn report_jni_error(
    env: &mut jni::JNIEnv<'_>,
    error: jni::errors::Error,
) -> jni::errors::Result<()> {
    if env.exception_check()? {
        Ok(())
    } else {
        env.throw_new("java/lang/RuntimeException", error.to_string())
    }
}

#[no_mangle]
pub extern "system" fn Java_example_JNINative_stampSum(
    mut env: jni::JNIEnv<'_>,
    _class: jni::objects::JClass<'_>,
    stamp: jni::objects::JObject<'_>,
) -> jni::sys::jlong {
    let v0 = match env.call_method(&stamp, "getSecs", "()J", &[])
        .and_then(|value| value.j())
    {
        Ok(value) => value,
        Err(error) => {
            if report_jni_error(&mut env, error).is_err() {
                std::process::abort();
            }
            return 0;
        }
    };
    let v1 = match env.call_method(&stamp, "getNanos", "()J", &[])
        .and_then(|value| value.j())
    {
        Ok(value) => value,
        Err(error) => {
            if report_jni_error(&mut env, error).is_err() {
                std::process::abort();
            }
            return 0;
        }
    };
    let v2 = source::Stamp { secs: v0, nanos: v1 };
    let v3 = source::stamp_sum(v2);
    v3
}
```

`stampSum(Stamp(12, 34))` returns `46L` once the native library is loaded —
which is what the harness's `init` block does when the binding's
`set_jni_native_init(..)` names a loader; this binding sets none.

## Checks

- The symbol is built from the package, the harness object and the method name
  — `Java_example_JNINative_stampSum` — so renaming the Kotlin function moves
  both, and the package prefix moves all three.
- The `jni` crate's types are spelled in full, because the wrapper lands in a
  file the binding crate `include!`s and must not depend on that crate's
  imports.
- If `getSecs` fails, `getNanos` and `stamp_sum` do not run; if `getNanos` fails,
  the source `Stamp` is never constructed. Either way the JVM sees an exception
  rather than a returned zero.
- The adapter supplies the two getter expressions and the reporting helper,
  which is generated into this module; every `match`, the reporting call, its
  failure branch, the terminal return, the construction and the call are the
  registry's.
- Source `i64` and `jlong` are the same Rust value here, so the scalar
  conversions render nothing. A child type needing real work would insert its
  own conversion between a getter and the construction.

[fn]: README.md
[fn_emit]: 07-emit.md
[struct_emit_jni]: ../struct/07-emit.jni.md
