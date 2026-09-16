<!-- spec: {"kind": "cell", "example": "fn", "stage": "04-values"} -->

[Stage chapter](../../stages/04-values.md) · [Element path][fn] · [Source crate](../../source.md)
Owner: the registry, on descriptions from the target adapter · Previous: [Record binding requests][fn_requests] · Next: [Assemble the native boundary][fn_boundary]

# Function taking an owned record — Plan value conversions

## Input

```text
Crossing { source: Stamp, direction: IntoRust  }    // for Param(0)
Crossing { source: i64,   direction: OutOfRust }    // for Return
```

## Result

There are two conversions: foreign argument to Rust `Stamp`, and Rust result
to foreign integer. A **node** is a reusable conversion plan. Neither node
calls `stamp_sum`; the next stage places that source call between them.

```text
node(input)  = Crossing { Stamp, IntoRust }
               relation: Stamp.fields
               children: [ node(i64, IntoRust) x2 ]   // the record path's plan
               body:     obtain secs carrier -> convert
                         obtain nanos carrier -> convert
                         construct source::Stamp { secs, nanos }

node(output) = Crossing { i64, OutOfRust }
               relation: atomic
               children: []
               body:     identity — source i64 and the target carrier are one value
```

The input node is [the record's own conversion][struct_values]; this function
refers to it, and so does anything else taking an owned `Stamp` under the same
policy. Their `NodeId`s are what [the boundary][fn_boundary] assembles.

## Checks

- Reuse depends on type, direction, [relation](../../stages/04-values.md#what-a-relation-is),
  effective policy and child conversions. A second owned `Stamp` input with
  the same choices can reuse this plan. C and JNI run separate generation jobs.
- An unsupported child makes the input node unsupported, and this function is
  skipped with that cause. Nothing partial is recorded.

## Language variants

- [C][fn_values_c]
- [Kotlin/JNI][fn_values_jni]

[fn]: README.md
[fn_requests]: 03-requests.md
[fn_boundary]: 05-boundary.md
[fn_values_c]: 04-values.c.md
[fn_values_jni]: 04-values.jni.md
[struct_values]: ../struct/04-values.md
