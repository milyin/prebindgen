//! The language-neutral source model: what a captured Rust type **means**.
//!
//! This is the beginning of issue #211's `SourceModel`. Its purpose is stated
//! there and is worth restating, because it is what every design choice here
//! answers to:
//!
//! > Language adapters do not parse captured source or keep parallel
//! > whitelists/classifiers for source syntax. Emitters do not recover semantic
//! > facts by re-reading raw source syntax.
//!
//! So a fact an adapter needs is a **node in this model**, never something it
//! re-derives from `syn`. The array extent is the worked example: a C header has
//! to be able to emit `uint8_t tag[MARKER_TAG_LEN]`, and the only way for the
//! adapter to know that the extent was written as a named const — rather than
//! guessing from syntax it should not be reading — is for the model to say so.
//!
//! [`SourceType`] also *is* the answer to the questions adapters currently ask
//! `syn` themselves: `is_scalar`, `is_string`, `is_vec`, `is_option`,
//! `box_inner`, `pat_match_top`. Those become `match` arms on a closed enum.
//! Deleting the duplicates is stages F5/F6 of the umbrella (#215); this module
//! is what makes it possible.
//!
//! # Boundary today
//!
//! * **Types** — complete. [`lower_type`] is total over the grammar in
//!   `docs/source-language.md`; anything else is a frontend error.
//! * **Items** — structs only ([`SourceStruct`]). Functions and enums keep
//!   their `syn` items for now, which #211 step 4 explicitly allows while
//!   adapters migrate.
//! * **Adapters** — only cbindgen's struct-field path consumes this. Everything
//!   else reads the `syn::Type` projection ([`SourceType::to_syn`]).

use std::fmt;

use quote::ToTokens;

use super::array_len::{lower_array_len, ConstIndex};

#[cfg(test)]
mod tests;

/// A Rust type as the frontend decided it.
///
/// The variants are the accepted type grammar: a form with no variant here is a
/// form the prebindgen source language does not accept. Acceptance is therefore
/// a consequence of lowering, the same contract [`lower_array_len`] established
/// for extents.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SourceType {
    /// A primitive with a fixed C/JVM counterpart.
    Scalar(ScalarKind),
    /// `String` — an owned, heap-allocated UTF-8 string.
    Str,
    /// `Option<T>`.
    Optional(Box<SourceType>),
    /// `Vec<T>`.
    Sequence(Box<SourceType>),
    /// `Box<T>`.
    Boxed(Box<SourceType>),
    /// `Result<T, E>`.
    Fallible {
        ok: Box<SourceType>,
        err: Box<SourceType>,
    },
    /// Any other named type: a `#[prebindgen]` struct or enum, or a foreign
    /// path.
    ///
    /// `name` is the normalized spelling WITHOUT generic arguments — the type's
    /// identity, not syntax to re-inspect; the arguments are modeled in `args`.
    /// A lifetime argument survives in `name` alone, since it is part of the
    /// spelling but carries no structure.
    Named {
        name: syn::Path,
        args: Vec<SourceType>,
    },
    /// `[T; N]`. The extent carries how it was written — see [`ArrayExtent`].
    Array {
        elem: Box<SourceType>,
        extent: ArrayExtent,
    },
    /// `&T` / `&mut T`.
    Ref {
        mutable: bool,
        inner: Box<SourceType>,
    },
    /// `[T]`, only ever behind a [`SourceType::Ref`].
    Slice(Box<SourceType>),
    /// `(A, B, …)`; the empty tuple is [`SourceType::Unit`].
    Tuple(Vec<SourceType>),
    /// `*const T` / `*mut T`.
    Ptr {
        mutable: bool,
        inner: Box<SourceType>,
    },
    /// `impl Fn(A, B, …) + Send + Sync + 'static` — the callback form.
    Callback { args: Vec<SourceType> },
    /// `()`.
    Unit,
}

/// The primitives the source language accepts. Mirrors the set every adapter
/// already treats as directly representable.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScalarKind {
    Bool,
    I8,
    I16,
    I32,
    I64,
    Isize,
    U8,
    U16,
    U32,
    U64,
    Usize,
    F32,
    F64,
}

impl ScalarKind {
    fn from_name(name: &str) -> Option<Self> {
        Some(match name {
            "bool" => Self::Bool,
            "i8" => Self::I8,
            "i16" => Self::I16,
            "i32" => Self::I32,
            "i64" => Self::I64,
            "isize" => Self::Isize,
            "u8" => Self::U8,
            "u16" => Self::U16,
            "u32" => Self::U32,
            "u64" => Self::U64,
            "usize" => Self::Usize,
            "f32" => Self::F32,
            "f64" => Self::F64,
            _ => return None,
        })
    }

    /// The Rust spelling — the identity this was lowered from.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Bool => "bool",
            Self::I8 => "i8",
            Self::I16 => "i16",
            Self::I32 => "i32",
            Self::I64 => "i64",
            Self::Isize => "isize",
            Self::U8 => "u8",
            Self::U16 => "u16",
            Self::U32 => "u32",
            Self::U64 => "u64",
            Self::Usize => "usize",
            Self::F32 => "f32",
            Self::F64 => "f64",
        }
    }

    fn to_syn(self) -> syn::Type {
        let ident = syn::Ident::new(self.as_str(), proc_macro2::Span::call_site());
        syn::parse_quote!(#ident)
    }
}

/// A fixed-size array's extent: its value, and how the source wrote it.
///
/// Both halves are load-bearing and belong to **different consumers**:
///
/// * `value` is the semantic length. It is what makes `[u8; A]` and `[u8; 4]`
///   one Rust type, one [`TypeKey`](crate::api::core::TypeKey) and one
///   converter, and it is what a generator needs when a destination language
///   has no way to reference a Rust const — a Kotlin surface that groups a
///   small array into scalars needs the count as a number.
/// * `source` is how it was written. A C header must be able to emit
///   `uint8_t tag[MARKER_TAG_LEN]`, because the symbolic extent is part of that
///   API's meaning: changing the size is then one edit rather than a hunt
///   through literals.
///
/// This lives on the **use site** — a field of a [`SourceStruct`] — and never
/// on anything keyed by type. Three fields whose extents are equal share one
/// `TypeKey`, so a type-keyed table could only report whichever of them was
/// stored last.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArrayExtent {
    pub value: usize,
    pub source: ExtentSource,
}

/// How an [`ArrayExtent`] was spelled at its use site.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExtentSource {
    /// Written as an integer literal — `[u8; 4]`.
    Literal,
    /// Written as the name of a `#[prebindgen]` const — `[u8; MARKER_TAG_LEN]`.
    Const(ConstId),
}

/// A `#[prebindgen]` const, identified the way the flat namespace identifies
/// everything: by name, plus the crate it was marked in.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConstId {
    pub name: String,
    /// Crate the const was marked in; `None` for an origin-less stream.
    pub origin: Option<String>,
}

impl ArrayExtent {
    /// The extent as generated Rust spells it by default: the number.
    ///
    /// An adapter that wants the symbolic form reads [`Self::source`] and builds
    /// its own — that is a spelling choice, and spelling choices are the
    /// adapter's half of the boundary.
    pub fn to_expr(&self) -> syn::Expr {
        let lit = syn::LitInt::new(&self.value.to_string(), proc_macro2::Span::call_site());
        syn::Expr::Lit(syn::ExprLit {
            attrs: Vec::new(),
            lit: syn::Lit::Int(lit),
        })
    }

    /// The const this extent named, if it named one.
    pub fn const_id(&self) -> Option<&ConstId> {
        match &self.source {
            ExtentSource::Literal => None,
            ExtentSource::Const(id) => Some(id),
        }
    }
}

/// A type the prebindgen source language does not accept.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnsupportedType {
    /// The offending type as written.
    pub offending: String,
    pub reason: UnsupportedTypeReason,
}

/// Why [`lower_type`] refused a type.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UnsupportedTypeReason {
    /// A syntactic form with no place in the language: a bare trait object, a
    /// closure type, a macro, `Self`, a never type, an inferred type.
    UnsupportedForm,
    /// `impl Trait` that is not `Fn(..) + Send + Sync + 'static`.
    DisallowedImplTrait,
    /// A generic that takes a fixed arity and did not get it — `Option` with no
    /// argument, `Result` with one.
    WrongGenericArity { expected: usize },
    /// A lifetime or const generic argument in a position the model does not
    /// represent.
    UnsupportedGenericArgument,
    /// The array's extent — see
    /// [`ArrayLenReason`](super::ArrayLenReason).
    BadArrayExtent(Box<super::UnsupportedArrayLen>),
}

impl fmt::Display for UnsupportedType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.reason {
            UnsupportedTypeReason::BadArrayExtent(e) => return write!(f, "{e}"),
            UnsupportedTypeReason::UnsupportedForm => write!(
                f,
                "type `{}` is a form the prebindgen source language does not accept",
                self.offending
            ),
            UnsupportedTypeReason::DisallowedImplTrait => write!(
                f,
                "type `{}` is an `impl Trait` other than \
                 `impl Fn(..) + Send + Sync + 'static`, the only one supported",
                self.offending
            ),
            UnsupportedTypeReason::WrongGenericArity { expected } => write!(
                f,
                "type `{}` needs exactly {expected} type argument(s)",
                self.offending
            ),
            UnsupportedTypeReason::UnsupportedGenericArgument => write!(
                f,
                "type `{}` has a generic argument that is not a type",
                self.offending
            ),
        }?;
        write!(f, " — see docs/source-language.md for the accepted grammar")
    }
}

impl std::error::Error for UnsupportedType {}

/// One field of a [`SourceStruct`].
#[derive(Clone, Debug)]
pub struct SourceField {
    pub name: syn::Ident,
    pub ty: SourceType,
}

/// A `#[prebindgen]` struct, as the frontend decided it.
///
/// Named fields only — the one struct shape whose fields are a boundary
/// surface. A tuple struct is still indexed (it can be an opaque handle) but
/// has no modeled fields.
#[derive(Clone, Debug, Default)]
pub struct SourceStruct {
    pub fields: Vec<SourceField>,
}

/// Lower a captured type to its model.
///
/// **Total over the accepted grammar**: `Ok` means every part of the type was
/// understood, so a form this function does not lower is a form the language
/// does not accept. There is no separate acceptance list to drift from — the
/// same contract, and for the same reason, as [`lower_array_len`].
///
/// `item_crate` is the origin of the item the type was written in; an extent
/// must name a const from that same crate.
pub(crate) fn lower_type(
    ty: &syn::Type,
    consts: &ConstIndex,
    item_crate: Option<&str>,
) -> Result<SourceType, UnsupportedType> {
    let fail = |reason| UnsupportedType {
        offending: ty.to_token_stream().to_string(),
        reason,
    };
    match ty {
        syn::Type::Group(g) => lower_type(&g.elem, consts, item_crate),
        syn::Type::Paren(p) => lower_type(&p.elem, consts, item_crate),
        syn::Type::Reference(r) => Ok(SourceType::Ref {
            mutable: r.mutability.is_some(),
            inner: Box::new(lower_type(&r.elem, consts, item_crate)?),
        }),
        syn::Type::Slice(s) => Ok(SourceType::Slice(Box::new(lower_type(
            &s.elem, consts, item_crate,
        )?))),
        syn::Type::Ptr(p) => Ok(SourceType::Ptr {
            mutable: p.mutability.is_some(),
            inner: Box::new(lower_type(&p.elem, consts, item_crate)?),
        }),
        syn::Type::Tuple(t) if t.elems.is_empty() => Ok(SourceType::Unit),
        syn::Type::Tuple(t) => Ok(SourceType::Tuple(
            t.elems
                .iter()
                .map(|e| lower_type(e, consts, item_crate))
                .collect::<Result<_, _>>()?,
        )),
        syn::Type::Array(a) => {
            let rendered = a.to_token_stream().to_string();
            let extent = lower_array_len(&a.len, &rendered, item_crate, consts)
                .map_err(|e| fail(UnsupportedTypeReason::BadArrayExtent(Box::new(e))))?;
            Ok(SourceType::Array {
                elem: Box::new(lower_type(&a.elem, consts, item_crate)?),
                extent,
            })
        }
        syn::Type::ImplTrait(_) => match crate::api::core::registry::extract_fn_trait_args(ty) {
            Some(args) => Ok(SourceType::Callback {
                args: args
                    .iter()
                    .map(|a| lower_type(a, consts, item_crate))
                    .collect::<Result<_, _>>()?,
            }),
            None => Err(fail(UnsupportedTypeReason::DisallowedImplTrait)),
        },
        syn::Type::Path(tp) => lower_path(ty, tp, consts, item_crate),
        _ => Err(fail(UnsupportedTypeReason::UnsupportedForm)),
    }
}

fn lower_path(
    ty: &syn::Type,
    tp: &syn::TypePath,
    consts: &ConstIndex,
    item_crate: Option<&str>,
) -> Result<SourceType, UnsupportedType> {
    let fail = |reason| UnsupportedType {
        offending: ty.to_token_stream().to_string(),
        reason,
    };
    // A qualified self (`<T as Trait>::Assoc`) is never normalized and has no
    // modeled meaning; its spelling is its identity, so it stays a named type.
    let Some(last) = tp.path.segments.last() else {
        return Err(fail(UnsupportedTypeReason::UnsupportedForm));
    };
    let name = last.ident.to_string();
    let args: Vec<SourceType> = match &last.arguments {
        syn::PathArguments::None => Vec::new(),
        syn::PathArguments::AngleBracketed(ab) => {
            let mut out = Vec::new();
            for a in &ab.args {
                match a {
                    syn::GenericArgument::Type(t) => out.push(lower_type(t, consts, item_crate)?),
                    // A lifetime is part of a type's identity but carries no
                    // structure to model; it survives in `name`'s spelling.
                    syn::GenericArgument::Lifetime(_) => {}
                    _ => return Err(fail(UnsupportedTypeReason::UnsupportedGenericArgument)),
                }
            }
            out
        }
        syn::PathArguments::Parenthesized(_) => {
            return Err(fail(UnsupportedTypeReason::UnsupportedForm))
        }
    };

    let arity = |n: usize| {
        if args.len() == n {
            Ok(())
        } else {
            Err(fail(UnsupportedTypeReason::WrongGenericArity {
                expected: n,
            }))
        }
    };
    let one = |mut args: Vec<SourceType>| Box::new(args.remove(0));

    // A bare primitive, before anything else. Guarded on `qself`/args so a
    // user type that happens to end in `u8` cannot be mistaken for one.
    if tp.qself.is_none() && tp.path.segments.len() == 1 && args.is_empty() {
        if let Some(kind) = ScalarKind::from_name(&name) {
            return Ok(SourceType::Scalar(kind));
        }
    }
    // Only bare / prelude-normalized spellings reach here: `normalize_type` has
    // already reduced `std::option::Option` and friends at ingest.
    match name.as_str() {
        "String" if args.is_empty() => Ok(SourceType::Str),
        "Option" => {
            arity(1)?;
            Ok(SourceType::Optional(one(args)))
        }
        "Vec" => {
            arity(1)?;
            Ok(SourceType::Sequence(one(args)))
        }
        "Box" => {
            arity(1)?;
            Ok(SourceType::Boxed(one(args)))
        }
        "Result" => {
            arity(2)?;
            let mut args = args;
            let err = Box::new(args.remove(1));
            let ok = Box::new(args.remove(0));
            Ok(SourceType::Fallible { ok, err })
        }
        _ => {
            // The name is the type's IDENTITY and nothing else: the generic
            // arguments live in `args`, modeled, so keeping a copy of them in
            // the path too would be two representations of one fact.
            let mut name = tp.path.clone();
            if let Some(last) = name.segments.last_mut() {
                last.arguments = syn::PathArguments::None;
            }
            Ok(SourceType::Named { name, args })
        }
    }
}

impl SourceType {
    /// The `syn::Type` this models — the **semantic** form, with every array
    /// extent as its number.
    ///
    /// This is the projection everything unmigrated consumes, `TypeKey`
    /// included, which is why `[u8; A]` and `[u8; 4]` stay one type and one
    /// converter. It is a projection, not a second source of truth: the model
    /// is what the frontend stores, and the `syn` item's field types are
    /// written from this.
    pub fn to_syn(&self) -> syn::Type {
        match self {
            SourceType::Scalar(k) => k.to_syn(),
            SourceType::Str => syn::parse_quote!(String),
            SourceType::Optional(inner) => {
                let t = inner.to_syn();
                syn::parse_quote!(Option<#t>)
            }
            SourceType::Sequence(inner) => {
                let t = inner.to_syn();
                syn::parse_quote!(Vec<#t>)
            }
            SourceType::Boxed(inner) => {
                let t = inner.to_syn();
                syn::parse_quote!(Box<#t>)
            }
            SourceType::Fallible { ok, err } => {
                let (o, e) = (ok.to_syn(), err.to_syn());
                syn::parse_quote!(Result<#o, #e>)
            }
            SourceType::Named { name, args } => {
                let mut path = name.clone();
                if !args.is_empty() {
                    if let Some(last) = path.segments.last_mut() {
                        let ts: Vec<syn::Type> = args.iter().map(|a| a.to_syn()).collect();
                        let ab: syn::AngleBracketedGenericArguments = syn::parse_quote!(<#(#ts),*>);
                        last.arguments = syn::PathArguments::AngleBracketed(ab);
                    }
                }
                syn::Type::Path(syn::TypePath { qself: None, path })
            }
            SourceType::Array { elem, extent } => {
                let (t, n) = (elem.to_syn(), extent.to_expr());
                syn::parse_quote!([#t; #n])
            }
            SourceType::Ref { mutable, inner } => {
                let t = inner.to_syn();
                if *mutable {
                    syn::parse_quote!(&mut #t)
                } else {
                    syn::parse_quote!(&#t)
                }
            }
            SourceType::Slice(inner) => {
                let t = inner.to_syn();
                syn::parse_quote!([#t])
            }
            SourceType::Tuple(elems) => {
                let ts: Vec<syn::Type> = elems.iter().map(|e| e.to_syn()).collect();
                syn::parse_quote!((#(#ts),*))
            }
            SourceType::Ptr { mutable, inner } => {
                let t = inner.to_syn();
                if *mutable {
                    syn::parse_quote!(*mut #t)
                } else {
                    syn::parse_quote!(*const #t)
                }
            }
            SourceType::Callback { args } => {
                let ts: Vec<syn::Type> = args.iter().map(|a| a.to_syn()).collect();
                syn::parse_quote!(impl Fn(#(#ts),*) + Send + Sync + 'static)
            }
            SourceType::Unit => syn::parse_quote!(()),
        }
    }

    /// The extent of this type when it is an array, else `None`. The one fact
    /// an adapter cannot get from [`Self::to_syn`], and the reason this model
    /// exists.
    pub fn array_extent(&self) -> Option<&ArrayExtent> {
        match self {
            SourceType::Array { extent, .. } => Some(extent),
            _ => None,
        }
    }

    /// Every extent reachable from this type, outermost first — so a nested
    /// `[[u8; A]; B]` yields `B` then `A`.
    ///
    /// Used to find which consts an emitted C type may name, and therefore
    /// which must reach the header as a `#define`.
    pub fn extents(&self) -> Vec<&ArrayExtent> {
        let mut out = Vec::new();
        self.collect_extents(&mut out);
        out
    }

    fn collect_extents<'a>(&'a self, out: &mut Vec<&'a ArrayExtent>) {
        match self {
            SourceType::Array { elem, extent } => {
                out.push(extent);
                elem.collect_extents(out);
            }
            SourceType::Optional(t)
            | SourceType::Sequence(t)
            | SourceType::Boxed(t)
            | SourceType::Slice(t)
            | SourceType::Ref { inner: t, .. }
            | SourceType::Ptr { inner: t, .. } => t.collect_extents(out),
            SourceType::Fallible { ok, err } => {
                ok.collect_extents(out);
                err.collect_extents(out);
            }
            SourceType::Tuple(ts) | SourceType::Callback { args: ts } => {
                ts.iter().for_each(|t| t.collect_extents(out))
            }
            SourceType::Named { args, .. } => args.iter().for_each(|t| t.collect_extents(out)),
            SourceType::Scalar(_) | SourceType::Str | SourceType::Unit => {}
        }
    }
}
