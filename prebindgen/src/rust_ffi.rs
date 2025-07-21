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
            processed_items: Vec::new(),
            finished: false,
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
    processed_items: Vec<syn::Item>,
    finished: bool,
}

impl RustFfi {
    /// Call method for use with batching - wrap in closure: |iter| rust_ffi.call(iter)
    pub fn call<I>(&mut self, iter: &mut I) -> Option<syn::Item>
    where
        I: Iterator<Item = syn::Item>,
    {
        if self.finished {
            return None;
        }

        if let Some(mut item) = iter.next() {
            // Process features
            let file = syn::File {
                shebang: None,
                attrs: vec![],
                items: vec![item.clone()],
            };
            let processed_file = crate::codegen::process_features(
                file,
                &self.builder.disabled_features,
                &self.builder.enabled_features,
                &self.builder.feature_mappings,
                &crate::record::SourceLocation::default(),
            );
            
            if let Some(processed_item) = processed_file.items.into_iter().next() {
                item = processed_item;
            } else {
                return self.call(iter); // Skip empty items
            }

            // Create parse config
            let exported_types = HashSet::new();
            let config = crate::record::ParseConfig {
                crate_name: "unknown",
                exported_types: &exported_types,
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
                    if let Err(_) = crate::codegen::convert_to_stub(function, &config, &mut self.type_replacements) {
                        return self.call(iter); // Skip on error
                    }
                }
                _ => {
                    // Replace types in non-function items
                    let _ = crate::codegen::replace_types_in_item(&mut item, &config, &mut self.type_replacements);
                }
            }

            self.processed_items.push(item.clone());
            Some(item)
        } else {
            // Iterator ended, generate type assertions
            self.finished = true;
            if !self.type_replacements.is_empty() {
                let assertions_file = crate::codegen::generate_type_assertions(&self.type_replacements);
                if let Some(assertion_item) = assertions_file.items.into_iter().next() {
                    return Some(assertion_item);
                }
            }
            None
        }
    }

    /// Convert to closure compatible with batching
    pub fn into_closure<I>(mut self) -> impl FnMut(&mut I) -> Option<syn::Item>
    where
        I: Iterator<Item = syn::Item>,
    {
        move |iter| self.call(iter)
    }
}