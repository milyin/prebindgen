<!-- spec: {"kind": "cell", "example": "struct", "stage": "01-source"} -->

# Record with scalar fields — Capture source items

[Stage chapter](../../stages/01-source.md) · [Element path][struct] · [Source fixture](../../source.md)

## Input

The source crate's `Stamp` declaration, marked for capture.

## Owner

The `#[prebindgen]` proc macro, which leaves the type in place and records it.

## Result

One captured type record: the name `Stamp`, its module path, the declaration as
written — a named-field struct with `secs: i64` and `nanos: i64`, both public —
its capture group, feature guards and source location. Field order is part of the
record, because it is part of the declaration.

## Checks

Capture keeps what it cannot interpret rather than dropping it: a field of a type
this grammar does not model leaves the record present and marked unsupported, so
the reason reaches the report at the end of the pipeline. Field privacy is
recorded, not judged — whether a private field blocks a representation is decided
much later, and only for representations that need to read it.

## Representation

The marked source:

```rust
#[prebindgen]
pub struct Stamp {
    pub secs: i64,
    pub nanos: i64,
}
```

The captured record, schematically:

```json
{
  "kind": "struct",
  "name": "Stamp",
  "module": "crate::source",
  "group": "default",
  "shape": "named",
  "fields": [
    { "name": "secs", "ty": "i64", "vis": "pub" },
    { "name": "nanos", "ty": "i64", "vis": "pub" }
  ],
  "location": { "file": "src/source.rs", "line": 1 }
}
```

## Along this element

Next: [Build and inspect the source model][struct_flat]

[struct]: README.md
[struct_flat]: 02-flat.md
