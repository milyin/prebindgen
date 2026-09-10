<!-- spec: {"kind": "cell", "example": "struct", "stage": "01-source"} -->

[Stage chapter](../../stages/01-source.md) · [Element path][struct] · [Source crate](../../source.md)
Owner: the `#[prebindgen]` proc macro · Next: [Build and inspect the source model][struct_flat]

# Record with scalar fields — Capture source items

## Input

```rust
#[prebindgen]
pub struct Stamp {
    pub secs: i64,
    pub nanos: i64,
}
```

## Result

```json
{
  "kind": "struct",
  "name": "Stamp",
  "content": "pub struct Stamp { pub secs: i64, pub nanos: i64 }",
  "source_location": { "file": "src/source.rs", "line": 1, "column": 1 }
}
```

## Checks

- A type is kept whole: its fields are its declaration, so field names, types,
  order and visibility all survive inside `content` without being fields of the
  entry.
- Nothing about those fields is interpreted here. A field whose type the model
  cannot describe, or a private one, does not stop the capture.
- No `cfg` guarded this item, so the field is absent.

[struct]: README.md
[struct_flat]: 02-flat.md
