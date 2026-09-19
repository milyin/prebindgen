<!-- spec: {"kind": "variant", "example": "struct", "stage": "05-represent", "language": "jni"} -->

[Stage chapter](../../stages/05-represent.md) · [Common cell][struct_represent] · [Element path][struct]
Owner: the registry; the JNI adapter describes the object [carrier](../../stages/05-represent.md#describing-target-values-and-operations) and its getters

# Struct with scalar fields — Represent and compose values — Kotlin/JNI

## Input

```text
Crossing { source: Stamp, direction: IntoRust }
relation: Stamp.fields, parts [secs, nanos]
policy:   DataClass { class: "example.Stamp" }
derived getter operations: "getSecs" / "getNanos", descriptor "()J"
```

## Result

This [representation](../../stages/05-represent.md#represent-and-compose-values) carries
the object as one reference, but its product protocol reads two properties.
The registry plans each read and combines the returned integers. The operation
below uses the current fields, with descriptive variables for the carrier
types. Its operand roles tell the registry where each input comes from.

```text
ReprSpec {
    layout:   Scalar(<the object reference>),
    protocol: Product { projections: [read_secs, read_nanos] },
}
```

```rust
// read_secs; read_nanos differs only in the getter it names.
PrimitiveSpec {
    operands: vec![
        OperandSpec::context("jni.env", OperationType::Carrier(jni_environment), Access::Exclusive),
        OperandSpec::value(OperationType::Carrier(stamp_object), Access::Shared),
    ],
    result: Some(OperationType::Carrier(jni_long)),
    failure: PrimitiveFailure::fallible(
        OperationType::Carrier(jni_error), // jni::errors::Error
        FailureCategory::Runtime,
    ),
    dependencies: vec![],                                   // calls the jni crate directly
    implementation: Operation::Target(JniPayload::Getter {
        name:       "getSecs".into(),
        descriptor: "()J".into(),                           // no arguments, returns a long
    }),
}
```

The environment operand allows JNI calls and is used exclusively; the object
operand is borrowed for the getter. `Runtime` identifies the error category
that the enclosing [wrapper](../../stages/06-boundary.md#assemble-the-wrapper-boundary) must handle. In the implementation the environment
has the named role `Context("jni.env")`, which the boundary binds to `env`.

Applied to `env` and the input object `stamp`, the getter description renders
one expression. `call_method` invokes the zero-argument method, and `.j()`
extracts its long value:

```rust
env.call_method(&stamp, "getSecs", "()J", &[])
    .and_then(|value| value.j())
```

## Checks

- The expression evaluates to `Result<jlong, jni::errors::Error>` and stops
  there: no `let`, no `match` on that result, no return. Those belong to the
  [wrapper](../../stages/06-boundary.md#assemble-the-wrapper-boundary) the registry composes.
- The environment is an operand, so no rendered fragment can depend on a
  variable named `env` in its caller.
- Getter names and descriptors are derived from Flat's fields during planning.
  The adapter also derives [the emitted class][struct_emit_jni] from those
  fields, keeping public properties and JNI accesses consistent.
- Both results are independent copies: neither integer remains tied to the
  object's reference frame. There is no explicit validity-contract field today.

[struct]: README.md
[struct_represent]: 05-represent.md
[struct_emit_jni]: 08-emit.jni.md
