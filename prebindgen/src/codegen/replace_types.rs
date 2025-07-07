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
        let source_crate_ident =
            syn::Ident::new(&source_crate_name, proc_macro2::Span::call_site());

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
        validate_type_for_ffi_impl(
            original_type,
            self.exported_types,
            self.allowed_prefixes,
            context,
        )?;

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
            strip_transparent_wrappers(core_type, self.transparent_wrappers, &mut has_wrapper);

        // Check if we should generate an assertion for this type
        *type_changed =
            has_wrapper || contains_exported_type(&local_core_type, self.exported_types);

        if *type_changed {
            // Create the original core type with proper crate prefixing
            let prefixed_original_core = prefix_exported_types_in_type(
                core_type,
                &self.source_crate_ident,
                self.exported_types,
            );

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
            // Reference-to-pointer conversion is always done for FFI
            convert_reference_to_pointer(&local_ref)
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
fn replace_types_in_item(
    item: &mut syn::Item,
    replacer: &ReplaceTypes,
    assertion_type_pairs: &mut HashSet<(String, String)>,
) {
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
            replace_types_in_fields(
                &mut syn::Fields::Named(item_union.fields.clone()),
                replacer,
                assertion_type_pairs,
            );
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
fn replace_types_in_signature(
    sig: &mut syn::Signature,
    replacer: &ReplaceTypes,
    assertion_type_pairs: &mut HashSet<(String, String)>,
) {
    // Replace parameter types
    for (i, input) in sig.inputs.iter_mut().enumerate() {
        if let syn::FnArg::Typed(pat_type) = input {
            replace_types_in_type_with_context(
                &mut pat_type.ty,
                replacer,
                assertion_type_pairs,
                &format!("parameter {}", i + 1),
            );
        }
    }

    // Replace return type
    if let syn::ReturnType::Type(_, return_type) = &mut sig.output {
        replace_types_in_type_with_context(
            return_type,
            replacer,
            assertion_type_pairs,
            "return type",
        );
    }
}

/// Replace types in a function block
fn replace_types_in_block(
    block: &mut syn::Block,
    _replacer: &ReplaceTypes,
    _assertion_type_pairs: &mut HashSet<(String, String)>,
) {
    // For now, we don't need to replace types in function bodies
    // This can be extended if needed
    let _ = block;
}

/// Replace types in struct/enum fields
fn replace_types_in_fields(
    fields: &mut syn::Fields,
    replacer: &ReplaceTypes,
    assertion_type_pairs: &mut HashSet<(String, String)>,
) {
    match fields {
        syn::Fields::Named(fields_named) => {
            for (i, field) in fields_named.named.iter_mut().enumerate() {
                let context = if let Some(field_name) = &field.ident {
                    format!("field '{field_name}'")
                } else {
                    format!("field {i}")
                };
                replace_types_in_type_with_context(
                    &mut field.ty,
                    replacer,
                    assertion_type_pairs,
                    &context,
                );
            }
        }
        syn::Fields::Unnamed(fields_unnamed) => {
            for (i, field) in fields_unnamed.unnamed.iter_mut().enumerate() {
                replace_types_in_type_with_context(
                    &mut field.ty,
                    replacer,
                    assertion_type_pairs,
                    &format!("field {i}"),
                );
            }
        }
        syn::Fields::Unit => {
            // No fields to process
        }
    }
}

/// Replace a type based on the replacement logic with context
fn replace_types_in_type_with_context(
    ty: &mut syn::Type,
    replacer: &ReplaceTypes,
    assertion_type_pairs: &mut HashSet<(String, String)>,
    context: &str,
) {
    // Try to convert the type using the same logic as FFI stub generation
    let mut type_changed = false;
    match replacer.convert_to_local_type(ty, assertion_type_pairs, &mut type_changed, context) {
        Ok(converted_type) => {
            // Apply conversion if type was changed OR if it's a reference (which should always become a pointer for FFI)
            let is_reference = matches!(ty, syn::Type::Reference(_));
            if type_changed || is_reference {
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
fn replace_types_in_type(
    ty: &mut syn::Type,
    replacer: &ReplaceTypes,
    assertion_type_pairs: &mut HashSet<(String, String)>,
) {
    replace_types_in_type_with_context(ty, replacer, assertion_type_pairs, "type");
}

/// Convert reference types to pointer types for FFI compatibility
fn convert_reference_to_pointer(ty: &syn::Type) -> syn::Type {
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

/// Strip transparent wrappers from a type and track if any were removed
///
/// Recursively removes transparent wrapper types (like `MaybeUninit<T>`) from a type,
/// returning the inner type. Sets the `has_wrapper` flag to indicate if any wrappers
/// were found and stripped.
#[roxygen]
fn strip_transparent_wrappers(
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
fn _generate_type_assertions(
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

/// Validate that a type is suitable for FFI use (internal implementation)
///
/// This function checks if a type can be safely used in FFI by verifying it's either:
/// - An absolute path (starting with `::`)
/// - A path starting with an allowed prefix
/// - A type defined in the exported types set
/// - A supported container type (reference, pointer, slice, array, tuple) with valid element types
#[roxygen]
fn validate_type_for_ffi_impl(
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

/// Check if a type contains any of the exported types
///
/// Recursively searches through a type and its generic arguments to determine
/// if it contains any types that are in the exported types set. This is used
/// to decide whether type assertions are needed.
#[roxygen]
fn contains_exported_type(
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
