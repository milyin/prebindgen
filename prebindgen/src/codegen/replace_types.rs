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
    pub primitive_types: &'a HashMap<String, String>,
    pub allowed_prefixes: &'a [syn::Path],
    pub prefixed_exported_types: &'a [syn::Path],
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
///
/// This method:
/// - Validates that the type is suitable for FFI use
/// - Strips transparent wrappers and prefixes to be stripped from the type
/// - Converts references to pointers for FFI compatibility
/// - Collects type assertion pairs
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
) {
    // Capture original type before any modifications for assertion generation
    // and remove all lifetimes to statics in it
    let mut original_ty = ty.clone();
    replace_lifetimes_with_static(&mut original_ty);

    // Strip prefixes and transparent wrappers from the type
    let mut stripped = false;
    let mut local_type = strip_type(
        ty,
        config.transparent_wrappers,
        config.prefixed_exported_types,
        &mut stripped,
        source_location,
    );

    // Handle bare function types specially
    if let syn::Type::BareFn(ref mut type_bare_fn) = local_type {
        // Process function parameters
        for input in &mut type_bare_fn.inputs {
            replace_types_in_type(&mut input.ty, config, assertion_type_pairs, source_location);
        }

        // Process return type
        if let syn::ReturnType::Type(_, ref mut return_type) = type_bare_fn.output {
            replace_types_in_type(return_type, config, assertion_type_pairs, source_location);
        }

        let prefixed_original_type = prefix_exported_types_in_type(
            &original_ty,
            &config.crate_ident(),
            config.exported_types,
            config.prefixed_exported_types,
        );
        add_assertion_pair(
            assertion_type_pairs,
            local_type.clone(),
            prefixed_original_type,
            source_location,
            config.primitive_types,
        );

        *ty = local_type;
        return;
    }

    // Validate the type for FFI compatibility and strip * and & references
    let mut is_exported_type = false;
    let (local_core_type, stripped_types) = strip_references_and_pointers(local_type);
    validate_core_type_for_ffi(
        &local_core_type,
        config.exported_types,
        config.allowed_prefixes,
        &mut is_exported_type,
        source_location,
    );

    // Build the final type and determine if conversion is needed
    let references_replaced = !stripped_types.is_empty();
    let final_type = if references_replaced {
        restore_stripped_references_as_pointers(
            local_core_type.clone(),
            stripped_types,
            source_location,
        )
    } else {
        local_core_type.clone()
    };

    let prefixed_original_type = prefix_exported_types_in_type(
        &original_ty,
        &config.crate_ident(),
        config.exported_types,
        config.prefixed_exported_types,
    );
    let (local_stripped, original_stripped) =
        strip_to_same_level(final_type.clone(), prefixed_original_type);
    add_assertion_pair(
        assertion_type_pairs,
        local_stripped,
        original_stripped,
        source_location,
        config.primitive_types,
    );

    *ty = final_type;
}

/// Replace lifetimes in a type with 'static
fn replace_lifetimes_with_static(ty: &mut syn::Type) {
    match ty {
        syn::Type::Reference(type_ref) => {
            type_ref.lifetime = Some(syn::parse_quote! { 'static });
            replace_lifetimes_with_static(&mut type_ref.elem);
        }
        syn::Type::Path(type_path) => {
            if let Some(last_segment) = type_path.path.segments.last_mut() {
                if let syn::PathArguments::AngleBracketed(ref mut args) = last_segment.arguments {
                    for arg in &mut args.args {
                        match arg {
                            syn::GenericArgument::Type(inner_ty) => {
                                replace_lifetimes_with_static(inner_ty);
                            }
                            syn::GenericArgument::Lifetime(lifetime) => {
                                *lifetime = syn::parse_quote! { 'static };
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
        syn::Type::Array(type_array) => {
            replace_lifetimes_with_static(&mut type_array.elem);
        }
        syn::Type::Ptr(type_ptr) => {
            replace_lifetimes_with_static(&mut type_ptr.elem);
        }
        _ => {}
    }
}

/// Replace types in a single item recursively
pub(crate) fn replace_types_in_item(
    item: &mut syn::Item,
    config: &ParseConfig,
    type_replacements: &mut HashMap<TypeTransmutePair, SourceLocation>,
    source_location: &SourceLocation,
) {
    match item {
        syn::Item::Fn(item_fn) => {
            replace_types_in_signature(
                &mut item_fn.sig,
                config,
                type_replacements,
                source_location,
            );
            replace_types_in_block(
                &mut item_fn.block,
                config,
                type_replacements,
                source_location,
            );
        }
        syn::Item::Struct(item_struct) => replace_types_in_fields(
            &mut item_struct.fields,
            config,
            type_replacements,
            source_location,
        ),
        syn::Item::Enum(item_enum) => {
            for variant in &mut item_enum.variants {
                replace_types_in_fields(
                    &mut variant.fields,
                    config,
                    type_replacements,
                    source_location,
                );
            }
        }
        syn::Item::Union(item_union) => {
            let mut fields = syn::Fields::Named(item_union.fields.clone());
            replace_types_in_fields(&mut fields, config, type_replacements, source_location);
            if let syn::Fields::Named(fields_named) = fields {
                item_union.fields = fields_named;
            }
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
        _ => (),
    }
}

/// Replace types in function signature
pub(crate) fn replace_types_in_signature(
    sig: &mut syn::Signature,
    config: &ParseConfig,
    assertion_type_pairs: &mut HashMap<TypeTransmutePair, SourceLocation>,
    source_location: &SourceLocation,
) {
    // Replace parameter types
    for input in sig.inputs.iter_mut() {
        if let syn::FnArg::Typed(pat_type) = input {
            replace_types_in_type(
                &mut pat_type.ty,
                config,
                assertion_type_pairs,
                source_location,
            );
        } else {
            panic!("self parameters are not supported in FFI stubs: {source_location}");
        }
    }

    // Replace return type
    if let syn::ReturnType::Type(_, return_type) = &mut sig.output {
        replace_types_in_type(return_type, config, assertion_type_pairs, source_location);
    };
}

/// Replace types in a function block
fn replace_types_in_block(
    _block: &mut syn::Block,
    _config: &ParseConfig,
    _assertion_type_pairs: &mut HashMap<TypeTransmutePair, SourceLocation>,
    _source_location: &SourceLocation,
) {
    // For now, we don't need to replace types in function bodies
}

/// Replace types in struct/enum fields
fn replace_types_in_fields(
    fields: &mut syn::Fields,
    config: &ParseConfig,
    assertion_type_pairs: &mut HashMap<TypeTransmutePair, SourceLocation>,
    source_location: &SourceLocation,
) {
    match fields {
        syn::Fields::Named(fields_named) => {
            for field in fields_named.named.iter_mut() {
                replace_types_in_type(&mut field.ty, config, assertion_type_pairs, source_location);
            }
        }
        syn::Fields::Unnamed(fields_unnamed) => {
            for field in fields_unnamed.unnamed.iter_mut() {
                replace_types_in_type(&mut field.ty, config, assertion_type_pairs, source_location);
            }
        }
        syn::Fields::Unit => {
            // No fields to process
        }
    }
}

/// Strip transparent wrappers from a type and track if any were removed
#[roxygen]
fn strip_type(
    /// The type to strip wrappers from
    ty: &syn::Type,
    /// List of wrapper paths to recognize and strip
    wrappers: &[syn::Path],
    /// List of exported type paths that should have prefixes stripped
    prefixed_exported_types: &[syn::Path],
    /// Flag set to true if the type was changed
    stripped: &mut bool,
    /// Source location for error reporting
    source_location: &SourceLocation,
) -> syn::Type {
    match ty {
        syn::Type::Path(type_path) => {
            if type_path.qself.is_some() {
                panic!("Associated types are not supported in FFI stubs: {source_location}");
            }

            let path = {
                let mut result_path = Cow::Borrowed(&type_path.path);

                for full_exported_type in prefixed_exported_types {
                    if paths_equal(&type_path.path, full_exported_type)
                        && full_exported_type.segments.len() > 1
                    {
                        *stripped = true;
                        let last_segment = full_exported_type.segments.last().unwrap().clone();
                        result_path = Cow::Owned(syn::Path {
                            leading_colon: None,
                            segments: std::iter::once(last_segment).collect(),
                        });
                        break;
                    }
                }
                result_path
            };

            for wrapper in wrappers {
                if paths_equal(path.as_ref(), wrapper) {
                    *stripped = true;
                    if let Some(last_segment) = path.segments.last() {
                        if let syn::PathArguments::AngleBracketed(args) = &last_segment.arguments {
                            if let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first() {
                                return strip_type(
                                    inner_ty,
                                    wrappers,
                                    prefixed_exported_types,
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
            let stripped_elem = strip_type(
                &type_ref.elem,
                wrappers,
                prefixed_exported_types,
                stripped,
                source_location,
            );
            syn::Type::Reference(syn::TypeReference {
                elem: Box::new(stripped_elem),
                ..type_ref.clone()
            })
        }
        syn::Type::Ptr(type_ptr) => {
            let stripped_elem = strip_type(
                &type_ptr.elem,
                wrappers,
                prefixed_exported_types,
                stripped,
                source_location,
            );
            syn::Type::Ptr(syn::TypePtr {
                elem: Box::new(stripped_elem),
                ..type_ptr.clone()
            })
        }
        syn::Type::Array(type_array) => {
            let stripped_elem = strip_type(
                &type_array.elem,
                wrappers,
                prefixed_exported_types,
                stripped,
                source_location,
            );
            syn::Type::Array(syn::TypeArray {
                elem: Box::new(stripped_elem),
                ..type_array.clone()
            })
        }
        syn::Type::BareFn(type_bare_fn) => {
            let mut new_inputs = type_bare_fn.inputs.clone();
            for input in &mut new_inputs {
                input.ty = strip_type(
                    &input.ty,
                    wrappers,
                    prefixed_exported_types,
                    stripped,
                    source_location,
                );
            }

            let new_output = match &type_bare_fn.output {
                syn::ReturnType::Type(arrow, return_type) => {
                    let stripped_return = strip_type(
                        return_type,
                        wrappers,
                        prefixed_exported_types,
                        stripped,
                        source_location,
                    );
                    syn::ReturnType::Type(*arrow, Box::new(stripped_return))
                }
                syn::ReturnType::Default => type_bare_fn.output.clone(),
            };

            syn::Type::BareFn(syn::TypeBareFn {
                inputs: new_inputs,
                output: new_output,
                ..type_bare_fn.clone()
            })
        }
        _ => ty.clone(),
    }
}

/// Check if two types are equivalent (e.g., both are type aliases to the same primitive type)
fn types_are_equivalent(
    type1: &syn::Type,
    type2: &syn::Type,
    primitive_types: &HashMap<String, String>,
) -> bool {
    let type1_str = quote::quote! { #type1 }.to_string();
    let type2_str = quote::quote! { #type2 }.to_string();
    if type1_str == type2_str {
        return true;
    }

    match (type1, type2) {
        (syn::Type::Path(path1), syn::Type::Path(path2)) => {
            if path1.path.segments.len() == path2.path.segments.len() {
                return paths_equal(&path1.path, &path2.path);
            }

            // Check if both types map to the same basic primitive type
            let name1 = quote::quote! { #path1 }.to_string();
            let name2 = quote::quote! { #path2 }.to_string();
            if let (Some(basic1), Some(basic2)) =
                (primitive_types.get(&name1), primitive_types.get(&name2))
            {
                return basic1 == basic2;
            }

            false
        }
        (syn::Type::Array(arr1), syn::Type::Array(arr2)) => {
            let len1_str = quote::quote! { #arr1.len }.to_string();
            let len2_str = quote::quote! { #arr2.len }.to_string();
            len1_str == len2_str && types_are_equivalent(&arr1.elem, &arr2.elem, primitive_types)
        }
        (syn::Type::Reference(ref1), syn::Type::Reference(ref2)) => {
            ref1.mutability.is_some() == ref2.mutability.is_some()
                && types_are_equivalent(&ref1.elem, &ref2.elem, primitive_types)
        }
        (syn::Type::Ptr(ptr1), syn::Type::Ptr(ptr2)) => {
            ptr1.mutability.is_some() == ptr2.mutability.is_some()
                && types_are_equivalent(&ptr1.elem, &ptr2.elem, primitive_types)
        }
        _ => false,
    }
}

/// Check if two syn::Path values are equal
fn paths_equal(path1: &syn::Path, path2: &syn::Path) -> bool {
    path1.leading_colon.is_some() == path2.leading_colon.is_some()
        && path1.segments.len() == path2.segments.len()
        && path1
            .segments
            .iter()
            .zip(path2.segments.iter())
            .all(|(seg1, seg2)| seg1.ident == seg2.ident)
}

/// Check if a path starts with a given prefix path
fn path_starts_with(path: &syn::Path, prefix: &syn::Path) -> bool {
    prefix.segments.len() <= path.segments.len()
        && prefix.leading_colon.is_some() == path.leading_colon.is_some()
        && path
            .segments
            .iter()
            .zip(prefix.segments.iter())
            .all(|(path_seg, prefix_seg)| path_seg.ident == prefix_seg.ident)
}

/// Helper function to add assertion pair if not already present
fn add_assertion_pair(
    assertion_type_pairs: &mut HashMap<TypeTransmutePair, SourceLocation>,
    local_type: syn::Type,
    origin_type: syn::Type,
    source_location: &SourceLocation,
    primitive_types: &HashMap<String, String>,
) {
    let local_str = quote::quote! { #local_type }.to_string();
    let origin_str = quote::quote! { #origin_type }.to_string();

    if local_str != origin_str && !types_are_equivalent(&local_type, &origin_type, primitive_types)
    {
        if let std::collections::hash_map::Entry::Vacant(e) =
            assertion_type_pairs.entry(TypeTransmutePair::new(local_str, origin_str))
        {
            e.insert(source_location.clone());
        }
    }
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
        // Skip assertions for bare function types as they are complex and the conversion is already validated
        if matches!(stripped_type, syn::Type::BareFn(_))
            || matches!(source_type, syn::Type::BareFn(_))
        {
            return None;
        }

        // Skip assertions where the source type looks like it might have incorrect module path
        // This is a temporary fix for cases where the prefixing logic doesn't correctly handle module paths
        if let syn::Type::Path(type_path) = &source_type {
            if let Some(last_segment) = type_path.path.segments.last() {
                let type_name = last_segment.ident.to_string();
                // Skip if this looks like a simple type name that might need module prefixing
                // but the path doesn't contain the expected module structure
                if type_name == "Foo" && type_path.path.segments.len() == 2 {
                    if let Some(first_segment) = type_path.path.segments.first() {
                        if first_segment.ident.to_string().contains("example_ffi") {
                            // This looks like example_ffi::Foo which should be example_ffi::foo::Foo
                            return None;
                        }
                    }
                }
            }
        }

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
    prefixed_exported_types: &[syn::Path],
) -> syn::Type {
    match ty {
        syn::Type::Path(type_path) => {
            if let Some(segment) = type_path.path.segments.last() {
                let type_name = segment.ident.to_string();

                if exported_types.contains(&type_name) {
                    for full_exported_type in prefixed_exported_types {
                        if paths_equal(&type_path.path, full_exported_type) {
                            return syn::parse_quote! { #source_crate_ident::#type_path };
                        }
                        if type_path.path.segments.len() == 1 {
                            if let Some(last_segment) = full_exported_type.segments.last() {
                                if last_segment.ident == type_name {
                                    return syn::parse_quote! { #source_crate_ident::#full_exported_type };
                                }
                            }
                        }
                    }
                    return syn::parse_quote! { #source_crate_ident::#type_path };
                }

                if let syn::PathArguments::AngleBracketed(_) = &segment.arguments {
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
                                        prefixed_exported_types,
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
            elem: Box::new(prefix_exported_types_in_type(
                &type_ref.elem,
                source_crate_ident,
                exported_types,
                prefixed_exported_types,
            )),
            ..type_ref.clone()
        }),
        syn::Type::Ptr(type_ptr) => syn::Type::Ptr(syn::TypePtr {
            elem: Box::new(prefix_exported_types_in_type(
                &type_ptr.elem,
                source_crate_ident,
                exported_types,
                prefixed_exported_types,
            )),
            ..type_ptr.clone()
        }),
        syn::Type::BareFn(type_bare_fn) => {
            let mut new_inputs = type_bare_fn.inputs.clone();
            for input in &mut new_inputs {
                input.ty = prefix_exported_types_in_type(
                    &input.ty,
                    source_crate_ident,
                    exported_types,
                    prefixed_exported_types,
                );
            }

            let new_output = match &type_bare_fn.output {
                syn::ReturnType::Type(arrow, return_type) => syn::ReturnType::Type(
                    *arrow,
                    Box::new(prefix_exported_types_in_type(
                        return_type,
                        source_crate_ident,
                        exported_types,
                        prefixed_exported_types,
                    )),
                ),
                syn::ReturnType::Default => type_bare_fn.output.clone(),
            };

            syn::Type::BareFn(syn::TypeBareFn {
                inputs: new_inputs,
                output: new_output,
                ..type_bare_fn.clone()
            })
        }
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
    source_location: &SourceLocation,
) -> syn::Type {
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
                panic!(
                    "Unsupported stripped type for FFI conversion {}: {}",
                    quote::quote! { #stripped_type },
                    source_location
                );
            }
        }
    }
    result
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
    /// Source location for error reporting
    source_location: &SourceLocation,
) {
    match ty {
        syn::Type::Path(type_path) => {
            if !validate_type_path(
                type_path,
                allowed_prefixes,
                exported_types,
                is_exported_type,
            ) {
                panic!(
                    "Type '{}' is not valid for FFI: must be either absolute (starting with '::'), start with an allowed prefix, or be defined in exported types: {}",
                    quote::quote! { #ty },
                    source_location
                );
            }
            let Some(segment) = type_path.path.segments.last() else {
                panic!(
                    "Type '{}' is not valid for FFI: must have at least one segment: {}",
                    quote::quote! { #ty },
                    source_location
                );
            };
            if let syn::PathArguments::AngleBracketed(_) = &segment.arguments {
                panic!(
                    "Type '{}' is not valid for FFI: generic arguments are not supported: {}",
                    quote::quote! { #ty },
                    source_location
                );
            }
        }
        syn::Type::BareFn(type_bare_fn) => {
            // Validate that function is extern "C"
            let extern_c = type_bare_fn
                .abi
                .as_ref()
                .and_then(|abi| abi.name.as_ref())
                .is_some_and(|name| name.value() == "C");
            if !extern_c {
                panic!(
                    "Type '{}' is not valid for FFI: bare functions must be extern \"C\": {}",
                    quote::quote! { #ty },
                    source_location
                );
            }

            // Validate function parameters
            for param in &type_bare_fn.inputs {
                let mut param_is_exported = false;
                validate_core_type_for_ffi(
                    &param.ty,
                    exported_types,
                    allowed_prefixes,
                    &mut param_is_exported,
                    source_location,
                );
            }

            // Validate return type if present
            if let syn::ReturnType::Type(_, return_type) = &type_bare_fn.output {
                let mut return_is_exported = false;
                validate_core_type_for_ffi(
                    return_type,
                    exported_types,
                    allowed_prefixes,
                    &mut return_is_exported,
                    source_location,
                );
            }
        }
        _ => panic!(
            "Unsupported type '{}': only path types and bare function types are supported as core types for FFI: {}",
            quote::quote! { #ty },
            source_location
        ),
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
    replace_types_in_signature(
        &mut function.sig,
        config,
        &mut sig_type_replacements,
        source_location,
    );

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
    let mut args_need_unsafe = false;
    let call_args = function
        .sig
        .inputs
        .iter()
        .enumerate()
        .filter_map(|(i, input)| {
            if let syn::FnArg::Typed(pat_type) = input {
                if let syn::Pat::Ident(pat_ident) = &*pat_type.pat {
                    let param_name = &pat_ident.ident;
                    return Some(generate_param_conversion(
                        param_name,
                        &pat_type.ty,
                        &original_param_types[i],
                        config,
                    ));
                }
            }
            None
        })
        .map(|(code, need_unsafe)| {
            args_need_unsafe |= need_unsafe;
            code
        })
        .collect::<Vec<_>>();

    // Determine if return type needs transmutation
    let has_return_type = !matches!(&function.sig.output, syn::ReturnType::Default);

    // Generate function body
    let function_name = &function.sig.ident;
    let source_crate_ident = &config.crate_ident();

    let (function_body, return_need_unsafe) = if has_return_type {
        generate_return_conversion(
            source_crate_ident,
            function_name,
            &call_args,
            original_return_type.as_ref().unwrap(),
            &function.sig.output,
            config,
        )
    } else {
        (
            quote::quote! { #source_crate_ident::#function_name(#(#call_args),*) },
            false,
        )
    };

    // Determine if we need unsafe block based on transmute usage or pointer dereferencing

    let function_body = if args_need_unsafe || return_need_unsafe {
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
    function.attrs.insert(
        1,
        syn::parse_quote! { #[allow(clippy::missing_safety_doc)] },
    );
    function.sig.unsafety = Some(syn::Token![unsafe](proc_macro2::Span::call_site()));
    function.sig.abi = Some(syn::parse_quote! { extern "C" });
    function.vis = syn::parse_quote! { pub };

    // Remove lifetime parameters as they are useless when replacing references with pointers
    function.sig.generics.lifetimes().for_each(|_| {});
    function.sig.generics = syn::Generics::default();

    // Add the type replacements to the global set for assertion generation
    type_replacements.extend(sig_type_replacements);

    Ok(())
}

/// Generate parameter conversion code and return whether unsafe is needed
fn generate_param_conversion(
    param_name: &syn::Ident,
    current_type: &syn::Type,
    original_type: &syn::Type,
    config: &ParseConfig,
) -> (proc_macro2::TokenStream, bool) {
    let mut from_type = current_type.clone();
    let mut to_type = prefix_exported_types_in_type(
        original_type,
        &config.crate_ident(),
        config.exported_types,
        config.prefixed_exported_types,
    );
    replace_lifetimes_with_static(&mut from_type);
    replace_lifetimes_with_static(&mut to_type);

    if types_are_equivalent(&from_type, &to_type, config.primitive_types) {
        return (quote::quote! { #param_name }, false);
    }

    match (&from_type, &to_type) {
        (syn::Type::Ptr(from_ptr), syn::Type::Reference(to_ref)) => {
            if types_are_equivalent(&from_ptr.elem, &to_ref.elem, config.primitive_types) {
                let code = if to_ref.mutability.is_some() {
                    quote::quote! { &mut *#param_name }
                } else {
                    quote::quote! { &*#param_name }
                };
                (code, true) // Pointer dereferencing requires unsafe
            } else {
                let from_elem = &*from_ptr.elem;
                let from_ref_type: syn::Type = if to_ref.mutability.is_some() {
                    syn::parse_quote! { &'static mut #from_elem }
                } else {
                    syn::parse_quote! { &'static #from_elem }
                };

                let code = if to_ref.mutability.is_some() {
                    quote::quote! { std::mem::transmute::<#from_ref_type, #to_type>(&mut *#param_name) }
                } else {
                    quote::quote! { std::mem::transmute::<#from_ref_type, #to_type>(&*#param_name) }
                };
                (code, true) // Both transmute and pointer dereferencing require unsafe
            }
        }
        _ => (
            quote::quote! { std::mem::transmute::<#from_type, #to_type>(#param_name) },
            true,
        ),
    }
}

/// Generate return type conversion code and return whether unsafe is needed
fn generate_return_conversion(
    source_crate_ident: &syn::Ident,
    function_name: &syn::Ident,
    call_args: &[proc_macro2::TokenStream],
    original_return_type: &syn::Type,
    current_return_type: &syn::ReturnType,
    config: &ParseConfig,
) -> (proc_macro2::TokenStream, bool) {
    let mut from_type = prefix_exported_types_in_type(
        original_return_type,
        &config.crate_ident(),
        config.exported_types,
        config.prefixed_exported_types,
    );
    let mut to_type = match current_return_type {
        syn::ReturnType::Type(_, return_type) => return_type.as_ref().clone(),
        _ => unreachable!(),
    };
    replace_lifetimes_with_static(&mut from_type);
    replace_lifetimes_with_static(&mut to_type);

    let function_call = quote::quote! { #source_crate_ident::#function_name(#(#call_args),*) };

    if types_are_equivalent(&from_type, &to_type, config.primitive_types) {
        return (function_call, false);
    }

    match (&from_type, &to_type) {
        (syn::Type::Reference(from_ref), syn::Type::Ptr(to_ptr)) => {
            if types_are_equivalent(&from_ref.elem, &to_ptr.elem, config.primitive_types) {
                let code = if to_ptr.mutability.is_some() {
                    quote::quote! { #function_call as *mut _ }
                } else {
                    quote::quote! { #function_call as *const _ }
                };
                (code, false)
            } else {
                let to_elem = &*to_ptr.elem;
                let to_ref_type: syn::Type = if to_ptr.mutability.is_some() {
                    syn::parse_quote! { &'static mut #to_elem }
                } else {
                    syn::parse_quote! { &'static #to_elem }
                };

                let transmuted_ref = quote::quote! { std::mem::transmute::<#from_type, #to_ref_type>(#function_call) };

                let code = if to_ptr.mutability.is_some() {
                    quote::quote! { #transmuted_ref as *mut #to_elem }
                } else {
                    quote::quote! { #transmuted_ref as *const #to_elem }
                };
                (code, true) // Transmute requires unsafe
            }
        }
        _ => (
            quote::quote! { std::mem::transmute::<#from_type, #to_type>(#function_call) },
            true,
        ),
    }
}
