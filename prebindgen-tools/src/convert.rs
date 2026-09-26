//! Custom conversions: a source type that crosses as another type.
//!
//! `convert!(Millis).input(fun!(millis_from_raw)).output(fun!(millis_to_raw))`
//! says that `Millis` never crosses as itself: it crosses as `u64` — the
//! **representation** — and these two functions convert. The adapter lowers
//! the representation like any other type and wraps the resulting wires in
//! the conversion's stages ([`Stage::apply`]).
//!
//! A stage is a function (a flat item, or a binding-local path with a
//! stated signature) or a standard trait impl (`From`, `Into`, `TryFrom`,
//! `TryInto`). A function returning `Result` makes its stage fallible; the
//! error is reported through its `Display`.

use prebindgen_flat::{
    flat::{Function, TypeKind, TypeRef},
    Flat,
};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

use crate::Qualifier;

/// A function a declaration refers to: a flat item by name, or a
/// binding-local path with its signature stated.
#[derive(Clone, Debug)]
pub struct FnRef {
    pub path: syn::Path,
    pub sig: Option<syn::Signature>,
}

impl FnRef {
    pub fn new(path: syn::Path) -> Self {
        Self { path, sig: None }
    }

    /// State the signature of a function the flat model does not know.
    pub fn sig(mut self, sig: syn::Signature) -> Self {
        self.sig = Some(sig);
        self
    }

    /// The last path segment: the flat name, or the local function's name.
    pub fn name(&self) -> &syn::Ident {
        &self.path.segments.last().expect("a non-empty path").ident
    }

    /// Whether this names a flat item (a bare name, no stated signature).
    pub fn is_flat(&self) -> bool {
        self.sig.is_none() && self.path.segments.len() == 1 && self.path.leading_colon.is_none()
    }

    /// The function as the model reads it: the flat item, or the stated
    /// signature lowered against the flat namespace.
    pub fn resolve(&self, flat: &Flat) -> Result<Function, String> {
        match &self.sig {
            Some(sig) => {
                let mut sig = sig.clone();
                sig.ident = self.name().clone();
                let item: syn::ItemFn = syn::parse_quote!(pub #sig { unimplemented!() });
                flat.lower_signature(&item)
                    .map_err(|e| format!("`{}`: {e}", path_string(&self.path)))
            }
            None => flat
                .function(self.name())
                .cloned()
                .ok_or_else(|| format!("`{}` is not a #[prebindgen] function", self.name())),
        }
    }

    /// The path generated code calls.
    pub fn callee(&self, q: &Qualifier<'_>) -> TokenStream {
        if self.is_flat() {
            q.path(self.name())
        } else {
            self.path.to_token_stream()
        }
    }
}

fn path_string(p: &syn::Path) -> String {
    p.to_token_stream().to_string().replace(' ', "")
}

/// How one direction of a conversion is carried out.
#[derive(Clone, Debug)]
pub enum Via {
    Fn(FnRef),
    From(syn::Type),
    Into(syn::Type),
    TryFrom(syn::Type),
    TryInto(syn::Type),
}

impl From<FnRef> for Via {
    fn from(f: FnRef) -> Self {
        Via::Fn(f)
    }
}

/// A declared conversion for one source type.
#[derive(Clone, Debug)]
pub struct Conversion {
    pub ty: syn::Type,
    pub input: Option<Via>,
    pub output: Option<Via>,
}

impl Conversion {
    pub fn new(ty: syn::Type) -> Self {
        Self {
            ty,
            input: None,
            output: None,
        }
    }

    /// How a representation value becomes the source type.
    pub fn input(mut self, via: impl Into<Via>) -> Self {
        self.input = Some(via.into());
        self
    }

    /// How the source type becomes its representation.
    pub fn output(mut self, via: impl Into<Via>) -> Self {
        self.output = Some(via.into());
        self
    }

    /// Resolve both directions against the model.
    pub fn resolve(&self, flat: &Flat) -> Result<ResolvedConversion, String> {
        let target = flat
            .classify(&self.ty)
            .map_err(|e| format!("convert!({}): {e}", self.ty.to_token_stream()))?;
        let input = self
            .input
            .as_ref()
            .map(|v| Stage::resolve(flat, v, Direction::In))
            .transpose()?;
        let output = self
            .output
            .as_ref()
            .map(|v| Stage::resolve(flat, v, Direction::Out))
            .transpose()?;
        Ok(ResolvedConversion {
            target,
            input,
            output,
        })
    }
}

/// A conversion resolved against the model.
#[derive(Clone, Debug)]
pub struct ResolvedConversion {
    /// The source type being converted.
    pub target: TypeRef,
    /// Representation → source.
    pub input: Option<Stage>,
    /// Source → representation.
    pub output: Option<Stage>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Direction {
    In,
    Out,
}

/// One direction of a conversion.
#[derive(Clone, Debug)]
pub struct Stage {
    /// The representation type on the boundary side.
    pub repr: TypeRef,
    /// Whether the stage can fail.
    pub fallible: bool,
    how: How,
}

#[derive(Clone, Debug)]
enum How {
    /// Call a function; `by_ref` when it takes its argument by reference.
    Call { fun: FnRef, by_ref: bool },
    From(TypeRef),
    Into(TypeRef),
    TryFrom(TypeRef),
    TryInto(TypeRef),
}

impl Stage {
    fn resolve(flat: &Flat, via: &Via, dir: Direction) -> Result<Self, String> {
        let classify = |t: &syn::Type| {
            flat.classify(t)
                .map_err(|e| format!("conversion type `{}`: {e}", t.to_token_stream()))
        };
        Ok(match via {
            Via::Fn(fun) => {
                let f = fun.resolve(flat)?;
                let param = f.params.first().ok_or_else(|| {
                    format!("conversion fn `{}` takes no argument", fun.name())
                })?;
                let (ret, fallible) = match f.ret.kind() {
                    TypeKind::Fallible { ok, .. } => ((**ok).clone(), true),
                    _ => (f.ret.clone(), false),
                };
                match dir {
                    Direction::In => Stage {
                        repr: param.ty.clone(),
                        fallible,
                        how: How::Call {
                            fun: fun.clone(),
                            by_ref: false,
                        },
                    },
                    Direction::Out => {
                        let by_ref = matches!(param.ty.kind(), TypeKind::Ref { .. });
                        Stage {
                            repr: ret,
                            fallible,
                            how: How::Call {
                                fun: fun.clone(),
                                by_ref,
                            },
                        }
                    }
                }
            }
            Via::From(t) => {
                let r = classify(t)?;
                Stage { repr: r.clone(), fallible: false, how: How::From(r) }
            }
            Via::Into(t) => {
                let r = classify(t)?;
                Stage { repr: r.clone(), fallible: false, how: How::Into(r) }
            }
            Via::TryFrom(t) => {
                let r = classify(t)?;
                Stage { repr: r.clone(), fallible: true, how: How::TryFrom(r) }
            }
            Via::TryInto(t) => {
                let r = classify(t)?;
                Stage { repr: r.clone(), fallible: true, how: How::TryInto(r) }
            }
        })
    }

    /// Apply the stage to `value`. For an input stage `value` is the
    /// representation and the result the source type; for an output stage
    /// the other way round. A fallible stage produces an expression using
    /// `?` on a `Result<_, String>`.
    pub fn apply(&self, q: &Qualifier<'_>, target: &TypeRef, value: &TokenStream) -> TokenStream {
        let t = q.ty(target);
        let call = match &self.how {
            How::Call { fun, by_ref } => {
                let callee = fun.callee(q);
                if *by_ref {
                    quote!(#callee(&#value))
                } else {
                    quote!(#callee(#value))
                }
            }
            How::From(r) => {
                let r = q.ty(r);
                quote!(<#t as ::core::convert::From<#r>>::from(#value))
            }
            How::Into(r) => {
                let r = q.ty(r);
                quote!(<#t as ::core::convert::Into<#r>>::into(#value))
            }
            How::TryFrom(r) => {
                let r = q.ty(r);
                quote!(<#t as ::core::convert::TryFrom<#r>>::try_from(#value))
            }
            How::TryInto(r) => {
                let r = q.ty(r);
                quote!(<#t as ::core::convert::TryInto<#r>>::try_into(#value))
            }
        };
        if self.fallible {
            quote!(#call.map_err(|__e| ::std::string::ToString::to_string(&__e))?)
        } else {
            call
        }
    }
}

/// `fun!(name)` / `fun!(crate::local)` — a function reference.
#[macro_export]
macro_rules! fun {
    ($($path:tt)+) => {
        $crate::convert::FnRef::new($crate::__syn::parse_quote!($($path)+))
    };
}

/// `convert!(Type)` — start a conversion declaration.
#[macro_export]
macro_rules! convert {
    ($($ty:tt)+) => {
        $crate::convert::Conversion::new($crate::__syn::parse_quote!($($ty)+))
    };
}

/// `from!(Repr)` — the input stage is `From<Repr>` on the source type.
#[macro_export]
macro_rules! from {
    ($($ty:tt)+) => { $crate::convert::Via::From($crate::__syn::parse_quote!($($ty)+)) };
}

/// `into!(Repr)` — the output stage is `Into<Repr>` on the source type.
#[macro_export]
macro_rules! into {
    ($($ty:tt)+) => { $crate::convert::Via::Into($crate::__syn::parse_quote!($($ty)+)) };
}

/// `try_from!(Repr)` — the input stage is `TryFrom<Repr>` on the source type.
#[macro_export]
macro_rules! try_from {
    ($($ty:tt)+) => { $crate::convert::Via::TryFrom($crate::__syn::parse_quote!($($ty)+)) };
}

/// `try_into!(Repr)` — the output stage is `TryInto<Repr>` on the source type.
#[macro_export]
macro_rules! try_into {
    ($($ty:tt)+) => { $crate::convert::Via::TryInto($crate::__syn::parse_quote!($($ty)+)) };
}

/// `sig!((a: A, b: B) -> R)` — a signature, for a binding-local function.
#[macro_export]
macro_rules! sig {
    (($($args:tt)*) -> $($ret:tt)+) => {
        $crate::__syn::parse_quote!(fn __sig($($args)*) -> $($ret)+)
    };
    (($($args:tt)*)) => {
        $crate::__syn::parse_quote!(fn __sig($($args)*))
    };
}

/// `ty!(T)` — a `syn::Type`.
#[macro_export]
macro_rules! ty {
    ($($t:tt)+) => { { let __t: $crate::__syn::Type = $crate::__syn::parse_quote!($($t)+); __t } };
}

/// `path!(a::b)` — a `syn::Path`.
#[macro_export]
macro_rules! path {
    ($($t:tt)+) => { { let __p: $crate::__syn::Path = $crate::__syn::parse_quote!($($t)+); __p } };
}

/// `expr!(e)` — a `syn::Expr`.
#[macro_export]
macro_rules! expr {
    ($($t:tt)+) => { { let __e: $crate::__syn::Expr = $crate::__syn::parse_quote!($($t)+); __e } };
}

/// `ident!(name)` — a `syn::Ident`.
#[macro_export]
macro_rules! ident {
    ($i:ident) => {{
        let __i: $crate::__syn::Ident = $crate::__syn::parse_quote!($i);
        __i
    }};
}
