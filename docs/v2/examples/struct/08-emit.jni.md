<!-- spec: {"kind": "variant", "example": "struct", "stage": "08-emit", "language": "jni"} -->

[Stage chapter](../../stages/08-emit.md) · [Common cell][struct_emit] · [Element path][struct]
Owner: the JNI adapter's Kotlin writer

# Struct with scalar fields — Emit bindings — Kotlin/JNI

## Input

```text
SurfaceSpec(public Stamp) frozen, with
    payload: data_class example.Stamp,
             properties [secs: Long, nanos: Long]
```

## Result

In the generated Kotlin, beside [the external function][fn_emit_jni]:

```kotlin
package example

public data class Stamp(val secs: Long, val nanos: Long)
```

Each constructor property is a Kotlin `Long`, matching Rust `i64`. `val` makes
the property read-only in Kotlin, and the Kotlin compiler provides a JVM getter.
The data class also supplies the usual Kotlin value operations such as equality.

No additional Rust boundary struct is needed for this [representation](../../stages/05-represent.md#represent-and-compose-values). The
[wrapper](../../stages/06-boundary.md#assemble-the-wrapper-boundary) receives a `JObject`, reads the getters and constructs the existing
source Rust `Stamp`. The source type still exists; only the extra ABI struct
used by the C target is absent.

## Checks

- The Kotlin compiler supplies the JVM getters `getSecs(): long` and
  `getNanos(): long` — the methods [the struct's operations][struct_represent_jni]
  call — so the class and the generated Rust fit together by
  construction.
- A class-name override is used consistently in the emitted class and the Kotlin
  `external` method signatures that refer to it. The generated Rust uses `JObject`
  rather than the class name. Getter names are derived from source field names.
- `Long` properties are what make the `()J` descriptor and the `jlong` result
  correct.

[struct]: README.md
[struct_emit]: 08-emit.md
[struct_represent_jni]: 05-represent.jni.md
[fn_emit_jni]: ../fn/08-emit.jni.md
