//! What a crossing IS: the type it names, the conversion for it, and which way
//! it goes.

use super::*;

/// What a type-table key names.
///
/// Two populations, and saying which is which is what keeps one origin per cell:
/// a type the flat API contains **is** a [`TypeRef`](crate::api::core::flat::TypeRef), reused whole, so its
/// classification and its source location are already there.
#[derive(Clone, Debug)]
pub(crate) enum TypeSubject {
    /// A type the flat API contains — the frontend's own reading, unmodified.
    Source(crate::api::core::flat::TypeRef),
    /// A type only the binding authored: a declared wire type with no
    /// `#[prebindgen]` item behind it, an [`unfold`](crate::api::core::unfold)
    /// leaf. It has no reading and no source location — a fact about it, rather
    /// than information that went missing.
    Adapter,
}

impl TypeSubject {
    /// Where the source wrote this type, or `None` when no source did.
    pub fn location(&self) -> Option<&SourceLocation> {
        match self {
            // Having a reading and having a reportable position are different
            // facts: a binding-local fn's types are lowered — so they have
            // readings — against no file at all. Reporting `:0:0` would invent a
            // position; `None` says what is true.
            TypeSubject::Source(t) => Some(&*t.origin.location).filter(|l| l.has_position()),
            TypeSubject::Adapter => None,
        }
    }

    /// The frontend's classification, or `None` for an adapter-authored type.
    ///
    /// Test-only: the cells carry it so a test can pin that a source reading
    /// survives into the table, but no production path re-reads it.
    #[cfg(test)]
    pub fn kind(&self) -> Option<&crate::api::core::flat::TypeKind> {
        match self {
            TypeSubject::Source(t) => Some(&t.kind),
            TypeSubject::Adapter => None,
        }
    }
}

/// One type-table cell: what the key names, and the adapter's answer for it.
pub(crate) struct TypeCell<M = ()> {
    /// The type itself, as the frontend reads it when it can.
    pub subject: TypeSubject,
    /// The binding asks for this cell **directly** — a declared fn's signature, a
    /// declared type, an `unfold` leaf — as opposed to reaching it through some
    /// converter's [`TypeEntry::subs`].
    ///
    /// A scan fact. Whether a converter is *needed* here is reachability from
    /// these roots, which [`crate::api::core::resolve`] derives rather than
    /// stores: the scan deliberately over-approximates the table (every nested
    /// position, every struct in both directions), so the roots are what say
    /// which of it has to work.
    pub root: bool,
    /// The adapter's converter, once resolved.
    pub entry: Option<TypeEntry<M>>,
}

/// Per-cell registry entry.
#[derive(Clone)]
pub struct TypeEntry<M = ()> {
    /// Wire/destination type — the form the value takes on the wire as
    /// chosen by the adapter (e.g. an `i64` handle for a JNI adapter, or
    /// a `*const T` raw pointer for a C adapter). Other converters that
    /// ask "what's the wire form of this rust type?" read this.
    pub destination: syn::Type,
    /// Complete generated function for the **wire-facing** stage of the
    /// converter (signature, body, attributes, lifetimes). The adapter
    /// owns the shape. Callers compute this stage's name via
    /// `function.sig.ident`.
    pub function: syn::ItemFn,
    /// **Rust-side** stages that compose with [`Self::function`] to form
    /// the full chain — copied verbatim from the resolving
    /// [`crate::api::core::prebindgen::ConverterImpl::pre_stages`]. See
    /// that field's docs for the chain-order semantics.
    pub pre_stages: Vec<Stage<M>>,
    /// Inner types whose function delegates to their converters. Empty for
    /// terminal converters; populated by wrapper converters. Used by the
    /// post-resolution propagation pass.
    pub subs: Vec<TypeKey>,
    /// Wire bit-patterns this converter never produces / always rejects.
    /// Wrappers (`Option<_>`, sum-typed enums) carve from this set for
    /// their own discriminants. See [`Niches`] for the cascade model.
    pub niches: Niches,
    /// Adapter-specific extras carried in by the
    /// [`crate::api::core::prebindgen::ConverterImpl`] that filled this
    /// slot. Emitter code reads this directly — the registry is the
    /// single source of truth for cross-language facts (C header names,
    /// JVM class names, etc.). Defaults to `()` for adapters that don't
    /// need any.
    pub metadata: M,
}

impl<M> TypeEntry<M> {
    /// The resolved form of what a generator built.
    ///
    /// The only difference is `subs`: a generator names its inners as types,
    /// and the table keys them.
    pub fn from_converter(c: crate::api::core::prebindgen::ConverterImpl<M>) -> Self {
        Self {
            destination: c.destination,
            function: c.function,
            pre_stages: c.pre_stages,
            subs: c.subs.iter().map(TypeKey::from_type).collect(),
            niches: c.niches,
            metadata: c.metadata,
        }
    }

    /// Identifier of the wire-facing converter function.
    pub fn converter_ident(&self) -> &syn::Ident {
        &self.function.sig.ident
    }

    /// Wire/destination type carried by this converter on success.
    pub fn wire_type(&self) -> &syn::Type {
        &self.destination
    }

    /// Rust-side stages in input execution order, after the wire-facing
    /// converter has decoded the wire value.
    pub fn input_stage_order(&self) -> impl Iterator<Item = (usize, &Stage<M>)> {
        self.pre_stages.iter().enumerate().rev()
    }

    /// Rust-side stages in output execution order, before the wire-facing
    /// converter encodes the final wire value.
    pub fn output_stage_order(&self) -> impl Iterator<Item = (usize, &Stage<M>)> {
        self.pre_stages.iter().enumerate()
    }

    /// Immediate converter dependencies recorded by the adapter when this entry
    /// resolved.
    pub fn dependency_keys(&self) -> &[TypeKey] {
        &self.subs
    }
}

/// Direction of a converter pair.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum Direction {
    /// Wire → Rust.
    Input,
    /// Rust → Wire.
    Output,
}

impl Direction {
    pub fn flip(self) -> Self {
        match self {
            Direction::Input => Direction::Output,
            Direction::Output => Direction::Input,
        }
    }
}
