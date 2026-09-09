<!-- spec: {"kind": "stage", "stage": "02-flat"} -->

# Build and inspect the source model

[Project contents](../README.md)

Status: proposed design. The API sketches state intended contracts, not
implemented functionality.

A capture entry says `stamp: Stamp` because that is what the source said. To
plan a binding, something has to know more than the text: that `Stamp` is a
record declared in this same crate, that it has two fields, and that both are
`i64`. Turning the captured text into that kind of answerable model is this
stage, and the library that does it is **Flat** (`prebindgen-flat`).

Flat reads the whole capture, lowers each declaration into a typed record,
resolves the references between them — the `Stamp` named by the parameter is
matched to the `Stamp` that was declared — and publishes the result:

```rust
// build.rs of a binding crate, continued from the previous chapter
let mut builder = FlatBuilder::new();
builder.add_captures(Source::new(source_crate::PREBINDGEN_OUT_DIR).items_all())?;
builder.add_local_function(stamp_from_millis_signature)?;  // optional, see below
let model = builder.build()?;   // the snapshot; nothing is added to it afterwards
```

Then it can be walked:

```rust
let function = model.function("stamp_sum").expect("captured");
let stamp = function.parameters().next().unwrap().ty(); // the Stamp parameter type
let record = stamp.as_record().expect("a record");      // its declaration
for field in record.fields() {
    println!("{}: {}", field.name().unwrap(), field.ty().type_ref()); // secs: i64, nanos: i64
}
```

Two words in that snippet carry the design. A **snapshot** is one finished,
immutable set of source records: once published, nothing is added to it, so
everything read from it agrees. A **view** — `FunctionView`, `TypeView`,
`RecordView`, `FieldView` — is a read-only handle to something inside a snapshot,
which remembers which snapshot it came from. That memory is why navigation works:
`parameters().next().unwrap().ty()` hands back a type view in the same snapshot,
so `as_record()` can look up the declaration without the caller repeating a name
lookup, and without a name from one build being resolved against another build's
declarations.

Flat answers questions about Rust; it takes no position on bindings. It will
report that `stamp_from_millis(i64) -> Stamp` takes one integer and returns
`Stamp`. Whether that function should therefore be used to *construct* `Stamp`
values for a binding is not something a view says — that decision belongs to the
[registry](03-requests.md#what-the-registry-does), and Flat has no API that
expresses it.

The same split governs failure. Capture data that is malformed or contradictory
is a `ModelError` and fails the build. A declaration whose shape the model does
not describe stays in the snapshot as an unsupported item, carrying its
location; whether that blocks a particular requested binding is decided much
later, and only for the bindings that actually need it.

## What Flat is for

Binding generation is not the only consumer of this model. The same navigation —
find a function, inspect its parameters, follow a type to its declaration, list
a record's fields or an enum's variants — is what an API documentation tool or a
source validator needs, and Flat is usable on its own, with no registry and no
target in the picture.

A view also retains the source information needed for diagnostics and for
eventual Rust emission, so a consumer that reports an error can point at the
line, and the writer at the end of the pipeline can reproduce a type as the
source spelled it.

What Flat models is a supported subset of Rust, not the language. It does not
promise the type inference, trait resolution or full semantics of the compiler,
and a declaration outside that subset is reported rather than approximated.

## What changes from the current API

Flat already has name lookup and typed source records. In the existing code,
[`Flat::function`](../../../prebindgen-flat/src/flat/mod.rs) returns `&Function`, and
[`Function`, `Struct`, `Param`, and `Field`](../../../prebindgen-flat/src/flat/element.rs)
contain most of the facts needed for inspection. [`TypeRef`](../../../prebindgen-flat/src/flat/ty.rs)
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
indices. A **local helper** is a Rust function whose signature the binding
frontend declares instead of obtaining it from annotated-source captures. The
frontend supplies that signature and its source module so Flat can describe the
helper alongside captured functions. Both inputs enter a mutable builder.
Building checks the complete set and publishes a snapshot. All views from that
snapshot retain shared ownership of its storage.

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

A snapshot includes the source language's explicit unsupported items.
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

These methods compare snapshot association — in the `Rc`-based storage above, an
identity comparison of the shared model data — and do not rebind the view. Equivalent
checks cover other view kinds. `ModelError` distinguishes a mismatched model
from other invalid model operations. Private constructors prevent forged
indices and mismatched internal pairs; runtime checks reject valid views from
the wrong snapshot. Rust lifetimes alone would not distinguish two models that
happen to be borrowed for the same duration.

If a future consumer needs import across snapshots, that is an explicit checked
operation with source mapping and validation. Matching names or key strings are
insufficient evidence of equivalence. Cross-snapshot import is outside the first
increment.

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
are added, which needs no large shared trait: a generic item view
supports enumeration and classification, while a function still exposes parameters,
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

## Derived types and source fidelity

Planning may need a type that no captured signature writes, such as `Option<T>`
around an existing modeled type. A type view should support checked structural
composition, so that such a type is built rather than spelled:

```rust
impl Flat {
    pub fn scalar(&self, kind: ScalarKind) -> TypeView;
}

impl TypeView {
    pub fn optional(&self) -> Result<TypeView, ModelError>;
    pub fn shared_borrow(&self) -> Result<TypeView, ModelError>;
}
```

`ScalarKind` is Flat's supported scalar vocabulary, so naming a scalar cannot
fail; wrapping an existing type can, which is why only the second returns a
`Result`. Composition retains the
operand's snapshot and builds a consistent reading inside Flat. The resulting
view owns the derived reading; it does not insert another captured declaration
into the immutable snapshot. Invalid grammar combinations return `ModelError`.
Borrow construction supplies a type description; the registry must separately
prove that the generated temporary or source borrow has the required lifetime.
More complex constructors must check that all operand views share a snapshot.

Inspection must preserve opaque declarations as opaque. Current Flat lowers
tuple structs to `Extern` records — its record kind for a declaration whose
fields it does not model — and does not lower those fields.
Returning a `RecordView` for them would therefore need an additional source-model
extension. The first views expose existing facts. Later structural extensions
may represent additional fields, but must not make previously accepted opaque
items fail because an unused field is outside the structural grammar.

Final Rust emission reads retained source facts through Flat's emission
facilities. A view is not a way back to the original syntax: any source fact
needed to render an operation belongs in Flat's structured model, where it can be
checked, rather than being recovered from the text by whoever needs it. Source legality and field accessibility must be represented
or diagnosed before the registry claims support for construction or projection.

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

## Elements at this stage

- [Function taking an owned record][fn_flat]
- [Record with scalar fields][struct_flat]

---

Previous: [Capture source items](01-source.md) · Next: [Record binding requests](03-requests.md)

[fn_flat]: ../examples/fn/02-flat.md
[struct_flat]: ../examples/struct/02-flat.md
