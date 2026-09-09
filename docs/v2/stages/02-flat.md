<!-- spec: {"kind": "stage", "stage": "02-flat"} -->

[Project contents](../README.md) · Previous: [Capture source items](01-source.md) · Next: [Record binding requests](03-requests.md)

# Build and inspect the source model

Status: proposed design. The API sketches state intended contracts, not
implemented functionality.

The examples in this chapter use one small source crate — a record and a function
over it, marked for binding generation:

```rust
#[prebindgen]
pub struct Stamp { pub secs: i64, pub nanos: i64 }

#[prebindgen]
pub fn stamp_sum(stamp: Stamp) -> i64;
```

## What Flat is for

Capture stored text. Of `stamp_sum` that text says its parameter is written
`Stamp`, which is all text can say; planning a binding needs to know that `Stamp`
is a record with two `i64` fields, declared in this same crate. **Flat**
(`prebindgen-flat`) is what answers that.

It answers it for an accepted subset of Rust, and refuses everything else at this
edge. That refusal is the point: a consumer matches a small set of structures
exhaustively, with no case left over for a form it has never met, and no stage
after this one re-checks what it was handed.

Nothing about it is specific to bindings — a documentation tool or a source
validator can walk the same model, with no registry and no target in sight.

## One flat namespace

Flat lowers each captured item into an **element** — a function, a type, a
constant — and puts every element into **one flat namespace**: a single index by
name, spanning every source crate that was read. That is what the crate's own
name refers to, and it shapes everything else here.

```rust
// build.rs of a binding crate, continued from the previous chapter
let model = Flat::builder()
    .source(source_crate::PREBINDGEN_OUT_DIR)   // the capture, read and parsed
    .build()?;
```

The namespace is flat in a literal sense. Before anything is lowered, every type
mentioned anywhere is normalized to a bare name: `std::option::Option<T>` becomes
`Option`, `source_a::TypeA` becomes `TypeA`, and an alias becomes whatever it
named. Paths are gathered from all items first, so the same type normalizes the
same way wherever it is mentioned. What survives is the set of names the
generated crate will actually be able to write.

Two names cannot collide in one namespace, so a duplicate is the one thing that
fails the build outright, however the source crates were arranged. Everything
else that goes wrong is per item, and does not fail: an item the model cannot
express becomes an **unsupported element** carrying its diagnosis and stays in
the model, so a consumer can enumerate what a source crate marked, refusals
included, instead of wondering what went missing.

## Names, and what happens when one leads nowhere

What is *not* in the model is a link between elements. Where `stamp_sum`'s
parameter mentions `Stamp`, the model records the name `Stamp` — not a pointer to
the declaration, not an index into a table. Whoever wants the declaration looks
the name up in the namespace, and the model is content to store a name until then.

That is what makes the namespace usable. A capture may mention a type before the
declaration is read, or one in another source crate entirely, and neither is a
problem: every name is looked up against the finished namespace, so the order the
captures arrived in changes nothing. And because a name has exactly one
declaration in that namespace, two mentions of `Stamp` are two mentions of the
same type — there is no way to end up holding two `Stamp`s that disagree.

A name can fail to resolve, and the reason is worth being clear about: the
namespace holds what was marked, not what the crate compiles. A marked function
may take a type whose declaration nobody marked, or a foreign type the crate
never gave a name to. Rust is happy with such code; a binding cannot be built
from it, because nothing here describes the value that would have to cross.

```rust
pub struct Missing;                        // ordinary Rust, but not marked

#[prebindgen]
pub struct Broken { pub field: Missing }   // marked, and names an unmarked type

#[prebindgen]
pub fn use_broken(value: Broken) {}        // marked, and takes that struct
```

Flat settles this while building the model, by looking up every type name each
element mentions. `Broken` mentions `Missing`, which no element declares, so
`Broken` cannot be described. Flat replaces it with an unsupported
entry that keeps its name and location and carries the reason, in the terms the
author can act on: *names the type `Missing`, which the flat API does not declare
— mark its declaration `#[prebindgen]`, or, for a foreign or crate-private type
used as a handle, give it a name here with `#[prebindgen] pub type Missing = ..;`*

That changes the namespace, because `Broken` has stopped being a declaration.
`use_broken` takes a `Broken` and was perfectly resolvable a moment ago; now the
name it mentions is gone as well. So Flat goes over the elements again, refuses
`use_broken` for the same reason, and keeps going until a pass refuses nothing
new — here, the third one.

Nothing is discarded on the way, and this does not fail the build. A refused
element stays in the model as an unsupported entry, and `Flat::unsupported()`
enumerates them with their names, locations and reasons, so a consumer can report
exactly what it cannot use. What to do about them is the consumer's decision, and
today's registry makes a strict one: it refuses to build a binding from a model
that contains any unsupported element, and lists all of them at once, so a source
crate that needs fixing is fixed in one pass rather than one item per rebuild.

What survives has no dead ends. Every type name in a surviving element has a
declaration in the namespace to look up, which is what lets every later stage
walk the model without a plan for what to do when a name leads nowhere.

## The shapes the model has

Everything above is about the namespace. The model itself is the other half of
what this crate is, and the more important one: a small set of structures that
hold an accepted subset of Rust and nothing else.

An element is one of five things:

```rust
enum Element {
    Function(Function),      // a name, parameters in order, and a return type
    Type(Type),              // a declared type, in one of the four shapes below
    Constant(Constant),      // a name and its type
    Guard(Guard),            // the injected feature assertion; no name, re-emitted as is
    Unsupported(Unsupported),// a marked item the subset cannot express, with its diagnosis
}

enum Type {
    Struct(Struct),   // fields, each with a name or a position, and a type
    Variant(Variant), // an enum with payloads: alternatives identified by position
    Enum(Enum),       // a fieldless enum: members identified by the integer Rust assigns
    Extern(Extern),   // a name, and nothing behind it
}
```

The four type shapes are the ones later stages actually work from. A **struct**
has fields the model describes, so a binding can take a value apart and put one
back together. The two **enum** shapes are one Rust keyword covering two
concepts, and the model separates them because they are identified differently: a
fieldless enum's members are identified by the value Rust assigns, which a C
header restates and a Kotlin enum entry carries, while an enum with payloads is a
sum whose alternatives are identified by position, since a foreign representation
numbers its own arms and Rust's discriminant would be the wrong number to use. An
**extern** is a name with nothing behind it — `pub type Session = zenoh::Session;`,
or a marked tuple struct — which is how a value that crosses as an opaque handle
enters the API deliberately, rather than by being mentioned somewhere.

Every type written anywhere in those elements — a parameter, a return, a field,
an alternative's payload, an array element — is one **type reading**: a kind,
plus the origin it was lowered from. The kinds are the whole accepted grammar:

```rust
enum TypeKind {
    Scalar(ScalarKind),                     // bool, the integers, isize/usize, f32, f64
    Str, String,
    Optional(TypeRef), Vec(TypeRef), Slice(TypeRef),
    Fallible { ok: TypeRef, err: TypeRef }, // Result<T, E>
    Named { id: TypeId, args: Vec<GenericArg> },
    Array { elem: TypeRef, extent: ArrayExtent },
    Ref { lifetime, mutable, inner: TypeRef },
    Boxed(TypeRef), Cow { lifetime, inner: TypeRef }, Uninit(TypeRef),
    Callback { args: Vec<TypeRef> },        // impl Fn(..) + Send + Sync + 'static
    Unit,
}
```

That list is the point of the crate, and reading it is most of understanding this
stage. A consumer matches these variants and has no other case to handle: there
is no arm for "some other Rust type", and no reason to walk `syn` looking for
one. Enforcement happens once, here — a marked item mentioning a form with no
variant in this grammar is refused at the door and becomes an unsupported
element, so no adapter downstream has to re-check what it was given or decide
what to do with a shape it has never heard of.

`Named` is the variant that holds a name — `Stamp`, plus any generic arguments
written with it. The record it names is a separate element, and getting from one
to the other is the lookup this chapter described earlier. Walking from the
function to its parameter's fields is therefore four steps:

```rust
let function = model.function("stamp_sum").unwrap();       // the Function element
let TypeKind::Named { id, .. } = function.params[0].ty.kind() else { … };
let Type::Struct(stamp) = model.resolve(id).unwrap() else { … };
let fields = &stamp.fields;                                // secs: i64, nanos: i64
```

`resolve` returns nothing only for a name nothing declares, and the pass above
has already refused the elements that could have asked. A consumer walking a
surviving element can follow every name it meets.

The same closure applies above the type level. A shape with no slot in an element
— an `async fn`, a variadic, a generic parameter — is refused rather than
approximated, because the missing piece would otherwise be dropped silently.
Lifetimes are not in that list: they are spelling, and the model keeps them.
Raw pointers are not in the grammar at all, and that absence is a statement about
the whole project: a source crate is idiomatic Rust, and pointers belong to the
stage that builds a native boundary out of it.

The grammar also keeps distinctions a destination language may well erase.
`String` and `str` are different kinds; so are `Vec<T>` and `[T]`; `Box<T>` and
`Cow<'a, T>` stay visible as wrappers rather than being flattened to what they
contain. That a C binding treats several of these alike is a decision for the C
adapter to take deliberately, at the point where it matters — not a decision the
source model takes for everyone by throwing the difference away.

Spelling survives all of this because each element also keeps its **origin**: the
exact syntax it was built from, and the source it arrived in. Generated Rust is the one artifact
that needs that fidelity — `B()` must not be re-spelled `B`, `= 0x07` must not
become `= 7` — so the source's own text rides along for emission to reuse. It is
not a second source of facts: the retained syntax is private to Flat, and the
rule for every consumer is to analyse the model and generate from the model.

Flat answers questions about Rust; it takes no position on bindings. It will
report that `stamp_from_millis(i64) -> Stamp` takes one integer and returns
`Stamp`. Whether that function should therefore be used to *construct* `Stamp`
values for a binding is not something the model says — that decision belongs to
the [registry](03-requests.md#what-the-registry-does), and Flat has no API that
expresses it.

## What changes from the current API

Everything above is what the library does today. What V2 adds is about *handles*,
not about facts, and it revisits none of it.

In the current code, [`Flat::function`](../../../prebindgen-flat/src/flat/mod.rs)
returns `&Function`, borrowed from the model, and
[`Function`, `Struct`, `Param`, and `Field`](../../../prebindgen-flat/src/flat/element.rs)
hold the facts an inspecting consumer needs.
[`TypeRef`](../../../prebindgen-flat/src/flat/ty.rs) already classifies a source
type and preserves its wrappers and references. Following a parameter's type to a
declaration is then the caller's job: take the name out of the reference, call
`Flat::resolve`, keep the model in hand for the next hop.

V2 replaces those borrows with **views**: `FunctionView`, `TypeView`,
`RecordView`, `FieldView` — read-only handles that carry the model they came from
rather than borrowing it. Three things follow.

1. Navigation composes. `parameters().next().ty().as_record()` works because each
   step hands back a view that still knows its model, so the lookup this chapter
   showed as an explicit `resolve` call happens inside the step that needs it.
2. A view outlives the handle it was obtained from, which is what lets a request
   or a completed plan retain one instead of copying facts out of it.
3. Which model a view belongs to is checkable, so a view from one build cannot be
   planned against another build's declarations even when the two contain a type
   with the same name.

Lookup, enumeration, source locations and ownership then follow the same rules
across every item kind, and the index behind a view stays private to Flat.

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

This is also a change from today, where a helper is added to a model that has
already been built. Making the builder the only way in is what lets the published
snapshot be complete: everything the model will ever contain is there when the
first view is handed out.

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
functions, type declarations, constants, guards and unsupported items. A guard
here is the [feature assertion](01-source.md#capture-source-items) injected when
the capture was read: an item the model carries and the writer emits, with no
name in any foreign API. Typed lookup returns `None` when
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

[fn_flat]: ../examples/fn/02-flat.md
[struct_flat]: ../examples/struct/02-flat.md
