<!-- spec: {"kind": "cell", "example": "struct", "stage": "06-retain"} -->

# Record with scalar fields — Retain supported output

[Stage chapter](../../stages/06-retain.md) · [Element path][struct] · [Source crate](../../source.md)
Previous: [Plan value conversions][struct_values] · Next: [Emit bindings][struct_emit]

## Input

The candidate public declaration for the record, its conversion node, its child
nodes, and the artifacts they require.

## Owner

The registry.

## Result

For this path the outcome is `Emitted`. The public declaration requires the
record's representation and, through it, both field conversions; all are ready,
so the declaration and its artifact are retained — the `repr(C)` aggregate for C,
the Kotlin data class for JNI. [The function][fn_retain] that requires this
record is retained in the same pass, and the artifact order puts the record's
declaration before the wrapper that takes it by value.

## Checks

The record is a root: it is retained even if no exported function uses it, and it
would still be retained had the function been skipped for a reason of its own.
The propagation runs the other way too — an unsupported field makes the record's
conversion unsupported, which skips the record and every declaration that
requires it, all carrying the one underlying cause with their own dependency
paths. Retention is all-or-nothing: a record is never emitted with a field
omitted, because a foreign type missing a field is a different type, not a
partial one.

## Representation

```text
SurfaceSpec {
  element:  ElementId(public Stamp in this target),
  requires: [ node(Stamp, IntoRust), node(i64, IntoRust) x2 ],
  members:  [],
  payload:  Stamp aggregate  |  example.Stamp data class
}

outcome(public Stamp) = Emitted { artifacts: [ type declaration ] }

artifact order:  type declaration -> native wrapper that uses it
```

[struct]: README.md
[struct_values]: 04-values.md
[struct_emit]: 07-emit.md
[fn_retain]: ../fn/06-retain.md
