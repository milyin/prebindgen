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
                s.attrs.retain(|attr| {
                    !attr
                        .path()
                        .segments
                        .iter()
                        .any(|seg| self.builder.derive_attrs.contains(&seg.ident.to_string()))
                });
            }
            Item::Enum(e) => {
                e.attrs.retain(|attr| {
                    !attr
                        .path()
                        .segments
                        .iter()
                        .any(|seg| self.builder.derive_attrs.contains(&seg.ident.to_string()))
                });
            }
            _ => {}
        }
        (item, source_location)
    }

    /// Convert to closure
    pub fn into_closure(self) -> impl FnMut((Item, SourceLocation)) -> (Item, SourceLocation) {
        move |item| self.call(item)
    }
}
