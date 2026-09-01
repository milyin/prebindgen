//! # prebindgen-flat
//!
//! The independent flat parser and model that collectors build on.
//!
//! [`flat::Flat::builder`] parses `(syn::Item, [`SourceLocation`](prebindgen::SourceLocation))`
//! records — read through [`prebindgen::Source`] — into one flat
//! namespace — the language-agnostic index of everything a `#[prebindgen]`
//! source crate declared. The Registry-based pipeline that resolves a binding
//! over this model — type conversion, boundary expansion, Rust emission —
//! ships in the separate
//! [`prebindgen-registry`](https://docs.rs/prebindgen-registry) crate, which
//! re-exports this crate's model modules at its own
//! root so a language adapter names one crate for the whole pipeline.
//!
//! Secondary artifacts such as C headers or Kotlin sources are produced by the
//! language adapter after the Rust registry is resolved; see the separate
//! `prebindgen-c` and `prebindgen-jni` crates.

/// The rendering protocol for a collector-owned callback key.
///
/// The flat layer supplies the operations because it owns captured syntax; each
/// collector decides which concrete key implements them and where that key is
/// handed out. `prebindgen-registry` supplies its own private receiver behind
/// the unconstructable `RustWriter` it hands to final callbacks.
pub use crate::flat::emit::RustEmitter;
pub mod flat;
pub mod shape;
pub mod types_util;

pub use self::flat::{Element, Flat, TypeKey, TypeKeyParseError};
