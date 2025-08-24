//! Utility functions related to code generation and transformation

use std::collections::{HashMap, HashSet};

/// Structure with rules to apply to CfgExpression
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CfgExprRules {
    /// Explicitly enabled features (replaced to true in the cfg expression)
    pub enabled_features: HashSet<String>,
    /// Explicitly disabled features (replaced to false in the cfg expression)
    pub disabled_features: HashSet<String>,
    /// Mapping from old feature names to new feature names
    pub feature_mappings: HashMap<String, String>,
    /// If true, unknown features are treated as disabled (skipped) instead of causing an error
    pub disable_unknown_features: bool,
    /// If Some, replace matching target architecture to true and any other to false
    pub enabled_target_arch: Option<String>,
    /// If Some, replace matching target vendor to true and any other to false
    pub enabled_target_vendor: Option<String>,
    /// If Some, replace matching target OS to true and any other to false
    pub enabled_target_os: Option<String>,
    /// If Some, replace matching target environment to true and any other to false
    pub enabled_target_env: Option<String>,
}

impl CfgExprRules {
    // Return true if any rule is exists
    pub fn is_active(&self) -> bool {
        !self.enabled_features.is_empty()
            || !self.disabled_features.is_empty()
            || !self.feature_mappings.is_empty()
            || self.disable_unknown_features
            || self.enabled_target_arch.is_some()
            || self.enabled_target_vendor.is_some()
            || self.enabled_target_os.is_some()
            || self.enabled_target_env.is_some()
    }
}

pub(crate) mod cfg_expr;
pub(crate) mod process_features;
pub(crate) mod replace_types;
#[cfg(test)]
pub(crate) mod tests;
