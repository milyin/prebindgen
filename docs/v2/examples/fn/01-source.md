<!-- spec: {"kind": "cell", "example": "fn", "stage": "01-source"} -->

# Function taking an owned record — Capture source items

[Stage chapter](../../stages/01-source.md) · [Element path][fn] · [Source fixture](../../source.md)

## Input

The source crate's `stamp_sum`, marked for capture, compiled as part of that
crate.

## Owner

The `#[prebindgen]` proc macro. It re-emits the function unchanged and writes one
record describing it.

## Result

One captured function record: the name `stamp_sum`, the module path it was
written in, its signature as written — one parameter named `stamp` of type
`Stamp`, returning `i64` — the capture group it was assigned, any feature guards
around it, and the source location. The record names `Stamp`; it does not contain
`Stamp`'s definition. That arrives separately, from [the record's own
capture][struct_source], and the two are related only by name until the next
stage resolves them.

## Checks

Capture is not resolution: a parameter type is retained as written, so a record
naming a type that no capture defines is possible and has to be diagnosed later,
not here. Capture is not selection either — this record exists whether or not any
binding exposes the function. The same source produces the same record, so an
unchanged crate regenerates identical capture output.

## Representation

The marked source:

```rust
#[prebindgen]
pub fn stamp_sum(stamp: Stamp) -> i64 {
    stamp.secs.wrapping_add(stamp.nanos)
}
```

The captured record, schematically — one line of the capture file:

```json
{
  "kind": "function",
  "name": "stamp_sum",
  "module": "crate::source",
  "group": "default",
  "signature": "pub fn stamp_sum(stamp: Stamp) -> i64",
  "location": { "file": "src/source.rs", "line": 6 }
}
```

The body is not part of the record: nothing downstream needs it. The generated
wrapper calls `crate::source::stamp_sum`, and the source crate keeps the
implementation.

## Along this element

Next: [Build and inspect the source model][fn_flat]

[fn]: README.md
[fn_flat]: 02-flat.md
[struct_source]: ../struct/01-source.md
