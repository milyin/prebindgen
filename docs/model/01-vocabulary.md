# The vocabulary

Everything in these chapters is stated in these terms. They are given in
dependency order: no entry uses a word that a later one defines.

**Declaration.** What a build script states about the binding before anything
is generated: which functions it exports, which constants, and how each type
crosses. Generation happens for what was declared and for nothing else: a
`#[prebindgen]` item nobody declared produces no output at all, and the
generator reports it as unclaimed. *Declared* is used throughout below in
exactly this sense.

**The registry.** `prebindgen-registry`, the language-agnostic half. It takes
the model and everything the adapter declared, and produces the generated Rust
file. Between the two it works out what the binding needs and in what order,
asks the adapter for each piece, and checks the answers against the model —
collecting every failure rather than stopping at the first.

It decides nothing about how a value crosses. It decides what to ask, in what
order to ask it, and whether the answers hold together. The entries below name
the pieces it asks about; the chapters after this one show them at work.

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

Wire values are the unit these chapters count in, so "two wire values" means two
parameters, not one parameter carrying two things. The two counts do not
correspond: one Rust `String` reaches C as **one** wire value, a `char *`, and
a WebAssembly boundary as **two**, a pointer and a length, because nothing
there can carry both at once. Going the other way, a C struct passed by value
is **one** wire value however many Rust fields went into it.

Only **wire** values pass the boundary; a `Sample` never does. So where this
document says a type *crosses*, it means a Rust value of that type is
**constructed** on the Rust side out of wire values that arrived, or
**deconstructed** into wire values that leave. Those two are the whole of what
happens at the boundary, and every entry below builds on them.

**Direction.** Which of those two happens at a given position: constructing a
Rust value, or deconstructing one. `Direction` is the type; its values are
`Direction::Construct` and `Direction::Deconstruct`.

**Crossing.** One Rust type and one direction: how a value of that type is
constructed at the boundary, or how it is deconstructed. See [directions and
crossings](03-directions-and-crossings.md).

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
