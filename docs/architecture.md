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

`prebindgen-tools` provides conversion expressions and composition utilities,
not item templates or writer callback traits. Adapters place `Input` and
`Output` expressions directly into their generated code. This allows each
boundary to choose its own layout and construction strategy.

## Recursion

An adapter builds the `Input` or `Output` of a whole type by recursing over
its structure.
`shape(ty, lookup)` reads one layer of a type: a scalar, text, a sequence and
how it is held, an `Option`, a `Box`, a borrow, a callback, or a named type
together with the adapter's setting for it and whether it is owned, shared or
exclusive. Each adapter's builder is one `match` over `Shape` per place — a
parameter, a result, a struct field, a callback argument — recursing into
what the layer holds.

The answers compose:

* `Input` (wires → value) and `Output` (value → wires) carry a conversion
  and a `Form`: the tree recording how each layer of the value crosses, whose
  leaves are the wires, typed by the adapter's own `WireType` enum. An input's
  expression reads its wires; an output converts whatever value the template
  hands it (`Output::apply`, `Output::bind`). A fallible conversion uses `?`
  on `Result<_, String>`; whoever places it decides where the error goes.
* `Input::wire`, `via`, `record`, `optional`, `sum`, `seq` and `parts` (and
  their `Output` counterparts) build one layer each. `via` wraps the
  representation's `Input` or `Output` in a declared conversion (`convert!`:
  functions or `From`/`TryFrom` impls), resolved once into a
  `ResolvedConversion`.
* `Place` names an occurrence of a type inside a generated element;
  `Overrides` gives a type's default decision, replaced at chosen places.
* A source item's `name` is an `ItemName`, which the flat model qualifies
  when it is built, so splicing it names the item from the generated crate
  (`Payload` → `perftest_flat::Payload`). `callback_arg_types` spells the
  parameter types of a callback closure, the one place an adapter writes a
  whole source type.

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
