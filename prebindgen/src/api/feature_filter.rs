use std::collections::{HashMap, HashSet};

use roxygen::roxygen;

use crate::{api::record::SourceLocation, codegen::process_features::process_item_features};

pub struct Builder {
    pub(crate) disabled_features: HashSet<String>,
    pub(crate) enabled_features: HashSet<String>,
    pub(crate) feature_mappings: HashMap<String, String>,
}

impl Builder {
    pub fn new() -> Self {
        Self {
            disabled_features: HashSet::new(),
            enabled_features: HashSet::new(),
            feature_mappings: HashMap::new(),
        }
    }

    /// Disable a feature in the generated code
    ///
    /// When processing code with `#[cfg(feature="...")]` attributes, code blocks
    /// guarded by disabled features will be completely skipped in the output.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let builder = prebindgen::Builder::new(path)
    ///     .disable_feature("experimental")
    ///     .disable_feature("deprecated");
    /// ```
    #[roxygen]
    pub fn disable_feature<S: Into<String>>(
        mut self,
        /// The name of the feature to disable
        feature: S,
    ) -> Self {
        self.disabled_features.insert(feature.into());
        self
    }

    /// Enable a feature in the generated code
    ///
    /// When processing code with `#[cfg(feature="...")]` attributes, code blocks
    /// guarded by enabled features will be included in the output with the
    /// `#[cfg(...)]` attribute removed.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let builder = prebindgen::Builder::new(path)
    ///     .enable_feature("experimental");
    /// ```
    #[roxygen]
    pub fn enable_feature<S: Into<String>>(
        mut self,
        /// The name of the feature to enable
        feature: S,
    ) -> Self {
        self.enabled_features.insert(feature.into());
        self
    }
    /// Map a feature name to a different name in the generated code
    ///
    /// When processing code with `#[cfg(feature="...")]` attributes, features
    /// that match the mapping will have their names replaced with the target
    /// feature name in the output.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let builder = prebindgen::Builder::new(path)
    ///     .match_feature("unstable", "unstable")
    ///     .match_feature("internal", "unstable");
    /// ```
    #[roxygen]
    pub fn match_feature<S1: Into<String>, S2: Into<String>>(
        mut self,
        /// The original feature name to match
        from: S1,
        /// The new feature name to use in the output
        to: S2,
    ) -> Self {
        self.feature_mappings.insert(from.into(), to.into());
        self
    }

    /// Build the Features instance
    pub fn build(self) -> FeatureFilter {
        FeatureFilter { builder: self }
    }
}

impl Default for Builder {
    fn default() -> Self {
        Self::new()
    }
}

pub struct FeatureFilter {
    builder: Builder,
}

impl FeatureFilter {
    // Call method to use with `filter_map` function
    pub fn call(&self, item: (syn::Item, SourceLocation)) -> Option<(syn::Item, SourceLocation)> {
        // Check if the item is affected by any feature flags
        let (mut item, source_location) = item;
        if process_item_features(
            &mut item,
            &self.builder.disabled_features,
            &self.builder.enabled_features,
            &self.builder.feature_mappings,
            &source_location,
        ) {
            Some((item, source_location))
        } else {
            None
        }
    }

    /// Convert to closure
    pub fn into_closure(
        self,
    ) -> impl FnMut((syn::Item, SourceLocation)) -> Option<(syn::Item, SourceLocation)> {
        move |item| self.call(item)
    }
}
