<!-- spec: {"kind": "variant", "example": "struct", "stage": "05-represent", "language": "jni"} -->

[Stage chapter](../../stages/05-represent.md) · [Common cell][struct_represent] · [Element path][struct]
Owner: the registry; the JNI frontend states the object [carrier](../../stages/05-represent.md#describing-target-values-and-operations) and its getter

# Struct with scalar fields — Represent and compose values — Kotlin/JNI

## Input

```text
Crossing { source: Stamp, direction: IntoRust }
relation: Stamp.fields, parts [secs, nanos]
representation: Product { via: Fields, carrier: a `JObject` of example.Stamp, read: Getter }
```

## Result

This [representation](../../stages/05-represent.md#represent-and-compose-values)
carries the object as one reference, and reads two properties out of it. The
JNI frontend states it when it reads `data_class!(Stamp)`:

```rust
let stamp_obj = binding.carrier(WireType::exact(
    JniClass::Object,                              // `jni::objects::JObject<'_>`
    Some(Accepts::of([JniClass::Long])),           // what a getter returning a `long` reads
    Jvm {
        descriptor: "Lexample/Stamp;".into(),
        kotlin: KotlinType::Value("example.Stamp".into()),
    },
));
Representation::Product {
    via: Via::Fields,
    carrier: stamp_obj,
    read: Operation::target(JniOp::Getter)
        .context("jni.env")                         // the JNI environment
        .fails(FailureCategory::Runtime, parse_quote!(jni::errors::Error)),
}
```

The getter names no property and no descriptor. The registry applies it once
per part, and feeds the JNI writer the object operand, the environment the
[wrapper](../../stages/06-boundary.md#assemble-the-wrapper-boundary)'s form
binds to `env`, the part, and the carrier the part resolved to — a `jlong`
whose descriptor is `J`. The writer turns the part's name into `getSecs` by
Kotlin's getter convention and the result's descriptor into `()J`, and writes
one expression. `call_method` invokes the zero-argument method, and `.j()`
extracts its long value:

```rust
env.call_method(&stamp, "getSecs", "()J", &[])
    .and_then(|value| value.j())
```

`Runtime` is the error category the enclosing wrapper must route.

## Checks

- The expression evaluates to `Result<jlong, jni::errors::Error>` and stops
  there: no `let`, no `match` on that result, no return. Those belong to the
  wrapper the registry composes.
- The environment is an operand, so no written fragment can depend on a
  variable named `env` in its caller.
- The getter name comes from the part the registry feeds, and the Kotlin
  property from the same part, read by the Kotlin writer from
  [the retained output][struct_emit_jni], so the two cannot disagree.
- Both results are independent copies: neither integer remains tied to the
  object's reference frame.

[struct]: README.md
[struct_represent]: 05-represent.md
[struct_emit_jni]: 08-emit.jni.md
