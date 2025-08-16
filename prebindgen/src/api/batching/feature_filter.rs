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
