<!-- spec: {"kind": "cell", "example": "struct", "stage": "06-retain"} -->

[Stage chapter](../../stages/06-retain.md) · [Element path][struct] · [Source crate](../../source.md)
Owner: the registry · Previous: [Plan value conversions][struct_values] · Next: [Emit bindings][struct_emit]

# Record with scalar fields — Retain supported output

## Input

The candidates this record produced, and what they require:

```text
candidate: SurfaceSpec(public Stamp)

requires:  node(Stamp, IntoRust)     // the record conversion
           node(i64, IntoRust)       // one cached child node used by both fields
```

## Result

Because both field [conversions](../../stages/04-values.md#plan-value-conversions) succeed, the public record can be retained.
The following summarizes that dependency relationship rather than showing the
actual `SurfaceSpec` fields: current public requirements refer to declaration
ids, while conversion dependencies are checked during planning.

```text
SurfaceSpec {
    declaration: DeclarationId("type:Stamp"),
    requires: [], // no other public declarations for these scalar fields
    rust:     [ repr(C) aggregate artifact ] for C, [] for JNI,
    payload:  None for C, Kotlin class metadata for JNI,
}

outcome(public Stamp) = Emitted

output: type declaration, followed by any generated wrappers
```

## Checks

An explicitly requested type is a root, so it can remain in the output
even if a function using it is skipped for a separate reason. Conversely, a
function requiring this type cannot remain if the type is unavailable. Cause
propagation preserves the explanation and adds the dependent declaration's path.

- The record is a root: retained even if no exported function uses it, and
  retained if [the function that does][fn_retain] is skipped for a reason of its
  own.
- Propagation runs the other way too: an unsupported field makes the record's
  [conversion](../../stages/04-values.md#plan-value-conversions) unsupported, which skips the record and every declaration requiring
  it, all carrying the one cause with their own dependency paths.
- Retention is all or nothing. A record is never emitted with a field omitted,
  because a foreign type missing a field is a different type.

[struct]: README.md
[struct_values]: 04-values.md
[struct_emit]: 07-emit.md
[fn_retain]: ../fn/06-retain.md
