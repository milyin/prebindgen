<!-- spec: {"example": "function", "kind": "variant", "language": "kotlin", "stage": "05-boundary"} -->

# Function: stamp_sum — Assemble the native boundary — kotlin

[Pipeline chapter](../../stages/05-boundary.md) · [Common contract](05-boundary.md) · [Example path](README.md)

## Input

Input node needs mutable JNI environment plus one object and can return a JNI error.

## Owner

JNI adapter supplies ABI/error descriptions; registry assembles FunctionPlan.

## Result

Signature: `extern "system" fn Java_example_Bindings_sum(env: JNIEnv, class: JClass, arg0: JObject) -> jlong`. Each getter Result is matched. On error, invoke report_jni_error; abort if that operation fails, otherwise return 0 with an exception pending. Success constructs Stamp, calls stamp_sum, returns jlong.

## Checks

The JNI class argument comes from @JvmStatic, not the source signature. Existing pending exceptions are preserved without further getter calls. The fallback zero is never a successful Kotlin result. The reporting helper has its own failure contract; its local ? cannot return from this wrapper.
