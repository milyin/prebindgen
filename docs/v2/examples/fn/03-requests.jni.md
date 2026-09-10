<!-- spec: {"kind": "variant", "example": "fn", "stage": "03-requests", "language": "jni"} -->

[Stage chapter](../../stages/03-requests.md) · [Common cell][fn_requests] · [Element path][fn]
Owner: the JNI frontend

# Function taking an owned record — Record binding requests — Kotlin/JNI

## Input

```rust
// build.rs
JniGen::builder()
    .source(source_crate::PREBINDGEN_OUT_DIR)
    .package(
        package!("example")
            .class(data_class!(Stamp))   // see the record path
            .fun(fun!(stamp_sum)),
    )
    .build();
```

## Result

```text
policy (JNI function):
    kotlin:     example.Bindings.sum             // the object holding the declaration
    symbol:     "Java_example_Bindings_sum"      // derived from that placement
    convention: extern "system", with (JNIEnv, JClass) supplied by the JVM
    input:      Stamp as one object, properties read through JNI
    output:     jlong native return
    failures:   reported to the JVM, then a default value returned
```

## Checks

- The Kotlin name and the symbol are one choice: `.name("…")` on the function
  declaration moves both, since the symbol is built from the package, the class
  holding the declaration and the method name.
- Where a failed property read goes is the adapter's convention, which is why
  [the boundary][fn_boundary_jni] has a route to plan at all.
- A Kotlin declaration needs a package, so there is no source name to fall back
  on the way C has one.

[fn]: README.md
[fn_requests]: 03-requests.md
[fn_boundary_jni]: 05-boundary.jni.md
