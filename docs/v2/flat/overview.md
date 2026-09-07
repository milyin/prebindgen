# Flat V2: purpose and scope

[V2 project contents](../README.md)

Status: proposed design. API sketches describe intended contracts, not implemented functionality.

Binding-generation companion: [Registry V2](../registry/overview.md).

## Purpose

Flat describes the Rust items captured from annotated source. A consumer needs
to find a function, inspect its parameters, follow their types to declarations,
and inspect fields or enum variants. Those operations are useful to a binding
frontend, the registry, an API documentation tool, or a source validator.

Flat V2 should provide that navigation through a consistent, read-only API. A
**view** is an inspectable handle that retains the source model containing its
information. Following a parameter to its type, or a record to a field, keeps
that model association. A view also retains the source information needed for
diagnostics and eventual Rust emission.

The independent utility of these views is checked source inspection. The
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

## Boundary with the registry project

The [registry's relation API](../registry/source-relations.md#4-describing-source-construction-and-decomposition)
consumes `FunctionView` and `RecordView`. The registry assigns conversion roles
and validates them against a requested direction and exact `TypeView`.

| Flat provides | Registry adds |
| --- | --- |
| Function parameters and complete return type | Whether the function constructs a value or projects one from an input. |
| Record shape and typed fields | Recursive field conversions and value construction/decomposition. |
| `Result` child types | Whether a selected constructor treats `Ok` as construction success and routes `Err` as failure. |
| Exact reference/wrapper structure and source access facts | Temporary lifetimes, borrow use and ownership in the generated conversion. |
| Stable snapshot association and normalized type keys | Conversion-cache identity including direction, selected relation and target policy. |
| Source locations and unsupported-item descriptions | Binding-specific dependency paths and skipped-output reports. |

The registry and language frontends use Flat independently. Flat has no
conversion-selection policy, target representation, recursive binding planner,
or dependency on the registry. A `FunctionView` has no `as_constructor()`
method; `ConstructorRelation::new(function)` belongs to the registry library.
