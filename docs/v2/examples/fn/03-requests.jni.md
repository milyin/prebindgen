<!-- spec: {"kind": "variant", "example": "fn", "stage": "03-requests", "language": "jni"} -->

[Stage chapter](../../stages/03-requests.md) · [Common cell][fn_requests] · [Element path][fn]
Owner: the JNI frontend

# Function taking an owned record — Record binding requests — Kotlin/JNI

## Input

```rust
// build.rs
JniGen::builder()
    .source(source_crate::PREBINDGEN_OUT_DIR)
    .set_package_prefix("example")
    .package(
        package!()
            .class(data_class!(Stamp))   // see the record path
            .fun(prebindgen_registry::fun!(stamp_sum)),
    )
    .build_with(prebindgen_jni::pipeline::Pipeline::V2)
    .expect("generate the JNI binding");
```

Enable the `v2` Cargo feature on `prebindgen-jni` for this build. The explicit
`build_with` call selects V2; plain `.build()` defaults to V1 unless
`PREBINDGEN_PIPELINE=v2` is set. Other imports are omitted in this excerpt.

## Result

The package prefix places the public function and generated native-method object
under `example`. `.fun(...)` requests a top-level Kotlin function. The native
method on `JNINative` connects it to Rust; Kotlin callers use `stampSum`, not
the long exported `Java_...` symbol. This summary records both names because
the public API and the JVM entry point have different naming roles:

```text
policy (JNI function):
    kotlin:     example.stampSum                    // the function a caller uses
    native:     example.JNINative.stampSum          // the harness method it delegates to
    symbol:     "Java_example_JNINative_stampSum"   // derived from that placement
    convention: extern "system", with (JNIEnv, receiver) supplied by the JVM
    input:      Stamp as one object, properties read through JNI
    output:     jlong native return
    failures:   reported to the JVM, then a default value returned
```

## Checks

- The public Kotlin function name and native method name are derived separately.
  `.name(...)` overrides the public name; the native method is derived from the
  Rust identifier through the method-name hook. The JNI symbol names that native
  method, so a public rename does not necessarily change the symbol.
- Where a failed property read goes is the adapter's convention, which is why
  [the boundary][fn_boundary_jni] has a route to plan at all.
- Package placement and default name derivation together determine the Kotlin
  API. Merely capturing a function does not choose its placement.

[fn]: README.md
[fn_requests]: 03-requests.md
[fn_boundary_jni]: 06-boundary.jni.md
