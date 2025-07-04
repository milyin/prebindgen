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


/// Convert reference types to pointer types for FFI compatibility
fn convert_reference_to_pointer(ty: &syn::Type) -> syn::Type {
    match ty {
        syn::Type::Reference(type_ref) => {
            // Convert &T to *const T and &mut T to *mut T
            let mutability = if type_ref.mutability.is_some() {
                syn::token::Mut::default()
            } else {
                return syn::Type::Ptr(syn::TypePtr {
                    star_token: syn::token::Star::default(),
                    const_token: Some(syn::token::Const::default()),
                    mutability: None,
                    elem: type_ref.elem.clone(),
                });
            };
            
            syn::Type::Ptr(syn::TypePtr {
                star_token: syn::token::Star::default(),
                const_token: None,
                mutability: Some(mutability),
                elem: type_ref.elem.clone(),
            })
        }
        _ => ty.clone(),
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
    file: syn::File,
    source_crate: &str,
    exported_types: &HashSet<String>,
    allowed_prefixes: &Vec<syn::Path>,
    edition: &str,
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
    
    let source_crate_name = source_crate.replace('-', "_");
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
    
    // Convert reference parameters to pointer parameters
    for input in extern_sig.inputs.iter_mut() {
        if let syn::FnArg::Typed(pat_type) = input {
            pat_type.ty = Box::new(convert_reference_to_pointer(&pat_type.ty));
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

    // Return a syn::File containing the stub function
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

        let input_file = syn::parse_file(function_content).unwrap();
        let result = transform_function_to_stub(
            input_file,
            "my-crate",
            &exported_types,
            &allowed_prefixes,
            "2021",
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

        let input_file = syn::parse_file(function_content).unwrap();
        let result = transform_function_to_stub(
            input_file,
            "my-crate",
            &exported_types,
            &allowed_prefixes,
            "2024",
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
        
        let result = transform_function_to_stub(
            empty_file,
            "my-crate",
            &exported_types,
            &allowed_prefixes,
            "2021",
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
        let result = transform_function_to_stub(
            multi_item_file,
            "my-crate",
            &exported_types,
            &allowed_prefixes,
            "2021",
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
        
        let struct_file = syn::parse_file(struct_content).unwrap();
        let result = transform_function_to_stub(
            struct_file,
            "my-crate",
            &exported_types,
            &allowed_prefixes,
            "2021",
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

        let input_file = syn::parse_file(function_content).unwrap();
        let result = transform_function_to_stub(
            input_file,
            "my-crate",
            &exported_types,
            &allowed_prefixes,
            "2021",
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
}
