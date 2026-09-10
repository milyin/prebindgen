<!-- spec: {"kind": "variant", "example": "fn", "stage": "04-values", "language": "jni"} -->

[Stage chapter](../../stages/04-values.md) · [Common cell][fn_values] · [Element path][fn]
Owner: the registry, on the JNI adapter's representation

# Function taking an owned record — Plan value conversions — Kotlin/JNI

## Input

The two crossings under the JNI policy: object-properties input for the record,
`jlong` for the result.

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

The getters are specified, with the fragment each renders, in
[the record's JNI value plan][struct_values_jni].

## Checks

- The input node is fallible because each property read crosses into the JVM.
  It records that; it decides nothing, so [the boundary][fn_boundary_jni] must
  route the `Runtime` category.
- A failed read stops the success path: no second getter, no construction, no
  call. The node yields no value and no fallback.
- `Independent` holds because both integers are copied out of the object;
  nothing in the converted value stays tied to the reference frame.

[fn]: README.md
[fn_values]: 04-values.md
[fn_boundary_jni]: 05-boundary.jni.md
[struct_values_jni]: ../struct/04-values.jni.md
