<!-- spec: {"kind": "variant", "example": "typedef", "stage": "08-emit", "language": "jni"} -->

[Stage chapter](../../stages/08-emit.md) · [Common cell][typedef_emit] · [Element path][typedef]
Owner: the common Rust writer and the JNI adapter's Kotlin writer

# Type alias declaring an opaque handle — Emit bindings — Kotlin/JNI

## Input

```text
SurfaceSpec(public Ledger) frozen, with
    payload: ptr_class example.Ledger,
             release JNINative.freeLedger(ptr: Long)
FunctionPlan(release of Ledger) frozen, with
    boundary: extern "system", symbol "Java_example_JNINative_freeLedger",
              (JNIEnv, receiver JObject, jlong) -> nothing
    body:     Release(ptr)
```

## Result

The Kotlin declarations, beside [the data class][struct_emit_jni] and in the
same file as [the other functions][fn_emit_jni]. The class holds the address
and gives it up exactly once: through `take()`, which the public functions
call for a handle they consume and `free()` calls for one they do not. What is
taken is forgotten, so a freed or consumed handle holds zero, and zero is what
the native side refuses.

```kotlin
package example

public class Ledger(ptr: Long) {
    private var ptr: Long = ptr

    internal fun take(): Long {
        val taken = ptr
        ptr = 0L
        return taken
    }

    public fun free() = JNINative.freeLedger(take())
}

public fun ledgerOpen(stamp: Stamp): Ledger = Ledger(JNINative.ledgerOpen(stamp))

public fun ledgerClose(ledger: Ledger): Long = JNINative.ledgerClose(ledger.take())

internal object JNINative {
    @JvmSynthetic
    external fun freeLedger(ptr: Long)

    @JvmSynthetic
    external fun ledgerOpen(stamp: Stamp): Long

    @JvmSynthetic
    external fun ledgerClose(ledger: Long): Long
}
```

The native methods carry the address as a `Long`; the public functions are
where it becomes a `Ledger`. The release
[wrapper](../../stages/06-boundary.md#assemble-the-native-boundary), and the
two wrappers the handle's
[conversions](../../stages/04-select.md#select-conversion-relations) render
into (`kotlin.rs`):

```rust
#[no_mangle]
pub extern "system" fn Java_example_JNINative_freeLedger(
    _env: jni::JNIEnv<'_>,
    _this: jni::objects::JObject<'_>,
    ptr: jni::sys::jlong,
) {
    drop(
        ::core::ptr::NonNull::new(ptr as *mut source::Ledger)
            .map(|handle| unsafe { Box::from_raw(handle.as_ptr()) }),
    );
}

#[no_mangle]
pub extern "system" fn Java_example_JNINative_ledgerOpen(
    mut env: jni::JNIEnv<'_>,
    _this: jni::objects::JObject<'_>,
    stamp: jni::objects::JObject<'_>,
) -> jni::sys::jlong {
    let v0 = match env
        .call_method(&stamp, "getSecs", "()J", &[])
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
    let v1 = match env
        .call_method(&stamp, "getNanos", "()J", &[])
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
    let v2 = source::Stamp {
        secs: v0,
        nanos: v1,
    };
    let v3 = source::ledger_open(v2);
    let v4 = Box::into_raw(Box::new(v3)) as jni::sys::jlong;
    v4
}

#[no_mangle]
pub extern "system" fn Java_example_JNINative_ledgerClose(
    mut env: jni::JNIEnv<'_>,
    _this: jni::objects::JObject<'_>,
    ledger: jni::sys::jlong,
) -> jni::sys::jlong {
    let v0 = match ::core::ptr::NonNull::new(ledger as *mut source::Ledger)
        .map(|handle| unsafe { *Box::from_raw(handle.as_ptr()) })
        .ok_or_else(|| String::from("null `Ledger` handle"))
    {
        Ok(value) => value,
        Err(error) => {
            if env.throw_new("java/lang/IllegalStateException", error).is_err() {
                std::process::abort();
            }
            return 0;
        }
    };
    let v1 = source::ledger_close(v0);
    v1
}
```

`ledgerClose(ledgerOpen(Stamp(12, 34)))` is intended to return `46L` once the
native library is loaded; `ledgerOpen(stamp).free()` releases without reading;
`ledgerClose` on a closed or freed `Ledger` throws `IllegalStateException`;
`free()` twice releases once. As with the struct's function, this fixture
configures no loader and does not execute the call in a JVM.

## Checks

- No Rust type is generated for the handle: what crosses is a `jlong`, and the
  Rust side owns the value it points to.
- If a property read in `ledgerOpen` fails, no `Ledger` is allocated; the JVM
  sees the exception and no handle leaks. If `ledgerClose` is handed zero, the
  binding route throws and the source function never runs.
- `take()` is not synchronized: two threads consuming one `Ledger` at once
  can both read the address before either zeroes it. The shipping adapter's
  v1 output locks around this; the v2 writer does not yet.

[typedef]: README.md
[typedef_emit]: 08-emit.md
[struct_emit_jni]: ../struct/08-emit.jni.md
[fn_emit_jni]: ../fn/08-emit.jni.md
