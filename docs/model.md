# The model a binding is described in

`prebindgen-registry` generates a language binding from one annotated Rust
source. What a binding author writes, and what a language adapter answers, are
both stated in the same small vocabulary. This document is that vocabulary and
the machinery immediately around it.

It describes the model, not the API surface an adapter programs against — the
table, sites, the `Compile` hooks and the error set are documented separately.

## Where the crates sit

Four take part.

- **`prebindgen`** — the `#[prebindgen]` proc macro. It captures each marked
item of the source crate into a data file at build time. The source crate
contains nothing about any foreign language.
- **`prebindgen-flat`** — parses those records into **the model**, `Flat`:
`Struct`, `Variant` (an enum whose alternatives carry payloads, each an
`Alternative`), `Enum`, `Function`, `Field`, and `TypeRef`, the model's
reading of one Rust type. `TypeKey` is the identity that same type is stored
under. Its captured delimiters remain part of the `Alternative`; final Rust
renderers pass the model node to Flat's shape renderer, so `A`, `A()` and
`A {}` remain distinct without copying that syntax fact into another model.
- **`prebindgen-registry`** — the language-agnostic half of generation. It is
what this document describes.
- **an adapter**, one per target language — `prebindgen-c`, whose generator type
is `Cbindgen`, and `prebindgen-jni`, whose generator type is `JniGen`.

A **binding crate** such as `zenoh-flat-c` runs an adapter over a model in its
build script, and compiles what comes out.

## The vocabulary

**Declaration.** What a build script states about the binding before anything
is generated: which functions it exports, which constants, and how each type
crosses. Generation happens for what was declared and for nothing else: a
`#[prebindgen]` item nobody declared produces no output at all, and the
generator reports it as unclaimed.

**The registry.** `prebindgen-registry`, the language-agnostic half. It takes
the model and everything the adapter declared, and produces the generated Rust
file. Between the two it works out what the binding needs and in what order,
asks the adapter for each piece, and checks the answers against the model —
collecting every failure rather than stopping at the first.

It decides nothing about how a value crosses. It decides what to ask, in what
order to ask it, and whether the answers hold together.

**Boundary.** Where the Rust library and the target language meet: the exported
functions the foreign side calls, and what those take and return.

Two kinds of value appear throughout. Both live on the Rust side, and the
difference is what each can do at the boundary.

**Rust value.** What the source crate's own API deals in — a `Sample`, a
`String`, a `u64`, and so on. What a `#[prebindgen]` function takes and
returns.

**Wire value.** One slot in the target language's calling convention: one
parameter, one return value, or one field of a struct the foreign side passes
by value. Its Rust type has an exact counterpart in that language's FFI — `*mut
c_char`, `jlong`, `i32` — so it crosses unchanged, which a `Sample` or a
`String` cannot. A generated converter is an ordinary Rust function whose only
purpose is turning one kind into the other.

Wire values are the unit this document counts in, so "two wire values" means
two parameters, not one parameter carrying two things. The two counts do not
correspond: one Rust `String` reaches C as **one** wire value, a `char *`, and
a WebAssembly boundary as **two**, a pointer and a length, because nothing
there can carry both at once. Going the other way, a C struct passed by value
is **one** wire value however many Rust fields went into it.

Only **wire** values pass the boundary; a `Sample` never does. So where this
document says a type *crosses*, it means a Rust value of that type is
**constructed** on the Rust side out of wire values that arrived, or
**deconstructed** into wire values that leave. Those two are the whole of what
happens at the boundary.

**Direction.** Which of those two happens at a given position: constructing a
Rust value, or deconstructing one. `Direction` is the type; its values are
`Direction::Construct` and `Direction::Deconstruct`.

**Crossing.** One Rust type and one direction: how a value of that type is
constructed at the boundary, or how it is deconstructed. See [directions and
crossings](#directions-and-crossings).

**Callback type.** A Rust type, written `impl Fn(A, B)` in the source and
classified by the model as `TypeKind::Callback` — a function the foreign side
supplies and the Rust side calls later. It crosses like any other type, and its
crossing constructs, because Rust *receives* the callable.

What is unusual is the values inside it. `A` and `B` are ones Rust already
*holds* and pushes out through the call, so their crossings deconstruct — the
one type in the model whose inner values run against the direction of the
crossing that names them.

**Site.** One position in the generated API where a value crosses — for
example, parameter 2 of an exported function, or a return value, or the `Err`
arm of a `Result`, or one argument of a callback. One Rust type usually crosses
at many sites, and it need not cross the same way at every one of them.

**Part.** A **Rust value that another Rust value is made of.** If a source
struct `Sample` holds a `KeyExpr` and a `ZBytes`, those two are its parts:
reached through accessors when `Sample` is deconstructed, and passed to a
constructor when it is built. Concretely a part is always one of a struct
field, a constructor argument, an accessor's result, an optional's payload, a
sequence's element, or a callback's argument.

Every part is itself a crossing, which is what makes the model one level deep:
a Rust value names its parts, and each part is then answered on its own rather
than nested inside the one that names it. A part runs the same direction as the
value that names it, with a callback's arguments the single exception.

**Shape.** How one Rust value is constructed, or deconstructed, stated in terms
of its parts: which `#[prebindgen]` functions assemble it or take it apart, and
whether it has parts at all. `Shape` is the type. It has six variants, and each
falls into one of two cases:

* **It has parts** — `Product` over several values, `Choice` over alternatives,
`Optional` and `Sequence` over one inner value, and `Invoke` over the arguments
of a callback. Each part is a Rust value with a crossing of its own, answered
the same way one level down.
* **It has none** — `Atomic`. Constructing decodes the wire values that arrived;
deconstructing encodes the ones that leave. The adapter writes both conversions
itself, and every chain of parts ends here.

> *For example*, the struct `Sample` above, holding a `KeyExpr` and a `ZBytes`.
> Under `Product`, constructing it calls `sample_new(key, payload)`, and
> deconstructing it reads those two values back out — through accessors such as
> `sample_key_expr` and `sample_payload`, or straight off the struct's fields.
> Under `Atomic`, the adapter converts the whole `Sample` in one piece, and its
> fields never appear at the boundary at all.

A shape never names a wire type. Every part is a Rust type from the model, and
the wire enters only in the second case, where the shape says nothing beyond
"no parts" and the adapter takes over. A `u64` is already FFI-safe, and it is
still the second case — not a shape half in Rust and half on the wire.

`Invoke` is the one variant whose parts run **against** the direction of the
crossing that names them: Rust holds the values it passes out through the call.

A shape carries only what the model cannot supply. `Product` has to name a
constructor or a set of accessors, because a `Sample` may be built and read
several ways and nothing but the declaration says which of them is used here.
`Invoke` names nothing at all: a callback type already carries its argument
types, and their direction follows from the crossing's. So a crossing of a
callback type takes `Invoke`, and the registry accepts no other shape there.

**The table.** What one adapter declares for one binding: every decision it
makes about how values cross. `Recipes` is the type. The adapter fills the
table before any of it is walked; the registry reads it and never adds to it.

**Recipe.** One row of the table — one such decision. Three things make one:

* the **crossing** it answers — a Rust type and a direction;
* a **recipe name**, chosen by the adapter and meaningful within that crossing;
* a **shape**.

Names such as `whole` and `parts` are policy vocabulary and are deliberately
reused under many crossings. `RecipeName` represents that local selector.
`RecipeKey` pairs it with the crossing key and is the globally unique identity
of one table row.

*Why the adapter declares recipes and the model does not.* The model already
knows how a Rust type **can** be taken apart: `Sample`'s fields are in it, and
so are `sample_new`'s parameters and every accessor's signature. What the model
cannot know is which of those splits a given target should use, or whether the
value should skip them and convert straight to the wire — that turns on what
the target language can express, and the two adapters answer differently for
the same type. So the model supplies the possibilities, the adapter declares
which of them are realized, and the registry checks each declaration against
the model rather than inventing one.

*Several recipes for one crossing* is how a type offers a choice. Asking how
`Sample` crosses then has no single answer: one site may take it whole and the
next take it apart, and both are right. The question becomes answerable only
once it names a recipe and a direction.

> *For example*, a Kotlin data class. JniGen declares two recipes on one
> crossing — a `whole` recipe with no parts, crossing as a single JVM object,
> and a `parts` recipe naming the struct's fields, crossing as one wire value
> per field. Each site picks between them.

**Fragment.** What an adapter works out for one **recipe**: which wire values
the crossing occupies, the Rust that converts them, and what has to be released
afterwards. It is the adapter's own type — the registry stores fragments and
passes them around but never looks inside one, asking only what Rust value it
yields. **Not a wire value:** a fragment may occupy none, one, or several.
`prebindgen-c`'s is `CFrag` and `prebindgen-jni`'s is `JFrag`.

**Plan.** What an adapter works out for one **site**: that site's fragment,
plus the exported function's signature, the call into the source crate, and the
cleanup after it. Also the adapter's own type.

Fragment and plan are the pair to keep apart. A fragment is built once per
recipe and reused at every site that recipe serves; a plan is built once per
site.

## How the file is built

A binding's build script constructs its adapter, hands it the model, and asks
for a file. What comes back is Rust source, written into the binding crate and
compiled with it.

An adapter may produce more beside it, and what else is its own decision.
JniGen writes the Kotlin that calls into the generated Rust, so both sides of
that boundary come out of one run. Cbindgen writes only the Rust, leaving the C
header to the separate `cbindgen` crate — the tool it is named after — which
reads the generated Rust afterwards. Neither choice changes how the Rust
file itself is built.

### Who calls whom

Two rounds, with different drivers. The registry drives the first; the writer
drives the second; the adapter answers both and holds everything it built in
between.

**First, resolving.** The registry works out which crossings the binding could
need and in what order — inner ones first, so a recipe's parts are ready before
the recipe that names them — and asks the adapter for a **converter** for each,
in that order.

A converter plan describes the operation that performs the crossing: a
constructing one will take wire values and return a Rust value, while a
deconstructing one will take a Rust value and produce wire values. The adapter
keeps the syntax-free plan and its registry-owned `OperationId`; it does not
choose a Rust function name or materialize the function body during this
round. Sites call the same semantic operation when their conversion contracts
are equal.

What comes back to the registry is not the function. It is one fact about it:
which other crossings that function's body calls into. A converter for
`Option<Sample>` calls the one for `Sample`, so it names `Sample`; a converter
that calls no other names nothing. Those edges are what the registry asked for
— it walks them to work out which crossings the binding actually reaches, and
so which converters have to exist.

**Then writing.** The generated code enters here. The adapter calls the writer,
handing it three things: the resolved registry, itself, and the converter plans
it has been holding since the first round. The plans contain model and adapter
facts, not generated source Rust syntax.

The writer then goes over the declared items in name order and calls the
adapter back once per kind — `on_function`, `on_struct`, `on_enum`, `on_const`
— handing over the model element and taking typed `syn::Item`s. The writer
keeps those items directly; no arbitrary token stream crosses the callback
boundary and no generated source is reparsed there. A callback may return zero
or several items: JniGen's constant hook returns two, a getter and an alias.

**Finally the file.** The writer concatenates, in this order:

1. the adapter's **prerequisites** — helper functions, type aliases, the
`#[repr(C)]` structs a C header reads — emitted first so everything below can
refer to them;
2. the converter plans it was handed. Operations are grouped by
their semantic `OperationId` before rendering, with a reachable representative
preferred when fragments retain separate reachability state. Both JniGen and
Cbindgen converter plans expose that identity, and composed calls retain the
same identity rather than a private Rust name. Only final rendering asks
`Emit` to allocate the identifier used by both definition and calls. The name
keeps model and adapter vocabulary as a readable semantic stem, with a stable
hash only as its collision suffix. Every converter plan and child call must
carry an operation identity; there is no preselected-name fallback;
3. the per-item output, in the order above;
4. the source crate's own feature guards, verbatim.

It then runs one cross-cutting pass over every item — the adapter's
`post_process_item`, which is where bare type references get qualified against
the source module — and writes the result.

### What the foreign side ends up calling

*Wrapper* is this document's word for a callable entry point in that file; the
API has no such type. Which declared items get one differs:

* a declared **function** gets a wrapper that calls it. Usually the function is
a `#[prebindgen]` one from the source crate, but a binding may declare a
function of its own instead, and the wrapper calls that the same way;
* a declared **constant** gets one built over a nullary function the adapter
synthesizes to return it, so the foreign side reads the constant by calling a
getter;
* a declared **struct or enum** gets none. What crosses is a value of that type,
and converting a value is a converter's job; both adapters emit nothing at all
per struct.

A function's wrapper does four things:

1. take in the wire values of each parameter;
2. construct one Rust value per parameter, by calling converters;
3. call the declared function;
4. deconstruct what it returned into wire values, hand those back, and release
whatever the call allocated or borrowed.

Converters are not among the entry points — they are internal, called by
wrappers and by each other. Some entry points are not wrappers either: a drop
per handle type, and whatever the target needs for releasing memory, both come
from the prerequisites and answer to no declared item.

A wrapper's signature is the adapter's to choose, and the two in tree differ:

> **C:** `pub unsafe extern "C" fn calculator_absorb(a: *mut calculator_t, b:
> *const calculator_t, out: *mut f64, e: *mut *mut c_char)` — wire types
> throughout, a `Result` return split into an `out` parameter and a `bool`, and
> an `e` parameter beside it for the error.
>
> **JNI:** also `extern "C"`, but named for the JVM's own lookup
> (`Java_io_zenoh_jni_...`) and led by the `JNIEnv` and `JClass` the JVM passes.
> Nothing is returned for an error: every wrapper takes a trailing handler
> parameter and reports through that.

A **site** is one position in this picture — a parameter, the return, the `Err`
arm, one argument of a callback. That is what makes a fragment per recipe and a
plan per site: the fragment is the converter's answer and is reused wherever
that crossing appears, while the plan is the wrapper's.

## Directions and crossings

### The direction

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

Only the arguments of a callback type swap. Rust receives the callable, so
that crossing constructs, while the values its arguments carry are ones Rust
already holds and pushes out through the call, so those crossings deconstruct.
The registry applies `swap` there, and no declaration states it.

### The crossing

```rust
/// One Rust type and one of the two directions: the question the table answers.
pub struct Crossing {
    ty: TypeRef,
    direction: Direction,
}
```

A word on `TypeRef`, since three of the accessors below return one. It belongs
to `prebindgen-flat`, and nothing here changes it: it is the model's
classification of a Rust type into a closed grammar — `Optional`, `Vec`, `Ref`,
`Callback`, `Named` and the rest — which an adapter matches on rather than
re-parsing syntax for itself. `TypeKey` is that same type reduced to an
identity a map can be keyed by.

Recipe compilation may inspect those model facts and retain a `TypeRef`, but
it has no capability to recover or emit the captured Rust syntax. In
particular, `Cx` carries no `Emit`, converter calls retain registry-owned
`OperationId` values rather than names derived from `TypeKey`, and language
adapters must not parse or classify a key's diagnostic text. Only the final
`write_rust` pass mints `Emit`; at that point resolution and glue planning are
complete, and the renderer may spell retained types and allocate private Rust
symbols while assembling the file. Needing source Rust syntax earlier means a
fact is missing from the Flat model and must be added there.

An operation is shared by its conversion contract, not by the recipe row that
happened to request it. For composed converters that contract includes the
shape, model carrier, ownership mode when deconstructing, direction, and the
adapter-declared intermediate representation. Adapter terminals likewise name
their semantic operation, such as borrowing or consuming an opaque handle.
The final writer turns that identity into a readable private symbol: a bounded
semantic stem followed by a stable hash. The hash disambiguates the name; it
does not replace the model and adapter vocabulary useful to a reader. The
writer also groups plans by this identity before invoking their renderer, so
sharing is decided before either the symbol or function body exists.

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

An adapter decides how to convert from the model facts of `value()`, retains
`spelled()` in its semantic plan, and writes that spelling only when the final
writer supplies `Emit`. `mode()` is the third answer, kept separate because
the table checks it: a constructor taking `Sample` cannot be handed a part
that only yields `Shared`.

Note `Box<Sample>` — `value()` keeps the wrapper because a `Box` is not a
borrow, while `key()` strips it, because `Sample`, `&Sample` and `Box<Sample>`
all reach **one** recipe. That is what makes a recipe declared once serve every
way its type can be written.

### The key

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

`Crossing` is what a site hands the compiler; `CrossingKey` is what groups rows
in the table. The narrowing is deliberate and one-way — `key()` exists, its
inverse does not — because rows are shared by every way their type can be
written. The fragment memo adds the spelled type to `RecipeKey`, since applying
one row to `T`, `&T` and `Box<T>` can require different Rust.

### The recipe's name

```rust
/// An adapter-chosen row name, reusable under different crossing keys.
pub struct RecipeName(String);

impl RecipeName {
    pub fn new(name: impl Into<String>) -> Self;
    /// The name the table gives the recipe it derives for an undeclared crossing.
    pub fn derived() -> Self;
    pub fn as_str(&self) -> &str;
}

/// The globally unique identity of one row in the recipe table.
pub struct RecipeKey {
    crossing: CrossingKey,
    name: RecipeName,
}
```

A crossing is identified by `CrossingKey`; one table row by `RecipeKey`, whose
value is the pair `(CrossingKey, RecipeName)`. The names are the adapter's own —
`prebindgen-c` uses `whole` for how many types cross on their own and `field` /
`parts` / `payload` for contextual alternatives — and the registry never
interprets one. `derived()` is the single reserved name, given to a row the
table derives for a crossing nobody declared.
