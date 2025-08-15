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
use quote::ToTokens;

use crate::{
    codegen::replace_types::{
        convert_to_stub, generate_standard_allowed_prefixes,
        generate_type_transmute_pair_assertions, replace_types_in_item, ParseConfig,
        TypeTransmutePair,
    },
    RustEdition, SourceLocation,
};

/// Builder for configuring FfiConverter instances
///
/// Configures how prebindgen items are converted into FFI-compatible Rust code
/// with options for type handling, wrapper stripping, and code generation.
///
/// # Example
///
/// ```
/// let builder = prebindgen::batching::ffi_converter::Builder::new("example_ffi")
///     .edition(prebindgen::RustEdition::Edition2024)
///     .strip_transparent_wrapper("std::mem::MaybeUninit")
///     .strip_transparent_wrapper("std::option::Option")
///     .allowed_prefix("libc")
///     .prefixed_exported_type("foo::Foo")
///     .build();
/// ```
pub struct Builder {
    pub(crate) source_crate_name: String,
    pub(crate) allowed_prefixes: Vec<syn::Path>,
    pub(crate) prefixed_exported_types: Vec<syn::Path>,
    pub(crate) transparent_wrappers: Vec<syn::Path>,
    pub(crate) edition: RustEdition,
}

impl Builder {
    /// Create a new Builder for configuring FfiConverter
    ///
    /// # Parameters
    ///
    /// * `source_crate_name` - Name of the source crate containing `#[prebindgen]` items
    ///
    /// # Example
    ///
    /// ```
    /// let builder = prebindgen::batching::ffi_converter::Builder::new("example_ffi");
    /// ```
    pub fn new(source_crate_name: impl Into<String>) -> Self {
        Self {
            source_crate_name: source_crate_name.into(),
            allowed_prefixes: generate_standard_allowed_prefixes(),
            prefixed_exported_types: Vec::new(),
            transparent_wrappers: Vec::new(),
            edition: RustEdition::default(),
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
    /// ```
    /// let builder = prebindgen::batching::ffi_converter::Builder::new("example_ffi")
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

    /// Add a transparent wrapper type to be stripped from structure fields and
    /// FFI function parameters
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
    /// ```
    /// let builder = prebindgen::batching::ffi_converter::Builder::new("example_ffi")
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

    /// Add a prefixed exported type that should have its module path stripped
    ///
    /// All `#[prebindgen]`-marked types are copied into the destination file without
    /// their module structure (flattened). However, when the generated FFI functions
    /// reference these types, they may still use the original module path from the
    /// source crate. This method tells the converter to strip the module prefix
    /// when accessing the flattened type in the destination.
    ///
    /// # Example
    ///
    /// In source crate:
    /// ```rust,ignore
    /// pub mod foo {
    ///     #[prebindgen]
    ///     pub struct Foo { /* ... */ }
    /// }
    ///
    /// #[prebindgen]
    /// pub fn process_foo(f: &foo::Foo) -> i32 { /* ... */ }
    /// ```
    ///
    /// In destination, without `prefixed_exported_type("foo::Foo")`:
    /// - Type `Foo` is copied (flattened)
    /// - Function still references `foo::Foo` (compilation error)
    ///
    /// With `prefixed_exported_type("foo::Foo")`:
    /// - Type `Foo` is copied (flattened)
    /// - Function references are changed from `foo::Foo` to `Foo`
    ///
    /// ```
    /// let builder = prebindgen::batching::ffi_converter::Builder::new("example_ffi")
    ///     .prefixed_exported_type("foo::Foo")
    ///     .prefixed_exported_type("bar::Bar");
    /// ```
    #[roxygen]
    pub fn prefixed_exported_type<S: AsRef<str>>(
        mut self,
        /// The full path to the exported type (e.g., "foo::Foo")
        full_type_path: S,
    ) -> Self {
        let path: syn::Path = syn::parse_str(full_type_path.as_ref()).unwrap();
        self.prefixed_exported_types.push(path);
        self
    }

    /// Set the Rust edition to use for generated code
    ///
    /// This affects how the `#[no_mangle]` attribute is generated:
    /// - For Edition2024 with Rust >= 1.82: `#[unsafe(no_mangle)]`
    /// - For Edition2021 or older Rust versions: `#[no_mangle]`
    ///
    /// Default is automatically detected based on the current compiler version.
    ///
    /// # Example
    ///
    /// ```
    /// let builder = prebindgen::batching::ffi_converter::Builder::new("example_ffi")
    ///     .edition(prebindgen::RustEdition::Edition2024);
    /// ```
    #[roxygen]
    pub fn edition(
        mut self,
        /// The Rust edition
        edition: RustEdition,
    ) -> Self {
        self.edition = edition;
        self
    }

    /// Build the FfiConverter instance with the configured options
    ///
    /// # Example
    ///
    /// ```
    /// let converter = prebindgen::batching::ffi_converter::Builder::new("example_ffi")
    ///     .edition(prebindgen::RustEdition::Edition2024)
    ///     .build();
    /// ```
    pub fn build(self) -> FfiConverter {
        // Prefill primitive types with real primitive types mapping to themselves
        let mut primitive_types = HashMap::new();
        for primitive in [
            "bool", "char", "i8", "i16", "i32", "i64", "i128", "isize", "u8", "u16", "u32", "u64",
            "u128", "usize", "f32", "f64", "str",
        ] {
            primitive_types.insert(primitive.to_string(), primitive.to_string());
        }

        FfiConverter {
            builder: self,
            stage: GenerationStage::Collect,
            source_items: Vec::new(),
            type_replacements: HashMap::new(),
            exported_types: HashSet::new(),
            primitive_types,
            followup_items: Vec::new(),
        }
    }
}

enum GenerationStage {
    /// First stage: go through all items in the iterator and collect all type names.
    /// These types will be copied to the destination file and transmuted to the original crate types when
    /// performing the ffi calls
    Collect,
    /// Second stage: copy #[prebindgen] marked types and functions to the destination file
    /// with necessary type name adjustments and generating function stubs
    Convert,
    /// Third stage: generate assertions for correctness of type transmute operations
    Followup,
}

/// Converts prebindgen items into FFI-compatible Rust code for language-specific binding generation
///
/// The `FfiConverter` is the core component that transforms items marked with `#[prebindgen]`
/// from a source FFI crate into a Rust file suitable for:
/// 1. **Binding generation**: Passing to tools like cbindgen, csbindgen, etc.
/// 2. **Library compilation**: Including as Rust source to create C-style static or dynamic libraries
///
/// # Functionality
///
/// The converter transforms prebindgen items by:
/// - Converting regular Rust functions into `#[no_mangle] extern "C"` proxy functions
/// - Copying type definitions with necessary adjustments for FFI compatibility
/// - Handling type transmutations between source and destination crate types
/// - Generating compile-time assertions to ensure type safety
///
/// # Example
/// ```
/// # use itertools::Itertools;
/// // In build.rs of a language-specific binding crate
/// # prebindgen::Source::init_doctest_simulate();
/// let source = prebindgen::Source::new("source_ffi");
///
/// let converter = prebindgen::batching::FfiConverter::builder(source.crate_name())
///     .edition(prebindgen::RustEdition::Edition2024)
///     .strip_transparent_wrapper("std::option::Option")
///     .strip_transparent_wrapper("std::mem::MaybeUninit")
///     .build();
///
/// // Process items using itertools::batching
/// let processed_items: Vec<_> = source
///     .items_all()
///     .batching(converter.into_closure())
///     .take(0) // Take 0 for doctest
///     .collect();
/// ```
pub struct FfiConverter {
    pub(crate) builder: Builder,
    /// Current generation stage
    stage: GenerationStage,
    /// Items read from the source iterator
    source_items: Vec<(syn::Item, SourceLocation)>,
    /// Copied types which needs transmute operations - filled on `Collect` stage and used on `Convert` stage
    exported_types: HashSet<String>,
    /// Types that are primitive or aliases to primitive types - prefilled with real primitives and updated during collection
    primitive_types: HashMap<String, String>,
    /// Type replacements made - filled on `Convert` stage and used to prepare assertion items for `Followup` stage
    type_replacements: HashMap<TypeTransmutePair, SourceLocation>,
    /// Items which are output in the end
    followup_items: Vec<(syn::Item, SourceLocation)>,
}

impl FfiConverter {
    /// Create a builder for configuring an FFI converter instance
    ///
    /// # Parameters
    ///
    /// * `source_crate_name` - Name of the source crate containing `#[prebindgen]` items
    ///
    /// # Example
    ///
    /// ```
    /// let converter = prebindgen::batching::FfiConverter::builder("example_ffi")
    ///     .edition(prebindgen::RustEdition::Edition2024)
    ///     .strip_transparent_wrapper("std::mem::MaybeUninit")
    ///     .build();
    /// ```
    pub fn builder(source_crate_name: impl Into<String>) -> Builder {
        Builder::new(source_crate_name)
    }

    fn collect_item(&mut self, item: syn::Item, source_location: SourceLocation) {
        // Update exported_types for type items
        // Create a unique key that includes cfg attributes to handle multiple definitions
        let get_type_key = |name: &str, item: &syn::Item| -> String {
            let cfg_attrs: Vec<String> = match item {
                syn::Item::Struct(s) => s.attrs.iter()
                    .filter(|attr| attr.path().is_ident("cfg"))
                    .map(|attr| attr.to_token_stream().to_string())
                    .collect(),
                syn::Item::Enum(e) => e.attrs.iter()
                    .filter(|attr| attr.path().is_ident("cfg"))
                    .map(|attr| attr.to_token_stream().to_string())
                    .collect(),
                syn::Item::Union(u) => u.attrs.iter()
                    .filter(|attr| attr.path().is_ident("cfg"))
                    .map(|attr| attr.to_token_stream().to_string())
                    .collect(),
                syn::Item::Type(t) => t.attrs.iter()
                    .filter(|attr| attr.path().is_ident("cfg"))
                    .map(|attr| attr.to_token_stream().to_string())
                    .collect(),
                _ => Vec::new(),
            };
            if cfg_attrs.is_empty() {
                name.to_string()
            } else {
                format!("{}#{}", name, cfg_attrs.join("|"))
            }
        };

        match &item {
            syn::Item::Struct(s) => {
                let type_name = s.ident.to_string();
                let type_key = get_type_key(&type_name, &item);
                self.exported_types.insert(type_key);
            }
            syn::Item::Enum(e) => {
                let type_name = e.ident.to_string();
                let type_key = get_type_key(&type_name, &item);
                self.exported_types.insert(type_key);
            }
            syn::Item::Union(u) => {
                let type_name = u.ident.to_string();
                let type_key = get_type_key(&type_name, &item);
                self.exported_types.insert(type_key);
            }
            syn::Item::Type(t) => {
                let type_name = t.ident.to_string();
                let type_key = get_type_key(&type_name, &item);
                self.exported_types.insert(type_key);

                // Check if this type alias points to a primitive type
                if let syn::Type::Path(type_path) = &*t.ty {
                    if let Some(last_segment) = type_path.path.segments.last() {
                        let target_type = last_segment.ident.to_string();
                        if let Some(basic_type) = self.primitive_types.get(&target_type).cloned() {
                            self.primitive_types
                                .insert(type_name.clone(), basic_type.clone());
                            let prefixed_path: syn::Path = syn::parse_str(&format!(
                                "{}::{}",
                                self.builder.source_crate_name.replace('-', "_"),
                                type_name
                            ))
                            .unwrap();
                            let prefixed_name = quote::quote! { #prefixed_path }.to_string();
                            self.primitive_types.insert(prefixed_name, basic_type);
                        }
                    }
                }
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
                primitive_types: &self.primitive_types,
                allowed_prefixes: &self.builder.allowed_prefixes,
                prefixed_exported_types: &self.builder.prefixed_exported_types,
                transparent_wrappers: &self.builder.transparent_wrappers,
                edition: self.builder.edition,
            };

            // Process based on item type
            match item {
                // Convert function to FFI stub
                syn::Item::Fn(ref mut function) => convert_to_stub(
                    function,
                    &config,
                    &mut self.type_replacements,
                    &source_location,
                ),
                // Replace types in non-function items
                _ => replace_types_in_item(
                    &mut item,
                    &config,
                    &mut self.type_replacements,
                    &source_location,
                ),
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
                self.followup_items
                    .push((align_assertion, source_location.clone()));
            }
        }
    }

    /// Process items from an iterator in batching mode
    ///
    /// This method implements the three-stage conversion process, consuming items
    /// from the iterator and returning converted FFI-compatible items one at a time.
    /// Used internally by `into_closure()` for integration with `itertools::batching`.
    ///
    /// # Parameters
    ///
    /// * `iter` - Iterator over `(syn::Item, SourceLocation)` pairs from prebindgen data
    ///
    /// # Returns
    ///
    /// `Some((syn::Item, SourceLocation))` for each converted item, `None` when complete
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

    /// Convert to closure compatible with `itertools::batching`
    ///
    /// This is the primary method for using `FfiConverter` in processing pipelines.
    /// The returned closure must be passed to `itertools::batching()` to transform
    /// a stream of prebindgen items into FFI-compatible Rust code.
    ///
    /// **Note**: Requires the `itertools` crate for the `batching` method.
    ///
    /// # Example
    ///
    /// ```
    /// # use itertools::Itertools;
    /// # prebindgen::Source::init_doctest_simulate();
    /// let source = prebindgen::Source::new("source_ffi");
    /// let converter = prebindgen::batching::FfiConverter::builder("example_ffi").build();
    ///
    /// // Use with itertools::batching
    /// let processed_items: Vec<_> = source
    ///     .items_all()
    ///     .batching(converter.into_closure())
    ///     .take(0) // Take 0 for doctest
    ///     .collect();
    /// ```
    pub fn into_closure<I>(mut self) -> impl FnMut(&mut I) -> Option<(syn::Item, SourceLocation)>
    where
        I: Iterator<Item = (syn::Item, SourceLocation)>,
    {
        move |iter| self.call(iter)
    }
}
