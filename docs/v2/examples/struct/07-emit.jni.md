<!-- spec: {"kind": "variant", "example": "struct", "stage": "07-emit", "language": "jni"} -->

[Stage chapter](../../stages/07-emit.md) · [Common cell][struct_emit] · [Element path][struct]
Owner: the JNI adapter's Kotlin writer

# Record with scalar fields — Emit bindings — Kotlin/JNI

## Input

```text
SurfaceSpec(public Stamp) frozen, with
    payload: data_class example.Stamp,
             properties [secs: Long, nanos: Long],
             getters "getSecs" / "getNanos", descriptor "()J"
```

## Result

In the generated Kotlin (`Bindings.kt`), beside
[the external function][fn_emit_jni]:

```kotlin
package example

data class Stamp(val secs: Long, val nanos: Long)
```

No Rust type is generated for this record: what crosses is a JVM object, and the
Rust side holds an object reference.

## Checks

- The Kotlin compiler supplies the JVM getters `getSecs(): long` and
  `getNanos(): long` — the methods [the record's operations][struct_values_jni]
  call — so the class and the generated native code fit together by
  construction.
- A naming override moves both or neither: one recorded metadata, two consumers.
- `Long` properties are what make the `()J` descriptor and the `jlong` result
  correct.

[struct]: README.md
[struct_emit]: 07-emit.md
[struct_values_jni]: 04-values.jni.md
[fn_emit_jni]: ../fn/07-emit.jni.md
