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
[generation proposal below](#proposed-generation-mechanism) closes that gap.

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

**Recipe.** One entry of the table — one such decision. Three things make one:

* the **crossing** it answers — a Rust type and a direction;
* a **name**, chosen by the adapter, since a crossing may have several recipes;
* a **shape**.

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

A converter is the Rust code that performs the crossing: a constructing one
takes wire values and returns a Rust value, a deconstructing one takes a Rust
value and produces wire values. The adapter renders that code now and keeps it.
C stores one complete `syn::ItemFn` in each fragment. JNI stores a main function
plus zero or more pre-stage functions, while a composed-only fragment emits no
converter function of its own. There is one fragment per recipe, reused by
every site that picks it, but a fragment may therefore contribute zero, one, or
several functions to the file.

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

## Proposed generation mechanism

The two-round account above describes the output accurately, but it hides the
most important implementation detail: resolving is already emitting Rust. That
is not required by the model and is the source of much of the orchestration
around generation.

### What the implementation actually does

The present call path is:

1. `RegistryBuilder::convert_with` derives the crossing order, constructs an
   `Emit`, and passes it into the adapter's conversion closure.
2. `recipe::Compiler` owns another `Emit` and exposes it through every
   `Compile` hook's context. C and JNI compile hooks use it to spell source-side
   `TypeRef`s while recipes and sites are still being resolved.
3. A compiled fragment holds generated syntax, including complete
   `syn::ItemFn`s where that recipe emits conversion functions. Both adapters
   keep the fragment memo behind
   `Rc<RefCell<Compiled<_>>>`, repeatedly clone it into `Compiler::resume`, and
   put the finished compiler back because later conversions consult earlier
   ones while they are being generated.
4. Each adapter copies the functions out again into `compiled_fns`. Its
   `write_rust` passes that separate slice beside the resolved registry to the
   shared writer.
5. The writer constructs another `Emit`, appends the already-generated
   converters, invokes the per-item emission hooks, parses each hook's
   `TokenStream` back into `syn::Item`s, appends guards, runs
   `post_process_item` over the whole AST, and writes the file.

The capability is therefore private but not late: it prevents an ordinary
adapter method from spelling a `TypeRef`, yet explicitly allows the adapter to
do so inside `convert_with` and `Compiler`. The comments in
`registry/declare.rs` call conversion an emission callback for exactly this
reason. This is the point where the implementation differs from the Flat
boundary stated at the start of this document.

The extra state is a consequence of that timing rather than of crossings. The
registry needs a conversion's semantic dependencies — the `Answer::over(...)`
edges — but it never reads the generated function body to derive them. Recipe
composition already knows those edges. Keeping a generated `ItemFn` alive
during the dependency walk therefore couples planning and rendering without
providing information to the registry.

### Separate planning from rendering

Generation should have one semantic phase and one Rust-syntax phase:

```text
captured records
    -> Flat
    -> declarations, recipes and bindings
    -> ordered crossings
    -> adapter-specific fragment and site plans
    -> validate and freeze the complete generation plan
    -> render Rust once, with Emit
    -> assemble, format and write the file
```

Everything through the frozen plan is **planning**. It may inspect Flat and the
resolved conversion table, and it may carry `TypeRef`s, but it cannot render
their captured syntax. The last two steps are **final emission**. They may spell
those stored `TypeRef`s, but cannot select a recipe, change a crossing, discover
a dependency, fall back to another converter, or reject a shape. Those
decisions are already frozen.

The plan types remain adapter-specific. There is no benefit in forcing C and
JNI into one code-generation IR. Each adapter's immutable store needs to carry,
at least:

* one fragment plan per reached recipe, including its semantic dependencies,
  wire slots, ownership and cleanup operations, stable generated symbol, and
  source-side positions as opaque `TypeRef`s;
* one artifact plan per declared function, type, constant, callback and other
  generated entry point, with the exact fragment selected for every site;
* the complete ordered artifact set, including prerequisites and target-side
  artifacts, so Rust and Kotlin/C output read the same frozen decisions; and
* typed planning and validation errors, all produced before a writable output
  path is touched.

The final Rust renderer receives only one of those item-specific plans and the
late spelling capability. It does not receive `&Registry`, a raw source
signature from which it could re-derive a crossing, or a global store from which
it could select a different plan. Adapter-authored wire syntax can be held in a
plan directly; each source-side Rust type is rendered from its `TypeRef` only at
this point.

Conceptually, the ownership looks like this; the names are illustrative rather
than a proposed public API:

```rust,ignore
struct Generation<P> {
    plans: P,                  // complete, validated, immutable
    artifacts: Vec<Artifact>, // already selected and ordered
}

trait RenderRust {
    type Plans;

    fn render_artifact(
        &self,
        plan: &ArtifactPlan<Self::Plans>,
        emit: &Emit,
    ) -> Result<Vec<syn::Item>, RenderError>;
}

fn write_rust<P, R: RenderRust<Plans = P>>(
    generation: &Generation<P>,
    renderer: &R,
    destination: impl AsRef<Path>,
) -> Result<PathBuf, WriteError>;
```

The **shared Rust writer** drives this last function. It mints `Emit`, walks the
already-ordered artifact envelopes, asks the adapter's renderer to turn each
item-specific plan into `syn::Item`s, appends model guards, formats the complete
file, and writes it. C and JNI may expose convenience `write_rust` methods, but
those delegate to this one pipeline; they do not reimplement assembly, error
handling, formatting, or destination handling.

The adapter owns the semantic contents of `Generation` and the small
plan-to-items renderer. The registry is a planning input, not an input to that
renderer. Calling the renderer is one explicit final-emission boundary rather
than the current protocol of item-kind callbacks interleaved with registry
access. The concrete API may instead use an item sink which owns the private
capability. The invariant is the same: `Emit` is minted only inside final file
assembly, and every adapter call that can reach it is a renderer over a frozen,
item-specific plan.

Returning `syn::Item`s (or pushing them into that sink) removes the current
`TokenStream -> syn::File -> syn::Item` parse round trip. Source qualification
should happen when the final renderer spells each stored `TypeRef`; once no
generated body exists before that point, `post_process_item` should either
disappear or become a syntax-only normalizer with no registry or planning
access. Ordering and deduplication likewise belong to stable artifact IDs in the
plan, not to generated function names inspected after rendering.

### What this deletes, and what it keeps

Moving the boundary deletes accidental coordination:

* `Emit` from `RegistryBuilder::convert_with`, `recipe::Compiler`, and
  `Compile` contexts;
* generated `syn::ItemFn`s from fragment planning and the duplicate
  `compiled_fns` caches;
* the `Rc<RefCell<Compiled<_>>>` clone/resume/finish exchange used to let
  generation observe its own partial output;
* lazy or later compiler resumes used to build site plans after conversion
  resolution; and
* writer callbacks that receive a registry, token parsing, and name-based
  converter deduplication.

It does **not** delete the Flat model, crossing dependency order, the distinction
between reusable fragments and per-site plans, adapter-specific wire layouts,
typed failures, or validation. Those are model complexity. In particular,
recursive crossings still require the registry's explicit cycle rule, and a
callback's reversed inner direction remains a Flat fact rather than renderer
logic.

### Migration

This can land as byte-identical stages:

1. Define frozen adapter-owned generation stores and eagerly build every
   fragment and site plan. Make both artifact writers consume the same store.
2. Replace generated converter functions inside C fragments with semantic
   operations and render them in C's final writer. This is the smaller adapter
   and establishes the boundary before JNI migration.
3. Do the same for JNI while deleting, rather than reproducing, the legacy
   `ConverterImpl`, `Stage`, `expand`, `unfold`, and `fn_plan` carriers tracked
   by [#506](https://github.com/milyin/prebindgen/issues/506).
4. Remove `Emit` from `convert_with` and `recipe::Compiler`; make it impossible
   to construct or receive a spelling capability anywhere in the planning call
   graph.
5. Reduce the shared writer to the single final-assembly driver above, replace
   the item-kind token callbacks with one plan-rendering boundary, and remove
   the `Prebindgen` emission protocol and `post_process_item`.

The ordering allows temporary semantic plans to coexist with old rendered
fragments, but the invariant is complete only after step 4. Each step keeps
generated Rust and Kotlin/C artifacts byte-for-byte unchanged; the boundary is
an internal simplification, not a binding ABI change.

This supplies the missing mechanism behind
[#195](https://github.com/milyin/prebindgen/issues/195)'s pure-emission rule:
renderers consume frozen plans only, while this section additionally says when
source Rust spelling becomes available. It also completes the emission-out
direction left by [#251](https://github.com/milyin/prebindgen/issues/251),
without making the legacy-plan deletion in #506 the new architecture.

The exit checks are mechanical:

* no function reachable from planning or validation receives `Emit`, implements
  `RustEmitter`, or renders a source-side `TypeRef`;
* no final renderer receives `&Registry`, rebuilds a site/fragment plan, or
  classifies a source signature;
* every source-side type in a plan remains a `TypeRef` until final rendering;
* validation and every artifact writer observe the same immutable plan store;
* all spelling sites are reachable only from final file assembly; and
* regeneration, workspace tests, clippy/rustdoc, and the JNI JVM covertest are
  unchanged and green.

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

`Crossing` is what a site hands the compiler; `CrossingKey` is what the table
and the fragment memo are keyed by. The narrowing is deliberate and one-way —
`key()` exists, its inverse does not — because a key names a recipe and a
recipe is shared by every way its type can be written.

### The recipe's name

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
