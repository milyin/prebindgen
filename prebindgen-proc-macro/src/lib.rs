use prebindgen::{Record, RecordKind, get_prebindgen_json_path};
use proc_macro::TokenStream;
use quote::quote;
use std::fs::{OpenOptions, metadata};
use std::io::Write;
use std::path::Path;
use syn::{DeriveInput, ItemFn};

/// Attribute macro that copies the annotated struct, enum, union, or function definition in the "source" ffi crate to prebindgen.json in OUT_DIR
#[proc_macro_attribute]
pub fn prebindgen(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input_clone = input.clone();

    // Get the full path to the prebindgen.json file
    let file_path = get_prebindgen_json_path();
    let dest_path = Path::new(&file_path);

    // Try to parse as different item types
    let (kind, name, content) = if let Ok(parsed) = syn::parse::<DeriveInput>(input.clone()) {
        // Handle struct, enum, union
        let kind = match &parsed.data {
            syn::Data::Struct(_) => RecordKind::Struct,
            syn::Data::Enum(_) => RecordKind::Enum,
            syn::Data::Union(_) => RecordKind::Union,
        };
        let tokens = quote! { #parsed };
        (kind, parsed.ident.to_string(), tokens.to_string())
    } else if let Ok(parsed) = syn::parse::<ItemFn>(input.clone()) {
        // Handle function
        // For functions, we want to store only the signature without the body
        let mut fn_sig = parsed.clone();
        fn_sig.block = syn::parse_quote! {{ /* placeholder */ }};
        let tokens = quote! { #fn_sig };
        (
            RecordKind::Function,
            parsed.sig.ident.to_string(),
            tokens.to_string(),
        )
    } else {
        // If we can't parse it, return the original input and skip processing
        return input_clone;
    };

    // Create the new record
    let new_record = Record::new(kind, name, content);

    // Convert record to JSON and append to file
    if let Ok(json_content) = serde_json::to_string(&new_record) {
        if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(dest_path) {
            // Check if file is empty (just created or was deleted)
            let is_empty = metadata(dest_path).map(|m| m.len() == 0).unwrap_or(true);

            if is_empty {
                // Write opening bracket for JSON array
                let _ = write!(file, "[{},", json_content);
            } else {
                // Just append the record with comma
                let _ = write!(file, "{},", json_content);
            }
            let _ = file.flush();
        }
    }

    // Return the original input unchanged
    input_clone
}

/// Proc-macro that returns the prebindgen json file path as a string literal
///
/// It should be used in the "source" ffi crate like this:
/// ```rust,ignore
/// use prebindgen_proc_macro::prebindgen_json_path;
///
/// const PREBINDGEN_JSON: &str = prebindgen_json_path!();
/// ```
///
/// This constant should be passed to `prebindgen_json_to_rs` function in the build.rs
/// of the "destination" ffi crate to generate the no-mangle extern "C" bindings to
/// the generated Rust code.
#[proc_macro]
pub fn prebindgen_json_path(_input: TokenStream) -> TokenStream {
    // Use the helper function to get the file path
    let file_path = get_prebindgen_json_path();
    let file_path = file_path.to_string_lossy();

    // Return just the string literal
    let expanded = quote! {
        #file_path
    };

    TokenStream::from(expanded)
}
