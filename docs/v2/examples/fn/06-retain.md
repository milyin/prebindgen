<!-- spec: {"kind": "cell", "example": "fn", "stage": "06-retain"} -->

# Function taking an owned record — Retain supported output

[Stage chapter](../../stages/06-retain.md) · [Element path][fn] · [Source fixture](../../source.md)

## Input

The candidate function plan, the public declaration the adapter described for it,
and everything they require: the input and output conversion nodes, the record's
public representation, and any generated helper the operations named.

## Owner

The registry. Targets have answered every local question by now; nothing they
return at this point can change what is retained.

## Result

For this fixture the outcome is `Emitted`. The retained set is the function plan,
the two conversion nodes, the public declaration, and the artifacts they need —
the native wrapper, the record's declaration from [the record path][struct_retain],
and, for JNI, the error-reporting helper. The artifacts are ordered so that a
declaration precedes its uses. The report records this element as emitted, with
the artifacts it produced and the symbol it exports.

## Checks

The function's requirements are transitive: it needs the record's conversion and
the record's public representation, so if either were unsupported this function
would be `Skipped`, carrying that same cause rather than a new one, and the
record's declaration would be skipped alongside it. Nothing partial is retained —
there is no wrapper that calls a conversion that was not retained. A helper kept
only because this function needs it stays distinguishable in the report from an
element the user asked to export.

## Representation

```text
outcome(exported stamp_sum) = Emitted {
  artifacts: [ native wrapper, public declaration ]
}

requirements resolved:
  node(input)  -> ready       (needs the record's conversion)
  node(output) -> ready
  record public representation -> ready
  JNI only: report_jni_error helper -> ready

report entry:
  element:  exported stamp_sum
  outcome:  Emitted
  symbol:   stamp_sum_c  |  Java_example_Bindings_sum
```

Had a field of the record been unsupported, the same table would read:

```text
outcome(record)            = Skipped { causes: [cause#1] }
outcome(exported stamp_sum)= Skipped { causes: [cause#1] }   // same cause, own path
```

## Along this element

Previous: [Assemble the native boundary][fn_boundary] · Next: [Emit bindings][fn_emit]

[fn]: README.md
[fn_boundary]: 05-boundary.md
[fn_emit]: 07-emit.md
[struct_retain]: ../struct/06-retain.md
