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

```text
policy (JNI record):
    representation: data_class
    kotlin_fqn:     example.Stamp
    properties:     secs:  Long -> getter "getSecs",  descriptor "()J"
                    nanos: Long -> getter "getNanos", descriptor "()J"
    record_input:   ObjectProperties
```

## Checks

- The alternative, `SeparateArguments`, passes the two fields as individual JNI
  arguments and needs no property reads. It is a different effective policy, so
  it produces a different conversion node for the same record.
- The class metadata recorded here is what the Kotlin writer emits *and* what
  the property reads call, so the two cannot drift apart.
- Object input means the conversion depends on the JVM at run time, which is why
  [its node is fallible][struct_values_jni] where the C one is not.

[struct]: README.md
[struct_requests]: 03-requests.md
[struct_values_jni]: 04-values.jni.md
