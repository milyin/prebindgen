# How the file is built

A binding's build script constructs its adapter, hands it the model, and asks
for a file. What comes back is Rust source, written into the binding crate and
compiled with it.

An adapter may produce more beside it, and what else is its own decision.
JniGen writes the Kotlin that calls into the generated Rust, so both sides of
that boundary come out of one run. Cbindgen writes only the Rust, leaving the C
header to the separate `cbindgen` crate — the tool it is named after — which
reads the generated Rust afterwards. Neither choice changes anything below.

## Who calls whom

Two rounds, with different drivers. The registry drives the first; the writer
drives the second; the adapter answers both and holds everything it built in
between.

**First, resolving.** The registry works out which crossings the binding could
need and in what order — inner ones first, so a recipe's parts are ready before
the recipe that names them — and asks the adapter for a **converter** for each,
in that order.

A converter is the Rust function that performs the crossing: a constructing one
takes wire values and returns a Rust value, a deconstructing one takes a Rust
value and produces wire values. The adapter builds each as a complete
`syn::ItemFn` — which is why a converter is always exactly one function — and
keeps it. There is one per recipe, so every site that picks that recipe calls
the same one.

What comes back to the registry is not the function. It is one fact about it:
which other crossings that function's body calls into. A converter for
`Option<Sample>` calls the one for `Sample`, so it names `Sample`; a converter
that calls no other names nothing. Those edges are what the registry asked for
— it walks them to work out which crossings the binding actually reaches, and
so which converters have to exist.

**Then writing.** The generated code enters here. The adapter calls the writer,
handing it three things: the resolved registry, itself, and the converters it
has been holding since the first round. So the registry never carries the
generated Rust at all — it goes straight from the adapter to the writer.

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

## What the foreign side ends up calling

*Wrapper* is this document's word for a callable entry point in that file; the API
has no such type. Which declared items get one differs:

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
