# Flat lookup and source navigation

[V2 project contents](../README.md)

Status: proposed design. API sketches describe intended contracts, not implemented functionality.

Binding-generation companion: [Registry V2](../registry/overview.md).

A view is a read-only handle that retains the [source snapshot](model.md#building-and-retaining-a-model)
containing its information. These methods let consumers navigate that source
without reconstructing types or attaching model identity themselves.

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

---

Previous: [Flat model](model.md) · Next: [Registry purpose](../registry/overview.md)
