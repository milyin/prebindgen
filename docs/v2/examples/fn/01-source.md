<!-- spec: {"kind": "cell", "example": "fn", "stage": "01-source"} -->

[Stage chapter](../../stages/01-source.md) · [Element path][fn] · [Source crate](../../source.md)
Owner: the `#[prebindgen]` proc macro · Next: [Build and inspect the source model][fn_flat]

# Function taking an owned record — Capture source items

## Input

```rust
#[prebindgen]
pub fn stamp_sum(stamp: Stamp) -> i64 {
    stamp.secs.wrapping_add(stamp.nanos)
}
```

## Result

The annotation makes the function's signature available to a later binding
build. The original implementation remains in the source crate; the generated
[wrapper](../../stages/06-boundary.md#assemble-the-native-boundary) will call it rather than copy its calculation. The following is a
readable illustration of the JSON record written as one line in `OUT_DIR`:

```json
{
  "kind": "function",
  "name": "stamp_sum",
  "content": "pub fn stamp_sum(stamp: Stamp) -> i64 { /* placeholder */ }",
  "source_location": { "file": "src/source.rs", "line": 6, "column": 1 }
}
```

`kind` tells the reader which Rust item to parse, `name` identifies it,
`content` holds Rust token text, and `source_location` gives a location for
diagnostics. Whitespace and the shown placeholder are illustrative. At this
point `Stamp` is still just a name in that text; capture has not looked up its
fields or chosen a C/JNI [representation](../../stages/05-represent.md#represent-and-compose-values).

## Checks

- The body is replaced by a placeholder: the generated [wrapper](../../stages/06-boundary.md#assemble-the-native-boundary) calls
  `stamp_sum`, so only the signature has to travel.
- `content` is text. `Stamp` here is a name in a string, matched to
  [the record's own capture][struct_source] only by the next stage.
- No `cfg` guarded this item, so the field is absent.
- The entry exists whether or not any binding exposes the function.

[fn]: README.md
[fn_flat]: 02-flat.md
[struct_source]: ../struct/01-source.md
