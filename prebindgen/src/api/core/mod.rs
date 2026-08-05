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

pub mod decl;
pub mod diagnostics;
pub mod domain;
pub mod expand;
pub mod flat;
/// The `Emit` capability lives with the flat model, because every one of its
/// methods delegates to a model method that is private to `flat` — that
/// pairing is what makes `Emit` the only route to a node's syntax. Re-exported
/// here so `core::emit` keeps naming it.
pub use self::flat::emit;
pub mod niches;
pub mod prebindgen;
pub mod registry;
pub(crate) mod resolve;
pub mod shape;
pub mod types_util;
pub mod unfold;
pub mod write;

pub use self::{
    decl::{
        ConvertDecl, ConvertSourceDecl, ConvertSpec, ExpandDecl, ExpandParamDecl, ExpandReturnDecl,
        FieldsDecl, FunctionDecl, LocalField, LocalVariant,
    },
    diagnostics::{warn_unclaimed, Claimed},
    domain::{DomainScalar, RepresentationDomain, ScalarValue},
    flat::{Element, Flat},
    niches::{NicheSlot, Niches},
    prebindgen::{ConverterImpl, NamePredicate, Prebindgen, Stage},
    registry::{
        Building, Conversions, Crossing, Decompositions, Direction, DuplicateNameError,
        NotExpressibleEntry, Registry, RegistryBuilder, ScanError, TypeEntry, TypeKey,
        TypeKeyParseError, WriteRustError,
    },
};
