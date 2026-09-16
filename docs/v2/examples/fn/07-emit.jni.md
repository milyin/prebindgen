<!-- spec: {"kind": "variant", "example": "fn", "stage": "07-emit", "language": "jni"} -->

[Stage chapter](../../stages/07-emit.md) · [Common cell][fn_emit] · [Element path][fn]
Owner: the common Rust writer and the JNI adapter's Kotlin writer

# Function taking an owned record — Emit bindings — Kotlin/JNI

## Input

```text
FunctionPlan(exported stamp_sum) frozen, with
    boundary: extern "system", symbol "Java_example_JNINative_stampSum",
              (JNIEnv, receiver JObject, JObject) -> jlong,
              Runtime -> report_jni_error then return 0, reporting failure -> abort
    node(input):  object carrier, property getters "getSecs" / "getNanos"
    node(output): Scalar(jlong), identity
```

## Result

The public function sits beside [the data class][struct_emit_jni]. It delegates
to `JNINative`, the generated object that groups native method declarations.
`external` means the implementation is in the native library. `internal`
keeps the object out of the public Kotlin API; `@JvmSynthetic` additionally
hides the method from ordinary Java source calls.

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
    _this: jni::objects::JObject<'_>,
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

The successful call is intended to make `stampSum(Stamp(12, 34))` return `46L`.
The consumer must first load the native library; this fixture configures no
loader. The generated Rust compiles and the Kotlin text is checked by
`v2check`, but that fixture does not yet execute this call in a JVM.

The native body reads `secs`, then `nanos`. Each `match` either obtains an
integer or reports the failure and returns. Only two successful reads reach
construction of `source::Stamp` and the call to `source::stamp_sum`. The
placeholder zero on the error path is not a successful result visible to Kotlin.

## Checks

- The symbol names the native method on `example.JNINative`. A public function
  rename and a native method rename are separate choices; changing only the
  public name need not change this symbol.
- The `jni` crate's types are spelled in full, because the wrapper lands in a
  file the binding crate `include!`s and must not depend on that crate's
  imports.
- If `getSecs` fails, `getNanos` and `stamp_sum` do not run; if `getNanos` fails,
  the source `Stamp` is never constructed. Either way the JVM sees an exception
  rather than a returned zero.
- The adapter supplies the getter expressions, reporting-call expression and
  reporting helper. The registry supplies each `match`, the check for reporting
  failure, the terminal return, record construction and the source function call.
- Source `i64` and `jlong` are the same Rust value here, so the scalar
  conversions render nothing. A child type needing real work would insert its
  own conversion between a getter and the construction.

[fn]: README.md
[fn_emit]: 07-emit.md
[struct_emit_jni]: ../struct/07-emit.jni.md
