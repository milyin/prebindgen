<!-- spec: {"kind": "cell", "example": "fn", "stage": "04-values"} -->

[Stage chapter](../../stages/04-values.md) · [Element path][fn] · [Source crate](../../source.md)
Previous: [Record binding requests][fn_requests] · Next: [Assemble the native boundary][fn_boundary]

# Function taking an owned record — Plan value conversions

## Input

Two crossings, one per site: `Stamp` in the `IntoRust` direction for `Param(0)`,
and `i64` in the `OutOfRust` direction for `Return`, each with the effective
policy resolved from the request.

## Owner

The registry drives both. It asks the adapter to select a relationship, resolves
the children of what was selected, asks the adapter to represent the value, and
composes the instructions. The adapter answers about representation and supplies
the operations; it never walks the record.

## Result

Two conversion nodes.

The input node is the record's `IntoRust` conversion — the one [the record path
describes][struct_values]. It is not a separate copy owned by this function: the
same node serves every site that converts an owned `Stamp` under the same
relationship and effective policy.

The output node converts source `i64` to the target's signed 64-bit carrier.
Here that is an identity: the value is preserved and no helper call is needed,
though the node still exists, with its own contract, because the boundary refers
to conversions by identity rather than by shape.

Neither node contains the call to `stamp_sum`. Their `NodeId`s are what
[the boundary][fn_boundary] assembles.

## Checks

Node identity includes the effective policy, so the C and JNI generations never
share a node even though both convert an owned `Stamp` from the same source
relationship. A second function taking an owned `Stamp` under the same policy
reuses this input node, while keeping its own parameter name and diagnostic path.
If the input node is unsupported — because a field's conversion is — this
function is skipped with that cause, and no partially planned node is recorded.

## Representation

The order the registry composes for the input side, and the call and result
around it:

```text
obtain secs carrier -> convert to source i64
obtain nanos carrier -> convert to source i64
construct source::Stamp { secs, nanos }
call source::stamp_sum once
convert its i64 result -> deliver through the native return
```

The first three lines are the input node's body; the last two belong to the
function, not to any conversion. The two scalar conversions are identities here,
which is why they add no code of their own — but the construction and the call
remain registry instructions regardless.

```text
node(input)  = Crossing { source: Stamp, direction: IntoRust }
               relation: Stamp.fields (record)
               children: [i64 IntoRust, i64 IntoRust]
node(output) = Crossing { source: i64, direction: OutOfRust }
               relation: scalar identity
               children: []
```

## Language variants

- [C][fn_values_c]
- [Kotlin/JNI][fn_values_jni]

[fn]: README.md
[fn_requests]: 03-requests.md
[fn_boundary]: 05-boundary.md
[fn_values_c]: 04-values.c.md
[fn_values_jni]: 04-values.jni.md
[struct_values]: ../struct/04-values.md
