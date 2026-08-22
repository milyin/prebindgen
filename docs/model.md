# The model a binding is described in

`prebindgen-registry` generates a language binding from one annotated Rust
source. What a binding author writes, and what a language adapter answers, are
both stated in the same small vocabulary. This document is that vocabulary and
the machinery immediately around it.

It describes the model, not the API surface an adapter programs against — the
table, sites, the `Compile` hooks and the error set are documented separately.

## Flat is the planning boundary

**Invariant.** Code generation must not have direct access to the actual Rust
type captured behind a `TypeRef` until final Rust emission: every declaration
has been resolved, every fragment and site plan is fixed, validation has
finished, and the only work left is assembling the generated file. If planning
needs a fact that it can obtain only by inspecting or rendering that Rust type,
the Flat model is incomplete. The fix belongs in Flat, where the fact must be
represented losslessly and tested, not in the generator as a syntax escape.

Before final emission, code may carry a `TypeRef` opaquely and use its model
answers — `TypeKind`, `TypeKey`, crossing mode, structural children, declared
fields and functions, and source location. It must not:

* receive an `Emit` or another `RustEmitter` capability;
* turn a source-side `TypeRef` into `syn::Type`, tokens, or text;
* reparse or pattern-match the captured Rust spelling to make a planning
  decision; or
* generate a converter or wrapper body merely to discover what it depends on.

Adapter-authored **wire** types are a separate matter. A C adapter is allowed to
state `*mut c_void`, and a JNI adapter `jlong`, as Rust syntax because those are
the adapter's output vocabulary, not source syntax hidden behind `TypeRef`.
Source-side positions remain `TypeRef`s in plans until the final renderer spells
them.

The current implementation enforces opacity at the `TypeRef` API but violates
the timing part of this invariant by handing `Emit` to conversion planning. The
[Rust-writing proposal](rust-writing.md) audits that path and proposes how to
close the gap.

## Where the crates sit

Four take part.

- **`prebindgen`** — the `#[prebindgen]` proc macro. It captures each marked
item of the source crate into a data file at build time. The source crate
contains nothing about any foreign language.
- **`prebindgen-flat`** — parses those records into **the model**, `Flat`:
`Struct`, `Variant` (an enum whose alternatives carry payloads, each an
`Alternative`), `Enum`, `Function`, `Field`, and `TypeRef`, the model's reading
of one Rust type. `TypeKey` is the identity that same type is stored under.
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

Only **wire** values pass the boundary; a `Sample` never does. A Rust value
reaches the boundary through a recursive walk of shapes. A shape with parts
relates the value to those parts, whose crossings continue the walk. An
`Atomic` shape ends the walk and hands that value to the adapter's wire
conversion.

### Parts and wire use different verbs

These are two different relations and use two different pairs of words:

| Relation | Toward the Rust value | Toward the boundary |
|---|---|---|
| parts ↔ Rust value | **construct** the value from its parts | **deconstruct** the value into its parts |
| wire values ↔ an `Atomic` terminal | **decode** the terminal from wire values | **encode** the terminal into wire values |

Here *terminal* means only that the selected shape is `Atomic`. It need not be a
scalar: an adapter may treat a whole `Sample` as one terminal.

`Direction::Construct` names the whole walk toward Rust: decode at its atomic
leaves, then construct enclosing values from their parts. `Direction::Deconstruct`
names the reverse walk toward the boundary: deconstruct values into parts, then
encode the atomic leaves. Construct/deconstruct and decode/encode are therefore
not synonyms.

**Direction.** Which way that recursive shape walk runs. `Direction` is the
type; its values are `Direction::Construct` and
`Direction::Deconstruct`.

**Crossing.** One Rust type and one direction: the question used to look up a
row in the recipe table. A crossing does not say whether its next step is parts
or wire values; the row's shape says that. See [directions and
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
* **It has none** — `Atomic`. In the construct direction the adapter decodes the
wire values that arrived; in the deconstruct direction it encodes the ones that
leave. The shape itself names no wire type, and every chain of parts ends here.

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

**Recipe.** One named row owned by the table — one such decision. Three things
identify and define the row:

* the **crossing key** under which the table files it — a normalized Rust type
and a direction;
* a **recipe name**, chosen by the adapter and meaningful within that crossing
key;
* a **shape**.

Names such as `whole` and `parts` are policy vocabulary and are deliberately
reused under many crossings. `RecipeName` represents that local selector.
`RecipeKey` pairs it with the crossing key and identifies the row position
throughout the table, whether or not a row is present there.

A recipe states the value–parts step at this layer, or `Atomic` to end the shape
walk. It never states a wire type or wire layout. Those appear only when the
adapter compiles the table row into a fragment.

*Why the adapter declares recipes and the model does not.* The model already
knows how a Rust type **can** be taken apart: `Sample`'s fields are in it, and
so are `sample_new`'s parameters and every accessor's signature. What the model
cannot know is which of those splits a given target should use, or whether the
shape walk should stop at the whole value — that turns on what the target
language can express, and the two adapters answer differently for the same
type. So the model supplies the possibilities, the adapter declares which of
them are realized, and the registry checks each declaration against the model
rather than inventing one.

*Several rows under one crossing key* is how the table offers a choice. Asking
for the `Sample` row by crossing key alone may be ambiguous: one site may select
`whole` and the next `parts`, and both are valid. A binding declaration selects
a `RecipeName`; when it names none, the table must have or derive a default.
Resolution promotes that local name to a `RecipeKey`, which is what the binding,
compiler and diagnostics carry from then on.

> *For example*, a Kotlin data class. Under the same crossing key JniGen
> declares a `whole` row with shape `Atomic` and a `parts` row with shape
> `Product` over the struct's fields. The compiled `JFrag` for `whole` may use
> one JVM object; the `parts` fragment may use one wire value per field. Those
> wire layouts belong to the fragments, not to the recipes. Each site selects
> one row.

**Fragment.** What an adapter compiles from one **recipe row applied to one
spelled crossing**, plus its child fragments. At a shape with parts it composes
construction or deconstruction with those children; at `Atomic` it supplies
decoding or encoding. The result records which wire values the whole walk
occupies, the Rust that converts them, and what has to be released afterwards.
It is the adapter's own type — the registry stores fragments and passes them
around but never looks inside one, asking only what Rust value it yields. **Not
a wire value:** a fragment may occupy none, one, or several. `prebindgen-c`'s is
`CFrag` and `prebindgen-jni`'s is `JFrag`.

One table row serves spellings that normalize to the same crossing key, but its
fragments may differ: `Sample`, `&Sample` and `Box<Sample>` share the row while
moving, borrowing and rebuilding the wrapper require different generated Rust.

**Plan.** What an adapter works out for one **site**: that site's fragment,
plus the exported function's signature, the call into the source crate, and the
cleanup after it. Also the adapter's own type.

Fragment and plan are the pair to keep apart. A fragment is memoized per spelled
crossing and selected recipe row, then reused at matching sites; a plan is built
once per site.

## How the file is built today

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

The adapter recursively compiles each reached recipe row into a fragment. At a
shape with parts, generated Rust combines child-fragment conversions with the
row's construction or deconstruction operation. At `Atomic`, the adapter writes
the decoder or encoder. The completed fragment therefore reaches wire values,
although the recipe row itself never names them. The adapter renders that code
now and keeps it. C stores one complete `syn::ItemFn` in each fragment. JNI
stores a main function plus zero or more pre-stage functions, while a
composed-only fragment emits no converter function of its own. A fragment is
memoized per spelled crossing and selected row, but may contribute zero, one,
or several functions to the file.

What comes back to the registry is not the function. It is one fact about it:
which other crossings that function's body calls into. A converter for
`Option<Sample>` calls the one for `Sample`, so it names `Sample`; a converter
that calls no other names nothing. Those edges are what the registry asked for
— it walks them to work out which crossings the binding actually reaches, and
so which converters have to exist.

**Then writing.** The rest of the generated code enters here. The adapter calls
the writer, handing it three things: the resolved registry, itself, and the
already-rendered converters it has been holding since the first round. So the
registry never carries the generated Rust at all — it goes straight from the
adapter to the writer.

The writer then goes over the declared items in name order and calls the
adapter back once per kind — `on_function`, `on_struct`, `on_enum`, `on_const`
— handing over the model element and taking a `TokenStream`. It parses that as
Rust items and keeps them all. Nothing else is checked: not that a function
comes out, not that it is one item, not that it mentions the item declared.
JniGen's constant hook returns two, a getter and an alias.

**Finally the file.** The writer concatenates, in this order:

1. the adapter's **prerequisites** — helper functions, type aliases, the
`#[repr(C)]` structs a C header reads — emitted first so everything below can
refer to them;
2. the converters it was handed, sorted by name and deduplicated, so one
function per name reaches the file however many crossings produced it;
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
2. decode atomic leaves and construct one Rust value per parameter, by calling
converters;
3. call the declared function;
4. deconstruct what it returned, encode its atomic leaves into wire values, hand
those back, and release whatever the call allocated or borrowed.

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

The implementation audit and proposal for moving all Rust syntax work after
planning live in [Rust writing after planning](rust-writing.md).

## Directions and crossings

### The direction

```rust
/// Which of the two directions a crossing is, as a value.
pub enum Direction {
    /// Walk toward this crossing's Rust value: decode an atomic terminal, or
    /// construct a value from its parts.
    Construct,
    /// Walk toward the boundary: deconstruct a value into its parts, or encode
    /// an atomic terminal.
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
/// One Rust type and one of the two directions: a query into the recipe table.
pub struct Crossing {
    ty: TypeRef,
    direction: Direction,
}
```

`Crossing` is a query, not a conversion. Its recipe-table row determines the
next shape step; only a compiled fragment reaches wire values.

A word on `TypeRef`, since three of the accessors below return one. It belongs
to `prebindgen-flat`, and nothing here changes it: it is the model's
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
all query the same table rows. That is what makes rows declared once serve every
way their type can be written.

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
    /// The name the table gives the row it derives for an undeclared crossing key.
    pub fn derived() -> Self;
    pub fn as_str(&self) -> &str;
}

/// The globally unique position of one row in the recipe table, whether or not
/// the table currently holds a row there.
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
