use prebindgen_flat::{
    flat::{Alternative, Field, Struct},
    Emit,
};
use proc_macro2::TokenStream;
use quote::{format_ident, ToTokens};

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
    /// Fields in source order.
    pub fn fields(&self) -> &'a [Field] {
        match self {
            Record::Struct(s) => &s.fields,
            Record::Alt(a) => &a.fields,
        }
    }

    /// `head { a: v0, b: v1 }` / `head(v0, v1)` / `head`, with the
    /// delimiters the source wrote.
    pub fn construct(&self, head: &TokenStream, values: &[TokenStream]) -> TokenStream {
        assert_eq!(
            self.fields().len(),
            values.len(),
            "record field count mismatch"
        );
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
