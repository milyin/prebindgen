# Architecture

prebindgen turns the `#[prebindgen]` items of a flat Rust crate into
language bindings:

```
#[prebindgen] items ──► prebindgen::Source ──► prebindgen_flat::Flat ──► adapter ──► generated Rust (+ C header / Kotlin)
   (source crate)        captured records        the flat model          (prebindgen-c, prebindgen-jni)
                                                                            │
                                                                            └── calls the generators in prebindgen-tools
```

* **The flat model** (`prebindgen-flat`) is what a source crate declared: its
  functions, structs, enums, sums, handles and constants, each type classified
  once (`TypeKind`) and each item spellable back as Rust.
* **An adapter** (`prebindgen-c`, `prebindgen-jni`) makes every decision. Its
  build-script builder collects declarations; the adapter turns them into a
  plan and writes the plan element by element.
* **`prebindgen-tools`** holds the generators the adapter calls — one per kind
  of Rust item — and the helpers its recursion uses. It keeps no state across
  elements: a generator writes one element from what it is given.

## An adapter

Both adapters have the same four parts:

```
builder.rs   the build script's API: declarations, collected as given
plan.rs      the declarations resolved against the flat model
lower        how each type crosses, by recursion over its structure
write        the plan written: each element through its generator, in order
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

**Writing** walks the items and calls a generator for each. The generated Rust
goes into one file; the JNI adapter writes the Kotlin for the same element
into the file of its package. Nothing is emitted on demand, so nothing needs
deduplicating while writing: a type is written where it is declared, and a
Kotlin interface several functions share (a callback, sink, builder, folder
or error handler) is planned once and written once.

## Generators

A generator takes the flat element it wraps and header customizations
through a builder, and asks for the boundary of each value through a callback
trait:

| Generator | Writes | The adapter's callback answers |
|---|---|---|
| `FunctionWriter` | an exported wrapper around a function | the result: the return type, extra out-wires and the return body (`Return`); each parameter: its wires and the wire→value conversion (`Input`); what a failed conversion does |
| `StructWriter` | a mirror struct and both conversions | each field: its wires, both directions |
| `SumWriter` | a mirror enum and both conversions | each field of each alternative, both directions |
| `ClosureWriter` | an `impl Fn(..)` over a foreign callback | each argument: its wires (`Output`) |

A callback gets only what it answers for — a parameter's name and type, a
field, an argument — and returns final wire types. How a `Vec<Payload>`
becomes a pointer and a length (C) or a count and one array per field (JNI)
is the adapter's answer.

## Recursion

A callback answers for a whole type by recursing over its structure.
`shape(ty, lookup)` reads one layer of a type: a scalar, text, a sequence and
how it is held, an `Option`, a `Box`, a borrow, a callback, or a named type
together with the adapter's setting for it and whether it is owned, shared or
exclusive. Each adapter's lowering is one `match` over `Shape` per place — a
parameter, a result, a struct field, a callback argument — recursing into
what the layer holds.

The answers compose:

* `Input` (wires → value) and `Output` (value → wires) carry an expression
  and the wires it reads or produces. A fallible one uses `?` on
  `Result<_, String>`; whoever places it decides where the error goes.
* `record_in` / `record_out` take a struct or a sum alternative apart into
  its fields, with the delimiters the source wrote.
* `Input::optional`, `Input::combine` and `Output::concat` build the rest.
* `Stage::decode` / `Stage::encode` wrap a representation's wires in a
  declared conversion (`convert!`: functions or `From`/`TryFrom` impls).
* `Qualifier` spells a flat type from the generated crate
  (`Payload` → `perftest_flat::Payload`).

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
prebindgen-tools        generators, recursion helpers, output   deps: prebindgen-flat
prebindgen-c            C / cbindgen adapter                    deps: prebindgen-tools
prebindgen-jni          JNI / Kotlin adapter                    deps: prebindgen-tools
prebindgen-c-runtime    called by generated C code              no deps
prebindgen-jni-runtime  called by generated JNI code            deps: jni
```

A source crate depends on `prebindgen` alone; a shipped binding library on a
runtime crate; only a binding crate's `build.rs` depends on an adapter.
