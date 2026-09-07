# Flat snapshots, identity and derived types

[V2 project contents](../README.md)

Status: proposed design. API sketches describe intended contracts, not implemented functionality.

Binding-generation companion: [Registry V2](../registry/overview.md).

The [inspection API](inspection.md) returns function, parameter, type and field
views. This page describes how Flat retains their source data and keeps each
view associated with the correct model.

## Building and retaining a model

A **snapshot** is one completed, immutable set of source records and lookup
indices. Source captures and declared local helper signatures enter a mutable
builder. Building checks the complete set and publishes a snapshot. All views
from that snapshot retain shared ownership of its storage.

```rust
impl FlatBuilder {
    // Existing capture ingestion also belongs on this builder.
    pub fn add_local_function(
        &mut self,
        signature: LocalFunctionInput,
    ) -> Result<(), ModelError>;

    pub fn build(self) -> Result<Flat, ModelError>;
}
```

`LocalFunctionInput` contains the helper's declared signature, source-module
qualification and available diagnostic location. Flat lowers the signature with
the same grammar as captured functions. Input parsing may consume Rust syntax;
inspection after lowering uses the typed model. Adding local helpers must check
name collisions and preserve the distinction between captured source modules
and helper-module qualification.

The frontend registers all helpers before creating registry requests containing
views. Registering a helper after publication requires building another
snapshot. That new snapshot has a different identity; previously issued views
continue to describe their original snapshot.

Cloning `Flat` or a view shares the snapshot. Dropping the original `Flat` value
does not invalidate views retained in requests or completed generation plans.
The first implementation can use `Rc` for shared ownership, consistent with
Flat's existing single-threaded source data. This design makes no `Send` or
`Sync` guarantee; parallel planning would require a separate storage audit.

A snapshot includes the source language's explicit unsupported-item records.
Flat must preserve their identity and diagnostic information. A construct that
Flat can retain as unsupported is different from malformed capture data or a
contradictory model, which returns `ModelError`. Whether an unsupported source
item prevents a requested binding is the registry's decision.

## Private storage and model consistency

The following internals illustrate the ownership rule. Every field is private
to Flat; callers receive accessors instead of mutable records.

```rust
pub struct Flat {
    data: Rc<ModelData>, // Immutable records, indices and source-emission data.
}

pub struct FunctionView {
    data: Rc<ModelData>,
    index: FunctionIndex, // Private index validated when Flat creates the view.
}

pub struct ParameterView {
    function: FunctionView,
    index: usize, // Private position validated against that function's signature.
}

pub struct TypeView {
    data: Rc<ModelData>,
    reading: Rc<TypeRef>, // Checked in this model; retained with its source context.
}

pub struct RecordView {
    ty: TypeView, // Exact type use, including applicable lifetime arguments.
    index: RecordIndex, // Private, derived from that type's declaration in its model.
}

pub struct FieldView {
    record: RecordView,
    index: usize, // Private position validated against that record.
}
```

`ModelData` is Flat's immutable storage; `FunctionIndex` and `RecordIndex` are
internal table positions. The shared storage identifies the snapshot. Flat derives a record index from
the retained type in that snapshot when creating the record view.

Views can be cloned but do not expose constructors accepting indices, source
records or model/type pairs. Accessors derive children from the retained parent.
A copied public `TypeRef` cannot be reattached to a different snapshot merely
because a type with the same key exists there.

The registry does need to check that incoming views belong to its source model.
Flat provides a check such as:

```rust
impl Flat {
    pub fn check_type(&self, ty: &TypeView) -> Result<(), ModelError>;
    pub fn check_function(&self, function: &FunctionView) -> Result<(), ModelError>;
}
```

These methods compare snapshot association and do not rebind the view. Equivalent
checks cover other view kinds. `ModelError` distinguishes a mismatched model
from other invalid model operations. Private constructors prevent forged
indices and mismatched internal pairs; runtime checks reject valid views from
the wrong snapshot. Rust lifetimes alone would not distinguish two models that
happen to be borrowed for the same duration.

If a future consumer needs import across snapshots, that is an explicit checked
operation with source mapping and validation. Matching names or key strings are
insufficient evidence of equivalence. Cross-snapshot import is outside the first
increment.

## Derived types and source fidelity

Planning may need a type that no captured signature writes, such as `Option<T>`
around an existing modeled type. A type view should support checked structural
composition without reparsing source spelling:

```rust
impl Flat {
    pub fn scalar(&self, kind: ScalarKind) -> TypeView;
}

impl TypeView {
    pub fn optional(&self) -> Result<TypeView, ModelError>;
    pub fn shared_borrow(&self) -> Result<TypeView, ModelError>;
}
```

`ScalarKind` is Flat's supported scalar vocabulary. Composition retains the
operand's snapshot and builds a consistent reading inside Flat. The resulting
view owns the derived reading; it does not insert another captured declaration
into the immutable snapshot. Invalid grammar combinations return `ModelError`.
Borrow construction supplies a type description; the registry must separately
prove that the generated temporary or source borrow has the required lifetime.
More complex constructors must check that all operand views share a snapshot.

Inspection must preserve opaque declarations as opaque. Current Flat lowers
tuple structs to opaque `Extern` records and does not lower their fields.
Returning a `RecordView` for them would therefore need an additional source-model
extension. The first views expose existing facts. Later structural extensions
may represent additional fields, but must not make previously accepted opaque
items fail because an unused field is outside the structural grammar.

Final Rust emission reads retained source facts through Flat's emission
facilities. Views are not a new route for reparsing captured syntax during
planning. Any new source fact needed to render an operation belongs in Flat's
structured model. Source legality and field accessibility must be represented
or diagnosed before the registry claims support for construction or projection.
