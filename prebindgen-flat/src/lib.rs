//! # prebindgen-flat
//!
//! The flat model prebindgen's registry pipeline is built over.
//!
//! [`flat::Flat::builder`] parses `(syn::Item, `[`SourceLocation`](prebindgen::SourceLocation)`)`
//! records — read through [`prebindgen::Source`] — into one flat
//! namespace — the language-agnostic index of everything a `#[prebindgen]`
//! source crate declared. The Registry-based pipeline that resolves a binding
//! over this model — type conversion, boundary expansion, Rust emission —
//! ships in the separate
//! [`prebindgen-registry`](https://docs.rs/prebindgen-registry) crate, which
//! re-exports this crate's modules (`flat`, `shape`, `types_util`) at its own
//! root so a language adapter names one crate for the whole pipeline.
//!
//! Secondary artifacts such as C headers or Kotlin sources are produced by the
//! language adapter after the Rust registry is resolved; see the separate
//! `prebindgen-c` and `prebindgen-jni` crates.

/// The capability to render captured Rust syntax — handed to an adapter's
/// emission callbacks and nowhere else, so code that decides cannot reach
/// what code that emits must spell. An out-of-crate adapter implements
/// `Prebindgen` (in the separate `prebindgen-registry` crate) and
/// therefore has to name this type.
pub use crate::flat::emit::Emit;
pub mod flat;
pub mod shape;
pub mod types_util;

pub use self::flat::{Element, Flat, TypeKey, TypeKeyParseError};
