<!-- spec: {"kind": "cell", "example": "struct", "stage": "01-source"} -->

[Stage chapter](../../stages/01-source.md) · [Element path][struct] · [Source crate](../../source.md)
Next: [Build and inspect the source model][struct_flat]

# Record with scalar fields — Capture source items

## Input

The source crate's `Stamp` declaration, marked for capture.

## Owner

The `#[prebindgen]` proc macro, which leaves the type in place and records it.

## Result

One line in the capture file: the kind (a struct), the name `Stamp`, and the
declaration's complete source text — which is why the field names, their types,
their order and their visibility all survive without any of them being a field of
the entry. No `cfg` guarded this item, so that field is absent.

## Checks

Nothing about the fields is interpreted here. A field whose type the model cannot
describe does not stop the capture, and neither does a private one: the text is
stored either way, and what can be done with it is settled later — the model
decides what it can describe, and a representation decides whether it needs to
read a field it is not allowed to.

## Representation

The marked source:

```rust
#[prebindgen]
pub struct Stamp {
    pub secs: i64,
    pub nanos: i64,
}
```

The line it appends to the capture file:

```json
{
  "kind": "struct",
  "name": "Stamp",
  "content": "pub struct Stamp { pub secs: i64, pub nanos: i64 }",
  "source_location": { "file": "src/source.rs", "line": 1, "column": 1 }
}
```

Unlike a function, a type is kept whole: its fields are its declaration, so there
is nothing to leave out.

[struct]: README.md
[struct_flat]: 02-flat.md
