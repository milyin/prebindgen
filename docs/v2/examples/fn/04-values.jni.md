<!-- spec: {"kind": "variant", "example": "fn", "stage": "04-values", "language": "jni"} -->

# Function taking an owned record — Plan value conversions — Kotlin/JNI

[Stage chapter](../../stages/04-values.md) · [Common cell][fn_values] · [Element path][fn]

## Input

The two crossings with the JNI policy: object-properties input for the record,
`jlong` for the result.

## Owner

The registry, using the JNI adapter's representation, its property-getter
operations and its runtime carriers.

## Result

The input node takes the JNI environment and the object reference and produces an
owned source `Stamp`. Unlike its C counterpart it is **fallible**: each property
read goes through the JVM and can fail with `jni::errors::Error`, in the
`Runtime` failure category. The node records that failure in its contract; it
does not decide what happens next. Its operations are [the getters specified in
the record's JNI plan][struct_values_jni].

The output node maps source `i64` onto the `jlong` carrier — the same Rust value,
so an identity conversion with no failure.

Because the input node can fail, [the JNI boundary][fn_boundary_jni] must have a
route for the `Runtime` category, and the configured policy supplies one.

## Checks

The environment is an operand of every getter operation, so no rendered fragment
can quietly depend on a variable named `env` in its caller. A failed read stops
the success path: the second getter, the construction of `Stamp` and the call to
`stamp_sum` do not run. The node never returns a fallback value of its own — a
conversion that failed produces no value, and only the boundary decides what the
native function returns.

## Representation

```text
node(input)  contract: produced = source Stamp (owned)
                       access   = Owned
                       validity = Independent
                       failures = { Runtime: jni::errors::Error }
             representation: object carrier + property-read protocol

node(output) contract: produced = carrier jlong
                       access   = Owned
                       validity = Independent
                       failures = {}
             representation: Scalar(jlong), identity conversion
```

The produced `Stamp` is `Independent`: both integers are copied out of the JVM
object, so nothing in the converted value stays tied to the object reference or
to the JNI local-reference frame. A field whose conversion produced a borrowed
JVM value would carry a scope requirement instead, and the boundary would have to
keep that frame open.

## Along this element

The record's getter operations and their rendered fragments are in
[the record's JNI value plan][struct_values_jni].

[fn]: README.md
[fn_values]: 04-values.md
[fn_boundary_jni]: 05-boundary.jni.md
[struct_values_jni]: ../struct/04-values.jni.md
