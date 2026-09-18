<!-- spec: {"kind": "cell", "example": "typedef", "stage": "01-source"} -->

[Stage chapter](../../stages/01-source.md) · [Element path][typedef] · [Source crate](../../source.md)
Owner: the `#[prebindgen]` proc macro · Next: [Build and inspect the source model][typedef_flat]

# Type alias declaring an opaque handle — Capture source items

## Input

```rust
#[prebindgen]
pub type Ledger = crate::ledger::Ledger;
```

The type behind the alias is declared in a module of the source crate that is
not marked, and its field is private to that crate.

## Result

```json
{
  "kind": "typealias",
  "name": "Ledger",
  "content": "pub type Ledger = crate::ledger::Ledger;",
  "source_location": { "file": "src/source.rs", "line": 10, "column": 1 }
}
```

Capture copies the alias out as text, target path included, and records where
it came from. Nothing follows the path: what `crate::ledger::Ledger` is,
whether it has fields, whether it implements `Drop`, is not the capture's to
know and not recorded. This is the
[source item](../../stages/01-source.md#capture-source-items) the next stage
reads back.

## Checks

- Marking the alias marks the *name*. `crate::ledger::Ledger` itself is not
  captured, and a function mentioning it by that path would fail the next
  stage: the flat namespace holds bare names only.
- No `cfg` guarded this item, so the field is absent.
- The entry exists whether or not any binding exposes the type.

[typedef]: README.md
[typedef_flat]: 02-flat.md
