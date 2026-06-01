//! Feature flag processing utilities for handling `#[cfg(feature="...")]` attributes.
//!
//! This module contains the logic for:
//! - Processing feature flags according to builder configuration
//! - Removing code blocks guarded by disabled features
//! - Removing cfg attributes for enabled features (keeping the code)
//! - Replacing feature names according to the mapping

use roxygen::roxygen;

use crate::{
    codegen::{cfg_expr::CfgExpr, CfgExprRules},
    SourceLocation,
};

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
    rules: &CfgExprRules,
    /// Source location information for error reporting
    source_location: &SourceLocation,
) -> bool {
    let attrs = match item {
        syn::Item::Fn(f) => {
            // Process function parameters recursively (so `#[cfg(...)]` on an
            // individual parameter is honored, just like struct fields / enum
            // variants) — avoids needing two whole-function variants.
            process_fn_inputs(&mut f.sig.inputs, rules, source_location);
            &mut f.attrs
        }
        syn::Item::Struct(s) => {
            // Process struct fields recursively
            process_struct_fields(&mut s.fields, rules, source_location);
            &mut s.attrs
        }
        syn::Item::Enum(e) => {
            // Process enum variants recursively
            process_enum_variants(&mut e.variants, rules, source_location);
            &mut e.attrs
        }
        syn::Item::Union(u) => {
            // Process union fields recursively
            process_union_fields(&mut u.fields, rules, source_location);
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
    process_attributes(attrs, rules, source_location)
}

/// Process struct fields for feature flags
fn process_struct_fields(
    fields: &mut syn::Fields,
    rules: &CfgExprRules,
    source_location: &SourceLocation,
) {
    match fields {
        syn::Fields::Named(fields_named) => {
            // Manual filtering since Punctuated doesn't have retain_mut
            let mut new_fields = syn::punctuated::Punctuated::new();
            for field in fields_named.named.pairs() {
                let mut field = field.into_value().clone();
                if process_attributes(&mut field.attrs, rules, source_location) {
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
                if process_attributes(&mut field.attrs, rules, source_location) {
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

/// Process function parameters for feature flags
///
/// Drops any parameter guarded by a disabled-feature `#[cfg(...)]` and removes
/// the `#[cfg(...)]` attribute from parameters guarded by enabled features.
/// Parameters without a cfg attribute are left untouched. Receiver (`self`)
/// parameters are always kept.
fn process_fn_inputs(
    inputs: &mut syn::punctuated::Punctuated<syn::FnArg, syn::Token![,]>,
    rules: &CfgExprRules,
    source_location: &SourceLocation,
) {
    let mut new_inputs = syn::punctuated::Punctuated::new();
    for input_pair in inputs.pairs() {
        let mut input = input_pair.into_value().clone();
        let keep = match &mut input {
            syn::FnArg::Typed(pt) => process_attributes(&mut pt.attrs, rules, source_location),
            syn::FnArg::Receiver(r) => process_attributes(&mut r.attrs, rules, source_location),
        };
        if keep {
            new_inputs.push(input);
        }
    }
    *inputs = new_inputs;
}

/// Process enum variants for feature flags
fn process_enum_variants(
    variants: &mut syn::punctuated::Punctuated<syn::Variant, syn::Token![,]>,
    rules: &CfgExprRules,
    source_location: &SourceLocation,
) {
    // Manual filtering since Punctuated doesn't have retain_mut
    let mut new_variants = syn::punctuated::Punctuated::new();
    for variant_pair in variants.pairs() {
        let mut variant = variant_pair.into_value().clone();
        // Process variant attributes
        let keep_variant = process_attributes(&mut variant.attrs, rules, source_location);

        if keep_variant {
            // Process variant fields if it's kept
            process_struct_fields(&mut variant.fields, rules, source_location);
            new_variants.push(variant);
        }
    }
    *variants = new_variants;
}

/// Process union fields for feature flags
fn process_union_fields(
    fields: &mut syn::FieldsNamed,
    rules: &CfgExprRules,
    source_location: &SourceLocation,
) {
    // Manual filtering since Punctuated doesn't have retain_mut
    let mut new_fields = syn::punctuated::Punctuated::new();
    for field_pair in fields.named.pairs() {
        let mut field = field_pair.into_value().clone();
        if process_attributes(&mut field.attrs, rules, source_location) {
            new_fields.push(field);
        }
    }
    fields.named = new_fields;
}

/// Process attributes for feature flags and return whether the item should be kept
fn process_attributes(
    attrs: &mut Vec<syn::Attribute>,
    rules: &CfgExprRules,
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
                        match cfg_expr.apply_rules(rules, source_location) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use quote::ToTokens;

    fn rules(enabled: &[&str], disabled: &[&str]) -> CfgExprRules {
        CfgExprRules {
            enabled_features: enabled.iter().map(|s| s.to_string()).collect(),
            disabled_features: disabled.iter().map(|s| s.to_string()).collect(),
            ..Default::default()
        }
    }

    /// A `#[cfg(feature = "...")]` on a single function parameter is honored:
    /// dropped when the feature is disabled, kept (cfg attr stripped) when
    /// enabled — without needing two whole-function variants.
    #[test]
    fn cfg_on_fn_parameter_is_filtered() {
        let src = SourceLocation::default();
        let make = || -> syn::Item {
            syn::parse_quote! {
                pub fn f(a: i32, #[cfg(feature = "unstable")] b: i64) -> i32 { a }
            }
        };

        // Disabled → the guarded parameter is removed; the rest is intact.
        let mut item = make();
        assert!(process_item_features(
            &mut item,
            &rules(&[], &["unstable"]),
            &src
        ));
        let s = item.to_token_stream().to_string();
        assert!(s.contains("a : i32"), "{s}");
        assert!(!s.contains("b : i64"), "{s}");
        assert!(!s.contains("cfg"), "cfg attr should be gone: {s}");

        // Enabled → the parameter is kept and its cfg attribute is stripped.
        let mut item = make();
        assert!(process_item_features(
            &mut item,
            &rules(&["unstable"], &[]),
            &src
        ));
        let s = item.to_token_stream().to_string();
        assert!(s.contains("a : i32"), "{s}");
        assert!(s.contains("b : i64"), "{s}");
        assert!(
            !s.contains("cfg"),
            "cfg attr should be stripped on keep: {s}"
        );
    }

    /// Parameters without a cfg attribute are untouched.
    #[test]
    fn fn_parameters_without_cfg_are_preserved() {
        let src = SourceLocation::default();
        let mut item: syn::Item = syn::parse_quote! {
            pub fn g(x: u8, y: u16, z: u32) {}
        };
        assert!(process_item_features(
            &mut item,
            &rules(&[], &["unstable"]),
            &src
        ));
        let s = item.to_token_stream().to_string();
        assert!(
            s.contains("x : u8") && s.contains("y : u16") && s.contains("z : u32"),
            "{s}"
        );
    }
}
