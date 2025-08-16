use std::collections::{HashMap, HashSet};

use roxygen::roxygen;

use crate::{api::record::SourceLocation, codegen::process_features::process_item_features};

/// Builder for configuring FeatureFilter instances
///
/// Configures how feature flags in `#[cfg(feature="...")]` attributes
/// are processed when generating FFI bindings.
///
/// # Example
///
/// ```
/// let builder = prebindgen::batching::feature_filter::Builder::new()
///     .disable_feature("unstable")
///     .enable_feature("std")
///     .match_feature("internal", "public")
///     .build();
/// ```
pub struct Builder {
    pub(crate) disabled_features: HashSet<String>,
    pub(crate) enabled_features: HashSet<String>,
    pub(crate) feature_mappings: HashMap<String, String>,
    // When true, unknown features are treated as disabled (skipped) instead of causing an error
    pub(crate) disable_unknown_features: bool,
    // Optional name of a features constant (accepted by predefined_features for API parity)
    pub(crate) features_constant: Option<String>,
}

impl Builder {
    /// Create a new Builder for configuring FeatureFilter
    ///
    /// # Example
    ///
    /// ```
    /// let builder = prebindgen::batching::feature_filter::Builder::new();
    /// ```
    pub fn new() -> Self {
        Self {
            disabled_features: HashSet::new(),
            enabled_features: HashSet::new(),
            feature_mappings: HashMap::new(),
            disable_unknown_features: false,
            features_constant: None,
        }
    }

    /// Disable a feature in the generated code
    ///
    /// When processing code with `#[cfg(feature="...")]` attributes, code blocks
    /// guarded by disabled features will be completely skipped in the output.
    ///
    /// # Example
    ///
    /// ```
    /// let builder = prebindgen::batching::feature_filter::Builder::new()
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
    /// ```
    /// let builder = prebindgen::batching::feature_filter::Builder::new()
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
    /// ```
    /// let builder = prebindgen::batching::feature_filter::Builder::new()
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

    /// Assume a predefined feature set: the provided list is enabled, all others are disabled.
    /// Also enables skipping of unknown features (treat them as disabled) to avoid reporting.
    #[roxygen]
    pub fn predefined_features<S, I, T>(
        mut self,
        /// Name of the feature constant defined in the source crate
        feature_constant: S,
        /// Iterator or collection of enabled feature names
        features: I,
    ) -> Self
    where
        S: Into<String>,
        I: IntoIterator<Item = T>,
        T: Into<String>,
    {
        // Reset previous configuration to avoid conflicts
        self.disabled_features.clear();
        self.enabled_features.clear();
        self.feature_mappings.clear();

        // Record constant name (not used at runtime here, kept for API completeness)
        self.features_constant = Some(feature_constant.into());

        // Enable exactly the provided features
        self.enabled_features
            .extend(features.into_iter().map(|f| f.into()));

        // Treat unknown features as disabled to "skip" them silently
        self.disable_unknown_features = true;
        self
    }

    /// Set whether unknown features should be treated as disabled (skipped) instead of reported.
    #[roxygen]
    pub fn disable_unknown_features(
        mut self,
        /// If true, unknown features are skipped instead of reported
        value: bool,
    ) -> Self {
        self.disable_unknown_features = value;
        self
    }

    /// Build the FeatureFilter instance with the configured options
    ///
    /// # Example
    ///
    /// ```
    /// let filter = prebindgen::batching::feature_filter::Builder::new()
    ///     .disable_feature("internal")
    ///     .build();
    /// ```
    pub fn build(self) -> FeatureFilter {
        FeatureFilter { builder: self }
    }
}

impl Default for Builder {
    fn default() -> Self {
        Self::new()
    }
}

/// Filters prebindgen items based on Rust feature flags
///
/// The `FeatureFilter` processes items with `#[cfg(feature="...")]` attributes,
/// allowing selective inclusion, exclusion, or renaming of feature-gated code
/// in the generated FFI bindings.
///
/// # Functionality
///
/// - **Disable features**: Skip items guarded by disabled features
/// - **Enable features**: Include items and remove their `#[cfg(...)]` attributes
/// - **Map features**: Rename feature flags in the output
///
/// # Example
/// ```
/// # prebindgen::Source::init_doctest_simulate();
/// let source = prebindgen::Source::new("source_ffi");
///
/// let feature_filter = prebindgen::batching::FeatureFilter::builder()
///     .disable_feature("unstable")
///     .disable_feature("internal")
///     .enable_feature("std")
///     .match_feature("experimental", "beta")
///     .build();
///
/// // Apply filter to items using itertools::batching
/// # use itertools::Itertools;
/// let filtered_items: Vec<_> = source
///     .items_all()
///     .batching(feature_filter.into_closure())
///     .take(0) // Take 0 for doctest
///     .collect();
/// ```
pub struct FeatureFilter {
    builder: Builder,
}

impl FeatureFilter {
    /// Create a builder for configuring a feature filter instance
    ///
    /// # Example
    ///
    /// ```
    /// let filter = prebindgen::batching::FeatureFilter::builder()
    ///     .disable_feature("unstable")
    ///     .enable_feature("std")
    ///     .build();
    /// ```
    pub fn builder() -> Builder {
        Builder::new()
    }
    /// Process items from an iterator in batching mode
    ///
    /// Consumes items until it finds one that should be kept according to feature flags.
    /// Returns that item, possibly with adjusted attributes. Otherwise `None` at end.
    pub fn call<I>(&mut self, iter: &mut I) -> Option<(syn::Item, SourceLocation)>
    where
        I: Iterator<Item = (syn::Item, SourceLocation)>,
    {
        for (mut item, source_location) in iter {
            if process_item_features(
                &mut item,
                &self.builder.disabled_features,
                &self.builder.enabled_features,
                &self.builder.feature_mappings,
                self.builder.disable_unknown_features,
                &source_location,
            ) {
                return Some((item, source_location));
            }
        }
        None
    }

    /// Convert to closure compatible with `itertools::batching`
    ///
    /// The returned closure should be passed to `.batching(...)` in the iterator chain.
    pub fn into_closure<I>(mut self) -> impl FnMut(&mut I) -> Option<(syn::Item, SourceLocation)>
    where
        I: Iterator<Item = (syn::Item, SourceLocation)>,
    {
        move |iter| self.call(iter)
    }
}
