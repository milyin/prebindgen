# Architecture

prebindgen reads captured `#[prebindgen]` Rust items into `prebindgen_flat::Flat`.
A language generator chooses its binding surface and writes generated Rust plus
the destination-language code that uses it.

This branch prototypes a new `prebindgen-tools` API. C and JNI migration is
deferred: their workspace members, examples, documentation, and integration
jobs are temporarily disabled. The tools API is reviewable in rustdoc and
verified by a small adapter and compiled generated Rust.

## Generated elements

The generator owns elements and their placement. A generated element can wrap
one flat element, group several, or be synthetic. Its kind need not match a
flat element's kind: a function may become a destination constant. Tools does
not prescribe function signatures, mirror layouts, or element writers.

`Place` identifies an occurrence within a generator-assigned element. Its
segments address parameters, fields, callback arguments, intermediate types,
source elements within a group, or generator-specific parts.

## Type relations

The model is:

```text
flat type ⇄ representation type(s) ⇄ wire type(s)
```

Representation types are ordinary Rust types. A `convert!` declaration can
convert `Millis` through `u64`; record helpers construct or decompose fields.
Input and output paths are independent.

Wire types are the generator's closed enum implementing `WireType`. Its
variants include semantic information: the same ABI integer can carry a
number, unsigned bits, or a handle of a particular class. `Wire<W>` pairs
that type with a boundary slot name.

## Resolving one element

A generator implements `ConversionPolicy`, declaring its wire enum, conversion
rule type, and destination metadata type. `Resolver` holds directional defaults.
The generator creates a `Scope` with the element's `Overrides`, then resolves
each typed occurrence:

1. The exact occurrence override, checked against its expected type/direction.
2. An exact normalized type default in that direction.
3. The policy's fallback.

Policy resolves children through the same scope and composes their conversions
using `Input`, `Output`, and `Stage`. `Scope::finish` rejects unused overrides.
Invalid explicit rules return errors; resolution never retries them as defaults.
Conversion cycles and mismatched plan root types/directions are errors.

## Resolved plans

`ConversionPlan<W, M>` retains the selected directional Rust conversion, a
shared `Form<W>` relation tree, and generator-specific destination metadata.
The tree describes direct wires, intermediates, parts, optional values, sums,
and adapter-written sequence loops. Metadata describes the destination value
as a whole, such as its class or constructor; an adapter can retain child
metadata in its own tree. Rust and destination generation consume the same
plan rather than reselect defaults.

`with_input` emits setup, binds the input once, and encloses the flat call.
Backing storage and RAII guards remain alive through that call and unwind
on errors. `emit_output` binds the source expression once and produces zero
wires as `()`, one as a bare value, and several as a tuple. The generator
chooses wire placement, error routing, and ownership transfer.

Root input setup is supported. Flattening conditional child storage, automatic
choice constructors, recursive forward declarations, and non-RAII cleanup are
deferred. The generator allocates names across plans; tools validates duplicate
wire names within each plan. Low-level combinators assert programmer-supplied
record and conversion-stage endpoint compatibility. Arbitrary emitted Rust
semantics and trait implementations still need compilation.

## Verification and documentation

The `prebindgen_tools::resolve` rustdoc module contains a runnable public API
walkthrough. Integration tests compile and run generated Rust for records,
intermediate conversions, optional decomposition, single evaluation, and
borrowed input storage with RAII cleanup on success and failure. These checks
do not establish C/JNI parity; migration and runtime integration remain deferred.
