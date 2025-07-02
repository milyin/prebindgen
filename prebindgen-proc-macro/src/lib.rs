use prebindgen::{Record, RecordKind, get_prebindgen_json_path};
use proc_macro::TokenStream;
use quote::quote;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use syn::{DeriveInput, parse_macro_input};

/// Attribute macro that copies the annotated struct or enum definition in the "source" ffi crate to prebindgen.json in OUT_DIR
#[proc_macro_attribute]
pub fn prebindgen(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input_clone = input.clone();
    let parsed = parse_macro_input!(input as DeriveInput);

    // Get the full path to the prebindgen.json file
    let file_path = get_prebindgen_json_path();
    let dest_path = Path::new(&file_path);

    // Determine the record kind
    let kind = match &parsed.data {
        syn::Data::Struct(_) => RecordKind::Struct,
        syn::Data::Enum(_) => RecordKind::Enum,
        syn::Data::Union(_) => RecordKind::Union,
    };

    // Convert the parsed input back to tokens for storing content
    let tokens = quote! { #parsed };
    let content = tokens.to_string();

    // Create the new record
    let new_record = Record::new(kind, parsed.ident.to_string(), content);

    // Convert record to JSON and append to file
    if let Ok(json_content) = serde_json::to_string(&new_record) {
        if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(dest_path) {
            // Write with leading comma to append to the JSON array
            let _ = write!(file, ",{}", json_content);
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
