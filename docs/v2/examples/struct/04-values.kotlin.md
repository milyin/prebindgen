<!-- spec: {"example": "struct", "kind": "variant", "language": "kotlin", "stage": "04-values"} -->

# Struct: Stamp — Plan value conversions — kotlin

[Pipeline chapter](../../stages/04-values.md) · [Common contract](04-values.md) · [Example path](README.md)

## Input

RecordRelation with two i64 children, selected JNI object/environment carriers and getter metadata.

## Owner

JNI adapter supplies getter descriptors and renderer; registry builds the record body.

## Result

Each PrimitiveSpec takes Exclusive environment and Shared object, returns jlong or runtime jni::errors::Error; successful integer validity Independent, resources none, generated dependencies empty. Payload `JniOperation::CallLongGetter { name, descriptor: "()J" }` selects getSecs or getNanos.

## Checks

Getter metadata must match the Kotlin compiler's class signatures. The expression is a local Result, not a match with an enclosing return. Registry applies the two primitives in order and constructs only on complete success. See the complete specification below.

## Concrete description or output

```rust
let read_secs = PrimitiveSpec {
    signature: PrimitiveSignature {
        operands: vec![
            OperandSpec {
                ty: OperationType::Carrier(jni_environment),
                access: Access::Exclusive,
            },
            OperandSpec {
                ty: OperationType::Carrier(stamp_object),
                access: Access::Shared,
            },
        ],
        results: vec![OperationType::Carrier(jni_long)],
    },
    failure: PrimitiveFailure::Fallible {
        error: OperationType::Carrier(jni_error),
        category: FailureCategory::Runtime,
    },
    validity: ValidityContract {
        results: vec![ResultValidity::Independent],
    },
    resources: ResourceContract::none(),
    dependencies: vec![],
    implementation: JniOperation::CallLongGetter {
        name: "getSecs".into(),
        descriptor: "()J".into(),
    },
};
```

The carrier IDs name JNIEnv, the object, jlong and jni::errors::Error. `()J` means no JVM arguments and signed 64-bit result. One primitive renderer produces:

```rust
env.call_method(&arg0, "getSecs", "()J", &[])
    .and_then(|value| value.j())
```

## Related example

[function: kotlin at this stage](../function/04-values.kotlin.md)
