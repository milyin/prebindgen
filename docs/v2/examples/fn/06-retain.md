<!-- spec: {"kind": "cell", "example": "fn", "stage": "06-retain"} -->

[Stage chapter](../../stages/06-retain.md) · [Element path][fn] · [Source crate](../../source.md)
Owner: the registry · Previous: [Assemble the native boundary][fn_boundary] · Next: [Emit bindings][fn_emit]

# Function taking an owned record — Retain supported output

## Input

The candidate function plan and public declaration, and everything they require:
both conversion nodes, [the record's representation][struct_retain], and any
generated helper the operations named.

## Result

```text
requirements:
    node(input)                   ready      // needs the record's conversion
    node(output)                  ready
    record public representation  ready
    report_jni_error artifact     ready      // JNI only

outcome(exported stamp_sum) = Emitted { artifacts: [ native wrapper, public declaration ] }

report entry:
    element: exported stamp_sum
    outcome: Emitted
    symbol:  stamp_sum  |  Java_example_Bindings_sum
```

Had a field of the record been unsupported:

```text
outcome(record)             = Skipped { causes: [cause#1] }
outcome(exported stamp_sum) = Skipped { causes: [cause#1] }   // same cause, own path
```

## Checks

- Requirements are transitive: this function needs the record's conversion and
  its public representation, so either being unsupported skips it too, carrying
  the same cause rather than a new one.
- Nothing partial is retained — no wrapper calling a conversion that was not.
- A helper kept only because this function needs it stays distinguishable in the
  report from an element the user asked to export.

[fn]: README.md
[fn_boundary]: 05-boundary.md
[fn_emit]: 07-emit.md
[struct_retain]: ../struct/06-retain.md
