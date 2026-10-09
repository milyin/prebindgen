use proc_macro2::{Literal, TokenStream};
use quote::{quote, ToTokens};

use crate::{
    api::{
        convert::Direction as ConvDirection,
        crossing::{
            Alternatives, Arm, Crossing, In, Node, Optional, Out, Presence, Sequence, Wrapper,
        },
    },
    Record, Wire, WireType,
};

/// An expression an adapter writes for one step: what a wire holds, a
/// sequence's loop. Fallible when it uses `?` on a `Result<_, String>`.
#[derive(Clone, Debug)]
pub struct Code {
    expr: TokenStream,
    fallible: bool,
}

impl Code {
    pub fn new(expr: impl ToTokens) -> Self {
        Self {
            expr: expr.to_token_stream(),
            fallible: false,
        }
    }

    /// An expression using `?` on a `Result<_, String>`.
    pub fn fallible(expr: impl ToTokens) -> Self {
        Self {
            fallible: true,
            ..Self::new(expr)
        }
    }
}

/// The Rust that rebuilds a source value from its wires, read by name: what
/// [`Crossing::decode`] makes of an [`In`] crossing.
#[derive(Clone, Debug)]
pub struct Input {
    expr: TokenStream,
    fallible: bool,
}

impl Input {
    fn of(code: Code) -> Self {
        Self {
            expr: code.expr,
            fallible: code.fallible,
        }
    }

    /// Evaluates to the value.
    pub fn expr(&self) -> &TokenStream {
        &self.expr
    }

    /// Whether [`Self::expr`] uses `?` on a `Result<_, String>`.
    pub fn is_fallible(&self) -> bool {
        self.fallible
    }

    /// The value as a `Result<T, String>` expression.
    pub fn result(&self) -> TokenStream {
        result_expr(&self.expr, self.fallible)
    }

    /// Wrap the value: `Box::new(e)`.
    pub fn map(mut self, f: impl FnOnce(&TokenStream) -> TokenStream) -> Self {
        self.expr = f(&self.expr);
        self
    }

    /// Wrap the value in a step using `?`.
    pub fn and_then(mut self, f: impl FnOnce(&TokenStream) -> TokenStream) -> Self {
        self.expr = f(&self.expr);
        self.fallible = true;
        self
    }
}

/// The Rust that turns a source value into its wires: what
/// [`Crossing::encode`] makes of an [`Out`] crossing. It converts whatever
/// value it is given ([`Self::apply`], [`Self::bind`]) and evaluates to the
/// wire values, a tuple of them in wire order when there are several.
#[derive(Clone, Debug)]
pub struct Output {
    names: Vec<syn::Ident>,
    /// Reads the value by the name [`value`] gives.
    body: TokenStream,
    fallible: bool,
}

/// The name the Rust leaving Rust reads its value by: what the code of a
/// [`Whole`](crate::Whole) out of Rust converts.
pub fn out_value() -> TokenStream {
    quote!(__v)
}

fn value() -> TokenStream {
    out_value()
}

impl Output {
    fn over<W>(wires: &[&Wire<W>], code: Code) -> Self {
        Self {
            names: wires.iter().map(|w| w.name.clone()).collect(),
            body: code.expr,
            fallible: code.fallible,
        }
    }

    /// Whether the conversion uses `?` on a `Result<_, String>`.
    pub fn is_fallible(&self) -> bool {
        self.fallible
    }

    /// This output over a value `f` derives from the one given: `(*v)` for a
    /// `Box`, a clone for a borrow.
    pub fn before(mut self, f: impl FnOnce(&TokenStream) -> TokenStream) -> Self {
        let (x, body) = (value(), &self.body);
        let derived = f(&x);
        if derived.to_string() != x.to_string() {
            self.body = quote!({ let #x = #derived; #body });
        }
        self
    }

    /// The conversion of `value`: evaluates to the wires. `value` is
    /// evaluated once.
    pub fn apply(&self, value: impl ToTokens) -> TokenStream {
        let (x, body) = (self::value(), &self.body);
        let value = value.to_token_stream();
        if value.to_string() == x.to_string() {
            return quote!({ #body });
        }
        quote!({ let #x = #value; #body })
    }

    /// `let <wires> = …;` — converts `value`, binding every wire by its own
    /// name.
    pub fn bind(&self, value: impl ToTokens) -> TokenStream {
        let (pat, e) = (self.pattern(), self.apply(value));
        quote!(let #pat = #e;)
    }

    /// The binding pattern for the wires: `w`, `(a, b)`, or `()`.
    pub fn pattern(&self) -> TokenStream {
        tuple(self.names.iter().map(|n| n.to_token_stream()))
    }

    /// The conversion of `value` as a `Result<_, String>` expression.
    pub fn result(&self, value: impl ToTokens) -> TokenStream {
        result_expr(&self.apply(value), self.fallible)
    }
}

/// What the adapter writes when a crossing is turned into Rust that reads
/// wires: everything a [`Crossing`] leaves to the boundary.
pub trait Decode<'f, W: WireType> {
    type Error: From<String>;

    /// The test that the option `at` is present, reading its flag or its
    /// inner wires.
    fn is_present(
        &self,
        at: &Crossing<'f, W, In>,
        opt: &Optional<'f, W, In>,
    ) -> Result<TokenStream, Self::Error>;

    /// The sequence `at` rebuilt from its wires; `elem` reads one element
    /// from the locals the loop binds.
    fn sequence(
        &self,
        at: &Crossing<'f, W, In>,
        seq: &Sequence<'f, W, In>,
        elem: Input,
    ) -> Result<Code, Self::Error>;

    /// The wrapper's value, given what it wraps. By default a `Box` boxes,
    /// a `Cow` owns, and a borrow passes the value for the caller to lend.
    fn wrapped(
        &self,
        _at: &Crossing<'f, W, In>,
        wrapper: Wrapper,
        inner: Input,
    ) -> Result<Input, Self::Error> {
        Ok(match wrapper {
            Wrapper::Box => inner.map(|e| quote!(::std::boxed::Box::new(#e))),
            Wrapper::Cow => inner.map(|e| quote!(::std::borrow::Cow::Owned(#e))),
            Wrapper::Ref(_) => inner,
        })
    }

    /// A converted value's representation, before the conversion runs on
    /// it — the place for a check of its domain. As is by default.
    fn representation(&self, _at: &Crossing<'f, W, In>, repr: Input) -> Result<Input, Self::Error> {
        Ok(repr)
    }
}

/// What the adapter writes when a crossing is turned into Rust that
/// produces wires.
pub trait Encode<'f, W: WireType> {
    type Error: From<String>;

    /// The value of the option `at`'s presence flag when it is present.
    fn present(&self, at: &Crossing<'f, W, Out>, flag: &Wire<W>) -> TokenStream;

    /// The sequence `value` written to its wires; `elem` converts one
    /// element, binding its wires as locals.
    fn sequence(
        &self,
        at: &Crossing<'f, W, Out>,
        seq: &Sequence<'f, W, Out>,
        elem: &Output,
        value: &TokenStream,
    ) -> Result<Code, Self::Error>;

    /// The output of the wrapper, given that of what it wraps. By default
    /// a `Box` is dereferenced, a borrow cloned, a `Cow` owned.
    fn wrapped(
        &self,
        _at: &Crossing<'f, W, Out>,
        wrapper: Wrapper,
        inner: Output,
    ) -> Result<Output, Self::Error> {
        Ok(match wrapper {
            Wrapper::Box => inner.before(|v| quote!((*#v))),
            Wrapper::Ref(_) => inner.before(|v| quote!(::core::clone::Clone::clone(#v))),
            Wrapper::Cow => inner.before(|v| quote!(#v.into_owned())),
        })
    }

    /// A converted value's representation output, which runs after the
    /// conversion — the place for a check of its domain. As is by default.
    fn representation(
        &self,
        _at: &Crossing<'f, W, Out>,
        repr: Output,
    ) -> Result<Output, Self::Error> {
        Ok(repr)
    }
}

impl<'f, W: WireType> Crossing<'f, W, In> {
    /// The Rust that rebuilds this value from its wires.
    pub fn decode<C: Decode<'f, W>>(&self, codec: &C) -> Result<Input, C::Error> {
        Ok(match self.node() {
            Node::Whole(w) => Input::of(w.code.clone()),
            Node::Unit => Input::of(Code::new(quote!(()))),
            Node::Fields(f) => {
                let values = f
                    .fields()
                    .map(|(_, c)| c.decode(codec))
                    .collect::<Result<Vec<_>, _>>()?;
                let head = f.item().name.to_token_stream();
                construct(Record::Struct(f.item()), &head, values)
            }
            Node::Alternatives(a) => {
                let t = &a.tag().name;
                let mut arms = Vec::new();
                for (arm, built) in a.arms().iter().zip(a.decode_each(codec)?) {
                    let i = Literal::usize_unsuffixed(arm.alternative().index);
                    let e = built.expr;
                    arms.push(quote!(#i => ::core::result::Result::Ok(#e)));
                }
                let msg = format!("{}: invalid tag {{}}", a.item().name);
                Input::of(Code::fallible(quote!((match #t {
                    #(#arms,)*
                    __t => ::core::result::Result::Err(::std::format!(#msg, __t)),
                })?)))
            }
            Node::Converted(c) => {
                let repr = codec.representation(self, c.repr().decode(codec)?)?;
                let (applied, fallible) =
                    c.conversion().apply(ConvDirection::In, &quote!(__repr))?;
                let e = repr.expr;
                Input {
                    expr: quote!({ let __repr = #e; #applied }),
                    fallible: repr.fallible || fallible,
                }
            }
            Node::Optional(o) => {
                let test = codec.is_present(self, o)?;
                let inner = o.inner().decode(codec)?;
                let e = inner.expr;
                Input {
                    expr: quote!(if #test { ::core::option::Option::Some(#e) } else { ::core::option::Option::None }),
                    fallible: inner.fallible,
                }
            }
            Node::Sequence(s) => {
                let elem = s.elem().decode(codec)?;
                Input::of(codec.sequence(self, s, elem)?)
            }
            Node::Wrapped(w) => {
                let inner = w.inner().decode(codec)?;
                codec.wrapped(self, w.wrapper(), inner)?
            }
            Node::Constructed(c) => {
                let args = c
                    .args()
                    .map(|(_, a)| a.decode(codec))
                    .collect::<Result<Vec<_>, _>>()?;
                let fallible = args.iter().any(|a| a.fallible);
                let exprs = args.iter().map(|a| &a.expr);
                let callee = &c.func().name;
                let call = quote!(#callee(#(#exprs),*));
                match c.func().ret.kind() {
                    prebindgen_flat::flat::TypeKind::Fallible { .. } => Input::of(Code::fallible(
                        quote!(#call.map_err(|__e| ::std::string::ToString::to_string(&__e))?),
                    )),
                    _ => Input {
                        expr: call,
                        fallible,
                    },
                }
            }
        })
    }
}

impl<'f, W: WireType> Alternatives<'f, W, In> {
    /// Each alternative rebuilt from its fields' wires, in order: the arms of
    /// a match on the tag, or on a mirror of the sum.
    pub fn decode_each<C: Decode<'f, W>>(&self, codec: &C) -> Result<Vec<Input>, C::Error> {
        self.arms()
            .iter()
            .map(|arm| arm.decode(&self.item().name.to_token_stream(), codec))
            .collect()
    }
}

impl<'f, W: WireType> Arm<'f, W, In> {
    /// This alternative rebuilt from its fields' wires; `sum` is the path of
    /// the sum it belongs to.
    pub fn decode<C: Decode<'f, W>>(
        &self,
        sum: &TokenStream,
        codec: &C,
    ) -> Result<Input, C::Error> {
        let values = self
            .fields()
            .map(|(_, c)| c.decode(codec))
            .collect::<Result<Vec<_>, _>>()?;
        let name = &self.alternative().name;
        Ok(construct(
            Record::Alt(self.alternative()),
            &quote!(#sum::#name),
            values,
        ))
    }
}

fn construct(record: Record<'_>, head: &TokenStream, values: Vec<Input>) -> Input {
    let fallible = values.iter().any(|v| v.fallible);
    let exprs: Vec<TokenStream> = values.into_iter().map(|v| v.expr).collect();
    Input {
        expr: record.construct(head, &exprs),
        fallible,
    }
}

/// One alternative taken apart: the pattern matching it, which binds its
/// fields, and the statements binding each field's wires by name.
pub struct ArmOutput {
    pub pattern: TokenStream,
    pub binds: TokenStream,
    pub fallible: bool,
}

impl<'f, W: WireType> Crossing<'f, W, Out> {
    /// The Rust that turns this value into its wires.
    pub fn encode<C: Encode<'f, W>>(&self, codec: &C) -> Result<Output, C::Error> {
        let wires = self.wires();
        let x = value();
        Ok(match self.node() {
            Node::Whole(w) => Output::over(&wires, w.code.clone()),
            Node::Unit => Output::over(&wires, Code::new(quote!(()))),
            Node::Fields(f) => {
                let record = Record::Struct(f.item());
                let binds = record.binds();
                let mut stmts = Vec::new();
                let mut fallible = false;
                for ((_, c), b) in f.fields().zip(&binds) {
                    let out = c.encode(codec)?;
                    fallible |= out.fallible;
                    stmts.push(out.bind(b));
                }
                let pat = record.pattern(&f.item().name.to_token_stream(), &binds);
                let values = tuple(wires.iter().map(|w| w.name.to_token_stream()));
                Output {
                    names: names(&wires),
                    body: quote!({ let #pat = #x; #(#stmts)* #values }),
                    fallible,
                }
            }
            Node::Alternatives(a) => {
                let arms = a.encode_each(codec)?;
                let fallible = arms.iter().any(|arm| arm.fallible);
                let cases = a.arms().iter().zip(&arms).enumerate().map(|(i, (_, out))| {
                    let tag = Literal::usize_unsuffixed(i);
                    let values = a.arms().iter().enumerate().flat_map(|(j, other)| {
                        other.wires().into_iter().map(move |w| {
                            if i == j {
                                w.name.to_token_stream()
                            } else {
                                w.ty.placeholder()
                            }
                        })
                    });
                    let values = tuple(std::iter::once(quote!(#tag)).chain(values));
                    let (pat, binds) = (&out.pattern, &out.binds);
                    quote!(#pat => { #binds #values })
                });
                Output {
                    names: names(&wires),
                    body: quote!(match #x { #(#cases),* }),
                    fallible,
                }
            }
            Node::Converted(c) => {
                let (applied, fallible) = c.conversion().apply(ConvDirection::Out, &x)?;
                let repr = codec.representation(self, c.repr().encode(codec)?)?;
                Output {
                    names: names(&wires),
                    body: repr.apply(applied),
                    fallible: repr.fallible || fallible,
                }
            }
            Node::Optional(o) => {
                let inner = o.inner().encode(codec)?;
                let flag = match o.presence() {
                    Presence::Flag(w) => Some((w, codec.present(self, w))),
                    Presence::Niche => None,
                };
                let inner_names = inner.names.iter().map(|n| n.to_token_stream());
                let some = tuple(flag.iter().map(|(_, v)| v.clone()).chain(inner_names));
                let none = tuple(wires.iter().map(|w| w.ty.placeholder()));
                let bind = inner.bind(quote!(__some));
                Output {
                    names: names(&wires),
                    body: quote!(match #x {
                        ::core::option::Option::Some(__some) => { #bind #some }
                        ::core::option::Option::None => #none,
                    }),
                    fallible: inner.fallible,
                }
            }
            Node::Sequence(s) => {
                let elem = s.elem().encode(codec)?;
                Output::over(&wires, codec.sequence(self, s, &elem, &x)?)
            }
            Node::Wrapped(w) => {
                let inner = w.inner().encode(codec)?;
                codec.wrapped(self, w.wrapper(), inner)?
            }
            Node::Constructed(never) => match *never {},
        })
    }
}

impl<'f, W: WireType> Alternatives<'f, W, Out> {
    /// Each alternative taken apart, in order: the arms of a match on the
    /// source value.
    pub fn encode_each<C: Encode<'f, W>>(&self, codec: &C) -> Result<Vec<ArmOutput>, C::Error> {
        self.arms()
            .iter()
            .map(|arm| arm.encode(&self.item().name.to_token_stream(), codec))
            .collect()
    }
}

impl<'f, W: WireType> Arm<'f, W, Out> {
    /// This alternative taken apart; `sum` is the path of the sum it
    /// belongs to.
    pub fn encode<C: Encode<'f, W>>(
        &self,
        sum: &TokenStream,
        codec: &C,
    ) -> Result<ArmOutput, C::Error> {
        let record = Record::Alt(self.alternative());
        let binds = record.binds();
        let mut stmts = Vec::new();
        let mut fallible = false;
        for ((_, c), b) in self.fields().zip(&binds) {
            let out = c.encode(codec)?;
            fallible |= out.fallible;
            stmts.push(out.bind(b));
        }
        let name = &self.alternative().name;
        Ok(ArmOutput {
            pattern: record.pattern(&quote!(#sum::#name), &binds),
            binds: quote!(#(#stmts)*),
            fallible,
        })
    }
}

fn names<W>(wires: &[&Wire<W>]) -> Vec<syn::Ident> {
    wires.iter().map(|w| w.name.clone()).collect()
}

/// `a` for one item, `(a, b)` otherwise.
fn tuple(items: impl IntoIterator<Item = TokenStream>) -> TokenStream {
    let items: Vec<TokenStream> = items.into_iter().collect();
    match items.as_slice() {
        [one] => one.clone(),
        _ => quote!((#(#items),*)),
    }
}

/// `expr` as a `Result<_, String>`: an immediately-invoked closure when it
/// uses `?`, an `Ok` otherwise.
pub fn result_expr(expr: &TokenStream, fallible: bool) -> TokenStream {
    if fallible {
        quote!((|| -> ::core::result::Result<_, ::std::string::String> {
            ::core::result::Result::Ok(#expr)
        })())
    } else {
        quote!(::core::result::Result::<_, ::std::string::String>::Ok(#expr))
    }
}
