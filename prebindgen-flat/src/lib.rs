//! # prebindgen-flat
//!
//! The flat model a prebindgen language adapter reads.
//!
//! [`flat::Flat::builder`] parses `(syn::Item, [`SourceLocation`](prebindgen::SourceLocation))`
//! records — read through [`prebindgen::Source`] — into one flat
//! namespace: the language-agnostic index of everything a `#[prebindgen]`
//! source crate declared. A language adapter (`prebindgen-c`,
//! `prebindgen-jni`) walks the elements it was asked to bind and writes the
//! binding through `prebindgen-tools`.

/// The capability to render captured Rust syntax, for code whose job is
/// producing Rust.
pub use crate::flat::emit::Emit;
pub mod flat;
pub mod shape;
pub mod types_util;

pub use self::flat::{Element, Flat, TypeKey, TypeKeyParseError};
