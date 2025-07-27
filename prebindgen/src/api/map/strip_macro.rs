//! Strip the specified macro attributes from the items in the source.

use crate::api::record::SourceLocation;
use roxygen::roxygen;
use std::collections::HashSet;
use syn::Item;

pub struct Builder {
    macro_attrs: HashSet<String>,
}

impl Builder {
    pub fn new() -> Self {
        Self {
            macro_attrs: HashSet::new(),
        }
    }

    /// Add a macro attribute to strip from the items.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let strip = prebindgen::StripMacros::builder()
    ///     .strip_macro("prebindgen")
    ///     .build();
    /// ```
    #[roxygen]
    pub fn strip_macro<S: Into<String>>(
        mut self,
        /// The macro attribute to strip
        macro_name: S,
    ) -> Self {
        self.macro_attrs.insert(macro_name.into());
        self
    }

    /// Build the StripMacros instance
    pub fn build(self) -> StripMacros {
        StripMacros { builder: self }
    }
}

impl Default for Builder {
    fn default() -> Self {
        Self::new()
    }
}

/// Strip specified macro attributes from the items in the source.
pub struct StripMacros {
    builder: Builder,
}

impl StripMacros {
    /// Create a builder for creating a strip macro instance
    pub fn builder() -> Builder {
        Builder::new()
    }

    // Call method to use with `map` function
    pub fn call(&self, item: (Item, SourceLocation)) -> (Item, SourceLocation) {
        let (mut item, source_location) = item;
        match &mut item {
            Item::Struct(s) => {
                self.strip_macros_from_attrs(&mut s.attrs);
            }
            Item::Enum(e) => {
                self.strip_macros_from_attrs(&mut e.attrs);
                // Also strip macros from enum variants
                for variant in &mut e.variants {
                    self.strip_macros_from_attrs(&mut variant.attrs);
                }
            }
            Item::Fn(f) => {
                self.strip_macros_from_attrs(&mut f.attrs);
            }
            _ => {}
        }
        (item, source_location)
    }

    fn strip_macros_from_attrs(&self, attrs: &mut Vec<syn::Attribute>) {
        attrs.retain(|attr| {
            if let Some(ident) = attr.path().get_ident() {
                !self.builder.macro_attrs.contains(&ident.to_string())
            } else {
                // For multi-segment paths, check each segment
                !attr.path().segments.iter().any(|segment| {
                    self.builder.macro_attrs.contains(&segment.ident.to_string())
                })
            }
        });
    }

    /// Convert to closure
    pub fn into_closure(self) -> impl FnMut((Item, SourceLocation)) -> (Item, SourceLocation) {
        move |item| self.call(item)
    }
}