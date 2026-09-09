<!-- spec: {"kind": "variant", "example": "fn", "stage": "05-boundary", "language": "jni"} -->

[Stage chapter](../../stages/05-boundary.md) · [Common cell][fn_boundary] · [Element path][fn]

# Function taking an owned record — Assemble the native boundary — Kotlin/JNI

## Input

The function plan's two nodes and the JNI function policy from
[the JNI request][fn_requests_jni], including its error policy.

## Owner

The registry, using the JNI adapter's `BoundarySpec` and its error-reporting
operation.

## Result

One exported symbol, `Java_example_Bindings_sum`, with the `system` calling
convention and the two parameters the JVM supplies — the environment and the
calling class — ahead of the object argument. The object feeds the input
conversion; the converted `jlong` is delivered through the native return.

Because [the input node is fallible][fn_values_jni], this boundary also has a
route for the `Runtime` category: report the error, then return zero. Reporting
is itself an operation with its own failure, and the policy terminates that path
by aborting. The registry places both — the branch, the reporting call, the check
on its result and the terminal return — around the fragments the adapter renders.

## Checks

Zero is not a result here: it is the value the native function must return while
a JVM exception is pending, and Kotlin observes the exception rather than a
successful zero. Reporting is never retried with the operation that just failed —
one failed report leads to the terminal action, not to a second report. After a
failed property read, no further JNI call is made on the success path.

## Representation

The reusable runtime helper the JNI adapter supplies (`jni_support.rs`):

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

Its primitive specification has an exclusive environment operand, an owned error
operand, no success value, a possible runtime error and a dependency on the
helper artifact. The JNI payload selects a call to this helper. The helper's
local `?` propagates its own failure to its caller; it never returns from the
generated native wrapper.

```text
BoundarySpec {
  abi:    extern "system", symbol "Java_example_Bindings_sum",
          synthetic operands: JNIEnv (exclusive), JClass (unused),
  inputs: [ InputPlacement { native arg 0 (JObject) -> node(input) } ],
  output: OutputPlacement::Return(node(output) -> jlong),
  failures: {
    Runtime: report via report_jni_error, then return 0;
             if reporting fails -> abort
  },
}
```

The signature this fixes, which [emission][fn_emit_jni] renders:

```rust
pub extern "system" fn Java_example_Bindings_sum(
    mut env: JNIEnv<'_>,
    _class: JClass<'_>,
    arg0: JObject<'_>,
) -> jlong
```

## Along this element

See the [common cell][fn_boundary] for the plan both targets share.

[fn]: README.md
[fn_boundary]: 05-boundary.md
[fn_requests_jni]: 03-requests.jni.md
[fn_values_jni]: 04-values.jni.md
[fn_emit_jni]: 07-emit.jni.md
