<!-- spec: {"kind": "variant", "example": "fn", "stage": "07-emit", "language": "jni"} -->

[Stage chapter](../../stages/07-emit.md) · [Common cell][fn_emit] · [Element path][fn]
Owner: the common Rust writer and the JNI adapter's Kotlin writer

# Function taking an owned record — Emit bindings — Kotlin/JNI

## Input

```text
FunctionPlan(exported stamp_sum) frozen, with
    boundary: extern "system", symbol "Java_example_Bindings_sum",
              (JNIEnv, JClass, JObject) -> jlong,
              Runtime -> report_jni_error then return 0, reporting failure -> abort
    node(input):  object carrier, property getters "getSecs" / "getNanos"
    node(output): Scalar(jlong), identity
```

## Result

The Kotlin declaration, beside [the data class][struct_emit_jni]:

```kotlin
package example

object Bindings {
    @JvmStatic
    external fun sum(stamp: Stamp): Long
}
```

The native wrapper it calls (`kotlin.rs`):

```rust
use crate::{jni_support::report_jni_error, source};
use jni::{objects::{JClass, JObject}, sys::jlong, JNIEnv};

#[no_mangle]
pub extern "system" fn Java_example_Bindings_sum(
    mut env: JNIEnv<'_>,
    _class: JClass<'_>,
    arg0: JObject<'_>,
) -> jlong {
    let v0 = match env.call_method(&arg0, "getSecs", "()J", &[])
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
    let v1 = match env.call_method(&arg0, "getNanos", "()J", &[])
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

`Bindings.sum(Stamp(12, 34))` returns `46L` once the native library is loaded,
which the harness does.

## Checks

- The symbol is built from the package, the object holding the declaration and
  the method name, so renaming the Kotlin method moves both.
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
