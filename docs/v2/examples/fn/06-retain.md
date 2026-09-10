<!-- spec: {"kind": "cell", "example": "fn", "stage": "06-retain"} -->

[Stage chapter](../../stages/06-retain.md) · [Element path][fn] · [Source crate](../../source.md)
Owner: the registry · Previous: [Assemble the native boundary][fn_boundary] · Next: [Emit bindings][fn_emit]

# Function taking an owned record — Retain supported output

## Input

The candidates this function produced, and what they require:

```text
candidate: FunctionPlan(exported stamp_sum)
candidate: SurfaceSpec(exported stamp_sum)

requires:  node(Stamp, IntoRust)              // the record's conversion
           node(i64, OutOfRust)
           the record's public representation // from the record path
           the error-signalling helper        // JNI only
```

## Result

```text
requirements:
    node(input)                   ready      // needs the record's conversion
    node(output)                  ready
    record public representation  ready
    error-signalling helper       ready      // JNI only

outcome(exported stamp_sum) = Emitted { artifacts: [ native wrapper, public declaration ] }

report entry:
    element: exported stamp_sum
    outcome: Emitted
    symbol:  stamp_sum  |  Java_example_JNINative_stampSum
```

Had a field of the record been unsupported:

```text
outcome(record)             = Skipped { causes: [cause#1] }
outcome(exported stamp_sum) = Skipped { causes: [cause#1] }   // same cause, own path
```

## Checks

- Requirements are transitive: this function needs the record's conversion and
  [its public representation][struct_retain], so either being unsupported skips
  it too, carrying the same cause rather than a new one.
- Nothing partial is retained — no wrapper calling a conversion that was not.
- A helper kept only because this function needs it stays distinguishable in the
  report from an element the user asked to export.

[fn]: README.md
[fn_boundary]: 05-boundary.md
[fn_emit]: 07-emit.md
[struct_retain]: ../struct/06-retain.md
