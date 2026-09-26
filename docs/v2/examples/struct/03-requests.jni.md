<!-- spec: {"kind": "variant", "example": "struct", "stage": "03-requests", "language": "jni"} -->

[Stage chapter](../../stages/03-requests.md) · [Common cell][struct_requests] · [Element path][struct]
Owner: the JNI frontend

# Struct with scalar fields — Record binding requests — Kotlin/JNI

## Input

```rust
// build.rs
JniGen::builder()
    .source(source_crate::PREBINDGEN_OUT_DIR)
    .set_package_prefix("example")
    .package(package!().class(data_class!(Stamp)))
    .build_with(prebindgen_jni::pipeline::Pipeline::V2)
    .expect("generate the Kotlin type");
```

This excerpt assumes the `v2` Cargo feature is enabled on `prebindgen-jni` and
omits imports. `build_with` explicitly selects the V2 engine; `.build()` would
instead use `PREBINDGEN_PIPELINE`, defaulting to V1 when that variable is unset.

## Result

The package prefix and `package!()` place the class in `example`, alongside
the binding's harness object. `data_class!(Stamp)`
requests public properties corresponding to the source fields. The [choice](../../stages/03-requests.md#what-a-choice-records)'s
`class` is the fully qualified Kotlin name: package plus class name. `()J` is a
JVM method descriptor: empty parentheses mean no arguments, and `J` means a
64-bit `long`. It describes the getter the generated JNI code will invoke.

```text
recorded in the binding by the JNI frontend, under `type:Stamp`:
    DataClass { class: "example.Stamp" }

derived during planning:
    properties: secs:  Long -> getter "getSecs",  descriptor "()J"
                nanos: Long -> getter "getNanos", descriptor "()J"
    input: one JVM object, read through property getters
```

## Checks

- A separate-arguments [representation](../../stages/05-represent.md#represent-and-compose-values) would pass the two fields individually
  and need no getter calls. The design describes that alternative, but this
  V2 struct path does not implement it.
- The adapter derives public property names and JNI getter calls from the
  same Flat fields. The class name comes from the recorded
  [choice](../../stages/03-requests.md#what-a-choice-records). Tests check that
  the generated Kotlin and the generated Rust agree.
- Object input means the
  [conversion](../../stages/04-select.md#select-conversion-relations) depends on the JVM at run time, which is why
  its [node](../../stages/05-represent.md#represent-and-compose-values) can fail,
  [as the JNI representation shows][struct_represent_jni], where the C one cannot.

[struct]: README.md
[struct_requests]: 03-requests.md
[struct_represent_jni]: 05-represent.jni.md
