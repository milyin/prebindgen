use prebindgen_flat::{
    flat::{Function, TypeKind, TypeRef},
    Flat,
};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

use crate::{Form, FormKind, Input, Output, Qualifier, WireType};

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
    pub fn resolve(&self, flat: &Flat) -> Result<ResolvedConversion, String> {
        let target = flat
            .classify(&self.ty)
            .map_err(|e| format!("convert!({}): {e}", self.ty.to_token_stream()))?;
        let input = self
            .input
            .as_ref()
            .map(|v| Stage::resolve(flat, v, &target, Direction::In))
            .transpose()?;
        let output = self
            .output
            .as_ref()
            .map(|v| Stage::resolve(flat, v, &target, Direction::Out))
            .transpose()?;
        Ok(ResolvedConversion {
            target,
            input,
            output,
        })
    }
}

/// A conversion resolved against the model: the source type crosses as
/// a representation type, converted on the way in, on the way out, or both.
///
/// The adapter lowers the representation like any other type, inside the
/// closure it passes to [`Self::decode`] or [`Self::encode`]; the conversion
/// wraps the result and records it as [`FormKind::Via`].
#[derive(Clone, Debug)]
pub struct ResolvedConversion {
    target: TypeRef,
    /// Representation → source.
    input: Option<Stage>,
    /// Source → representation.
    output: Option<Stage>,
}

impl ResolvedConversion {
    /// The type a value arrives as, when the input direction is declared.
    pub fn input_repr(&self) -> Option<&TypeRef> {
        self.input.as_ref().map(|s| &s.repr)
    }

    /// The type a value leaves as, when the output direction is declared.
    pub fn output_repr(&self) -> Option<&TypeRef> {
        self.output.as_ref().map(|s| &s.repr)
    }

    /// The source value from its representation. `repr` lowers the
    /// representation type it is given; its input is converted here.
    /// Fails when the conversion declares no input.
    pub fn decode<W: WireType, E: From<String>>(
        &self,
        q: &Qualifier<'_>,
        repr: impl FnOnce(&TypeRef) -> Result<Input<W>, E>,
    ) -> Result<Input<W>, E> {
        let stage = self.stage(&self.input, "input")?;
        let repr = repr(&stage.repr)?;
        let applied = stage.apply(q, &quote!(__repr));
        let e = repr.expr;
        Ok(Input {
            form: self.via(repr.form),
            expr: quote!({ let __repr = #e; #applied }),
            fallible: repr.fallible || stage.fallible,
        })
    }

    /// The source `value` sent out as its representation. `repr` lowers the
    /// representation type it is given, reading the converted value from
    /// the expression it is given. Fails when the conversion declares no
    /// output.
    pub fn encode<W: WireType, E: From<String>>(
        &self,
        q: &Qualifier<'_>,
        value: &TokenStream,
        repr: impl FnOnce(&TypeRef, TokenStream) -> Result<Output<W>, E>,
    ) -> Result<Output<W>, E> {
        let stage = self.stage(&self.output, "output")?;
        let applied = stage.apply(q, value);
        let repr = repr(&stage.repr, quote!(__repr))?;
        let e = &repr.expr;
        Ok(Output {
            expr: quote!({ let __repr = #applied; #e }),
            fallible: repr.fallible || stage.fallible,
            form: self.via(repr.form),
        })
    }

    /// The functions the conversion calls, so an adapter can count them as
    /// used.
    pub fn functions(&self) -> impl Iterator<Item = &FnRef> {
        [&self.input, &self.output]
            .into_iter()
            .flatten()
            .filter_map(|s| match &s.how {
                How::Call { fun, .. } => Some(fun),
                _ => None,
            })
    }

    fn stage<'s>(&self, stage: &'s Option<Stage>, what: &str) -> Result<&'s Stage, String> {
        stage
            .as_ref()
            .ok_or_else(|| format!("convert!({}) declares no {what}", self.target))
    }

    fn via<W>(&self, repr: Form<W>) -> Form<W> {
        Form {
            ty: self.target.clone(),
            kind: FormKind::Via(Box::new(repr)),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Direction {
    In,
    Out,
}

/// One direction of a conversion.
#[derive(Clone, Debug)]
struct Stage {
    /// The type being converted.
    target: TypeRef,
    /// The representation type on the boundary side.
    repr: TypeRef,
    /// Whether the stage can fail.
    fallible: bool,
    how: How,
}

#[derive(Clone, Debug)]
enum How {
    /// Call a function; `by_ref` when it takes its argument by reference.
    Call {
        fun: FnRef,
        by_ref: bool,
    },
    From(TypeRef),
    Into(TypeRef),
    TryFrom(TypeRef),
    TryInto(TypeRef),
}

impl Stage {
    fn resolve(flat: &Flat, via: &Via, target: &TypeRef, dir: Direction) -> Result<Self, String> {
        let classify = |t: &syn::Type| {
            flat.classify(t)
                .map_err(|e| format!("conversion type `{}`: {e}", t.to_token_stream()))
        };
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
                // borrow of it).
                let facing = match dir {
                    Direction::In => &ret,
                    Direction::Out => match param.ty.kind() {
                        TypeKind::Ref { inner, .. } => &**inner,
                        _ => &param.ty,
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
                match dir {
                    Direction::In => Stage {
                        target: target.clone(),
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
                            target: target.clone(),
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
                Stage {
                    target: target.clone(),
                    repr: r.clone(),
                    fallible: false,
                    how: How::From(r),
                }
            }
            Via::Into(t) => {
                let r = classify(t)?;
                Stage {
                    target: target.clone(),
                    repr: r.clone(),
                    fallible: false,
                    how: How::Into(r),
                }
            }
            Via::TryFrom(t) => {
                let r = classify(t)?;
                Stage {
                    target: target.clone(),
                    repr: r.clone(),
                    fallible: true,
                    how: How::TryFrom(r),
                }
            }
            Via::TryInto(t) => {
                let r = classify(t)?;
                Stage {
                    target: target.clone(),
                    repr: r.clone(),
                    fallible: true,
                    how: How::TryInto(r),
                }
            }
        })
    }

    /// Apply the stage to `value`. For an input stage `value` is the
    /// representation and the result the source type; for an output stage
    /// the other way round. A fallible stage produces an expression using
    /// `?` on a `Result<_, String>`.
    fn apply(&self, q: &Qualifier<'_>, value: &TokenStream) -> TokenStream {
        let t = q.ty(&self.target);
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
