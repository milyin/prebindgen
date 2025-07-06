//! Code generation utilities for transforming Rust function definitions into FFI stubs.
//!
//! This module contains all the logic for:
//! - Parsing and validating function signatures for FFI compatibility
//! - Generating `#[no_mangle] extern "C"` wrapper functions
//! - Handling type transformations and validations
//! - Creating appropriate parameter names and call arguments
//! - Processing feature flags (`#[cfg(feature="...")]`) in generated code

use std::collections::{HashMap, HashSet};

/// Generate allowed prefixes that include standard prelude types and modules
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
        "i8", "i16", "i32", "i64", "i128", "isize",
        "u8", "u16", "u32", "u64", "u128", "usize", 
        "f32", "f64",
        "str",
    ];
    
    prefix_strings
        .into_iter()
        .filter_map(|s| syn::parse_str(s).ok())
        .collect()
}

/// Helper function to check if a type contains any of the exported types
pub(crate) fn contains_exported_type(
    ty: &syn::Type,
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

/// Validate generic arguments recursively
fn validate_generic_arguments(
    args: &syn::AngleBracketedGenericArguments,
    exported_types: &HashSet<String>,
    allowed_prefixes: &[syn::Path],
    context: &str,
) -> Result<(), String> {
    for arg in &args.args {
        if let syn::GenericArgument::Type(inner_ty) = arg {
            validate_type_for_ffi(
                inner_ty, 
                exported_types, 
                allowed_prefixes, 
                &format!("{} (generic argument)", context)
            )?;
        }
    }
    Ok(())
}

/// Helper function to validate that a type is either absolute (starting with ::) or defined in exported types
pub(crate) fn validate_type_for_ffi(
    ty: &syn::Type,
    exported_types: &HashSet<String>,
    allowed_prefixes: &[syn::Path],
    context: &str,
) -> Result<(), String> {
    match ty {
        syn::Type::Path(type_path) => {
            // Validate the type path (includes absolute paths, allowed prefixes, and exported types)
            if validate_type_path(type_path, allowed_prefixes) {
                if let Some(segment) = type_path.path.segments.last() {
                    if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                        validate_generic_arguments(args, exported_types, allowed_prefixes, context)?;
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
                            validate_generic_arguments(args, exported_types, allowed_prefixes, context)?;
                        }
                        return Ok(());
                    }
                }
            }
            
            // Invalid type path
            Err(format!(
                "Type '{}' in {} is not valid for FFI: must be either absolute (starting with '::'), start with an allowed prefix, or be defined in exported types",
                quote::quote! { #ty }, context
            ))
        }
        syn::Type::Reference(type_ref) => {
            validate_type_for_ffi(&type_ref.elem, exported_types, allowed_prefixes, &format!("{} (reference)", context))
        }
        syn::Type::Ptr(type_ptr) => {
            validate_type_for_ffi(&type_ptr.elem, exported_types, allowed_prefixes, &format!("{} (pointer)", context))
        }
        syn::Type::Slice(type_slice) => {
            validate_type_for_ffi(&type_slice.elem, exported_types, allowed_prefixes, &format!("{} (slice element)", context))
        }
        syn::Type::Array(type_array) => {
            validate_type_for_ffi(&type_array.elem, exported_types, allowed_prefixes, &format!("{} (array element)", context))
        }
        syn::Type::Tuple(type_tuple) => {
            for (i, elem_ty) in type_tuple.elems.iter().enumerate() {
                validate_type_for_ffi(elem_ty, exported_types, allowed_prefixes, &format!("{} (tuple element {})", context, i))?;
            }
            Ok(())
        }
        _ => {
            Err(format!(
                "Unsupported type '{}' in {}: only path types, references, pointers, slices, arrays, and tuples are supported for FFI",
                quote::quote! { #ty }, context
            ))
        }
    }
}

/// Convert a remote type to its local equivalent and collect assertion pairs
/// Returns the local type used in the stub (stripped of wrappers and crate prefixes, with references converted to pointers)
/// If type needs conversion (has transparent wrappers or exported types), stores the assertion pair
fn convert_to_local_type(
    original_type: &syn::Type,
    exported_types: &HashSet<String>,
    transparent_wrappers: &[syn::Path],
    source_crate_name: &str,
    assertion_type_pairs: &mut HashSet<(String, String)>
) -> syn::Type {
    // Extract the core type to process (for references, this is the referenced type)
    let (core_type, is_reference, ref_info) = match original_type {
        syn::Type::Reference(type_ref) => (
            &*type_ref.elem, 
            true, 
            Some((type_ref.and_token, type_ref.lifetime.clone(), type_ref.mutability))
        ),
        _ => (original_type, false, None)
    };
    
    // Strip transparent wrappers from the core type
    let mut has_wrapper = false;
    let local_core_type = strip_transparent_wrappers_for_assertion(core_type, transparent_wrappers, &mut has_wrapper);
    
    // Check if we should generate an assertion for this type
    let should_convert = has_wrapper || contains_exported_type(&local_core_type, exported_types);
    
    if should_convert {
        // Create the original core type with proper crate prefixing
        let prefixed_original_core = prefix_exported_types_in_type(core_type, source_crate_name, exported_types);
        
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
        convert_reference_to_pointer(&local_ref)
    } else if should_convert {
        // Non-reference type that needed conversion
        local_core_type
    } else {
        // No conversion needed, return original type
        original_type.clone()
    }
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

/// Generate call arguments with pointer-to-reference conversion and transmute for exported types
fn generate_call_arguments(
    original_inputs: &syn::punctuated::Punctuated<syn::FnArg, syn::token::Comma>,
    exported_types: &HashSet<String>,
) -> Vec<proc_macro2::TokenStream> {
    original_inputs.iter().filter_map(|input| match input {
        syn::FnArg::Typed(pat_type) => {
            if let syn::Pat::Ident(pat_ident) = &*pat_type.pat {
                let param_name = &pat_ident.ident;
                
                match &*pat_type.ty {
                    syn::Type::Reference(type_ref) => {
                        // For reference parameters, convert pointer back to reference
                        if type_ref.mutability.is_some() {
                            // &mut T parameter becomes *mut T in FFI, convert back with &mut *param_name
                            if contains_exported_type(&type_ref.elem, exported_types) {
                                Some(quote::quote! { unsafe { std::mem::transmute(&mut *#param_name) } })
                            } else {
                                Some(quote::quote! { &mut *#param_name })
                            }
                        } else {
                            // &T parameter becomes *const T in FFI, convert back with &*param_name
                            if contains_exported_type(&type_ref.elem, exported_types) {
                                Some(quote::quote! { unsafe { std::mem::transmute(&*#param_name) } })
                            } else {
                                Some(quote::quote! { &*#param_name })
                            }
                        }
                    }
                    _ => {
                        // For non-reference parameters, use as-is with optional transmute
                        if contains_exported_type(&pat_type.ty, exported_types) {
                            Some(quote::quote! { unsafe { std::mem::transmute(#param_name) } })
                        } else {
                            Some(quote::quote! { #param_name })
                        }
                    }
                }
            } else {
                None
            }
        }
        _ => None,
    }).collect()
}

/// Generate the function body that calls the original function
fn generate_function_body(
    return_type: &syn::ReturnType,
    function_name: &syn::Ident,
    source_crate_ident: &syn::Ident,
    call_args: &[proc_macro2::TokenStream],
    exported_types: &HashSet<String>,
) -> proc_macro2::TokenStream {
    match return_type {
        syn::ReturnType::Default => {
            // Void function
            quote::quote! {
                #source_crate_ident::#function_name(#(#call_args),*);
            }
        }
        syn::ReturnType::Type(_, return_ty) => {
            // Function with return value
            if contains_exported_type(return_ty, exported_types) {
                quote::quote! {
                    let result = #source_crate_ident::#function_name(#(#call_args),*);
                    unsafe { std::mem::transmute(result) }
                }
            } else {
                quote::quote! {
                    #source_crate_ident::#function_name(#(#call_args),*)
                }
            }
        }
    }
}

/// Validate all function parameters for FFI compatibility
fn validate_function_parameters(
    inputs: &syn::punctuated::Punctuated<syn::FnArg, syn::token::Comma>,
    function_name: &syn::Ident,
    exported_types: &HashSet<String>,
    allowed_prefixes: &[syn::Path],
) -> Result<(), String> {
    for (i, input) in inputs.iter().enumerate() {
        if let syn::FnArg::Typed(pat_type) = input {
            validate_type_for_ffi(
                &pat_type.ty,
                exported_types,
                allowed_prefixes,
                &format!("parameter {} of function '{}'", i + 1, function_name),
            ).map_err(|e| format!("Invalid FFI function parameter: {}", e))?;
        }
    }
    Ok(())
}

/// Transform a function prototype to a no_mangle extern "C" function and collect assertion pairs
/// Returns the stub function and the collected assertion pairs separately for later deduplication
pub(crate) fn transform_function_to_stub(
    file: syn::File,
    source_crate: &str,
    exported_types: &HashSet<String>,
    allowed_prefixes: &[syn::Path],
    transparent_wrappers: &[syn::Path],
    edition: &str,
    assertion_type_pairs: &mut HashSet<(String, String)>
) -> Result<syn::File, String> {
    // Validate that the file contains exactly one function
    if file.items.len() != 1 {
        return Err(format!("Expected exactly one item in file, found {}", file.items.len()));
    }
    
    let parsed_function = match &file.items[0] {
        syn::Item::Fn(item_fn) => item_fn,
        item => return Err(format!("Expected function item, found {:?}", std::mem::discriminant(item))),
    };
    
    let function_name = &parsed_function.sig.ident;

    // Validate function signature
    validate_function_parameters(&parsed_function.sig.inputs, function_name, exported_types, allowed_prefixes)?;
    
    // Prepare source crate name for type collection
    let source_crate_name = source_crate.replace('-', "_");
    
    // Validate return type
    if let syn::ReturnType::Type(_, return_type) = &parsed_function.sig.output {
        validate_type_for_ffi(
            return_type,
            exported_types,
            allowed_prefixes,
            &format!("return type of function '{}'", function_name),
        ).map_err(|e| format!("Invalid FFI function return type: {}", e))?;
    }

    // Generate components
    let call_args = generate_call_arguments(&parsed_function.sig.inputs, exported_types);
    
    let source_crate_ident = syn::Ident::new(&source_crate_name, proc_macro2::Span::call_site());
    
    let function_body = generate_function_body(
        &parsed_function.sig.output,
        function_name,
        &source_crate_ident,
        &call_args,
        exported_types,
    );

    // Determine the appropriate no_mangle attribute based on Rust edition
    // Edition 2024 uses #[unsafe(no_mangle)], while older editions use #[no_mangle]
    let no_mangle_attr: syn::Attribute = if edition == "2024" {
        syn::parse_quote! { #[unsafe(no_mangle)] }
    } else {
        syn::parse_quote! { #[no_mangle] }
    };

    // Build the extern "C" function signature:
    // 1. Start with the original function signature
    // 2. Convert references to pointers for FFI compatibility
    // 3. Add extern "C" ABI specifier
    // 4. Mark function as unsafe
    let mut extern_sig = parsed_function.sig.clone();
    
    // Convert return type and collect type assertion pairs
    if let syn::ReturnType::Type(arrow, return_type) = &extern_sig.output {
        let local_return_type = convert_to_local_type(return_type, exported_types, transparent_wrappers, &source_crate_name, assertion_type_pairs);
        extern_sig.output = syn::ReturnType::Type(*arrow, Box::new(local_return_type));
    }
    
    // Convert reference parameters to pointer parameters and collect type assertion pairs
    for input in extern_sig.inputs.iter_mut() {
        if let syn::FnArg::Typed(pat_type) = input {
            // Convert type and collect assertion pairs (handles both reference and non-reference types)
            let local_type = convert_to_local_type(&pat_type.ty, exported_types, transparent_wrappers, &source_crate_name, assertion_type_pairs);
            
            // Use the local type
            pat_type.ty = Box::new(local_type);
        }
    }
    
    extern_sig.abi = Some(syn::Abi {
        extern_token: syn::token::Extern::default(),
        name: Some(syn::LitStr::new("C", proc_macro2::Span::call_site())),
    });
    extern_sig.unsafety = Some(syn::token::Unsafe::default());

    // Create the function body that will call the original implementation
    let body = syn::parse_quote! {
        {
            #function_body
        }
    };

    // Build the complete extern function
    let mut attrs = vec![no_mangle_attr];
    attrs.extend(parsed_function.attrs.clone());

    let extern_function = syn::ItemFn {
        attrs,
        vis: parsed_function.vis.clone(),
        sig: extern_sig,
        block: Box::new(body),
    };

    // Return a syn::File containing only the stub function (no assertions)
    Ok(syn::File {
        shebang: file.shebang,
        attrs: file.attrs,
        items: vec![syn::Item::Fn(extern_function)],
    })
}

/// Process code content to handle feature flags according to builder configuration.
/// 
/// This function analyzes code for `#[cfg(feature="...")]` attributes using syn syntax parsing and:
/// - Removes code blocks guarded by disabled features
/// - Removes cfg attributes for enabled features (including the code)
/// - Replaces feature names according to the mapping (keeping the cfg attribute)
pub(crate) fn process_features(
    mut file: syn::File,
    disabled_features: &HashSet<String>,
    enabled_features: &HashSet<String>,
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
    // Extract and process cfg attributes
    let mut keep_item = true;
    let mut remove_attrs = Vec::new();
    
    let attrs = match item {
        syn::Item::Fn(f) => &mut f.attrs,
        syn::Item::Struct(s) => &mut s.attrs,
        syn::Item::Enum(e) => &mut e.attrs,
        syn::Item::Union(u) => &mut u.attrs,
        syn::Item::Type(t) => &mut t.attrs,
        syn::Item::Const(c) => &mut c.attrs,
        syn::Item::Static(s) => &mut s.attrs,
        syn::Item::Mod(m) => &mut m.attrs,
        syn::Item::Use(u) => &mut u.attrs,
        syn::Item::Impl(i) => &mut i.attrs,
        syn::Item::Trait(t) => &mut t.attrs,
        _ => return true, // Keep other items as-is
    };
    
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
pub(crate) fn generate_type_assertions(assertion_type_pairs: &HashSet<(String, String)>) -> Vec<syn::Item> {
    let mut assertions = Vec::new();
    
    for (stripped_type_str, source_type_str) in assertion_type_pairs {
        // Parse the type strings back into syn::Type for proper code generation
        if let (Ok(stripped_type), Ok(source_type)) = (
            syn::parse_str::<syn::Type>(stripped_type_str),
            syn::parse_str::<syn::Type>(source_type_str)
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



/// Strip transparent wrappers and track if any were stripped
fn strip_transparent_wrappers_for_assertion(
    ty: &syn::Type, 
    transparent_wrappers: &[syn::Path],
    has_wrapper: &mut bool
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
                                return strip_transparent_wrappers_for_assertion(inner_ty, transparent_wrappers, has_wrapper);
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
fn prefix_exported_types_in_type(ty: &syn::Type, source_crate_name: &str, exported_types: &HashSet<String>) -> syn::Type {
    match ty {
        syn::Type::Path(type_path) => {
            if let Some(segment) = type_path.path.segments.last() {
                let type_name = segment.ident.to_string();
                
                // Only prefix if this is an exported type
                if exported_types.contains(&type_name) && type_path.path.segments.len() == 1 {
                    let source_crate_ident = syn::Ident::new(source_crate_name, proc_macro2::Span::call_site());
                    return syn::parse_quote! { #source_crate_ident::#type_path };
                }
                
                // Handle generic arguments recursively
                if let syn::PathArguments::AngleBracketed(_args) = &segment.arguments {
                    let mut new_path = type_path.path.clone();
                    if let Some(last_segment) = new_path.segments.last_mut() {
                        if let syn::PathArguments::AngleBracketed(ref mut args) = last_segment.arguments {
                            for arg in &mut args.args {
                                if let syn::GenericArgument::Type(inner_ty) = arg {
                                    *inner_ty = prefix_exported_types_in_type(inner_ty, source_crate_name, exported_types);
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
        syn::Type::Reference(type_ref) => {
            syn::Type::Reference(syn::TypeReference {
                and_token: type_ref.and_token,
                lifetime: type_ref.lifetime.clone(),
                mutability: type_ref.mutability,
                elem: Box::new(prefix_exported_types_in_type(&type_ref.elem, source_crate_name, exported_types)),
            })
        }
        syn::Type::Ptr(type_ptr) => {
            syn::Type::Ptr(syn::TypePtr {
                star_token: type_ptr.star_token,
                const_token: type_ptr.const_token,
                mutability: type_ptr.mutability,
                elem: Box::new(prefix_exported_types_in_type(&type_ptr.elem, source_crate_name, exported_types)),
            })
        }
        _ => ty.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{HashMap, HashSet};

    #[test]
    fn test_process_features_disable() {
        let content = r#"
#[cfg(feature = "experimental")]
pub struct ExperimentalStruct {
    pub field: i32,
}

pub struct RegularStruct {
    pub field: i32,
}
"#;

        let mut disabled_features = HashSet::new();
        disabled_features.insert("experimental".to_string());
        let enabled_features = HashSet::new();
        let feature_mappings = HashMap::new();

        let file = syn::parse_file(content).unwrap();
        let result = process_features(file, &disabled_features, &enabled_features, &feature_mappings);
        let result_str = prettyplease::unparse(&result);
        
        // Should not contain the experimental struct
        assert!(!result_str.contains("ExperimentalStruct"));
        // Should still contain the regular struct
        assert!(result_str.contains("RegularStruct"));
    }

    #[test]
    fn test_process_features_enable() {
        let content = r#"
#[cfg(feature = "std")]
pub struct StdStruct {
    pub field: i32,
}

pub struct RegularStruct {
    pub field: i32,
}
"#;

        let disabled_features = HashSet::new();
        let mut enabled_features = HashSet::new();
        enabled_features.insert("std".to_string());
        let feature_mappings = HashMap::new();

        let file = syn::parse_file(content).unwrap();
        let result = process_features(file, &disabled_features, &enabled_features, &feature_mappings);
        let result_str = prettyplease::unparse(&result);
        
        // Should contain the std struct without cfg attribute
        assert!(result_str.contains("StdStruct"));
        assert!(!result_str.contains(r#"cfg(feature = "std")"#));
        // Should still contain the regular struct
        assert!(result_str.contains("RegularStruct"));
    }

    #[test]
    fn test_process_features_mapping() {
        let content = r#"
#[cfg(feature = "unstable")]
pub struct UnstableStruct {
    pub field: i32,
}

pub struct RegularStruct {
    pub field: i32,
}
"#;

        let disabled_features = HashSet::new();
        let enabled_features = HashSet::new();
        let mut feature_mappings = HashMap::new();
        feature_mappings.insert("unstable".to_string(), "stable".to_string());

        let file = syn::parse_file(content).unwrap();
        let result = process_features(file, &disabled_features, &enabled_features, &feature_mappings);
        let result_str = prettyplease::unparse(&result);
        
        // Should contain the struct with mapped feature name
        assert!(result_str.contains("UnstableStruct"));
        assert!(result_str.contains(r#"cfg(feature = "stable")"#));
        assert!(!result_str.contains(r#"cfg(feature = "unstable")"#));
        // Should still contain the regular struct
        assert!(result_str.contains("RegularStruct"));
    }

    #[test]
    fn test_process_features_complex_syn_parsing() {
        let content = r#"
#[cfg(feature = "async")]
pub struct AsyncStruct {
    pub field: i32,
}

#[cfg(feature = "sync")]
impl AsyncStruct {
    pub fn new() -> Self {
        Self { field: 0 }
    }
}

#[cfg(feature = "deprecated")]
pub fn old_function() {
    // deprecated function
}

pub enum RegularEnum {
    A,
    B,
}
"#;

        let mut disabled_features = HashSet::new();
        disabled_features.insert("deprecated".to_string());
        
        let mut enabled_features = HashSet::new();
        enabled_features.insert("async".to_string());
        
        let mut feature_mappings = HashMap::new();
        feature_mappings.insert("sync".to_string(), "synchronous".to_string());

        let file = syn::parse_file(content).unwrap();
        let result = process_features(file, &disabled_features, &enabled_features, &feature_mappings);
        let result_str = prettyplease::unparse(&result);
        
        // Should not contain the deprecated function
        assert!(!result_str.contains("old_function"));
        
        // Should contain AsyncStruct without cfg attribute
        assert!(result_str.contains("AsyncStruct"));
        assert!(!result_str.contains(r#"cfg(feature = "async")"#));
        
        // Should contain the impl block with mapped feature name
        assert!(result_str.contains("impl AsyncStruct"));
        assert!(result_str.contains(r#"cfg(feature = "synchronous")"#));
        assert!(!result_str.contains(r#"cfg(feature = "sync")"#));
        
        // Should still contain the regular enum
        assert!(result_str.contains("RegularEnum"));
    }

    #[test]
    fn test_transform_function_to_stub() {
        let function_content = r#"
pub fn example_function(x: i32, y: &str) -> i32 {
    42
}
"#;

        let exported_types = HashSet::new();
        let allowed_prefixes = generate_standard_allowed_prefixes();
        let transparent_wrappers = Vec::new();
        let mut assertion_type_pairs = HashSet::new();

        let input_file = syn::parse_file(function_content).unwrap();
        let result = transform_function_to_stub(
            input_file,
            "my-crate",
            &exported_types,
            &allowed_prefixes,
            &transparent_wrappers,
            "2021",
            &mut assertion_type_pairs,
        ).unwrap();

        let result_str = prettyplease::unparse(&result);
        
        // Should contain the no_mangle attribute
        assert!(result_str.contains("no_mangle"));
        // Should be an unsafe extern "C" function
        assert!(result_str.contains("unsafe extern \"C\""));
        // Should have original parameter names
        assert!(result_str.contains("x"));
        assert!(result_str.contains("y"));
        // Should convert &str to *const str in signature
        assert!(result_str.contains("*const str"));
        // Should convert pointer back to reference in function call
        assert!(result_str.contains("&*y"));
        // Should call the original function from the source crate
        assert!(result_str.contains("my_crate::example_function"));
    }

    #[test]
    fn test_transform_function_to_stub_edition_2024() {
        let function_content = r#"
pub fn example_function() -> i32 {
    42
}
"#;

        let exported_types = HashSet::new();
        let allowed_prefixes = generate_standard_allowed_prefixes();
        let transparent_wrappers = Vec::new();
        let mut assertion_type_pairs = HashSet::new();

        let input_file = syn::parse_file(function_content).unwrap();
        let result = transform_function_to_stub(
            input_file,
            "my-crate",
            &exported_types,
            &allowed_prefixes,
            &transparent_wrappers,
            "2024",
            &mut assertion_type_pairs,
        ).unwrap();

        let result_str = prettyplease::unparse(&result);
        
        // Should contain the unsafe no_mangle attribute for 2024 edition
        assert!(result_str.contains("#[unsafe(no_mangle)]"));
    }

    #[test]
    fn test_transform_function_to_stub_wrong_item_count() {
        // Test with empty file
        let empty_file = syn::File {
            shebang: None,
            attrs: vec![],
            items: vec![],
        };
        
        let exported_types = HashSet::new();
        let allowed_prefixes = generate_standard_allowed_prefixes();
        let transparent_wrappers = Vec::new();
        let mut assertion_type_pairs = HashSet::new();
        
        let result = transform_function_to_stub(
            empty_file,
            "my-crate",
            &exported_types,
            &allowed_prefixes,
            &transparent_wrappers,
            "2021",
            &mut assertion_type_pairs,
        );
        
        match result {
            Err(error_msg) => assert!(error_msg.contains("Expected exactly one item")),
            Ok(_) => panic!("Expected error but got success"),
        }
        
        // Test with multiple items
        let function_content = r#"
pub fn first_function() -> i32 { 42 }
pub fn second_function() -> i32 { 24 }
"#;
        
        let multi_item_file = syn::parse_file(function_content).unwrap();
        let mut assertion_type_pairs = HashSet::new();
        let result = transform_function_to_stub(
            multi_item_file,
            "my-crate",
            &exported_types,
            &allowed_prefixes,
            &transparent_wrappers,
            "2021",
            &mut assertion_type_pairs,
        );
        
        match result {
            Err(error_msg) => assert!(error_msg.contains("Expected exactly one item")),
            Ok(_) => panic!("Expected error but got success"),
        }
    }

    #[test]
    fn test_transform_function_to_stub_wrong_item_type() {
        let struct_content = r#"
pub struct MyStruct {
    field: i32,
}
"#;
        
        let exported_types = HashSet::new();
        let allowed_prefixes = generate_standard_allowed_prefixes();
        let transparent_wrappers = Vec::new();
        let mut assertion_type_pairs = HashSet::new();
        
        let struct_file = syn::parse_file(struct_content).unwrap();
        let result = transform_function_to_stub(
            struct_file,
            "my-crate",
            &exported_types,
            &allowed_prefixes,
            &transparent_wrappers,
            "2021",
            &mut assertion_type_pairs,
        );
        
        match result {
            Err(error_msg) => assert!(error_msg.contains("Expected function item")),
            Ok(_) => panic!("Expected error but got success"),
        }
    }

    #[test]
    fn test_transform_function_with_references() {
        let function_content = r#"
pub fn copy_bar(
    dst: &mut std::mem::MaybeUninit<Bar>,
    src: &Bar,
) -> i32 {
    42
}
"#;

        let mut exported_types = HashSet::new();
        exported_types.insert("Bar".to_string());
        let allowed_prefixes = generate_standard_allowed_prefixes();
        let transparent_wrappers = Vec::new();
        let mut assertion_type_pairs = HashSet::new();

        let input_file = syn::parse_file(function_content).unwrap();
        let result = transform_function_to_stub(
            input_file,
            "my-crate",
            &exported_types,
            &allowed_prefixes,
            &transparent_wrappers,
            "2021",
            &mut assertion_type_pairs,
        ).unwrap();

        let result_str = prettyplease::unparse(&result);
        
        // Should contain the no_mangle attribute
        assert!(result_str.contains("no_mangle"));
        // Should be an unsafe extern "C" function
        assert!(result_str.contains("unsafe extern \"C\""));
        // Should convert &mut T to *mut T
        assert!(result_str.contains("*mut"));
        // Should convert &T to *const T  
        assert!(result_str.contains("*const"));
        // Should convert pointers back to references in function call
        assert!(result_str.contains("&mut *dst"));
        assert!(result_str.contains("&*src"));
        // Should call the original function from the source crate
        assert!(result_str.contains("my_crate::copy_bar"));
    }

    #[test]
    fn test_transform_function_with_transparent_wrapper_assertions() {
        let function_content = r#"
pub fn copy_bar(
    dst: &mut std::mem::MaybeUninit<Bar>,
    src: &Bar,
) -> std::mem::MaybeUninit<i32> {
    std::mem::MaybeUninit::new(42)
}
"#;

        let mut exported_types = HashSet::new();
        exported_types.insert("Bar".to_string());
        let allowed_prefixes = generate_standard_allowed_prefixes();
        
        let mut transparent_wrappers = Vec::new();
        let maybe_uninit_path: syn::Path = syn::parse_quote! { std::mem::MaybeUninit };
        transparent_wrappers.push(maybe_uninit_path);
        let mut assertion_type_pairs = HashSet::new();

        let input_file = syn::parse_file(function_content).unwrap();
        let result = transform_function_to_stub(
            input_file,
            "my-crate",
            &exported_types,
            &allowed_prefixes,
            &transparent_wrappers,
            "2021",
            &mut assertion_type_pairs,
        ).unwrap();

        // Generate assertions from collected pairs and append to result
        let assertions = generate_type_assertions(&assertion_type_pairs);
        let mut complete_result = result;
        complete_result.items.extend(assertions);
        
        let result_str = prettyplease::unparse(&complete_result);
        
        // Should contain the extern function
        assert!(result_str.contains("no_mangle"));
        assert!(result_str.contains("unsafe extern \"C\""));
        
        // Should contain compile-time assertions for size and alignment
        assert!(result_str.contains("std::mem::size_of"));
        assert!(result_str.contains("std::mem::align_of"));
        assert!(result_str.contains("Size mismatch between stub parameter type and source crate type"));
        assert!(result_str.contains("Alignment mismatch between stub parameter type and source crate type"));
        
        // Should have assertions for the stripped types (MaybeUninit only in this test)
        assert!(result_str.contains("MaybeUninit"));
    }

    #[test]
    fn test_convert_reference_to_pointer() {
        // Test mutable reference conversion
        let mut_ref: syn::Type = syn::parse_quote! { &mut i32 };
        let converted = convert_reference_to_pointer(&mut_ref);
        let converted_str = quote::quote! { #converted }.to_string();
        assert!(converted_str.contains("* mut i32"));

        // Test immutable reference conversion
        let ref_type: syn::Type = syn::parse_quote! { &str };
        let converted = convert_reference_to_pointer(&ref_type);
        let converted_str = quote::quote! { #converted }.to_string();
        assert!(converted_str.contains("* const str"));

        // Test non-reference type (should remain unchanged)
        let regular_type: syn::Type = syn::parse_quote! { i32 };
        let converted = convert_reference_to_pointer(&regular_type);
        let converted_str = quote::quote! { #converted }.to_string();
        assert_eq!(converted_str, "i32");
    }

    #[test]
    fn test_strip_transparent_wrapper() {
        let mut transparent_wrappers = Vec::new();
        let maybe_uninit_path: syn::Path = syn::parse_quote! { std::mem::MaybeUninit };
        transparent_wrappers.push(maybe_uninit_path);

        let function_content = r#"
pub fn copy_bar(
    dst: &mut std::mem::MaybeUninit<Bar>,
    src: &Bar,
) -> i32 {
    42
}
"#;

        let mut exported_types = HashSet::new();
        exported_types.insert("Bar".to_string());
        let allowed_prefixes = generate_standard_allowed_prefixes();
        let mut assertion_type_pairs = HashSet::new();

        let input_file = syn::parse_file(function_content).unwrap();
        let result = transform_function_to_stub(
            input_file,
            "my-crate",
            &exported_types,
            &allowed_prefixes,
            &transparent_wrappers,
            "2021",
            &mut assertion_type_pairs,
        ).unwrap();

        let result_str = prettyplease::unparse(&result);
        
        // Should contain the no_mangle attribute
        assert!(result_str.contains("no_mangle"));
        // Should be an unsafe extern "C" function
        assert!(result_str.contains("unsafe extern \"C\""));
        // Should strip MaybeUninit wrapper and convert &mut MaybeUninit<Bar> to *mut Bar
        assert!(result_str.contains("*mut Bar"));
        // Should convert &Bar to *const Bar
        assert!(result_str.contains("*const Bar"));
        // The function signature should NOT contain MaybeUninit (but assertions might)
        // Let's split the check - function signature vs assertions
        let lines: Vec<&str> = result_str.lines().collect();
        let function_lines: Vec<&str> = lines.iter()
            .take_while(|line| !line.contains("const _"))
            .cloned()
            .collect();
        let function_code = function_lines.join("\n");
        assert!(!function_code.contains("MaybeUninit"));
        // Should call the original function from the source crate
        assert!(result_str.contains("my_crate::copy_bar"));
    }

    #[test]
    fn test_strip_transparent_wrappers_nested() {
        let transparent_wrappers = vec![
            syn::parse_quote! { std::mem::MaybeUninit },
            syn::parse_quote! { std::mem::ManuallyDrop },
        ];

        // Test nested transparent wrappers: MaybeUninit<ManuallyDrop<T>>
        let nested_type: syn::Type = syn::parse_quote! { 
            std::mem::MaybeUninit<std::mem::ManuallyDrop<i32>> 
        };
        
        let mut has_wrapper = false;
        let stripped = strip_transparent_wrappers_for_assertion(&nested_type, &transparent_wrappers, &mut has_wrapper);
        let stripped_str = quote::quote! { #stripped }.to_string();
        
        // Should strip both wrappers and leave just i32
        assert_eq!(stripped_str, "i32");
        
        // Should have detected wrappers
        assert!(has_wrapper);
    }

    #[test]
    fn test_type_assertions_generation() {
        // Test the assertion generation function directly
        let mut assertion_type_pairs = HashSet::new();
        assertion_type_pairs.insert((
            "std::mem::MaybeUninit<i32>".to_string(),
            "my_crate::i32".to_string()
        ));
        assertion_type_pairs.insert((
            "String".to_string(),
            "my_crate::String".to_string()
        ));

        let assertions = generate_type_assertions(&assertion_type_pairs);
        assert_eq!(assertions.len(), 4); // 2 types × 2 assertions each (size + alignment)

        let assertions_str = assertions.iter()
            .map(|item| {
                let file = syn::File {
                    shebang: None,
                    attrs: vec![],
                    items: vec![item.clone()],
                };
                prettyplease::unparse(&file)
            })
            .collect::<Vec<_>>()
            .join("\n");

        // Should contain size and alignment checks
        assert!(assertions_str.contains("std::mem::size_of"));
        assert!(assertions_str.contains("std::mem::align_of"));
        assert!(assertions_str.contains("Size mismatch between stub parameter type and source crate type"));
        assert!(assertions_str.contains("Alignment mismatch between stub parameter type and source crate type"));
    }

    #[test]
    fn test_exported_type_assertions() {
        let function_content = r#"
pub fn process_data(
    data: &MyExportedStruct,
    mut output: &mut AnotherExportedType,
) -> ExportedEnum {
    ExportedEnum::Success
}
"#;

        let mut exported_types = HashSet::new();
        exported_types.insert("MyExportedStruct".to_string());
        exported_types.insert("AnotherExportedType".to_string());
        exported_types.insert("ExportedEnum".to_string());
        
        let allowed_prefixes = generate_standard_allowed_prefixes();
        let transparent_wrappers = Vec::new();
        let mut assertion_type_pairs = HashSet::new();

        let input_file = syn::parse_file(function_content).unwrap();
        let result = transform_function_to_stub(
            input_file,
            "my-source-crate",
            &exported_types,
            &allowed_prefixes,
            &transparent_wrappers,
            "2021",
            &mut assertion_type_pairs,
        ).unwrap();

        // Generate assertions from collected pairs and append to result
        let assertions = generate_type_assertions(&assertion_type_pairs);
        let mut complete_result = result;
        complete_result.items.extend(assertions);

        let result_str = prettyplease::unparse(&complete_result);
        
        // Should contain the extern function
        assert!(result_str.contains("no_mangle"));
        assert!(result_str.contains("unsafe extern \"C\""));
        
        // Should contain compile-time assertions for exported types
        assert!(result_str.contains("std::mem::size_of"));
        assert!(result_str.contains("std::mem::align_of"));
        
        // Should have assertions comparing local types vs source crate types
        assert!(result_str.contains("my_source_crate::"));
        assert!(result_str.contains("Size mismatch between stub parameter type and source crate type"));
        assert!(result_str.contains("Alignment mismatch between stub parameter type and source crate type"));
    }

    #[test]
    fn test_corrected_assertion_logic() {
        // Test case: function with transparent wrapper and exported type
        let function_content = r#"
pub fn test_func(wrapper: &std::mem::MaybeUninit<ExportedType>) -> ExportedType {
    ExportedType::default()
}
"#;

        let mut exported_types = HashSet::new();
        exported_types.insert("ExportedType".to_string());
        
        let allowed_prefixes = generate_standard_allowed_prefixes();
        
        let mut transparent_wrappers = Vec::new();
        let maybe_uninit_path: syn::Path = syn::parse_quote! { std::mem::MaybeUninit };
        transparent_wrappers.push(maybe_uninit_path);
        let mut assertion_type_pairs = HashSet::new();

        let input_file = syn::parse_file(function_content).unwrap();
        let result = transform_function_to_stub(
            input_file,
            "source_crate",
            &exported_types,
            &allowed_prefixes,
            &transparent_wrappers,
            "2021",
            &mut assertion_type_pairs,
        ).unwrap();

        // Generate assertions from collected pairs and append to result
        let assertions = generate_type_assertions(&assertion_type_pairs);
        let mut complete_result = result;
        complete_result.items.extend(assertions);

        let result_str = prettyplease::unparse(&complete_result);
        
        println!("Generated code:\n{}", result_str);
        
        // Should contain the extern function
        assert!(result_str.contains("no_mangle"));
        assert!(result_str.contains("unsafe extern \"C\""));
        
        // Should contain compile-time assertions
        assert!(result_str.contains("const _:"));
        assert!(result_str.contains("std::mem::size_of"));
        assert!(result_str.contains("std::mem::align_of"));
        
        // Should have the correct assertion message
        assert!(result_str.contains("Size mismatch between stub parameter type and source crate type"));
        assert!(result_str.contains("Alignment mismatch between stub parameter type and source crate type"));
        
        // Should have assertions for:
        // 1. Parameter: Stripped type (ExportedType) vs original type (std::mem::MaybeUninit<source_crate::ExportedType>)
        // 2. Return type: ExportedType vs source_crate::ExportedType
        assert!(result_str.contains("source_crate::ExportedType"));
        assert!(result_str.contains("MaybeUninit < source_crate::ExportedType"));
        
        // Should NOT generate duplicate assertions - count occurrences
        let size_assert_count = result_str.matches("std::mem::size_of").count();
        let align_assert_count = result_str.matches("std::mem::align_of").count();
        
        // We expect exactly 2 assertions: one for parameter, one for return type
        // Each assertion has both size and alignment checks, so 4 total checks
        assert_eq!(size_assert_count, 4, "Expected exactly 4 size assertions (2 pairs)");
        assert_eq!(align_assert_count, 4, "Expected exactly 4 alignment assertions (2 pairs)");
        
        // Should have stripped the wrapper in the FFI signature (parameter should be *const ExportedType, not *const MaybeUninit<ExportedType>)
        assert!(result_str.contains("*const ExportedType"));
        assert!(!result_str.contains("*const std :: mem :: MaybeUninit"));
    }

    #[test]
    fn test_convert_to_local_type_function() {
        let mut exported_types = HashSet::new();
        exported_types.insert("ExportedType".to_string());
        
        let transparent_wrappers = vec![
            syn::parse_quote! { std::mem::MaybeUninit },
        ];
        let source_crate_name = "test_crate";

        // Test with transparent wrapper + exported type
        let wrapped_type: syn::Type = syn::parse_quote! { std::mem::MaybeUninit<ExportedType> };
        let mut assertion_pairs = HashSet::new();
        let result = convert_to_local_type(&wrapped_type, &exported_types, &transparent_wrappers, source_crate_name, &mut assertion_pairs);
        
        let local_str = quote::quote! { #result }.to_string();
        assert_eq!(local_str, "ExportedType"); // Stripped of wrapper
        
        // Should have collected an assertion pair
        assert_eq!(assertion_pairs.len(), 1);
        let (local_type_str, original_type_str) = assertion_pairs.iter().next().unwrap();
        assert_eq!(local_type_str, "ExportedType");
        assert!(original_type_str.contains("std :: mem :: MaybeUninit < test_crate :: ExportedType >"));
        
        // Test with regular type that doesn't need conversion
        let regular_type: syn::Type = syn::parse_quote! { i32 };
        let mut assertion_pairs = HashSet::new();
        let result = convert_to_local_type(&regular_type, &exported_types, &transparent_wrappers, source_crate_name, &mut assertion_pairs);
        
        let result_str = quote::quote! { #result }.to_string();
        assert_eq!(result_str, "i32"); // No change
        assert_eq!(assertion_pairs.len(), 0); // No assertion pairs collected
        
        // Test with exported type but no wrapper
        let exported_only: syn::Type = syn::parse_quote! { ExportedType };
        let mut assertion_pairs = HashSet::new();
        let result = convert_to_local_type(&exported_only, &exported_types, &transparent_wrappers, source_crate_name, &mut assertion_pairs);
        
        let local_str = quote::quote! { #result }.to_string();
        assert_eq!(local_str, "ExportedType"); // No change since no wrapper
        
        // Should have collected an assertion pair
        assert_eq!(assertion_pairs.len(), 1);
        let (local_type_str, original_type_str) = assertion_pairs.iter().next().unwrap();
        assert_eq!(local_type_str, "ExportedType");
        assert_eq!(original_type_str, "test_crate :: ExportedType");
    }
}
