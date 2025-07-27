//! Type replacement utilities for converting types from their original form to FFI-compatible form.
//!
//! This module contains the logic for:
//! - Converting types using the same logic as FFI stub generation
//! - Handling transparent wrapper stripping
//! - Processing exported types with proper crate prefixing
//! - Generating type assertion pairs for compile-time validation

use roxygen::roxygen;
use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
};

use crate::SourceLocation;

/// Configuration parameters for parsing records
pub(crate) struct ParseConfig<'a> {
    pub crate_name: &'a str,
    pub exported_types: &'a HashSet<String>,
    pub allowed_prefixes: &'a [syn::Path],
    pub strip_prefixes: &'a [syn::Path],
    pub transparent_wrappers: &'a [syn::Path],
    pub edition: &'a str,
}

impl<'a> ParseConfig<'a> {
    pub fn crate_ident(&self) -> syn::Ident {
        // Convert crate name to identifier (replace dashes with underscores)
        let source_crate_name = self.crate_name.replace('-', "_");
        syn::Ident::new(&source_crate_name, proc_macro2::Span::call_site())
    }
}

/// Represents a type assertion pair for compile-time validation
///
/// This structure holds the local (stub) type and the corresponding source crate type
/// to ensure they are compatible for transmutation during FFI calls.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TypeTransmutePair {
    /// The local type used in the stub (e.g., stripped of transparent wrappers)
    pub local_type: String,
    /// The original type from the source crate (with proper crate prefixing)
    pub origin_type: String,
}

impl TypeTransmutePair {
    /// Create a new type assertion pair
    pub fn new(local_type: String, origin_type: String) -> Self {
        Self {
            local_type,
            origin_type,
        }
    }
}

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

/// Convert a remote type to its local equivalent, validate FFI compatibility, and collect assertion pairs
/// Returns true if transmute is needed, false otherwise
///
/// This method:
/// - Validates that the type is suitable for FFI use
/// - Strips transparent wrappers and prefixes to be stripped from the type
/// - Converts references to pointers for FFI compatibility
/// - Collects type assertion pairs when conversion is needed
/// - Sets the `was_converted` flag to indicate if the type was modified
#[roxygen]
pub(crate) fn replace_types_in_type(
    /// Destination type for the converted type
    ty: &mut syn::Type,
    /// Configuration containing parsing and validation options
    config: &ParseConfig,
    /// Mutable set to collect assertion pairs
    assertion_type_pairs: &mut HashMap<TypeTransmutePair, SourceLocation>,
    /// Source location for error reporting
    source_location: &SourceLocation,
) -> Result<bool, String> {
    // Strip prefixes and transparent wrappers from the type
    let mut stripped = false;
    let local_type = strip_type(
        ty,
        config.transparent_wrappers,
        config.strip_prefixes,
        &mut stripped,
        source_location,
    );

    // Validate the type for FFI compatibility and strip * and & references
    let mut is_exported_type = false;
    let (local_core_type, stripped_types) = strip_references_and_pointers(local_type);
    validate_core_type_for_ffi(
        &local_core_type,
        config.exported_types,
        config.allowed_prefixes,
        &mut is_exported_type,
    )?;

    // Build the final type and determine if conversion is needed
    let references_replaced = !stripped_types.is_empty();
    let core_type_changed = stripped || is_exported_type;
    let conversion_needed = references_replaced || core_type_changed;

    if conversion_needed {
        let final_type = if references_replaced {
            restore_stripped_references_as_pointers(local_core_type.clone(), stripped_types)?
        } else {
            local_core_type.clone()
        };

        // Generate assertion pair by stripping both types to same level
        if core_type_changed {
            let prefixed_original_type =
                prefix_exported_types_in_type(ty, &config.crate_ident(), config.exported_types);

            // Strip both types to the same level until first path type
            let (local_stripped, original_stripped) =
                strip_to_same_level(final_type.clone(), prefixed_original_type);

            let local_str = quote::quote! { #local_stripped }.to_string();
            let original_str = quote::quote! { #original_stripped }.to_string();
            if let std::collections::hash_map::Entry::Vacant(e) =
                assertion_type_pairs.entry(TypeTransmutePair::new(local_str, original_str))
            {
                e.insert(source_location.clone());
            }
        }

        *ty = final_type;
        Ok(true)
    } else {
        // No conversion needed, keep original type
        Ok(false)
    }
}

/// Replace types in a single item recursively
pub(crate) fn replace_types_in_item(
    item: &mut syn::Item,
    config: &ParseConfig,
    type_replacements: &mut HashMap<TypeTransmutePair, SourceLocation>,
    source_location: &SourceLocation,
) -> Result<bool, String> {
    match item {
        syn::Item::Fn(item_fn) => {
            let (_, sig_changed) = replace_types_in_signature(
                &mut item_fn.sig,
                config,
                type_replacements,
                source_location,
            )?;
            let block_changed = replace_types_in_block(
                &mut item_fn.block,
                config,
                type_replacements,
                source_location,
            )?;
            Ok(sig_changed || block_changed)
        }
        syn::Item::Struct(item_struct) => replace_types_in_fields(
            &mut item_struct.fields,
            config,
            type_replacements,
            source_location,
        ),
        syn::Item::Enum(item_enum) => {
            let mut any_changed = false;
            for variant in &mut item_enum.variants {
                any_changed |= replace_types_in_fields(
                    &mut variant.fields,
                    config,
                    type_replacements,
                    source_location,
                )?;
            }
            Ok(any_changed)
        }
        syn::Item::Union(item_union) => {
            let mut fields = syn::Fields::Named(item_union.fields.clone());
            let changed =
                replace_types_in_fields(&mut fields, config, type_replacements, source_location)?;
            if let syn::Fields::Named(fields_named) = fields {
                item_union.fields = fields_named;
            }
            Ok(changed)
        }
        syn::Item::Type(item_type) => replace_types_in_type(
            &mut item_type.ty,
            config,
            type_replacements,
            source_location,
        ),
        syn::Item::Const(item_const) => replace_types_in_type(
            &mut item_const.ty,
            config,
            type_replacements,
            source_location,
        ),
        syn::Item::Static(item_static) => replace_types_in_type(
            &mut item_static.ty,
            config,
            type_replacements,
            source_location,
        ),
        _ => Ok(false),
    }
}

/// Replace types in function signature
pub(crate) fn replace_types_in_signature(
    sig: &mut syn::Signature,
    config: &ParseConfig,
    assertion_type_pairs: &mut HashMap<TypeTransmutePair, SourceLocation>,
    source_location: &SourceLocation,
) -> Result<(Vec<bool>, bool), String> {
    // Replace parameter types
    let mut parameters_changed = Vec::new();
    for input in sig.inputs.iter_mut() {
        if let syn::FnArg::Typed(pat_type) = input {
            parameters_changed.push(replace_types_in_type(
                &mut pat_type.ty,
                config,
                assertion_type_pairs,
                source_location,
            )?);
        } else {
            return Err("self parameters are not supported in FFI stubs".into());
        }
    }

    // Replace return type
    let mut return_type_changed = false;
    if let syn::ReturnType::Type(_, return_type) = &mut sig.output {
        return_type_changed =
            replace_types_in_type(return_type, config, assertion_type_pairs, source_location)?;
    };
    Ok((parameters_changed, return_type_changed))
}

/// Replace types in a function block
fn replace_types_in_block(
    _block: &mut syn::Block,
    _config: &ParseConfig,
    _assertion_type_pairs: &mut HashMap<TypeTransmutePair, SourceLocation>,
    _source_location: &SourceLocation,
) -> Result<bool, String> {
    // For now, we don't need to replace types in function bodies
    Ok(false)
}

/// Replace types in struct/enum fields
fn replace_types_in_fields(
    fields: &mut syn::Fields,
    config: &ParseConfig,
    assertion_type_pairs: &mut HashMap<TypeTransmutePair, SourceLocation>,
    source_location: &SourceLocation,
) -> Result<bool, String> {
    let mut any_changed = false;
    match fields {
        syn::Fields::Named(fields_named) => {
            for field in fields_named.named.iter_mut() {
                any_changed |= replace_types_in_type(
                    &mut field.ty,
                    config,
                    assertion_type_pairs,
                    source_location,
                )?;
            }
        }
        syn::Fields::Unnamed(fields_unnamed) => {
            for field in fields_unnamed.unnamed.iter_mut() {
                any_changed |= replace_types_in_type(
                    &mut field.ty,
                    config,
                    assertion_type_pairs,
                    source_location,
                )?;
            }
        }
        syn::Fields::Unit => {
            // No fields to process
        }
    }
    Ok(any_changed)
}

/// Strip transparent wrappers from a type and track if any were removed
///
/// Recursively removes transparent wrapper types (like `MaybeUninit<T>`) from a type,
/// returning the inner type. Sets the `has_wrapper` flag to indicate if any wrappers
/// were found and stripped.
#[roxygen]
fn strip_type(
    /// The type to strip wrappers from
    ty: &syn::Type,
    /// List of wrapper paths to recognize and strip
    wrappers: &[syn::Path],
    /// List of type prefixes to recognize and strip
    prefixes: &[syn::Path],
    /// Flag set to true if the type was changed
    stripped: &mut bool,
    /// Source location for error reporting
    source_location: &SourceLocation,
) -> syn::Type {
    match ty {
        syn::Type::Path(type_path) => {
            // Associated types are not supported, report error
            if type_path.qself.is_some() {
                panic!("Associated types are not supported in FFI stubs: {source_location}");
            }

            // Check if this type path starts with any prefix in prefixes
            let mut prefixes_iter = prefixes.iter();
            let path = loop {
                let Some(prefix) = prefixes_iter.next() else {
                    break Cow::Borrowed(&type_path.path);
                };
                if path_starts_with(&type_path.path, prefix) {
                    *stripped = true;
                    // Remove the prefix segments from the path
                    let iter = type_path.path.segments.iter().skip(prefix.segments.len());
                    break Cow::Owned(syn::Path {
                        leading_colon: type_path.path.leading_colon,
                        segments: iter.cloned().collect(),
                    });
                }
            };

            // Check if this type path matches any transparent wrapper
            for wrapper in wrappers {
                if paths_equal(path.as_ref(), wrapper) {
                    *stripped = true;
                    // Extract the first generic argument if present
                    if let Some(last_segment) = path.segments.last() {
                        if let syn::PathArguments::AngleBracketed(args) = &last_segment.arguments {
                            if let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first() {
                                return strip_type(
                                    inner_ty,
                                    wrappers,
                                    prefixes,
                                    stripped,
                                    source_location,
                                );
                            }
                        }
                    }
                }
            }

            syn::Type::Path(syn::TypePath {
                qself: type_path.qself.clone(),
                path: path.into_owned(),
            })
        }
        syn::Type::Reference(type_ref) => {
            // Recursively strip wrappers from the referenced type
            let stripped_elem = strip_type(
                &type_ref.elem,
                wrappers,
                prefixes,
                stripped,
                source_location,
            );
            syn::Type::Reference(syn::TypeReference {
                and_token: type_ref.and_token,
                lifetime: type_ref.lifetime.clone(),
                mutability: type_ref.mutability,
                elem: Box::new(stripped_elem),
            })
        }
        syn::Type::Ptr(type_ptr) => {
            // Recursively strip wrappers from the pointed-to type
            let stripped_elem = strip_type(
                &type_ptr.elem,
                wrappers,
                prefixes,
                stripped,
                source_location,
            );
            syn::Type::Ptr(syn::TypePtr {
                star_token: type_ptr.star_token,
                const_token: type_ptr.const_token,
                mutability: type_ptr.mutability,
                elem: Box::new(stripped_elem),
            })
        }
        syn::Type::Array(type_array) => {
            // Recursively strip wrappers from the array element type
            let stripped_elem = strip_type(
                &type_array.elem,
                wrappers,
                prefixes,
                stripped,
                source_location,
            );
            syn::Type::Array(syn::TypeArray {
                bracket_token: type_array.bracket_token,
                elem: Box::new(stripped_elem),
                semi_token: type_array.semi_token,
                len: type_array.len.clone(),
            })
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

/// Generate compile-time assertions for a single type transmute pair
///
/// Creates size and alignment assertions to ensure that stripped types (used in FFI stubs)
/// are compatible with their original types (from the source crate). Returns a pair of
/// syn::Item objects for size and alignment assertions.
#[roxygen]
pub(crate) fn generate_type_transmute_pair_assertions(
    /// Single TypeTransmutePair to create assertions for
    assertion_pair: &TypeTransmutePair,
) -> Option<(syn::Item, syn::Item)> {
    // Parse the type strings back into syn::Type for proper code generation
    if let (Ok(stripped_type), Ok(source_type)) = (
        syn::parse_str::<syn::Type>(&assertion_pair.local_type),
        syn::parse_str::<syn::Type>(&assertion_pair.origin_type),
    ) {
        // Generate size assertion: stripped type (stub parameter) vs source crate type (original)
        let size_assertion: syn::Item = syn::parse_quote! {
            const _: () = assert!(
                std::mem::size_of::<#stripped_type>() == std::mem::size_of::<#source_type>(),
                "Size mismatch between stub parameter type and source crate type"
            );
        };

        // Generate alignment assertion: stripped type (stub parameter) vs source crate type (original)
        let align_assertion: syn::Item = syn::parse_quote! {
            const _: () = assert!(
                std::mem::align_of::<#stripped_type>() == std::mem::align_of::<#source_type>(),
                "Alignment mismatch between stub parameter type and source crate type"
            );
        };

        Some((size_assertion, align_assertion))
    } else {
        None
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

/// Strip both types to the same level until first path type is reached
/// This ensures meaningful comparisons by stripping references/pointers equally
fn strip_to_same_level(
    mut local_type: syn::Type,
    mut original_type: syn::Type,
) -> (syn::Type, syn::Type) {
    loop {
        let local_is_path = matches!(local_type, syn::Type::Path(_));
        let original_is_path = matches!(original_type, syn::Type::Path(_));

        // Stop if either type is a path type
        if local_is_path || original_is_path {
            return (local_type, original_type);
        }

        // Strip one level from both types if possible
        let local_stripped = match &local_type {
            syn::Type::Reference(type_ref) => Some((*type_ref.elem).clone()),
            syn::Type::Ptr(type_ptr) => Some((*type_ptr.elem).clone()),
            _ => None,
        };

        let original_stripped = match &original_type {
            syn::Type::Reference(type_ref) => Some((*type_ref.elem).clone()),
            syn::Type::Ptr(type_ptr) => Some((*type_ptr.elem).clone()),
            _ => None,
        };

        // If both can be stripped, continue; otherwise stop
        match (local_stripped, original_stripped) {
            (Some(local), Some(original)) => {
                local_type = local;
                original_type = original;
            }
            _ => return (local_type, original_type),
        }
    }
}

/// removes references, pointers and arrays from a type, returns
/// the core type and a vector of stripped types, each with "!" (bang) type inside
fn strip_references_and_pointers(mut ty: syn::Type) -> (syn::Type, Vec<syn::Type>) {
    let mut stripped = vec![];
    loop {
        let mut elem = Box::new(syn::Type::Never(syn::TypeNever {
            bang_token: syn::token::Not::default(),
        }));
        if let syn::Type::Reference(type_ref) = &mut ty {
            std::mem::swap(&mut type_ref.elem, &mut elem);
        } else if let syn::Type::Ptr(type_ptr) = &mut ty {
            std::mem::swap(&mut type_ptr.elem, &mut elem);
        } else if let syn::Type::Array(type_array) = &mut ty {
            std::mem::swap(&mut type_array.elem, &mut elem);
        } else {
            // Not a reference, pointer, or array: return the core type
            return (ty, stripped);
        }
        stripped.push(ty);
        ty = *elem;
    }
}

/// convert references to pointers for FFI compatibility
fn restore_stripped_references_as_pointers(
    ty: syn::Type,
    stripped: Vec<syn::Type>,
) -> Result<syn::Type, String> {
    let mut result = ty;
    for stripped_type in stripped.into_iter().rev() {
        match stripped_type {
            syn::Type::Reference(syn::TypeReference { mutability, .. }) => {
                result = syn::Type::Ptr(syn::TypePtr {
                    star_token: syn::token::Star::default(),
                    const_token: mutability.is_none().then(syn::token::Const::default),
                    mutability: mutability.is_some().then(syn::token::Mut::default),
                    elem: Box::new(result),
                });
            }
            syn::Type::Array(type_array @ syn::TypeArray { .. }) => {
                result = syn::Type::Array(syn::TypeArray {
                    elem: Box::new(result),
                    ..type_array
                });
            }
            syn::Type::Ptr(type_ptr @ syn::TypePtr { .. }) => {
                result = syn::Type::Ptr(syn::TypePtr {
                    elem: Box::new(result),
                    ..type_ptr
                });
            }
            _ => {
                return Err(format!(
                    "Unsupported stripped type for FFI conversion: {}",
                    quote::quote! { #stripped_type }
                ));
            }
        }
    }
    Ok(result)
}

/// Checks if type is exported type or prefix matches allowed prefixes
///
/// This function checks if a type can be safely used in FFI by verifying it's either:
/// - An absolute path (starting with `::`)
/// - A path starting with an allowed prefix
/// - A type defined in the exported types set
#[roxygen]
fn validate_core_type_for_ffi(
    /// The type to validate for FFI compatibility
    ty: &syn::Type,
    /// Set of exported type names that are considered valid
    exported_types: &HashSet<String>,
    /// List of allowed path prefixes (e.g., std::, core::, etc.)
    allowed_prefixes: &[syn::Path],
    /// Flag indicating if the type is an exported type
    is_exported_type: &mut bool,
) -> Result<(), String> {
    if let syn::Type::Path(type_path) = ty {
        // Validate the type path (includes absolute paths, allowed prefixes, and exported types)
        if !validate_type_path(
            type_path,
            allowed_prefixes,
            exported_types,
            is_exported_type,
        ) {
            return Err(format!(
                "Type '{}' is not valid for FFI: must be either absolute (starting with '::'), start with an allowed prefix, or be defined in exported types",
                quote::quote! { #ty },
            ));
        }
        let Some(segment) = type_path.path.segments.last() else {
            return Err(format!(
                "Type '{}' is not valid for FFI: must have at least one segment",
                quote::quote! { #ty },
            ));
        };
        if let syn::PathArguments::AngleBracketed(_) = &segment.arguments {
            return Err(format!(
                "Type '{}' is not valid for FFI: generic arguments are not supported",
                quote::quote! { #ty },
            ));
        }
        Ok(())
    } else {
        Err(format!(
            "Unsupported type '{}': only path types are supported as core types for FFI",
            quote::quote! { #ty },
        ))
    }
}

/// Validate if a type path is allowed for FFI use
fn validate_type_path(
    type_path: &syn::TypePath,
    allowed_prefixes: &[syn::Path],
    exported_types: &HashSet<String>,
    is_exported_type: &mut bool,
) -> bool {
    *is_exported_type = false;
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

    // Check if it's a single identifier that's an exported type
    if type_path.path.segments.len() == 1 {
        if let Some(segment) = type_path.path.segments.first() {
            let type_name = segment.ident.to_string();
            if exported_types.contains(&type_name) {
                *is_exported_type = true;
                return true;
            }
        }
    }

    false
}

/// Check if a path starts with a given prefix path
fn path_starts_with(path: &syn::Path, prefix: &syn::Path) -> bool {
    if prefix.segments.len() > path.segments.len() {
        return false;
    }

    if prefix.leading_colon.is_some() != path.leading_colon.is_some() {
        return false;
    }

    for (path_segment, prefix_segment) in path.segments.iter().zip(prefix.segments.iter()) {
        if path_segment.ident != prefix_segment.ident {
            return false;
        }
    }

    true
}

/// Create a stub implementation for a function with transmutes applied
///
/// This function takes the original function signature, applies type replacements,
/// and creates a new function body that applies transmutes only to types that were
/// actually replaced. The decision to transmute is based on whether the original
/// parameter type appears as an origin_type in the type pairs.
///
/// The collected type replacement pairs for assertion generation are added to the provided set.
pub(crate) fn convert_to_stub(
    function: &mut syn::ItemFn,
    config: &ParseConfig,
    type_replacements: &mut HashMap<TypeTransmutePair, SourceLocation>,
    source_location: &SourceLocation,
) -> Result<(), String> {
    // Extract original types before transformation
    let mut original_param_types = Vec::new();
    for input in &function.sig.inputs {
        match input {
            syn::FnArg::Typed(pat_type) => {
                original_param_types.push((*pat_type.ty).clone());
            }
            syn::FnArg::Receiver(_) => {
                return Err(
                    "FFI functions cannot have receiver arguments (like 'self')".to_string()
                );
            }
        }
    }

    let original_return_type = match &function.sig.output {
        syn::ReturnType::Type(_, return_type) => Some((**return_type).clone()),
        syn::ReturnType::Default => None,
    };

    // Apply type replacements to the function signature
    let mut sig_type_replacements = type_replacements.clone();
    let (params_changed, return_changed) = replace_types_in_signature(
        &mut function.sig,
        config,
        &mut sig_type_replacements,
        source_location,
    )?;

    // Determine if we need unsafe block
    let need_unsafe_block = function.sig.unsafety.is_some()
        || params_changed.iter().any(|&changed| changed)
        || return_changed;

    // Check for unsupported parameter patterns first
    for input in &function.sig.inputs {
        if let syn::FnArg::Typed(pat_type) = input {
            if matches!(&*pat_type.pat, syn::Pat::Wild(_)) {
                return Err(
                    "Wildcard parameters ('_') are not supported in FFI functions".to_string(),
                );
            }
        }
    }

    // Build call arguments with conditional transmutes
    let call_args: Vec<_> = function
        .sig
        .inputs
        .iter()
        .enumerate()
        .filter_map(|(i, input)| {
            if let syn::FnArg::Typed(pat_type) = input {
                if let syn::Pat::Ident(pat_ident) = &*pat_type.pat {
                    let param_name = &pat_ident.ident;
                    let param_changed = params_changed.get(i).copied().unwrap_or(false);

                    let arg = if param_changed {
                        quote::quote! { std::mem::transmute(#param_name) }
                    } else {
                        quote::quote! { #param_name }
                    };
                    return Some(arg);
                }
            }
            None
        })
        .collect();

    // Determine if return type needs transmutation
    let has_return_type = !matches!(&function.sig.output, syn::ReturnType::Default);
    let return_needs_transmute = has_return_type && return_changed;

    // Generate function body
    let function_name = &function.sig.ident;
    let source_crate_ident = &config.crate_ident();

    // Check if the original return type was a reference that got converted to a pointer
    let is_converted_return_reference = if let Some(original_ret) = &original_return_type {
        matches!(original_ret, syn::Type::Reference(_))
            && matches!(&function.sig.output, syn::ReturnType::Type(_, ret_ty) if matches!(**ret_ty, syn::Type::Ptr(_)))
    } else {
        false
    };

    let function_body = match (
        has_return_type,
        return_needs_transmute || is_converted_return_reference,
    ) {
        (true, true) => quote::quote! {
            std::mem::transmute(#source_crate_ident::#function_name(#(#call_args),*))
        },
        (true, false) => quote::quote! { #source_crate_ident::#function_name(#(#call_args),*) },
        (false, _) => quote::quote! { #source_crate_ident::#function_name(#(#call_args),*) },
    };

    let function_body = if need_unsafe_block {
        quote::quote! { unsafe { #function_body } }
    } else {
        function_body
    };

    // Update function with new body and FFI attributes
    function.block = Box::new(syn::parse_quote! { { #function_body } });

    let no_mangle_attr = if config.edition == "2024" {
        syn::parse_quote! { #[unsafe(no_mangle)] }
    } else {
        syn::parse_quote! { #[no_mangle] }
    };

    function.attrs.insert(0, no_mangle_attr);
    function.sig.unsafety = Some(syn::Token![unsafe](proc_macro2::Span::call_site()));
    function.sig.abi = Some(syn::parse_quote! { extern "C" });
    function.vis = syn::parse_quote! { pub };

    // Add the type replacements to the global set for assertion generation
    type_replacements.extend(sig_type_replacements);

    Ok(())
}
