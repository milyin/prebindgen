//! Strip the specified derive attributes from the items in the source.

use crate::api::record::SourceLocation;
use roxygen::roxygen;
use std::collections::HashSet;
use syn::Item;
use quote::ToTokens;

pub struct Builder {
    derive_attrs: HashSet<String>,
}

impl Builder {
    pub fn new() -> Self {
        Self {
            derive_attrs: HashSet::new(),
        }
    }

    /// Add a derive attribute to strip from the items.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let strip = prebindgen::StripDerive::builder()
    ///     .strip_derive("Debug")
    ///     .build();
    /// ```
    #[roxygen]
    pub fn strip_derive<S: Into<String>>(
        mut self,
        /// The derive attribute to strip
        derive: S,
    ) -> Self {
        self.derive_attrs.insert(derive.into());
        self
    }

    /// Build the StripDerive instance
    pub fn build(self) -> StripDerives {
        StripDerives { builder: self }
    }
}

impl Default for Builder {
    fn default() -> Self {
        Self::new()
    }
}

/// Strip specified derive attributes from the items in the source.
pub struct StripDerives {
    builder: Builder,
}

impl StripDerives {
    /// Create a builder for creating a strip derive instance
    pub fn builder() -> Builder {
        Builder::new()
    }

    // Call method to use with `map` function
    pub fn call(&self, item: (Item, SourceLocation)) -> (Item, SourceLocation) {
        let (mut item, source_location) = item;
        match &mut item {
            Item::Struct(s) => {
                self.strip_derives_from_attrs(&mut s.attrs);
            }
            Item::Enum(e) => {
                self.strip_derives_from_attrs(&mut e.attrs);
            }
            _ => {}
        }
        (item, source_location)
    }

    fn strip_derives_from_attrs(&self, attrs: &mut Vec<syn::Attribute>) {
        for attr in attrs.iter_mut() {
            if attr.path().is_ident("derive") {
                if let Ok(list) = attr.parse_args_with(syn::punctuated::Punctuated::<syn::Path, syn::Token![,]>::parse_terminated) {
                    let filtered: syn::punctuated::Punctuated<syn::Path, syn::Token![,]> = list
                        .into_iter()
                        .filter(|path| !self.builder.derive_attrs.contains(&path.get_ident().unwrap().to_string()))
                        .collect();
                    
                    if !filtered.is_empty() {
                        attr.meta = syn::Meta::List(syn::MetaList {
                            path: attr.path().clone(),
                            delimiter: syn::MacroDelimiter::Paren(Default::default()),
                            tokens: filtered.to_token_stream(),
                        });
                    }
                }
            }
        }
        
        attrs.retain(|attr| {
            !attr.path().is_ident("derive") || 
            attr.parse_args_with(syn::punctuated::Punctuated::<syn::Path, syn::Token![,]>::parse_terminated)
                .map_or(true, |list: syn::punctuated::Punctuated<syn::Path, syn::Token![,]>| !list.is_empty())
        });
    }

    /// Convert to closure
    pub fn into_closure(self) -> impl FnMut((Item, SourceLocation)) -> (Item, SourceLocation) {
        move |item| self.call(item)
    }
}
