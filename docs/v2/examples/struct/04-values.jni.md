<!-- spec: {"kind": "variant", "example": "struct", "stage": "04-values", "language": "jni"} -->

[Stage chapter](../../stages/04-values.md) · [Common cell][struct_values] · [Element path][struct]
Owner: the registry; the JNI adapter describes the object carrier and its getters

# Record with scalar fields — Plan value conversions — Kotlin/JNI

## Input

The record crossing under the JNI policy: one JVM object whose properties carry
the two field values, read through the JNI environment.

## Result

The representation, and one operation per property:

```text
ReprSpec {
    layout:   Scalar(<the object reference>),
    protocol: Product { projections: [read_secs, read_nanos], … },
}
```

```rust
// read_secs; read_nanos differs only in the getter it names.
PrimitiveSpec {
    signature: PrimitiveSignature {
        operands: vec![
            OperandSpec { ty: OperationType::Carrier(jni_environment), access: Access::Exclusive },
            OperandSpec { ty: OperationType::Carrier(stamp_object),    access: Access::Shared },
        ],
        results: vec![OperationType::Carrier(jni_long)],
    },
    failure: PrimitiveFailure::Fallible {
        error:    OperationType::Carrier(jni_error),       // jni::errors::Error
        category: FailureCategory::Runtime,
    },
    validity:     ValidityContract { results: vec![ResultValidity::Independent] },
    resources:    ResourceContract::none(),
    dependencies: vec![],                                   // calls the jni crate directly
    implementation: JniOperation::CallLongGetter {
        name:       "getSecs".into(),
        descriptor: "()J".into(),                           // no arguments, returns a long
    },
}
```

Applied to an environment named `env` and an object named `arg0`, that
description renders one expression:

```rust
env.call_method(&arg0, "getSecs", "()J", &[])
    .and_then(|value| value.j())
```

## Checks

- The expression evaluates to `Result<jlong, jni::errors::Error>` and stops
  there: no `let`, no `match` on that result, no return. Those belong to the
  wrapper the registry composes.
- The environment is an operand, so no rendered fragment can depend on a
  variable named `env` in its caller.
- Getter names and descriptors come from the class metadata recorded with the
  request — the same metadata [the emitted class][struct_emit_jni] is rendered
  from.
- Both results are `Independent`: the integers are copied out, so nothing stays
  tied to the object or its reference frame.

[struct]: README.md
[struct_values]: 04-values.md
[struct_emit_jni]: 07-emit.jni.md
