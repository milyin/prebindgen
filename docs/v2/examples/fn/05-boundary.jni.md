<!-- spec: {"kind": "variant", "example": "fn", "stage": "05-boundary", "language": "jni"} -->

[Stage chapter](../../stages/05-boundary.md) · [Common cell][fn_boundary] · [Element path][fn]
Owner: the registry, on the JNI adapter's `BoundarySpec`

# Function taking an owned record — Assemble the native boundary — Kotlin/JNI

## Input

```text
node(input)  : produces an owned source Stamp, failures { Runtime: jni::errors::Error }
node(output) : produces jlong, failures {}

policy (JNI function): placement example.Bindings.sum, extern "system",
                       input one object, output jlong,
                       Runtime -> preserve pending exception else throw,
                       failure while reporting -> abort
```

## Result

```text
BoundarySpec {
    abi:      extern "system", symbol "Java_example_Bindings_sum",
              synthetic operands: JNIEnv (exclusive), JClass (unused),
    inputs:   [ InputPlacement { native arg 0 (JObject) -> node(input) } ],
    output:   OutputPlacement::Return(node(output) -> jlong),
    failures: { Runtime: report through report_jni_error, then return 0;
                         if reporting fails -> abort },
}
```

Fixing the signature that [emission][fn_emit_jni] renders:

```rust
#[no_mangle]
pub extern "system" fn Java_example_Bindings_sum(
    mut env: JNIEnv<'_>,
    _class: JClass<'_>,
    arg0: JObject<'_>,
) -> jlong
```

The reporting operation is a generated artifact the JNI adapter contributes:

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

Its specification takes an exclusive environment operand and an owned error,
produces no value, can fail with a runtime error, and depends on that artifact.
The `?` inside propagates its own failure to its caller; it never returns from
the generated wrapper.

## Checks

- Zero is not a result: it is what a native method must return while an
  exception is pending, and Kotlin observes the exception. That route is
  [the recorded policy's][fn_requests_jni], not a writer default.
- Reporting is never retried with the operation that just failed — one failed
  report leads to the terminal action.
- After a failed property read, no further JNI call is made on the success
  path. The route exists because [the input node declares that
  failure][fn_values_jni]; a category the policy leaves unrouted would skip the
  function.

[fn]: README.md
[fn_boundary]: 05-boundary.md
[fn_requests_jni]: 03-requests.jni.md
[fn_values_jni]: 04-values.jni.md
[fn_emit_jni]: 07-emit.jni.md
