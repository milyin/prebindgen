<!-- spec: {"kind": "root"} -->

# V2: from a Rust API to C and Kotlin bindings

Suppose you have a Rust library and want people to call it from C and Kotlin.
Your Rust function may take a struct by value, but a Kotlin caller has a JVM
object. Someone must read that object's properties, construct the Rust struct,
call your function, and return the answer. C needs a different entry point, even
though both callers ultimately use the same Rust implementation.

prebindgen generates that connecting code. You mark the Rust items that the
generator may inspect, then configure which items each language should expose.
This guide explains how the second generation engine, called **V2**, turns those
inputs into bindings. It assumes familiarity with functions, types, build scripts
and foreign function interfaces, but no knowledge of prebindgen.

There are two ways to read the guide:

**Follow the stages** to learn the architecture. The eight chapters start with
collecting annotated Rust items and end with writing generated files. Each stage
explains why it is needed, what information it receives, and what it produces.

**Follow one example** to see the stages work together. The appendix traces a
function and the struct it accepts from Rust source to generated C and Kotlin.
Each page shows the input and result of one stage. These examples are also
specification cases: the generator's behavior should agree with them.

V2 is an opt-in engine with limited implemented coverage. The default engine,
**V1**, lives in `prebindgen-registry` and has its own
[architecture documentation](../model.md). Use that document to understand a
build that has not selected V2.

These pages describe both the implemented V2 subset and the contract for its
extensions. A design description is not a promise of current support. Where a chapter
describes something the engine does not do yet, it says so at that point; the
contracts designed for those extensions are gathered on
[the extensions page](extensions.md), and
[the implementation page](implementation.md#what-it-does-not-settle) keeps the
whole list. Tracked as [issue #720](https://github.com/milyin/prebindgen/issues/720).
The engine switch and the unsupported-output reporting exist, and so does the
first increment of the pipeline itself: the two element paths specified here —
a function taking an owned record, and that record — are planned, assembled and
emitted by `prebindgen-registry-v2`, for both targets, through the real
frontends. A binding crate's `build.rs` is the same under either engine;
`PREBINDGEN_PIPELINE=v2` makes `prebindgen-c` and `prebindgen-jni` hand their
declarations to this engine instead of v1's, and every declaration the engine
cannot lower yet comes back as a reported skip — the arrangement that lets a V1
configuration run through V2 during the transition, and the one that
[ends with it](stages/07-retain.md#unsupported-requests-and-public-api-dependencies),
a request V2 cannot generate being a build failure in the finished engine. `examples/v2check` compiles the
result for the source crate below.
[The implementation page](implementation.md#the-first-increment-as-built)
records what building them settled and what is still only described. Elsewhere,
Rust API sketches illustrate intended contracts rather than published APIs.

## What is being generated

The input is a Rust crate that marks the items it wants exposed. The chapters
illustrate the pipeline with one small example of that:

```rust
#[prebindgen]
pub struct Stamp {
    pub secs: i64,
    pub nanos: i64,
}

#[prebindgen]
pub fn stamp_sum(stamp: Stamp) -> i64 {
    stamp.secs.wrapping_add(stamp.nanos)
}
```

From it, and from what a binding crate configures, the C output is a header a C
program compiles against, backed by a generated Rust function that C actually
calls:

```c
typedef struct Stamp { int64_t secs; int64_t nanos; } Stamp;
int64_t stamp_sum(struct Stamp stamp);
```

and the Kotlin/JNI output is a Kotlin class and function, backed by a different
generated Rust function that the JVM calls through a native method on the
binding's harness object:

```kotlin
public data class Stamp(val secs: Long, val nanos: Long)
public fun stampSum(stamp: Stamp): Long = JNINative.stampSum(stamp)
internal object JNINative {
    @JvmSynthetic
    external fun stampSum(stamp: Stamp): Long
}
```

The example explicitly chooses `Stamp` as the C type name; the C frontend's
default type base name would be `stamp`. The function keeps `stamp_sum`.
For Kotlin, the configuration also chooses a package, and the frontend derives
the camel-case function name `stampSum`. `JNINative` is the generated object
that declares native methods: `external` tells the JVM that Rust supplies the
implementation. The public Kotlin function delegates to that method.

Neither generated Rust function is written by hand, and neither is a
transliteration of the other: the C one receives a struct by value and reads its
members, the JNI one receives a JVM object and calls its property getters through
the Java Native Interface. What they share is everything about the *source* side
— two field values, one `Stamp`, one call — and sharing that work is what this
architecture is for.

## The components

Two crates in this picture are yours; the rest are what prebindgen is.

**What you write.** The **source crate** is an ordinary Rust library — the
example above is one — that marks the items it wants exposed and otherwise knows
nothing about C or Kotlin. (Such crates are conventionally named `…-flat`, after
the flat namespace of items a binding sees, which is why the model built from one
is called Flat.) Beside it stands one **binding crate per target language**: a
`cdylib` or `staticlib` whose build script configures the generator, whose
`lib.rs` includes the Rust that generator produced, and which is the thing you
ship — a native library plus a C header, or a native library plus Kotlin sources
in a JAR.

Separating the crates lets the source library remain useful to ordinary Rust
callers while each binding crate chooses its own exported interface and library
format. `cdylib` produces a native dynamic library; `staticlib` produces a native
static library. Rust does allow an implementation crate to define foreign entry
points itself, but keeping the generated entry points in binding crates avoids
making that implementation responsible for every target language.

**The generator**, which runs inside the binding crate's build script and is a
build-dependency only:

- **`prebindgen-proc-macro`** provides the `#[prebindgen]` attribute, and
  **`prebindgen`** reads back what it captured — one record per marked item.
- **`prebindgen-flat`** builds the queryable source model over those captured
  items:
  which functions exist, what a parameter's type is, which declaration a type
  name refers to, what fields a record has.
- **`prebindgen-registry-v2`** is the engine this document specifies. It plans
  every [conversion](stages/04-select.md#select-conversion-relations) a requested binding needs, resolves what depends on what,
  renders the Rust [wrappers](stages/06-boundary.md#assemble-the-native-boundary), and reports what it could not generate. It shares
  the capture and model crates with the V1 engine and depends on nothing else of
  it, so "V2 never falls back to V1 for an item" is a property of the dependency
  graph rather than a promise.
- **`prebindgen-c`** and **`prebindgen-jni`** are the **language adapters**, one
  crate per **target** — a language together with its native calling interface,
  C or Kotlin through JNI. Each adapter answers to two callers, which is why the
  chapters address it under two names. Facing you, it is the **frontend**: the
  builder your build script configures with what to expose and how it should look
  in that language. Facing the engine, it is the **target adapter**: the
  implementation the engine queries while planning — what carries a `Stamp`, how
  a member of it is read, what this exported function's native signature is. One
  crate, two directions; the first records decisions, the second is made to spell
  them out item by item. Producing the target language's own declarations is part
  of the same job — the JNI adapter emits the Kotlin, since only it knows what a
  Kotlin class should look like. The C adapter is the exception, and only because
  a well-established tool already does that work: `cbindgen` derives C headers
  from Rust source, so there is nothing for a C emitter to add.
- **`cbindgen`** is external, and only a C binding crate runs it — over the
  generated Rust, to produce the header.

**The runtime**, which is linked into what you ship rather than into the build:
**`prebindgen-c-runtime`** (traits for opaque values passed by value across the C
ABI) and **`prebindgen-jni-runtime`** (string and array encoding, the binding
error type, cached JVM method ids). Generated code calls these at run time, so a
binding crate depends on one of them the ordinary way. None of the generator
ships inside a binding.

The crate boundaries separate build-time tools from runtime support. For example,
the source crate needs annotation support but does not need the JNI generator.
A shipped native library needs the runtime helpers its generated code calls,
but does not need the parser and formatter used to generate that code. A C-only
binding can also avoid depending on the Kotlin adapter.

## The pipeline

```text
captured Rust source + declared local helper signatures
  -> Flat builds the checked source model
  -> language frontend records binding requests from Flat views and user choices
  -> registry plans value conversions using Flat facts and target descriptions
  -> registry assembles the native boundary of each exported function
  -> registry retains complete supported plans and reports skipped requests
       -> common Rust writer -> native Rust -> cbindgen -> C headers
       -> JNI's Kotlin writer -> Kotlin declarations
```

The chapters run in that order, and their vocabulary does not: most of the
nouns name what flows *between* stages, so a chapter uses a word before the
chapter that specifies it. [The vocabulary](concepts.md) introduces each of
those words once; read it first.

1. [Capture source items](stages/01-source.md)
2. [Build and inspect the source model](stages/02-flat.md)
3. [Record binding requests](stages/03-requests.md)
4. [Select conversion relations](stages/04-select.md)
5. [Represent and compose values](stages/05-represent.md)
6. [Assemble the native boundary](stages/06-boundary.md)
7. [Retain supported output](stages/07-retain.md)
8. [Emit bindings](stages/08-emit.md)

Then: [implementation sequence and acceptance](implementation.md), which is not a
pipeline stage but the plan for building one, and
[the extension contracts](extensions.md), the design for what the stages do not
do yet, kept apart so that each chapter describes only what its cells show. Beside the generated code the
engine produces [a report](report.md) of what became of each declaration.
The build script can write it beside the code with `write_report`. It is a
diagnostic that nothing in the pipeline reads, described on its own page.

This order is the order of information dependencies, not a requirement to make
eight passes over the project. The registry interleaves selection, child planning
and [representation](stages/05-represent.md#represent-and-compose-values) inside one recursive walk — the two
chapters that describe it are the descent and the ascent of that walk; binding choices and local helper
signatures can be recorded before model construction, and the request stage turns
those choices into requests once the Flat model exists. The owned view types
described in the source-model chapter are a future extension.

## The appendix: elements and their paths

All the paths are specified against [one source crate](source.md), which grows as
paths are added.

- [Function taking an owned record][fn] — `stamp_sum(Stamp) -> i64`
- [Record with scalar fields][struct] — `Stamp { secs: i64, nanos: i64 }`

An **element kind** is a kind of [source item](stages/01-source.md#capture-source-items) — a function, a record, an enum —
as this appendix organizes it. (Inside the pipeline the word is not used: the
chapters say *source item* for what the model captured and *declaration* for
what the binding asked to expose, identified by a `DeclarationId`, such as
`fn:stamp_sum`. Current ids do not distinguish two foreign placements of the
same source item.) Element kinds form a tree: a kind at the root, more specific
variants below it, each becoming its own path when its behavior differs from its
parent's. Two paths are specified today; the rest name the id they will use when
they are written.

```text
fn                      specified   function taking an owned record, returning a scalar
  fn_fallible           deferred    function returning Result
  fn_callback           deferred    function taking a callback
  fn_borrowed_param     deferred    function taking a borrowed parameter
  fn_complex_return     deferred    function returning a non-scalar value
struct                  specified   record with scalar fields
  struct_nested         deferred    record with a record field
  struct_option_field   deferred    record with an optional field
  struct_vec_field      deferred    record with a sequence field
enum                    deferred    enum with payload variants
const                   deferred    exported constant
```

Deferring a path is a statement about this appendix, not about the architecture:
the chapters already specify optional values, sequences, variants, callbacks and
error routing in general terms. A deferred path is missing its worked example, and
writing one may well expose a gap in the general contract — that is what the
appendix is for.

The same ids are the document's link vocabulary. `fn` is that element's path;
`fn_select` is that element at the selection stage; `fn_select_c` is the C
variant of that cell. A chapter links to the elements that reach it, an element
cell links back to its chapter, and both use these ids.
[The format contract](FORMAT.md) states the grammar and the rules a new path has
to satisfy.

Only applicable combinations exist. The record path has no native-boundary cell,
because a record exports no function of its own; its conversion is reached
through the function that uses it. A missing cell claims nothing about support —
whether something is supported is stated in the contract of a cell that does exist.

## What V2 has to deliver

Both engines consume the existing examples' complete source inputs and recorded
frontend choices. V2 emits its supported subset and reports the missing capability
for every skipped request; it does not fall back to V1 for individual items.
Emitted bindings preserve logical behavior, ownership, error handling and declared
interfaces. Byte-identical generated text is not required.

The frontend crate's `v2` Cargo feature — on `prebindgen-c` or `prebindgen-jni`,
whichever the binding crate depends on — makes the engine available. `.build()` selects the pipeline
from `PREBINDGEN_PIPELINE=v1|v2`, defaulting to V1 when unset; an explicit
`build_with(Pipeline::...)` overrides that. Selecting V2 without the feature is an
error. [Issue #719](https://github.com/milyin/prebindgen/issues/719) covers that
switch, whose scaffold already exists.

## Keeping the document consistent

[The format contract](FORMAT.md) defines page identities, link ids and the rules
for adding a path or a stage. [manifest.json](manifest.json) lists the stages,
element paths, languages and applicable cells. [The validator](validate.py) checks
the manifest and the documents against each other:

```sh
python3 docs/v2/validate.py
python3 -m unittest discover -s docs/v2 -p 'test_*.py'
```

Both use only Python's standard library. They check structure — identities, ids,
links, anchors, indexes, backlinks, required sections, and that each word of
[the vocabulary](concepts.md) is defined once and linked at its first mention on
every other page — and cannot check whether the prose describes a correct
compiler.

[fn]: examples/fn/README.md
[struct]: examples/struct/README.md
