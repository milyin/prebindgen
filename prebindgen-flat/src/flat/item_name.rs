use std::fmt;

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

/// The name of a source item — a function, type or constant — and the path
/// generated code reaches it by.
///
/// The generated code lives in the binding crate, not the source crate, so a
/// reference to a source item must be qualified: `source_crate::Payload`.
/// The model resolves that path once, when it is built, from the crate that
/// captured the item (as renamed by
/// [`FlatBuilder::source_named`](crate::flat::FlatBuilder::source_named)) or
/// the builder's [`default_module`](crate::flat::FlatBuilder::default_module).
///
/// Spliced into tokens it is that qualified path, so a reference is right by
/// default. A site that defines something under the item's own name asks for
/// [`Self::ident`] instead. [`Display`](fmt::Display) prints the bare name, for
/// messages and derived identifiers.
///
/// A name relative to its parent — a parameter, a field, a sum alternative, an
/// enum value — is never qualified, and stays a [`syn::Ident`].
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ItemName {
    ident: syn::Ident,
    module: Option<syn::Path>,
}

impl ItemName {
    /// A name not yet placed in a module; the builder places it.
    pub(crate) fn bare(ident: syn::Ident) -> Self {
        Self {
            ident,
            module: None,
        }
    }

    /// An item the binding names by its full path, such as a binding-local
    /// function `crate::helpers::label_in`: the last segment is the name, the
    /// rest the module.
    pub fn from_path(path: &syn::Path) -> Self {
        let mut module = path.clone();
        let last = module
            .segments
            .pop()
            .expect("a non-empty path")
            .into_value();
        let module = (!module.segments.is_empty()).then(|| {
            // `pop` leaves the separator before the popped segment behind.
            let mut m = module;
            if let Some(pair) = m.segments.pop() {
                m.segments.push_value(pair.into_value());
            }
            m
        });
        Self {
            ident: last.ident,
            module,
        }
    }

    /// The identifier alone, for a site that defines something under the
    /// item's own name or derives one from it.
    pub fn ident(&self) -> &syn::Ident {
        &self.ident
    }

    /// Place the name in `module`, keeping a module it already has.
    pub(crate) fn place(&mut self, module: Option<&syn::Path>) {
        if self.module.is_none() {
            self.module = module.cloned();
        }
    }
}

impl ToTokens for ItemName {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ident = &self.ident;
        tokens.extend(match &self.module {
            Some(m) => quote!(#m::#ident),
            None => quote!(#ident),
        });
    }
}

impl fmt::Display for ItemName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.ident.fmt(f)
    }
}

impl PartialEq<syn::Ident> for ItemName {
    fn eq(&self, other: &syn::Ident) -> bool {
        self.ident == *other
    }
}

#[cfg(test)]
mod tests {
    use super::ItemName;

    #[test]
    fn a_path_splits_into_module_and_name() {
        let tokens = |p: syn::Path| {
            let n = ItemName::from_path(&p);
            (quote::quote!(#n).to_string(), n.to_string())
        };
        assert_eq!(
            tokens(syn::parse_quote!(crate::helpers::label_in)),
            ("crate :: helpers :: label_in".into(), "label_in".into())
        );
        assert_eq!(
            tokens(syn::parse_quote!(::other::f)),
            (":: other :: f".into(), "f".into())
        );
        assert_eq!(tokens(syn::parse_quote!(f)), ("f".into(), "f".into()));
    }
}
