# Directions and crossings

## The direction

```rust
/// Which of the two directions a crossing is, as a value.
pub enum Direction {
    /// Build this crossing's Rust value — from its parts, or from wire values
    /// where the shape has no parts.
    Construct,
    /// Take this crossing's Rust value apart — into its parts, or into wire
    /// values where the shape has none.
    Deconstruct,
}

impl Direction {
    /// The other direction.
    pub fn swap(self) -> Self;
}
```

`swap` has one caller: a crossing of a callback type. Rust receives the
callable, so that crossing constructs, while the values its arguments carry are
ones Rust already holds and pushes out through the call, so those crossings
deconstruct. The registry applies `swap` there, and no declaration states it.

## The crossing

```rust
/// One Rust type and one of the two directions: the question the table answers.
pub struct Crossing {
    ty: TypeRef,
    direction: Direction,
}
```

A word on `TypeRef`, since three of the accessors below return one. It belongs
to `prebindgen-flat`, and this proposal does not change it: it is the model's
classification of a Rust type into a closed grammar — `Optional`, `Vec`, `Ref`,
`Callback`, `Named` and the rest — which an adapter matches on rather than
re-parsing syntax for itself. `TypeKey` is that same type reduced to an
identity a map can be keyed by.

The generated code has to name Rust types. Where a source function's parameter
is written `&Sample`, the converter for it has to produce a `&Sample`, because
that is what the call takes and `Sample` would not compile there. What
converts, though, is a `Sample` — and the `&` is what tells the wrapper it may
not move out of what it was handed. One `TypeRef` holds all three answers, so
`Crossing` gives each its own accessor rather than leaving every adapter to
peel the type itself.

```rust
impl Crossing {
    /// `ty` is kept exactly as the site wrote it — borrow and transparent
    /// wrappers included. Only the key normalizes.
    pub fn new(ty: TypeRef, direction: Direction) -> Self;

    /// The type exactly as the site wrote it: `&Sample`, `Box<Sample>`,
    /// `Sample`. What generated Rust writes to name this position.
    pub fn spelled(&self) -> &TypeRef;

    /// The Rust value that crosses: the written type with a borrow peeled off.
    /// `&Sample` and `Sample` both answer `Sample`.
    pub fn value(&self) -> &TypeRef;

    /// Which direction.
    pub fn direction(&self) -> Direction;

    /// Whether that value is handed over or reached through a borrow, read off
    /// the way it was written: `&mut T` is `Exclusive`, `&T` is `Shared`,
    /// anything else is `Owned`.
    pub fn mode(&self) -> Mode;

    /// The erased form, for maps and diagnostics.
    pub fn key(&self) -> CrossingKey;
}
```

`spelled()` and `value()` read the same `TypeRef` two ways, and the key a
third:

| | `&Sample` | `Box<Sample>` | `Sample` |
|---|---|---|---|
| `spelled()` | `&Sample` | `Box<Sample>` | `Sample` |
| `value()` | `Sample` | `Box<Sample>` | `Sample` |
| `mode()` | `Shared` | `Owned` | `Owned` |
| `key().ty` | `Sample` | `Sample` | `Sample` |

An adapter decides how to convert from `value()`, and writes `spelled()` into
the generated code. `mode()` is the third answer, kept separate because the
table checks it: a constructor taking `Sample` cannot be handed a part that
only yields `Shared`.

Note `Box<Sample>` — `value()` keeps the wrapper because a `Box` is not a
borrow, while `key()` strips it, because `Sample`, `&Sample` and `Box<Sample>`
all reach **one** recipe. That is what makes a recipe declared once serve every
way its type can be written.

## The key

```rust
/// A crossing identified rather than described, the way `TypeKey` identifies
/// what `TypeRef` describes. What a map key and an error report carry.
pub struct CrossingKey {
    /// The value that crosses, with borrow and transparent wrappers — `Box`,
    /// `Cow` and friends — gone.
    pub ty: TypeKey,
    pub direction: Direction,
}
```

`Crossing` is what a site hands the compiler; `CrossingKey` is what the table
and the fragment memo are keyed by. The narrowing is deliberate and one-way —
`key()` exists, its inverse does not — because a key names a recipe and a
recipe is shared by every way its type can be written.

## The recipe's name

```rust
/// Names one of several answers a crossing may have. Adapters mint these; the
/// table attaches no meaning to any particular name.
pub struct RecipeId(String);

impl RecipeId {
    pub fn new(name: impl Into<String>) -> Self;
    /// The name the table gives the recipe it derives for an undeclared crossing.
    pub fn derived() -> Self;
    pub fn as_str(&self) -> &str;
}
```

A crossing is identified by `CrossingKey`; one of its recipes by `(CrossingKey,
RecipeId)`. The names are the adapter's own — `prebindgen-c` uses `whole` for
how a type crosses on its own and `in_field` / `parts` / `payload` for how the
same type crosses inside a container — and the registry never reads one.
`derived()` is the single reserved name, given to the recipe the table builds
for a crossing nobody declared.
