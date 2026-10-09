use std::fmt;

use prebindgen_flat::flat::{Alternative, Field, Struct, TypeRef, Variant};
use proc_macro2::{Literal, TokenStream};
use quote::{quote, ToTokens};

use crate::{api::convert::Direction, Record, ResolvedConversion, Seg};

/// The adapter's closed set of boundary types — one enum per adapter.
///
/// A variant may say more than its Rust type does: JNI's `jlong` is a Kotlin
/// `Long`, a `ULong` or a handle of some class, and each is its own variant.
/// The foreign-side writer reads that meaning off the wire, so it never has to
/// repeat a decision that an override or a default made for it.
pub trait WireType: Clone + fmt::Debug {
    /// The Rust type of a slot of this kind in the generated file.
    fn rust(&self) -> TokenStream;
    /// The value a slot holds when it carries nothing: an absent optional,
    /// an alternative of a sum that was not taken.
    fn placeholder(&self) -> TokenStream;
}

/// One named slot of the generated boundary.
#[derive(Clone, Debug)]
pub struct Wire<W> {
    pub name: syn::Ident,
    pub ty: W,
}

impl<W: WireType> Wire<W> {
    pub fn new(name: syn::Ident, ty: W) -> Self {
        Self { name, ty }
    }

    /// `name: RustType`, for a parameter list or a struct body.
    pub fn decl(&self) -> TokenStream {
        let (n, t) = (&self.name, self.ty.rust());
        quote!(#n: #t)
    }
}

/// How a source value of [`Form::ty`] relates to its wires: the decision the
/// adapter made, recorded so the foreign side can follow it.
///
/// Transparent wrappers — `&T`, `Box<T>`, `Cow<T>` — leave no node: their
/// form is the form of `T`.
#[derive(Clone, Debug)]
pub struct Form<W> {
    pub ty: TypeRef,
    pub kind: FormKind<W>,
}

/// The ways a value can cross. Every one but [`FormKind::Wire`] is a
/// composition of other forms.
#[derive(Clone, Debug)]
pub enum FormKind<W> {
    /// The value is one wire.
    Wire(Wire<W>),
    /// The value is converted to another source type, which crosses as this.
    Via(Box<Form<W>>),
    /// The value is taken apart: each part crosses on its own wires, in order.
    /// A record's fields; `()` has no parts.
    Parts(Vec<(Seg, Form<W>)>),
    /// `Some(inner)` or `None`. Without a presence wire, the inner wires
    /// themselves tell absence (a null pointer).
    Optional {
        presence: Option<Wire<W>>,
        inner: Box<Form<W>>,
    },
    /// A tag selecting one alternative; every alternative's fields have wires,
    /// those not taken hold placeholders. Indexed by alternative.
    Sum {
        tag: Wire<W>,
        alts: Vec<Vec<(Seg, Form<W>)>>,
    },
    /// A sequence on `wires`; each element relates to them as `elem`. The
    /// element's own wires are per-element locals, not boundary slots.
    Seq {
        wires: Vec<Wire<W>>,
        elem: Box<Form<W>>,
    },
}

impl<W> Form<W> {
    /// The boundary slots the value occupies, in the order the Rust side
    /// reads or produces them and the foreign side passes or unpacks them.
    pub fn wires(&self) -> Vec<&Wire<W>> {
        let mut out = Vec::new();
        self.collect(&mut out);
        out
    }

    fn collect<'a>(&'a self, out: &mut Vec<&'a Wire<W>>) {
        match &self.kind {
            FormKind::Wire(w) => out.push(w),
            FormKind::Via(f) => f.collect(out),
            FormKind::Parts(parts) => parts.iter().for_each(|(_, f)| f.collect(out)),
            FormKind::Optional { presence, inner } => {
                out.extend(presence);
                inner.collect(out)
            }
            FormKind::Sum { tag, alts } => {
                out.push(tag);
                alts.iter().flatten().for_each(|(_, f)| f.collect(out))
            }
            FormKind::Seq { wires, .. } => out.extend(wires),
        }
    }
}

fn form<W>(ty: &TypeRef, kind: FormKind<W>) -> Form<W> {
    Form {
        ty: ty.clone(),
        kind,
    }
}

/// Wires → a source value: a parameter on its way into the source call, a
/// field arriving from the foreign side.
///
/// `expr` reads the wires by name and evaluates to the value. A **fallible**
/// one uses `?` on a `Result<_, String>`; whoever places it decides where the
/// error goes ([`Self::result`]).
#[derive(Clone, Debug)]
pub struct Input<W> {
    pub form: Form<W>,
    pub expr: TokenStream,
    pub fallible: bool,
}

impl<W: WireType> Input<W> {
    /// `ty` arrives on one wire; `expr` reads it.
    pub fn wire(ty: &TypeRef, wire: Wire<W>, expr: impl ToTokens) -> Self {
        Self {
            form: form(ty, FormKind::Wire(wire)),
            expr: expr.to_token_stream(),
            fallible: false,
        }
    }

    /// A converted value: it arrives as the conversion's representation
    /// type, and the conversion turns that into the source type. `repr`
    /// builds the [`Input`] of the representation type it is given. Fails
    /// when the conversion declares no input.
    pub fn via<E: From<String>>(
        conversion: &ResolvedConversion,
        repr: impl FnOnce(&TypeRef) -> Result<Input<W>, E>,
    ) -> Result<Self, E> {
        let (applied, fallible) = conversion.apply(Direction::In, &quote!(__repr))?;
        let repr = repr(conversion.repr())?;
        let e = repr.expr;
        Ok(Self {
            form: form(conversion.target(), FormKind::Via(Box::new(repr.form))),
            expr: quote!({ let __repr = #e; #applied }),
            fallible: repr.fallible || fallible,
        })
    }

    /// `ty` arrives as parts; `build` gets their values in order.
    pub fn parts(
        ty: &TypeRef,
        parts: Vec<(Seg, Input<W>)>,
        build: impl FnOnce(Vec<TokenStream>) -> TokenStream,
    ) -> Self {
        let fallible = parts.iter().any(|(_, p)| p.fallible);
        let (forms, exprs) = parts
            .into_iter()
            .map(|(s, p)| ((s, p.form), p.expr))
            .unzip();
        Self {
            form: form(ty, FormKind::Parts(forms)),
            expr: build(exprs),
            fallible,
        }
    }

    /// A struct rebuilt from its fields' inputs, in field order.
    pub fn record(s: &Struct, fields: Vec<Input<W>>) -> Self {
        let head = s.name.to_token_stream();
        let segs = s.fields.iter().map(Seg::field);
        Self::parts(s.type_ref(), segs.zip(fields).collect(), |values| {
            Record::Struct(s).construct(&head, &values)
        })
    }

    /// `Some(inner)` when `test` holds, else `None`. `presence`, when given,
    /// is the wire `test` reads; without one, `test` reads the inner wires.
    pub fn optional(
        ty: &TypeRef,
        presence: Option<Wire<W>>,
        test: impl ToTokens,
        inner: Input<W>,
    ) -> Self {
        let (e, test) = (inner.expr, test.to_token_stream());
        Self {
            form: form(
                ty,
                FormKind::Optional {
                    presence,
                    inner: Box::new(inner.form),
                },
            ),
            expr: quote!(if #test { ::core::option::Option::Some(#e) } else { ::core::option::Option::None }),
            fallible: inner.fallible,
        }
    }

    /// A sum rebuilt from the alternative `tag` selects. `alts[i]` holds the
    /// field inputs of alternative `i`; an unknown tag is an error.
    pub fn sum(v: &Variant, tag: Wire<W>, alts: Vec<Vec<Input<W>>>) -> Self {
        let head = v.name.to_token_stream();
        let t = &tag.name;
        let mut arms = Vec::new();
        let mut forms = Vec::new();
        for (alt, fields) in v.alternatives.iter().zip(alts) {
            let built = Self::alternative(&head, alt, fields, v.type_ref());
            let (i, e) = (Literal::usize_unsuffixed(alt.index), built.expr);
            arms.push(quote!(#i => ::core::result::Result::Ok(#e)));
            let FormKind::Parts(parts) = built.form.kind else {
                unreachable!("an alternative is built from parts")
            };
            forms.push(parts);
        }
        let msg = format!("{}: invalid tag {{}}", v.name);
        Self {
            form: form(
                v.type_ref(),
                FormKind::Sum {
                    tag: tag.clone(),
                    alts: forms,
                },
            ),
            expr: quote!((match #t {
                #(#arms,)*
                __t => ::core::result::Result::Err(::std::format!(#msg, __t)),
            })?),
            fallible: true,
        }
    }

    fn alternative(
        head: &TokenStream,
        alt: &Alternative,
        fields: Vec<Input<W>>,
        ty: &TypeRef,
    ) -> Self {
        let name = &alt.name;
        let head = quote!(#head::#name);
        let segs = alt.fields.iter().map(Seg::field);
        Self::parts(ty, segs.zip(fields).collect(), |values| {
            Record::Alt(alt).construct(&head, &values)
        })
    }

    /// A sequence arriving on `wires`; `expr` is the adapter's loop, `elem`
    /// the form of one element. Mark it fallible if the loop uses `?`.
    pub fn seq(ty: &TypeRef, wires: Vec<Wire<W>>, elem: Form<W>, expr: impl ToTokens) -> Self {
        Self {
            form: form(
                ty,
                FormKind::Seq {
                    wires,
                    elem: Box::new(elem),
                },
            ),
            expr: expr.to_token_stream(),
            fallible: false,
        }
    }

    /// Wrap the value, keeping the form: `Box::new(e)`, a borrow.
    pub fn map(mut self, f: impl FnOnce(TokenStream) -> TokenStream) -> Self {
        self.expr = f(self.expr);
        self
    }

    /// Wrap the value in a step using `?`.
    pub fn and_then(self, f: impl FnOnce(TokenStream) -> TokenStream) -> Self {
        self.map(f).mark_fallible()
    }

    /// Declare that [`Self::expr`] uses `?`.
    pub fn mark_fallible(mut self) -> Self {
        self.fallible = true;
        self
    }

    /// The boundary slots, in order; see [`Form::wires`].
    pub fn wires(&self) -> Vec<&Wire<W>> {
        self.form.wires()
    }

    /// The value as a `Result<T, String>` expression.
    pub fn result(&self) -> TokenStream {
        result_expr(&self.expr, self.fallible)
    }
}

/// A source value → wires: a return value, a callback argument, a field
/// leaving for the foreign side.
///
/// An output converts whatever value it is given: the template supplies the
/// value with [`Self::apply`] or [`Self::bind`], and the output supplies the
/// wires. It evaluates to the wire value, or a tuple of them in
/// [`Self::wires`] order when there are several (`()` for none). Fallibility
/// as for [`Input`].
#[derive(Clone, Debug)]
pub struct Output<W> {
    pub form: Form<W>,
    /// The conversion, reading the value by the name [`value`] gives.
    body: TokenStream,
    pub fallible: bool,
}

/// The name an output's conversion reads its value by. Private: the value is
/// bound to it by [`Output::apply`], so no template has to know it.
fn value() -> TokenStream {
    quote!(__v)
}

impl<W: WireType> Output<W> {
    /// An output of form `form`, whose conversion `body` builds from the
    /// value it is handed. For a composition the constructors below do not
    /// cover; a part's own output goes into `body` with [`Self::bind`].
    pub fn new(form: Form<W>, body: impl FnOnce(&TokenStream) -> TokenStream) -> Self {
        Self {
            form,
            body: body(&value()),
            fallible: false,
        }
    }

    /// `ty` leaves on one wire, which `body` computes from the value.
    pub fn wire(
        ty: &TypeRef,
        wire: Wire<W>,
        body: impl FnOnce(&TokenStream) -> TokenStream,
    ) -> Self {
        Self::new(form(ty, FormKind::Wire(wire)), body)
    }

    /// Nothing crosses; the value is dropped.
    pub fn unit(ty: &TypeRef) -> Self {
        Self::new(form(ty, FormKind::Parts(Vec::new())), |_| quote!(()))
    }

    /// A converted value: the conversion turns the value into its
    /// representation type, which leaves instead. `repr` builds the
    /// [`Output`] of the representation type it is given. Fails when the
    /// conversion declares no output.
    pub fn via<E: From<String>>(
        conversion: &ResolvedConversion,
        repr: impl FnOnce(&TypeRef) -> Result<Output<W>, E>,
    ) -> Result<Self, E> {
        let (applied, fallible) = conversion.apply(Direction::Out, &value())?;
        let repr = repr(conversion.repr())?;
        Ok(Self {
            body: repr.apply(applied),
            fallible: repr.fallible || fallible,
            form: form(conversion.target(), FormKind::Via(Box::new(repr.form))),
        })
    }

    /// `ty` leaves as parts. `prelude` runs first, reading the value —
    /// typically a `let` destructuring it. Each part then converts its own
    /// value: an expression valid after the prelude.
    pub fn parts(
        ty: &TypeRef,
        prelude: impl FnOnce(&TokenStream) -> TokenStream,
        parts: Vec<(Seg, TokenStream, Output<W>)>,
    ) -> Self {
        let fallible = parts.iter().any(|(_, _, p)| p.fallible);
        let binds: Vec<TokenStream> = parts.iter().map(|(_, v, p)| p.bind(v)).collect();
        let forms: Vec<(Seg, Form<W>)> = parts.into_iter().map(|(s, _, p)| (s, p.form)).collect();
        let f = form(ty, FormKind::Parts(forms));
        let values = tuple(f.wires().iter().map(|w| w.name.to_token_stream()));
        let prelude = prelude(&value());
        Self {
            form: f,
            body: quote!({ #prelude #(#binds)* #values }),
            fallible,
        }
    }

    /// A struct taken apart. `field` builds each field's output.
    pub fn record<E>(
        s: &Struct,
        mut field: impl FnMut(&Field) -> Result<Output<W>, E>,
    ) -> Result<Self, E> {
        let record = Record::Struct(s);
        let binds = record.binds();
        let mut parts = Vec::new();
        for (f, b) in s.fields.iter().zip(&binds) {
            parts.push((Seg::field(f), b.to_token_stream(), field(f)?));
        }
        let pat = record.pattern(&s.name.to_token_stream(), &binds);
        Ok(Self::parts(s.type_ref(), |v| quote!(let #pat = #v;), parts))
    }

    /// An `Option` leaving as `presence` (with the value it holds for `Some`)
    /// beside `inner`'s wires. Without `presence`, `None` is told by the
    /// inner wires' placeholders.
    pub fn optional(
        ty: &TypeRef,
        presence: Option<(Wire<W>, TokenStream)>,
        inner: Output<W>,
    ) -> Self {
        let (presence, yes) = presence.unzip();
        let wires: Vec<&Wire<W>> = presence.iter().chain(inner.wires()).collect();
        let some = tuple(
            yes.into_iter()
                .chain(inner.wires().iter().map(|w| w.name.to_token_stream())),
        );
        let none = tuple(wires.iter().map(|w| w.ty.placeholder()));
        let bind = inner.bind(quote!(__some));
        let x = value();
        Self {
            body: quote!(match #x {
                ::core::option::Option::Some(__some) => { #bind #some }
                ::core::option::Option::None => #none,
            }),
            fallible: inner.fallible,
            form: form(
                ty,
                FormKind::Optional {
                    presence,
                    inner: Box::new(inner.form),
                },
            ),
        }
    }

    /// A sum leaving as `tag` plus every alternative's wires; the taken
    /// alternative fills its own, the others hold placeholders. `field`
    /// builds each field's output.
    pub fn sum<E>(
        v: &Variant,
        tag: Wire<W>,
        mut field: impl FnMut(&Alternative, &Field) -> Result<Output<W>, E>,
    ) -> Result<Self, E> {
        let head = v.name.to_token_stream();
        let mut alts = Vec::new();
        for alt in &v.alternatives {
            let record = Record::Alt(alt);
            let binds = record.binds();
            let mut outs = Vec::new();
            for (f, b) in alt.fields.iter().zip(&binds) {
                outs.push((Seg::field(f), b.to_token_stream(), field(alt, f)?));
            }
            let name = &alt.name;
            alts.push((record.pattern(&quote!(#head::#name), &binds), outs));
        }
        let fallible = alts.iter().flat_map(|(_, o)| o).any(|(_, _, o)| o.fallible);
        let arms = alts.iter().enumerate().map(|(i, (pat, outs))| {
            let binds = outs.iter().map(|(_, b, o)| o.bind(b));
            let tag = Literal::usize_unsuffixed(i);
            let values = alts.iter().enumerate().flat_map(|(j, (_, other))| {
                other.iter().flat_map(|(_, _, o)| o.wires()).map(move |w| {
                    if i == j {
                        w.name.to_token_stream()
                    } else {
                        w.ty.placeholder()
                    }
                })
            });
            let values = tuple(std::iter::once(quote!(#tag)).chain(values));
            quote!(#pat => { #(#binds)* #values })
        });
        let x = value();
        let body = quote!(match #x { #(#arms),* });
        let alts = alts
            .into_iter()
            .map(|(_, outs)| outs.into_iter().map(|(s, _, o)| (s, o.form)).collect())
            .collect();
        Ok(Self {
            form: form(v.type_ref(), FormKind::Sum { tag, alts }),
            body,
            fallible,
        })
    }

    /// A sequence leaving on `wires`; `body` is the adapter's loop over the
    /// value, `elem` the form of one element. Mark it fallible if the loop
    /// uses `?`.
    pub fn seq(
        ty: &TypeRef,
        wires: Vec<Wire<W>>,
        elem: Form<W>,
        body: impl FnOnce(&TokenStream) -> TokenStream,
    ) -> Self {
        let f = form(
            ty,
            FormKind::Seq {
                wires,
                elem: Box::new(elem),
            },
        );
        Self::new(f, body)
    }

    /// This output over a value `f` derives from the one given: `(*v)` for
    /// a `Box`, a clone for a borrow. The mirror of [`Input::map`].
    pub fn before(mut self, f: impl FnOnce(&TokenStream) -> TokenStream) -> Self {
        let (x, body) = (value(), &self.body);
        let derived = f(&x);
        // An identity derivation changes nothing.
        if derived.to_string() != x.to_string() {
            self.body = quote!({ let #x = #derived; #body });
        }
        self
    }

    /// Declare that the conversion uses `?`.
    pub fn mark_fallible(mut self) -> Self {
        self.fallible = true;
        self
    }

    /// The boundary slots, in order; see [`Form::wires`].
    pub fn wires(&self) -> Vec<&Wire<W>> {
        self.form.wires()
    }

    /// The conversion of `value`: an expression evaluating to the wires. The
    /// value is evaluated once.
    pub fn apply(&self, value: impl ToTokens) -> TokenStream {
        let (x, body) = (self::value(), &self.body);
        let value = value.to_token_stream();
        // A nested output handed the value under the very name it reads.
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
        tuple(self.wires().iter().map(|w| w.name.to_token_stream()))
    }

    /// The conversion of `value` as a `Result<_, String>` expression.
    pub fn result(&self, value: impl ToTokens) -> TokenStream {
        result_expr(&self.apply(value), self.fallible)
    }
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
