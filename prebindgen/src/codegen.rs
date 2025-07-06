//! Code generation utilities for transforming Rust function definitions into FFI stubs.
//!
//! This module contains all the logic for:
//! - Parsing and validating function signatures for FFI compatibility
//! - Generating `#[no_mangle] extern "C"` wrapper functions
//! - Handling type transformations and validations
//! - Creating appropriate parameter names and call arguments
//! - Processing feature flags (`#[cfg(feature="...")]`) in generated code

use roxygen::roxygen;
use std::collections::{HashMap, HashSet};

/// Codegen structure containing common parameters for code generation
///
/// This structure holds all the configuration and state needed for processing
/// features and transforming functions to FFI stubs. It centralizes the parameters
/// that were previously passed individually to multiple functions.
pub(crate) struct Codegen<'a> {
    /// The name of the source crate (with dashes converted to underscores)
    pub crate_name: &'a str,
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
        Self {
            crate_name,
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

        // Validate function signature
        validate_function_parameters(
            &parsed_function.sig.inputs,
            function_name,
            self.exported_types,
            self.allowed_prefixes,
            source_location,
        )?;

        // Prepare source crate name for type collection
        let source_crate_name = self.crate_name.replace('-', "_");

        // Validate return type
        if let syn::ReturnType::Type(_, return_type) = &parsed_function.sig.output {
            validate_type_for_ffi(
                return_type,
                self.exported_types,
                self.allowed_prefixes,
                &format!("return type of function '{function_name}'"),
            )
            .map_err(|e| format!("Invalid FFI function return type: {} (at {}:{}:{})", e, source_location.file, source_location.line, source_location.column))?;
        }

        // Generate components and build call arguments inline
        let source_crate_ident = syn::Ident::new(&source_crate_name, proc_macro2::Span::call_site());

        // Build the extern "C" function signature:
        // 1. Start with the original function signature
        // 2. Convert references to pointers for FFI compatibility
        // 3. Add extern "C" ABI specifier
        // 4. Mark function as unsafe
        let mut extern_sig = parsed_function.sig.clone();

        // Convert return type and collect type assertion pairs
        let mut result_type_changed = false;
        if let syn::ReturnType::Type(arrow, return_type) = &extern_sig.output {
            let local_return_type = convert_to_local_type(
                return_type,
                self.exported_types,
                self.transparent_wrappers,
                &source_crate_ident,
                assertion_type_pairs,
                &mut result_type_changed,
            );
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
            let local_type = convert_to_local_type(
                &pat_type.ty,
                self.exported_types,
                self.transparent_wrappers,
                &source_crate_ident,
                assertion_type_pairs,
                &mut type_changed,
            );

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
}

/// Generate allowed prefixes that include standard prelude types and modules
///
/// Creates a list of syn::Path values representing standard library prefixes that are
/// considered safe for FFI use. This includes core library modules, standard collections,
/// primitive types, and common external crates like libc.
///
/// Returns a vector of parsed paths that can be used for type validation.
pub(crate) fn generate_standard_allowed_prefixes() -> Vec<syn::Path> {
    let prefix_strings = vec![
        // Core standard library modules
        "std",
        "core",
        "alloc",
        // Standard prelude types (these are implicitly imported)
        // Note: These are typically available without prefix, but we include them for completeness
        "Option",
        "Result",
        "Some",
        "None",
        "Ok",
        "Err",
        "Vec",
        "String",
        "Box",
        "Rc",
        "Arc",
        "Cell",
        "RefCell",
        "Mutex",
        "RwLock",
        "HashMap",
        "HashSet",
        "BTreeMap",
        "BTreeSet",
        // Standard collections
        "std::collections",
        "std::vec",
        "std::string",
        "std::boxed",
        "std::rc",
        "std::sync",
        "std::cell",
        // Core types and modules
        "core::option",
        "core::result",
        "core::mem",
        "core::ptr",
        "core::slice",
        "core::str",
        "core::fmt",
        "core::convert",
        "core::ops",
        "core::cmp",
        "core::clone",
        "core::marker",
        // Common external crates often used in FFI
        "libc",
        "c_char",
        "c_int",
        "c_uint",
        "c_long",
        "c_ulong",
        "c_void",
        // Standard primitive types (though these don't need prefixes)
        "bool",
        "char",
        "i8",
        "i16",
        "i32",
        "i64",
        "i128",
        "isize",
        "u8",
        "u16",
        "u32",
        "u64",
        "u128",
        "usize",
        "f32",
        "f64",
        "str",
    ];

    prefix_strings
        .into_iter()
        .filter_map(|s| syn::parse_str(s).ok())
        .collect()
}

/// Check if a type contains any of the exported types
///
/// Recursively searches through a type and its generic arguments to determine
/// if it contains any types that are in the exported types set. This is used
/// to decide whether type assertions are needed.
#[roxygen]
pub(crate) fn contains_exported_type(
    /// The type to check for exported type usage
    ty: &syn::Type,
    /// Set of exported type names to search for
    exported_types: &HashSet<String>,
) -> bool {
    match ty {
        syn::Type::Path(type_path) => {
            // Check if the type itself is defined
            if let Some(segment) = type_path.path.segments.last() {
                let type_name = segment.ident.to_string();
                if exported_types.contains(&type_name) {
                    return true;
                }

                // Check generic arguments recursively
                if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                    for arg in &args.args {
                        if let syn::GenericArgument::Type(inner_ty) = arg {
                            if contains_exported_type(inner_ty, exported_types) {
                                return true;
                            }
                        }
                    }
                }
            }
            false
        }
        syn::Type::Reference(type_ref) => {
            // Check the referenced type
            contains_exported_type(&type_ref.elem, exported_types)
        }
        syn::Type::Ptr(type_ptr) => {
            // Check the pointed-to type
            contains_exported_type(&type_ptr.elem, exported_types)
        }
        syn::Type::Slice(type_slice) => {
            // Check the slice element type
            contains_exported_type(&type_slice.elem, exported_types)
        }
        syn::Type::Array(type_array) => {
            // Check the array element type
            contains_exported_type(&type_array.elem, exported_types)
        }
        syn::Type::Tuple(type_tuple) => {
            // Check all tuple element types
            type_tuple
                .elems
                .iter()
                .any(|elem_ty| contains_exported_type(elem_ty, exported_types))
        }
        _ => false,
    }
}

/// Validate if a type path is allowed for FFI use
fn validate_type_path(type_path: &syn::TypePath, allowed_prefixes: &[syn::Path]) -> bool {
    // Check if the path is absolute (starts with ::)
    if type_path.path.leading_colon.is_some() {
        return true;
    }

    // Check if the path starts with any of the allowed prefixes
    for allowed_prefix in allowed_prefixes {
        if path_starts_with(&type_path.path, allowed_prefix) {
            return true;
        }
    }

    false
}

/// Check if a path starts with a given prefix path
fn path_starts_with(path: &syn::Path, prefix: &syn::Path) -> bool {
    if prefix.segments.len() > path.segments.len() {
        return false;
    }

    for (path_segment, prefix_segment) in path.segments.iter().zip(prefix.segments.iter()) {
        if path_segment.ident != prefix_segment.ident {
            return false;
        }
    }

    true
}

/// Validate generic arguments recursively for FFI compatibility
///
/// Checks all type arguments within angle brackets (e.g., `Vec<T>`, `HashMap<K, V>`)
/// to ensure they are valid for FFI use.
#[roxygen]
fn validate_generic_arguments(
    /// The generic arguments to validate
    args: &syn::AngleBracketedGenericArguments,
    /// Set of exported type names that are considered valid
    exported_types: &HashSet<String>,
    /// List of allowed path prefixes for type validation
    allowed_prefixes: &[syn::Path],
    /// Context string for error reporting
    context: &str,
) -> Result<(), String> {
    for arg in &args.args {
        if let syn::GenericArgument::Type(inner_ty) = arg {
            validate_type_for_ffi(
                inner_ty,
                exported_types,
                allowed_prefixes,
                &format!("{context} (generic argument)"),
            )?;
        }
    }
    Ok(())
}

/// Validate that a type is suitable for FFI use
///
/// This function checks if a type can be safely used in FFI by verifying it's either:
/// - An absolute path (starting with `::`)
/// - A path starting with an allowed prefix
/// - A type defined in the exported types set
/// - A supported container type (reference, pointer, slice, array, tuple) with valid element types
#[roxygen]
pub(crate) fn validate_type_for_ffi(
    /// The type to validate for FFI compatibility
    ty: &syn::Type,
    /// Set of exported type names that are considered valid
    exported_types: &HashSet<String>,
    /// List of allowed path prefixes (e.g., std::, core::, etc.)
    allowed_prefixes: &[syn::Path],
    /// Context string for error reporting (e.g., "parameter 1 of function 'foo'")
    context: &str,
) -> Result<(), String> {
    match ty {
        syn::Type::Path(type_path) => {
            // Validate the type path (includes absolute paths, allowed prefixes, and exported types)
            if validate_type_path(type_path, allowed_prefixes) {
                if let Some(segment) = type_path.path.segments.last() {
                    if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                        validate_generic_arguments(
                            args,
                            exported_types,
                            allowed_prefixes,
                            context,
                        )?;
                    }
                }
                return Ok(());
            }

            // Check if it's a single identifier that's an exported type
            if type_path.path.segments.len() == 1 {
                if let Some(segment) = type_path.path.segments.first() {
                    let type_name = segment.ident.to_string();
                    if exported_types.contains(&type_name) {
                        if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                            validate_generic_arguments(
                                args,
                                exported_types,
                                allowed_prefixes,
                                context,
                            )?;
                        }
                        return Ok(());
                    }
                }
            }

            // Invalid type path
            Err(format!(
                "Type '{}' in {} is not valid for FFI: must be either absolute (starting with '::'), start with an allowed prefix, or be defined in exported types",
                quote::quote! { #ty },
                context
            ))
        }
        syn::Type::Reference(type_ref) => validate_type_for_ffi(
            &type_ref.elem,
            exported_types,
            allowed_prefixes,
            &format!("{context} (reference)"),
        ),
        syn::Type::Ptr(type_ptr) => validate_type_for_ffi(
            &type_ptr.elem,
            exported_types,
            allowed_prefixes,
            &format!("{context} (pointer)"),
        ),
        syn::Type::Slice(type_slice) => validate_type_for_ffi(
            &type_slice.elem,
            exported_types,
            allowed_prefixes,
            &format!("{context} (slice element)"),
        ),
        syn::Type::Array(type_array) => validate_type_for_ffi(
            &type_array.elem,
            exported_types,
            allowed_prefixes,
            &format!("{context} (array element)"),
        ),
        syn::Type::Tuple(type_tuple) => {
            for (i, elem_ty) in type_tuple.elems.iter().enumerate() {
                validate_type_for_ffi(
                    elem_ty,
                    exported_types,
                    allowed_prefixes,
                    &format!("{context} (tuple element {i})"),
                )?;
            }
            Ok(())
        }
        _ => Err(format!(
            "Unsupported type '{}' in {}: only path types, references, pointers, slices, arrays, and tuples are supported for FFI",
            quote::quote! { #ty },
            context
        )),
    }
}

/// Convert a remote type to its local equivalent and collect assertion pairs
///
/// This function:
/// - Strips transparent wrappers from the type
/// - Converts references to pointers for FFI compatibility
/// - Collects type assertion pairs when conversion is needed
/// - Sets the `was_converted` flag to indicate if the type was modified
#[roxygen]
pub(crate) fn convert_to_local_type(
    /// The original type to convert
    original_type: &syn::Type,
    /// Set of exported type names
    exported_types: &HashSet<String>,
    /// List of transparent wrapper paths to strip
    transparent_wrappers: &[syn::Path],
    /// Source crate identifier for prefixing
    source_crate_ident: &syn::Ident,
    /// Mutable set to collect assertion pairs
    assertion_type_pairs: &mut HashSet<(String, String)>,
    /// Mutable boolean flag set to true if type was changed and needs transmute
    type_changed: &mut bool,
) -> syn::Type {
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
        strip_transparent_wrappers(core_type, transparent_wrappers, &mut has_wrapper);

    // Check if we should generate an assertion for this type
    *type_changed = has_wrapper || contains_exported_type(&local_core_type, exported_types);

    if *type_changed {
        // Create the original core type with proper crate prefixing
        let prefixed_original_core =
            prefix_exported_types_in_type(core_type, source_crate_ident, exported_types);

        // Store the assertion pair
        let local_core_str = quote::quote! { #local_core_type }.to_string();
        let prefixed_original_core_str = quote::quote! { #prefixed_original_core }.to_string();
        assertion_type_pairs.insert((local_core_str, prefixed_original_core_str));
    }

    // Build the final type based on whether the original was a reference
    if is_reference {
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
        convert_reference_to_pointer(&local_ref)
    } else if *type_changed {
        // Non-reference type that needed conversion
        local_core_type
    } else {
        // No conversion needed, return original type
        original_type.clone()
    }
}

/// Convert reference types to pointer types for FFI compatibility
pub(crate) fn convert_reference_to_pointer(ty: &syn::Type) -> syn::Type {
    match ty {
        syn::Type::Reference(type_ref) => {
            // Convert &T to *const T and &mut T to *mut T
            if type_ref.mutability.is_some() {
                syn::Type::Ptr(syn::TypePtr {
                    star_token: syn::token::Star::default(),
                    const_token: None,
                    mutability: Some(syn::token::Mut::default()),
                    elem: type_ref.elem.clone(),
                })
            } else {
                syn::Type::Ptr(syn::TypePtr {
                    star_token: syn::token::Star::default(),
                    const_token: Some(syn::token::Const::default()),
                    mutability: None,
                    elem: type_ref.elem.clone(),
                })
            }
        }
        _ => {
            // For non-reference types, return as-is
            ty.clone()
        }
    }
}

/// Validate function parameters for FFI compatibility
///
/// Checks each typed parameter in a function signature to ensure it can be safely
/// used in FFI contexts according to the validation rules.
#[roxygen]
fn validate_function_parameters(
    /// The function parameters to validate
    inputs: &syn::punctuated::Punctuated<syn::FnArg, syn::token::Comma>,
    /// The function name for error reporting
    function_name: &syn::Ident,
    /// Set of exported type names that are considered valid
    exported_types: &HashSet<String>,
    /// List of allowed path prefixes for type validation
    allowed_prefixes: &[syn::Path],
    /// Source location information for error reporting
    source_location: &crate::SourceLocation,
) -> Result<(), String> {
    for (i, input) in inputs.iter().enumerate() {
        if let syn::FnArg::Typed(pat_type) = input {
            validate_type_for_ffi(
                &pat_type.ty,
                exported_types,
                allowed_prefixes,
                &format!("parameter {} of function '{}'", i + 1, function_name),
            )
            .map_err(|e| format!("Invalid FFI function parameter: {} (at {}:{}:{})", e, source_location.file, source_location.line, source_location.column))?;
        }
    }
    Ok(())
}

/// Process code content to handle feature flags according to builder configuration
///
/// This function analyzes code for `#[cfg(feature="...")]` attributes using syn syntax parsing and:
/// - Removes code blocks guarded by disabled features
/// - Removes cfg attributes for enabled features (keeping the code)
/// - Replaces feature names according to the mapping (keeping the cfg attribute)
#[roxygen]
pub(crate) fn process_features(
    /// The parsed file to process for feature flags
    mut file: syn::File,
    /// Set of feature names that should cause code removal
    disabled_features: &HashSet<String>,
    /// Set of feature names that should have their cfg attributes removed
    enabled_features: &HashSet<String>,
    /// Mapping from old feature names to new feature names
    feature_mappings: &HashMap<String, String>,
) -> syn::File {
    // Process items in the file
    file.items.retain_mut(|item| {
        process_item_features(item, disabled_features, enabled_features, feature_mappings)
    });

    file
}

/// Process a single item (struct, enum, function, etc.) for feature flags
fn process_item_features(
    item: &mut syn::Item,
    disabled_features: &HashSet<String>,
    enabled_features: &HashSet<String>,
    feature_mappings: &HashMap<String, String>,
) -> bool {
    let attrs = match item {
        syn::Item::Fn(f) => &mut f.attrs,
        syn::Item::Struct(s) => {
            // Process struct fields recursively
            process_struct_fields(&mut s.fields, disabled_features, enabled_features, feature_mappings);
            &mut s.attrs
        },
        syn::Item::Enum(e) => {
            // Process enum variants recursively
            process_enum_variants(&mut e.variants, disabled_features, enabled_features, feature_mappings);
            &mut e.attrs
        },
        syn::Item::Union(u) => {
            // Process union fields recursively
            process_union_fields(&mut u.fields, disabled_features, enabled_features, feature_mappings);
            &mut u.attrs
        },
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
    process_attributes(attrs, disabled_features, enabled_features, feature_mappings)
}

/// Process struct fields for feature flags
fn process_struct_fields(
    fields: &mut syn::Fields,
    disabled_features: &HashSet<String>,
    enabled_features: &HashSet<String>,
    feature_mappings: &HashMap<String, String>,
) {
    match fields {
        syn::Fields::Named(fields_named) => {
            // Manual filtering since Punctuated doesn't have retain_mut
            let mut new_fields = syn::punctuated::Punctuated::new();
            for field in fields_named.named.pairs() {
                let mut field = field.into_value().clone();
                if process_attributes(&mut field.attrs, disabled_features, enabled_features, feature_mappings) {
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
                if process_attributes(&mut field.attrs, disabled_features, enabled_features, feature_mappings) {
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
) {
    // Manual filtering since Punctuated doesn't have retain_mut
    let mut new_variants = syn::punctuated::Punctuated::new();
    for variant_pair in variants.pairs() {
        let mut variant = variant_pair.into_value().clone();
        // Process variant attributes
        let keep_variant = process_attributes(&mut variant.attrs, disabled_features, enabled_features, feature_mappings);
        
        if keep_variant {
            // Process variant fields if it's kept
            process_struct_fields(&mut variant.fields, disabled_features, enabled_features, feature_mappings);
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
) {
    // Manual filtering since Punctuated doesn't have retain_mut
    let mut new_fields = syn::punctuated::Punctuated::new();
    for field_pair in fields.named.pairs() {
        let mut field = field_pair.into_value().clone();
        if process_attributes(&mut field.attrs, disabled_features, enabled_features, feature_mappings) {
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
) -> bool {
    let mut keep_item = true;
    let mut remove_attrs = Vec::new();

    for (i, attr) in attrs.iter_mut().enumerate() {
        // Check if this is a cfg attribute
        if attr.path().is_ident("cfg") {
            // Parse the meta to extract feature information
            if let syn::Meta::List(meta_list) = &attr.meta {
                if let Ok(cfg_expr) = syn::parse2::<CfgExpr>(meta_list.tokens.clone()) {
                    if let Some(feature_name) = extract_feature_from_cfg(&cfg_expr) {
                        // Check if feature should be disabled
                        if disabled_features.contains(&feature_name) {
                            keep_item = false;
                            break;
                        }

                        // Check if feature should be enabled (remove cfg)
                        if enabled_features.contains(&feature_name) {
                            remove_attrs.push(i);
                            break;
                        }

                        // Check if feature should be mapped
                        if let Some(new_feature) = feature_mappings.get(&feature_name) {
                            // Update the attribute with the new feature name
                            let new_meta = syn::parse_quote! {
                                cfg(feature = #new_feature)
                            };
                            attr.meta = new_meta;
                            break;
                        }
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

/// Simple cfg expression to handle basic feature checks
#[derive(Debug, Clone)]
enum CfgExpr {
    Feature(String),
    Other,
}

impl syn::parse::Parse for CfgExpr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        // Parse the entire content as tokens and look for feature patterns
        let tokens = input.parse::<proc_macro2::TokenStream>()?;
        let token_string = tokens.to_string();

        // Use regex to extract feature name from the token stream
        use regex::Regex;
        let feature_regex = Regex::new(r#"feature\s*=\s*"([^"]+)""#).unwrap();

        if let Some(captures) = feature_regex.captures(&token_string) {
            let feature_name = captures[1].to_string();
            Ok(CfgExpr::Feature(feature_name))
        } else {
            Ok(CfgExpr::Other)
        }
    }
}

/// Extract feature name from a cfg expression
fn extract_feature_from_cfg(cfg_expr: &CfgExpr) -> Option<String> {
    match cfg_expr {
        CfgExpr::Feature(name) => Some(name.clone()),
        CfgExpr::Other => None,
    }
}

/// Check if two syn::Path values are equal
fn paths_equal(path1: &syn::Path, path2: &syn::Path) -> bool {
    // Compare leading colons
    if path1.leading_colon.is_some() != path2.leading_colon.is_some() {
        return false;
    }

    // Compare segments
    if path1.segments.len() != path2.segments.len() {
        return false;
    }

    for (seg1, seg2) in path1.segments.iter().zip(path2.segments.iter()) {
        if seg1.ident != seg2.ident {
            return false;
        }
        // For transparent wrapper detection, we only care about the path name,
        // not the generic arguments
    }

    true
}

/// Generate compile-time assertions for type pairs
///
/// Creates size and alignment assertions to ensure that stripped types (used in FFI stubs)
/// are compatible with their original types (from the source crate). This provides
/// compile-time safety for type transmutations performed during FFI calls.
#[roxygen]
pub(crate) fn generate_type_assertions(
    /// Set of (local_type, source_type) string pairs to create assertions for
    assertion_type_pairs: &HashSet<(String, String)>,
) -> Vec<syn::Item> {
    let mut assertions = Vec::new();

    for (stripped_type_str, source_type_str) in assertion_type_pairs {
        // Parse the type strings back into syn::Type for proper code generation
        if let (Ok(stripped_type), Ok(source_type)) = (
            syn::parse_str::<syn::Type>(stripped_type_str),
            syn::parse_str::<syn::Type>(source_type_str),
        ) {
            // Generate size assertion: stripped type (stub parameter) vs source crate type (original)
            let size_assertion: syn::Item = syn::parse_quote! {
                const _: () = assert!(
                    std::mem::size_of::<#stripped_type>() == std::mem::size_of::<#source_type>(),
                    "Size mismatch between stub parameter type and source crate type"
                );
            };
            assertions.push(size_assertion);

            // Generate alignment assertion: stripped type (stub parameter) vs source crate type (original)
            let align_assertion: syn::Item = syn::parse_quote! {
                const _: () = assert!(
                    std::mem::align_of::<#stripped_type>() == std::mem::align_of::<#source_type>(),
                    "Alignment mismatch between stub parameter type and source crate type"
                );
            };
            assertions.push(align_assertion);
        }
    }

    assertions
}

/// Strip transparent wrappers from a type and track if any were removed
///
/// Recursively removes transparent wrapper types (like `MaybeUninit<T>`) from a type,
/// returning the inner type. Sets the `has_wrapper` flag to indicate if any wrappers
/// were found and stripped.
#[roxygen]
pub(crate) fn strip_transparent_wrappers(
    /// The type to strip wrappers from
    ty: &syn::Type,
    /// List of transparent wrapper paths to recognize and strip
    transparent_wrappers: &[syn::Path],
    /// Flag set to true if any wrappers were found and stripped
    has_wrapper: &mut bool,
) -> syn::Type {
    match ty {
        syn::Type::Path(type_path) => {
            // Check if this type path matches any transparent wrapper
            for wrapper in transparent_wrappers {
                if paths_equal(&type_path.path, wrapper) {
                    *has_wrapper = true;
                    // Extract the first generic argument if present
                    if let Some(last_segment) = type_path.path.segments.last() {
                        if let syn::PathArguments::AngleBracketed(args) = &last_segment.arguments {
                            if let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first() {
                                return strip_transparent_wrappers(
                                    inner_ty,
                                    transparent_wrappers,
                                    has_wrapper,
                                );
                            }
                        }
                    }
                }
            }
            // No wrapper found, return as-is
            ty.clone()
        }
        _ => ty.clone(),
    }
}

/// Recursively prefix exported types in a type with the source crate name
fn prefix_exported_types_in_type(
    ty: &syn::Type,
    source_crate_ident: &syn::Ident,
    exported_types: &HashSet<String>,
) -> syn::Type {
    match ty {
        syn::Type::Path(type_path) => {
            if let Some(segment) = type_path.path.segments.last() {
                let type_name = segment.ident.to_string();

                // Only prefix if this is an exported type
                if exported_types.contains(&type_name) && type_path.path.segments.len() == 1 {
                    return syn::parse_quote! { #source_crate_ident::#type_path };
                }

                // Handle generic arguments recursively
                if let syn::PathArguments::AngleBracketed(_args) = &segment.arguments {
                    let mut new_path = type_path.path.clone();
                    if let Some(last_segment) = new_path.segments.last_mut() {
                        if let syn::PathArguments::AngleBracketed(ref mut args) =
                            last_segment.arguments
                        {
                            for arg in &mut args.args {
                                if let syn::GenericArgument::Type(inner_ty) = arg {
                                    *inner_ty = prefix_exported_types_in_type(
                                        inner_ty,
                                        source_crate_ident,
                                        exported_types,
                                    );
                                }
                            }
                        }
                    }
                    return syn::Type::Path(syn::TypePath {
                        qself: type_path.qself.clone(),
                        path: new_path,
                    });
                }
            }
            ty.clone()
        }
        syn::Type::Reference(type_ref) => syn::Type::Reference(syn::TypeReference {
            and_token: type_ref.and_token,
            lifetime: type_ref.lifetime.clone(),
            mutability: type_ref.mutability,
            elem: Box::new(prefix_exported_types_in_type(
                &type_ref.elem,
                source_crate_ident,
                exported_types,
            )),
        }),
        syn::Type::Ptr(type_ptr) => syn::Type::Ptr(syn::TypePtr {
            star_token: type_ptr.star_token,
            const_token: type_ptr.const_token,
            mutability: type_ptr.mutability,
            elem: Box::new(prefix_exported_types_in_type(
                &type_ptr.elem,
                source_crate_ident,
                exported_types,
            )),
        }),
        _ => ty.clone(),
    }
}

#[cfg(test)]
mod tests;


