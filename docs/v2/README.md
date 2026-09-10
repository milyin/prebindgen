<!-- spec: {"kind": "root"} -->

# V2: the binding pipeline, stage by stage and element by element

This is the specification for the second generation pipeline. It is written to be
read two ways.

**Down the pipeline.** Seven chapters, one per stage, from the captured Rust
source to the emitted C and Kotlin bindings. Each chapter explains what arrives,
who owns the decisions, what leaves, and what happens when a request cannot be
served. Read in order, they are the developer documentation for the design.

**Along one element.** The appendix takes one kind of source element — a
function, a record — and follows it through every stage that applies to it,
showing its exact representation at each step: the captured item, the Flat views,
the recorded request, the conversion plans and target operation descriptions, the
native boundary, the retained output, and the generated code. Read that way, the
document is an example-based specification: the general contract in a chapter and
its concrete application to a fixed input are two views of the same requirement.

Status: proposed architecture for [issue #720](https://github.com/milyin/prebindgen/issues/720).
The engine switch and the unsupported-output reporting exist, and so does the
first increment of the pipeline itself: the two element paths specified here —
a function taking an owned record, and that record — are planned, assembled and
emitted by `prebindgen-registry-v2`, for both targets, and `examples/v2check`
compiles the result. [The implementation page](implementation.md#the-first-increment-as-built)
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
int64_t stamp_sum(struct Stamp arg0);
```

and the Kotlin/JNI output is a Kotlin class and method, backed by a different
generated Rust function that the JVM calls:

```kotlin
data class Stamp(val secs: Long, val nanos: Long)

fun stampSum(stamp: Stamp, onError: JniErrorHandler<Long>): Long
```

The C declarations carry the source names, because a foreign name defaults to the
name the source used; Kotlin's cannot, since a Kotlin declaration also needs a
package and a class to live in, which the binding crate supplies.

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

They are two crates rather than one because a `#[no_mangle] extern "C"` function
can only be exported from a `cdylib` or a `staticlib`. Keep the FFI layer in the
library that implements the functionality and nothing can export it; move it into
the binding crate and you write it again for every language you bind. Generating
it into each binding crate, from one annotated source, is what this project is
for.

**The generator**, which runs inside the binding crate's build script and is a
build-dependency only:

- **`prebindgen-proc-macro`** provides the `#[prebindgen]` attribute, and
  **`prebindgen`** reads back what it captured — one record per marked item.
- **`prebindgen-flat`** builds the queryable source model over those captured
  items:
  which functions exist, what a parameter's type is, which declaration a type
  name refers to, what fields a record has.
- **`prebindgen-registry-v2`** is the engine this document specifies. It plans
  every conversion a requested binding needs, resolves what depends on what,
  renders the Rust wrappers, and reports what it could not generate. It shares
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

1. [Capture source items](stages/01-source.md)
2. [Build and inspect the source model](stages/02-flat.md)
3. [Record binding requests](stages/03-requests.md)
4. [Plan value conversions](stages/04-values.md)
5. [Assemble the native boundary](stages/05-boundary.md)
6. [Retain supported output](stages/06-retain.md)
7. [Emit bindings](stages/07-emit.md)

Then: [implementation sequence and acceptance](implementation.md), which is not a
pipeline stage but the plan for building one.

This order is the order of information dependencies, not a requirement to make
seven passes over the project. The registry interleaves selection, child planning
and representation inside one recursive walk; binding choices and local helper
signatures can be recorded before Flat is published, and the request stage turns
those choices into requests once the Flat views exist.

## The appendix: elements and their paths

All the paths are specified against [one source crate](source.md), which grows as
paths are added.

- [Function taking an owned record][fn] — `stamp_sum(Stamp) -> i64`
- [Record with scalar fields][struct] — `Stamp { secs: i64, nanos: i64 }`

An **element kind** is a kind of source declaration — a function, a record, an
enum — as this appendix organizes it. (Inside the pipeline, the chapters use
`ElementId` for something narrower: one *requested output*, such as `stamp_sum`
exposed at one Kotlin placement. Same adjective, different noun; the chapters say
which they mean.) Element kinds form a tree: a kind at the root, more specific
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
`fn_values` is that element at the value-planning stage; `fn_values_c` is the C
variant of that cell. A chapter links to the elements that reach it, an element
cell links back to its chapter, and both use these ids.
[The format contract](FORMAT.md) states the grammar and the rules a new path has
to satisfy.

Only applicable combinations exist. The record path has no native-boundary cell,
because a record exports no function of its own; its conversion is reached
through the function that uses it. A missing cell claims nothing about support —
support outcomes belong in the contract of a cell that does exist.

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
links, anchors, indexes, backlinks, required sections — and cannot check whether
the prose describes a correct compiler.

[fn]: examples/fn/README.md
[struct]: examples/struct/README.md
