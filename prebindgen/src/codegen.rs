//! Code generation utilities for transforming Rust function definitions into FFI stubs.
//!
//! This module contains all the logic for:
//! - Parsing and validating function signatures for FFI compatibility
//! - Generating `#[no_mangle] extern "C"` wrapper functions
//! - Handling type transformations and validations
//! - Creating appropriate parameter names and call arguments

use std::collections::HashSet;

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
fn validate_type_path(type_path: &syn::TypePath, allowed_prefixes: &Vec<syn::Path>) -> bool {
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
    allowed_prefixes: &Vec<syn::Path>,
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
    allowed_prefixes: &Vec<syn::Path>,
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

/// Generate the appropriate no_mangle attribute based on Rust edition
fn generate_no_mangle_attribute(edition: &str) -> &'static str {
    match edition {
        "2024" => "#[unsafe(no_mangle)]",
        _ => "#[no_mangle]",
    }
}

/// Create parameter names with underscore prefix for extern "C" function
fn create_extern_parameters(inputs: &syn::punctuated::Punctuated<syn::FnArg, syn::token::Comma>) -> Vec<syn::FnArg> {
    inputs.iter().map(|input| match input {
        syn::FnArg::Typed(pat_type) => {
            if let syn::Pat::Ident(pat_ident) = &*pat_type.pat {
                let prefixed_ident = syn::Ident::new(
                    &format!("_{}", pat_ident.ident), 
                    pat_ident.ident.span()
                );
                let mut new_pat_ident = pat_ident.clone();
                new_pat_ident.ident = prefixed_ident;
                let mut new_pat_type = pat_type.clone();
                new_pat_type.pat = Box::new(syn::Pat::Ident(new_pat_ident));
                syn::FnArg::Typed(new_pat_type)
            } else {
                input.clone()
            }
        }
        _ => input.clone(),
    }).collect()
}

/// Generate call arguments with optional transmute for exported types
fn generate_call_arguments(
    inputs: &syn::punctuated::Punctuated<syn::FnArg, syn::token::Comma>,
    exported_types: &HashSet<String>,
) -> Vec<proc_macro2::TokenStream> {
    inputs.iter().filter_map(|input| match input {
        syn::FnArg::Typed(pat_type) => {
            if let syn::Pat::Ident(pat_ident) = &*pat_type.pat {
                let param_name = syn::Ident::new(
                    &format!("_{}", pat_ident.ident), 
                    pat_ident.ident.span()
                );
                
                if contains_exported_type(&pat_type.ty, exported_types) {
                    Some(quote::quote! { unsafe { std::mem::transmute(#param_name) } })
                } else {
                    Some(quote::quote! { #param_name })
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
    allowed_prefixes: &Vec<syn::Path>,
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

/// Transform a function prototype to a no_mangle extern "C" function that calls the original function
pub(crate) fn transform_function_to_stub(
    function_content: &str,
    source_crate: &str,
    exported_types: &HashSet<String>,
    allowed_prefixes: &Vec<syn::Path>,
    edition: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    // Parse the function using syn
    let parsed: syn::ItemFn = syn::parse_str(function_content)?;
    let function_name = &parsed.sig.ident;

    // Validate function signature
    validate_function_parameters(&parsed.sig.inputs, function_name, exported_types, allowed_prefixes)?;
    
    // Validate return type
    if let syn::ReturnType::Type(_, return_type) = &parsed.sig.output {
        validate_type_for_ffi(
            return_type,
            exported_types,
            allowed_prefixes,
            &format!("return type of function '{}'", function_name),
        ).map_err(|e| format!("Invalid FFI function return type: {}", e))?;
    }

    // Generate components
    let extern_inputs = create_extern_parameters(&parsed.sig.inputs);
    let call_args = generate_call_arguments(&parsed.sig.inputs, exported_types);
    
    let source_crate_name = source_crate.replace('-', "_");
    let source_crate_ident = syn::Ident::new(&source_crate_name, proc_macro2::Span::call_site());
    
    let function_body = generate_function_body(
        &parsed.sig.output,
        function_name,
        &source_crate_ident,
        &call_args,
        exported_types,
    );

    // Build the final function string
    let no_mangle_attr = generate_no_mangle_attribute(edition);
    let visibility = &parsed.vis;
    let return_type = &parsed.sig.output;
    
    let extern_params_str = extern_inputs
        .iter()
        .map(|arg| quote::quote! { #arg }.to_string())
        .collect::<Vec<_>>()
        .join(", ");

    let stub = format!(
        "{}\n{} unsafe extern \"C\" fn {}({}) {} {{\n{}\n}}",
        no_mangle_attr,
        quote::quote! { #visibility },
        function_name,
        extern_params_str,
        quote::quote! { #return_type },
        function_body.to_string().trim()
    );

    Ok(stub)
}
