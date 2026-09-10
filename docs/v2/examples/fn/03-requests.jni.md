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
    kotlin:     top-level fun example.stampSum   // camel-cased Rust name
    extern:     example.JNINative.stampSum       // where the external fun is declared
    symbol:     "Java_example_JNINative_stampSum"
    convention: extern "C", with (JNIEnv, JClass) supplied by the JVM
    input:      Stamp as one object, properties read through JNI
    output:     jlong native return
    failures:   signalled to the error handler the caller passed, and the
                wrapper returns a default the Kotlin side never observes
```

## Checks

- The Kotlin name and the symbol are one choice: `.name("…")` on the function
  declaration moves both, since the symbol is built from the package, the
  `JNINative` object and the method name.
- The error handler is part of every generated function's signature, which is
  why [the boundary][fn_boundary_jni] has somewhere to report a failed property
  read.
- A Kotlin declaration needs a package, so there is no source name to fall back
  on the way C has one.

[fn]: README.md
[fn_requests]: 03-requests.md
[fn_boundary_jni]: 05-boundary.jni.md
