<!-- spec: {"kind": "cell", "example": "fn_callback", "stage": "01-source"} -->

[Stage chapter](../../stages/01-source.md) · [Element path][fn_callback] · [Source crate](../../source.md)
Owner: the `#[prebindgen]` proc macro · Next: [Build and inspect the source model][fn_callback_flat]

# Function taking a callback — Capture source items

## Input

```rust
#[prebindgen]
pub fn stamp_each(stamp: Stamp, each: impl Fn(i64) + Send + Sync + 'static) {
    each(stamp.secs);
    each(stamp.nanos);
}
```

## Result

Capture records the function as it records [`stamp_sum`][fn_source]: its
signature as text, its body replaced. The following is a readable illustration
of the JSON record written as one line in `OUT_DIR`:

```json
{
  "kind": "function",
  "name": "stamp_each",
  "content": "pub fn stamp_each(stamp: Stamp, each: impl Fn(i64) + Send + Sync + 'static) { /* placeholder */ }",
  "source_location": { "file": "src/source.rs", "line": 10, "column": 1 }
}
```

The callback's type is part of that text, bounds included. Nothing about the
callback is recorded on its own: `impl Fn(i64)` is a type the function
mentions, not an item the source crate declares, so there is no separate
record for it and nothing to annotate.

## Checks

- The bounds travel with the type. `Send`, `Sync` and `'static` are what let
  the generated closure be called from another thread after the [wrapper](../../stages/06-boundary.md#assemble-the-wrapper-boundary) has
  returned, and the next stage reads them rather than assuming them.
- The body is replaced by a placeholder: that `stamp_each` calls `each` twice,
  in field order, is the source function's business, and the
  [wrapper](../../stages/06-boundary.md#assemble-the-wrapper-boundary) never
  sees it.
- The entry exists whether or not any binding exposes the function.

[fn_callback]: README.md
[fn_callback_flat]: 02-flat.md
[fn_source]: ../fn/01-source.md
