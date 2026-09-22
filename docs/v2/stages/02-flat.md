<!-- spec: {"kind": "stage", "stage": "02-flat"} -->

[Project contents](../README.md) · Previous: [Capture source items](01-source.md) · Next: [Record binding requests](03-requests.md)

# Build and inspect the source model

The first part of this chapter describes the implemented `prebindgen-flat`
library, which both engines use to inspect [source items](01-source.md#capture-source-items). The section
[What V2 changes](#what-v2-changes) describes a planned API based on owned views;
those views are not implemented yet. Current V2 uses the existing borrowed API.

The examples in this chapter use one small source crate — a struct and a function
over it, marked for binding generation:

```rust
#[prebindgen]
pub struct Stamp { pub secs: i64, pub nanos: i64 }

#[prebindgen]
pub fn stamp_sum(stamp: Stamp) -> i64;
```

## What Flat is for

After capture, the generator has Rust text such as `stamp: Stamp`. To generate
a [conversion](04-select.md#select-conversion-relations), it needs more than that spelling: is `Stamp` a struct, which
fields does it have, and what are their types? **Flat**, the source model
provided by `prebindgen-flat`, turns the captured syntax into structured answers.

Flat recognizes a defined subset of Rust. For supported items, consumers inspect
typed records and enums rather than parsing source text again. An item outside
that subset is retained with an unsupported diagnosis. This lets the model say
what was captured even when it cannot describe the item's structure. Later
stages still check whether they can generate a binding for a modeled type;
being valid in Flat does not establish target-language support.

Nothing about it is specific to bindings — a documentation tool or a source
validator can walk the same model, with no registry and no target in sight.

## One flat namespace

Flat converts each captured item into an **element**, its structured record for
a function, type, constant, guard or unsupported item. Compiler documentation
often calls this conversion **lowering**: translating input into a simpler
internal model. Named elements enter **one flat namespace**, a single
index shared by all the source crates being read. A lookup asks for `Stamp`, not
for a sequence of nested Rust modules.

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

When `stamp_sum`'s parameter mentions `Stamp`, its type record stores a name.
It does not directly point to the struct declaration. A consumer uses
`Flat::resolve` to look that name up in the completed namespace. This distinction
explains the explicit lookup in the code example below.

Collecting the namespace before resolving names makes forward references work.
A function can mention a type whose capture is read later, including a type
from another source crate. Within one model, a name identifies at most one
declaration; duplicate names are rejected rather than chosen by arrival order.

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

That changes which entries can serve as supported type declarations.
`use_broken` takes a `Broken` and was perfectly resolvable a moment ago; now the
name it mentions is gone as well. So Flat goes over the elements again, refuses
`use_broken` for the same reason, and keeps going until a pass refuses nothing
new — here, the third one.

Nothing is discarded on the way, and this does not fail the build. A refused
element stays in the model as an unsupported entry, and `Flat::unsupported()`
enumerates them with their names, locations and reasons, so a consumer can report
exactly what it cannot use. Consumers decide what to do with those entries.
The V1 registry rejects a model containing any unsupported element and lists
them together. V2 checks the items its declarations actually request; unrelated
unsupported elements do not by themselves fail the run. A request whose source
name cannot be found as the required supported kind fails source validation.

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
    Guard(Guard),            // an unnamed assertion retained for emission
    Unsupported(Unsupported),// a marked item the subset cannot express, with its diagnosis
}

enum Type {
    Struct(Struct),   // named fields and their types (or no fields for a unit struct)
    Variant(Variant), // an enum with payloads: alternatives identified by position
    Enum(Enum),       // a fieldless enum: members identified by the integer Rust assigns
    Extern(Extern),   // a name, and nothing behind it
}
```

The four type shapes preserve different information:

- A **struct** exposes modeled fields. A later planner can use those fields to
  construct or read a value, provided the operation is legal and supported.
- A **variant** describes an enum whose alternatives may contain payloads.
  Alternative positions identify its branches in the model. The foreign
  [representation](05-represent.md#represent-and-compose-values) separately chooses
  how to encode which branch is active.
- An **enum** describes fieldless alternatives and their integer discriminants.
  A target can use those values when generating a C or Kotlin enum.
- An **extern** deliberately exposes only a type name. An alias such as
  `pub type Session = zenoh::Session;` can introduce an external type without
  describing its internals. Current Flat also treats tuple structs as opaque.
  A handle representation can use such a type without reading its fields.

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

Read this enum as Flat's vocabulary for a type occurrence. `Optional` holds the
inner type of `Option<T>`; `Fallible` holds both types of `Result<T, E>`;
`Array` holds an element type and length; `Ref` preserves a borrow's mutability
and lifetime information. `Named` leads to a separately declared type. `Unit`
represents `()`, so a function without a value result still has a modeled
return type. Each nested `TypeRef` describes another occurrence using the same
vocabulary.

A consumer can match these cases exhaustively. If capture contains a form that
the grammar cannot represent, Flat reports the enclosing item as unsupported.
The language adapters therefore do not each need a Rust syntax parser.

`Named` is the variant that holds a name — `Stamp`, plus any generic arguments
written with it. The struct it names is a separate element, and getting from one
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
stage that builds a [wrapper boundary](06-boundary.md#assemble-the-wrapper-boundary) out of it.

The grammar also keeps distinctions a destination language may well erase.
`String` and `str` are different kinds; so are `Vec<T>` and `[T]`; `Box<T>` and
`Cow<'a, T>` stay visible as the enclosing types they are rather than being flattened to what they
contain. That a C binding treats several of these alike is a decision for the C
adapter to take deliberately, at the point where it matters — not a decision the
source model takes for everyone by throwing the difference away.

Spelling survives all of this because each element also keeps its **origin**: the
exact syntax it was built from, and the source it arrived in. Generated Rust is the one output
that needs that fidelity — `B()` must not be re-spelled `B`, `= 0x07` must not
become `= 7` — so the source's own text rides along for emission to reuse. It is
not a second source of facts: the retained syntax is private to Flat, and the
rule for every consumer is to analyse the model and generate from the model.

That rule is about a phase, and it governs both engines. **Planning** is
everything up to final Rust emission: recording requests, selecting
[relations](04-select.md#what-a-relation-is),
describing representations, assembling a boundary. Planning may carry a type
opaquely and use what the model says about it — its kind, its identity, its
parts, its declared fields, its source location — and may not obtain the syntax
behind it, branch on rendered text to reach a decision, or generate a body in
order to discover what that body depends on. **Emission** is where syntax is
legitimate, and where the writers reproduce what the source wrote.

Formatting a type in an error message is allowed. The restriction is about the
source of a decision: a planner should inspect `TypeKind::Optional`, for example,
rather than print a type and test whether its text starts with `Option`.

The types an adapter *authors* are outside the rule entirely. `*mut c_void`,
`jlong`, a `repr(C)` aggregate the binding declares — these are the adapter's
own output vocabulary rather than captured source syntax, so writing them as
syntax during planning is what an adapter is for. That is why a [carrier](05-represent.md#describing-target-values-and-operations) a target
describes here holds a real `syn::Type`, while a source-side position stays a
handle to the model. The rule constrains where a fact may come *from*, not which
types may be spelled.

The engines enforce this separation differently. Both distinguish a rendering
**protocol** (the interface for emitting source types) from an **emission
capability** (the object an engine permits its emission code to use). The
protocol lives with the model, in `prebindgen-flat` — object-safe,
generate-only, emitting source types from the model's own facts rather than
exposing captured type syntax for inspection. Guards and enum discriminants
are special cases copied verbatim for emission. Implementing it is a
deliberate act, which is why each engine establishes its own rather than
borrowing the other's.

In V1 the capability is `prebindgen_registry::RustWriter`, and the phase is
structural rather than a promise: its constructor is private, it holds the
frozen source-module map, it implements the protocol through a private receiver
no public API names, and an emission callback is the only thing that receives
one. The registry does not re-export the protocol through its own model path, so
an adapter depending on the registry alone cannot even name it, and compile-fail
tests pin both restrictions. Two escapes are deliberate: the non-default
`testing` feature hands the capability out so an adapter's test suite can render
in isolation — nothing stops a crate enabling that feature as an ordinary
dependency — and a crate that depends on `prebindgen-flat` directly can
implement the protocol itself, which is what lets a consumer that is not a
binding generator reuse the model without gaining access to retained syntax.

V2 has its own private writer, and reaches an adapter differently: a target is
asked to render one operation and receives the payload it described plus the
operand names the writer allocated, never a rendering capability. Same rule,
same protocol, a different way of keeping planning away from syntax. For the
escapes in either engine the rule is a convention, stated here, rather than a boundary
the compiler enforces.

Flat answers questions about Rust; it takes no position on bindings. It will
report that `stamp_from_millis(i64) -> Stamp` takes one integer and returns
`Stamp`. Whether that function should therefore be used to *construct* `Stamp`
values for a binding is not something the model says — that decision belongs to
the [registry](03-requests.md#what-the-registry-does), and Flat has no API that
expresses it.

## What V2 changes

Everything above is what the library does today, and everything from here to the
end of the chapter is proposed instead of built. It is also the one stage
[the first increment](../implementation.md#the-first-increment-as-built) did not
build: the engine plans over the borrowed API above, and its frozen result owns
the model, so a plan cannot outlive the model it was planned against and no view
from another snapshot can be offered. That holds while a run has exactly one
model, which is what makes the views below the next thing this stage needs rather
than the first. None of it revisits the model's
answers: the elements, the grammar, the namespace and the refusals all stay as
described. What changes is how the model is held and handed out.

In the current code, [`Flat::function`](../../../prebindgen-flat/src/flat/mod.rs)
returns `&Function`, borrowed from the model, and
[`Function`, `Struct`, `Param`, and `Field`](../../../prebindgen-flat/src/flat/element.rs)
hold the facts an inspecting consumer needs.
[`TypeRef`](../../../prebindgen-flat/src/flat/ty.rs) already classifies a source
type and preserves its enclosing types and references. Following a parameter's type to a
declaration is then the caller's job: take the name out of the reference, call
`Flat::resolve`, keep the model in hand for the next hop.

The proposed API replaces those borrows with **views**: `FunctionView`, `TypeView`,
`StructView`, `FieldView` — read-only handles that carry the model they came from
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

### Building and retaining a model

A **snapshot** is one completed, immutable set of source structs and lookup
indices. An **entity** is one real item of the API — a type, a function or a
constant, with its whole description — and where it came from is not a kind
but an origin. A `#[prebindgen]` item is an entity the source captured. An
item the binding defines itself is an entity the binding stated: a helper
function with the signature `fun!(crate::x).sig(..)` declares, or a type the
source never exported that the binding represents as a handle. Both enter the
same mutable builder, and building checks the complete set and publishes a
snapshot. All views from that snapshot retain shared ownership of its storage.

```rust
impl FlatBuilder {
    pub fn items(self, items: impl IntoIterator<Item = (syn::Item, SourceLocation)>) -> Self;
    pub fn local_function(self, sig: syn::Signature, module: syn::Path) -> Self;
    pub fn local_type(self, name: syn::Ident) -> Self;
    pub fn build(self) -> Result<Flat, ParseError>;
}
```

A local function is lowered with the same grammar as a captured one, from the
signature the binding states; `module` is where generated code reaches it —
`crate::helpers` for a helper at `crate::helpers::x` — and its origin records
that as its crate stamp, since for a captured item the stamp means the same
thing. A name the source already captured is a duplicate, and the build
refuses it: the generated call must not be a coin toss between the two. A
local type becomes an extern — a type whose contents the model does not see,
which is exactly what it is — reached at the binding's crate root; when the
source captured a type of that name, that one is what the binding meant, and
the local declaration steps aside.

The model keeps the captured items apart from the binding's own where a
consumer needs the distinction: `Flat::captured` iterates the former alone,
which is what a report counts as the API it was generated against;
`Flat::is_binding_local` answers for one name; and the source-module list is
frozen from the captured stream alone, so a binding-local item never changes
which module an unqualified reference resolves against. Everything else —
lookup by name, the typed accessors, planning — sees one namespace.

The frontend registers its own items before building, so the snapshot is
complete when the first view is handed out. Registering a helper after publication requires building another
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

### Private storage and model consistency

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

pub struct StructView {
    ty: TypeView, // Exact type use, including applicable lifetime arguments.
    index: StructIndex, // Private, derived from that type's declaration in its model.
}

pub struct FieldView {
    view: StructView,
    index: usize, // Private position validated against that struct.
}
```

`ModelData` is Flat's immutable storage; `FunctionIndex` and `StructIndex` are
internal table positions. The shared storage identifies the snapshot. Flat derives a struct index from
the retained type in that snapshot when creating the struct view.

Views can be cloned but do not expose constructors accepting indices, source
records or model/type pairs. Accessors derive children from the retained parent.
A copied public `TypeRef` cannot be reattached to a different snapshot merely
because a type with the same key exists there.

A view therefore belongs to exactly one snapshot, and navigation stays inside
it: `parameters()`, `declaration()` and every other accessor derive the child
from the parent's own storage. A foreign view can only arrive at an operation
that *accepts* a view from its caller — a `Flat` method taking one, or a
registry ingress point such as declaring an item or planning a conversion.
Those operations validate the view themselves and report a mismatch through
their own result:

```rust
impl Flat {
    pub fn check_type(&self, ty: &TypeView) -> Result<(), ModelError>;
    pub fn check_function(&self, function: &FunctionView) -> Result<(), ModelError>;
}
```

These methods compare snapshot association — in the `Rc`-based storage above, an
identity comparison of the shared model data — and do not rebind the view. Equivalent
checks cover other view kinds. `ModelError` distinguishes a mismatched model
from other invalid model operations. Consumers may call them to pre-validate,
but rejection must not depend on a consumer remembering to: an operation that
takes a view is responsible for checking it, so a new adapter cannot opt out of
the rule by omission. One check at ingress covers everything reached from that
handle afterwards, which costs one pointer comparison per declared item.

The check runs in release builds, not only under `debug_assert`. Mixing
snapshots does not crash: a name resolved against the wrong model finds a
same-named declaration of a different shape and emits bindings that compile and
are wrong at the ABI boundary. That is a silent wrong-output failure, so it has
to fail where it happens.

Private constructors prevent forged indices and mismatched internal pairs;
runtime checks reject valid views from the wrong snapshot. Rust lifetimes alone
would not distinguish two models that happen to be borrowed for the same
duration. An invariant brand parameter would move the check to compile time, at
the price of a lifetime parameter on every public type, models that cannot share
a collection, and inference errors in place of `ModelError`; the runtime check
is the cheaper trade here. This design assumes one build may hold more than one
model — several source crates, or per-group snapshots — which is what makes the
check worth its API cost.

If a future consumer needs import across snapshots, that is an explicit checked
operation with source mapping and validation. Matching names or key strings are
insufficient evidence of equivalence. Cross-snapshot import is outside the first
increment.

### Lookup and navigation

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
the capture was read: an item the model retains without giving it a foreign
API name. Both engines re-emit it into the generated Rust.
Typed lookup returns `None` when
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
a struct exposes fields, and an enum exposes variants.

### Type readings and type views

A **type reading**, the existing `TypeRef`, describes a type occurrence such as
`Stamp` or `&Stamp`, including its structure and retained source information.
The proposed **type view**, `TypeView`, pairs that reading with the model that
gives its names meaning. A reading saying `Stamp` is not sufficient by itself
to find fields; the view knows which model's `Stamp` declaration to consult.

This design keeps both because structural readings are already useful to Flat's
parser and emission code. V2 source inspection and registry planning use
`TypeView`. A bare reading carries no model identity, so one taken out of a
view cannot be handed to a different model and resolved there: names that
match across snapshots are not evidence of the same declaration. To resolve a
type against another model, obtain a view from that model rather than reusing
the reading.

```rust
impl TypeView {
    pub fn type_ref(&self) -> &TypeRef;
    pub fn key(&self) -> TypeKey;
    pub fn location(&self) -> &SourceLocation;

    pub fn declaration(&self) -> Option<TypeDeclView>;
    pub fn as_record(&self) -> Option<StructView>;
    pub fn referent(&self) -> Option<TypeView>;
    pub fn optional_inner(&self) -> Option<TypeView>;
    pub fn result_parts(&self) -> Option<(TypeView, TypeView)>;
}

impl TypeDeclView {
    pub fn name(&self) -> &str;
    pub fn location(&self) -> &SourceLocation;
    pub fn ty(&self) -> TypeView;
}

impl StructView {
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
a struct only when the exact type is a structurally available struct.
`referent()` explicitly follows `&T` or `&mut T`; the original view retains the
reference and its mutability. Equivalent explicit accessors are needed for the
other supported type forms.

`FieldShape` preserves named, tuple and unit forms. `FieldView::name()` is absent
for a positional field, while `index()` always records source order. Field
identity includes its owner; fields in two enum variants remain distinct even
when both occupy index zero. Extending enum views should preserve variant
identity, payload shape, and modeled discriminant information. This source
variant identity does not choose a registry [conversion](04-select.md#select-conversion-relations) alternative or foreign
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

### Derived types and source fidelity

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

`ScalarKind` enumerates supported scalars, so choosing one needs no failure
result. The proposed operations on `TypeView` return `Result` because a requested
combination must be checked against the grammar. Composition retains the
operand's snapshot and builds a consistent reading inside Flat. The resulting
view owns the derived reading; it does not insert another captured declaration
into the immutable snapshot. Invalid grammar combinations return `ModelError`.
Borrow construction supplies a type description; the registry must separately
prove that the generated temporary or source borrow has the required lifetime.
More complex constructors must check that all operand views share a snapshot.

Inspection must preserve opaque declarations as opaque. Current Flat lowers
tuple structs to `Extern` — its kind for a declaration whose
fields it does not model — and does not lower those fields.
Returning a `StructView` for them would therefore need an additional source-model
extension. The first views expose existing facts. Later structural extensions
may represent additional fields, but must not make previously accepted opaque
items fail because an unused field is outside the structural grammar.

Final Rust emission reads retained source facts through Flat's emission
facilities. A view is not a way back to the original syntax: any source fact
needed to render an operation belongs in Flat's structured model, where it can be
checked, rather than being recovered from the text by whoever needs it. Source legality and field accessibility must be represented
or diagnosed before the registry claims support for construction or projection.

### A useful consumer without a registry

A source-inspection tool can list the fields of a function's struct parameter:

```rust
let function = model.function("normalize").ok_or("missing normalize")?;
for parameter in function.parameters() {
    let ty = parameter.ty();
    if let Some(view) = ty.as_struct() {
        for field in view.fields() {
            // Display source facts; this does not select a binding representation.
            println!("{}: {}", field.index(), field.ty().type_ref());
        }
    }
}
```

For `normalize(stamp: Stamp)`, this reaches the `Stamp` fields. For a parameter
of type `&Stamp`, the consumer explicitly calls `referent()` before asking for a
struct. A documentation tool might display that distinction, while the registry
uses it when checking borrowing requirements.

This path is the first acceptance example for the API. It should work with only
`prebindgen-flat`, independently of registry or language-generator crates.

## Elements at this stage

- [Function taking an owned struct][fn_flat]
- [Struct with scalar fields][struct_flat]
- [Type alias declaring an opaque handle][typedef_flat]

[fn_flat]: ../examples/fn/02-flat.md
[struct_flat]: ../examples/struct/02-flat.md
[typedef_flat]: ../examples/typedef/02-flat.md
