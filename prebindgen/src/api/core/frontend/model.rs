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
    /// `path` carries the type's **identity** — leading `::`, every segment,
    /// and the final ident — with the last segment's generic arguments stripped
    /// out into `args`. Keeping them in both places would be two
    /// representations of one fact.
    ///
    /// `args` preserves the arguments **in source order**, lifetimes included,
    /// which is what makes the projection reconstruct `Foo<'a, T>` exactly. A
    /// lifetime is part of the spelling and nothing more: it is carried
    /// verbatim, never modeled as structure, because it means nothing to a
    /// destination language.
    Named {
        path: syn::TypePath,
        args: Vec<NamedArg>,
    },
    /// `[T; N]`. The extent carries how it was written — see [`ArrayExtent`].
    Array {
        elem: Box<SourceType>,
        extent: ArrayExtent,
    },
    /// `&T` / `&'a T` / `&mut T`.
    Ref {
        lifetime: Option<syn::Lifetime>,
        mutable: bool,
        inner: Box<SourceType>,
    },
    /// `[T]`, only ever behind a [`SourceType::Ref`].
    Slice(Box<SourceType>),
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

/// One generic argument of a [`SourceType::Named`], in source order.
///
/// A lifetime is kept verbatim rather than modeled: it is part of the type's
/// spelling — `Foo<'static>` is not `Foo` — but carries nothing a destination
/// language can act on.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NamedArg {
    Lifetime(syn::Lifetime),
    Type(SourceType),
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
    /// `impl Trait` that is not an accepted callback — see
    /// [`CallbackReject`](crate::api::core::registry::CallbackReject), which
    /// says whether the form is reserved for a future release or cannot work
    /// at all.
    DisallowedImplTrait(crate::api::core::registry::CallbackReject),
    /// A generic that takes a fixed arity and did not get it — `Option` with no
    /// argument, `Result` with one.
    WrongGenericArity { expected: usize },
    /// A non-empty tuple. Only `()` is in the language: no adapter has ever
    /// lowered a tuple, so accepting one would defer the failure to a late
    /// "unresolved type" instead of naming it here.
    UnsupportedTuple,
    /// A path with a qualified self — `<T as Trait>::Assoc`.
    ///
    /// The frontend never captures `impl` blocks, so it cannot know what an
    /// associated type resolves to; carrying the spelling would only move the
    /// failure downstream.
    AssociatedType,
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
            UnsupportedTypeReason::DisallowedImplTrait(reason) => write!(
                f,
                "type `{}` is not an accepted callback — {}",
                self.offending,
                reason.describe()
            ),
            UnsupportedTypeReason::WrongGenericArity { expected } => write!(
                f,
                "type `{}` needs exactly {expected} type argument(s)",
                self.offending
            ),
            UnsupportedTypeReason::UnsupportedTuple => write!(
                f,
                "type `{}` is a tuple; only the unit `()` is supported — return the \
                 components separately, or wrap them in a `#[prebindgen]` struct",
                self.offending
            ),
            UnsupportedTypeReason::AssociatedType => write!(
                f,
                "type `{}` is an associated type; `#[prebindgen]` never captures `impl` \
                 blocks, so its resolution is unknowable here — name the concrete type",
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

/// A `#[prebindgen]` enum, as the frontend decided it: a **tag** — which
/// alternative is live — plus one **field group per variant**.
///
/// One model covers both enum shapes on purpose. A fieldless enum is the
/// degenerate sum whose every group is empty, so a lowering written for the
/// general case collapses to "just a tag" for it. The distinction survives only
/// as [`Self::is_unit`], because the *declarators* differ (`enum_class!` /
/// `.enum_type()` accept only the degenerate case) and a refusal should name
/// the right declarator rather than assert on `syn::Fields`.
///
/// Core describes the sum; an adapter decides what its groups look like on the
/// wire — jnigen overlays them in the signature, cbindgen overlays them in
/// memory as a `#[repr(C)]` union. Nothing here names a wire detail.
#[derive(Clone, Debug)]
pub struct SourceEnum {
    /// The enum's ident as declared — the spelling adapters use to build
    /// `Enum::Variant` constructor paths.
    pub name: syn::Ident,
    /// Variants in declaration order; `variants[i].tag == i as i32`.
    pub variants: Vec<SourceVariant>,
}

impl SourceEnum {
    /// True when every variant is fieldless, i.e. the value is exactly its
    /// discriminant.
    pub fn is_unit(&self) -> bool {
        self.variants.iter().all(SourceVariant::is_unit)
    }

    /// The first payload-carrying variant — the offender an adapter names when
    /// refusing a sum where only a fieldless enum is accepted.
    pub fn first_payload_variant(&self) -> Option<&SourceVariant> {
        self.variants.iter().find(|v| !v.is_unit())
    }

    /// Every variant paired with the value Rust assigns it.
    ///
    /// This is the numbering a destination language needs when it has no way to
    /// reference a Rust constant: a Kotlin `enum class` entry is `NAME(3)`, and
    /// the generated `int → variant` decode matches on the same numbers, so
    /// both come from here and cannot drift.
    ///
    /// `Err` names the first variant whose discriminant the frontend could not
    /// evaluate. That is a refusal for *this* consumer only — a C mirror
    /// re-emits the spelling and never asks. See [`Discriminant`].
    pub fn discriminant_values(&self) -> Result<Vec<(&syn::Ident, i64)>, &syn::Ident> {
        self.variants
            .iter()
            .map(|v| match v.discriminant.value {
                Some(n) => Ok((&v.name, n)),
                None => Err(&v.name),
            })
            .collect()
    }
}

/// One alternative of a [`SourceEnum`].
#[derive(Clone, Debug)]
pub struct SourceVariant {
    /// The variant ident as declared (`PeriodicQueries`).
    pub name: syn::Ident,
    /// Declaration-order tag, `0..N-1`.
    ///
    /// This is **not** the discriminant. A sum's alternatives are identified by
    /// position, because the payload mirror an adapter builds carries no `repr`
    /// and numbers its arms itself; see [`Self::discriminant`] for the other
    /// numbering.
    pub tag: i32,
    /// The value Rust itself assigns this variant. Distinct from [`Self::tag`]:
    /// this one is observable from outside the binding, and only a fieldless
    /// enum crosses as it.
    pub discriminant: Discriminant,
    /// The variant's payload, in declaration order. Empty for a unit variant —
    /// the group that contributes nothing but its tag.
    pub fields: Vec<SourceVariantField>,
    /// How the variant is **written**, taken from `syn::Fields` at lowering.
    ///
    /// Not derivable from [`Self::fields`]: `B()` and `C {}` have no payload
    /// and still must be spelled `E::B()` / `E::C {}` wherever Rust names
    /// them, so the emptiness of the group and the delimiters around it are
    /// two different facts. Use [`Self::is_unit`] for the first,
    /// [`VariantShape::spell`] for the second.
    pub shape: VariantShape,
}

impl SourceVariant {
    /// True when this variant carries no payload — the group question, not the
    /// syntax one. `B()` is unit by this test and still [`VariantShape::Tuple`].
    pub fn is_unit(&self) -> bool {
        self.fields.is_empty()
    }
}

/// How a [`SourceVariant`] is written.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VariantShape {
    /// No delimiters — `V`.
    Unit,
    /// Positional payload — `V(A, B)`, including the empty `V()`.
    Tuple,
    /// Named payload — `V { a: A }`, including the empty `V {}`.
    Named,
}

impl VariantShape {
    /// Spell one variant: `head`, `head(parts…)` or `head { parts… }`.
    ///
    /// The single place the delimiters are chosen, for patterns and
    /// constructors alike and in either direction — `head` is the variant's
    /// path and each part is an already-rendered `bind` / `member: bind`.
    pub fn spell(
        self,
        head: proc_macro2::TokenStream,
        parts: &[proc_macro2::TokenStream],
    ) -> proc_macro2::TokenStream {
        match self {
            VariantShape::Unit => head,
            VariantShape::Tuple => quote::quote!(#head(#(#parts),*)),
            VariantShape::Named => quote::quote!(#head { #(#parts),* }),
        }
    }
}

/// One payload field of a [`SourceVariant`].
#[derive(Clone, Debug)]
pub struct SourceVariantField {
    /// How the field is addressed in a pattern: `Named(ident)` for a struct
    /// variant, `Unnamed(index)` for a tuple variant.
    pub member: syn::Member,
    /// Leaf name, following the nested-prefix convention:
    /// `<variant_snake>_<field>` for a named field, `<variant_snake>_<i>` for a
    /// tuple field.
    pub leaf_name: String,
    pub ty: SourceType,
}

/// A variant's discriminant: the value Rust assigns it, and how the source
/// wrote it.
///
/// Both halves are load-bearing, and they belong to **different consumers** —
/// exactly the split [`ArrayExtent`] makes for an extent:
///
/// * `value` is the number. A destination language with no way to reference a
///   Rust constant needs it: a Kotlin `enum class` entry is `NAME(3)`, and the
///   generated `jint → variant` decode matches on it.
/// * `source` is the spelling. A C mirror re-emits it **verbatim**, which is
///   what lets a `const`- or `cfg`-driven discriminant keep working, and what
///   lets a value outside `i64` exist at all.
///
/// `value` is therefore `None` whenever the spelling is something the frontend
/// cannot evaluate at generation time. That is not a failure here: a consumer
/// that needs the number refuses, naming the variant, while one that only
/// re-emits the spelling carries on.
#[derive(Clone, Debug)]
pub struct Discriminant {
    /// Rust's own assignment rule: an explicit `= N` sets the value, an
    /// implicit variant takes the previous value plus one, starting at 0.
    /// `None` once an unevaluable spelling has broken the chain.
    pub value: Option<i64>,
    pub source: DiscriminantSource,
}

/// How a [`Discriminant`] was spelled at its declaration site.
#[derive(Clone, Debug)]
pub enum DiscriminantSource {
    /// Written with no `=` — the value follows from position.
    Implicit,
    /// Written as `= <expr>`, kept **exactly as spelled**.
    ///
    /// The expression is carried rather than re-rendered from
    /// [`Discriminant::value`] because a C mirror re-emits it verbatim: `= 0x07`
    /// must stay `0x07`, and a `const`- or `cfg`-driven value has no number to
    /// render from in the first place.
    ///
    /// This is the model's one carrier of open syntax, and it is deliberate.
    /// Narrowing it to a closed representation would delete support the C
    /// backend has today. It is the same class of leak #211's stage F7 tracks
    /// for `Niches`, and it is listed there.
    Explicit(syn::Expr),
}

/// Lower a captured enum to its model.
///
/// Same contract as [`lower_type`]: every variant field type goes through it,
/// so an enum whose payload the language does not accept is refused here rather
/// than reaching an adapter. The `Err` names the offending variant and field.
///
/// A discriminant is never a reason to refuse — see [`Discriminant`]. An
/// unevaluable spelling yields `value: None` and breaks the implicit chain for
/// the variants after it, which is the honest answer: Rust would still compile
/// them, and only a consumer that needs the number is affected.
pub(crate) fn lower_enum(
    e: &syn::ItemEnum,
    consts: &ConstIndex,
    item_crate: Option<&str>,
) -> Result<SourceEnum, (syn::Ident, Option<syn::Ident>, UnsupportedType)> {
    let mut variants = Vec::with_capacity(e.variants.len());
    let mut next: Option<i64> = Some(0);
    for (i, v) in e.variants.iter().enumerate() {
        let (value, source) = match v.discriminant.as_ref() {
            Some((_, expr)) => (
                int_literal(expr),
                DiscriminantSource::Explicit(expr.clone()),
            ),
            None => (next, DiscriminantSource::Implicit),
        };
        // `checked_add`: a discriminant at the top of the range is valid Rust
        // (`#[repr(u64)] enum E { A = i64::MAX as u64, B }`), so overflow ends
        // the numeric chain the same way an unevaluable spelling does. The
        // spelling itself is untouched, and a C mirror still re-emits it.
        next = value.and_then(|n| n.checked_add(1));
        let prefix = crate::api::core::types_util::pascal_to_snake(&v.ident.to_string());
        let mut fields = Vec::with_capacity(v.fields.len());
        for (fi, f) in v.fields.iter().enumerate() {
            let ty = lower_type(&f.ty, consts, item_crate)
                .map_err(|e| (v.ident.clone(), f.ident.clone(), e))?;
            let (member, leaf_name) = match &f.ident {
                Some(id) => (syn::Member::Named(id.clone()), format!("{prefix}_{id}")),
                None => (
                    syn::Member::Unnamed(syn::Index::from(fi)),
                    format!("{prefix}_{fi}"),
                ),
            };
            fields.push(SourceVariantField {
                member,
                leaf_name,
                ty,
            });
        }
        variants.push(SourceVariant {
            name: v.ident.clone(),
            tag: i as i32,
            discriminant: Discriminant { value, source },
            fields,
            shape: match v.fields {
                syn::Fields::Unit => VariantShape::Unit,
                syn::Fields::Unnamed(_) => VariantShape::Tuple,
                syn::Fields::Named(_) => VariantShape::Named,
            },
        });
    }
    Ok(SourceEnum {
        name: e.ident.clone(),
        variants,
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
            lifetime: r.lifetime.clone(),
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
        // Only the unit is in the language. Refusing here names the type;
        // accepting would defer the failure to an "unresolved type" much later.
        syn::Type::Tuple(_) => Err(fail(UnsupportedTypeReason::UnsupportedTuple)),
        syn::Type::Array(a) => {
            let rendered = a.to_token_stream().to_string();
            let extent = lower_array_len(&a.len, &rendered, item_crate, consts)
                .map_err(|e| fail(UnsupportedTypeReason::BadArrayExtent(Box::new(e))))?;
            Ok(SourceType::Array {
                elem: Box::new(lower_type(&a.elem, consts, item_crate)?),
                extent,
            })
        }
        // The callback gate lives in `registry` because it is also the live
        // acceptance check for function parameters, which are not modeled yet.
        // Mapping its reason through keeps ONE authority.
        syn::Type::ImplTrait(_) => match crate::api::core::registry::extract_fn_trait_sig(ty) {
            Ok(args) => Ok(SourceType::Callback {
                args: args
                    .iter()
                    .map(|a| lower_type(a, consts, item_crate))
                    .collect::<Result<_, _>>()?,
            }),
            Err(reason) => Err(fail(UnsupportedTypeReason::DisallowedImplTrait(reason))),
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
    // An associated type is refused rather than carried: the frontend never
    // captures `impl` blocks, so what `<T as Trait>::Assoc` resolves to is
    // unknowable here, and keeping the spelling would only move the failure
    // downstream.
    if tp.qself.is_some() {
        return Err(fail(UnsupportedTypeReason::AssociatedType));
    }
    let Some(last) = tp.path.segments.last() else {
        return Err(fail(UnsupportedTypeReason::UnsupportedForm));
    };
    let name = last.ident.to_string();

    // Arguments in SOURCE ORDER, lifetimes kept verbatim. Order and lifetimes
    // are what make the projection reconstruct the type exactly.
    let args: Vec<NamedArg> = match &last.arguments {
        syn::PathArguments::None => Vec::new(),
        syn::PathArguments::AngleBracketed(ab) => {
            let mut out = Vec::new();
            for a in &ab.args {
                match a {
                    syn::GenericArgument::Type(t) => {
                        out.push(NamedArg::Type(lower_type(t, consts, item_crate)?))
                    }
                    syn::GenericArgument::Lifetime(lt) => out.push(NamedArg::Lifetime(lt.clone())),
                    _ => return Err(fail(UnsupportedTypeReason::UnsupportedGenericArgument)),
                }
            }
            out
        }
        syn::PathArguments::Parenthesized(_) => {
            return Err(fail(UnsupportedTypeReason::UnsupportedForm))
        }
    };

    // A builtin must be spelled BARE. `normalize_type` has already reduced the
    // real std paths (`std::option::Option` → `Option`) at ingest and
    // deliberately leaves unknown crate paths alone, so anything still carrying
    // a prefix is a foreign type that merely shares a name — `foreign::Option`
    // is not `Option`, and collapsing it would silently retype the field.
    let is_bare = tp.path.leading_colon.is_none() && tp.path.segments.len() == 1;
    if is_bare {
        if args.is_empty() {
            if let Some(kind) = ScalarKind::from_name(&name) {
                return Ok(SourceType::Scalar(kind));
            }
            if name == "String" {
                return Ok(SourceType::Str);
            }
        }
        // A builtin generic takes types only; a lifetime argument on one is not
        // a shape this language has.
        let types: Option<Vec<SourceType>> = args
            .iter()
            .map(|a| match a {
                NamedArg::Type(t) => Some(t.clone()),
                NamedArg::Lifetime(_) => None,
            })
            .collect();
        if let Some(mut types) = types {
            let arity = |n: usize| {
                if types.len() == n {
                    Ok(())
                } else {
                    Err(fail(UnsupportedTypeReason::WrongGenericArity {
                        expected: n,
                    }))
                }
            };
            match name.as_str() {
                "Option" => {
                    arity(1)?;
                    return Ok(SourceType::Optional(Box::new(types.remove(0))));
                }
                "Vec" => {
                    arity(1)?;
                    return Ok(SourceType::Sequence(Box::new(types.remove(0))));
                }
                "Box" => {
                    arity(1)?;
                    return Ok(SourceType::Boxed(Box::new(types.remove(0))));
                }
                "Result" => {
                    arity(2)?;
                    let err = Box::new(types.remove(1));
                    let ok = Box::new(types.remove(0));
                    return Ok(SourceType::Fallible { ok, err });
                }
                _ => {}
            }
        }
    }

    // Everything else keeps its full path as identity, with the last segment's
    // arguments stripped out into `args` so one fact has one representation.
    let mut path = tp.clone();
    if let Some(last) = path.path.segments.last_mut() {
        last.arguments = syn::PathArguments::None;
    }
    Ok(SourceType::Named { path, args })
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
            SourceType::Named { path, args } => {
                let mut out = path.clone();
                if !args.is_empty() {
                    if let Some(last) = out.path.segments.last_mut() {
                        // Rebuilt in SOURCE ORDER, lifetimes included, so
                        // `Foo<'a, T>` comes back exactly as written.
                        let rendered: Vec<proc_macro2::TokenStream> = args
                            .iter()
                            .map(|a| match a {
                                NamedArg::Lifetime(lt) => quote::quote!(#lt),
                                NamedArg::Type(t) => {
                                    let t = t.to_syn();
                                    quote::quote!(#t)
                                }
                            })
                            .collect();
                        let ab: syn::AngleBracketedGenericArguments =
                            syn::parse_quote!(<#(#rendered),*>);
                        last.arguments = syn::PathArguments::AngleBracketed(ab);
                    }
                }
                syn::Type::Path(out)
            }
            SourceType::Array { elem, extent } => {
                let (t, n) = (elem.to_syn(), extent.to_expr());
                syn::parse_quote!([#t; #n])
            }
            SourceType::Ref {
                lifetime,
                mutable,
                inner,
            } => {
                let t = inner.to_syn();
                match (lifetime, mutable) {
                    (Some(lt), true) => syn::parse_quote!(&#lt mut #t),
                    (Some(lt), false) => syn::parse_quote!(&#lt #t),
                    (None, true) => syn::parse_quote!(&mut #t),
                    (None, false) => syn::parse_quote!(&#t),
                }
            }
            SourceType::Slice(inner) => {
                let t = inner.to_syn();
                syn::parse_quote!([#t])
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
            SourceType::Callback { args } => args.iter().for_each(|t| t.collect_extents(out)),
            SourceType::Named { args, .. } => args.iter().for_each(|a| {
                if let NamedArg::Type(t) = a {
                    t.collect_extents(out)
                }
            }),
            SourceType::Scalar(_) | SourceType::Str | SourceType::Unit => {}
        }
    }
}
