//! Types: a closed classification paired with the syntax it was read from.
//!
//! [`Type`] is the pattern the whole element model follows — `kind` says what
//! the type *means*, `syntax` is the tokens the source wrote. Consumers
//! **classify off `kind` and spell off `syntax`**; see the [module docs](super)
//! for why that split is the point.
//!
//! [`TypeKind`] is total over the accepted grammar: a form with no variant here
//! is a form the language does not accept, so acceptance is a consequence of
//! lowering rather than a second list that can drift from it. Same contract, and
//! for the same reason, as [`lower_array_len`].

use std::fmt;

use quote::ToTokens;

use super::array_len::{lower_array_len, ArrayExtent, ConstIndex, UnsupportedArrayLen};

/// A type as the language decided it, plus the exact syntax it came from.
///
/// The `syntax` slice is what removes the pressure to make `kind` lossless: a
/// lifetime, the spelling `0x07`, a `crate::`-qualified path or an elided
/// argument all survive here at zero modelling cost, so `kind` can stay
/// language-neutral and small.
#[derive(Clone, Debug)]
pub struct Type {
    /// What the type means — the closed, destination-neutral classification.
    pub kind: TypeKind,
    /// The type exactly as the source wrote it. Feed this to `quote!` when
    /// generated Rust has to name the type; never `match` on it to decide what
    /// the type is.
    pub syntax: syn::Type,
}

impl Type {
    /// The extent of this type when it is an array, else `None`.
    pub fn array_extent(&self) -> Option<&ArrayExtent> {
        match &self.kind {
            TypeKind::Array { extent, .. } => Some(extent),
            _ => None,
        }
    }

    /// Every extent reachable from this type, outermost first — so a nested
    /// `[[u8; A]; B]` yields `B` then `A`.
    ///
    /// Used to find which consts an emitted C type may name, and therefore which
    /// must reach the header as a `#define`.
    pub fn extents(&self) -> Vec<&ArrayExtent> {
        let mut out = Vec::new();
        self.collect_extents(&mut out);
        out
    }

    fn collect_extents<'a>(&'a self, out: &mut Vec<&'a ArrayExtent>) {
        match &self.kind {
            TypeKind::Array { elem, extent } => {
                out.push(extent);
                elem.collect_extents(out);
            }
            TypeKind::Optional(t)
            | TypeKind::Sequence(t)
            | TypeKind::Boxed(t)
            | TypeKind::Slice(t)
            | TypeKind::Ref { inner: t, .. }
            | TypeKind::Ptr { inner: t, .. } => t.collect_extents(out),
            TypeKind::Fallible { ok, err } => {
                ok.collect_extents(out);
                err.collect_extents(out);
            }
            TypeKind::Callback { args } | TypeKind::Named { args, .. } => {
                args.iter().for_each(|t| t.collect_extents(out))
            }
            TypeKind::Scalar(_) | TypeKind::Str | TypeKind::Unit => {}
        }
    }
}

/// What a [`Type`] means. The variants are the accepted type grammar.
#[derive(Clone, Debug)]
pub enum TypeKind {
    /// A primitive with a fixed C/JVM counterpart.
    Scalar(ScalarKind),
    /// `String` — an owned, heap-allocated UTF-8 string.
    Str,
    /// `Option<T>`.
    Optional(Box<Type>),
    /// `Vec<T>`.
    Sequence(Box<Type>),
    /// `Box<T>`.
    Boxed(Box<Type>),
    /// `Result<T, E>`.
    Fallible { ok: Box<Type>, err: Box<Type> },
    /// Any other named type: a `#[prebindgen]` struct or enum, or a foreign
    /// path.
    ///
    /// `path` is the type's **identity**, with the last segment's generic
    /// arguments stripped out into `args` so one fact has one representation.
    /// `args` holds the *type* arguments only — a lifetime argument says nothing
    /// a destination language can act on, and the full spelling is in
    /// [`Type::syntax`] for whoever has to re-emit it.
    Named { path: syn::Path, args: Vec<Type> },
    /// `[T; N]`.
    Array {
        elem: Box<Type>,
        extent: ArrayExtent,
    },
    /// `&T` / `&'a T` / `&mut T`. The lifetime is spelling, so it lives in
    /// [`Type::syntax`] rather than here.
    Ref { mutable: bool, inner: Box<Type> },
    /// `[T]`, only ever behind a [`TypeKind::Ref`].
    Slice(Box<Type>),
    /// `*const T` / `*mut T`.
    Ptr { mutable: bool, inner: Box<Type> },
    /// `impl Fn(A, B, …) + Send + Sync + 'static` — the callback form.
    Callback { args: Vec<Type> },
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
    /// `impl Trait` that is not the accepted callback form — anything but
    /// `impl Fn(..) + Send + Sync + 'static` returning `()`.
    DisallowedImplTrait,
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
    /// A generic argument that is neither a type nor a lifetime — a const
    /// generic, an associated-type binding.
    UnsupportedGenericArgument,
    /// The array's extent — see [`ArrayLenReason`](super::ArrayLenReason).
    BadArrayExtent(Box<UnsupportedArrayLen>),
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
                "type `{}` is not an accepted callback — the only `impl Trait` in the language is \
                 `impl Fn(..) + Send + Sync + 'static` returning `()`",
                self.offending
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
                "type `{}` has a generic argument that is neither a type nor a lifetime",
                self.offending
            ),
        }?;
        write!(f, " — see docs/source-language.md for the accepted grammar")
    }
}

impl std::error::Error for UnsupportedType {}

/// Lower one captured type.
///
/// **Total over the accepted grammar**: `Ok` means every part of the type was
/// understood, so a form this function does not lower is a form the language
/// does not accept.
///
/// `item_crate` is the origin of the item the type was written in; an extent
/// must name a const from that same crate.
pub(crate) fn lower_type(
    ty: &syn::Type,
    consts: &ConstIndex,
    item_crate: Option<&str>,
) -> Result<Type, UnsupportedType> {
    let fail = |reason| UnsupportedType {
        offending: ty.to_token_stream().to_string(),
        reason,
    };
    // Every arm builds `kind` only; the syntax slice is attached once, here, so
    // no arm can forget it or attach a rebuilt approximation.
    let kind = match ty {
        // A group or paren wraps the same type. Its inner node keeps the inner
        // spelling, which is the one a consumer wants to emit.
        syn::Type::Group(g) => return lower_type(&g.elem, consts, item_crate),
        syn::Type::Paren(p) => return lower_type(&p.elem, consts, item_crate),
        syn::Type::Reference(r) => TypeKind::Ref {
            mutable: r.mutability.is_some(),
            inner: Box::new(lower_type(&r.elem, consts, item_crate)?),
        },
        syn::Type::Slice(s) => TypeKind::Slice(Box::new(lower_type(&s.elem, consts, item_crate)?)),
        syn::Type::Ptr(p) => TypeKind::Ptr {
            mutable: p.mutability.is_some(),
            inner: Box::new(lower_type(&p.elem, consts, item_crate)?),
        },
        syn::Type::Tuple(t) if t.elems.is_empty() => TypeKind::Unit,
        // Only the unit is in the language. Refusing here names the type;
        // accepting would defer the failure to an "unresolved type" much later.
        syn::Type::Tuple(_) => return Err(fail(UnsupportedTypeReason::UnsupportedTuple)),
        syn::Type::Array(a) => {
            let rendered = a.to_token_stream().to_string();
            let extent = lower_array_len(&a.len, &rendered, item_crate, consts)
                .map_err(|e| fail(UnsupportedTypeReason::BadArrayExtent(Box::new(e))))?;
            TypeKind::Array {
                elem: Box::new(lower_type(&a.elem, consts, item_crate)?),
                extent,
            }
        }
        // The callback shape is decided by `extract_fn_trait_args`, which is
        // also what the registry accepts today — ONE authority for the form.
        syn::Type::ImplTrait(_) => match crate::api::core::registry::extract_fn_trait_args(ty) {
            Some(args) => TypeKind::Callback {
                args: args
                    .iter()
                    .map(|a| lower_type(a, consts, item_crate))
                    .collect::<Result<_, _>>()?,
            },
            None => return Err(fail(UnsupportedTypeReason::DisallowedImplTrait)),
        },
        syn::Type::Path(tp) => lower_path(ty, tp, consts, item_crate)?,
        _ => return Err(fail(UnsupportedTypeReason::UnsupportedForm)),
    };
    Ok(Type {
        kind,
        syntax: ty.clone(),
    })
}

fn lower_path(
    ty: &syn::Type,
    tp: &syn::TypePath,
    consts: &ConstIndex,
    item_crate: Option<&str>,
) -> Result<TypeKind, UnsupportedType> {
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

    // Type arguments only. A lifetime argument is accepted and dropped: it is
    // part of the spelling (`Foo<'a>` is not `Foo`), and the spelling is in
    // `Type::syntax`, so modelling it would be a second copy of one fact.
    let mut has_lifetime_arg = false;
    let args: Vec<Type> = match &last.arguments {
        syn::PathArguments::None => Vec::new(),
        syn::PathArguments::AngleBracketed(ab) => {
            let mut out = Vec::new();
            for a in &ab.args {
                match a {
                    syn::GenericArgument::Type(t) => {
                        out.push(lower_type(t, consts, item_crate)?);
                    }
                    syn::GenericArgument::Lifetime(_) => has_lifetime_arg = true,
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
        if args.is_empty() && !has_lifetime_arg {
            if let Some(kind) = ScalarKind::from_name(&name) {
                return Ok(TypeKind::Scalar(kind));
            }
            if name == "String" {
                return Ok(TypeKind::Str);
            }
        }
        // A builtin generic takes types only; a lifetime argument on one is not
        // a shape this language has.
        if !has_lifetime_arg {
            let mut args = args;
            let arity = |n: usize| {
                if args.len() == n {
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
                    return Ok(TypeKind::Optional(Box::new(args.remove(0))));
                }
                "Vec" => {
                    arity(1)?;
                    return Ok(TypeKind::Sequence(Box::new(args.remove(0))));
                }
                "Box" => {
                    arity(1)?;
                    return Ok(TypeKind::Boxed(Box::new(args.remove(0))));
                }
                "Result" => {
                    arity(2)?;
                    let err = Box::new(args.remove(1));
                    let ok = Box::new(args.remove(0));
                    return Ok(TypeKind::Fallible { ok, err });
                }
                _ => return Ok(named(tp, args)),
            }
        }
    }
    Ok(named(tp, args))
}

/// `Named` with the last segment's arguments stripped out of the identity path.
fn named(tp: &syn::TypePath, args: Vec<Type>) -> TypeKind {
    let mut path = tp.path.clone();
    if let Some(last) = path.segments.last_mut() {
        last.arguments = syn::PathArguments::None;
    }
    TypeKind::Named { path, args }
}
