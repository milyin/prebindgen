use std::fmt;

use prebindgen_flat::flat::{Alternative, Field, Struct, TypeRef, Variant};
use proc_macro2::{Literal, TokenStream};
use quote::{quote, ToTokens};

use crate::{Qualifier, Record, Seg};

/// The adapter's closed set of boundary types — one enum per adapter.
///
/// A variant may say more than its Rust type does: JNI's `jlong` is a Kotlin
/// `Long`, a `ULong` or a handle of some class, and each is its own variant.
/// The foreign-side writer reads that meaning off the wire, so it never has to
/// repeat a decision that an override or a default made for it.
pub trait WireType: Clone + fmt::Debug {
    /// The Rust type of a slot of this kind in the generated file.
    fn rust_type(&self) -> syn::Type;
    /// An inert value for an absent optional or inactive alternative.
    ///
    /// Return `None` if this wire has no supported placeholder. Combinators
    /// that need one then return an error. Placeholder wires must never be
    /// decoded as active values; handles must not allocate resources here.
    fn placeholder(&self) -> Option<TokenStream> {
        None
    }
}

/// One named slot of the generated boundary.
#[derive(Clone, Debug)]
pub struct Wire<W> {
    /// Binding name within the generated element.
    pub name: syn::Ident,
    /// The generator's wire variant, including destination semantics.
    pub ty: W,
}

impl<W: WireType> Wire<W> {
    /// Pair a generated name with one of the generator's supported wire types.
    pub fn new(name: syn::Ident, ty: W) -> Self {
        Self { name, ty }
    }

    /// `name: RustType`, for a parameter list or a struct body.
    pub fn decl(&self) -> TokenStream {
        let (n, t) = (&self.name, self.ty.rust_type());
        quote!(#n: #t)
    }
}

/// How a source value of [`Form::ty`] relates to its wires: the decision the
/// adapter made, recorded so the foreign side can follow it.
///
/// Transparent wrappers must retain the original source type. An adapter
/// can use `map_type` to wrap a child conversion while retaining its wires.
#[derive(Clone, Debug)]
pub struct Form<W> {
    /// The flat type represented at this node.
    pub ty: TypeRef,
    /// The selected relation to boundary slots or child forms.
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
    /// The boundary wires, in order.
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
    /// Shared structure for the destination writer.
    pub form: Form<W>,
    /// Wires-to-flat-value Rust expression.
    pub expr: TokenStream,
    /// Whether the expression uses `?` with a string error.
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
    /// Panics on missing, extra, or mismatched field types (a policy bug).
    pub fn record(q: &Qualifier<'_>, s: &Struct, fields: Vec<Input<W>>) -> Self {
        assert_fields(&s.fields, &fields);
        let head = q.path(&s.name);
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
        let prebindgen_flat::flat::TypeKind::Optional(expected) = ty.kind() else {
            panic!("optional input requires an Option type");
        };
        assert_eq!(
            expected.key(),
            inner.form.ty.key(),
            "optional input type mismatch"
        );
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
    pub fn sum(q: &Qualifier<'_>, v: &Variant, tag: Wire<W>, alts: Vec<Vec<Input<W>>>) -> Self {
        assert_eq!(
            v.alternatives.len(),
            alts.len(),
            "sum alternative count mismatch"
        );
        let head = q.path(&v.name);
        let t = &tag.name;
        let mut arms = Vec::new();
        let mut forms = Vec::new();
        for (alt, fields) in v.alternatives.iter().zip(alts) {
            let built = Self::alternative(&head, alt, fields, v.type_ref());
            let (i, e) = (Literal::usize_unsuffixed(alt.index), built.expr);
            arms.push(quote!(#i => ::core::result::Result::Ok(#e)));
            let FormKind::Parts(parts) = built.form.kind else {
                unreachable!()
            };
            forms.push(parts);
        }
        let msg = format!("invalid tag {{}} for `{}`", v.name);
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
        assert_fields(&alt.fields, &fields);
        let name = &alt.name;
        let head = quote!(#head::#name);
        let segs = alt.fields.iter().map(Seg::field);
        Self::parts(ty, segs.zip(fields).collect(), |values| {
            Record::Alt(alt).construct(&head, &values)
        })
    }

    /// A sequence arriving on `wires`; `expr` is the adapter's loop, `elem`
    /// the form of one element.
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
            fallible: true,
        }
    }

    /// Wrap the value, keeping the form: `Box::new(e)`, a borrow.
    pub fn map(mut self, f: impl FnOnce(TokenStream) -> TokenStream) -> Self {
        self.expr = f(self.expr);
        self
    }

    /// Wrap a value into `ty`, preserving the selected child relation.
    /// Use for `Box::new`, borrowing, or other transparent source wrappers.
    pub fn map_type(mut self, ty: &TypeRef, f: impl FnOnce(TokenStream) -> TokenStream) -> Self {
        self.expr = f(self.expr);
        self.form = form(ty, FormKind::Via(Box::new(self.form)));
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

    /// Boundary slots read by this expression, in order.
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
/// `expr` evaluates to the wire value, or a tuple of them in order when
/// there are several (`()` for none). Fallibility as for [`Input`].
#[derive(Clone, Debug)]
pub struct Output<W> {
    /// Shared structure for the destination writer.
    pub form: Form<W>,
    /// Flat-value-to-wires Rust expression: `()`, a bare value, or a tuple.
    pub expr: TokenStream,
    /// Whether the expression uses `?` with a string error.
    pub fallible: bool,
}

impl<W: WireType> Output<W> {
    /// `ty` leaves on one wire, computed by `expr`.
    pub fn wire(ty: &TypeRef, wire: Wire<W>, expr: impl ToTokens) -> Self {
        Self {
            form: form(ty, FormKind::Wire(wire)),
            expr: expr.to_token_stream(),
            fallible: false,
        }
    }

    /// Nothing crosses; `value` is dropped.
    pub fn unit(ty: &TypeRef, value: impl ToTokens) -> Self {
        let v = value.to_token_stream();
        Self {
            form: form(ty, FormKind::Parts(Vec::new())),
            expr: quote!({ let _ = #v; }),
            fallible: false,
        }
    }

    /// `ty` leaves as parts. `prelude` runs first — typically a `let`
    /// destructuring the value the parts read.
    pub fn parts(ty: &TypeRef, prelude: TokenStream, parts: Vec<(Seg, Output<W>)>) -> Self {
        let fallible = parts.iter().any(|(_, p)| p.fallible);
        let binds: Vec<TokenStream> = parts.iter().map(|(_, p)| p.bind()).collect();
        let forms: Vec<(Seg, Form<W>)> = parts.into_iter().map(|(s, p)| (s, p.form)).collect();
        let f = form(ty, FormKind::Parts(forms));
        let values = tuple(f.wires().iter().map(|w| w.name.to_token_stream()));
        Self {
            form: f,
            expr: quote!({ #prelude #(#binds)* #values }),
            fallible,
        }
    }

    /// A struct taken apart. `field` builds each field's output from the
    /// binding that holds it.
    pub fn record<E>(
        q: &Qualifier<'_>,
        s: &Struct,
        value: &TokenStream,
        mut field: impl FnMut(&Field, TokenStream) -> Result<Output<W>, E>,
    ) -> Result<Self, E> {
        let record = Record::Struct(s);
        let binds = record.binds();
        let mut parts = Vec::new();
        for (f, b) in s.fields.iter().zip(&binds) {
            let out = field(f, b.to_token_stream())?;
            assert_eq!(
                f.ty.key(),
                out.form.ty.key(),
                "record output field type mismatch"
            );
            parts.push((Seg::field(f), out));
        }
        let pat = record.pattern(&q.path(&s.name), &binds);
        Ok(Self::parts(s.type_ref(), quote!(let #pat = #value;), parts))
    }

    /// `value: Option<_>` leaving as `presence` (with the value it holds for
    /// `Some`) beside the inner wires. Without `presence`, `None` is told by
    /// the inner wires' placeholders. `inner` builds from the `Some` binding.
    pub fn optional(
        ty: &TypeRef,
        presence: Option<(Wire<W>, TokenStream)>,
        value: &TokenStream,
        inner: impl FnOnce(TokenStream) -> Result<Output<W>, String>,
    ) -> Result<Self, String> {
        let inner = inner(quote!(__some))?;
        let prebindgen_flat::flat::TypeKind::Optional(expected) = ty.kind() else {
            return Err("optional output requires an Option type".into());
        };
        if expected.key() != inner.form.ty.key() {
            return Err("optional output type mismatch".into());
        }
        let (presence, yes) = presence.unzip();
        let wires: Vec<&Wire<W>> = presence.iter().chain(inner.wires()).collect();
        let some = tuple(
            yes.into_iter()
                .chain(inner.wires().iter().map(|w| w.name.to_token_stream())),
        );
        let none = tuple(
            wires
                .iter()
                .map(|w| placeholder(w))
                .collect::<Result<Vec<_>, _>>()?,
        );
        let bind = inner.bind();
        Ok(Self {
            expr: quote!(match #value {
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
        })
    }

    /// A sum leaving as `tag` plus every alternative's wires; the taken
    /// alternative fills its own, the others hold placeholders. `field`
    /// builds each field's output from the binding that holds it.
    pub fn sum(
        q: &Qualifier<'_>,
        v: &Variant,
        tag: Wire<W>,
        value: &TokenStream,
        mut field: impl FnMut(&Alternative, &Field, TokenStream) -> Result<Output<W>, String>,
    ) -> Result<Self, String> {
        let head = q.path(&v.name);
        let mut alts = Vec::new();
        for alt in &v.alternatives {
            let record = Record::Alt(alt);
            let binds = record.binds();
            let mut outs = Vec::new();
            for (f, b) in alt.fields.iter().zip(&binds) {
                let out = field(alt, f, b.to_token_stream())?;
                if f.ty.key() != out.form.ty.key() {
                    return Err("sum output field type mismatch".into());
                }
                outs.push((Seg::field(f), out));
            }
            let name = &alt.name;
            alts.push((record.pattern(&quote!(#head::#name), &binds), outs));
        }
        let fallible = alts.iter().flat_map(|(_, o)| o).any(|(_, o)| o.fallible);
        let arms = alts
            .iter()
            .enumerate()
            .map(|(i, (pat, outs))| {
                let binds = outs.iter().map(|(_, o)| o.bind());
                let tag = Literal::usize_unsuffixed(i);
                let values = alts.iter().enumerate().flat_map(|(j, (_, other))| {
                    other.iter().flat_map(|(_, o)| o.wires()).map(move |w| {
                        if i == j {
                            Ok(w.name.to_token_stream())
                        } else {
                            placeholder(w)
                        }
                    })
                });
                let values = values.collect::<Result<Vec<_>, String>>()?;
                let values = tuple(std::iter::once(quote!(#tag)).chain(values));
                Ok(quote!(#pat => { #(#binds)* #values }))
            })
            .collect::<Result<Vec<_>, String>>()?;
        let expr = quote!(match #value { #(#arms),* });
        let alts = alts
            .into_iter()
            .map(|(_, outs)| outs.into_iter().map(|(s, o)| (s, o.form)).collect())
            .collect();
        Ok(Self {
            form: form(v.type_ref(), FormKind::Sum { tag, alts }),
            expr,
            fallible,
        })
    }

    /// A sequence leaving on `wires`; `expr` is the adapter's loop, `elem`
    /// the form of one element.
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
            fallible: true,
        }
    }

    /// Declare that [`Self::expr`] uses `?`.
    pub fn mark_fallible(mut self) -> Self {
        self.fallible = true;
        self
    }

    /// Boundary slots produced by this expression, in order.
    pub fn wires(&self) -> Vec<&Wire<W>> {
        self.form.wires()
    }

    /// `let <wires> = expr;` — binds every wire by its own name.
    pub fn bind(&self) -> TokenStream {
        let (pat, e) = (self.pattern(), &self.expr);
        quote!(let #pat = #e;)
    }

    /// The binding pattern for the wires: `w`, `(a, b)`, or `()`.
    pub fn pattern(&self) -> TokenStream {
        tuple(self.wires().iter().map(|w| w.name.to_token_stream()))
    }

    /// The wires as a `Result<_, String>` expression.
    pub fn result(&self) -> TokenStream {
        result_expr(&self.expr, self.fallible)
    }
}

fn placeholder<W: WireType>(wire: &Wire<W>) -> Result<TokenStream, String> {
    wire.ty
        .placeholder()
        .ok_or_else(|| format!("wire `{}` has no placeholder", wire.name))
}

fn assert_fields<W>(fields: &[Field], inputs: &[Input<W>]) {
    assert_eq!(fields.len(), inputs.len(), "record field count mismatch");
    for (field, input) in fields.iter().zip(inputs) {
        assert_eq!(
            field.ty.key(),
            input.form.ty.key(),
            "record field type mismatch"
        );
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
