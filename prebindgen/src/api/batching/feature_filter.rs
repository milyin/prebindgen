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
/// let builder = prebindgen::filter_map::feature_filter::Builder::new()
///     .disable_feature("unstable")
///     .enable_feature("std")
///     .match_feature("internal", "public")
///     .build();
/// ```
pub struct Builder {
    pub(crate) disabled_features: HashSet<String>,
    pub(crate) enabled_features: HashSet<String>,
    pub(crate) feature_mappings: HashMap<String, String>,
}

impl Builder {
    /// Create a new Builder for configuring FeatureFilter
    ///
    /// # Example
    ///
    /// ```
    /// let builder = prebindgen::filter_map::feature_filter::Builder::new();
    /// ```
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
    /// ```
    /// let builder = prebindgen::filter_map::feature_filter::Builder::new()
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
    /// let builder = prebindgen::filter_map::feature_filter::Builder::new()
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
    /// let builder = prebindgen::filter_map::feature_filter::Builder::new()
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

    /// Build the FeatureFilter instance with the configured options
    ///
    /// # Example
    ///
    /// ```
    /// let filter = prebindgen::filter_map::feature_filter::Builder::new()
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
/// let feature_filter = prebindgen::filter_map::FeatureFilter::builder()
///     .disable_feature("unstable")
///     .disable_feature("internal")
///     .enable_feature("std")
///     .match_feature("experimental", "beta")
///     .build();
///
/// // Apply filter to items
/// let filtered_items: Vec<_> = source
///     .items_all()
///     .filter_map(feature_filter.into_closure())
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
    /// let filter = prebindgen::filter_map::FeatureFilter::builder()
    ///     .disable_feature("unstable")
    ///     .enable_feature("std")
    ///     .build();
    /// ```
    pub fn builder() -> Builder {
        Builder::new()
    }
    /// Process a single item through the feature filter
    ///
    /// Returns `Some(item)` if the item should be included, `None` if it should be filtered out.
    /// Used internally by `into_closure()` for integration with `filter_map`.
    ///
    /// # Parameters
    ///
    /// * `item` - A `(syn::Item, SourceLocation)` pair to process
    ///
    /// # Returns
    ///
    /// `Some((syn::Item, SourceLocation))` if included, `None` if filtered out
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

    /// Convert to closure compatible with `filter_map`
    ///
    /// This is the primary method for using `FeatureFilter` in processing pipelines.
    /// The returned closure can be passed to `filter_map()` to selectively include
    /// items based on their feature flags.
    ///
    /// # Example
    ///
    /// ```
    /// # prebindgen::Source::init_doctest_simulate();
    /// let source = prebindgen::Source::new("source_ffi");
    /// let filter = prebindgen::filter_map::FeatureFilter::builder()
    ///     .disable_feature("internal")
    ///     .build();
    ///
    /// // Use with filter_map
    /// let filtered_items: Vec<_> = source
    ///     .items_all()
    ///     .filter_map(filter.into_closure())
    ///     .take(0) // Take 0 for doctest
    ///     .collect();
    /// ```
    pub fn into_closure(
        self,
    ) -> impl FnMut((syn::Item, SourceLocation)) -> Option<(syn::Item, SourceLocation)> {
        move |item| self.call(item)
    }
}
