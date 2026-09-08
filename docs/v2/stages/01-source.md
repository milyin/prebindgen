<!-- spec: {"kind": "stage", "stage": "01-source"} -->

# Capture source items

[Project contents](../README.md)

The annotated crate is parsed once, here, into a stream of records describing
what it declares; every later stage works from those records rather than from
Rust source. Two steps further on do parse Rust — a helper signature a binding
crate declares itself, at the end of this chapter, and `cbindgen` reading the
*generated* wrappers to derive the C header — but never the captured source
again.

A **source crate** is an ordinary Rust library that marks the items it wants
available to binding generators:

```rust
use prebindgen::prebindgen;

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

The `#[prebindgen]` attribute leaves each item exactly where it is — the crate
still compiles, and Rust callers still use `Stamp` and `stamp_sum` normally. What
it adds is a side effect at compile time: for each marked item it writes one
record describing that declaration into the crate's build output directory, the
one Cargo gives every crate as `OUT_DIR`. A record holds the declaration as
written, the module path it was written in, the feature guards around it, and the
file and line for diagnostics. (The [function][fn_source] and
[record][struct_source] paths show one such record in full.)

The attribute goes on a function, a struct, an enum or a constant. It is not a
general Rust reader: a declaration whose shape the capture grammar does not
describe — a generic function, say — is recorded as unsupported rather than
lowered, which is the first of the two outcomes described at the end of this
chapter.

Two more things travel with the records, both easy to miss because neither is a
declaration. **Feature guards** are the `#[cfg]` conditions around a captured
item; they are carried so that generated Rust can be gated exactly as the source
was, and so a binding built with different features does not export items its
source crate did not compile. A **guard item** is a compile-time check the
capture emits rather than the user writing it — the assertion that the binding
crate's feature selection matches the source crate's. It has no name in any
foreign API, and it is re-emitted into the generated Rust verbatim.

The source crate then re-exports that directory as a constant, so a binding
crate's build script can read the capture without knowing where Cargo put it:

```rust
// build.rs of a binding crate
let items = Source::new(source_crate::PREBINDGEN_OUT_DIR).items_all();
```

Marking an item is not a statement about bindings. It does not say that `Stamp`
can be represented in C, that `stamp_sum` can be called from Kotlin, or that
either will appear in the generated API. It says only that the declaration is
available for a binding crate to ask about. Which items are exposed, under which
names and with which representation, is settled two stages later, when a
[binding request](03-requests.md) names them.

Captures are not the only input to the source model. A binding crate can also
declare a **local helper**: a Rust function that the binding crate itself
provides, that generated code may call, and that no capture describes — for
example a `fn stamp_from_millis(millis: i64) -> Stamp` written in the binding
crate to build a source value that has no public constructor. The frontend
declares the helper's signature, and from the next stage on it is inspected like
any captured function. Helpers are mentioned here because the source model is
built from both inputs, not from the captures alone.

Two outcomes have to stay distinct. A construct the capture grammar does not
model — a signature shape it cannot describe — is kept as an explicit
*unsupported record*, so the reason survives all the way to the report at the end
of the pipeline, and so an item that some other target could still export is not
silently lost. Malformed capture data is different: a record that cannot be
parsed, or a set that contradicts itself, is an error that fails the build rather
than quietly shrinking the generated API.

## Elements at this stage

- [Function taking an owned record][fn_source]
- [Record with scalar fields][struct_source]

---

Previous: [Project contents](../README.md) · Next: [Build and inspect the source model](02-flat.md)

[fn_source]: ../examples/fn/01-source.md
[struct_source]: ../examples/struct/01-source.md
