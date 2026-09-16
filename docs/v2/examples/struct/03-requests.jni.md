<!-- spec: {"kind": "variant", "example": "struct", "stage": "03-requests", "language": "jni"} -->

[Stage chapter](../../stages/03-requests.md) · [Common cell][struct_requests] · [Element path][struct]
Owner: the JNI frontend

# Record with scalar fields — Record binding requests — Kotlin/JNI

## Input

```rust
// build.rs
JniGen::builder()
    .source(source_crate::PREBINDGEN_OUT_DIR)
    .package(package!("example").class(data_class!(Stamp)))
    .build();
```

## Result

`package!("example")` chooses where the class lives, and `data_class!(Stamp)`
requests public properties corresponding to the source fields. The policy's
`class` is the fully qualified Kotlin name: package plus class name. `()J` is a
JVM method descriptor: empty parentheses mean no arguments, and `J` means a
64-bit `long`. It describes the getter the generated JNI code will invoke.

```text
recorded policy: DataClass { class: "example.Stamp" }

derived during planning:
    properties: secs:  Long -> getter "getSecs",  descriptor "()J"
                nanos: Long -> getter "getNanos", descriptor "()J"
    input: one JVM object, read through property getters
```

## Checks

- A separate-arguments representation would pass the two fields individually
  and need no getter calls. That alternative is described by the design but
  is not implemented for this V2 record path.
- The adapter derives the public property names and native getter calls from
  the same Flat fields. The class name comes from the recorded policy. Tests
  check that generated Kotlin and native references agree.
- Object input means the conversion depends on the JVM at run time, which is why
  [its node is fallible][struct_values_jni] where the C one is not.

[struct]: README.md
[struct_requests]: 03-requests.md
[struct_values_jni]: 04-values.jni.md
