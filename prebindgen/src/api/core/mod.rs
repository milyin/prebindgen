//! Core: the flat model prebindgen's registry pipeline is built over.
//!
//! [`flat::Flat::builder`] parses `(syn::Item, SourceLocation)` records into one
//! flat namespace — the language-agnostic index of everything a `#[prebindgen]`
//! source crate declared. The Registry-based pipeline that resolves a binding
//! over this model — type conversion, boundary expansion, Rust emission — now
//! ships in the separate [`prebindgen-registry`](https://docs.rs/prebindgen-registry)
//! crate, which re-exports this module (`flat`, `shape`, `types_util`) at its own
//! root so a language adapter names one crate for the whole pipeline.
//!
//! Secondary artifacts such as C headers or Kotlin sources are produced by the
//! language adapter after the Rust registry is resolved; see the separate
//! `prebindgen-c` and `prebindgen-jni` crates.

pub mod flat;
/// The `Emit` capability lives with the flat model, because every one of its
/// methods delegates to a model method that is private to `flat` — that
/// pairing is what makes `Emit` the only route to a node's syntax. Re-exported
/// here so `core::emit` keeps naming it.
pub use self::flat::emit;
pub mod shape;
pub mod types_util;

pub use self::flat::{Element, Flat, TypeKey, TypeKeyParseError};
