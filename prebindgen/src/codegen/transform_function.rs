//! Function transformation utilities for converting Rust functions into FFI stubs.
//!
//! This module contains the logic for:
//! - Parsing and validating function signatures for FFI compatibility
//! - Generating `#[no_mangle] extern "C"` wrapper functions
//! - Handling type transformations and validations
//! - Creating appropriate parameter names and call arguments

use roxygen::roxygen;

/// Remove the function body from a function definition
///
/// This function takes a parsed function and removes its implementation,
/// leaving only the function signature. The body is replaced with an empty block.
#[roxygen]
pub fn trim_implementation(
    /// The function to trim
    mut function: syn::ItemFn,
) -> syn::ItemFn {
    // Replace the function body with an empty block
    function.block = Box::new(syn::parse_quote! { {} });
    function
}

/// Create a stub implementation for a function with transmutes applied
///
/// This function takes a function signature and a replacement map, then creates
/// a new function body that applies transmutes to types specified in the map.
/// The replacement map maps from local type strings to target type strings.
#[roxygen]
pub fn create_stub_implementation(
    /// The function to create a stub implementation for
    mut function: syn::ItemFn,
    /// The source crate identifier for calling the original function
    source_crate_ident: &syn::Ident,
) -> Result<syn::ItemFn, String> {
    let function_name = &function.sig.ident;
    
    // Build call arguments with transmutes where needed
    let mut call_args = Vec::new();
    for input in &function.sig.inputs {
        let syn::FnArg::Typed(pat_type) = input else {
            return Err("FFI functions cannot have receiver arguments (like 'self')".to_string());
        };
        
        if let syn::Pat::Ident(pat_ident) = &*pat_type.pat {
            let param_name = &pat_ident.ident;
            call_args.push(quote::quote! { unsafe { std::mem::transmute(#param_name) } });
        }
    }
    
    let has_return_type = !matches!(&function.sig.output, syn::ReturnType::Default);

    // Generate the function body
    let function_body = if has_return_type {
        quote::quote! {
            let result = #source_crate_ident::#function_name(#(#call_args),*);
            unsafe { std::mem::transmute(result) }
        }
    } else {
        // Direct call without transmute
        quote::quote! {
            #source_crate_ident::#function_name(#(#call_args),*)
        }
    };
    
    // Create the function body block
    let body: syn::Block = syn::parse_quote! {
        {
            #function_body
        }
    };
    
    // Update function body and return the modified function
    function.block = Box::new(body);
    Ok(function)
}
