<!-- spec: {"kind": "variant", "example": "struct", "stage": "07-emit", "language": "jni"} -->

[Stage chapter](../../stages/07-emit.md) · [Common cell][struct_emit] · [Element path][struct]
Owner: the JNI adapter's Kotlin writer

# Record with scalar fields — Emit bindings — Kotlin/JNI

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

No additional Rust boundary struct is needed for this representation. The native
wrapper receives a `JObject`, reads the getters and constructs the existing
source Rust `Stamp`. The source type still exists; only the extra ABI struct
used by the C target is absent.

## Checks

- The Kotlin compiler supplies the JVM getters `getSecs(): long` and
  `getNanos(): long` — the methods [the record's operations][struct_values_jni]
  call — so the class and the generated native code fit together by
  construction.
- A class-name override is used consistently in the emitted class and native
  references. Getter names are derived separately from the source field names.
- `Long` properties are what make the `()J` descriptor and the `jlong` result
  correct.

[struct]: README.md
[struct_emit]: 07-emit.md
[struct_values_jni]: 04-values.jni.md
[fn_emit_jni]: ../fn/07-emit.jni.md
