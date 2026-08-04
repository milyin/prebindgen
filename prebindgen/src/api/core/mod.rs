//! Core: language-agnostic primitives for the Registry-based pipeline.
//!
//! The pipeline is phase-oriented, and a generator drives it:
//!
//! 1. [`flat::Flat::builder`] parses `(syn::Item, SourceLocation)` records into
//!    one flat namespace; [`registry::Registry::builder`] starts describing a
//!    binding over the model.
//! 2. The generator states its binding — which elements it exports, which types
//!    cross, how composites decompose — and the builder derives the crossing
//!    set from it.
//! 3. [`registry::Registry::crossings`] hands that set over inner-first; the
//!    generator builds a conversion for each and returns them all through
//!    `RegistryBuilder::convert_with`, and `build` checks the set is complete.
//! 4. The emitter writes prerequisites, converters, per-item wrapper Rust, and
//!    verbatim anonymous consts.
//!
//! Secondary artifacts such as C headers or Kotlin sources are produced by the
//! language adapter after the Rust registry is resolved.

pub mod diagnostics;
pub mod domain;
pub mod emit;
pub mod expand;
pub mod flat;
pub mod gravestone;
pub mod niches;
pub mod prebindgen;
pub mod registry;
pub(crate) mod resolve;
pub mod shape;
pub mod types_util;
pub mod unfold;
pub(crate) mod write;

pub use self::{
    diagnostics::{warn_unclaimed, Claimed},
    domain::{DomainScalar, RepresentationDomain, ScalarValue},
    flat::{Element, Flat},
    gravestone::{Gravestone, Transmute},
    niches::{NicheSlot, Niches},
    prebindgen::{ConverterImpl, Prebindgen, Stage},
    registry::{
        Building, Conversions, Crossing, Decompositions, Direction, DuplicateNameError,
        NotExpressibleEntry, Registry, ScanError, TypeEntry, TypeKey, TypeKeyParseError,
        WriteRustError,
    },
};
