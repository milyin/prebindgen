//! Filter items in a sequence of (syn::Item, SourceLocation) by replacing specific types
//! Mostly useful for explicitly appending full paths to types, e.g.
//! replacing `Option` with `std::option::Option` or `MaybeUninit` with `std::mem::MaybeUninit`

use crate::api::record::SourceLocation;
use std::collections::HashMap;
use syn::{visit_mut::VisitMut, Item, Type, TypePath};

/// Builder for configuring ReplaceTypes instances
///
/// Configures type name replacements. While any type can be replaced with any other,
/// the primary use case is replacing import-dependent names with fully qualified paths.
///
/// # Example
///
/// ```
/// let builder = prebindgen::map::replace_types::Builder::new()
///     .replace_type("Option", "std::option::Option")
///     .replace_type("mem::MaybeUninit", "std::mem::MaybeUninit")
///     .replace_type("Vec", "std::vec::Vec")
///     .build();
/// ```
pub struct Builder {
    type_replacements: HashMap<String, String>,
}

impl Builder {
    /// Create a new Builder for configuring ReplaceTypes
    ///
    /// # Example
    ///
    /// ```
    /// let builder = prebindgen::map::replace_types::Builder::new();
    /// ```
    pub fn new() -> Self {
        Self {
            type_replacements: HashMap::new(),
        }
    }

    /// Add a type replacement mapping
    ///
    /// # Parameters
    ///
    /// * `from` - The type name to replace (e.g., "Option", "mem::MaybeUninit", "MyType")
    /// * `to` - The replacement type (e.g., "std::option::Option", "std::mem::MaybeUninit", "crate::NewType")
    ///
    /// # Example
    ///
    /// ```
    /// let builder = prebindgen::map::replace_types::Builder::new()
    ///     .replace_type("Option", "std::option::Option")
    ///     .replace_type("mem::MaybeUninit", "std::mem::MaybeUninit");
    /// ```
    pub fn replace_type<S: Into<String>>(mut self, from: S, to: S) -> Self {
        self.type_replacements.insert(from.into(), to.into());
        self
    }

    /// Build the ReplaceTypes instance with the configured options
    ///
    /// # Example
    ///
    /// ```
    /// let replacer = prebindgen::map::replace_types::Builder::new()
    ///     .replace_type("Option", "std::option::Option")
    ///     .build();
    /// ```
    pub fn build(self) -> ReplaceTypes {
        ReplaceTypes { builder: self }
    }
}

impl Default for Builder {
    fn default() -> Self {
        Self::new()
    }
}

/// Replaces type names throughout items with specified alternatives
///
/// While `ReplaceTypes` can perform any type name replacement, its primary purpose is
/// converting import-dependent type names to fully qualified paths. When the source crate
/// uses imports like `use std::mem; mem::MaybeUninit`, the generated FFI code would also
/// need the same imports to compile. `ReplaceTypes` converts these to self-contained
/// fully qualified names.
///
/// # Problem
///
/// Source crate code:
/// ```rust,ignore
/// use std::mem;
///
/// #[prebindgen]
/// pub fn process(data: &mut mem::MaybeUninit<i32>) { /* ... */ }
/// ```
///
/// Without replacement, generated code would require:
/// ```rust,ignore
/// use std::mem;  // User must add this import
/// include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
/// ```
///
/// # Solution
///
/// Replace `mem::MaybeUninit` with `std::mem::MaybeUninit` so the generated code
/// is self-contained and doesn't require additional imports.
///
/// # Example
/// ```
/// let source = prebindgen::Source::new("source_ffi");
/// # let source = prebindgen::Source::doctest_simulate();
///
/// let type_replacer = prebindgen::map::ReplaceTypes::builder()
///     .replace_type("Option", "std::option::Option")
///     .replace_type("mem::MaybeUninit", "std::mem::MaybeUninit")
///     .build();
///
/// // Apply before FfiConverter to ensure fully qualified types
/// let processed_items: Vec<_> = source
///     .items_all()
///     .map(type_replacer.into_closure())
///     .take(0) // Take 0 for doctest
///     .collect();
/// ```
pub struct ReplaceTypes {
    builder: Builder,
}

impl ReplaceTypes {
    /// Create a builder for configuring a replace types instance
    ///
    /// # Example
    ///
    /// ```
    /// let replacer = prebindgen::map::ReplaceTypes::builder()
    ///     .replace_type("Option", "std::option::Option")
    ///     .replace_type("mem::MaybeUninit", "std::mem::MaybeUninit")
    ///     .build();
    /// ```
    pub fn builder() -> Builder {
        Builder::new()
    }

    /// Process a single item to replace specified type names
    ///
    /// Replaces configured type names with their specified alternatives throughout
    /// the item. Used internally by `into_closure()` for integration with `map`.
    ///
    /// # Parameters
    ///
    /// * `item` - A `(syn::Item, SourceLocation)` pair to process
    ///
    /// # Returns
    ///
    /// The same item with type names replaced
    pub fn call(&self, item: (Item, SourceLocation)) -> (Item, SourceLocation) {
        let (mut item, source_location) = item;
        let mut visitor = TypeReplacer {
            replacements: &self.builder.type_replacements,
        };
        visitor.visit_item_mut(&mut item);
        (item, source_location)
    }

    /// Convert to closure compatible with `map`
    ///
    /// This is the primary method for using `ReplaceTypes` in processing pipelines.
    /// The returned closure can be passed to `map()` to replace type names with
    /// their configured alternatives, typically before `FfiConverter` processing.
    ///
    /// # Example
    ///
    /// ```
    /// let source = prebindgen::Source::new("source_ffi");
    /// # let source = prebindgen::Source::doctest_simulate();
    /// let type_replacer = prebindgen::map::ReplaceTypes::builder()
    ///     .replace_type("Option", "std::option::Option")
    ///     .build();
    ///
    /// // Use with map before FfiConverter
    /// let processed_items: Vec<_> = source
    ///     .items_all()
    ///     .map(type_replacer.into_closure())
    ///     .take(0) // Take 0 for doctest
    ///     .collect();
    /// ```
    pub fn into_closure(self) -> impl FnMut((Item, SourceLocation)) -> (Item, SourceLocation) {
        move |item| self.call(item)
    }
}

struct TypeReplacer<'a> {
    replacements: &'a HashMap<String, String>,
}

impl<'a> VisitMut for TypeReplacer<'a> {
    fn visit_type_mut(&mut self, ty: &mut Type) {
        if let Type::Path(TypePath { path, .. }) = ty {
            let current_path = path
                .segments
                .iter()
                .map(|seg| seg.ident.to_string())
                .collect::<Vec<_>>()
                .join("::");

            if let Some(replacement) = self.replacements.get(&current_path) {
                if let Ok(Type::Path(TypePath { path: new_path, .. })) =
                    syn::parse_str::<Type>(replacement)
                {
                    // Preserve and process generic arguments
                    let mut original_args = path.segments.last().unwrap().arguments.clone();
                    if let syn::PathArguments::AngleBracketed(ref mut args) = original_args {
                        for arg in &mut args.args {
                            if let syn::GenericArgument::Type(inner_ty) = arg {
                                self.visit_type_mut(inner_ty);
                            }
                        }
                    }

                    // Apply replacement with preserved arguments
                    path.segments = new_path.segments;
                    if let Some(last_segment) = path.segments.last_mut() {
                        last_segment.arguments = original_args;
                    }
                    return;
                }
            }
        }

        syn::visit_mut::visit_type_mut(self, ty);
    }
}
