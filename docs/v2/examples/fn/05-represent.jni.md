<!-- spec: {"kind": "variant", "example": "fn", "stage": "05-represent", "language": "jni"} -->

[Stage chapter](../../stages/05-represent.md) · [Common cell][fn_represent] · [Element path][fn]
Owner: the registry, on the JNI adapter's [representation](../../stages/05-represent.md#represent-and-compose-values)

# Function taking an owned struct — Represent and compose values — Kotlin/JNI

## Input

The two selected [conversions](../../stages/04-select.md#select-conversion-relations), with the JNI [policy](../../stages/03-requests.md#what-policy-means) recorded for them:

```text
Stamp, IntoRust,  Stamp.fields [secs: i64 atomic, nanos: i64 atomic]   policy: DataClass, read object properties
i64,   OutOfRust, atomic                                               policy: jlong carrier
```

## Result

```text
node(input)  representation: object carrier + property-read protocol
             operations:     property getters (see below)
             contract:       produces an owned source Stamp
                             validity Independent
                             failures { Runtime: jni::errors::Error }

node(output) representation: Scalar(jlong)
             contract:       produces the carrier, validity Independent, failures {}
```

The [wrapper](../../stages/06-boundary.md#assemble-the-wrapper-boundary)'s input is an object reference. A getter call obtains each long
property, and the registry uses the resulting integers to construct an owned
Rust `Stamp`. `Runtime` classifies failures from JNI so the next stage can route
them. `Independent` describes the resulting copied value; it is not a claim
that the full planned validity-contract API is already implemented.

[The struct's JNI page][struct_represent_jni] describes the getter operations.

## Checks

- The input [node](../../stages/05-represent.md#represent-and-compose-values) is fallible because each property read crosses into the JVM.
  It records that; it decides nothing, so [the boundary][fn_boundary_jni] must
  route the `Runtime` category.
- If the first getter fails, the second getter does not run. If either getter
  fails, the struct is not constructed and the source function is not called.
- `Independent` holds because both integers are copied out of the object;
  nothing in the converted value stays tied to the reference frame.

[fn]: README.md
[fn_represent]: 05-represent.md
[fn_boundary_jni]: 06-boundary.jni.md
[struct_represent_jni]: ../struct/05-represent.jni.md
