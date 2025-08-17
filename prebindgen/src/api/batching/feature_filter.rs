use std::collections::{HashMap, HashSet};

use roxygen::roxygen;

use crate::{api::record::SourceLocation, codegen::process_features::process_item_features};

/// Builder for configuring FeatureFilter instances
///
/// Configures how feature flags in `#[cfg(feature="...")]` attributes
/// are processed when generating FFI bindings.
///
/// This filter is usually not necessary: the `Source` by default automatically reads
/// features enabled in the crate and removes any code guarded by disabled features.
///
/// But if necessary this option can be disabled (see and FeatureFilter can be applied
/// explicitly.
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
    // Source crate features constant name and features list in format "crate/f1 crate/f2"
    pub(crate) features_assert: Option<(String, String)>,
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
            features_assert: None,
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

    /// Automatically filter features according to provided list
    /// In the beginning put assert that list matches the actual features list of imported source crate
    /// The feature list comes directly from the prebindgen output directory, so it should match the
    /// actual features of the source crate. Unfortunately this is not always guaranteed:
    /// the crate may be built with different set of features as a destination crate dependency and
    /// as it's build.rs dependency. The assert allows to detect such mismatches early.
    ///
    /// To fix the issue 
    /// - make sure that build dependency of ffi crate is consistent with the main dependency
    /// - avoid hidden features which may be silently turned on by build --all-features
    #[roxygen]
    pub fn predefined_features<S1: Into<String>, S2: Into<String>>(
        mut self,
        /// Name of the feature` constant defined in the source crate
        features_constant: S1,
        /// List of source crate features in format "crate/f1 crate/f2"
        features_list: S2,
    ) -> Self
    {
        // Reset previous configuration to avoid conflicts
        self.disabled_features.clear();
        self.enabled_features.clear();
        self.feature_mappings.clear();

        let features_constant = features_constant.into();
        let features_list = features_list.into();

        // Enable exactly the provided features
        // remove from each feature part "crate_name/", no
        // matter which crate is it, just strip the prefix
        self.enabled_features.extend(
            features_list
                .split_whitespace()
                .map(|f| f.split('/').last().unwrap_or(f).to_string()),
        );

        // Record constant name (not used at runtime here, kept for API completeness)
        self.features_assert = Some((
            features_constant,
            features_list
        ));

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
        // Determine if this filter is active (i.e., not pass-through)
        let active = self.features_assert.is_some()
            || self.disable_unknown_features
            || !self.disabled_features.is_empty()
            || !self.enabled_features.is_empty()
            || !self.feature_mappings.is_empty();

        // Optionally create a prelude assertion comparing FEATURES const path with expected features string
        let mut prelude_item: Option<(syn::Item, SourceLocation)> = None;
        if let Some((const_name, features_list)) = self.features_assert.clone() {

            let features_lit = syn::LitStr::new(&features_list, proc_macro2::Span::call_site());

            // Parse the provided constant name into a path/expression so it is not quoted as a string.
            // This allows using values like `my_crate::FEATURES` or `FEATURES`.
            let const_path: syn::Expr =
                syn::parse_str(&const_name).expect("invalid features constant path");

            // Prefer a fully-qualified path if provided in feature_constant
            let item: syn::Item = syn::parse_quote! {
                const _: () = {
                    prebindgen::konst::assertc_eq!(
                        #const_path,
                        #features_lit,
                        "prebindgen: features mismatch between source crate and prebindgen generated file.\n\
                        This usually happens if source crate is compiled with different feature set\n\
                        for build dependencies and for library usage. You may need to explicitly set\n\
                        the necessary features."
                    );
                };
            };
            prelude_item = Some((item, SourceLocation::default()));
        }

        FeatureFilter {
            builder: self,
            prelude_item,
            prelude_emitted: false,
            active,
        }
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
    prelude_item: Option<(syn::Item, SourceLocation)>,
    prelude_emitted: bool,
    active: bool,
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
        // Emit prelude item if configured
        if !self.prelude_emitted {
            if let Some(prelude) = self.prelude_item.take() {
                self.prelude_emitted = true;
                return Some(prelude);
            }
            self.prelude_emitted = true;
        }

        if !self.active {
            return iter.next();
        }
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
