//! Function transformation utilities for converting Rust functions into FFI stubs.
//!
//! This module contains the logic for:
//! - Parsing and validating function signatures for FFI compatibility
//! - Generating `#[no_mangle] extern "C"` wrapper functions
//! - Handling type transformations and validations
//! - Creating appropriate parameter names and call arguments

use roxygen::roxygen;
use std::collections::HashSet;

/// Codegen structure containing common parameters for code generation
///
/// This structure holds all the configuration and state needed for processing
/// features and transforming functions to FFI stubs. It centralizes the parameters
/// that were previously passed individually to multiple functions.
struct Codegen<'a> {
    /// The source crate identifier (crate name with dashes converted to underscores)
    pub source_crate_ident: syn::Ident,
    /// Set of exported type names that are valid for FFI
    pub exported_types: &'a HashSet<String>,
    /// List of allowed path prefixes for type validation
    pub allowed_prefixes: &'a [syn::Path],
    /// List of transparent wrapper types to strip during conversion
    pub transparent_wrappers: &'a [syn::Path],
    /// Rust edition string (e.g., "2021", "2024") for proper attribute generation
    pub edition: &'a str,
}

impl<'a> Codegen<'a> {
    /// Create a new Codegen with the specified parameters
    pub fn new(
        crate_name: &'a str,
        exported_types: &'a HashSet<String>,
        allowed_prefixes: &'a [syn::Path],
        transparent_wrappers: &'a [syn::Path],
        edition: &'a str,
    ) -> Self {
        // Convert crate name to identifier (replace dashes with underscores)
        let source_crate_name = crate_name.replace('-', "_");
        let source_crate_ident = syn::Ident::new(&source_crate_name, proc_macro2::Span::call_site());
        
        Self {
            source_crate_ident,
            exported_types,
            allowed_prefixes,
            transparent_wrappers,
            edition,
        }
    }

    /// Transform a function definition into an FFI stub
    ///
    /// This method takes a parsed Rust function and transforms it into a `#[no_mangle] extern "C"`
    /// wrapper function that calls the original function from the source crate.
    #[roxygen]
    pub(crate) fn transform_function_to_stub(
        &self,
        /// The parsed file containing exactly one function definition
        file: syn::File,
        /// Mutable set to collect type assertion pairs for compile-time validation
        assertion_type_pairs: &mut HashSet<(String, String)>,
        /// Source location information for error reporting
        source_location: &crate::SourceLocation,
    ) -> Result<syn::File, String> {
        // Validate that the file contains exactly one function
        if file.items.len() != 1 {
            return Err(format!(
                "Expected exactly one item in file, found {}",
                file.items.len()
            ));
        }

        let parsed_function = match &file.items[0] {
            syn::Item::Fn(item_fn) => item_fn,
            item => {
                return Err(format!(
                    "Expected function item, found {:?}",
                    std::mem::discriminant(item)
                ));
            }
        };

        let function_name = &parsed_function.sig.ident;

        // Build the extern "C" function signature:
        // 1. Start with the original function signature
        // 2. Convert references to pointers for FFI compatibility
        // 3. Add extern "C" ABI specifier
        // 4. Mark function as unsafe
        let mut extern_sig = parsed_function.sig.clone();

        // Convert return type and collect type assertion pairs
        let mut result_type_changed = false;
        if let syn::ReturnType::Type(arrow, return_type) = &extern_sig.output {
            let local_return_type = self.convert_to_local_type(
                return_type,
                assertion_type_pairs,
                &mut result_type_changed,
                &format!("return type of function '{function_name}'"),
            ).map_err(|e| format!("Invalid FFI function return type: {} (at {}:{}:{})", e, source_location.file, source_location.line, source_location.column))?;
            extern_sig.output = syn::ReturnType::Type(*arrow, Box::new(local_return_type));
        }

        // Convert reference parameters to pointer parameters and collect type assertion pairs
        // Also build call arguments with appropriate transmute/conversion logic
        let mut call_args = Vec::new();
        for input in extern_sig.inputs.iter_mut() {
            let syn::FnArg::Typed(pat_type) = input else {
                panic!(
                    "FFI functions cannot have receiver arguments (like 'self'). \
                     All parameters must be typed arguments for C compatibility."
                );
            };
            // Convert type and collect assertion pairs (handles both reference and non-reference types)
            let mut type_changed = false;
            let local_type = self.convert_to_local_type(
                &pat_type.ty,
                assertion_type_pairs,
                &mut type_changed,
                &format!("parameter of function '{function_name}'"),
            ).map_err(|e| format!("Invalid FFI function parameter: {} (at {}:{}:{})", e, source_location.file, source_location.line, source_location.column))?;

            // Generate call argument based on the original type and conversion status
            if let syn::Pat::Ident(pat_ident) = &*pat_type.pat {
                let param_name = &pat_ident.ident;
                // Build the call argument based on the original type and conversion status
                let base_arg = match &*pat_type.ty {
                    syn::Type::Reference(type_ref) => {
                        if type_ref.mutability.is_some() {
                            // &mut T parameter => *mut T in FFI, convert back with &mut *param
                            quote::quote! { &mut *#param_name }
                        } else {
                            // &T parameter => *const T in FFI, convert back with &*param
                            quote::quote! { &*#param_name }
                        }
                    }
                    _ => {
                        // Non-reference parameters are passed through
                        quote::quote! { #param_name }
                    }
                };

                // Wrap with transmute if the type was changed due to exported type conversion
                let final_arg = if type_changed {
                    quote::quote! { unsafe { std::mem::transmute(#base_arg) } }
                } else {
                    base_arg
                };
                call_args.push(final_arg);
            }

            // Update the parameter type in the signature to the local type
            pat_type.ty = Box::new(local_type);
        }

        // Mark function as unsafe and use C ABI
        extern_sig.unsafety = Some(syn::Token![unsafe](proc_macro2::Span::call_site()));
        extern_sig.abi = Some(syn::Abi {
            extern_token: syn::Token![extern](proc_macro2::Span::call_site()),
            name: Some(syn::LitStr::new("C", proc_macro2::Span::call_site())),
        });

        // Generate the function body
        let source_crate_ident = &self.source_crate_ident;
        let function_body = if result_type_changed {
            // If return type was converted, wrap the result in transmute
            quote::quote! {
                let result = #source_crate_ident::#function_name(#(#call_args),*);
                unsafe { std::mem::transmute(result) }
            }
        } else {
            // Direct call without transmute
            match &extern_sig.output {
                syn::ReturnType::Default => {
                    quote::quote! {
                        #source_crate_ident::#function_name(#(#call_args),*)
                    }
                }
                syn::ReturnType::Type(_, _) => {
                    quote::quote! {
                        #source_crate_ident::#function_name(#(#call_args),*)
                    }
                }
            }
        };

        // Determine the appropriate no_mangle attribute based on Rust edition
        // Edition 2024 uses #[unsafe(no_mangle)], while older editions use #[no_mangle]
        let no_mangle_attr: syn::Attribute = if self.edition == "2024" {
            syn::parse_quote! { #[unsafe(no_mangle)] }
        } else {
            syn::parse_quote! { #[no_mangle] }
        };

        // Create the function body that will call the original implementation
        let body = syn::parse_quote! {
            {
                #function_body
            }
        };

        // Build the final extern function
        let extern_function = syn::ItemFn {
            attrs: vec![no_mangle_attr],
            vis: syn::Visibility::Public(syn::Token![pub](proc_macro2::Span::call_site())),
            sig: extern_sig,
            block: Box::new(body),
        };

        Ok(syn::File {
            shebang: file.shebang,
            attrs: file.attrs,
            items: vec![syn::Item::Fn(extern_function)],
        })
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

/// Transform a function definition into an FFI stub
///
/// This function takes a parsed Rust function and transforms it into a `#[no_mangle] extern "C"`
/// wrapper function that calls the original function from the source crate.
/// This is a public interface that doesn't perform type replacement.
#[roxygen]
pub fn transform_function_to_stub(
    /// The parsed file containing exactly one function definition
    file: syn::File,
    /// The source crate name
    crate_name: &str,
    /// Set of exported type names that are valid for FFI
    exported_types: &HashSet<String>,
    /// List of allowed path prefixes for type validation
    allowed_prefixes: &[syn::Path],
    /// List of transparent wrapper types to strip during conversion
    transparent_wrappers: &[syn::Path],
    /// Rust edition string (e.g., "2021", "2024") for proper attribute generation
    edition: &str,
    /// Source location information for error reporting
    source_location: &crate::SourceLocation,
) -> Result<(syn::File, HashSet<(String, String)>), String> {
    // Create a temporary codegen instance
    let codegen = Codegen::new(
        crate_name,
        exported_types,
        allowed_prefixes,
        transparent_wrappers,
        edition,
    );

    // Transform the function and collect type replacement pairs
    let mut assertion_type_pairs = HashSet::new();
    let transformed_file = codegen.transform_function_to_stub(
        file,
        &mut assertion_type_pairs,
        source_location,
    )?;

    Ok((transformed_file, assertion_type_pairs))
}
