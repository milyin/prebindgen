//! Feature flag processing utilities for handling `#[cfg(feature="...")]` attributes.
//!
//! This module contains the logic for:
//! - Processing feature flags according to builder configuration
//! - Removing code blocks guarded by disabled features
//! - Removing cfg attributes for enabled features (keeping the code)
//! - Replacing feature names according to the mapping

use roxygen::roxygen;
use std::collections::{HashMap, HashSet};

use crate::{codegen::cfg_expr::CfgExpr, SourceLocation};

/// Process a single item (struct, enum, function, etc.) for feature flags
///
/// This function analyzes code for `#[cfg(feature="...")]` attributes using syn syntax parsing and:
/// - Removes code blocks guarded by disabled features
/// - Removes cfg attributes for enabled features (keeping the code)
/// - Replaces feature names according to the mapping (keeping the cfg attribute)
#[roxygen]
pub(crate) fn process_item_features(
    /// The item to process for feature flags
    item: &mut syn::Item,
    /// Set of feature names that should have their cfg attributes removed
    disabled_features: &HashSet<String>,
    /// Mapping from old feature names to new feature names
    enabled_features: &HashSet<String>,
    /// Mapping from old feature names to new feature names
    feature_mappings: &HashMap<String, String>,
    /// If true, unknown features are treated as disabled (skipped) instead of causing an error
    disable_unknown_features: bool,
    /// Selected target parameters to evaluate target_* cfgs
    enabled_target_arch: &Option<String>,
    enabled_target_vendor: &Option<String>,
    enabled_target_os: &Option<String>,
    enabled_target_env: &Option<String>,
    /// Source location information for error reporting
    source_location: &SourceLocation,
) -> bool {
    let attrs = match item {
        syn::Item::Fn(f) => &mut f.attrs,
        syn::Item::Struct(s) => {
            // Process struct fields recursively
            process_struct_fields(
                &mut s.fields,
                disabled_features,
                enabled_features,
                feature_mappings,
                disable_unknown_features,
                enabled_target_arch,
                enabled_target_vendor,
                enabled_target_os,
                enabled_target_env,
                source_location,
            );
            &mut s.attrs
        }
        syn::Item::Enum(e) => {
            // Process enum variants recursively
            process_enum_variants(
                &mut e.variants,
                disabled_features,
                enabled_features,
                feature_mappings,
                disable_unknown_features,
                enabled_target_arch,
                enabled_target_vendor,
                enabled_target_os,
                enabled_target_env,
                source_location,
            );
            &mut e.attrs
        }
        syn::Item::Union(u) => {
            // Process union fields recursively
            process_union_fields(
                &mut u.fields,
                disabled_features,
                enabled_features,
                feature_mappings,
                disable_unknown_features,
                enabled_target_arch,
                enabled_target_vendor,
                enabled_target_os,
                enabled_target_env,
                source_location,
            );
            &mut u.attrs
        }
        syn::Item::Type(t) => &mut t.attrs,
        syn::Item::Const(c) => &mut c.attrs,
        syn::Item::Static(s) => &mut s.attrs,
        syn::Item::Mod(m) => &mut m.attrs,
        syn::Item::Use(u) => &mut u.attrs,
        syn::Item::Impl(i) => &mut i.attrs,
        syn::Item::Trait(t) => &mut t.attrs,
        _ => return true, // Keep other items as-is
    };

    // Use the centralized attribute processing function
    process_attributes(
        attrs,
        disabled_features,
        enabled_features,
    feature_mappings,
    disable_unknown_features,
    enabled_target_arch,
    enabled_target_vendor,
    enabled_target_os,
    enabled_target_env,
        source_location,
    )
}

/// Process struct fields for feature flags
fn process_struct_fields(
    fields: &mut syn::Fields,
    disabled_features: &HashSet<String>,
    enabled_features: &HashSet<String>,
    feature_mappings: &HashMap<String, String>,
    disable_unknown_features: bool,
    enabled_target_arch: &Option<String>,
    enabled_target_vendor: &Option<String>,
    enabled_target_os: &Option<String>,
    enabled_target_env: &Option<String>,
    source_location: &SourceLocation,
) {
    match fields {
        syn::Fields::Named(fields_named) => {
            // Manual filtering since Punctuated doesn't have retain_mut
            let mut new_fields = syn::punctuated::Punctuated::new();
            for field in fields_named.named.pairs() {
                let mut field = field.into_value().clone();
                if process_attributes(
                    &mut field.attrs,
                    disabled_features,
                    enabled_features,
                    feature_mappings,
                    disable_unknown_features,
                    enabled_target_arch,
                    enabled_target_vendor,
                    enabled_target_os,
                    enabled_target_env,
                    source_location,
                ) {
                    new_fields.push(field);
                }
            }
            fields_named.named = new_fields;
        }
        syn::Fields::Unnamed(fields_unnamed) => {
            // Manual filtering since Punctuated doesn't have retain_mut
            let mut new_fields = syn::punctuated::Punctuated::new();
            for field in fields_unnamed.unnamed.pairs() {
                let mut field = field.into_value().clone();
                if process_attributes(
                    &mut field.attrs,
                    disabled_features,
                    enabled_features,
                    feature_mappings,
                    disable_unknown_features,
                    enabled_target_arch,
                    enabled_target_vendor,
                    enabled_target_os,
                    enabled_target_env,
                    source_location,
                ) {
                    new_fields.push(field);
                }
            }
            fields_unnamed.unnamed = new_fields;
        }
        syn::Fields::Unit => {
            // No fields to process
        }
    }
}

/// Process enum variants for feature flags
fn process_enum_variants(
    variants: &mut syn::punctuated::Punctuated<syn::Variant, syn::Token![,]>,
    disabled_features: &HashSet<String>,
    enabled_features: &HashSet<String>,
    feature_mappings: &HashMap<String, String>,
    disable_unknown_features: bool,
    enabled_target_arch: &Option<String>,
    enabled_target_vendor: &Option<String>,
    enabled_target_os: &Option<String>,
    enabled_target_env: &Option<String>,
    source_location: &SourceLocation,
) {
    // Manual filtering since Punctuated doesn't have retain_mut
    let mut new_variants = syn::punctuated::Punctuated::new();
    for variant_pair in variants.pairs() {
        let mut variant = variant_pair.into_value().clone();
        // Process variant attributes
        let keep_variant = process_attributes(
            &mut variant.attrs,
            disabled_features,
            enabled_features,
            feature_mappings,
            disable_unknown_features,
            enabled_target_arch,
            enabled_target_vendor,
            enabled_target_os,
            enabled_target_env,
            source_location,
        );

        if keep_variant {
            // Process variant fields if it's kept
            process_struct_fields(
                &mut variant.fields,
                disabled_features,
                enabled_features,
                feature_mappings,
                disable_unknown_features,
                enabled_target_arch,
                enabled_target_vendor,
                enabled_target_os,
                enabled_target_env,
                source_location,
            );
            new_variants.push(variant);
        }
    }
    *variants = new_variants;
}

/// Process union fields for feature flags
fn process_union_fields(
    fields: &mut syn::FieldsNamed,
    disabled_features: &HashSet<String>,
    enabled_features: &HashSet<String>,
    feature_mappings: &HashMap<String, String>,
    disable_unknown_features: bool,
    enabled_target_arch: &Option<String>,
    enabled_target_vendor: &Option<String>,
    enabled_target_os: &Option<String>,
    enabled_target_env: &Option<String>,
    source_location: &SourceLocation,
) {
    // Manual filtering since Punctuated doesn't have retain_mut
    let mut new_fields = syn::punctuated::Punctuated::new();
    for field_pair in fields.named.pairs() {
        let mut field = field_pair.into_value().clone();
        if process_attributes(
            &mut field.attrs,
            disabled_features,
            enabled_features,
            feature_mappings,
            disable_unknown_features,
            enabled_target_arch,
            enabled_target_vendor,
            enabled_target_os,
            enabled_target_env,
            source_location,
        ) {
            new_fields.push(field);
        }
    }
    fields.named = new_fields;
}

/// Process attributes for feature flags and return whether the item should be kept
fn process_attributes(
    attrs: &mut Vec<syn::Attribute>,
    disabled_features: &HashSet<String>,
    enabled_features: &HashSet<String>,
    feature_mappings: &HashMap<String, String>,
    disable_unknown_features: bool,
    enabled_target_arch: &Option<String>,
    enabled_target_vendor: &Option<String>,
    enabled_target_os: &Option<String>,
    enabled_target_env: &Option<String>,
    source_location: &SourceLocation,
) -> bool {
    let mut keep_item = true;
    let mut remove_attrs = Vec::new();

    for (i, attr) in attrs.iter_mut().enumerate() {
        // Check if this is a cfg attribute
        if attr.path().is_ident("cfg") {
            // Parse the meta to extract cfg information
            if let syn::Meta::List(meta_list) = &attr.meta {
                // Parse the cfg expression using our advanced parser
                match CfgExpr::parse_from_tokens(&meta_list.tokens) {
                    Ok(cfg_expr) => {
                        // Apply strict feature processing
                        match cfg_expr.process_features_strict(
                            enabled_features,
                            disabled_features,
                            feature_mappings,
                            disable_unknown_features,
                            enabled_target_arch,
                            enabled_target_vendor,
                            enabled_target_os,
                            enabled_target_env,
                            source_location,
                        ) {
                            Some(processed_expr) => {
                                // Check if the processed expression is CfgExpr::False
                                if matches!(processed_expr, CfgExpr::False) {
                                    // Expression evaluates to false, exclude this item
                                    keep_item = false;
                                    break;
                                } else {
                                    // Expression still exists after processing, update the cfg attribute
                                    let new_tokens = processed_expr.to_tokens();
                                    let new_meta = syn::parse_quote! {
                                        cfg(#new_tokens)
                                    };
                                    attr.meta = new_meta;
                                }
                            }
                            None => {
                                // Expression evaluates to true, remove the cfg attribute
                                remove_attrs.push(i);
                            }
                        }
                    }
                    Err(_) => {
                        // If we can't parse the cfg expression, leave it as-is
                        // This preserves the original behavior for unsupported expressions
                    }
                }
            }
        }
    }

    // Remove attributes that should be removed (in reverse order to maintain indices)
    for &i in remove_attrs.iter().rev() {
        attrs.remove(i);
    }

    keep_item
}
