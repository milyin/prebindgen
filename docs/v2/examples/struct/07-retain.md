<!-- spec: {"kind": "cell", "example": "struct", "stage": "07-retain"} -->

[Stage chapter](../../stages/07-retain.md) · [Element path][struct] · [Source crate](../../source.md)
Owner: the registry · Previous: [Represent and compose values][struct_represent] · Next: [Emit bindings][struct_emit]

# Struct with scalar fields — Retain supported output

## Input

The candidates this struct produced, and what they require:

```text
candidate: SurfaceSpec(public Stamp)

requires:  node(Stamp, IntoRust)     // the struct conversion
           node(i64, IntoRust)       // one cached child node used by both fields
```

## Result

Because both field [conversions](../../stages/04-select.md#select-conversion-relations) succeed, the public struct can be retained.
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

- The struct is a root: retained even if no exported function uses it, and
  retained if [the function that does][fn_retain] is skipped for a reason of its
  own.
- Propagation runs the other way too: an unsupported field makes the struct's
  [conversion](../../stages/04-select.md#select-conversion-relations) unsupported, which skips the struct and every declaration requiring
  it, all carrying the one cause with their own dependency paths.
- Retention is all or nothing. A struct is never emitted with a field omitted,
  because a foreign type missing a field is a different type.

[struct]: README.md
[struct_represent]: 05-represent.md
[struct_emit]: 08-emit.md
[fn_retain]: ../fn/07-retain.md
