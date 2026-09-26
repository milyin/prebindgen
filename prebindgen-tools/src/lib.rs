//! # prebindgen-tools
//!
//! Building blocks for a prebindgen **language adapter**.
//!
//! An adapter reads the flat model of what `#[prebindgen]` captured
//! ([`flat`], re-exported from `prebindgen-flat`) and writes everything a
//! binding needs: the generated Rust file of exported wrappers, and whatever
//! the destination language needs beside it (a C header comes from
//! cbindgen; Kotlin sources come from the adapter itself). There is no
//! pipeline between the model and the adapter: the adapter walks the
//! elements it was asked to bind and decides, element by element, how each
//! one crosses. This crate makes the writing part short.
//!
//! ## Wrapper writers
//!
//! One writer per kind of generated item. Each takes the flat element it
//! wraps, an optional new name and header customizations through a builder,
//! and asks the adapter for every decision through a small callback trait:
//!
//! | Writer | Wraps | Callbacks |
//! |---|---|---|
//! | [`FunctionWriter`] | a [`Function`](flat::flat::Function) | [`FunctionCallbacks`]: parameter → wires + wire→source [`Input`]; return type → wire type, out-wires and the return body; the failure fragment |
//! | [`StructWriter`] | a [`Struct`](flat::flat::Struct), as a mirror type | [`FieldCallbacks`]: field → wires, both directions |
//! | [`SumWriter`] | a [`Variant`](flat::flat::Variant), as a mirror enum | [`FieldCallbacks`] |
//! | [`ClosureWriter`] | an `impl Fn(..)` parameter | [`ClosureCallbacks`]: argument → wires ([`Output`]) |
//!
//! The writers expect **final** wire types: how a `Vec<Payload>` becomes a
//! pointer and a length, or a `jobject`, is the adapter's decision, made in
//! its callback.
//!
//! ## Recursion
//!
//! Conversions compose. [`Input`] and [`Output`] carry an expression plus the
//! wires it reads or produces, and combine: [`record_in`] / [`record_out`]
//! decompose a struct or a sum alternative into its fields,
//! [`Input::optional`] adds a presence test, [`Input::map`] wraps a value.
//! A conversion that can fail uses `?` on `Result<_, String>` and is placed by
//! whoever consumes it. [`RustFile::once`] emits a named helper at most once,
//! so a recursive lowering can turn every type into one converter function
//! and refer to it by name.
//!
//! ## Everything else
//!
//! * [`Qualifier`] — names a flat item from generated code
//!   (`Payload` → `perftest_flat::Payload`).
//! * [`names`] — case conversion and identifier helpers.
//! * [`convert`] — the `convert!` vocabulary: a source type that crosses as
//!   another type through functions or `From`/`TryFrom` impls.

pub mod closure;
pub mod convert;
pub mod file;
pub mod function;
pub mod names;
pub mod qualify;
pub mod record;
pub mod wire;

pub use prebindgen_flat as flat;

pub use crate::{
    closure::{ClosureCallbacks, ClosureWriter},
    convert::{Conversion, FnRef, ResolvedConversion, Stage, Via},
    file::RustFile,
    function::{FunctionCallbacks, FunctionWriter, Return},
    qualify::Qualifier,
    record::{record_in, record_out, FieldCallbacks, Record, StructMirror, StructWriter, SumMirror, SumWriter},
    wire::{Input, Output, Wire},
};

#[doc(hidden)]
pub use syn as __syn;
