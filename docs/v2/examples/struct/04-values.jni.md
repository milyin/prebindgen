<!-- spec: {"kind": "variant", "example": "struct", "stage": "04-values", "language": "jni"} -->

# Record with scalar fields — Plan value conversions — Kotlin/JNI

[Stage chapter](../../stages/04-values.md) · [Common cell][struct_values] · [Element path][struct]

## Input

The record crossing with the JNI policy: one JVM object whose properties carry
the two field values, read through the JNI environment.

## Owner

The registry composes the conversion; the JNI adapter describes the object
carrier, the environment carrier and one property-getter operation per field.
The adapter describes each getter as a local operation — it does not build the
record converter.

## Result

The representation is an object carrier with a product protocol whose projections
are the two getter operations. Each getter takes the environment and the object,
returns a JNI `jlong`, and can fail with `jni::errors::Error` in the `Runtime`
category. Those failures compose into the record node's contract, which is why
[the record's JNI conversion is fallible][fn_values_jni] where its C counterpart
is not.

Both successful integer results are independent of the object after access: no
reference or resource ownership escapes the read, so the constructed `Stamp` has
no dependency on the JVM object or its reference frame.

## Checks

The environment is an explicit operand, not an ambient variable. The getter names
and descriptors come from the class metadata recorded with the request, the same
metadata the Kotlin writer uses, so [the emitted class][struct_emit_jni] and
these calls cannot disagree. The getter operation calls the existing `jni` crate
directly, so its generated-helper dependency list is empty — the binding's build
requirements still include that crate.

A failed read must stop the conversion. The primitive describes the failure; it
does not decide the outcome, and it must not be rendered in a way that continues
calling JNI methods after one has failed.

## Representation

For `getSecs`, the stored specification is:

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

`jni_environment` describes the Rust `JNIEnv` runtime carrier, used through a
mutable borrow for this operation. `stamp_object` describes the input object
reference. `jni_long` is the JNI signed 64-bit integer carrier; `jni_error` is
the Rust `jni::errors::Error` type. `()J` is the JVM method descriptor for a
method taking no arguments and returning a signed 64-bit integer. These are
runtime/representation facts supplied by the JNI adapter.

The second specification changes the getter name to `getNanos`.

`JniOperation::CallLongGetter` is the rendering payload. The JNI implementation
provides a renderer that takes that payload and the operand expressions assigned
by the common writer. With environment operand `env`, object operand `arg0`, and
the metadata above, **this primitive alone** renders:

```rust
env.call_method(&arg0, "getSecs", "()J", &[])
    .and_then(|value| value.j())
```

The expression returns `Result<jlong, jni::errors::Error>`. The `.and_then`
extracts a typed integer from this one getter result. It performs no recursive
source conversion and makes no return from the enclosing native function.
`PrimitiveSpec` stores the getter description, not a finished `match`, converter
body or native wrapper. The common writer calls the JNI operation renderer only
while rendering an application in a completed registry plan.

## Along this element

See the [common cell][struct_values] for the record conversion both targets share.
Where the failures these getters declare are actually routed is
[the function's JNI boundary][fn_boundary_jni].

[struct]: README.md
[struct_values]: 04-values.md
[struct_emit_jni]: 07-emit.jni.md
[fn_values_jni]: ../fn/04-values.jni.md
[fn_boundary_jni]: ../fn/05-boundary.jni.md
