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
    /// Path to call in generated Rust; a bare name without [`Self::sig`]
    /// refers to a captured flat function.
    pub path: syn::Path,
    /// Signature supplied for a function outside the flat model.
    pub sig: Option<syn::Signature>,
}

impl FnRef {
    /// Create a reference to a flat function or a binding-local function.
    /// Supply [`Self::sig`] for the latter before [`Self::resolve`].
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
    /// signature read against the flat model's type names.
    pub fn resolve(&self, flat: &Flat) -> Result<Function, String> {
        match &self.sig {
            Some(sig) => {
                let mut sig = sig.clone();
                sig.ident = self.name().clone();
                let item: syn::ItemFn = syn::parse_quote!(pub #sig { unimplemented!() });
                flat.lower_signature(&item)
                    .map_err(|e| format!("`{}`: {e}", path_string(&self.path)))
            }
            None if !self.is_flat() => Err(format!(
                "`{}` is not a #[prebindgen] function: state its signature with `.sig(sig!(..))`",
                path_string(&self.path)
            )),
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
    /// Call a source or binding-local function; see [`FnRef`].
    Fn(FnRef),
    /// Use `Source::from(Repr)` for input conversion.
    From(syn::Type),
    /// Use `Source::into(Repr)` for output conversion.
    Into(syn::Type),
    /// Use `Source::try_from(Repr)` for fallible input conversion.
    TryFrom(syn::Type),
    /// Use `Source::try_into(Repr)` for fallible output conversion.
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
    /// The source type that will cross as a representation type.
    pub ty: syn::Type,
    /// Representation to source; `None` leaves this direction undeclared.
    pub input: Option<Via>,
    /// Source to representation; `None` leaves this direction undeclared.
    pub output: Option<Via>,
}

impl Conversion {
    /// Start a declaration for `ty`; the [`convert!`](crate::convert!) macro is shorthand.
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
    ///
    /// Each declared direction is read first, for how it converts and which
    /// representation type it names. The directions must name the same one,
    /// and at least one must be declared; that type is then the conversion's
    /// representation.
    pub fn resolve(&self, flat: &Flat) -> Result<ResolvedConversion, String> {
        let target = flat
            .classify(&self.ty)
            .map_err(|e| format!("convert!({}): {e}", self.ty.to_token_stream()))?;
        let read = |via: &Option<Via>, dir| {
            via.as_ref()
                .map(|v| Stage::resolve(flat, v, &target, dir))
                .transpose()
        };
        let (input, output) = (
            read(&self.input, Direction::In)?,
            read(&self.output, Direction::Out)?,
        );
        let repr = match (&input, &output) {
            (Some((_, i)), Some((_, o))) if i.key() != o.key() => {
                return Err(format!(
                    "convert!({target}): the input converts from `{i}` but the output \
                     converts to `{o}`; both directions must use one representation type"
                ))
            }
            (Some((_, r)), _) | (None, Some((_, r))) => r.clone(),
            (None, None) => {
                return Err(format!(
                    "convert!({target}) declares neither `.input(..)` nor `.output(..)`"
                ))
            }
        };
        Ok(ResolvedConversion {
            target,
            repr,
            input: input.map(|(s, _)| s),
            output: output.map(|(s, _)| s),
        })
    }
}

/// A conversion resolved against the model: the source type crosses as
/// its representation type, converted on the way in, on the way out, or
/// both. Both directions share the representation, so the foreign side sees
/// one type for the source type whichever way a value goes.
///
/// An adapter keeps it as its declaration for the source type, and passes
/// it to [`Input::via`](crate::Input::via) or
/// [`Output::via`](crate::Output::via) when it builds a value of that type.
#[derive(Clone, Debug)]
pub struct ResolvedConversion {
    target: TypeRef,
    repr: TypeRef,
    /// Representation → source.
    input: Option<Stage>,
    /// Source → representation.
    output: Option<Stage>,
}

impl ResolvedConversion {
    /// The representation type: what the source type crosses as.
    pub fn repr(&self) -> &TypeRef {
        &self.repr
    }

    /// The functions the conversion calls, so an adapter can count them as
    /// used.
    pub fn functions(&self) -> impl Iterator<Item = &FnRef> {
        [&self.input, &self.output]
            .into_iter()
            .flatten()
            .filter_map(|s| match &s.how {
                How::Call { fun, .. } => Some(&**fun),
                _ => None,
            })
    }

    /// The source type being converted.
    pub fn target(&self) -> &TypeRef {
        &self.target
    }

    /// The conversion call in direction `dir` applied to `value`, and
    /// whether it can fail. Fails when that direction is not declared.
    pub(crate) fn apply(
        &self,
        dir: Direction,
        q: &Qualifier<'_>,
        value: &TokenStream,
    ) -> Result<(TokenStream, bool), String> {
        let (stage, what) = match dir {
            Direction::In => (&self.input, "input"),
            Direction::Out => (&self.output, "output"),
        };
        let stage = stage
            .as_ref()
            .ok_or_else(|| format!("convert!({}) declares no {what}", self.target))?;
        Ok((
            stage.apply(q, &self.target, &self.repr, value),
            stage.fallible,
        ))
    }
}

/// Which way a conversion runs: into the source type, or out of it.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Direction {
    In,
    Out,
}

/// One direction of a conversion: how it converts. The types it converts
/// between are the conversion's.
#[derive(Clone, Debug)]
struct Stage {
    fallible: bool,
    how: How,
}

#[derive(Clone, Debug)]
enum How {
    /// Call a function; `by_ref` when it takes its argument by reference.
    Call {
        fun: Box<FnRef>,
        by_ref: bool,
    },
    From,
    Into,
    TryFrom,
    TryInto,
}

impl Stage {
    /// Read one declared direction: the stage, and the representation type
    /// it converts from (input) or to (output).
    fn resolve(
        flat: &Flat,
        via: &Via,
        target: &TypeRef,
        dir: Direction,
    ) -> Result<(Self, TypeRef), String> {
        let classify = |t: &syn::Type| {
            flat.classify(t)
                .map_err(|e| format!("conversion type `{}`: {e}", t.to_token_stream()))
        };
        let stage = |fallible, how| Stage { fallible, how };
        Ok(match via {
            Via::Fn(fun) => {
                let f = fun.resolve(flat)?;
                let name = path_string(&fun.path);
                let [param] = f.params.as_slice() else {
                    return Err(format!(
                        "conversion fn `{name}` must take exactly one argument, takes {}",
                        f.params.len()
                    ));
                };
                let (ret, fallible) = match f.ret.kind() {
                    TypeKind::Fallible { ok, .. } => ((**ok).clone(), true),
                    _ => (f.ret.clone(), false),
                };
                // The side facing the converted type must be that type: an
                // input stage returns it, an output stage takes it (or a
                // borrow of it). The other side is the representation.
                let (facing, repr) = match dir {
                    Direction::In => (&ret, param.ty.clone()),
                    Direction::Out => match param.ty.kind() {
                        TypeKind::Ref { inner, .. } => (&**inner, ret.clone()),
                        _ => (&param.ty, ret.clone()),
                    },
                };
                if facing.key() != target.key() {
                    let (what, side) = match dir {
                        Direction::In => ("input", "return"),
                        Direction::Out => ("output", "take"),
                    };
                    return Err(format!(
                        "{what} conversion fn `{name}` must {side} `{target}`, not `{facing}`"
                    ));
                }
                let by_ref =
                    dir == Direction::Out && matches!(param.ty.kind(), TypeKind::Ref { .. });
                let fun = Box::new(fun.clone());
                (stage(fallible, How::Call { fun, by_ref }), repr)
            }
            Via::From(t) => (stage(false, How::From), classify(t)?),
            Via::Into(t) => (stage(false, How::Into), classify(t)?),
            Via::TryFrom(t) => (stage(true, How::TryFrom), classify(t)?),
            Via::TryInto(t) => (stage(true, How::TryInto), classify(t)?),
        })
    }

    /// Apply the stage to `value`, between `target` and its representation
    /// `repr`. For an input stage `value` is the representation and the
    /// result the source type; for an output stage the other way round. A
    /// fallible stage produces an expression using `?` on a
    /// `Result<_, String>`.
    fn apply(
        &self,
        q: &Qualifier<'_>,
        target: &TypeRef,
        repr: &TypeRef,
        value: &TokenStream,
    ) -> TokenStream {
        let (t, r) = (q.ty(target), q.ty(repr));
        let call = match &self.how {
            How::Call { fun, by_ref } => {
                let callee = fun.callee(q);
                if *by_ref {
                    quote!(#callee(&#value))
                } else {
                    quote!(#callee(#value))
                }
            }
            How::From => quote!(<#t as ::core::convert::From<#r>>::from(#value)),
            How::Into => quote!(<#t as ::core::convert::Into<#r>>::into(#value)),
            How::TryFrom => quote!(<#t as ::core::convert::TryFrom<#r>>::try_from(#value)),
            How::TryInto => quote!(<#t as ::core::convert::TryInto<#r>>::try_into(#value)),
        };
        if self.fallible {
            quote!(#call.map_err(|__e| ::std::string::ToString::to_string(&__e))?)
        } else {
            call
        }
    }
}
