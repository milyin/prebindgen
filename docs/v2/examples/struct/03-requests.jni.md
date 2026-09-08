<!-- spec: {"kind": "variant", "example": "struct", "stage": "03-requests", "language": "jni"} -->

# Record with scalar fields — Record binding requests — Kotlin/JNI

[Stage chapter](../../stages/03-requests.md) · [Common cell][struct_requests] · [Element path][struct]

## Input

A `build.rs` declaring the Kotlin package and `Stamp` as a data class, with
object-properties input at the native boundary.

## Owner

The JNI frontend. Its class metadata — package, class name, property names and
their JVM descriptors — is recorded here and used both by the Kotlin writer and
by the operations that read the object, so the two cannot drift apart.

## Result

The record's policy selects the Kotlin data class `example.Stamp` with two `Long`
properties, and `ObjectProperties` as the way values of it reach a native
wrapper: one object reference, whose properties are read through JNI.

The alternative the JNI frontend also offers, `SeparateArguments`, would pass the
two fields as individual JNI arguments and need no property reads. It is a
different effective policy, so it would produce a different conversion node for
the same record — which is precisely what node identity is for.

## Checks

A naming override has to affect the Kotlin declaration and the getter descriptors
together: they are one recorded choice, and a generated getter call that does not
match the emitted class is a defect, not a policy. Object input also means the
conversion depends on the JVM at runtime, which is why [its
node is fallible][struct_values_jni] while the C one is not.

## Representation

```rust
// build.rs, JNI frontend (schematic)
JniGen::builder()
    .package(package!("example").data_class(data_class!(Stamp)))
    .build();
```

```text
policy (JNI record):
  representation: data_class
  kotlin_fqn:     example.Stamp
  properties:     secs: Long  -> getter "getSecs", descriptor "()J"
                  nanos: Long -> getter "getNanos", descriptor "()J"
  record_input:   ObjectProperties
```

## Along this element

See the [common cell][struct_requests] for the request and the parts both targets share.

[struct]: README.md
[struct_requests]: 03-requests.md
[struct_values_jni]: 04-values.jni.md
