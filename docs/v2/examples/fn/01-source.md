<!-- spec: {"kind": "cell", "example": "fn", "stage": "01-source"} -->

# Function taking an owned record — Capture source items

[Stage chapter](../../stages/01-source.md) · [Element path][fn] · [Source crate](../../source.md)
Next: [Build and inspect the source model][fn_flat]

## Input

The source crate's `stamp_sum`, marked for capture, compiled as part of that
crate.

## Owner

The `#[prebindgen]` proc macro. It re-emits the function unchanged and writes one
record describing it.

## Result

One line in the capture file: the kind (a function), the name `stamp_sum`, the
declaration's source text with its body replaced by a placeholder, and the file
and line it came from. There is no `cfg` on this item, so that field is absent.

The text mentions `Stamp`, and that is all it does — the entry does not contain
`Stamp`'s definition, and nothing here has matched the two. The definition
arrives separately, from [the record's own capture][struct_source]; connecting
them is the next stage's work.

## Checks

The body is not kept, because no binding needs it: the generated wrapper calls
the real `stamp_sum` in the source crate, so the signature is the whole of what
travels. Capture is not selection either — this entry exists whether or not any
binding exposes the function. The same source produces the same entry, so an
unchanged crate regenerates identical capture output.

## Representation

The marked source:

```rust
#[prebindgen]
pub fn stamp_sum(stamp: Stamp) -> i64 {
    stamp.secs.wrapping_add(stamp.nanos)
}
```

The line it appends to the capture file:

```json
{
  "kind": "function",
  "name": "stamp_sum",
  "content": "pub fn stamp_sum(stamp: Stamp) -> i64 { /* placeholder */ }",
  "source_location": { "file": "src/source.rs", "line": 6, "column": 1 }
}
```

`content` is text, not a parsed signature: `Stamp` here is a name in a string.
When a binding crate reads this line back, the text is parsed into the item it
came from and stamped with the crate that captured it — which is how the next
stage can tell whose `Stamp` this is.

[fn]: README.md
[fn_flat]: 02-flat.md
[struct_source]: ../struct/01-source.md
