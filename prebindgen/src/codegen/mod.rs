//! Code generation utilities for transforming Rust function definitions into FFI stubs.
//!
//! This module contains all the logic for:
//! - Parsing and validating function signatures for FFI compatibility
//! - Generating `#[no_mangle] extern "C"` wrapper functions
//! - Handling type transformations and validations
//! - Creating appropriate parameter names and call arguments
//! - Processing feature flags (`#[cfg(feature="...")]`) in generated code

use roxygen::roxygen;
use std::collections::HashSet;

pub mod transform_function;
pub mod process_features;
pub mod replace_types;

// Re-export the main functions
pub use transform_function::transform_function_to_stub;
pub use process_features::process_features;
pub use replace_types::replace_types;

/// Generate allowed prefixes that include standard prelude types and modules
///
/// Creates a list of syn::Path values representing standard library prefixes that are
/// considered safe for FFI use. This includes core library modules, standard collections,
/// primitive types, and common external crates like libc.
///
/// Returns a vector of parsed paths that can be used for type validation.
pub fn generate_standard_allowed_prefixes() -> Vec<syn::Path> {
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
            validate_type_for_ffi_impl(
                inner_ty,
                exported_types,
                allowed_prefixes,
                &format!("{context} (generic argument)"),
            )?;
        }
    }
    Ok(())
}

/// Validate that a type is suitable for FFI use (internal implementation)
///
/// This function checks if a type can be safely used in FFI by verifying it's either:
/// - An absolute path (starting with `::`)
/// - A path starting with an allowed prefix
/// - A type defined in the exported types set
/// - A supported container type (reference, pointer, slice, array, tuple) with valid element types
#[roxygen]
pub(crate) fn validate_type_for_ffi_impl(
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
        syn::Type::Reference(type_ref) => validate_type_for_ffi_impl(
            &type_ref.elem,
            exported_types,
            allowed_prefixes,
            &format!("{context} (reference)"),
        ),
        syn::Type::Ptr(type_ptr) => validate_type_for_ffi_impl(
            &type_ptr.elem,
            exported_types,
            allowed_prefixes,
            &format!("{context} (pointer)"),
        ),
        syn::Type::Slice(type_slice) => validate_type_for_ffi_impl(
            &type_slice.elem,
            exported_types,
            allowed_prefixes,
            &format!("{context} (slice element)"),
        ),
        syn::Type::Array(type_array) => validate_type_for_ffi_impl(
            &type_array.elem,
            exported_types,
            allowed_prefixes,
            &format!("{context} (array element)"),
        ),
        syn::Type::Tuple(type_tuple) => {
            for (i, elem_ty) in type_tuple.elems.iter().enumerate() {
                validate_type_for_ffi_impl(
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
pub(crate) fn _generate_type_assertions(
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
pub(crate) fn prefix_exported_types_in_type(
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
