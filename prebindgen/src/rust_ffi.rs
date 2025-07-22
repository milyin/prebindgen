//! The core functionality of Prebinding library: generation of FFI rust source from items marked with
//! `#[prebindgen]` attribute.
//!
//! The concept is following:
//! There are two crates:
//! -- library_ffi crate which exports set of repr-C structures and set of functions with `#[prebindgen]` attribute.
//! The important moment is that these functions are not `extern "C"`
//! -- library_binding crate (e.g. for library_c or library_cs)
//! This crate includes single (or multiple if necessary) source file which contains copies of the structures above
//! and `#[no_mangle] extern "C"` functions which call the original functions from library_ffi crate.
//!
//! This allows to
//! - have a single ffi implementation and reuse it for different language bindings without squashing all to single crate
//! - adapt the ffi source to specificity of different binding generators

use std::collections::{HashMap, HashSet};

/// Builder for configuring RustFfi without file operations
pub struct Builder {
    pub(crate) allowed_prefixes: Vec<syn::Path>,
    pub(crate) disabled_features: HashSet<String>,
    pub(crate) enabled_features: HashSet<String>,
    pub(crate) feature_mappings: HashMap<String, String>,
    pub(crate) transparent_wrappers: Vec<syn::Path>,
    pub(crate) edition: String,
}

impl Builder {
    /// Create a new RustFfi builder
    pub fn new() -> Self {
        Self {
            allowed_prefixes: Vec::new(),
            disabled_features: HashSet::new(),
            enabled_features: HashSet::new(),
            feature_mappings: HashMap::new(),
            transparent_wrappers: Vec::new(),
            edition: "2021".to_string(),
        }
    }

    /// Add an allowed type prefix for FFI compatibility
    pub fn allowed_prefix<S: AsRef<str>>(mut self, prefix: S) -> Self {
        let path: syn::Path = syn::parse_str(prefix.as_ref()).unwrap();
        self.allowed_prefixes.push(path);
        self
    }

    /// Disable a feature (items with this feature will be excluded)
    pub fn disable_feature<S: Into<String>>(mut self, feature: S) -> Self {
        self.disabled_features.insert(feature.into());
        self
    }

    /// Enable a feature (cfg attributes for this feature will be removed)
    pub fn enable_feature<S: Into<String>>(mut self, feature: S) -> Self {
        self.enabled_features.insert(feature.into());
        self
    }

    /// Map one feature name to another
    pub fn match_feature<S1: Into<String>, S2: Into<String>>(mut self, from: S1, to: S2) -> Self {
        self.feature_mappings.insert(from.into(), to.into());
        self
    }

    /// Add a transparent wrapper type to be stripped
    pub fn strip_transparent_wrapper<S: AsRef<str>>(mut self, wrapper: S) -> Self {
        let path: syn::Path = syn::parse_str(wrapper.as_ref()).unwrap();
        self.transparent_wrappers.push(path);
        self
    }

    /// Set the Rust edition
    pub fn edition<S: Into<String>>(mut self, edition: S) -> Self {
        self.edition = edition.into();
        self
    }

    /// Build the RustFfi instance
    pub fn build(self) -> RustFfi {
        RustFfi {
            builder: self,
            stage: RustFfiGenerationStage::Collect,
            source_items: Vec::new(),
            type_replacements: HashSet::new(),
            exported_types: HashSet::new(),
            followup_items: Vec::new(),
        }
    }
}

impl Default for Builder {
    fn default() -> Self {
        Self::new()
    }
}

enum RustFfiGenerationStage {
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

/// RustFfi structure that mirrors Prebindgen functionality without file operations
pub struct RustFfi {
    pub(crate) builder: Builder,
    /// Current generation stage
    stage: RustFfiGenerationStage,
    /// Items read from the source iterator
    source_items: Vec<(syn::Item, Option<crate::record::SourceLocation>)>,
    /// Copied types which needs transmute operations - filled on `Collect` stage and used on `Convert` stage
    exported_types: HashSet<String>,
    /// Type replacements made - filled on `Convert` stage and used to prepare assertion items for `Followup` stage
    type_replacements: HashSet<crate::codegen::TypeTransmutePair>,
    /// Items which are output in the end
    followup_items: Vec<(syn::Item, Option<crate::record::SourceLocation>)>,
}

impl RustFfi {
    fn collect_item<I>(
        &mut self,
        item: syn::Item,
        source_location: Option<crate::record::SourceLocation>,
    ) {
        // Process features
            if !crate::codegen::process_features::process_item_features(
                &mut item.clone(),
                &self.builder.disabled_features,
                &self.builder.enabled_features,
                &self.builder.feature_mappings,
                &source_location.unwrap_or_default(),
            ) {
                return;
            }

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

    }

    fn convert(&mut self) -> Option<(syn::Item, Option<crate::record::SourceLocation>)> {
        if let Some((item, source_location)) = self.source_items.pop() {
            // Create parse config
            let config = crate::record::ParseConfig {
                crate_name: "unknown",
                exported_types: &self.exported_types,
                disabled_features: &self.builder.disabled_features,
                enabled_features: &self.builder.enabled_features,
                feature_mappings: &self.builder.feature_mappings,
                allowed_prefixes: &self.builder.allowed_prefixes,
                transparent_wrappers: &self.builder.transparent_wrappers,
                edition: &self.builder.edition,
            };

            // Process based on item type
            match &mut item {
                syn::Item::Fn(function) => {
                    // Convert function to FFI stub
                    if let Err(e) = crate::codegen::convert_to_stub(
                        function,
                        &config,
                        &mut self.type_replacements,
                    ) {
                        panic!(
                            "Failed to convert function {}{}: {}",
                            function.sig.ident, source_location, e
                        );
                    }
                }
                _ => {
                    // Replace types in non-function items
                    let _ = crate::codegen::replace_types_in_item(
                        &mut item,
                        &config,
                        &mut self.type_replacements,
                    );
                }
            }

            return Some((item, source_location));
        }

        None
    }

    fn generate_assertions(
        &mut self,
    ) -> Option<(syn::Item, Option<crate::record::SourceLocation>)> {
        // Generate assertions for type transmute correctness
        for replacement in &self.type_replacements {
            if let Some((size_assertion, align_assertion)) =
                crate::codegen::generate_type_transmute_pair_assertions(replacement)
            {
                self.followup_items.push((size_assertion, None));
                self.followup_items.push((align_assertion, None));
            }
        }
    }

    /// Call method for use with batching - wrap in closure: |iter| rust_ffi.call(iter)
    pub fn call<I>(
        &mut self,
        iter: &mut I,
    ) -> Option<(syn::Item, Option<crate::record::SourceLocation>)>
    where
        I: Iterator<Item = (syn::Item, Option<crate::record::SourceLocation>)>,
    {
        loop {
            match self.stage {
                RustFfiGenerationStage::Collect => {
                    // Collect stage: collect type names and prepare for conversion
                    // Consumes the iterator until the end and swtitches to Convert stage
                    if let Some((item, source_location)) = iter.next() {
                        self.collect_item(item, source_location);
                    } else {
                        self.stage = RustFfiGenerationStage::Convert;
                    }
                }
                RustFfiGenerationStage::Convert => {
                    // Convert stage: process items harvested on the Collect stage one by one.
                    // If all items are processed, generate assertions and switch to Followup stage
                    if let Some((item, source_location)) = self.convert() {
                        return Some((item, source_location));
                    } else {
                        self.generate_assertions();
                        self.stage = RustFfiGenerationStage::Followup;
                    }
                }
                RustFfiGenerationStage::Followup => {
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
    ) -> impl FnMut(&mut I) -> Option<(syn::Item, Option<crate::record::SourceLocation>)>
    where
        I: Iterator<Item = (syn::Item, Option<crate::record::SourceLocation>)>,
    {
        move |iter| self.call(iter)
    }
}
