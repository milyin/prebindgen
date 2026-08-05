//! Spelling: how a captured type is written, reduced to one canonical form.
//!
//! Ingest-time machinery, and the frontend's own — it decides what spelling a
//! type *has* before anything keys on it, which is the same authority that
//! decides what a type *means*. It lived in `api/core/types_util` until #229's
//! L2d, next to the classifiers, where it looked like one of them; it is not.
//! Nothing outside `api/core` ever called it.
//!
//! Two things key on [`canonical_type`] — this module's type index and
//! [`TypeKey`](crate::TypeKey) — and they must agree, which is why the
//! reduction has exactly one definition and neither spells it out itself.
//!
//! The ledger counts *constructing* a watched syn variant as well as matching
//! one, so [`type_from_ident`] is here for the same reason the rest is: writing
//! a `syn::Type::Path` is spelling, and spelling belongs to the module that owns
//! the grammar.

use std::collections::HashMap;

use prebindgen::SourceLocation;

/// Normalize a type to its canonical flat-namespace spelling (issue #95).
/// The COMPLETE equivalence rule set — any spelling not listed is preserved
/// verbatim:
///
/// 1. `Type::Group` / `Type::Paren` wrappers unwrap (`(Foo)` ≡ `Foo`).
/// 2. A multi-segment path headed by `crate` / `self` reduces to its final
///    segment, keeping that segment's generic arguments (`crate::a::Foo<T>`
///    ≡ `Foo<T>`). Sound because the flat namespace indexes at most one
///    item per bare ident, and a `crate::` path in a captured item can only
///    denote the source crate's own item.
/// 3. A multi-segment path headed by a name in `source_modules` (the
///    `#[prebindgen]` source crates chained into the registry,
///    hyphens-as-underscores) reduces the same way (`myflat::Foo` ≡ `Foo`).
///    Pure callers pass `&[]`.
/// 4. A **prelude** path reduces to the bare name the language knows it by —
///    exactly [`Normalization::PRELUDE`], with `core`/`alloc` read as `std`.
///    Each entry names a *constructor*, so arguments are preserved:
///    `std::vec::Vec<Foo>` ≡ `Vec<Foo>`.
///
///    Nothing else. `std::ffi::CString` stays qualified, and so does a
///    foreign path (`zenoh::KeyExpr`) **even when an alias names that
///    type**: a `#[prebindgen] pub type` is a one-way road, bringing a
///    foreign type into the flat API under a name that is thereafter the
///    only way to spell it. It declares
///    an [`Extern`](crate::flat::Extern); it is not an equivalence.
///
///    That keeps the rule meaning-preserving, which is the whole contract
///    here: reduction may choose among spellings of ONE type, never change
///    what a type is. Treating an alias as an equivalence broke that —
///    `Vec<u8>` ≡ `Bytes` turns a sequence into an extern — and no
///    key-shape refinement fixes the category error.
/// 5. Lifetimes are NOT normalized (`&'a T` ≠ `&T`, `Foo<'static>` ≠ `Foo`)
///    — a lifetime is part of the spelling a foreign-type declaration relies
///    on (`ptr_class!(ZKeyExpr<'static>)`), so collapsing it would make two
///    distinct declarations collide.
///
/// Idempotent; recurses through references, slices, tuples, pointers,
/// generic arguments, and `impl Trait` bounds. Paths with a qualified self
/// (`<T as Trait>::Assoc`) are left untouched.
/// What a captured path may be reduced against: the ingested source crates' own
/// modules, and every name an alias gives to a foreign path.
///
/// One value rather than a bare `&[String]`, because reduction has one rule and
/// two sources of aliases feeding it — see [`normalize_type`]'s rule list.
/// [`Self::default`] is the prelude alone, which is what a caller normalizing a
/// lone type (rather than an ingested stream) wants.
#[derive(Clone, Debug)]
pub struct Normalization {
    /// Module name per ingested source, first-seen order. The first doubles as the
    /// default module for references with no recorded origin.
    pub source_modules: Vec<String>,
    /// Constructor path → the bare name the language knows it by, from
    /// [`Self::PRELUDE`] alone. Matched with the use site's type arguments ignored
    /// and preserved, because a prelude entry names a constructor:
    /// `std::vec::Vec` is every `Vec<T>`.
    ///
    /// A crate's `#[prebindgen] pub type` is deliberately **not** here — see
    /// [`normalize_type`]'s rule 4.
    constructors: HashMap<String, String>,
}

impl Normalization {
    /// The names the language **pre-declares**, so no source crate has to write
    /// them — exactly Rust's own idea of a prelude, a set of `use`s you need not
    /// write. A crate need not write `use std::vec::Vec`, and need not write
    /// `#[prebindgen] pub type Vec = std::vec::Vec` either, for the same reason.
    ///
    /// Not identical to Rust's prelude: it adds `MaybeUninit`, which the grammar
    /// recognises for out-parameters, and `Cow`, which it treats as transparent.
    /// Its entries are exactly the bare names
    /// [`lower_path`](crate::flat) classifies as builtins and that have a
    /// std path at all — `str` has none, and neither do the scalars.
    ///
    /// Written with the `std` root; `core` and `alloc` are re-exports of the same
    /// items, so a leading `core`/`alloc` is read as `std` before matching.
    pub const PRELUDE: &'static [(&'static str, &'static str)] = &[
        ("std::vec::Vec", "Vec"),
        ("std::option::Option", "Option"),
        ("std::result::Result", "Result"),
        ("std::string::String", "String"),
        ("std::boxed::Box", "Box"),
        ("std::mem::MaybeUninit", "MaybeUninit"),
        ("std::borrow::Cow", "Cow"),
    ];

    /// The prelude alone: no ingested sources, no declared aliases.
    pub fn prelude() -> Self {
        Self {
            source_modules: Vec::new(),
            constructors: Self::PRELUDE
                .iter()
                .map(|(path, name)| ((*path).to_string(), (*name).to_string()))
                .collect(),
        }
    }

    /// Collect from a captured stream, before anything is normalized.
    ///
    /// The single entry point — `FlatBuilder::build` — builds
    /// this, so they cannot normalize differently. Gathering every module and alias
    /// first is what makes reduction order-independent: a signature may name a type
    /// whose alias is declared later, or in another source.
    pub fn from_items(items: &[(syn::Item, SourceLocation)]) -> Self {
        let mut out = Self::prelude();
        for (_, loc) in items {
            if let Some(crate_name) = &loc.crate_name {
                let module = crate_name.replace('-', "_");
                if !out.source_modules.contains(&module) {
                    out.source_modules.push(module);
                }
            }
        }
        out
    }

    /// The bare name the language knows this constructor by, arguments ignored.
    fn constructor_of(&self, path: &syn::Path) -> Option<&str> {
        self.constructors
            .get(&constructor_key(path))
            .map(String::as_str)
    }
}

impl Default for Normalization {
    fn default() -> Self {
        Self::prelude()
    }
}

/// A path as a key: segments joined, arguments dropped, and a leading
/// `core`/`alloc` read as `std` since they re-export the same items.
///
/// Only [`Normalization::constructors`] is keyed this way, and a constructor is
/// exactly a path without arguments — `std::vec::Vec` matches every `Vec<T>`.
fn constructor_key(path: &syn::Path) -> String {
    let mut out = String::new();
    for (i, seg) in path.segments.iter().enumerate() {
        if i > 0 {
            out.push_str("::");
        }
        let mut ident = seg.ident.to_string();
        if i == 0 && (ident == "core" || ident == "alloc") {
            ident = "std".to_string();
        }
        out.push_str(&ident);
    }
    out
}

/// A type reduced to the spelling everything keys on: prelude-normalized, so
/// `std::option::Option<T>` and `Option<T>` are one entry.
///
/// The **single** definition of that reduction. Two things key on it — the
/// model's type index ([`Flat::type_ref`](crate::flat::Flat::type_ref))
/// and [`TypeKey`](crate::TypeKey) — and they have to agree, so neither
/// spells it out itself.
///
/// Deliberately `prelude()` rather than a source-module-aware normalization: a
/// key must mean the same thing before and after ingestion knows what the source
/// modules are.
pub fn canonical_type(ty: &syn::Type) -> syn::Type {
    let mut t = ty.clone();
    normalize_type(&mut t, &Normalization::prelude());
    t
}

/// [`canonical_type`] as tokens — the string form both indexes use as their key.
pub fn canonical_spelling(ty: &syn::Type) -> String {
    use quote::ToTokens;
    canonical_type(ty).to_token_stream().to_string()
}

pub fn normalize_type(ty: &mut syn::Type, against: &Normalization) {
    use syn::visit_mut::VisitMut;
    struct Normalizer<'a> {
        against: &'a Normalization,
    }
    impl VisitMut for Normalizer<'_> {
        fn visit_type_mut(&mut self, ty: &mut syn::Type) {
            // Unwrap (possibly nested) group/paren wrappers in place.
            loop {
                match ty {
                    syn::Type::Group(g) => *ty = (*g.elem).clone(),
                    syn::Type::Paren(p) => *ty = (*p.elem).clone(),
                    _ => break,
                }
            }
            if let syn::Type::Path(tp) = ty {
                if tp.qself.is_none() {
                    reduce_flat_path(&mut tp.path, self.against);
                }
            }
            syn::visit_mut::visit_type_mut(self, ty);
        }
    }
    Normalizer { against }.visit_type_mut(ty);
}

/// Apply [`normalize_type`] to every type position inside an item — fn
/// signatures, struct fields, enum variants, const types. The ingest-time
/// pass ([`crate::flat::FlatBuilder::build`]) that makes
/// captured spellings canonical before any key is formed, so every
/// downstream `TypeKey::from_type` sees the flat spelling.
pub fn normalize_item_types(item: &mut syn::Item, against: &Normalization) {
    use syn::visit_mut::VisitMut;

    struct ItemNormalizer<'a> {
        against: &'a Normalization,
    }
    impl VisitMut for ItemNormalizer<'_> {
        fn visit_type_mut(&mut self, ty: &mut syn::Type) {
            // Normalizes the whole subtree; no further descent needed.
            normalize_type(ty, self.against);
        }
    }
    ItemNormalizer { against }.visit_item_mut(item);
}

/// The path-reduction step of [`normalize_type`]: collapse a reducible
/// multi-segment path to its final segment. See the rule list there.
fn reduce_flat_path(path: &mut syn::Path, against: &Normalization) {
    if path.segments.len() < 2 {
        return;
    }

    // A prelude entry names a CONSTRUCTOR, so arguments are ignored when matching
    // and preserved when rewriting: `std::vec::Vec<Foo>` is `Vec<Foo>`. A crate's
    // own alias is NOT consulted — see rule 4.
    if let Some(name) = against.constructor_of(path) {
        let mut last = path.segments.last().expect("len checked").clone();
        last.ident = syn::Ident::new(name, last.ident.span());
        path.leading_colon = None;
        path.segments = std::iter::once(last).collect();
        return;
    }

    // Otherwise only a prefix into the flat namespace reduces, to the final
    // segment: this crate's own path, or an ingested source's module.
    let head = path
        .segments
        .first()
        .expect("len checked")
        .ident
        .to_string();
    let reduce = match head.as_str() {
        "crate" | "self" => true,
        other => against.source_modules.iter().any(|m| m == other),
    };
    if reduce {
        let last = path.segments.last().expect("len checked").clone();
        path.leading_colon = None;
        path.segments = std::iter::once(last).collect();
    }
}
