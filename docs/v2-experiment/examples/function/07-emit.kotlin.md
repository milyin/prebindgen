<!-- spec: {"example": "function", "kind": "variant", "language": "kotlin", "stage": "07-emit"} -->

# Function: stamp_sum — Emit bindings — kotlin

[Pipeline chapter](../../stages/07-emit.md) · [Common contract](07-emit.md) · [Example path](README.md)

## Input

Retained JNI wrapper, Kotlin native method, input type and error helper.

## Owner

Common writer composes the Rust wrapper, invoking the JNI getter renderer; Kotlin writer renders the public native declaration.

## Result

Expected native method, Rust wrapper and reporting helper appear below. Kotlin Stamp is specified on the struct emission path.

## Checks

Kotlin bytecode must have static native `sum(Lexample/Stamp;)J`. Rust symbol must be Java_example_Bindings_sum. Test harness loads the library. Getter and reporting failures require runtime tests before the capability is implemented.

## Concrete description or output

```kotlin
package example

object Bindings {
    @JvmStatic
    external fun sum(stamp: Stamp): Long
}
```

The common Rust writer emits the native wrapper:

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

The JNI-provided reporting helper is retained as a required artifact in `jni_support`:

```rust
use jni::{errors::Error, JNIEnv};

pub fn report_jni_error(env: &mut JNIEnv<'_>, error: Error)
    -> jni::errors::Result<()>
{
    if env.exception_check()? {
        Ok(())
    } else {
        env.throw_new("java/lang/RuntimeException", error.to_string())
    }
}
```

The helper primitive takes Exclusive environment and Owned error, returns unit or a runtime error, has no escaping resource, and depends on this helper artifact. Its implementation is a selected helper call. The adapter provides its runtime body. The registry places the helper call, checks its error and applies the selected terminal action. The getter renderers supply only the call_method/and_then expressions; the registry supplies the match and source construction/call.

## Related example

[struct: kotlin at this stage](../struct/07-emit.kotlin.md)
