//! Strip the specified macro attributes from the items in the source.

use crate::api::record::SourceLocation;
use roxygen::roxygen;
use std::collections::HashSet;
use syn::Item;

/// Builder for configuring StripMacros instances
///
/// Configures which macro attributes should be stripped from types
/// to prevent compilation issues in FfiConverter output.
///
/// # Example
///
/// ```
/// let builder = prebindgen::map::strip_macro::Builder::new()
///     .strip_macro("default")
///     .strip_macro("serde")
///     .build();
/// ```
pub struct Builder {
    macro_attrs: HashSet<String>,
}

impl Builder {
    /// Create a new Builder for configuring StripMacros
    ///
    /// # Example
    ///
    /// ```
    /// let builder = prebindgen::map::strip_macro::Builder::new();
    /// ```
    pub fn new() -> Self {
        Self {
            macro_attrs: HashSet::new(),
        }
    }

    /// Add a macro attribute to strip from the items.
    ///
    /// # Example
    ///
    /// ```
    /// let strip = prebindgen::map::strip_macro::Builder::new()
    ///     .strip_macro("default")
    ///     .strip_macro("serde")
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

    /// Build the StripMacros instance with the configured options
    ///
    /// # Example
    ///
    /// ```
    /// let strip_macros = prebindgen::map::strip_macro::Builder::new()
    ///     .strip_macro("default")
    ///     .build();
    /// ```
    pub fn build(self) -> StripMacros {
        StripMacros { builder: self }
    }
}

impl Default for Builder {
    fn default() -> Self {
        Self::new()
    }
}

/// Strips macro attributes from types to prevent compilation issues in FfiConverter output
///
/// Removes macro attributes that are no longer needed or cause compilation failures
/// in the converted FFI code. This is often used in conjunction with `StripDerives`
/// to remove macros that depend on stripped derive attributes.
///
/// # Common Use Case
///
/// When `StripDerives` removes `#[derive(Default)]`, the `#[default]` attribute
/// on enum variants becomes invalid and must also be stripped to prevent compilation errors.
///
/// # Example
/// ```
/// let source = prebindgen::Source::new("source_ffi");
///
/// let strip_macros = prebindgen::map::StripMacros::builder()
///     .strip_macro("default")  // Remove #[default] from enum variants
///     .strip_macro("serde")    // Remove serde-related macros
///     .build();
/// 
/// // Apply after StripDerives to clean up orphaned macros
/// let processed_items: Vec<_> = source
///     .items_all()
///     .map(strip_macros.into_closure())
///     .take(0) // Take 0 for doctest
///     .collect();
/// ```
pub struct StripMacros {
    builder: Builder,
}

impl StripMacros {
    /// Create a builder for configuring a strip macro instance
    ///
    /// # Example
    ///
    /// ```
    /// let strip_macros = prebindgen::map::StripMacros::builder()
    ///     .strip_macro("default")
    ///     .strip_macro("serde")
    ///     .build();
    /// ```
    pub fn builder() -> Builder {
        Builder::new()
    }

    /// Process a single item to strip specified macro attributes
    ///
    /// Removes the configured macro attributes from structs, enums (including variants),
    /// and functions. Used internally by `into_closure()` for integration with `map`.
    ///
    /// # Parameters
    ///
    /// * `item` - A `(syn::Item, SourceLocation)` pair to process
    ///
    /// # Returns
    ///
    /// The same item with specified macro attributes removed
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

    /// Convert to closure compatible with `map`
    ///
    /// This is the primary method for using `StripMacros` in processing pipelines.
    /// The returned closure can be passed to `map()` to remove macro attributes
    /// from items, typically after `StripDerives` processing.
    ///
    /// # Example
    /// ```
    /// let source = prebindgen::Source::new("source_ffi");
    /// let strip_macros = prebindgen::map::StripMacros::builder()
    ///     .strip_macro("default")
    ///     .build();
    /// 
    /// // Use with map after StripDerives
    /// let processed_items: Vec<_> = source
    ///     .items_all()
    ///     .map(strip_macros.into_closure())
    ///     .take(0) // Take 0 for doctest
    ///     .collect();
    /// ```
    pub fn into_closure(self) -> impl FnMut((Item, SourceLocation)) -> (Item, SourceLocation) {
        move |item| self.call(item)
    }
}