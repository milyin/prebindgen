<!-- spec: {"kind": "cell", "example": "fn", "stage": "07-retain"} -->

[Stage chapter](../../stages/07-retain.md) · [Element path][fn] · [Source crate](../../source.md)
Owner: the registry · Previous: [Assemble the native boundary][fn_boundary] · Next: [Emit bindings][fn_emit]

# Function taking an owned struct — Retain supported output

## Input

Planning has already produced the input and result
[conversions](../../stages/04-select.md#select-conversion-relations) and a native
[wrapper](../../stages/06-boundary.md#assemble-the-native-boundary) plan. A
missing conversion would have skipped the function before this point. Retention
now checks the public declaration that the function's signature requires:

```text
candidate: FunctionPlan(exported stamp_sum)
candidate: SurfaceSpec(exported stamp_sum)

SurfaceSpec.requires: [ DeclarationId("type:Stamp") ]
```

## Result

```text
required declaration: type:Stamp       emitted
function declaration: fn:stamp_sum     emitted

generated C symbol:   stamp_sum
generated JNI symbol: Java_example_JNINative_stampSum
```

The table summarizes declaration
[outcomes](../../stages/07-retain.md#retain-supported-output), not the report's
serialized schema. The type is available, so the function can be kept. After
retention, generation collects supporting code needed by the kept plans,
including the JNI error-reporting helper; it is not a separate declaration
waiting for a readiness decision.

The C report's placement is the exported symbol `stamp_sum`. The JNI report's
placement is the public Kotlin name `example.stampSum`, not the `Java_...`
native symbol. Both symbols are shown here to connect the retained request to
its eventual entry point.

If a field [conversion](../../stages/04-select.md#select-conversion-relations) were unsupported, the same reason would propagate to
both the struct and its caller. The cause numbers below are explanatory labels;
current reports copy the reason and dependency path rather than use a cause-id table:

```text
outcome(struct)             = Skipped { causes: [cause#1] }
outcome(exported stamp_sum) = Skipped { causes: [cause#1] }   // same cause, own path
```

## Checks

- Requirements are transitive: this function needs the struct's [conversion](../../stages/04-select.md#select-conversion-relations) and
  its public [representation](../../stages/05-represent.md#represent-and-compose-values), [retained on its own path][struct_retain], so either being unsupported skips
  it too, carrying the same cause rather than a new one.
- Nothing partial is retained: no
  [wrapper](../../stages/06-boundary.md#assemble-the-native-boundary) can call a conversion that was not retained.
- The JNI reporting helper is supporting output, not a separately requested
  public function. It has no declaration and therefore no
  [outcome](../../stages/07-retain.md#retain-supported-output) of its own.

[fn]: README.md
[fn_boundary]: 06-boundary.md
[fn_emit]: 08-emit.md
[struct_retain]: ../struct/07-retain.md
