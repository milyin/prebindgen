<!-- spec: {"kind": "variant", "example": "fn", "stage": "05-represent", "language": "jni"} -->

[Stage chapter](../../stages/05-represent.md) · [Common cell][fn_represent] · [Element path][fn]
Owner: the registry, from the JNI frontend's [representations](../../stages/05-represent.md#represent-and-compose-values)

# Function taking an owned struct — Represent and compose values — Kotlin/JNI

## Input

The two selected [conversions](../../stages/04-select.md#select-conversion-relations), each with the
representation its rule named:

```text
Stamp, IntoRust,  Stamp.fields [secs: i64 atomic, nanos: i64 atomic]   Product over a `JObject`, read: Getter
i64,   OutOfRust, atomic                                               Terminal over `jlong`, Identity
```

## Result

```text
node(input)  carrier:    a JObject of example.Stamp, members [secs: jlong, nanos: jlong]
             operations: the JNI getter, applied per part (see below)
             produces:   an owned source Stamp
                         failures { Runtime: jni::errors::Error }

node(output) carrier:    jlong
             operations: none — the identity renders nothing
             produces:   the carrier; failures {}
```

The [wrapper](../../stages/06-boundary.md#assemble-the-wrapper-boundary)'s input is an object reference. A getter call obtains each long
property, and the registry uses the resulting integers to construct an owned
Rust `Stamp`. `Runtime` classifies failures from JNI so the next stage can route
them. The integers are copied out of the object, so nothing in the converted
value stays tied to the reference frame, and no validity contract is needed
yet.

[The struct's JNI page][struct_represent_jni] describes the getter operations.

## Checks

- The input [node](../../stages/05-represent.md#represent-and-compose-values) is fallible because each property read crosses into the JVM.
  It records that; it decides nothing, so [the boundary][fn_boundary_jni] must
  route the `Runtime` category.
- If the first getter fails, the second getter does not run. If either getter
  fails, the struct is not constructed and the source function is not called.
- Nothing the conversion produces borrows the object: both integers are
  copied out of it.

[fn]: README.md
[fn_represent]: 05-represent.md
[fn_boundary_jni]: 06-boundary.jni.md
[struct_represent_jni]: ../struct/05-represent.jni.md
