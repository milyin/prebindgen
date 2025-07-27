//! Strip the specified derive attributes from the items in the source.

use crate::api::record::SourceLocation;
use roxygen::roxygen;
use std::collections::HashSet;
use syn::Item;

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
                if let syn::Meta::List(meta_list) = attr.meta.clone() {
                    let mut new_tokens = Vec::new();
                    
                    // Parse the derive list and filter out unwanted derives
                    let tokens = meta_list.tokens.clone();
                    let mut token_iter = tokens.into_iter().peekable();
                    
                    while let Some(token) = token_iter.next() {
                        if let proc_macro2::TokenTree::Ident(ident) = &token {
                            if !self.builder.derive_attrs.contains(&ident.to_string()) {
                                new_tokens.push(token);
                                // Add comma if there's more tokens and next isn't already a comma
                                if token_iter.peek().is_some() {
                                    if let Some(proc_macro2::TokenTree::Punct(punct)) = token_iter.peek() {
                                        if punct.as_char() == ',' {
                                            new_tokens.push(token_iter.next().unwrap());
                                        }
                                    }
                                }
                            } else {
                                // Skip the derive we want to remove
                                // Also skip the following comma if present
                                if let Some(proc_macro2::TokenTree::Punct(punct)) = token_iter.peek() {
                                    if punct.as_char() == ',' {
                                        token_iter.next();
                                    }
                                }
                            }
                        } else {
                            new_tokens.push(token);
                        }
                    }
                    
                    // Update the attribute with filtered derives
                    let new_tokens_stream = new_tokens.into_iter().collect();
                    attr.meta = syn::Meta::List(syn::MetaList {
                        path: meta_list.path,
                        delimiter: meta_list.delimiter,
                        tokens: new_tokens_stream,
                    });
                }
            }
        }
        
        // Remove empty derive attributes
        attrs.retain(|attr| {
            if attr.path().is_ident("derive") {
                if let syn::Meta::List(meta_list) = &attr.meta {
                    !meta_list.tokens.is_empty()
                } else {
                    true
                }
            } else {
                true
            }
        });
    }

    /// Convert to closure
    pub fn into_closure(self) -> impl FnMut((Item, SourceLocation)) -> (Item, SourceLocation) {
        move |item| self.call(item)
    }
}
