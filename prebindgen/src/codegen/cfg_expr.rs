//! Feature expression evaluation for `#[cfg(...)]` attributes.
//!
//! This module provides functionality to parse and evaluate complex cfg expressions
//! that include logical operators like `any`, `all`, and `not`, as well as simple
//! feature checks.

use crate::{codegen::CfgExprRules, SourceLocation};

/// Represents a cfg expression that can be evaluated against a set of enabled/disabled features
#[derive(Debug, Clone, PartialEq)]
pub enum CfgExpr {
    // Simple predicates first
    /// A simple feature check: `feature = "name"`
    Feature(String),
    /// Target architecture check: `target_arch = "arch"`
    TargetArch(String),
    /// Target vendor check: `target_vendor = "vendor"`
    TargetVendor(String),
    /// Target OS check: `target_os = "os"`
    TargetOs(String),
    /// Target environment check: `target_env = "env"`
    TargetEnv(String),

    // Logical operators next
    /// Logical NOT: `not(expr)`
    Not(Box<CfgExpr>),
    /// Logical AND: `all(expr1, expr2, ...)`
    All(Vec<CfgExpr>),
    /// Logical OR: `any(expr1, expr2, ...)`
    Any(Vec<CfgExpr>),

    // Fallbacks
    /// Any other cfg expression we don't specifically handle
    Other(String),
    /// Explicit false value (for feature processing)
    False,
}

impl CfgExpr {
    /// Parse a cfg expression from syn tokens
    pub fn parse_from_tokens(tokens: &proc_macro2::TokenStream) -> Result<Self, String> {
        let token_string = tokens.to_string();
        Self::parse_from_string(&token_string)
    }

    /// Parse a cfg expression from a string representation
    pub fn parse_from_string(input: &str) -> Result<Self, String> {
        let input = input.trim();

        // Handle logical expressions first (before simple features)
        if let Some(inner) = strip_function_call(input, "not") {
            let inner_expr = Self::parse_from_string(&inner)?;
            return Ok(CfgExpr::Not(Box::new(inner_expr)));
        }

        if let Some(inner) = strip_function_call(input, "all") {
            let exprs = parse_comma_separated(&inner)?;
            return Ok(CfgExpr::All(exprs));
        }

        if let Some(inner) = strip_function_call(input, "any") {
            let exprs = parse_comma_separated(&inner)?;
            return Ok(CfgExpr::Any(exprs));
        }

        // Handle simple feature expressions
        if let Some(feature_name) = extract_simple_feature(input) {
            return Ok(CfgExpr::Feature(feature_name));
        }

        // Handle target_arch expressions
        if let Some(arch) = extract_target_arch(input) {
            return Ok(CfgExpr::TargetArch(arch));
        }
        // Handle target_vendor expressions
        if let Some(vendor) = extract_target_vendor(input) {
            return Ok(CfgExpr::TargetVendor(vendor));
        }

        // Handle target_os expressions
        if let Some(os) = extract_target_os(input) {
            return Ok(CfgExpr::TargetOs(os));
        }

        // Handle target_env expressions
        if let Some(env) = extract_target_env(input) {
            return Ok(CfgExpr::TargetEnv(env));
        }

        // If we can't parse it, store it as "Other"
        Ok(CfgExpr::Other(input.to_string()))
    }

    /// Process features according to the rules:
    /// - Features in enabled list: replaced with true and removed from expression
    /// - Features in disabled list: replaced with false and removed from expression  
    /// - Features in mapping list: renamed
    /// - Any unmapped feature remaining: panic with "unmapped feature"
    /// - If architecture, os, target, env condition is specified, replace
    ///   matching condition with false and unmatching to true
    /// - If architecture, os, target, env condition is not specified, leave as is
    pub fn apply_rules(
        &self,
        rules: &CfgExprRules,
        source_location: &SourceLocation,
    ) -> Option<Self> {
        match self {
            CfgExpr::Feature(name) => {
                if rules.enabled_features.contains(name) {
                    // Feature is enabled - replace with true (remove from expression)
                    None // This means "always true", caller should handle removal
                } else if rules.disabled_features.contains(name) {
                    // Feature is disabled - replace with false (remove from expression)
                    Some(CfgExpr::False) // Explicit false value
                } else if let Some(new_name) = rules.feature_mappings.get(name) {
                    // Feature should be mapped
                    Some(CfgExpr::Feature(new_name.clone()))
                } else if rules.disable_unknown_features {
                    // Treat unknown feature as disabled
                    Some(CfgExpr::False)
                } else {
                    // Unmapped feature - panic with source location information
                    panic!("unmapped feature: {name} (at {source_location})");
                }
            }
            CfgExpr::TargetVendor(val) => {
                if let Some(sel) = rules.enabled_target_vendor.as_ref() {
                    if val == sel {
                        None
                    } else {
                        Some(CfgExpr::False)
                    }
                } else {
                    Some(self.clone())
                }
            }
            CfgExpr::TargetArch(val) => {
                if let Some(sel) = rules.enabled_target_arch.as_ref() {
                    if val == sel {
                        None
                    } else {
                        Some(CfgExpr::False)
                    }
                } else {
                    Some(self.clone())
                }
            }
            CfgExpr::All(exprs) => {
                let mut processed_exprs = Vec::new();
                for expr in exprs {
                    match expr.apply_rules(rules, source_location) {
                        Some(CfgExpr::False) => {
                            // If any expression in All is false, the whole All is false
                            return Some(CfgExpr::False);
                        }
                        Some(processed) => processed_exprs.push(processed),
                        None => {
                            // This expression evaluated to true, can be omitted from All
                        }
                    }
                }
                match processed_exprs.len() {
                    0 => None, // All expressions were true, so All is true
                    1 => Some(processed_exprs.into_iter().next().unwrap()), // Simplify single expression
                    _ => Some(CfgExpr::All(processed_exprs)),
                }
            }
            CfgExpr::Any(exprs) => {
                let mut processed_exprs = Vec::new();
                let mut has_true = false;
                for expr in exprs {
                    match expr.apply_rules(rules, source_location) {
                        Some(CfgExpr::False) => {
                            // False expressions in Any can be omitted
                        }
                        Some(processed) => processed_exprs.push(processed),
                        None => {
                            // This expression evaluates to true
                            has_true = true;
                        }
                    }
                }
                if has_true {
                    None // If any expression in Any is true, the whole Any is true
                } else {
                    match processed_exprs.len() {
                        0 => Some(CfgExpr::False), // All expressions were false, so Any is false
                        1 => Some(processed_exprs.into_iter().next().unwrap()), // Simplify single expression
                        _ => Some(CfgExpr::Any(processed_exprs)),
                    }
                }
            }
            CfgExpr::Not(expr) => {
                match expr.apply_rules(rules, source_location) {
                    Some(CfgExpr::False) => None, // not(false) = true
                    Some(processed) => Some(CfgExpr::Not(Box::new(processed))),
                    None => Some(CfgExpr::False), // not(true) = false
                }
            }
            CfgExpr::TargetOs(val) => {
                if let Some(sel) = rules.enabled_target_os.as_ref() {
                    if val == sel {
                        None
                    } else {
                        Some(CfgExpr::False)
                    }
                } else {
                    Some(self.clone())
                }
            }
            CfgExpr::TargetEnv(val) => {
                if let Some(sel) = rules.enabled_target_env.as_ref() {
                    if val == sel {
                        None
                    } else {
                        Some(CfgExpr::False)
                    }
                } else {
                    Some(self.clone())
                }
            }
            CfgExpr::Other(_) => Some(self.clone()),
            CfgExpr::False => Some(CfgExpr::False),
        }
    }

    /// Convert back to a token stream for syn attributes
    pub fn to_tokens(&self) -> proc_macro2::TokenStream {
        match self {
            CfgExpr::Feature(name) => {
                quote::quote! { feature = #name }
            }
            CfgExpr::TargetArch(arch) => {
                quote::quote! { target_arch = #arch }
            }
            CfgExpr::TargetVendor(vendor) => {
                quote::quote! { target_vendor = #vendor }
            }
            CfgExpr::TargetOs(os) => {
                quote::quote! { target_os = #os }
            }
            CfgExpr::TargetEnv(env) => {
                quote::quote! { target_env = #env }
            }
            CfgExpr::All(exprs) => {
                let tokens: Vec<_> = exprs.iter().map(|e| e.to_tokens()).collect();
                quote::quote! { all(#(#tokens),*) }
            }
            CfgExpr::Any(exprs) => {
                let tokens: Vec<_> = exprs.iter().map(|e| e.to_tokens()).collect();
                quote::quote! { any(#(#tokens),*) }
            }
            CfgExpr::Not(expr) => {
                let inner = expr.to_tokens();
                quote::quote! { not(#inner) }
            }
            CfgExpr::Other(content) => {
                // Try to parse as a token stream
                if let Ok(tokens) = content.parse::<proc_macro2::TokenStream>() {
                    tokens
                } else {
                    // Fallback: treat as identifier
                    let ident = proc_macro2::Ident::new(content, proc_macro2::Span::call_site());
                    quote::quote! { #ident }
                }
            }
            CfgExpr::False => {
                // This should not normally be converted to tokens, but if needed, use a false literal
                quote::quote! { any() } // any() with no arguments is always false
            }
        }
    }
}

/// Extract a simple feature name from expressions like `feature = "name"`
fn extract_simple_feature(input: &str) -> Option<String> {
    use regex::Regex;
    let feature_regex = Regex::new(r#"feature\s*=\s*"([^"]+)""#).unwrap();

    feature_regex
        .captures(input)
        .map(|captures| captures[1].to_string())
}

/// Extract target architecture from expressions like `target_arch = "x86_64"`
fn extract_target_arch(input: &str) -> Option<String> {
    use regex::Regex;
    let arch_regex = Regex::new(r#"target_arch\s*=\s*"([^"]+)""#).unwrap();

    arch_regex
        .captures(input)
        .map(|captures| captures[1].to_string())
}

/// Extract target vendor from expressions like `target_vendor = "apple"`
fn extract_target_vendor(input: &str) -> Option<String> {
    use regex::Regex;
    let re = Regex::new(r#"target_vendor\s*=\s*\"([^\"]+)\""#).unwrap();
    re.captures(input).map(|c| c[1].to_string())
}

/// Extract target OS from expressions like `target_os = "macos"`
fn extract_target_os(input: &str) -> Option<String> {
    use regex::Regex;
    let re = Regex::new(r#"target_os\s*=\s*\"([^\"]+)\""#).unwrap();
    re.captures(input).map(|c| c[1].to_string())
}

/// Extract target env from expressions like `target_env = "gnu"`
fn extract_target_env(input: &str) -> Option<String> {
    use regex::Regex;
    let re = Regex::new(r#"target_env\s*=\s*\"([^\"]+)\""#).unwrap();
    re.captures(input).map(|c| c[1].to_string())
}

/// Strip a function call wrapper, returning the inner content
/// For example: `not(feature = "test")` -> `feature = "test"`
fn strip_function_call(input: &str, function_name: &str) -> Option<String> {
    let input = input.trim();

    // Handle function calls with or without spaces: "not(" and "not ("
    let pattern1 = format!("{function_name}(");
    let pattern2 = format!("{function_name} (");

    if (input.starts_with(&pattern1) || input.starts_with(&pattern2)) && input.ends_with(')') {
        let start = if input.starts_with(&pattern1) {
            pattern1.len()
        } else {
            pattern2.len()
        };
        let end = input.len() - 1;

        if start < end {
            Some(input[start..end].to_string())
        } else {
            Some(String::new())
        }
    } else {
        None
    }
}

/// Parse comma-separated expressions
fn parse_comma_separated(input: &str) -> Result<Vec<CfgExpr>, String> {
    let mut exprs = Vec::new();
    let mut current = String::new();
    let mut paren_depth = 0;
    let mut in_quotes = false;
    let mut escape_next = false;

    for ch in input.chars() {
        if escape_next {
            current.push(ch);
            escape_next = false;
            continue;
        }

        match ch {
            '\\' => {
                escape_next = true;
                current.push(ch);
            }
            '"' => {
                in_quotes = !in_quotes;
                current.push(ch);
            }
            '(' if !in_quotes => {
                paren_depth += 1;
                current.push(ch);
            }
            ')' if !in_quotes => {
                paren_depth -= 1;
                current.push(ch);
            }
            ',' if !in_quotes && paren_depth == 0 => {
                if !current.trim().is_empty() {
                    exprs.push(CfgExpr::parse_from_string(current.trim())?);
                }
                current.clear();
            }
            _ => {
                current.push(ch);
            }
        }
    }

    if !current.trim().is_empty() {
        exprs.push(CfgExpr::parse_from_string(current.trim())?);
    }

    Ok(exprs)
}
