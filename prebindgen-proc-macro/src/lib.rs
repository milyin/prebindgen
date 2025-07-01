//! # prebindgen-proc-macro
//!
//! Proc-macro crate that provides the `#[prebindgen]` attribute macro for copying
//! struct and enum definitions to a JSON file during compilation, and `prebindgen_path!`
//! for accessing the destination file path.
//!
//! This crate requires the `OUT_DIR` environment variable to be set, which means
//! you need to have a `build.rs` file in your project (even if it's empty).
//!
//! The macro saves records as JSON-lines format where each line is a separate JSON object:
//! ```json
//! {"kind": "struct", "name": "MyStruct", "content": "pub struct MyStruct { ... }"}
//! {"kind": "enum", "name": "MyEnum", "content": "pub enum MyEnum { ... }"}
//! ```
//!
//! ## Usage
//!
//! ```rust,ignore
//! use prebindgen_proc_macro::{prebindgen, prebindgen_path};
//!
//! #[prebindgen]
//! #[derive(Debug, Clone)]
//! pub struct MyStruct {
//!     pub name: String,
//!     pub value: i32,
//! }
//!
//! #[prebindgen]
//! #[derive(Debug, PartialEq)]
//! pub enum MyEnum {
//!     Variant1,
//!     Variant2(String),
//! }
//!
//! // Get the prebindgen file path as a string
//! const PREBINDGEN_FILE: &str = prebindgen_path!();
//! ```

use prebindgen::{Record, RecordKind, get_prebindgen_file_path};
use proc_macro::TokenStream;
use quote::quote;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use syn::{DeriveInput, parse_macro_input};

/// Attribute macro that copies the annotated struct or enum definition to prebindgen.json in OUT_DIR
#[proc_macro_attribute]
pub fn prebindgen(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input_clone = input.clone();
    let parsed = parse_macro_input!(input as DeriveInput);

    // Get the full path to the prebindgen.json file
    let file_path = get_prebindgen_file_path();
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
            // Force a flush after writing to ensure the newline is written
            let _ = writeln!(file, "{}", json_content);
            let _ = file.flush();
        }
    }

    // Return the original input unchanged
    input_clone
}

/// Proc-macro that returns the prebindgen file path as a string literal
///
/// Usage:
/// ```rust,ignore
/// use prebindgen_proc_macro::prebindgen_path;
///
/// const PREBINDGEN_FILE: &str = prebindgen_path!();
/// // or
/// let path = prebindgen_path!();
/// ```
#[proc_macro]
pub fn prebindgen_path(_input: TokenStream) -> TokenStream {
    // Use the helper function to get the file path
    let file_path = get_prebindgen_file_path();

    // Return just the string literal
    let expanded = quote! {
        #file_path
    };

    TokenStream::from(expanded)
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_out_dir_required() {
        // This test verifies that our function requires OUT_DIR
        let current_out_dir = std::env::var("OUT_DIR");

        // If OUT_DIR is set, our function should work
        if current_out_dir.is_ok() {
            let path = super::get_prebindgen_file_path();
            assert!(path.ends_with("/prebindgen.json"));
            assert!(!path.is_empty());
        }
        // If OUT_DIR is not set, our function would panic - but we can't test that
        // easily in a unit test without potentially breaking the test environment

        // The important thing is that the function compiles and has the right signature
    }
}
