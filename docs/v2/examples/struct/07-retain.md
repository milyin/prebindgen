<!-- spec: {"kind": "cell", "example": "struct", "stage": "07-retain"} -->

[Stage chapter](../../stages/07-retain.md) · [Element path][struct] · [Source crate](../../source.md)
Owner: the registry · Previous: [Represent and compose values][struct_represent] · Next: [Emit bindings][struct_emit]

# Struct with scalar fields — Retain supported output

## Input

The candidates this struct produced, and what they require:

```text
candidate: output type:Stamp, with node(Stamp, IntoRust) at its root

planned:   node(Stamp, IntoRust)     // the struct conversion
           node(i64, IntoRust)       // one cached child node used by both fields
requires:  nothing                   // an i64 names no type another output declares
```

## Result

Because both field [conversions](../../stages/04-select.md#select-conversion-relations) succeed, the public struct can be retained.
Conversion dependencies are checked during planning; retention checks only the
public declarations the planned values name.

```text
Retained {
    output:       type:Stamp,
    declaration:  public Stamp,
    output_value: node(Stamp, IntoRust),
}

outcome(public Stamp) = Emitted
```

What the retained output becomes is the target's to write: the
[wire type](../../stages/05-represent.md#describing-target-values-and-operations)
its root [node](../../stages/05-represent.md#represent-and-compose-values)
resolved to — the `repr(C)` aggregate for C, a `JObject` for JNI —
is fed to the target's wire type writer at emission, and the Kotlin writer reads
the output's metadata, the `example.Stamp` data class.

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
