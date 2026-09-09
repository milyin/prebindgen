<!-- spec: {"kind": "stage", "stage": "01-source"} -->

# Capture source items

[Project contents](../README.md)

Capture copies each marked declaration out of the crate as source text and says
where it came from. It does not analyse it: what a declaration means — which type
a parameter names, what fields a record has — is worked out one stage later, from
these snippets.

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
it adds is a side effect at compile time: for each marked item it appends one
line to a capture file in the crate's build output directory, the one Cargo gives
every crate as `OUT_DIR`. The line holds what kind of item it is, its name, its
complete source text as written, the `cfg` condition that guarded it if there was
one, and the file and line it came from. Nothing else: no signature broken into
parts, no field list, no resolved type. (The [function][fn_source] and
[record][struct_source] paths show one such line in full.)

The attribute goes on a function, a struct, an enum, a union, a type alias or a
constant, and `#[prebindgen("group")]` writes into a named group instead of the
default one, so a binding crate can consume part of a large surface. Whether a
captured declaration can actually be modelled — a generic function cannot, for
one — is not decided here; the item is captured either way, and the answer comes
with the [source model](02-flat.md).

Two more things travel with the captures, both easy to miss because neither is a
declaration. **Feature guards** are the `#[cfg]` conditions around a captured
item; they are carried so that generated Rust can be gated exactly as the source
was, and so a binding built with different features does not export items its
source crate did not compile. A **guard item** is a compile-time check the
capture emits rather than the user writing it — the assertion that the binding
crate's feature selection matches the source crate's. It has no name in any
foreign API, and it is re-emitted into the generated Rust verbatim.

The source crate re-exports the capture directory as a constant, so a binding
crate's build script can read it without knowing where Cargo put it:

```rust
// build.rs of a binding crate
let items = Source::new(source_crate::PREBINDGEN_OUT_DIR).items_all();
```

Reading is where the stored text becomes syntax again: each line is parsed back
into the item it was, and stamped with the crate it came from, so streams from
several source crates can be read together without losing track of which item
belongs to which. That pair — the item, and where it came from — is what the next
stage receives. It is still Rust syntax at that point, not yet a model of it.

Marking an item is not a statement about bindings. It does not say that `Stamp`
can be represented in C, that `stamp_sum` can be called from Kotlin, or that
either will appear in the generated API. It says only that the declaration is
available for a binding crate to ask about. Which items are exposed, under which
names and with which representation, is settled two stages later, when a
[binding request](03-requests.md) names them. Where the generated code will
*call* them is not recorded here either: the binding crate configures the module
path its source items are reached through, since only it knows the name the
source crate has among its dependencies.

Captures are not the only input to the source model. A binding crate can also
declare a **local helper**: a Rust function that the binding crate itself
provides, that generated code may call, and that no capture describes — for
example a `fn stamp_from_millis(millis: i64) -> Stamp` written in the binding
crate to build a source value that has no public constructor. The frontend
declares the helper's signature, and from the next stage on it is inspected like
any captured function. Helpers are mentioned here because the source model is
built from both inputs, not from the captures alone.

Capture data that cannot be read — a line that does not parse, or a set of files
that contradicts itself — is an error, and it fails the build rather than quietly
shrinking the generated API. That is the only failure this stage has. Everything
else it collects travels onward, including what no target will turn out to
support.

## Elements at this stage

- [Function taking an owned record][fn_source]
- [Record with scalar fields][struct_source]

---

Previous: [Project contents](../README.md) · Next: [Build and inspect the source model](02-flat.md)

[fn_source]: ../examples/fn/01-source.md
[struct_source]: ../examples/struct/01-source.md
