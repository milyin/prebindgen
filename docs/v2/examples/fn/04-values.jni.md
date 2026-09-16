<!-- spec: {"kind": "variant", "example": "fn", "stage": "04-values", "language": "jni"} -->

[Stage chapter](../../stages/04-values.md) · [Common cell][fn_values] · [Element path][fn]
Owner: the registry, on the JNI adapter's representation

# Function taking an owned record — Plan value conversions — Kotlin/JNI

## Input

The two crossings, with the JNI policy recorded for them:

```text
Crossing { source: Stamp, direction: IntoRust  }   policy: DataClass, read object properties
Crossing { source: i64,   direction: OutOfRust }   policy: jlong carrier
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

The native input is an object reference. A getter call obtains each long
property, and the registry uses the resulting integers to construct an owned
Rust `Stamp`. `Runtime` classifies failures from JNI so the next stage can route
them. `Independent` describes the resulting copied value; it is not a claim
that the full planned validity-contract API is already implemented.

[The record's JNI value plan][struct_values_jni] describes the getter operations.

## Checks

- The input node is fallible because each property read crosses into the JVM.
  It records that; it decides nothing, so [the boundary][fn_boundary_jni] must
  route the `Runtime` category.
- If the first getter fails, the second getter does not run. If either getter
  fails, the record is not constructed and the source function is not called.
- `Independent` holds because both integers are copied out of the object;
  nothing in the converted value stays tied to the reference frame.

[fn]: README.md
[fn_values]: 04-values.md
[fn_boundary_jni]: 05-boundary.jni.md
[struct_values_jni]: ../struct/04-values.jni.md
