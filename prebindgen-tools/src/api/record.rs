use prebindgen_flat::{
    flat::{Alternative, Field, Struct, Variant},
    Emit,
};
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};

use crate::wire::{Input, Output, Wire};

/// A struct or one alternative of a sum: something with fields and
/// delimiters.
#[derive(Clone, Copy)]
pub enum Record<'a> {
    /// A source struct's fields and delimiters.
    Struct(&'a Struct),
    /// One sum alternative's fields and delimiters.
    Alt(&'a Alternative),
}

impl<'a> Record<'a> {
    /// Fields in source order. [`record_in`] and [`record_out`] preserve it.
    pub fn fields(&self) -> &'a [Field] {
        match self {
            Record::Struct(s) => &s.fields,
            Record::Alt(a) => &a.fields,
        }
    }

    /// `head { a: v0, b: v1 }` / `head(v0, v1)` / `head`, with the
    /// delimiters the source wrote.
    pub fn construct(&self, head: &TokenStream, values: &[TokenStream]) -> TokenStream {
        let parts: Vec<TokenStream> = self
            .fields()
            .iter()
            .zip(values)
            .map(|(f, v)| f.bind(v))
            .collect();
        self.shape(head, &parts)
    }

    /// The pattern binding each field to the matching ident in `binds`.
    pub fn pattern(&self, head: &TokenStream, binds: &[syn::Ident]) -> TokenStream {
        self.construct(
            head,
            &binds
                .iter()
                .map(|b| b.to_token_stream())
                .collect::<Vec<_>>(),
        )
    }

    /// Fresh binding names for the fields: `__f0`, `__f1`, …
    pub fn binds(&self) -> Vec<syn::Ident> {
        (0..self.fields().len())
            .map(|i| format_ident!("__f{}", i))
            .collect()
    }

    fn shape(&self, head: &TokenStream, parts: &[TokenStream]) -> TokenStream {
        let emit = Emit::new();
        match self {
            Record::Struct(s) => emit.shape(*s, head.clone(), parts),
            Record::Alt(a) => emit.shape(*a, head.clone(), parts),
        }
    }
}

/// The adapter's decision for one field.
pub trait FieldCallbacks {
    /// Why this adapter could not lower a field.
    type Error;

    /// The wires `field` arrives on, and how they rebuild it.
    fn field_in(&mut self, field: &Field) -> Result<Input, Self::Error>;

    /// The wires `field` leaves on, reading the field's value from `value`
    /// (an owned binding).
    fn field_out(&mut self, field: &Field, value: &TokenStream) -> Result<Output, Self::Error>;
}

/// A record rebuilt from its fields' inputs: wires concatenated in field
/// order, values placed into the record's constructor `head`.
pub fn record_in(record: Record<'_>, head: &TokenStream, fields: Vec<Input>) -> Input {
    Input::combine(fields, |values| record.construct(head, &values))
}

/// A record taken apart into its fields' outputs. `src` is the record
/// value; `field` builds each field's output from the binding holding it.
pub fn record_out<E>(
    record: Record<'_>,
    head: &TokenStream,
    src: &TokenStream,
    mut field: impl FnMut(&Field, &TokenStream) -> Result<Output, E>,
) -> Result<Output, E> {
    let binds = record.binds();
    let mut parts = Vec::new();
    for (f, b) in record.fields().iter().zip(&binds) {
        parts.push(field(f, &b.to_token_stream())?);
    }
    let pat = record.pattern(head, &binds);
    Ok(Output::concat(quote!(let #pat = #src;), parts))
}

/// A `#[repr(C)]`-style mirror of a source struct and its two conversions.
///
/// Add [`Self::def`] to a [`RustFile`](crate::RustFile). The [`Self::input`]
/// and [`Self::output`] expressions refer to a mirror or source value named
/// `v`, respectively; embed them in the adapter's conversion functions.
pub struct StructMirror {
    /// The mirror's definition.
    pub def: TokenStream,
    /// The mirror's fields, in order.
    pub wires: Vec<Wire>,
    /// Mirror value bound to `v` → source value.
    pub input: Input,
    /// Source value bound to `v` → mirror value.
    pub output: Output,
}

/// Writes a mirror struct for a source struct.
pub struct StructWriter<'s> {
    source: &'s Struct,
    name: syn::Ident,
    source_path: TokenStream,
    attrs: Vec<TokenStream>,
}

impl<'s> StructWriter<'s> {
    /// A mirror of `source` (named by `source_path` in generated code),
    /// called `name`.
    pub fn new(source: &'s Struct, source_path: impl ToTokens, name: syn::Ident) -> Self {
        Self {
            source,
            name,
            source_path: source_path.to_token_stream(),
            attrs: Vec::new(),
        }
    }

    /// An attribute on the mirror (`#[repr(C)]`, `#[derive(..)]`).
    pub fn attr(mut self, attr: impl ToTokens) -> Self {
        self.attrs.push(attr.to_token_stream());
        self
    }

    /// Ask `cb` for each field's input and output, then return the mirror
    /// definition and conversions. See [`StructMirror`] for how to place them.
    pub fn write<C: FieldCallbacks>(self, cb: &mut C) -> Result<StructMirror, C::Error> {
        let record = Record::Struct(self.source);
        let mut ins = Vec::new();
        let mut outs = Vec::new();
        let binds = record.binds();
        for (f, b) in self.source.fields.iter().zip(&binds) {
            ins.push(cb.field_in(f)?);
            outs.push(cb.field_out(f, &b.to_token_stream())?);
        }
        let name = &self.name;
        let wires: Vec<Wire> = ins.iter().flat_map(|i| i.wires.clone()).collect();
        let decls = wires.iter().map(|w| {
            let d = w.decl();
            quote!(pub #d)
        });
        let attrs = &self.attrs;
        let def = quote! {
            #(#attrs)*
            pub struct #name { #(#decls),* }
        };
        let wire_names: Vec<&syn::Ident> = wires.iter().map(|w| &w.name).collect();
        let head = &self.source_path;
        let rebuilt = record_in(record, head, ins);
        let input = Input {
            wires: Vec::new(),
            expr: {
                let e = &rebuilt.expr;
                quote!({ let #name { #(#wire_names),* } = v; #e })
            },
            fallible: rebuilt.fallible,
            pass: None,
        };
        let pat = record.pattern(head, &binds);
        let fallible = outs.iter().any(|o| o.fallible);
        let out_binds: Vec<TokenStream> = outs.iter().map(Output::bind).collect();
        let output = Output {
            wires: Vec::new(),
            expr: quote!({ let #pat = v; #(#out_binds)* #name { #(#wire_names),* } }),
            fallible,
        };
        Ok(StructMirror {
            def,
            wires,
            input,
            output,
        })
    }
}

/// A mirror of a source sum: an enum whose alternatives carry the wires.
///
/// Add [`Self::def`] to a [`RustFile`](crate::RustFile). The input and output
/// expressions refer to a value named `v`, just as in [`StructMirror`].
pub struct SumMirror {
    /// The mirror enum definition.
    pub def: TokenStream,
    /// Per alternative, its payload wires.
    pub alternatives: Vec<Vec<Wire>>,
    /// Mirror value bound to `v` → source value (a `match`).
    pub input: Input,
    /// Source value bound to `v` → mirror value (a `match`).
    pub output: Output,
}

/// Writes a mirror enum for a source sum.
///
/// Each alternative retains the source's unit, tuple, or named-field form.
/// Supply one [`Wire`] per field through [`FieldCallbacks`] so the mirror's
/// payload still matches the source alternative's field shape.
///
/// ```
/// use prebindgen::SourceLocation;
/// use prebindgen_tools::{flat::{Flat, flat::{Field, Type}},
///     FieldCallbacks, Input, Output, SumWriter, Wire, ident};
/// use proc_macro2::TokenStream;
/// use quote::{format_ident, quote};
///
/// let item = syn::parse_quote!(pub enum Code { Empty, Number(i32) });
/// let flat = Flat::builder()
///     .items([(item, SourceLocation::default())])
///     .build().unwrap();
/// let Some(Type::Variant(code)) = flat.declared_type("Code") else { panic!() };
/// struct Scalar;
/// impl FieldCallbacks for Scalar {
///     type Error = String;
///     fn field_in(&mut self, field: &Field) -> Result<Input, String> {
///         Ok(Input::identity(Wire::new(format_ident!("w{}", field.index), quote!(i32))))
///     }
///     fn field_out(&mut self, field: &Field, value: &TokenStream)
///         -> Result<Output, String> {
///         Ok(Output::single(
///             Wire::new(format_ident!("w{}", field.index), quote!(i32)), value
///         ))
///     }
/// }
/// let mirror = SumWriter::new(code, quote!(Code), ident!(CodeWire))
///     .write(&mut Scalar).unwrap();
/// assert!(mirror.def.to_string().contains("enum CodeWire"));
/// assert_eq!(mirror.alternatives[0].len(), 0);
/// assert_eq!(mirror.alternatives[1].len(), 1);
/// ```
pub struct SumWriter<'v> {
    source: &'v Variant,
    name: syn::Ident,
    source_path: TokenStream,
    attrs: Vec<TokenStream>,
}

impl<'v> SumWriter<'v> {
    /// A mirror of `source` (named by `source_path` in generated code),
    /// called `name`.
    pub fn new(source: &'v Variant, source_path: impl ToTokens, name: syn::Ident) -> Self {
        Self {
            source,
            name,
            source_path: source_path.to_token_stream(),
            attrs: Vec::new(),
        }
    }

    /// An attribute on the mirror enum, such as `#[repr(C)]`.
    pub fn attr(mut self, attr: impl ToTokens) -> Self {
        self.attrs.push(attr.to_token_stream());
        self
    }

    /// Ask `cb` about each alternative's fields and return the mirror
    /// definition and conversions. See [`SumMirror`] for how to place them.
    /// Each field must lower to one wire for the mirror's field shape.
    pub fn write<C: FieldCallbacks>(self, cb: &mut C) -> Result<SumMirror, C::Error> {
        let name = &self.name;
        let src = &self.source_path;
        let mut variants = Vec::new();
        let mut in_arms = Vec::new();
        let mut out_arms = Vec::new();
        let mut alternatives = Vec::new();
        let mut in_fallible = false;
        let mut out_fallible = false;
        for alt in &self.source.alternatives {
            let record = Record::Alt(alt);
            let aname = &alt.name;
            let binds = record.binds();
            let mut ins = Vec::new();
            let mut outs = Vec::new();
            for (f, b) in alt.fields.iter().zip(&binds) {
                ins.push(cb.field_in(f)?);
                outs.push(cb.field_out(f, &b.to_token_stream())?);
            }
            // A mirror alternative has one mirror field per source field; a
            // field that needs several wires is kept whole as a tuple would
            // lose the per-field names C sees, so it is refused here by
            // construction: the adapter hands one wire per field.
            let wires: Vec<Wire> = ins.iter().flat_map(|i| i.wires.clone()).collect();
            let mirror_head = quote!(#name::#aname);
            let source_head = quote!(#src::#aname);
            // Declaration: same delimiters as the source, wire types in place.
            let tys: Vec<TokenStream> = wires.iter().map(|w| w.ty.clone()).collect();
            let decl = record.construct(&quote!(#aname), &tys);
            variants.push(decl);
            // In: match the mirror alternative, binding its wires by name.
            let wire_binds: Vec<syn::Ident> = wires.iter().map(|w| w.name.clone()).collect();
            let mirror_pat = record.pattern(&mirror_head, &wire_binds);
            let rebuilt = record_in(record, &source_head, ins);
            in_fallible |= rebuilt.fallible;
            let e = &rebuilt.expr;
            in_arms.push(quote!(#mirror_pat => #e));
            // Out: match the source alternative, produce the mirror one.
            let source_pat = record.pattern(&source_head, &binds);
            out_fallible |= outs.iter().any(|o| o.fallible);
            let out_binds: Vec<TokenStream> = outs.iter().map(Output::bind).collect();
            let values: Vec<TokenStream> = wires.iter().map(|w| w.name.to_token_stream()).collect();
            let rebuilt_mirror = record.construct(&mirror_head, &values);
            out_arms.push(quote!(#source_pat => { #(#out_binds)* #rebuilt_mirror }));
            alternatives.push(wires);
        }
        let attrs = &self.attrs;
        let def = quote! {
            #(#attrs)*
            pub enum #name { #(#variants),* }
        };
        Ok(SumMirror {
            def,
            alternatives,
            input: Input {
                wires: Vec::new(),
                expr: quote!(match v { #(#in_arms),* }),
                fallible: in_fallible,
                pass: None,
            },
            output: Output {
                wires: Vec::new(),
                expr: quote!(match v { #(#out_arms),* }),
                fallible: out_fallible,
            },
        })
    }
}
