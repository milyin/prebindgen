<!-- spec: {"kind": "variant", "example": "struct", "stage": "07-emit", "language": "jni"} -->

# Record with scalar fields — Emit bindings — Kotlin/JNI

[Stage chapter](../../stages/07-emit.md) · [Common cell][struct_emit] · [Element path][struct]

## Input

The frozen class metadata: package, class name, property names, their Kotlin
types and JVM descriptors.

## Owner

The JNI implementation's Kotlin writer. No Rust type is generated for this
record: the value that crosses is a JVM object, and the Rust side holds only an
object reference.

## Result

A Kotlin data class in the requested package, whose properties are the record's
fields. The Kotlin compiler supplies the JVM getters `getSecs(): long` and
`getNanos(): long` — the very methods [the record's JNI operations
call][struct_values_jni] — so the emitted class and the generated native code fit
together by construction.

## Checks

The class metadata that named the getters in the operation descriptors is the
metadata that renders this declaration; a naming override has to move both or
neither. The property types must match the carriers the operations expect: a
`Long` property is what makes the `()J` descriptor and the `jlong` result
correct. Loading the native library is the harness's job, not this file's.

## Representation

The generated Kotlin (`Bindings.kt`), alongside
[the external function declaration][fn_emit_jni]:

```kotlin
package example

data class Stamp(val secs: Long, val nanos: Long)
```

## Along this element

See the [common cell][struct_emit] for what both targets emit for this record.

[struct]: README.md
[struct_emit]: 07-emit.md
[struct_values_jni]: 04-values.jni.md
[fn_emit_jni]: ../fn/07-emit.jni.md
