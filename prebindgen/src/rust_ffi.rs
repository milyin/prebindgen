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
            type_replacements: HashSet::new(),
            exported_types: HashSet::new(),
            finished: false,
            pending_items: Vec::new(),
        }
    }
}

impl Default for Builder {
    fn default() -> Self {
        Self::new()
    }
}

/// RustFfi structure that mirrors Prebindgen functionality without file operations
pub struct RustFfi {
    pub(crate) builder: Builder,
    type_replacements: HashSet<crate::codegen::TypeTransmutePair>,
    exported_types: HashSet<String>,
    finished: bool,
    pending_items: Vec<syn::Item>,
}

impl RustFfi {
    /// Call method for use with batching - wrap in closure: |iter| rust_ffi.call(iter)
    pub fn call<I>(&mut self, iter: &mut I) -> Option<(syn::Item, Option<crate::record::SourceLocation>)>
    where
        I: Iterator<Item = (syn::Item, Option<crate::record::SourceLocation>)>,
    {
        // Return pending items first
        if let Some(pending_item) = self.pending_items.pop() {
            return Some((pending_item, None));
        }

        if !self.finished {
            loop {
                let Some((mut item, source_location)) = iter.next() else {
                    self.finished = true;
                    break;
                };

                // Process features
                if !crate::codegen::process_features::process_item_features(
                    &mut item,
                    &self.builder.disabled_features,
                    &self.builder.enabled_features,
                    &self.builder.feature_mappings,
                    &source_location.unwrap_or_default(),
                ) {
                    continue; // Skip filtered items
                }

                // Update exported_types for type items
                match &item {
                    syn::Item::Struct(s) => { self.exported_types.insert(s.ident.to_string()); }
                    syn::Item::Enum(e) => { self.exported_types.insert(e.ident.to_string()); }
                    syn::Item::Union(u) => { self.exported_types.insert(u.ident.to_string()); }
                    syn::Item::Type(t) => { self.exported_types.insert(t.ident.to_string()); }
                    _ => {}
                }

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

                let mut new_type_replacements = HashSet::new();

                // Process based on item type
                match &mut item {
                    syn::Item::Fn(function) => {
                        // Convert function to FFI stub
                        if let Err(e) = crate::codegen::convert_to_stub(function, &config, &mut new_type_replacements) {
                            let location = source_location.as_ref().map(|l| format!(" at {}", l)).unwrap_or_default();
                            panic!("Failed to convert function {}{}: {}", function.sig.ident, location, e);
                        }
                    }
                    _ => {
                        // Replace types in non-function items
                        let _ = crate::codegen::replace_types_in_item(&mut item, &config, &mut new_type_replacements);
                    }
                }

                // Check for new type replacements and add assertions
                for new_replacement in new_type_replacements {
                    if !self.type_replacements.contains(&new_replacement) {
                        self.type_replacements.insert(new_replacement.clone());
                        let assertions_file = crate::codegen::generate_type_assertions(&[new_replacement].iter().cloned().collect());
                        self.pending_items.extend(assertions_file.items);
                    }
                }

                return Some((item, source_location));
            }
        }

        // Return remaining pending items
        if let Some(pending_item) = self.pending_items.pop() {
            return Some((pending_item, None));
        }

        None
    }

    /// Convert to closure compatible with batching
    pub fn into_closure<I>(mut self) -> impl FnMut(&mut I) -> Option<(syn::Item, Option<crate::record::SourceLocation>)>
    where
        I: Iterator<Item = (syn::Item, Option<crate::record::SourceLocation>)>,
    {
        move |iter| self.call(iter)
    }
}