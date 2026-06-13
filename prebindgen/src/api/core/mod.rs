//! Core: language-agnostic primitives for the Registry-based pipeline.
//!
//! The new API pipeline is intentionally phase-oriented:
//!
//! 1. [`registry::Registry::from_items`] indexes `(syn::Item, SourceLocation)`
//!    records into one flat namespace. This phase is index-only: it does not
//!    inspect function signatures or mark any type required.
//! 2. [`registry::Registry::scan_declared`] asks the configured
//!    [`prebindgen::Prebindgen`] adapter which functions and types it claims,
//!    then scans only those items into input/output type requirements.
//! 3. Adapter-provided constructor and deconstructor declarations are resolved
//!    into expansion/unfold plans and register their leaf requirements.
//! 4. The fixed-point resolver asks the adapter for input/output converters
//!    until no unresolved type advances, then propagates `ConverterImpl::subs`
//!    from required roots.
//! 5. [`registry::Registry::write_rust`] emits adapter prerequisites,
//!    converters, per-item wrapper Rust, and passthrough items.
//!
//! Secondary artifacts such as C headers or Kotlin sources are produced by the
//! language adapter after the Rust registry is resolved.

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
