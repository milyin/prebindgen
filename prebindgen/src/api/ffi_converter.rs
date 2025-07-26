//! The core functionality of Prebinding library: generation of FFI rust source from items marked with
//! `#[prebindgen]` attribute.
//!
//! The concept is following: to make FFI interface to the some rust library, you need to create two crates:
//!
//! -- library_ffi crate which exports set of repr-C structures and set of functions with `#[prebindgen]` attribute.
//! Important:  these functions are not `extern "C"`, they are just regular Rust functions
//! -- library_binding crate (e.g. library_c or library_cs)
//! This crate uses `include!` macro to include source file which
//! contains copies of the structures above and `#[no_mangle] extern "C"` proxy functions which call the
//! original functions from library_ffi crate.
//!
//! This allows to
//! - have a single ffi implementation and reuse it for different language bindings without squashing all to single crate
//! - adapt the ffi source to specificity of different binding generators

use roxygen::roxygen;
use std::collections::{HashMap, HashSet};

use crate::{
    codegen::replace_types::{
        convert_to_stub, generate_standard_allowed_prefixes, generate_type_transmute_pair_assertions, replace_types_in_item, ParseConfig, TypeTransmutePair
    }, SourceLocation
};

/// Builder for configuring RustFfi without file operations
pub struct Builder {
    pub(crate) source_crate_name: String,
    pub(crate) allowed_prefixes: Vec<syn::Path>,
    pub(crate) transparent_wrappers: Vec<syn::Path>,
    pub(crate) edition: String,
}

impl Builder {
    /// Create a new RustFfi builder
    pub fn new(source_crate_name: impl Into<String>) -> Self {
        // Generate comprehensive allowed prefixes including standard prelude
        Self {
            source_crate_name: source_crate_name.into(),
            allowed_prefixes: generate_standard_allowed_prefixes(),
            transparent_wrappers: Vec::new(),
            edition: "2021".to_string(),
        }
    }

    /// Add an allowed type prefix for FFI validation
    ///
    /// This method allows you to specify additional type prefixes that should be
    /// considered valid for FFI functions, beyond the comprehensive set of default
    /// allowed prefixes that includes the standard prelude, core library types,
    /// primitive types, and common FFI types.
    ///
    /// # Default Allowed Prefixes
    ///
    /// The builder automatically includes prefixes for:
    /// - Standard library modules (`std`, `core`, `alloc`)
    /// - Standard prelude types (`Option`, `Result`, `Vec`, `String`, etc.)
    /// - Core library modules (`core::mem`, `core::ptr`, etc.)
    /// - Primitive types (`bool`, `i32`, `u64`, etc.)
    /// - Common FFI types (`libc`, `c_char`, `c_int`, etc.)
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let builder = prebindgen::Builder::new(path)
    ///     .allowed_prefix("libc")
    ///     .allowed_prefix("core");
    /// ```
    #[roxygen]
    pub fn allowed_prefix<S: AsRef<str>>(
        mut self,
        /// The additional type prefix to allow
        prefix: S,
    ) -> Self {
        let path: syn::Path = syn::parse_str(prefix.as_ref()).unwrap();
        self.allowed_prefixes.push(path);
        self
    }

    /// Add a transparent wrapper type to be stripped from FFI function parameters
    ///
    /// Transparent wrappers are types that wrap other types but have the same
    /// memory layout (like `std::mem::MaybeUninit<T>`). When generating FFI stubs,
    /// these wrappers will be stripped from parameter types to create simpler
    /// C-compatible function signatures.
    ///
    /// For example, if you add `std::mem::MaybeUninit` as a transparent wrapper:
    /// - `&mut std::mem::MaybeUninit<Foo>` becomes `*mut Foo` in the FFI signature
    /// - `&std::mem::MaybeUninit<Bar>` becomes `*const Bar` in the FFI signature
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let builder = prebindgen::Builder::new(path)
    ///     .strip_transparent_wrapper("std::mem::MaybeUninit")
    ///     .strip_transparent_wrapper("std::mem::ManuallyDrop");
    /// ```
    #[roxygen]
    pub fn strip_transparent_wrapper<S: AsRef<str>>(
        mut self,
        /// The transparent wrapper type to strip (e.g., "std::mem::MaybeUninit")
        wrapper: S,
    ) -> Self {
        let path: syn::Path = syn::parse_str(wrapper.as_ref()).unwrap();
        self.transparent_wrappers.push(path);
        self
    }

    /// Set the Rust edition to use for generated code
    ///
    /// This affects how the `#[no_mangle]` attribute is generated:
    /// - For edition "2024": `#[unsafe(no_mangle)]`
    /// - For other editions: `#[no_mangle]`
    ///
    /// Default is "2024" if not specified.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let builder = prebindgen::Builder::new(path)
    ///     .edition("2021");
    /// ```
    #[roxygen]
    pub fn edition<S: Into<String>>(
        mut self,
        /// The Rust edition ("2021", "2024", etc.)
        edition: S,
    ) -> Self {
        self.edition = edition.into();
        self
    }

    /// Build the FfiConverter instance
    pub fn build(self) -> FfiConverter {
        FfiConverter {
            builder: self,
            stage: GenerationStage::Collect,
            source_items: Vec::new(),
            type_replacements: HashMap::new(),
            exported_types: HashSet::new(),
            followup_items: Vec::new(),
        }
    }
}

enum GenerationStage {
    /// First stage: go through all items in the iterator and collect all type names.
    /// These types will be copied to the destination file and transmuted to the original crate types when
    /// performing the ffi calls
    Collect,
    /// Second stage: copy $[prebindgen] marked types and functions to the destination file
    /// with necessary type name adjustments and generating function stubs
    Convert,
    /// Third stage: generate assertions for correctness of type transmute operations
    Followup,
}

/// Ffi structure that mirrors Prebindgen functionality without file operations
pub struct FfiConverter {
    pub(crate) builder: Builder,
    /// Current generation stage
    stage: GenerationStage,
    /// Items read from the source iterator
    source_items: Vec<(syn::Item, SourceLocation)>,
    /// Copied types which needs transmute operations - filled on `Collect` stage and used on `Convert` stage
    exported_types: HashSet<String>,
    /// Type replacements made - filled on `Convert` stage and used to prepare assertion items for `Followup` stage
    type_replacements: HashMap<TypeTransmutePair, SourceLocation>,
    /// Items which are output in the end
    followup_items: Vec<(syn::Item, SourceLocation)>,
}

impl FfiConverter {
    fn collect_item(&mut self, item: syn::Item, source_location: SourceLocation) {
        // Update exported_types for type items
        match &item {
            syn::Item::Struct(s) => {
                self.exported_types.insert(s.ident.to_string());
            }
            syn::Item::Enum(e) => {
                self.exported_types.insert(e.ident.to_string());
            }
            syn::Item::Union(u) => {
                self.exported_types.insert(u.ident.to_string());
            }
            syn::Item::Type(t) => {
                self.exported_types.insert(t.ident.to_string());
            }
            _ => {}
        }

        // Store the item and its source location
        self.source_items.push((item, source_location));
    }

    fn convert(&mut self) -> Option<(syn::Item, SourceLocation)> {
        if let Some((mut item, source_location)) = self.source_items.pop() {
            // Create parse config
            let config = ParseConfig {
                crate_name: &self.builder.source_crate_name,
                exported_types: &self.exported_types,
                allowed_prefixes: &self.builder.allowed_prefixes,
                transparent_wrappers: &self.builder.transparent_wrappers,
                edition: &self.builder.edition,
            };

            // Process based on item type
            match item {
                syn::Item::Fn(ref mut function) => {
                    // Convert function to FFI stub
                    if let Err(e) = convert_to_stub(
                        function,
                        &config,
                        &mut self.type_replacements,
                        &source_location,
                    ) {
                        panic!(
                            "Failed to convert function {function}{source_location}: {e}",
                            function = function.sig.ident,
                        );
                    }
                }
                _ => {
                    // Replace types in non-function items
                    let _ = replace_types_in_item(
                        &mut item,
                        &config,
                        &mut self.type_replacements,
                        &source_location,
                    );
                }
            }

            return Some((item, source_location));
        }

        None
    }

    fn generate_assertions(&mut self) {
        // Generate assertions for type transmute correctness
        for (replacement, source_location) in &self.type_replacements {
            if let Some((size_assertion, align_assertion)) =
                generate_type_transmute_pair_assertions(replacement)
            {
                self.followup_items
                    .push((size_assertion, source_location.clone()));
                self.followup_items.push((align_assertion, source_location.clone()));
            }
        }
    }

    /// Call method for use with batching - wrap in closure: |iter| rust_ffi.call(iter)
    pub fn call<I>(&mut self, iter: &mut I) -> Option<(syn::Item, SourceLocation)>
    where
        I: Iterator<Item = (syn::Item, SourceLocation)>,
    {
        loop {
            match self.stage {
                GenerationStage::Collect => {
                    // Collect stage: collect type names and prepare for conversion
                    // Consumes the iterator until the end and swtitches to Convert stage
                    if let Some((item, source_location)) = iter.next() {
                        self.collect_item(item, source_location);
                    } else {
                        self.stage = GenerationStage::Convert;
                    }
                }
                GenerationStage::Convert => {
                    // Convert stage: process items harvested on the Collect stage one by one.
                    // If all items are processed, generate assertions and switch to Followup stage
                    if let Some((item, source_location)) = self.convert() {
                        return Some((item, source_location));
                    } else {
                        self.generate_assertions();
                        self.stage = GenerationStage::Followup;
                    }
                }
                GenerationStage::Followup => {
                    // Followup stage: return items generated in the previous stages
                    if let Some((item, source_location)) = self.followup_items.pop() {
                        return Some((item, source_location));
                    } else {
                        return None; // No more items to return
                    }
                }
            }
        }
    }

    /// Convert to closure compatible with batching
    pub fn into_closure<I>(
        mut self,
    ) -> impl FnMut(&mut I) -> Option<(syn::Item, SourceLocation)>
    where
        I: Iterator<Item = (syn::Item, SourceLocation)>,
    {
        move |iter| self.call(iter)
    }
}
