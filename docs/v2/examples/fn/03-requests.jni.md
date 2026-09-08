<!-- spec: {"kind": "variant", "example": "fn", "stage": "03-requests", "language": "jni"} -->

# Function taking an owned record — Record binding requests — Kotlin/JNI

[Stage chapter](../../stages/03-requests.md) · [Common cell][fn_requests] · [Element path][fn]

## Input

A `build.rs` that declares the Kotlin package and the placement of `stamp_sum`,
alongside the declaration of [`Stamp` as a data class][struct_requests_jni].

## Owner

The JNI frontend. It decides the Kotlin placement — package, object, method name
— and from that the JNI symbol, since a JNI symbol is a function of where the
`external fun` is declared.

## Result

The function request carries a JNI function policy: the Kotlin placement
`example.Bindings.sum`, the JNI symbol derived from it, the `system` calling
convention, the environment and class parameters the JVM supplies, `jlong` as the
result carrier, and the error policy for the call — preserve a pending JVM
exception, otherwise throw `RuntimeException`, and abort if reporting itself
fails. The record parameter takes the object-properties input recorded for
`Stamp`, so the wrapper receives one object rather than two integers.

## Checks

The placement and the symbol have to agree: renaming the Kotlin method changes
the symbol the Rust wrapper must export, and both come from this one recorded
choice. The error policy is part of the request, not a writer default — a policy
the frontend cannot express is an unsupported request here, not a silently
different behavior in the generated wrapper.

## Representation

```rust
// build.rs, JNI frontend (schematic)
JniGen::builder()
    .package(package!("example").data_class(data_class!(Stamp)))
    .function("stamp_sum", placement!("example.Bindings.sum"))
    .build();
```

```text
policy (JNI function):
  placement:  example.Bindings.sum
  symbol:     "Java_example_Bindings_sum"
  convention: extern "system", with (JNIEnv, JClass) supplied by the JVM
  input:      Stamp as one object; properties read through JNI
  output:     jlong native return
  failures:   Runtime -> preserve pending exception, else throw RuntimeException;
              failure while reporting -> abort
```

## Along this element

See the [common cell][fn_requests] for the request identities both targets share.

[fn]: README.md
[fn_requests]: 03-requests.md
[struct_requests_jni]: ../struct/03-requests.jni.md
