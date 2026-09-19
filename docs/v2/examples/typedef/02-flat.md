<!-- spec: {"kind": "cell", "example": "typedef", "stage": "02-flat"} -->

[Stage chapter](../../stages/02-flat.md) · [Element path][typedef] · [Source crate](../../source.md)
Owner: Flat · Previous: [Capture source items][typedef_source] · Next: [Record binding requests][typedef_requests]

# Type alias declaring an opaque handle — Build and inspect the source model

## Input

From capture, the line this alias produced, parsed back into the item it holds:

```json
{ "kind": "typealias", "name": "Ledger",
  "content": "pub type Ledger = crate::ledger::Ledger;",
  "source_location": { "file": "src/source.rs", "line": 10, "column": 1 } }
```

## Result

One element in the namespace, under the name `Ledger`:

```text
Element::Type(Type::Extern(Extern {
    name:   Ledger,
    target: Some("crate :: ledger :: Ledger"),   // informational; not classified
    origin: <the captured syntax, src/source.rs:10>,
}))
```

An **extern** is Flat's word for a declared type with nothing behind it: no
fields, no alternatives, no values. It is the element a parameter or return
typed `Ledger` resolves to:

```rust
let open = model.function("ledger_open").unwrap();
let TypeKind::Named { id, .. } = open.ret.kind() else { … };
let Type::Extern(ledger) = model.resolve(id).unwrap() else { … };  // a name, nothing behind it
assert_eq!(ledger.target.as_deref(), Some("crate :: ledger :: Ledger"));
```

`target` is text. It says what the alias pointed at, so an adapter that
recognises a particular foreign type may; the model does not decide that a
`std` target, or any other, is special. Declaring the alias is what lets
`ledger_open` and `ledger_close` survive: their signatures name `Ledger`, and a
name nothing declares refuses the function that uses it.

## Checks

- No later stage can take a `Ledger` apart, and none will try: the one
  [relation](../../stages/04-select.md#what-a-relation-is) the registry offers
  for an extern is the atomic one.
- A generic alias (`pub type Handle<T> = …`) is refused here, because an extern
  has no binder for the parameter and `Handle<u8>` would otherwise resolve
  against it.
- A tuple struct is also an extern: its positional fields are not modelled, so
  it crosses the way an alias does or not at all.

[typedef]: README.md
[typedef_source]: 01-source.md
[typedef_requests]: 03-requests.md
