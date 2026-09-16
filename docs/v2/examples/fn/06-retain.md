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
           report_jni_error artifact          // JNI only
```

## Result

```text
requirements:
    node(input)                   ready      // needs the record's conversion
    node(output)                  ready
    record public representation  ready
    report_jni_error artifact     ready      // JNI only

outcome(exported stamp_sum) = Emitted { artifacts: [ native wrapper, public declaration ] }

request outcome summary:
    element: exported stamp_sum
    outcome: Emitted

generated symbol (not a report field): stamp_sum | Java_example_JNINative_stampSum
```

The table summarizes dependency outcomes; it is not the report's serialized
schema. `ready` means the required plan or public type is available. Only when
all requirements succeed can the function be kept. The report identifies this
request as `fn:stamp_sum`; generated symbols are shown here to connect the
request to the eventual C/JNI entry points.

If a field conversion were unsupported, the same reason would propagate to
both the record and its caller. The cause numbers below are explanatory labels;
current reports copy the reason and dependency path rather than use a cause-id table:

```text
outcome(record)             = Skipped { causes: [cause#1] }
outcome(exported stamp_sum) = Skipped { causes: [cause#1] }   // same cause, own path
```

## Checks

- Requirements are transitive: this function needs the record's conversion and
  [its public representation][struct_retain], so either being unsupported skips
  it too, carrying the same cause rather than a new one.
- Nothing partial is retained — no wrapper calling a conversion that was not.
- The JNI reporting helper is supporting output, not a separately requested
  public function. Keeping it does not add a user-facing function request.

[fn]: README.md
[fn_boundary]: 05-boundary.md
[fn_emit]: 07-emit.md
[struct_retain]: ../struct/06-retain.md
