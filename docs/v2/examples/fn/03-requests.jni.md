<!-- spec: {"kind": "variant", "example": "fn", "stage": "03-requests", "language": "jni"} -->

[Stage chapter](../../stages/03-requests.md) · [Common cell][fn_requests] · [Element path][fn]
Owner: the JNI frontend

# Function taking an owned record — Record binding requests — Kotlin/JNI

## Input

```rust
// build.rs (schematic)
JniGen::builder()
    .package(package!("example").data_class(data_class!(Stamp)))
    .function("stamp_sum", placement!("example.Bindings.sum"))
    .runtime_errors(RuntimeErrors::PreservePendingElseThrow("java/lang/RuntimeException"))
    .build();
```

## Result

```text
policy (JNI function):
    placement:  example.Bindings.sum
    symbol:     "Java_example_Bindings_sum"      // derived from the placement
    convention: extern "system", with (JNIEnv, JClass) supplied by the JVM
    input:      Stamp as one object, properties read through JNI
    output:     jlong native return
    failures:   Runtime -> preserve a pending exception, else throw RuntimeException;
                a failure while reporting -> abort
```

## Checks

- Placement and symbol are one choice: renaming the Kotlin method changes the
  symbol the wrapper must export.
- The error convention is configuration, not a writer default. What
  [the boundary][fn_boundary_jni] does when a property read fails is decided
  here.
- A Kotlin declaration needs a package and a class, so there is no source name
  to fall back on the way C has one.

[fn]: README.md
[fn_requests]: 03-requests.md
[fn_boundary_jni]: 05-boundary.jni.md
