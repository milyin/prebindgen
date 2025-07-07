//! Type replacement utilities for converting types from their original form to FFI-compatible form.
//!
//! This module contains the logic for:
//! - Converting types using the same logic as FFI stub generation
//! - Handling transparent wrapper stripping
//! - Processing exported types with proper crate prefixing
//! - Generating type assertion pairs for compile-time validation

#![allow(dead_code)]

use roxygen::roxygen;
use std::collections::HashSet;

/// Replace types structure containing common parameters for type replacement
///
/// This structure holds all the configuration and state needed for processing
/// type replacements. It centralizes the parameters that are used for converting
/// types from their original form to their local FFI-compatible form.
struct ReplaceTypes<'a> {
    /// The source crate identifier (crate name with dashes converted to underscores)
    pub source_crate_ident: syn::Ident,
    /// Set of exported type names that are valid for FFI
    pub exported_types: &'a HashSet<String>,
    /// List of allowed path prefixes for type validation
    pub allowed_prefixes: &'a [syn::Path],
    /// List of transparent wrapper types to strip during conversion
    pub transparent_wrappers: &'a [syn::Path],
}

impl<'a> ReplaceTypes<'a> {
    /// Create a new ReplaceTypes with the specified parameters
    pub fn new(
        crate_name: &'a str,
        exported_types: &'a HashSet<String>,
        allowed_prefixes: &'a [syn::Path],
        transparent_wrappers: &'a [syn::Path],
    ) -> Self {
        // Convert crate name to identifier (replace dashes with underscores)
        let source_crate_name = crate_name.replace('-', "_");
        let source_crate_ident = syn::Ident::new(&source_crate_name, proc_macro2::Span::call_site());
        
        Self {
            source_crate_ident,
            exported_types,
            allowed_prefixes,
            transparent_wrappers,
        }
    }

    /// Convert a remote type to its local equivalent, validate FFI compatibility, and collect assertion pairs
    ///
    /// This method:
    /// - Validates that the type is suitable for FFI use
    /// - Strips transparent wrappers from the type
    /// - Converts references to pointers for FFI compatibility
    /// - Collects type assertion pairs when conversion is needed
    /// - Sets the `was_converted` flag to indicate if the type was modified
    #[roxygen]
    pub(crate) fn convert_to_local_type(
        &self,
        /// The original type to convert and validate
        original_type: &syn::Type,
        /// Mutable set to collect assertion pairs
        assertion_type_pairs: &mut HashSet<(String, String)>,
        /// Mutable boolean flag set to true if type was changed and needs transmute
        type_changed: &mut bool,
        /// Context string for error reporting (e.g., "parameter 1 of function 'foo'")
        context: &str,
    ) -> Result<syn::Type, String> {
        // First validate the type for FFI compatibility
        super::validate_type_for_ffi_impl(original_type, self.exported_types, self.allowed_prefixes, context)?;

        // Extract the core type to process (for references, this is the referenced type)
        let (core_type, is_reference, ref_info) = match original_type {
            syn::Type::Reference(type_ref) => (
                &*type_ref.elem,
                true,
                Some((
                    type_ref.and_token,
                    type_ref.lifetime.clone(),
                    type_ref.mutability,
                )),
            ),
            _ => (original_type, false, None),
        };

        // Strip transparent wrappers from the core type
        let mut has_wrapper = false;
        let local_core_type =
            super::strip_transparent_wrappers(core_type, self.transparent_wrappers, &mut has_wrapper);

        // Check if we should generate an assertion for this type
        *type_changed = has_wrapper || super::contains_exported_type(&local_core_type, self.exported_types);

        if *type_changed {
            // Create the original core type with proper crate prefixing
            let prefixed_original_core =
                super::prefix_exported_types_in_type(core_type, &self.source_crate_ident, self.exported_types);

            // Store the assertion pair
            let local_core_str = quote::quote! { #local_core_type }.to_string();
            let prefixed_original_core_str = quote::quote! { #prefixed_original_core }.to_string();
            assertion_type_pairs.insert((local_core_str, prefixed_original_core_str));
        }

        // Build the final type based on whether the original was a reference
        let result = if is_reference {
            let (and_token, lifetime, mutability) = ref_info.unwrap();
            // Create a reference to the local type, then convert to pointer
            let local_ref = syn::Type::Reference(syn::TypeReference {
                and_token,
                lifetime,
                mutability,
                elem: Box::new(local_core_type),
            });
            // Mark as converted only if the referenced type needed conversion
            // Reference-to-pointer conversion is always done for FFI, but transmute is only needed
            // if the referenced type itself is converted (wrapper stripped or exported type)
            super::convert_reference_to_pointer(&local_ref)
        } else if *type_changed {
            // Non-reference type that needed conversion
            local_core_type
        } else {
            // No conversion needed, return original type
            original_type.clone()
        };

        Ok(result)
    }
}

/// Replace types in code using type conversion
///
/// This function takes code and converts types from their original form to their
/// local FFI-compatible form using the same logic as the FFI stub generation.
#[roxygen]
pub fn replace_types(
    /// The code to process for type replacements
    mut file: syn::File,
    /// The source crate name
    crate_name: &str,
    /// Set of exported type names that are valid for FFI
    exported_types: &HashSet<String>,
    /// List of allowed path prefixes for type validation
    allowed_prefixes: &[syn::Path],
    /// List of transparent wrapper types to strip during conversion
    transparent_wrappers: &[syn::Path],
) -> (syn::File, HashSet<(String, String)>) {
    // Create the type replacer
    let replacer = ReplaceTypes::new(
        crate_name,
        exported_types,
        allowed_prefixes,
        transparent_wrappers,
    );

    let mut assertion_type_pairs = HashSet::new();

    // Apply type replacements throughout the file
    for item in &mut file.items {
        replace_types_in_item(item, &replacer, &mut assertion_type_pairs);
    }

    (file, assertion_type_pairs)
}

/// Replace types in a single item recursively
fn replace_types_in_item(item: &mut syn::Item, replacer: &ReplaceTypes, assertion_type_pairs: &mut HashSet<(String, String)>) {
    match item {
        syn::Item::Fn(item_fn) => {
            replace_types_in_signature(&mut item_fn.sig, replacer, assertion_type_pairs);
            replace_types_in_block(&mut item_fn.block, replacer, assertion_type_pairs);
        }
        syn::Item::Struct(item_struct) => {
            replace_types_in_fields(&mut item_struct.fields, replacer, assertion_type_pairs);
        }
        syn::Item::Enum(item_enum) => {
            for variant in &mut item_enum.variants {
                replace_types_in_fields(&mut variant.fields, replacer, assertion_type_pairs);
            }
        }
        syn::Item::Union(item_union) => {
            replace_types_in_fields(&mut syn::Fields::Named(item_union.fields.clone()), replacer, assertion_type_pairs);
        }
        syn::Item::Type(item_type) => {
            replace_types_in_type(&mut item_type.ty, replacer, assertion_type_pairs);
        }
        syn::Item::Const(item_const) => {
            replace_types_in_type(&mut item_const.ty, replacer, assertion_type_pairs);
        }
        syn::Item::Static(item_static) => {
            replace_types_in_type(&mut item_static.ty, replacer, assertion_type_pairs);
        }
        _ => {
            // Other items don't contain types we need to replace
        }
    }
}

/// Replace types in function signature
fn replace_types_in_signature(sig: &mut syn::Signature, replacer: &ReplaceTypes, assertion_type_pairs: &mut HashSet<(String, String)>) {
    // Replace parameter types
    for (i, input) in sig.inputs.iter_mut().enumerate() {
        if let syn::FnArg::Typed(pat_type) = input {
            replace_types_in_type_with_context(&mut pat_type.ty, replacer, assertion_type_pairs, &format!("parameter {}", i + 1));
        }
    }

    // Replace return type
    if let syn::ReturnType::Type(_, return_type) = &mut sig.output {
        replace_types_in_type_with_context(return_type, replacer, assertion_type_pairs, "return type");
    }
}

/// Replace types in a function block
fn replace_types_in_block(block: &mut syn::Block, _replacer: &ReplaceTypes, _assertion_type_pairs: &mut HashSet<(String, String)>) {
    // For now, we don't need to replace types in function bodies
    // This can be extended if needed
    let _ = block;
}

/// Replace types in struct/enum fields
fn replace_types_in_fields(fields: &mut syn::Fields, replacer: &ReplaceTypes, assertion_type_pairs: &mut HashSet<(String, String)>) {
    match fields {
        syn::Fields::Named(fields_named) => {
            for (i, field) in fields_named.named.iter_mut().enumerate() {
                let context = if let Some(field_name) = &field.ident {
                    format!("field '{field_name}'")
                } else {
                    format!("field {i}")
                };
                replace_types_in_type_with_context(&mut field.ty, replacer, assertion_type_pairs, &context);
            }
        }
        syn::Fields::Unnamed(fields_unnamed) => {
            for (i, field) in fields_unnamed.unnamed.iter_mut().enumerate() {
                replace_types_in_type_with_context(&mut field.ty, replacer, assertion_type_pairs, &format!("field {i}"));
            }
        }
        syn::Fields::Unit => {
            // No fields to process
        }
    }
}

/// Replace a type based on the replacement logic with context
fn replace_types_in_type_with_context(ty: &mut syn::Type, replacer: &ReplaceTypes, assertion_type_pairs: &mut HashSet<(String, String)>, context: &str) {
    // Try to convert the type using the same logic as FFI stub generation
    let mut type_changed = false;
    match replacer.convert_to_local_type(ty, assertion_type_pairs, &mut type_changed, context) {
        Ok(converted_type) => {
            if type_changed {
                *ty = converted_type;
            }
        }
        Err(_) => {
            // If conversion fails, leave the type as-is
            // This preserves the original behavior for unsupported types
        }
    }
}

/// Replace a type based on the replacement logic
fn replace_types_in_type(ty: &mut syn::Type, replacer: &ReplaceTypes, assertion_type_pairs: &mut HashSet<(String, String)>) {
    replace_types_in_type_with_context(ty, replacer, assertion_type_pairs, "type");
}
