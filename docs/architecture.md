# Architecture

prebindgen turns the `#[prebindgen]` items of a flat Rust crate into
language bindings:

```
#[prebindgen] items ──► prebindgen::Source ──► prebindgen_flat::Flat ──► adapter ──► generated Rust (+ C header / Kotlin)
   (source crate)        captured records        the flat model          (prebindgen-c, prebindgen-jni)
                                                                            │
                                                                            └── uses type and conversion utilities in prebindgen-tools
```

* **The flat model** (`prebindgen-flat`) is what a source crate declared: its
  functions, structs, enums, sums, handles and constants, each type classified
  once (`TypeKind`) and each item spellable back as Rust.
* **An adapter** (`prebindgen-c`, `prebindgen-jni`) makes every decision. Its
  build-script builder collects declarations; the adapter turns them into a
  plan and writes the plan element by element.
* **`prebindgen-tools`** provides type inspection, source naming, conversion
  composition and generated-file utilities. Item generation belongs to the
  adapters.

## An adapter

Both adapters have the same four parts:

```
builder.rs   the build script's API: declarations, collected as given
plan.rs      the declarations resolved against the flat model
lower        how each type crosses, by recursion over its structure
write        the plan written: each element in order
```

**The plan** holds two things. The *settings* say how each declared type
crosses — for C an opaque handle, an enum or union mirror, a data or
`repr(C)` struct, an error, or a conversion; for JNI a Kotlin class of some
kind, or a conversion. The *items* are the elements to write, each with every
decision about it already made: its exported name, and for JNI its Kotlin
name, how each parameter crosses and how the result and the error leave.
Every declaration error surfaces while the plan is made.

The items are grouped by kind, each kind in the order it was declared. C
writes the declared types, then the callbacks' closure structs, then the
functions. JNI goes package by package in declaration order, writing each
package's classes (with their members), then its functions, then its
constants; the shared Kotlin interfaces follow all packages.

**Writing** walks the items and emits each using adapter-specific code. The generated Rust
goes into one file; the JNI adapter writes the Kotlin for the same element
into the file of its package. Nothing is emitted on demand, so nothing needs
deduplicating while writing: a type is written where it is declared, and a
Kotlin interface several functions share (a callback, sink, builder, folder
or error handler) is planned once and written once.

## Code generation

Each adapter generates its own wrappers, callback closures and destination
representations. The C adapter builds C ABI signatures and mirror types; the
JNI adapter builds JNI signatures and the corresponding Kotlin surface.
Their code owns parameter evaluation order, source calls, return delivery,
resource handling and failure policy.

`prebindgen-tools` decides and writes how one value crosses, not whole
items: it has no item templates. Adapters place the `Input` and `Output`
expressions of a value's crossing directly into their generated code. This allows each
boundary to choose its own layout and construction strategy.

## Ways, choices, crossings

How a value crosses is decided before any code is written, as a tree with
the shape of the type:

* `Ways` registers every way a type *may* cross, once: whole on one wire
  (as the adapter's own declaration — a handle, an enum), as the fields of
  its struct, as a tag and its sum's alternatives, as the representation a
  `convert!` declares, or — into Rust only — built by a constructor
  function. Each way holds the model item that fixes its parts, so a way
  cannot have parts its item does not have.
* `Choices` says which way each occurrence takes, per direction: a default
  per type, replaced at chosen `Place`s (`send.p`, `send.p.x`). A way id is
  typed by the directions it can serve, so a constructor cannot be chosen
  for a value leaving Rust. A type with a single way needs no choice; a type
  with several and no choice is an error naming it.
* `resolve` walks one occurrence's type, installs the chosen way at each
  node, and asks the adapter (`Lower`) only what the model leaves open:
  which values cross whole and on which wire, how an option tells absence,
  which wires carry a sequence. The result is a `Crossing`, typed by its
  direction (`In` or `Out`), whose nodes only `resolve` can build: a
  `Fields` node has one crossing per field of its struct, an `Optional` one
  of the option's inner type.
* `Crossing::decode` and `Crossing::encode` turn a crossing into Rust
  (`Input`: wires → value; `Output`: value → wires). The adapter writes
  what a single wire holds and a sequence's loop (`Decode`, `Encode`);
  every composition comes from the crossing.

The crossing's wires are the generated element's parameters, and the
foreign-side writer reads the same nodes, so both sides follow one decision.

A source item's `name` is an `ItemName`, which the flat model qualifies when
it is built, so splicing it names the item from the generated crate
(`Payload` → `perftest_flat::Payload`). `callback_arg_types` spells the
parameter types of a callback closure, the one place an adapter writes a
whole source type.

Both adapters still build their values with the hand-composed builders that
preceded crossings, kept as `prebindgen_tools::legacy` until each adapter
resolves crossings instead.

## The two adapters

**C** lowers each value to one C-ABI wire per slot. Opaque handles are boxed
pointers with a typed destructor; enums and sums are `#[repr(C)]` mirrors
validated on the way in (`MaybeUninit`, tag checks); data structs are mirrors
converted field by field; `#[repr(C)]` structs are reinterpreted in place.
Results lower to return values and out-parameters, and `Result` adds a
`char **e` error slot.

**JNI** lowers every value to *leaves* — JNI primitives, strings, primitive
arrays, object arrays — and writes the Kotlin that assembles and takes apart
the objects, so generated Rust never reads a Kotlin field. The same recursion
yields the leaves and all four conversions (Kotlin encode/decode, Rust
decode/encode); a result with several leaves reaches Kotlin through one sink
upcall. Output expansions (`expand_return!`) deliver a value as its fields to
a builder, folder, callback or error handler; input expansions
(`expand_param!`) let a parameter be built by a constructor or passed as a
handle, chosen by a selector.

## Crates

```
prebindgen              the base: Source, SourceLocation, the capture format
prebindgen-proc-macro   #[prebindgen]
prebindgen-flat         the flat model                          deps: prebindgen
prebindgen-tools        type inspection, conversion composition, output   deps: prebindgen-flat
prebindgen-c            C / cbindgen adapter                    deps: prebindgen-tools
prebindgen-jni          JNI / Kotlin adapter                    deps: prebindgen-tools
prebindgen-c-runtime    called by generated C code              no deps
prebindgen-jni-runtime  called by generated JNI code            deps: jni
```

A source crate depends on `prebindgen` alone; a shipped binding library on a
runtime crate; only a binding crate's `build.rs` depends on an adapter.
