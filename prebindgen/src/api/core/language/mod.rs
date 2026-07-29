//! The prebindgen **source language**: one parser from captured records to
//! [`Element`]s.
//!
//! > Naming: `core::language` is the *source* language — the Rust subset a
//! > `#[prebindgen]` crate may write. `api::lang` is the *destination* adapters
//! > (C, JNI). They are opposite ends of the pipeline.
//!
//! ```text
//! Source(s) ──items──> Language ──Elements──> Registry ──> adapters
//!   raw records          parse +               indexes       classify off `kind`
//!   (syn::Item)          validate              elements      spell off `syntax`
//! ```
//!
//! # What an element is
//!
//! Two things at once, and that pairing is the whole design:
//!
//! * a **closed classification** — [`TypeKind`], the field list, the variant
//!   list — that says what the source *means*, in terms every destination
//!   language shares;
//! * the **exact syntax** each part was built from, sliced down to the
//!   parameter, field, variant and type.
//!
//! So the rule for every consumer is:
//!
//! > **Classify off `kind`, spell off `syntax`.**
//!
//! Matching a `syn::Type` or `syn::Expr` variant outside this module is a
//! classifier, and issue #211 says classification lives here alone. Passing a
//! `syntax` slice into `quote!` is spelling, and spelling the source is exactly
//! what generated Rust must do — see [`spell`] for the helpers that do it.
//!
//! # What earns a variant
//!
//! A concept, not a Rust spelling. The test is whether a *destination* language
//! would act on the distinction; if only Rust can see it, it is spelling, and
//! the slice already carries it:
//!
//! | Rust writes | The model says | Because |
//! |---|---|---|
//! | `String`, `str` | [`TypeKind::Str`] | one concept, two Rust types |
//! | `Vec<T>`, `[T]` | [`TypeKind::Sequence`] | a run of `T`; owned vs borrowed is the [`Ref`](TypeKind::Ref) layer's fact |
//! | `Box<T>` | whatever `T` is | an owned `T` either way |
//! | `struct S;`, `struct S {}` | zero fields | the delimiters are spelling |
//! | no `->`, `-> ()` | [`TypeKind::Unit`] | the same function |
//! | `*const T` | *rejected* | a source crate is idiomatic Rust; the adapter owns pointers |
//!
//! The identities follow the same rule: a nominal type is a [`TypeId`] — a
//! name — not a `syn::Path`, so nothing downstream has to take a path apart to
//! learn what a type is.
//!
//! # Why the syntax rides along
//!
//! The generated Rust glue is itself a destination artifact, and the only one
//! that needs syntax fidelity: `B()` must not be re-spelled `B`, `= 0x07` must
//! not become `= 7`, `Foo<'a>` is not `Foo`. A model that carries no syntax has
//! to become *lossless* to serve it — which is how a language-neutral IR turns
//! back into a second `syn`. Carrying the original slice costs nothing and lets
//! the classification stay small: a lifetime, a delimiter and a literal's base
//! are simply not modelled facts.
//!
//! # Where acceptance is enforced
//!
//! Lowering is **total over the accepted grammar**: a form with no variant in
//! [`TypeKind`] is a form the language does not accept, so there is no second
//! acceptance list to drift from it.
//!
//! But an item the language cannot express is not automatically a build failure.
//! A source crate may mark items no binding uses, and those have never been
//! required to be expressible — the pipeline scans a signature only once an
//! adapter *declares* it. So parsing diagnoses per item and defers the raising:
//! such an item becomes [`Element::Unsupported`], carrying the diagnosis, inert
//! until something declares it. Only whole-stream rules — a duplicate name in
//! the flat namespace — are [`ParseError`]s, because no declaration can make
//! two items with one name unambiguous.

use std::fmt;

use quote::ToTokens;

mod array_len;
#[cfg(test)]
mod boundary;
mod element;
pub mod spell;
mod ty;

#[cfg(test)]
mod tests;

use self::{array_len::ConstIndex, ty::lower_type};
pub use self::{
    array_len::{ArrayExtent, ArrayLenReason, ConstId, ExtentSource, UnsupportedArrayLen},
    element::{
        Const, Element, Enum, Field, Function, Param, Passthrough, Struct, Unsupported, Variant,
    },
    ty::{ScalarKind, Type, TypeId, TypeKind, UnsupportedType, UnsupportedTypeReason},
};
use crate::SourceLocation;

/// The parser for the prebindgen source language.
///
/// Carries no configuration: what the language accepts is a property of the
/// language, not of the call site. See the [module docs](self) for the contract.
///
/// ```ignore
/// let elements = Language::new().parse(
///     flat.items_all().chain(helpers.items_in_groups(&["api"])),
/// )?;
/// ```
#[derive(Clone, Copy, Debug, Default)]
pub struct Language;

impl Language {
    pub fn new() -> Self {
        Self
    }

    /// Parse a captured item stream into elements.
    ///
    /// Takes any `(syn::Item, SourceLocation)` iterator — typically
    /// `source.items_all()` — so item-level selection and multi-source
    /// composition stay upstream, where they already are.
    ///
    /// **Transactional**: an `Err` yields no elements at all, so a refused
    /// stream cannot leave a half-built model behind.
    ///
    /// Order-independent: source modules are gathered, and consts indexed,
    /// before anything is lowered — so a cross-source type reference and an
    /// array length may both name something declared later in the stream.
    pub fn parse<I>(&self, items: I) -> Result<Vec<Element>, ParseError>
    where
        I: IntoIterator<Item = (syn::Item, SourceLocation)>,
    {
        let mut items: Vec<(syn::Item, SourceLocation)> = items.into_iter().collect();

        // Pass 0: normalize every item's types to the canonical flat spelling
        // before a single one is classified. `std::option::Option<T>` is an
        // `Option`, and `source_a::TypeA` is `TypeA` — decisions this module
        // owns, so it must be the one to see the reduced form. Gathering EVERY
        // module name first is what makes a cross-source reference in an
        // earlier item normalize the same as in a later one.
        //
        // The consequence is deliberate and stated in `Type::syntax`: a slice
        // is the spelling generation must EMIT, which is the normalized one —
        // the flat namespace is what the generated crate can actually name.
        let mut modules: Vec<String> = Vec::new();
        for (_, loc) in &items {
            if let Some(crate_name) = &loc.crate_name {
                let module = crate_name.replace('-', "_");
                if !modules.contains(&module) {
                    modules.push(module);
                }
            }
        }
        for (item, _) in &mut items {
            crate::api::core::types_util::normalize_item_types(item, &modules);
        }

        // Pass 1: the consts an array length may name. Unnamed `const _` items
        // are excluded here for the same reason they are passed through below —
        // they are not addressable.
        let consts = ConstIndex::new(items.iter().filter_map(|(item, loc)| match item {
            syn::Item::Const(c) if c.ident != "_" => Some((
                c.ident.to_string(),
                (*c.expr).clone(),
                loc.crate_name.clone(),
            )),
            _ => None,
        }));

        // Pass 2: lower, checking the flat namespace as we go.
        let mut out: Vec<Element> = Vec::with_capacity(items.len());
        let mut seen: Vec<(syn::Ident, SourceLocation)> = Vec::new();
        for (item, loc) in items {
            let element = lower_item(item, loc, &consts);
            if let Some(name) = element.name() {
                if let Some((first_name, first)) = seen.iter().find(|(n, _)| n == name) {
                    return Err(ParseError::DuplicateName(Box::new(DuplicateName {
                        name: first_name.clone(),
                        first: first.clone(),
                        second: element.location().clone(),
                    })));
                }
                seen.push((name.clone(), element.location().clone()));
            }
            out.push(element);
        }
        Ok(out)
    }
}

/// If `ty` is `impl Fn(T1, T2, ...) + Send + Sync + 'static`, return the `Fn`
/// argument types in declaration order. Otherwise `None`.
///
/// The callback grammar, and the language's alone: [`TypeKind::Callback`] is
/// exactly what this accepts, so acceptance cannot drift from classification.
/// The registry re-exports it for the consumers that have not migrated yet.
pub fn extract_fn_trait_args(ty: &syn::Type) -> Option<Vec<syn::Type>> {
    let syn::Type::ImplTrait(it) = ty else {
        return None;
    };
    let mut args: Option<Vec<syn::Type>> = None;
    let mut has_send = false;
    let mut has_sync = false;
    let mut has_static = false;
    for bound in &it.bounds {
        match bound {
            syn::TypeParamBound::Trait(tb) => {
                let last = tb.path.segments.last()?;
                let name = last.ident.to_string();
                match name.as_str() {
                    "Fn" => {
                        let syn::PathArguments::Parenthesized(p) = &last.arguments else {
                            return None;
                        };
                        args = Some(p.inputs.iter().cloned().collect());
                    }
                    "Send" => has_send = true,
                    "Sync" => has_sync = true,
                    _ => return None,
                }
            }
            syn::TypeParamBound::Lifetime(lt) if lt.ident == "static" => has_static = true,
            _ => return None,
        }
    }
    if has_send && has_sync && has_static {
        args
    } else {
        None
    }
}

/// A rule of the language that no single item can satisfy on its own, and that
/// no adapter declaration can excuse.
#[derive(Clone, Debug)]
pub enum ParseError {
    /// Two `#[prebindgen]` items share a name. Names live in one flat namespace
    /// across every ingested source crate, so this is ambiguous however the
    /// crates are arranged.
    DuplicateName(Box<DuplicateName>),
}

/// The two items of a [`ParseError::DuplicateName`].
#[derive(Clone, Debug)]
pub struct DuplicateName {
    pub name: syn::Ident,
    pub first: SourceLocation,
    pub second: SourceLocation,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::DuplicateName(d) => write!(
                f,
                "duplicate `#[prebindgen]` name `{}`: first at {}, again at {} — marked items \
                 share one flat namespace across all source crates",
                d.name, d.first, d.second
            ),
        }
    }
}

impl std::error::Error for ParseError {}

/// Why one item could not be expressed in the language.
///
/// Carried by [`Element::Unsupported`] rather than raised at parse time: see
/// the [module docs](self) on where acceptance is enforced.
#[derive(Clone, Debug)]
pub enum ItemError {
    /// A `self` receiver. `#[prebindgen]` captures free functions only.
    UnsupportedReceiver,
    /// A parameter pattern that is not a plain name — `(a, b): (u8, u8)`.
    UnsupportedParamPattern { pattern: String },
    /// A parameter's type is not in the language.
    ParamType {
        param: syn::Ident,
        source: UnsupportedType,
    },
    /// A return type is not in the language.
    ReturnType { source: UnsupportedType },
    /// A named struct field's type is not in the language.
    FieldType {
        field: syn::Ident,
        source: UnsupportedType,
    },
    /// A variant payload's type is not in the language.
    VariantFieldType {
        variant: syn::Ident,
        /// The field's name, or its position for a tuple variant.
        field: String,
        source: UnsupportedType,
    },
    /// A const's type is not in the language.
    ConstType { source: UnsupportedType },
}

impl fmt::Display for ItemError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ItemError::UnsupportedReceiver => write!(
                f,
                "takes a `self` receiver; `#[prebindgen]` captures free functions only"
            ),
            ItemError::UnsupportedParamPattern { pattern } => write!(
                f,
                "parameter pattern `{pattern}` is not a plain name — bind each parameter to one \
                 identifier"
            ),
            ItemError::ParamType { param, source } => {
                write!(f, "parameter `{param}`: {source}")
            }
            ItemError::ReturnType { source } => write!(f, "return type: {source}"),
            ItemError::FieldType { field, source } => write!(f, "field `{field}`: {source}"),
            ItemError::VariantFieldType {
                variant,
                field,
                source,
            } => write!(f, "variant `{variant}` field `{field}`: {source}"),
            ItemError::ConstType { source } => write!(f, "const type: {source}"),
        }
    }
}

impl std::error::Error for ItemError {}

/// Lower one captured item. Total: every item becomes an element, and an item
/// whose contents the language cannot express becomes [`Element::Unsupported`]
/// rather than failing the parse.
fn lower_item(item: syn::Item, loc: SourceLocation, consts: &ConstIndex) -> Element {
    let item_crate = loc.crate_name.clone();
    let crate_ref = item_crate.as_deref();
    match item {
        syn::Item::Fn(f) => match lower_fn(&f, &loc, consts, crate_ref) {
            Ok(func) => Element::Function(func),
            Err(error) => unsupported(f.sig.ident.clone(), syn::Item::Fn(f), loc, error),
        },
        syn::Item::Struct(s) => match lower_struct(&s, &loc, consts, crate_ref) {
            Ok(st) => Element::Struct(st),
            Err(error) => unsupported(s.ident.clone(), syn::Item::Struct(s), loc, error),
        },
        syn::Item::Enum(e) => match lower_enum(&e, &loc, consts, crate_ref) {
            Ok(en) => Element::Enum(en),
            Err(error) => unsupported(e.ident.clone(), syn::Item::Enum(e), loc, error),
        },
        // Unnamed `const _` items (each source's injected `konst` feature
        // guard) live outside the flat namespace: several sources may each
        // carry one, all passed through verbatim.
        syn::Item::Const(c) if c.ident == "_" => Element::Passthrough(Passthrough {
            location: loc,
            syntax: syn::Item::Const(c),
        }),
        syn::Item::Const(c) => match lower_type(&c.ty, consts, crate_ref) {
            Ok(ty) => Element::Const(Const {
                name: c.ident.clone(),
                ty,
                location: loc,
                syntax: c,
            }),
            Err(source) => unsupported(
                c.ident.clone(),
                syn::Item::Const(c),
                loc,
                ItemError::ConstType { source },
            ),
        },
        other => Element::Passthrough(Passthrough {
            location: loc,
            syntax: other,
        }),
    }
}

fn unsupported(
    name: syn::Ident,
    syntax: syn::Item,
    location: SourceLocation,
    error: ItemError,
) -> Element {
    Element::Unsupported(Unsupported {
        name,
        location,
        error: Box::new(error),
        syntax,
    })
}

fn lower_fn(
    f: &syn::ItemFn,
    loc: &SourceLocation,
    consts: &ConstIndex,
    item_crate: Option<&str>,
) -> Result<Function, ItemError> {
    let mut params = Vec::with_capacity(f.sig.inputs.len());
    for input in &f.sig.inputs {
        let pt = match input {
            syn::FnArg::Receiver(_) => return Err(ItemError::UnsupportedReceiver),
            syn::FnArg::Typed(pt) => pt,
        };
        let syn::Pat::Ident(pat) = &*pt.pat else {
            return Err(ItemError::UnsupportedParamPattern {
                pattern: pt.pat.to_token_stream().to_string(),
            });
        };
        let name = pat.ident.clone();
        let ty = lower_type(&pt.ty, consts, item_crate).map_err(|source| ItemError::ParamType {
            param: name.clone(),
            source,
        })?;
        params.push(Param {
            name,
            ty,
            syntax: pt.clone(),
        });
    }
    // An elided return and a written `-> ()` are the same function. The model
    // says so once, here, instead of leaving every consumer to normalize one to
    // the other — which is what they all do today, in eight separate copies.
    let ret = match &f.sig.output {
        syn::ReturnType::Default => Type {
            kind: TypeKind::Unit,
            syntax: syn::parse_quote!(()),
        },
        syn::ReturnType::Type(_, t) => {
            lower_type(t, consts, item_crate).map_err(|source| ItemError::ReturnType { source })?
        }
    };
    Ok(Function {
        name: f.sig.ident.clone(),
        params,
        ret,
        location: loc.clone(),
        syntax: f.clone(),
    })
}

fn lower_struct(
    s: &syn::ItemStruct,
    loc: &SourceLocation,
    consts: &ConstIndex,
    item_crate: Option<&str>,
) -> Result<Struct, ItemError> {
    let fields = match &s.fields {
        syn::Fields::Named(named) => {
            let mut out = Vec::with_capacity(named.named.len());
            for (index, f) in named.named.iter().enumerate() {
                let name = f.ident.clone().expect("named fields have idents");
                let ty = lower_type(&f.ty, consts, item_crate).map_err(|source| {
                    ItemError::FieldType {
                        field: name.clone(),
                        source,
                    }
                })?;
                out.push(Field {
                    name: Some(name),
                    index,
                    ty,
                    syntax: f.clone(),
                });
            }
            Some(out)
        }
        // Opaque: a tuple struct's contents are not a boundary surface, so they
        // are not lowered and a field type outside the grammar is not an error.
        syn::Fields::Unnamed(_) => None,
        syn::Fields::Unit => Some(Vec::new()),
    };
    Ok(Struct {
        name: s.ident.clone(),
        fields,
        location: loc.clone(),
        syntax: s.clone(),
    })
}

fn lower_enum(
    e: &syn::ItemEnum,
    loc: &SourceLocation,
    consts: &ConstIndex,
    item_crate: Option<&str>,
) -> Result<Enum, ItemError> {
    let mut variants = Vec::with_capacity(e.variants.len());
    // Rust's own numbering rule: an explicit `= N` sets the value, an implicit
    // variant takes the previous plus one, starting at 0.
    let mut next: Option<i64> = Some(0);
    for (tag, v) in e.variants.iter().enumerate() {
        let discriminant = match v.discriminant.as_ref() {
            Some((_, expr)) => int_literal(expr),
            None => next,
        };
        // `checked_add`: a discriminant at the top of the range is valid Rust
        // (`#[repr(u64)] enum E { A = i64::MAX as u64, B }`), so running out of
        // `i64` ends the numeric chain exactly as an unevaluable spelling does.
        // The spelling is untouched either way — it is in `Variant::syntax`.
        next = discriminant.and_then(|n| n.checked_add(1));

        let mut fields = Vec::with_capacity(v.fields.len());
        for (index, f) in v.fields.iter().enumerate() {
            let ty = lower_type(&f.ty, consts, item_crate).map_err(|source| {
                ItemError::VariantFieldType {
                    variant: v.ident.clone(),
                    field: match &f.ident {
                        Some(id) => id.to_string(),
                        None => index.to_string(),
                    },
                    source,
                }
            })?;
            fields.push(Field {
                name: f.ident.clone(),
                index,
                ty,
                syntax: f.clone(),
            });
        }
        variants.push(Variant {
            name: v.ident.clone(),
            tag: tag as i32,
            discriminant,
            fields,
            syntax: v.clone(),
        });
    }
    Ok(Enum {
        name: e.ident.clone(),
        variants,
        location: loc.clone(),
        syntax: e.clone(),
    })
}

/// Pull a signed integer out of a literal expression (`5`, `-3`, `0x07`).
/// `None` for anything else — a `const`, a path, arithmetic.
fn int_literal(expr: &syn::Expr) -> Option<i64> {
    match expr {
        syn::Expr::Lit(lit) => match &lit.lit {
            syn::Lit::Int(int) => int.base10_parse::<i64>().ok(),
            _ => None,
        },
        syn::Expr::Unary(syn::ExprUnary {
            op: syn::UnOp::Neg(_),
            expr,
            ..
        }) => int_literal(expr).map(|v| -v),
        _ => None,
    }
}
