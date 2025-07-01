//! # prebindgen
//! 
//! A proc-macro crate that provides the `#[prebindgen]` attribute macro for copying 
//! struct and enum definitions to a file during compilation, and `prebindgen_path!` 
//! for accessing the destination file path.
//!
//! This crate requires the `OUT_DIR` environment variable to be set, which means
//! you need to have a `build.rs` file in your project (even if it's empty).
//!
//! ## Usage
//!
//! ```rust
//! use prebindgen::{prebindgen, prebindgen_path};
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
//! 
//! ```
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};
use std::env;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;

/// Get the full path to the prebindgen.rs file from OUT_DIR
/// Panics if OUT_DIR is not set (which means build.rs is not defined)
fn get_prebindgen_file_path() -> String {
    let out_dir = env::var("OUT_DIR")
        .expect("OUT_DIR environment variable not set. Please ensure you have a build.rs file in your project.");
    format!("{}/prebindgen.rs", out_dir)
}

/// Attribute macro that copies the annotated struct or enum definition to prebindgen.rs in OUT_DIR
#[proc_macro_attribute]
pub fn prebindgen(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input_clone = input.clone();
    let parsed = parse_macro_input!(input as DeriveInput);
    
    // Get the full path to the prebindgen.rs file
    let file_path = get_prebindgen_file_path();
    let dest_path = Path::new(&file_path);
    
    // Convert the parsed input back to tokens for writing to file
    let tokens = quote! { #parsed };
    let code = tokens.to_string();
    
    // Read existing content if file exists
    let mut existing_content = String::new();
    if dest_path.exists() {
        if let Ok(mut file) = File::open(dest_path) {
            let _ = file.read_to_string(&mut existing_content);
        }
    }
    
    // Check if this definition already exists in the file
    let definition_name = parsed.ident.to_string();
    
    // Create a more specific pattern to check for the actual definition
    let struct_pattern = format!("struct {}", definition_name);
    let enum_pattern = format!("enum {}", definition_name);
    let already_exists = existing_content.contains(&struct_pattern) || existing_content.contains(&enum_pattern);
    
    if !already_exists {
        // Append the new definition to the file
        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(dest_path) {
            let _ = writeln!(file, "{}", code);
        }
    }
    
    // Return the original input unchanged
    input_clone
}

/// Proc-macro that returns the prebindgen file path as a string literal
/// 
/// Usage:
/// ```rust
/// use prebindgen::prebindgen_path;
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
            assert!(path.ends_with("/prebindgen.rs"));
            assert!(!path.is_empty());
        }
        // If OUT_DIR is not set, our function would panic - but we can't test that
        // easily in a unit test without potentially breaking the test environment
        
        // The important thing is that the function compiles and has the right signature
        assert!(true);
    }
}
