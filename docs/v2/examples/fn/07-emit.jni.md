<!-- spec: {"kind": "variant", "example": "fn", "stage": "07-emit", "language": "jni"} -->

# Function taking an owned record — Emit bindings — Kotlin/JNI

[Stage chapter](../../stages/07-emit.md) · [Common cell][fn_emit] · [Element path][fn]

## Input

The frozen JNI function plan, the object representation of the record, and the
error routes the boundary fixed.

## Owner

The common Rust writer renders the native wrapper, calling the JNI operation
renderer for each getter expression. The JNI implementation's Kotlin writer
renders the public declaration.

## Result

The Kotlin side declares the external function next to
[the data class][struct_emit_jni]; the Rust side exports the matching symbol.
After the application loads the native library, `Bindings.sum(Stamp(12, 34))`
returns `46L`.

## Checks

The Kotlin declaration and the exported symbol have to agree, and both come from
the same recorded placement. If `getSecs` fails, `getNanos` and `stamp_sum` do
not run; if `getNanos` fails, the source `Stamp` is never constructed. In both
cases the JVM sees an exception rather than a returned zero. Loading the native
library belongs to the test harness, not to the generated code.

## Representation

The generated Kotlin (`Bindings.kt`), alongside the data class:

```kotlin
package example

object Bindings {
    @JvmStatic
    external fun sum(stamp: Stamp): Long
}
```

The generated Rust (`kotlin.rs`), combining the getter expressions supplied by
the JNI renderer with the registry's control flow:

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

The JNI adapter supplies the symbol and calling convention, the environment and
class parameter conventions, the carrier types, the getter operations and the
error policy. The registry supplies each `match`, the placement of the reporting
operation, its failure branch, the terminal return, the source construction and
the source call.

The source `i64` and JNI `jlong` use the same Rust value representation here. A
target scalar rule declares that relationship, and the registry resolves the
identity conversions. More complicated child types would insert their own
registry-planned conversions between each getter and `Stamp` construction.

## Along this element

See the [common cell][fn_emit] for the instruction-by-instruction mapping.

[fn]: README.md
[fn_emit]: 07-emit.md
[struct_emit_jni]: ../struct/07-emit.jni.md
