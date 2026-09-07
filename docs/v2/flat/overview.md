# Flat V2: purpose and scope

[V2 project contents](../README.md)

Status: proposed design. API sketches describe intended contracts, not implemented functionality.

Binding-generation companion: [Registry V2](../registry/overview.md).

## Purpose

Flat describes the Rust items captured from annotated source. A consumer needs
to find a function, inspect its parameters, follow their types to declarations,
and inspect fields or enum variants. Those operations are useful to an API
documentation tool, a source validator, a language frontend, or the registry
that later plans the bindings.

Flat V2 should provide that navigation through a consistent, read-only API. A
**view** is an inspectable handle that retains the source model containing its
information. Following a parameter to its type, or a record to a field, keeps
that model association. A view also retains the source information needed for
diagnostics and eventual Rust emission.

Flat supplies these views independently of binding generation. The
[registry](../registry/overview.md#1-what-the-registry-does) adds decisions about which
values to convert and how to combine conversions. For example, Flat reports that
`stamp_from_millis(i64) -> Stamp` takes one integer and returns `Stamp`. The
registry decides whether that function is selected to construct a value for a
binding.

This project covers Flat's supported source language. It does not promise the
type inference, trait resolution, or full Rust semantics of the Rust compiler.

## What changes from the current API

Flat already has name lookup and typed source records. In the existing code,
[`Flat::function`](../../../prebindgen-flat/src/flat/mod.rs) returns `&Function`, and
[`Function`, `Struct`, `Param`, and `Field`](../../../prebindgen-flat/src/flat/element.rs)
contain most of the facts needed for inspection. [`TypeRef`](../../../prebindgen-flat/src/flat/ty.rs)
already classifies source types and preserves wrappers and references.

The new API adds three guarantees around those facts:

1. Public views retain one immutable model and expose read-only records. A
   consumer cannot change a signature or field while retaining unrelated source
   data behind that record.
2. Navigation returns views associated with that same model, including derived
   child types. Consumers do not have to repeat name lookup and attach model
   identity themselves.
3. Lookup, enumeration, source locations, identity checks, and model ownership
   follow the same rules across item kinds.

Function lookup returns `FunctionView` directly. The view supports inspection
and retains the model; Flat keeps the underlying function index private.

---

Previous: [Project contents](../README.md) · Next: [Flat model](model.md)
