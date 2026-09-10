<!-- spec: {"kind": "variant", "example": "fn", "stage": "05-boundary", "language": "jni"} -->

[Stage chapter](../../stages/05-boundary.md) · [Common cell][fn_boundary] · [Element path][fn]
Owner: the registry, on the JNI adapter's `BoundarySpec`

# Function taking an owned record — Assemble the native boundary — Kotlin/JNI

## Input

```text
node(input)  : produces an owned source Stamp, failures { Runtime: jni::errors::Error }
node(output) : produces jlong, failures {}

policy (JNI function): top-level example.stampSum, extern declared on JNINative,
                       extern "C", input one object, output jlong,
                       Runtime -> signal the caller's error handler
```

## Result

```text
BoundarySpec {
    abi:      extern "C", symbol "Java_example_JNINative_stampSum",
              synthetic operands: JNIEnv (exclusive), JClass (unused),
                                  __error_sink (the caller's error handler),
    inputs:   [ InputPlacement { native arg 0 (JObject) -> node(input) } ],
    output:   OutputPlacement::Return(node(output) -> jlong),
    failures: { Runtime: signal __error_sink with the message, then return 0 },
}
```

Fixing the signature that [emission][fn_emit_jni] renders:

```rust
#[no_mangle]
pub unsafe extern "C" fn Java_example_JNINative_stampSum<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    arg0: jni::objects::JObject<'a>,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::sys::jlong
```

The reporting operation takes the environment, the sink and a message, produces
no value, and depends on the cached identifier for the handler's method:

```text
PrimitiveSpec {
    operands: [ JNIEnv (exclusive), the sink object (shared), the message ],
    results:  [],
    implementation: call the runtime's signal_binding_error,
    dependencies:   [ the cached handler method id ],
}
```

## Checks

- Zero is not a result: a native method must return something, and the Kotlin
  wrapper checks the handler before looking at the returned value. That route is
  [the frontend's convention][fn_requests_jni], not a writer default.
- One report per failure: nothing re-reports through an operation that has
  already failed.
- After a failed property read, no further JNI call is made on the success
  path. The route exists because [the input node declares that
  failure][fn_values_jni]; a category the policy leaves unrouted would skip the
  function.

[fn]: README.md
[fn_boundary]: 05-boundary.md
[fn_requests_jni]: 03-requests.jni.md
[fn_values_jni]: 04-values.jni.md
[fn_emit_jni]: 07-emit.jni.md
