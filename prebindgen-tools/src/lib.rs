//! # prebindgen-tools
//!
//! Building blocks for a prebindgen **language adapter**.
//!
//! An adapter reads the flat model of what `#[prebindgen]` captured
//! ([`flat`], re-exported from `prebindgen-flat`), turns its build script's
//! declarations into a plan of the elements to bind, and then writes each
//! element: the Rust wrapper through a generator here, and whatever the
//! destination language needs beside it (Kotlin text, for the JNI adapter)
//! itself. This crate holds no plan and no state across elements: every
//! generator writes one element from what it is given.
//!
//! ## Generators
//!
//! One generator per kind of Rust item. Each takes the flat element it
//! wraps and header customizations through a builder, and asks for the
//! boundary of each value through a small callback trait:
//!
//! | Generator | Writes | Callbacks |
//! |---|---|---|
//! | [`FunctionWriter`] | an exported wrapper around a [`Function`](flat::flat::Function) | [`FunctionCallbacks`]: the result's return type, out-wires and body; each parameter's wires and wire→value [`Input`]; the failure fragment |
//! | [`StructWriter`] | a mirror struct of a [`Struct`](flat::flat::Struct), with both conversions | [`FieldCallbacks`]: each field's wires, both directions |
//! | [`SumWriter`] | a mirror enum of a [`Variant`](flat::flat::Variant), with both conversions | [`FieldCallbacks`] |
//! | [`ClosureWriter`] | an `impl Fn(..)` closure over a foreign callback | [`ClosureCallbacks`]: each argument's wires ([`Output`]) |
//!
//! The generators expect **final** wire types: how a `Vec<Payload>` becomes
//! a pointer and a length, or a count and a column per field, is the
//! adapter's answer in its callback.
//!
//! ## Recursion
//!
//! A callback answers for a whole type by recursing over its structure.
//! [`shape()`] reads one layer — an `Option`, a borrow, a sequence, a named
//! type with the adapter's own setting for it — so every adapter's recursion
//! is one `match` over [`Shape`]. The answers compose: [`Input`] and
//! [`Output`] carry an expression plus the wires it reads or produces;
//! [`record_in`] / [`record_out`] take a struct or a sum alternative apart
//! into its fields; [`Input::optional`], [`Input::combine`] and
//! [`Output::concat`] build the rest; [`Stage::decode`] / [`Stage::encode`]
//! wrap a representation in a declared conversion. A conversion that can
//! fail uses `?` on `Result<_, String>`, and whoever places it decides where
//! the error goes.
//!
//! ## Everything else
//!
//! * [`Qualifier`] — names a flat item from generated code
//!   (`Payload` → `perftest_flat::Payload`).
//! * [`names`] — case conversion and identifier helpers.
//! * [`mod@convert`] — the `convert!` vocabulary: a source type that crosses as
//!   another type through functions or `From`/`TryFrom` impls.
//! * [`RustFile`] — the generated file: items in order, formatted, written
//!   only when changed.

pub mod closure;
pub mod convert;
pub mod file;
pub mod function;
pub mod names;
pub mod qualify;
pub mod record;
pub mod shape;
pub mod wire;

pub use prebindgen_flat as flat;
#[doc(hidden)]
pub use syn as __syn;

pub use crate::{
    closure::{ClosureCallbacks, ClosureWriter},
    convert::{Conversion, FnRef, ResolvedConversion, Stage, Via},
    file::RustFile,
    function::{FunctionCallbacks, FunctionWriter, Return},
    qualify::Qualifier,
    record::{
        record_in, record_out, FieldCallbacks, Record, StructMirror, StructWriter, SumMirror,
        SumWriter,
    },
    shape::{shape, Access, Holding, Shape},
    wire::{Input, Output, Wire},
};

/// Refuse a model the frontend could not read in full: every
/// [`Unsupported`](flat::flat::Element::Unsupported) element, reported at
/// once. An adapter calls this before looking at any declaration, so a
/// binding is built against the whole source or not at all.
pub fn check_supported(flat: &flat::Flat) -> Result<(), String> {
    let bad: Vec<String> = flat.unsupported().map(|u| u.error.to_string()).collect();
    if bad.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "the source declares items the flat model cannot express:\n  {}",
            bad.join("\n  ")
        ))
    }
}

#[cfg(test)]
mod tests;
