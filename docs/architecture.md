# Architecture

prebindgen turns the `#[prebindgen]` items of a flat Rust crate into
language bindings. Three layers do it:

```
#[prebindgen] items ──► prebindgen::Source ──► prebindgen_flat::Flat ──► adapter ──► generated Rust (+ C header / Kotlin)
   (source crate)        captured records        the flat model          (prebindgen-c, prebindgen-jni)
                                                                            │
                                                                            └── writes through prebindgen-tools
```

* **The flat model** (`prebindgen-flat`) is what a source crate declared: its
  functions, structs, enums, sums, handles and constants, each type classified
  once (`TypeKind`) and each item spellable back as Rust.
* **The adapter** (`prebindgen-c`, `prebindgen-jni`) is the only place
  decisions are made. It walks the items its build script declared and
  decides, per position, how each value crosses: which wires, which
  conversions, what the foreign side sees.
* **`prebindgen-tools`** writes what the adapter decided. It holds no
  decisions of its own.

## Writers and callbacks

Each kind of generated Rust item has a writer. A writer takes the flat element
it wraps, a new name and header customizations through a builder, and asks the
adapter for every decision through a callback trait:

| Writer | Element | The adapter answers |
|---|---|---|
| `FunctionWriter` | a function | per parameter: its wires and the wire→value conversion (`Input`); for the result: the return type, extra out-wires and the return body (`Return`); what a failed conversion does |
| `StructWriter` | a struct, as a mirror type | per field: its wires, both directions |
| `SumWriter` | a data-carrying enum, as a mirror enum | per field of each alternative, both directions |
| `ClosureWriter` | an `impl Fn(..)` parameter | per argument: its wires (`Output`) |

The writers expect final wire types. How a `Vec<Payload>` becomes a pointer and
a length (C) or a count and one array per field (JNI) is the adapter's answer,
given in its callback.

## Recursion

Conversions compose, so an adapter lowers nested types by recursion over
`TypeKind` and combines the parts:

* `Input` (wires → value) and `Output` (value → wires) carry an expression and
  the wires it reads or produces. A fallible one uses `?` on
  `Result<_, String>`; whoever places it decides where the error goes.
* `record_in` / `record_out` decompose a struct or a sum alternative into its
  fields, with the delimiters the source wrote.
* `Input::optional`, `Input::combine`, `Output::concat` build the rest.
* `RustFile::claim` / `once` emit a named helper — one converter per type —
  at most once, so recursive types refer to their helpers by name.
* `Qualifier` spells a flat type from the generated crate
  (`Payload` → `perftest_flat::Payload`).
* `convert!` resolves a declared conversion (functions or `From`/`TryFrom`
  impls) into stages an adapter applies around a representation type.

## The two adapters

**C** lowers each declared type to a C-ABI wire: opaque handles are boxed
pointers with a typed destructor; enums and sums are `#[repr(C)]` mirrors
validated on the way in (`MaybeUninit`, tag checks); data structs are mirrors
converted field by field; `#[repr(C)]` structs are reinterpreted in place.
Results lower to out-parameters, and `Result` adds a `char **e` error slot.

**JNI** lowers every value to *leaves* — JNI primitives, strings, primitive
arrays, object arrays — and generates the Kotlin that assembles and takes apart
the objects, so generated Rust never reads a Kotlin field. One codec per type
yields the leaves and all four conversions (Kotlin encode/decode, Rust
decode/encode); a result with several leaves reaches Kotlin through one sink
upcall. Output expansions (`expand_return!`) deliver a value as its fields to a
builder, folder, callback or error handler; input expansions (`expand_param!`)
let a parameter be built by a constructor or passed as a handle, chosen by a
selector.

## Crates

```
prebindgen              the base: Source, SourceLocation, the capture format
prebindgen-proc-macro   #[prebindgen]
prebindgen-flat         the flat model                          deps: prebindgen
prebindgen-tools        writers, composition, output            deps: prebindgen-flat
prebindgen-c            C / cbindgen adapter                    deps: prebindgen-tools
prebindgen-jni          JNI / Kotlin adapter                    deps: prebindgen-tools
prebindgen-c-runtime    called by generated C code              no deps
prebindgen-jni-runtime  called by generated JNI code            deps: jni
```

A source crate depends on `prebindgen` alone; a shipped binding library on a
runtime crate; only a binding crate's `build.rs` depends on an adapter.
