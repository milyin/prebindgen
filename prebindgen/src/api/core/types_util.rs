//! Shared `syn::Type` shape utilities — the Option/Vec/reference peelers and
//! short-name helpers every pipeline stage needs. One definition here
//! replaces the per-module copies that used to live in `core::unfold`,
//! `core::expand`, and the jnigen adapter.

use std::collections::HashMap;

use proc_macro2::Span;

use crate::SourceLocation;

/// The single-segment path type for a bare item ident (`Foo` → `Foo`) —
/// direct construction, no string round trip, cannot fail.
pub fn type_from_ident(ident: &syn::Ident) -> syn::Type {
    syn::Type::Path(syn::TypePath {
        qself: None,
        path: syn::Path::from(ident.clone()),
    })
}

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
///    an [`Extern`](crate::core::flat::Extern); it is not an equivalence.
///
///    That keeps the rule meaning-preserving, which is the whole contract
///    here: reduction may choose among spellings of ONE type, never change
///    what a type is. Treating an alias as an equivalence broke that —
///    `Vec<u8>` ≡ `Bytes` turns a sequence into an extern — and no
///    key-shape refinement fixes the category error.
/// 5. Lifetimes are NOT normalized (`&'a T` ≠ `&T`, `Foo<'static>` ≠ `Foo`)
///    — [`match_pattern`] treats lifetimes as fixed structure and
///    foreign-type declarations (`ptr_class!(ZKeyExpr<'static>)`) rely on
///    the verbatim spelling.
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
    /// [`lower_path`](crate::core::flat) classifies as builtins and that have a
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
    /// Both entry points — `FlatBuilder::build` and `Registry::from_items` — build
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
/// model's type index ([`Flat::type_ref`](crate::core::flat::Flat::type_ref))
/// and [`TypeKey`](crate::core::TypeKey) — and they have to agree, so neither
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
/// pass ([`crate::api::core::registry::Registry::from_items`]) that makes
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

/// If `ty` is `Option<Inner>` (by last path segment), return `Inner`.
pub fn option_inner_type(ty: &syn::Type) -> Option<syn::Type> {
    generic_inner(ty, "Option")
}

/// If `ty` is `Vec<Inner>` (by last path segment), return `Inner`.
pub fn vec_inner_type(ty: &syn::Type) -> Option<syn::Type> {
    generic_inner(ty, "Vec")
}

fn generic_inner(ty: &syn::Type, wrapper: &str) -> Option<syn::Type> {
    let syn::Type::Path(tp) = ty else { return None };
    let seg = tp.path.segments.last()?;
    if seg.ident != wrapper {
        return None;
    }
    let syn::PathArguments::AngleBracketed(ab) = &seg.arguments else {
        return None;
    };
    match ab.args.first()? {
        syn::GenericArgument::Type(inner) => Some(inner.clone()),
        _ => None,
    }
}

/// Last path-segment ident of a path type, **generics permitting**
/// (`Option<T>` → `Option`, `Vec<u8>` → `Vec`). Contrast with
/// [`bare_path_ident`], which is `None` for any generic/non-path shape.
pub fn path_tail_ident(ty: &syn::Type) -> Option<syn::Ident> {
    match ty {
        syn::Type::Path(tp) => tp.path.segments.last().map(|s| s.ident.clone()),
        _ => None,
    }
}

/// True when `ty`'s last path segment is `name` (`path_tail_is(ty, "Vec")`).
fn path_tail_is(ty: &syn::Type, name: &str) -> bool {
    path_tail_ident(ty).is_some_and(|i| i == name)
}

/// True when `ty` is `Option<…>` (by last path segment).
pub fn is_option_type(ty: &syn::Type) -> bool {
    path_tail_is(ty, "Option")
}

/// True when `ty` is `Vec<…>` (by last path segment).
#[cfg(feature = "unstable-cbindgen")]
pub fn is_vec_type(ty: &syn::Type) -> bool {
    path_tail_is(ty, "Vec")
}

/// True when `ty` is `Result<…>` (by last path segment).
#[cfg(feature = "unstable-cbindgen")]
pub fn is_result_type(ty: &syn::Type) -> bool {
    path_tail_is(ty, "Result")
}

/// True when `ty` is the unit type `()`.
#[cfg(feature = "unstable-cbindgen")]
pub fn is_unit(ty: &syn::Type) -> bool {
    matches!(ty, syn::Type::Tuple(t) if t.elems.is_empty())
}

/// If `ty` is `Result<T, E>` (by last path segment), return `(T, E)`.
pub fn result_parts(ty: &syn::Type) -> Option<(syn::Type, syn::Type)> {
    let syn::Type::Path(tp) = ty else { return None };
    let seg = tp.path.segments.last()?;
    if seg.ident != "Result" {
        return None;
    }
    let syn::PathArguments::AngleBracketed(ab) = &seg.arguments else {
        return None;
    };
    let mut args = ab.args.iter().filter_map(|a| match a {
        syn::GenericArgument::Type(t) => Some(t.clone()),
        _ => None,
    });
    let ok = args.next()?;
    let err = args.next()?;
    Some((ok, err))
}

/// If `ty` is `Result<T, E>`, return `T`.
pub fn result_ok_type(ty: &syn::Type) -> Option<syn::Type> {
    result_parts(ty).map(|(ok, _)| ok)
}

/// If `ty` is `Result<T, E>`, return `E`.
pub fn result_err_type(ty: &syn::Type) -> Option<syn::Type> {
    result_parts(ty).map(|(_, err)| err)
}

/// First angle-bracketed **type** argument of a path type (`T` of `Option<T>`
/// / `Vec<T>` / `Result<T, _>`), skipping lifetime/const args. `None` when
/// there is no type argument.
#[cfg(feature = "unstable-cbindgen")]
pub fn first_type_arg(ty: &syn::Type) -> Option<syn::Type> {
    let syn::Type::Path(tp) = ty else { return None };
    let seg = tp.path.segments.last()?;
    let syn::PathArguments::AngleBracketed(ab) = &seg.arguments else {
        return None;
    };
    ab.args.iter().find_map(|a| match a {
        syn::GenericArgument::Type(t) => Some(t.clone()),
        _ => None,
    })
}

/// True when `ty` is `Option<&T>` / `Option<&mut T>`.
pub fn is_option_ref(ty: &syn::Type) -> bool {
    option_inner_type(ty).is_some_and(|inner| matches!(inner, syn::Type::Reference(_)))
}

/// The bare ident of a plain path type (`ZThing` → `ZThing`); `None` for
/// references, generics, or multi-shape types.
pub fn bare_path_ident(ty: &syn::Type) -> Option<syn::Ident> {
    let syn::Type::Path(tp) = ty else { return None };
    let seg = tp.path.segments.last()?;
    if !matches!(seg.arguments, syn::PathArguments::None) {
        return None;
    }
    Some(seg.ident.clone())
}

/// Strip any nesting of `&` / `Option<…>` / `Vec<…>` layers down to the core
/// type (`Option<&Vec<ZThing>>` → `ZThing`).
pub fn peel_ref_option_vec(ty: &syn::Type) -> syn::Type {
    let mut t = ty.clone();
    loop {
        if let syn::Type::Reference(r) = &t {
            t = (*r.elem).clone();
            continue;
        }
        if let Some(inner) = option_inner_type(&t).or_else(|| vec_inner_type(&t)) {
            t = inner;
            continue;
        }
        return t;
    }
}

/// Build an identifier at call-site span.
pub(crate) fn ident(s: &str) -> syn::Ident {
    syn::Ident::new(s, Span::call_site())
}

/// Convert a `PascalCase` / `camelCase` identifier to `snake_case`
/// (`ZKeyExpr` → `z_key_expr`). The single implementation behind the
/// public `prebindgen::lang::snake_case` re-export and the sum-variant
/// leaf naming in [`SumSpec`].
pub fn pascal_to_snake(s: &str) -> String {
    let mut out = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if i != 0 {
                out.push('_');
            }
            out.extend(c.to_lowercase());
        } else {
            out.push(c);
        }
    }
    out
}

// ── Enum shape: the one definition of "is this enum C-like" ────────────

/// How a captured `enum` can cross a language boundary — the single
/// classifier both adapters consult instead of each asserting on
/// `syn::Fields` itself.
///
/// The two shapes are not two mechanisms: a [`Unit`](EnumShape::Unit) enum
/// is the degenerate sum whose every variant group is empty, so a lowering
/// written for [`Sum`](EnumShape::Sum) collapses to "just a tag" for it.
/// The distinction exists because the *declarators* differ — `enum_class!`
/// / `.enum_type()` accept only the degenerate case, and handing them a
/// payload enum is an error naming the sum declarator rather than a shape
/// assertion.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EnumShape {
    /// Every variant is fieldless: the value is exactly its discriminant.
    Unit,
    /// At least one variant carries a payload.
    Sum,
}

/// Classify an enum. See [`EnumShape`].
pub fn enum_shape(e: &syn::ItemEnum) -> EnumShape {
    if e.variants
        .iter()
        .all(|v| matches!(v.fields, syn::Fields::Unit))
    {
        EnumShape::Unit
    } else {
        EnumShape::Sum
    }
}

/// The first payload-carrying variant of an enum, if any — the offender an
/// adapter names when rejecting a [`Sum`](EnumShape::Sum) where only a
/// [`Unit`](EnumShape::Unit) enum is accepted.
pub fn first_payload_variant(e: &syn::ItemEnum) -> Option<&syn::Variant> {
    e.variants
        .iter()
        .find(|v| !matches!(v.fields, syn::Fields::Unit))
}

/// The language-neutral description of a data-carrying enum: a **tag** —
/// which alternative is live — plus one **leaf group per variant**.
///
/// Core describes the sum; adapters decide what its leaves look like on the
/// wire (`JniGen` overlays the groups in the signature, `Cbindgen` overlays
/// them in memory as a `#[repr(C)]` union). Nothing here names a wire
/// detail — in particular a payload enum carries no `repr`, so tags are
/// declaration order and never an explicit discriminant.
/// The neutral description lands before either lowering, so both adapters
/// read one definition instead of growing a private one each; the
/// `dead_code` allow covers that gap and goes away with the first adapter
/// that reads a sum.
#[allow(dead_code)]
pub struct SumSpec {
    /// Canonical key of the enum type.
    pub key: crate::api::core::registry::TypeKey,
    /// The enum's ident as declared in the source crate — the spelling
    /// adapters use to build `Enum::Variant` constructor paths.
    pub source: syn::Ident,
    /// Variants in declaration order; `variants[i].tag == i as i32`.
    pub variants: Vec<SumVariant>,
}

/// One alternative of a [`SumSpec`].
#[allow(dead_code)]
pub struct SumVariant {
    /// The variant ident as declared (`PeriodicQueries`).
    pub ident: syn::Ident,
    /// Declaration-order tag, `0..N-1`.
    pub tag: i32,
    /// The variant's payload, in declaration order. Empty for a unit
    /// variant — the group that contributes nothing but its tag.
    pub fields: Vec<SumField>,
}

/// One payload field of a [`SumVariant`].
#[allow(dead_code)]
pub struct SumField {
    /// How the field is addressed in a pattern: `Named(ident)` for a
    /// struct variant, `Unnamed(index)` for a tuple variant.
    pub member: syn::Member,
    /// Leaf name, following the existing nested-prefix convention:
    /// `<variant_snake>_<field>` for a named field, `<variant_snake>_<i>`
    /// for a tuple field.
    pub name: String,
    /// The field's declared type.
    pub ty: syn::Type,
}

#[allow(dead_code)]
impl SumSpec {
    /// Describe `e` as a sum. Every enum has a description — a
    /// [`Unit`](EnumShape::Unit) enum yields all-empty groups, which is
    /// exactly the "tag only" lowering — so this never fails and never
    /// consults [`enum_shape`].
    pub fn from_item_enum(e: &syn::ItemEnum) -> Self {
        let variants = e
            .variants
            .iter()
            .enumerate()
            .map(|(i, v)| {
                let prefix = pascal_to_snake(&v.ident.to_string());
                let fields = v
                    .fields
                    .iter()
                    .enumerate()
                    .map(|(fi, f)| match &f.ident {
                        Some(id) => SumField {
                            member: syn::Member::Named(id.clone()),
                            name: format!("{prefix}_{id}"),
                            ty: f.ty.clone(),
                        },
                        None => SumField {
                            member: syn::Member::Unnamed(syn::Index::from(fi)),
                            name: format!("{prefix}_{fi}"),
                            ty: f.ty.clone(),
                        },
                    })
                    .collect();
                SumVariant {
                    ident: v.ident.clone(),
                    tag: i as i32,
                    fields,
                }
            })
            .collect();
        Self {
            key: crate::api::core::registry::TypeKey::from_ident(&e.ident),
            source: e.ident.clone(),
            variants,
        }
    }
}

#[allow(dead_code)]
impl SumVariant {
    /// True when this variant carries no payload — its leaf group is empty
    /// and it contributes only its tag.
    pub fn is_unit(&self) -> bool {
        self.fields.is_empty()
    }
}

/// Resolve each enum variant to its discriminant value following Rust's own
/// assignment rule: an explicit `= N` sets the value, an implicit variant
/// takes the previous value plus one (starting at 0).
///
/// The single source of truth for every int↔variant mapping in the
/// pipeline — the Kotlin `value(N)` constants, the generated `jint →
/// variant` decode, and the `#[repr(C)]` mirror `Cbindgen` emits — keeping
/// them from drifting and removing the need for a hand-written
/// `TryFrom<i32>` on the source enum. Non-literal discriminants are
/// rejected because prebindgen cannot reliably evaluate arbitrary
/// expressions at codegen time.
///
/// This describes the **unit** enum's wire numbering. A payload enum's
/// alternatives are identified by the declaration-order tag of
/// [`SumSpec`], never by a discriminant.
pub fn enum_discriminant_values(e: &syn::ItemEnum) -> Vec<(syn::Ident, i64)> {
    let mut out = Vec::with_capacity(e.variants.len());
    let mut next: i64 = 0;
    for variant in &e.variants {
        let value = match variant.discriminant.as_ref() {
            Some((_, expr)) => extract_int_literal(expr).unwrap_or_else(|| {
                panic!(
                    "enum `{}` variant `{}` has a non-literal discriminant; use a literal integer value (e.g. `= 1`) or an implicit discriminant",
                    e.ident,
                    variant.ident
                )
            }),
            None => next,
        };
        out.push((variant.ident.clone(), value));
        next = value + 1;
    }
    out
}

/// Pull a signed integer out of a `syn::Expr` literal (`5`, `-3`, `0x07`).
/// Returns `None` for anything else (constants, paths, arithmetic).
fn extract_int_literal(expr: &syn::Expr) -> Option<i64> {
    match expr {
        syn::Expr::Lit(lit) => match &lit.lit {
            syn::Lit::Int(int) => int.base10_parse::<i64>().ok(),
            _ => None,
        },
        syn::Expr::Unary(syn::ExprUnary {
            op: syn::UnOp::Neg(_),
            expr,
            ..
        }) => extract_int_literal(expr).map(|v| -v),
        _ => None,
    }
}

#[cfg(test)]
mod tests;
