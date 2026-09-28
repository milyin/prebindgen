use prebindgen_flat::{
    flat::{Alternative, Field, Struct},
    Emit,
};
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};

use crate::wire::{Input, Output};

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
