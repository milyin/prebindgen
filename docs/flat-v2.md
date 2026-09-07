# Flat V2: checked source inspection

Status: proposed design. This document specifies the source-model API used by
[the V2 registry and language-generator project](v2-architecture.md). Both designs
support [issue #720](https://github.com/milyin/prebindgen/issues/720). Rust examples
are schematic contracts, not implemented APIs.

## Purpose

Flat describes the Rust items captured from annotated source. A consumer needs
to find a function, inspect its parameters, follow their types to declarations,
and inspect fields or enum variants. Those operations are useful to a binding
frontend, the registry, an API documentation tool, or a source validator.

Flat V2 should provide that navigation through a consistent, read-only API. A
**view** is an inspectable handle that retains the source model containing its
information. Following a parameter to its type, or a record to a field, keeps
that model association. A view also retains the source information needed for
diagnostics and eventual Rust emission.

The independent utility of these views is checked source inspection. The
[registry](v2-architecture.md#1-what-the-registry-does) adds decisions about which
values to convert and how to combine conversions. For example, Flat reports that
`stamp_from_millis(i64) -> Stamp` takes one integer and returns `Stamp`. The
registry decides whether that function is selected to construct a value for a
binding.

This project covers Flat's supported source language. It does not promise the
type inference, trait resolution, or full Rust semantics of the Rust compiler.

## What changes from the current API

Flat already has name lookup and typed source records. In the existing code,
[`Flat::function`](../prebindgen-flat/src/flat/mod.rs) returns `&Function`, and
[`Function`, `Struct`, `Param`, and `Field`](../prebindgen-flat/src/flat/element.rs)
contain most of the facts needed for inspection. [`TypeRef`](../prebindgen-flat/src/flat/ty.rs)
already classifies source types and preserves wrappers and references.

The new API adds three guarantees around those facts:

1. Public views retain one immutable model and expose read-only records. A
   consumer cannot change a signature or field while retaining unrelated source
   data behind that record.
2. Navigation returns views associated with that same model, including derived
   child types. Consumers do not have to repeat name lookup and attach model
   identity themselves.
3. Lookup, enumeration, source locations, identity checks, and model ownership
   follow the same rules across item kinds.

Function lookup returns `FunctionView` directly. The view supports inspection
and retains the model; Flat keeps the underlying function index private.

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

## Lookup and navigation

Lookup and enumeration return the same kind of handle. The prototypes below
show the first useful path through the API:

```rust
impl Flat {
    pub fn function(&self, name: &str) -> Option<FunctionView>;
    pub fn functions(&self) -> impl Iterator<Item = FunctionView>;

    pub fn declared_type(&self, name: &str) -> Option<TypeDeclView>;
    pub fn types(&self) -> impl Iterator<Item = TypeDeclView>;

    pub fn element(&self, name: &str) -> Option<ElementView>;
    pub fn elements(&self) -> impl Iterator<Item = ElementView>;
}

impl FunctionView {
    pub fn name(&self) -> &str;
    pub fn location(&self) -> &SourceLocation;
    pub fn parameters(&self) -> impl Iterator<Item = ParameterView>;
    pub fn return_type(&self) -> TypeView;
}

impl ParameterView {
    pub fn name(&self) -> &str;
    pub fn index(&self) -> usize;
    pub fn location(&self) -> &SourceLocation;
    pub fn ty(&self) -> TypeView;
}
```

`TypeDeclView` describes a named type declaration. `ElementView` distinguishes
functions, type declarations, constants, guards and unsupported items; guards
are captured checks with no named public API. Typed lookup returns `None` when
no accepted item of that kind exists under the name. A consumer needing to
distinguish a missing name, wrong item kind and unsupported declaration uses
`element`. Enumeration preserves source order.

`SourceLocation` identifies the captured source or declared helper for
diagnostics. Every item and child view exposes its location. Unnamed items use
an optional name on `ElementView`; named function views need no optional name.
Parameters keep declaration order and their containing function's identity.
A parameter index is not a globally unique source identifier.

Constants and enum inspection should follow these conventions as their views
are added. This does not require an enormous common trait. A generic item view
supports enumeration and classification; a function still exposes parameters,
a record exposes fields, and an enum exposes variants.

## Type readings and type views

A **type reading**, the existing `TypeRef`, describes one Rust type's structure
and the source information retained by Flat. A **type view**, `TypeView`, adds
the immutable model in which that reading is interpreted. Only the type view
can follow named types to that model's declarations.

This design keeps both because structural readings are already useful to Flat's
parser and emission code. V2 source inspection and registry planning use
`TypeView`. Returning a reading for inspection or emission does not make that
reading a replacement for a checked view in another model.

```rust
impl TypeView {
    pub fn type_ref(&self) -> &TypeRef;
    pub fn key(&self) -> TypeKey;
    pub fn location(&self) -> &SourceLocation;

    pub fn declaration(&self) -> Option<TypeDeclView>;
    pub fn as_record(&self) -> Option<RecordView>;
    pub fn referent(&self) -> Option<TypeView>;
    pub fn optional_inner(&self) -> Option<TypeView>;
    pub fn result_parts(&self) -> Option<(TypeView, TypeView)>;
}

impl TypeDeclView {
    pub fn name(&self) -> &str;
    pub fn location(&self) -> &SourceLocation;
    pub fn ty(&self) -> TypeView;
}

impl RecordView {
    pub fn ty(&self) -> TypeView;
    pub fn shape(&self) -> FieldShape;
    pub fn fields(&self) -> impl Iterator<Item = FieldView>;
}

impl FieldView {
    pub fn name(&self) -> Option<&str>;
    pub fn index(&self) -> usize;
    pub fn location(&self) -> &SourceLocation;
    pub fn ty(&self) -> TypeView;
}
```

`TypeKey` is Flat's normalized equality/hash key for a type reading. A view's
`key()` derives the key from its retained reading. The key alone does not carry
model identity and cannot construct a view. Key text is useful for diagnostics
and internal indexing; planning APIs accept views instead of user-supplied keys.

`declaration()` follows an exact named type to its declaration. It does not
silently peel `Box`, `Cow`, references or optional values. `as_record()` returns
a record only when the exact type is a structurally available record.
`referent()` explicitly follows `&T` or `&mut T`; the original view retains the
reference and its mutability. Equivalent explicit accessors are needed for the
other supported wrappers.

`FieldShape` preserves named, tuple and unit forms. `FieldView::name()` is absent
for a positional field, while `index()` always records source order. Field
identity includes its owner; fields in two enum variants remain distinct even
when both occupy index zero. Extending enum views should preserve variant
identity, payload shape, and modeled discriminant information. This source
variant identity does not choose a registry conversion alternative or foreign
tag encoding.

`result_parts()` returns the `Ok` and `Err` type views of a `Result`, without
interpreting either as a conversion success or failure. A return type of `()`
is still a type view; it is not represented by a missing return type.

These accessors preserve the exact type use. Lifetime arguments and any other
supported arguments must be reflected when looking up field types. A declaration
of a Rust type with arbitrary type/const generic parameters is not supported by
the current Flat grammar. Adding generic instantiation requires explicit Flat
support and tests; a view must not substitute a bare declaration for an
instantiated type and silently lose its arguments.

## A useful consumer without a registry

A source-inspection tool can list the fields of a function's record parameter:

```rust
let function = model.function("normalize").ok_or("missing normalize")?;
for parameter in function.parameters() {
    let ty = parameter.ty();
    if let Some(record) = ty.as_record() {
        for field in record.fields() {
            // Display source facts; this does not select a binding representation.
            println!("{}: {}", field.index(), field.ty().type_ref());
        }
    }
}
```

For `normalize(stamp: Stamp)`, this reaches the `Stamp` fields. For a parameter
of type `&Stamp`, the consumer explicitly calls `referent()` before asking for a
record. A documentation tool might display that distinction, while the registry
uses it when checking borrowing requirements.

This path is the first acceptance example for the API. It should work with only
`prebindgen-flat`, independently of registry or language-generator crates.

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

## Boundary with the registry project

The [registry's relation API](v2-architecture.md#4-describing-source-construction-and-decomposition)
consumes `FunctionView` and `RecordView`. The registry assigns conversion roles
and validates them against a requested direction and exact `TypeView`.

| Flat provides | Registry adds |
| --- | --- |
| Function parameters and complete return type | Whether the function constructs a value or projects one from an input. |
| Record shape and typed fields | Recursive field conversions and value construction/decomposition. |
| `Result` child types | Whether a selected constructor treats `Ok` as construction success and routes `Err` as failure. |
| Exact reference/wrapper structure and source access facts | Temporary lifetimes, borrow use and ownership in the generated conversion. |
| Stable snapshot association and normalized type keys | Conversion-cache identity including direction, selected relation and target policy. |
| Source locations and unsupported-item descriptions | Binding-specific dependency paths and skipped-output reports. |

The registry and language frontends use Flat independently. Flat has no
conversion-selection policy, target representation, recursive binding planner,
or dependency on the registry. A `FunctionView` has no `as_constructor()`
method; `ConstructorRelation::new(function)` belongs to the registry library.

## Implementation sequence and acceptance

Implement this API incrementally in `prebindgen-flat`. A V2 module can coexist
with the current API during migration; `Flat` in this document denotes that V2
model. This proposal does not require a new standalone crate or an immediate
breaking rewrite of V1 consumers.

1. Add builder-to-snapshot ownership, helper registration checks and direct
   function lookup/enumeration. Preserve unsupported records and locations.
2. Add parameter/result `TypeView`s, exact record navigation and field views.
   Implement the independent inspection example using existing captured inputs.
3. Add snapshot checks and checked type composition. Have V2 registry requests
   retain views, and derive cache keys privately from those views.
4. Extend constants, enums and other type forms using the same conventions when
   their consumers need them. Existing inputs remain retained and reportable
   throughout; unsupported structural access is not a reason to lose an item.

Required validation for implementation:

- A standalone Flat consumer navigates function → parameter → record → field
  without a registry dependency or source reparsing.
- Lookup and enumeration agree; views remain usable after the original `Flat`
  value is dropped, and cloned views refer to the same snapshot.
- A view from another snapshot is rejected even when both snapshots contain a
  type with the same name and key.
- External callers cannot forge views or mutate their retained records.
- Local helpers are validated before publication, including duplicate names and
  source-module qualification; existing opaque helper types remain representable.
- Reference, optional and fallible navigation preserves wrappers and exact child
  types. Modeled lifetime arguments survive field navigation and emission.
- Opaque items, unsupported records, guards and locations survive migration.
- Derived type views preserve model association without changing source-item
  enumeration, and type/key consistency holds by construction.
- Flat tests validate source inspection and emission. The registry project's
  C/JNI tests separately validate conversion behavior and ownership.
