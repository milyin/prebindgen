<!-- spec: {"kind": "cell", "example": "struct", "stage": "06-retain"} -->

[Stage chapter](../../stages/06-retain.md) · [Element path][struct] · [Source crate](../../source.md)
Owner: the registry · Previous: [Plan value conversions][struct_values] · Next: [Emit bindings][struct_emit]

# Record with scalar fields — Retain supported output

## Input

The candidates this record produced, and what they require:

```text
candidate: SurfaceSpec(public Stamp)

requires:  node(Stamp, IntoRust)     // the record conversion
           node(i64, IntoRust) x2    // its two children
```

## Result

```text
SurfaceSpec {
    element:  ElementId(public Stamp in this target),
    requires: [ node(Stamp, IntoRust), node(i64, IntoRust) x2 ],
    members:  [],
    payload:  the repr(C) aggregate  |  the example.Stamp data class
}

outcome(public Stamp) = Emitted { artifacts: [ type declaration ] }

artifact order: type declaration -> the wrapper that takes it by value
```

## Checks

- The record is a root: retained even if no exported function uses it, and
  retained if [the function that does][fn_retain] is skipped for a reason of its
  own.
- Propagation runs the other way too: an unsupported field makes the record's
  conversion unsupported, which skips the record and every declaration requiring
  it, all carrying the one cause with their own dependency paths.
- Retention is all or nothing. A record is never emitted with a field omitted,
  because a foreign type missing a field is a different type.

[struct]: README.md
[struct_values]: 04-values.md
[struct_emit]: 07-emit.md
[fn_retain]: ../fn/06-retain.md
