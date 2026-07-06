use roxygen::roxygen;

use crate::{
    api::record::SourceLocation,
    codegen::{process_features::process_item_features, CfgExprRules},
};

/// Builder for configuring CfgFilter instances
///
/// Configures how flags in `#[cfg(...)]` attributes
/// are processed when generating FFI bindings.
/// Supports features and target architecture filtering.
///
/// Used internally by `Source`, which automatically reads the features enabled
/// in the source crate and removes any code guarded by disabled features.
pub struct Builder {
    /// Source crate features constant name and features list in format "crate/f1 crate/f2"
    pub(crate) features_assert: Option<(String, String)>,
    /// Rules for cfg expression processing
    rules: CfgExprRules,
}

impl Builder {
    /// Create a new Builder for configuring CfgFilter
    pub fn new() -> Self {
        Self {
            features_assert: None,
            rules: CfgExprRules::default(),
        }
    }

    /// Enable a specific target architecture. All other architectures are treated as disabled.
    ///
    /// Only one architecture can be enabled. Calling this again overwrites the previous choice.
    #[roxygen]
    pub fn enable_target_arch<S: Into<String>>(
        mut self,
        /// The target architecture value to enable (e.g., "x86_64", "aarch64")
        arch: S,
    ) -> Self {
        self.rules.enabled_target_arch = Some(arch.into());
        self
    }

    /// Enable a specific target vendor. All other vendors are treated as disabled.
    ///
    /// Only one vendor can be enabled. Calling this again overwrites the previous choice.
    #[roxygen]
    pub fn enable_target_vendor<S: Into<String>>(
        mut self,
        /// The target vendor value to enable (e.g., "apple", "pc")
        vendor: S,
    ) -> Self {
        self.rules.enabled_target_vendor = Some(vendor.into());
        self
    }

    /// Enable a specific target operating system. All other OS values are treated as disabled.
    ///
    /// Only one OS can be enabled. Calling this again overwrites the previous choice.
    #[roxygen]
    pub fn enable_target_os<S: Into<String>>(
        mut self,
        /// The target operating system to enable (e.g., "macos", "linux", "windows")
        os: S,
    ) -> Self {
        self.rules.enabled_target_os = Some(os.into());
        self
    }

    /// Enable a specific target environment. All other environments are treated as disabled.
    ///
    /// Only one environment can be enabled. Calling this again overwrites the previous choice.
    #[roxygen]
    pub fn enable_target_env<S: Into<String>>(
        mut self,
        /// The target environment to enable (e.g., "gnu", "musl", "msvc")
        env: S,
    ) -> Self {
        self.rules.enabled_target_env = Some(env.into());
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
    ) -> Self {
        // Reset previous configuration to avoid conflicts
        self.rules.disabled_features.clear();
        self.rules.enabled_features.clear();
        self.rules.feature_mappings.clear();

        let features_constant = features_constant.into();
        let features_list = features_list.into();

        // Enable exactly the provided features
        // remove from each feature part "crate_name/", no
        // matter which crate is it, just strip the prefix
        self.rules.enabled_features.extend(
            features_list
                .split_whitespace()
                .map(|f| f.split('/').next_back().unwrap_or(f).to_string()),
        );

        // Record constant name (not used at runtime here, kept for API completeness)
        self.features_assert = Some((features_constant, features_list));

        // Treat unknown features as disabled to "skip" them silently
        self.rules.disable_unknown_features = true;
        self
    }

    /// Build the CfgFilter instance with the configured options
    pub fn build(self) -> CfgFilter {
        // Determine if this filter is active (i.e., not pass-through)
        let active = self.features_assert.is_some() || self.rules.is_active();

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
                    konst::assertc_eq!(
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

        CfgFilter {
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
/// The `CfgFilter` processes items with `#[cfg(feature="...")]` attributes,
/// allowing selective inclusion, exclusion, or renaming of feature-gated code
/// in the generated FFI bindings. `Source` builds one internally from the
/// source crate's recorded feature set and applies it lazily via
/// `itertools::batching`.
pub struct CfgFilter {
    builder: Builder,
    prelude_item: Option<(syn::Item, SourceLocation)>,
    prelude_emitted: bool,
    active: bool,
}

impl CfgFilter {
    /// Create a builder for configuring a cfg filter instance
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
            if process_item_features(&mut item, &self.builder.rules, &source_location) {
                return Some((item, source_location));
            }
        }
        None
    }
}
