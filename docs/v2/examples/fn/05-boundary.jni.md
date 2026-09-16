<!-- spec: {"kind": "variant", "example": "fn", "stage": "05-boundary", "language": "jni"} -->

[Stage chapter](../../stages/05-boundary.md) · [Common cell][fn_boundary] · [Element path][fn]
Owner: the registry, on the JNI adapter's `BoundarySpec`

# Function taking an owned record — Assemble the native boundary — Kotlin/JNI

## Input

```text
node(input)  : produces an owned source Stamp, failures { Runtime: jni::errors::Error }
node(output) : produces jlong, failures {}

policy (JNI function): example.JNINative.stampSum, extern "system",
                       input one object, output jlong,
                       Runtime -> report to the JVM, then return a default
```

## Result

```text
BoundarySpec {
    abi:      extern "system", symbol "Java_example_JNINative_stampSum",
              synthetic operands: JNIEnv (exclusive), receiver JObject (unused),
    inputs:   [ InputPlacement { native arg `stamp` (JObject) -> node(input) } ],
    output:   OutputPlacement::Return(node(output) -> jlong),
    failures: { Runtime: report through report_jni_error, then return 0;
                         if reporting fails -> abort },
}
```

JNI supplies two arguments in addition to the user's `stamp`: `env` permits
calls into the JVM, and `_this` is the `JNINative` singleton receiving the native
method call. The [wrapper](../../stages/05-boundary.md#assemble-the-native-boundary) needs `env` for property reads and error reporting;
it does not otherwise use `_this`. `jlong` is JNI's signed 64-bit integer type.

These roles determine the signature that [emission][fn_emit_jni] renders:

```rust
#[no_mangle]
pub extern "system" fn Java_example_JNINative_stampSum(
    mut env: jni::JNIEnv<'_>,
    _this: jni::objects::JObject<'_>,
    stamp: jni::objects::JObject<'_>,
) -> jni::sys::jlong
```

The reporting operation is a generated [artifact](../../stages/04-values.md#individual-target-operations) this adapter contributes:

```rust
pub fn report_jni_error(env: &mut jni::JNIEnv<'_>, error: jni::errors::Error)
    -> jni::errors::Result<()>
{
    if env.exception_check()? {
        Ok(())
    } else {
        env.throw_new("java/lang/RuntimeException", error.to_string())
    }
}
```

The helper first checks whether the JVM already has a pending exception. If so,
it preserves it. Otherwise it throws `RuntimeException` with the JNI error's
message. Its `Result<()>` tells the generated
[wrapper](../../stages/05-boundary.md#assemble-the-native-boundary) whether
reporting succeeded. The `?` returns from this helper if `exception_check`
fails, not from the wrapper. The wrapper checks the helper's result and aborts
if reporting failed.

## Checks

- Zero is the chosen placeholder for this integer-returning method while an
  exception is pending, and Kotlin observes the exception. That route is
  [the adapter's convention][fn_requests_jni], not a writer default.
- Reporting is never retried with the operation that just failed — one failed
  report leads to the terminal action.
- After a failed property read, no further JNI call is made on the success
  path. The route exists because the input [node](../../stages/04-values.md#plan-value-conversions) [declares that
  failure][fn_values_jni]; a category the [policy](../../stages/03-requests.md#what-policy-means) leaves unrouted would skip the
  function.

[fn]: README.md
[fn_boundary]: 05-boundary.md
[fn_requests_jni]: 03-requests.jni.md
[fn_values_jni]: 04-values.jni.md
[fn_emit_jni]: 07-emit.jni.md
