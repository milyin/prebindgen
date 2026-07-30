//! Types: a closed classification paired with the syntax it was read from.
//!
//! [`TypeRef`] is the pattern the whole element model follows — `kind` says what
//! the type *means*, `syntax` is the tokens the source wrote. Consumers
//! **classify off `kind` and spell off `syntax`**; see the [module docs](super)
//! for why that split is the point.
//!
//! [`TypeKind`] is total over the accepted grammar: a form with no variant here
//! is a form the language does not accept, so acceptance is a consequence of
//! lowering rather than a second list that can drift from it. Same contract, and
//! for the same reason, as [`lower_array_len`].

use std::{fmt, rc::Rc};

use quote::ToTokens;

use super::{
    array_len::{lower_array_len, ArrayExtent, ConstIndex, UnsupportedArrayLen},
    origin::Origin,
};
use crate::SourceLocation;

/// A type as the language decided it, plus the exact syntax it came from.
///
/// The [`Origin::syntax`] slice is what removes the pressure to make `kind`
/// lossless: a lifetime, an elided argument, a `Box` that changes nothing
/// outside Rust all survive there at zero modelling cost, so `kind` can stay
/// language-neutral and small.
#[derive(Clone, Debug)]
pub struct TypeRef {
    /// What the type means — the closed, destination-neutral classification.
    pub kind: TypeKind,
    /// The type as generated Rust must spell it — the source's own tokens,
    /// normalized to the flat namespace the generated crate can name (see
    /// [`Flat::parse`](super::Flat::parse)) — plus the source they came
    /// from.
    ///
    /// The syntax can say strictly more than `kind` does — `Box<String>` is a
    /// `Str` here — which is the point: what Rust needs and no destination
    /// language can see lives in the tokens, not in the classification.
    pub origin: Origin<syn::Type>,
}

impl TypeRef {
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

    /// The first nominal type reachable from here that `declared` does not hold.
    ///
    /// Recurses the same structure [`Self::collect_extents`] walks: what is
    /// reachable is what a destination language will have to convert, so every
    /// layer's inner reference counts.
    pub(super) fn first_unresolved(
        &self,
        declared: &std::collections::HashSet<String>,
    ) -> Option<String> {
        match &self.kind {
            TypeKind::Named { id } => (!declared.contains(&id.name)).then(|| id.name.clone()),
            TypeKind::Optional(t) | TypeKind::Sequence(t) | TypeKind::Ref { inner: t, .. } => {
                t.first_unresolved(declared)
            }
            TypeKind::Array { elem, .. } => elem.first_unresolved(declared),
            TypeKind::Fallible { ok, err } => ok
                .first_unresolved(declared)
                .or_else(|| err.first_unresolved(declared)),
            TypeKind::Callback { args } => args.iter().find_map(|a| a.first_unresolved(declared)),
            TypeKind::Scalar(_) | TypeKind::Str | TypeKind::Unit => None,
        }
    }

    /// This type and every type reachable inside it, outermost first.
    ///
    /// The nested positions are real [`TypeRef`]s carrying their own spelling and
    /// origin, so a consumer that indexes types finds `Foo` from `Vec<Foo>` with
    /// the classification already made rather than a sub-path to re-read.
    ///
    /// A [`Named`](TypeKind::Named)'s generic arguments are **not** among them:
    /// [`TypeId`] keeps a name and nothing else, so `MyBox<Foo>` reaches no `Foo`
    /// here. The full spelling is in [`Self::origin`] for whoever needs it.
    pub fn walk(&self) -> Vec<&TypeRef> {
        let mut out = Vec::new();
        self.collect_refs(&mut out);
        out
    }

    fn collect_refs<'a>(&'a self, out: &mut Vec<&'a TypeRef>) {
        out.push(self);
        match &self.kind {
            TypeKind::Optional(t) | TypeKind::Sequence(t) | TypeKind::Ref { inner: t, .. } => {
                t.collect_refs(out)
            }
            TypeKind::Array { elem, .. } => elem.collect_refs(out),
            TypeKind::Fallible { ok, err } => {
                ok.collect_refs(out);
                err.collect_refs(out);
            }
            TypeKind::Callback { args } => args.iter().for_each(|t| t.collect_refs(out)),
            TypeKind::Named { .. } | TypeKind::Scalar(_) | TypeKind::Str | TypeKind::Unit => {}
        }
    }

    fn collect_extents<'a>(&'a self, out: &mut Vec<&'a ArrayExtent>) {
        match &self.kind {
            TypeKind::Array { elem, extent } => {
                out.push(extent);
                elem.collect_extents(out);
            }
            TypeKind::Optional(t) | TypeKind::Sequence(t) | TypeKind::Ref { inner: t, .. } => {
                t.collect_extents(out)
            }
            TypeKind::Fallible { ok, err } => {
                ok.collect_extents(out);
                err.collect_extents(out);
            }
            TypeKind::Callback { args } => args.iter().for_each(|t| t.collect_extents(out)),
            TypeKind::Named { .. } => {}
            TypeKind::Scalar(_) | TypeKind::Str | TypeKind::Unit => {}
        }
    }
}

/// What a [`TypeRef`] means. The variants are the accepted type grammar.
///
/// One Rust spelling per concept is **not** the rule here — several are. A
/// concept earns a variant when a destination language would act on it; a
/// spelling that changes nothing outside Rust folds into the concept it carries
/// and survives in [`TypeRef::origin`]:
///
/// | Spelling | Kind | Why |
/// |---|---|---|
/// | `String`, `str` | [`Str`](TypeKind::Str) | one concept, two Rust types |
/// | `Vec<T>`, `[T]` | [`Sequence`](TypeKind::Sequence) | a run of `T`; owned vs borrowed is the [`Ref`](TypeKind::Ref) layer's fact, not a second variant |
/// | `Box<T>` | *whatever `T` is* | an owned `T` either way; nothing outside Rust can tell |
#[derive(Clone, Debug)]
pub enum TypeKind {
    /// A primitive with a fixed C/JVM counterpart.
    Scalar(ScalarKind),
    /// A UTF-8 string — `String` owned, `str` behind a [`Ref`](TypeKind::Ref).
    ///
    /// Both spellings are one concept: `&str` and `&String` are each a borrowed
    /// string and classify identically, which is what every adapter already
    /// does by hand.
    Str,
    /// `Option<T>`.
    Optional(Box<TypeRef>),
    /// A run of `T` — `Vec<T>` owned, `[T]` behind a [`Ref`](TypeKind::Ref).
    ///
    /// One variant, because ownership is already the [`Ref`](TypeKind::Ref)
    /// layer's fact: `&[T]` is `Ref(Sequence)`, `Vec<T>` is `Sequence`. A
    /// second variant would encode ownership twice and let the two copies
    /// disagree. `[T; N]` is *not* this — a fixed extent is a different
    /// concept, see [`Array`](TypeKind::Array).
    Sequence(Box<TypeRef>),
    /// `Result<T, E>`.
    Fallible { ok: Box<TypeRef>, err: Box<TypeRef> },
    /// Any other named type: a `#[prebindgen]` struct or enum, or a foreign
    /// path.
    ///
    /// `id` is the type's **identity** — a name, not syntax, so nothing outside
    /// this module has to take a path apart to learn what a type is. The last
    /// segment's generic arguments live in `args`, and only the *type*
    /// arguments: a lifetime argument says nothing a destination language can
    /// act on. The full spelling is in [`TypeRef::origin`] for whoever re-emits it.
    Named { id: TypeId },
    /// `[T; N]` — a run of `T` whose length is known at compile time.
    ///
    /// Deliberately not a [`Sequence`](TypeKind::Sequence) with an optional
    /// extent: a fixed array crosses by value as a primitive array, a `Vec`
    /// crosses as a heap collection, and every adapter branches between the two
    /// at every site.
    Array {
        elem: Box<TypeRef>,
        /// Boxed: an extent carries an [`Origin`] over the length expression, which
        /// makes it the size outlier among the kinds, and an array is the rare one.
        /// The same trade-off [`Unsupported::error`](super::Unsupported) makes.
        extent: Box<ArrayExtent>,
    },
    /// A borrow — `&T`, `&mut T`, or `&mut MaybeUninit<T>`. The lifetime is
    /// spelling, so it lives in [`TypeRef::origin`] rather than here.
    ///
    /// This is the ownership layer for every concept underneath it: `&str` is
    /// `Ref(Str)`, `&[T]` is `Ref(Sequence)`. A shared-ownership handle
    /// (`Arc<T>`, `Rc<T>`) belongs here too when the language accepts one.
    ///
    /// `inner` is always the borrowed *value's* type, so an out-parameter's
    /// `MaybeUninit` is absorbed into [`RefMode::Out`] rather than wrapping it:
    /// uninitialized-ness is a property of the **borrow**, not of the type, and
    /// it is meaningless anywhere else.
    Ref { mode: RefMode, inner: Box<TypeRef> },
    /// `impl Fn(A, B, …) + Send + Sync + 'static` — the callback form.
    Callback { args: Vec<TypeRef> },
    /// `()`.
    Unit,
}

/// What a borrow permits, and what the callee owes.
///
/// One axis with three values rather than a `mutable` flag plus a wrapper, so the
/// combinations that mean nothing at a boundary — a shared borrow of
/// uninitialized storage, an owned `MaybeUninit` — cannot be written down.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RefMode {
    /// `&T` — the callee may read it and must not write it.
    Shared,
    /// `&mut T` — the callee may read and write an already-valid `T`.
    Exclusive,
    /// `&mut MaybeUninit<T>` — an **out-parameter**: the caller supplies storage
    /// only, and the callee's job is to make it a valid `T`. The callee may not
    /// read it first.
    ///
    /// This is a boundary concept every destination language has (C's `T *out`),
    /// which is why it is modelled here rather than left as a nominal
    /// `MaybeUninit` for each adapter to recognise.
    Out,
}

/// A nominal type's identity: a name, and nothing else.
///
/// `#[prebindgen]` names live in one flat namespace — a duplicate is a
/// [`ParseError`](super::ParseError) — so the name is the whole address. It
/// deliberately carries **no crate**: a reference carries a name, and the
/// declaring crate belongs to the declaration, reachable by looking the name up
/// among the elements. Putting the use site's crate here would make the same
/// type compare unequal to itself across two source crates.
///
/// A name rather than a `syn::Path` on purpose: an identity kept as syntax
/// makes every consumer take a path apart to learn what a type is, which is the
/// re-classification issue #211 exists to stop — and one the boundary ledger
/// would not even see, since it watches `syn::Type` and `syn::Expr`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypeId {
    /// The path as written, minus any generic arguments — `Foo`,
    /// `foreign::Option`. Normalized, so a reducible std or source-module path
    /// has already collapsed to its final segment.
    pub name: String,
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
    /// A syntactic form with no place in the language: a raw pointer, a bare
    /// trait object, a closure type, a macro, `Self`, a never type, an inferred
    /// type.
    ///
    /// A `#[prebindgen]` crate is idiomatic Rust — the adapter owns the lowering
    /// to pointers — so `*const T` / `*mut T` are refused here rather than
    /// modelled. No adapter has a selection arm for one, so accepting them would
    /// only defer the failure to a late "unresolved type".
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
    /// `MaybeUninit<T>` somewhere other than behind a `&mut`.
    ///
    /// Uninitialized storage is a property of a *borrow*, not of a type — see
    /// [`RefMode::Out`]. Owned, returned or stored in a field it promises nothing
    /// a destination language can use, and reading it would be undefined.
    OwnedUninit,
    /// `&MaybeUninit<T>` — a shared borrow of uninitialized storage.
    ///
    /// A shared borrow promises a readable `T`, and this supplies storage that may
    /// not be one. Only `&mut MaybeUninit<T>` means anything: see
    /// [`RefMode::Out`].
    SharedUninit,
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
            UnsupportedTypeReason::OwnedUninit => write!(
                f,
                "type `{}` is uninitialized storage outside an out-parameter. Only `&mut \
                 MaybeUninit<T>` means anything at a boundary \u{2014} it says the caller supplies \
                 the slot and the callee fills it; owned or in a field it promises nothing, and \
                 reading it would be undefined",
                self.offending
            ),
            UnsupportedTypeReason::SharedUninit => write!(
                f,
                "type `{}` is a shared borrow of uninitialized storage: `&T` promises a readable \
                 `T`, which this may not be. Use `&mut MaybeUninit<T>` for an out-parameter",
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
/// `at` is the origin of the item this type was written in — the location every
/// node lowered from that item shares, and the crate an array extent's const
/// must come from.
pub(crate) fn lower_type(
    ty: &syn::Type,
    consts: &ConstIndex,
    at: &Rc<SourceLocation>,
) -> Result<TypeRef, UnsupportedType> {
    let fail = |reason| UnsupportedType {
        offending: ty.to_token_stream().to_string(),
        reason,
    };
    // Every arm builds `kind` only; the origin is attached once, here, so no arm
    // can forget it or attach a rebuilt approximation.
    let kind = match ty {
        // A group or paren wraps the same type. Its inner node keeps the inner
        // spelling, which is the one a consumer wants to emit.
        syn::Type::Group(g) => return lower_type(&g.elem, consts, at),
        syn::Type::Paren(p) => return lower_type(&p.elem, consts, at),
        // The mode is read off the borrow AND its target together, because
        // `&mut MaybeUninit<T>` is one concept — an out-parameter — rather than a
        // mutable borrow of a distinct `MaybeUninit` type.
        syn::Type::Reference(r) => {
            let (mode, target) = match maybe_uninit_inner(&r.elem) {
                Some(inner) if r.mutability.is_some() => (RefMode::Out, inner),
                // `&MaybeUninit<T>` promises a readable `T` and supplies storage
                // that may not be one. Nothing at a boundary can use it.
                Some(_) => return Err(fail(UnsupportedTypeReason::SharedUninit)),
                None if r.mutability.is_some() => (RefMode::Exclusive, (*r.elem).clone()),
                None => (RefMode::Shared, (*r.elem).clone()),
            };
            TypeKind::Ref {
                mode,
                inner: Box::new(lower_type(&target, consts, at)?),
            }
        }
        // `[T]` is the borrowed spelling of the same concept `Vec<T>` owns.
        syn::Type::Slice(s) => TypeKind::Sequence(Box::new(lower_type(&s.elem, consts, at)?)),
        _ if is_unit_type(ty) => TypeKind::Unit,
        // Only the unit is in the language. Refusing here names the type;
        // accepting would defer the failure to an "unresolved type" much later.
        syn::Type::Tuple(_) => return Err(fail(UnsupportedTypeReason::UnsupportedTuple)),
        syn::Type::Array(a) => {
            let rendered = a.to_token_stream().to_string();
            let extent = lower_array_len(&a.len, &rendered, at, consts)
                .map_err(|e| fail(UnsupportedTypeReason::BadArrayExtent(Box::new(e))))?;
            TypeKind::Array {
                elem: Box::new(lower_type(&a.elem, consts, at)?),
                extent: Box::new(extent),
            }
        }
        // The callback shape is decided by `extract_fn_trait_args`, this
        // module's own — and the pipeline's only — authority for the form.
        syn::Type::ImplTrait(_) => match super::extract_fn_trait_args(ty) {
            Some(args) => TypeKind::Callback {
                args: args
                    .iter()
                    .map(|a| lower_type(a, consts, at))
                    .collect::<Result<_, _>>()?,
            },
            None => return Err(fail(UnsupportedTypeReason::DisallowedImplTrait)),
        },
        syn::Type::Path(tp) => lower_path(ty, tp, consts, at)?,
        _ => return Err(fail(UnsupportedTypeReason::UnsupportedForm)),
    };
    Ok(TypeRef {
        kind,
        origin: Origin::new(ty.clone(), Rc::clone(at)),
    })
}

fn lower_path(
    ty: &syn::Type,
    tp: &syn::TypePath,
    consts: &ConstIndex,
    at: &Rc<SourceLocation>,
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
    // `TypeRef::origin`, so modelling it would be a second copy of one fact.
    let mut has_lifetime_arg = false;
    let args: Vec<TypeRef> = match &last.arguments {
        syn::PathArguments::None => Vec::new(),
        syn::PathArguments::AngleBracketed(ab) => {
            let mut out = Vec::new();
            for a in &ab.args {
                match a {
                    syn::GenericArgument::Type(t) => {
                        out.push(lower_type(t, consts, at)?);
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
            // `String` and `str` are one concept. `str` is unsized and so only
            // ever appears behind a `&`, which the `Ref` layer already records
            // — classifying it as a nominal type instead would send every
            // adapter looking for an item named `str` to resolve.
            if name == "String" || name == "str" {
                return Ok(TypeKind::Str);
            }
        }
        // A builtin generic takes TYPE arguments only — a lifetime on one is not a
        // shape this language has — with one exception: `Cow`'s own signature HAS a
        // lifetime, so it is the one builtin where a lifetime argument is expected
        // rather than refused.
        if !has_lifetime_arg || name == "Cow" {
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
                // Reached here, it is not behind a `&mut`, so it is not an
                // out-parameter — see `RefMode::Out`, which is the only place
                // uninitialized storage means anything.
                "MaybeUninit" => return Err(fail(UnsupportedTypeReason::OwnedUninit)),
                // `Box<T>` **is** `T`: an owned value either way, and no
                // destination language can tell the two apart. So it carries no
                // kind of its own and classifies as whatever it wraps — the
                // `Box` survives in `TypeRef::origin`, which is what generated
                // Rust spells. (A shared-ownership handle would classify as a
                // `Ref` for the same reason, when the language accepts one.)
                "Box" => {
                    arity(1)?;
                    return Ok(args.remove(0).kind);
                }
                // `Cow<'_, T>` **is** `T`, for the same reason `Box<T>` is: borrowed
                // or owned, and no destination language can tell. Both adapters
                // already say exactly that — cbindgen lowers it "just like `Vec<T>`
                // outputs", and jnigen's converter body is
                // `byte_array_from_slice(&v)`, which works by deref and is identical
                // to the `Vec<u8>` one. So this classification *predicts* their
                // behaviour instead of leaving it a special case.
                //
                // The `Cow` survives in `TypeRef::origin`, which is where an adapter
                // reads the param type its generated fn must spell — and it must,
                // since `Cow<'_, [u8]>` is not interchangeable with `Vec<u8>` in a
                // Rust signature.
                "Cow" => {
                    arity(1)?;
                    return Ok(args.remove(0).kind);
                }
                "Result" => {
                    arity(2)?;
                    let err = Box::new(args.remove(1));
                    let ok = Box::new(args.remove(0));
                    return Ok(TypeKind::Fallible { ok, err });
                }
                _ => return Ok(named(tp)),
            }
        }
    }
    Ok(named(tp))
}

/// If `ty` is a bare `MaybeUninit<T>`, the `T` it holds storage for.
///
/// Bare, for the reason every builtin generic is: `normalize_type` has already
/// reduced the real std paths at ingest, so anything still carrying a prefix is a
/// foreign type that merely shares the name.
fn maybe_uninit_inner(ty: &syn::Type) -> Option<syn::Type> {
    let syn::Type::Path(tp) = ty else { return None };
    if tp.qself.is_some() || tp.path.leading_colon.is_some() || tp.path.segments.len() != 1 {
        return None;
    }
    let seg = &tp.path.segments[0];
    if seg.ident != "MaybeUninit" {
        return None;
    }
    let syn::PathArguments::AngleBracketed(ab) = &seg.arguments else {
        return None;
    };
    match ab.args.first() {
        Some(syn::GenericArgument::Type(t)) if ab.args.len() == 1 => Some(t.clone()),
        _ => None,
    }
}

/// True when `ty` is the unit type `()`.
///
/// The language's one answer to that question: [`lower_type`] classifies it as
/// [`TypeKind::Unit`], and the callback grammar uses it to insist a callback
/// returns nothing.
pub(crate) fn is_unit_type(ty: &syn::Type) -> bool {
    match ty {
        syn::Type::Tuple(t) => t.elems.is_empty(),
        // A parenthesized or grouped `()` is still `()`.
        syn::Type::Paren(p) => is_unit_type(&p.elem),
        syn::Type::Group(g) => is_unit_type(&g.elem),
        _ => false,
    }
}

/// `Named` with the identity read off the path: every segment joined, minus the
/// generic arguments, which are already in `args`.
/// `Named` with the identity read off the path: every segment joined, minus the
/// generic arguments.
///
/// The arguments are **lowered but not retained** — a bad type inside one is still
/// diagnosed, it just leaves no trace. Nothing could read them: a surviving
/// reference resolves to a declared type, and no declaration takes type parameters,
/// so `Foo<u8>` against a declared `Foo` would not compile in the source crate.
/// Accepting *instantiated* generics — `Wrapper<u8>` as its own declared type — is
/// what would bring the field back.
fn named(tp: &syn::TypePath) -> TypeKind {
    let name = tp
        .path
        .segments
        .iter()
        .map(|s| s.ident.to_string())
        .collect::<Vec<_>>()
        .join("::");
    TypeKind::Named {
        id: TypeId { name },
    }
}
