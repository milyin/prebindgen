//! Core: language-agnostic primitives for the Registry-based pipeline.
//!
//! Public entry point: [`registry::Registry::from_items`] scans a stream
//! of `(syn::Item, SourceLocation)` into a flat type table;
//! [`registry::Registry::write_rust`]
//! resolves every required type using the configured
//! [`prebindgen::Prebindgen`] back-end and emits the bindings file.
//! Secondary artifacts (C headers, Kotlin sources, …) are produced by the
//! language adapter that implements [`prebindgen::Prebindgen`] — none are
//! built in yet (see the staged unification plan).

pub mod expand;
pub mod gravestone;
pub mod niches;
pub mod prebindgen;
pub mod registry;
pub(crate) mod resolve;
pub mod shape;
pub mod types_util;
pub mod unfold;
pub(crate) mod write;

pub use self::gravestone::{Gravestone, Transmute};
pub use self::niches::{NicheSlot, Niches};
pub use self::prebindgen::{ConverterImpl, Prebindgen, Stage};
pub use self::registry::{Direction, Registry, ScanError, TypeEntry, TypeKey, WriteRustError};
