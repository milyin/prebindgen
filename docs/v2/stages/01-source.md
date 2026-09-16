<!-- spec: {"kind": "stage", "stage": "01-source"} -->

[Project contents](../README.md) · Next: [Build and inspect the source model](02-flat.md)

# Capture source items

The generator first needs to learn which Rust items it may expose. It does not
scan your whole project and guess. You mark selected items with `#[prebindgen]`,
and the annotation records them when Rust compiles the source crate. This step is
called **capture**. Its output is a set of saved Rust snippets that a binding
crate can read later in its own build script.

Capture records syntax and its origin. The next stage interprets that syntax:
for example, it connects the name `Stamp` in a function parameter to the struct
that declares `secs` and `nanos`.

A **source crate** is an ordinary Rust library that marks the items it wants
available to binding generators; each marked item is a **source item**, the thing
every later stage reads its source facts from. Most of what a binding declares
names one; a binding may also declare things the source never exported — a
callback signature, a helper of its own, a type such as `String` — which the
[request chapter](03-requests.md) covers. The chapters use this crate
throughout:

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

The attribute keeps the item available to ordinary Rust callers and additionally
writes a capture record at compile time. A function's implementation body is
removed from the captured copy: generation needs its signature and calls the
original implementation. Type declarations retain their fields in the captured
text. The capture files live in the source
crate's build output directory, exposed through `OUT_DIR`. Each line is a JSON
record, so the file format is called JSONL (JSON Lines).

The record identifies the item and preserves its Rust text, conditional-build
information and source location. It does not yet contain a resolved field list
or a link from a parameter to its type declaration. The [function][fn_source]
and [record][struct_source] pages show the shape of these records; the saved
token text need not preserve the original whitespace.

The attribute goes on a function, a struct, an enum, a union, a type alias or a
constant, and `#[prebindgen("group")]` writes into a named group instead of the
default one, so a binding crate can consume part of a large surface. Whether a
captured declaration can actually be modelled — a generic function cannot, for
one — is not decided here; the item is captured either way, and the answer comes
with the [source model](02-flat.md).

Consider a function enabled only by `#[cfg(feature = "extra")]`. A binding
must not call that function when the source library was built without `extra`.
There are two related checks to keep generation and compilation consistent.

The first is filtering parsed `cfg` attributes when capture is read. Conditions
can come from the captured item text or the macro's explicit `cfg` argument.
A condition known to be false removes the item; a condition known to be true
is removed after its decision has been applied. Conditions the reader cannot
evaluate remain on the item. An ordinary `#[cfg]` processed by Rust before
`#[prebindgen]` may prevent capture altogether. The feature assertion below
is intended to check that generation's feature decisions agree with the linked
source crate. V1 emits that check; V2 currently omits it, as explained below.

The second is a generated **feature assertion**. Cargo can compile the source
crate twice with different feature sets —
once as a build-dependency of the binding crate, where the capture is filtered,
and once as an ordinary dependency, which is what the generated code is finally
linked against. If those two disagree, the generated code was filtered against
one build and compiled against another. So reading the capture prepends one item
to the stream: a `const _` assertion comparing the source crate's own `FEATURES`
constant with the feature list the capture was filtered by, which fails
compilation with an explanatory message when they differ. It has no name in any
foreign API. The V1 writer carries it into generated Rust unchanged. Current
V2 keeps the guard in Flat but does not emit it, so a V2 binding does not yet
get this protection against mismatched source features. The
[implementation limitations](../implementation.md#what-it-does-not-settle)
track this gap. `v2check` bypasses capture and therefore does not test it.

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
names and with which [representation](04-values.md#plan-value-conversions),
is settled two stages later, when a [binding request](03-requests.md) names
them. Capture metadata also records the source crate name, which supplies the
default path for generated source calls. The C frontend can override that path
with `.source_module(...)` for a different dependency name or module arrangement.
The current JNI V2 route instead uses the first source module's crate name.

Captures are not the only input to the source model. A binding crate can also
declare a **local helper**: a Rust function that the binding crate itself
provides, that generated code may call, and that no capture describes — for
example a `fn stamp_from_millis(millis: i64) -> Stamp` written in the binding
crate to build a source value that has no public constructor. The frontend
declares the helper's signature, and from the next stage on it is inspected like
any captured function. Helpers are mentioned here because the source model is
built from both inputs, not from the captures alone.

Unreadable capture data is a build error: for example, malformed JSON/Rust text,
an invalid capture-directory layout, or an incompatible description file. This
is different from a well-formed item that a target cannot support. Such items
continue to the source-model and planning stages, where their limitations can
be diagnosed in context.

## Elements at this stage

- [Function taking an owned record][fn_source]
- [Record with scalar fields][struct_source]

[fn_source]: ../examples/fn/01-source.md
[struct_source]: ../examples/struct/01-source.md
