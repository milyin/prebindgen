//! # prebindgen
//! 
//! A proc-macro crate that provides the `#[prebindgen]` attribute macro for copying 
//! struct and enum definitions to a file during compilation, and `prebindgen_path!` 
//! for accessing the destination directory path.
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
//! // Get the prebindgen destination directory as a string constant
//! prebindgen_path!(PREBINDGEN_DIR);
//! 
//! // Now you can use PREBINDGEN_DIR to access the path at runtime
//! fn get_prebindgen_file_path() -> String {
//!     format!("{}/prebindgen.rs", PREBINDGEN_DIR)
//! }
//! ```
//!
//! The macro will copy these definitions to `prebindgen.rs` in your `OUT_DIR` 
//! (when available during build), or to a unique directory in the system temp 
//! directory when `OUT_DIR` is not available.
//!
//! You can then include this file using:
//!
//! ```ignore
//! include!(concat!(env!("OUT_DIR"), "/prebindgen.rs"));
//! ```
//! 
//! Or use the `prebindgen_path!` macro to get the directory path:
//! 
//! ```ignore
//! prebindgen_path!(DEST_DIR);
//! let file_path = format!("{}/prebindgen.rs", DEST_DIR);
//! ```

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Ident};
use std::env;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;
use std::sync::{Mutex, Once};
use std::time::{SystemTime, UNIX_EPOCH};

// Global destination directory path, initialized once
static DEST_DIR_INIT: Once = Once::new();
static DEST_DIR: Mutex<Option<String>> = Mutex::new(None);

/// Get or initialize the global destination directory path
fn get_dest_dir() -> String {
    DEST_DIR_INIT.call_once(|| {
        let dir = if let Ok(out_dir) = env::var("OUT_DIR") {
            out_dir
        } else {
            // Generate a random subpath in temp directory
            let temp_dir = env::temp_dir();
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis();
            let pid = std::process::id();
            let random_suffix = format!("{:x}", timestamp % 0xFFFFFF); // Use last 6 hex digits
            let random_name = format!("prebindgen_{}_{}", pid, random_suffix);
            let fallback_dir = temp_dir.join(random_name);
            let _ = std::fs::create_dir_all(&fallback_dir);
            fallback_dir.to_string_lossy().to_string()
        };
        
        *DEST_DIR.lock().unwrap() = Some(dir);
    });
    
    DEST_DIR.lock().unwrap().as_ref().unwrap().clone()
}

/// Attribute macro that copies the annotated struct or enum definition to prebindgen.rs in OUT_DIR
#[proc_macro_attribute]
pub fn prebindgen(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input_clone = input.clone();
    let parsed = parse_macro_input!(input as DeriveInput);
    
    // Get the global destination directory
    let dest_dir = get_dest_dir();
    let dest_path = Path::new(&dest_dir).join("prebindgen.rs");
    
    // Convert the parsed input back to tokens for writing to file
    let tokens = quote! { #parsed };
    let code = tokens.to_string();
    
    // Read existing content if file exists
    let mut existing_content = String::new();
    if dest_path.exists() {
        if let Ok(mut file) = File::open(&dest_path) {
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
            .open(&dest_path) {
            let _ = writeln!(file, "{}", code);
        }
    }
    
    // Return the original input unchanged
    input_clone
}

/// Proc-macro that generates a constant with the prebindgen destination directory path
/// 
/// Usage:
/// ```rust
/// use prebindgen::prebindgen_path;
/// 
/// prebindgen_path!(PREBINDGEN_DIR);
/// // This generates: const PREBINDGEN_DIR: &str = "/path/to/prebindgen/dir";
/// ```
#[proc_macro]
pub fn prebindgen_path(input: TokenStream) -> TokenStream {
    let const_name = if input.is_empty() {
        quote::format_ident!("PREBINDGEN_PATH")
    } else {
        parse_macro_input!(input as Ident)
    };
    
    // Use the same global destination directory as prebindgen
    let dest_dir = get_dest_dir();
    
    let expanded = quote! {
        pub const #const_name: &str = #dest_dir;
    };
    
    TokenStream::from(expanded)
}

#[cfg(test)]
mod tests {    
    #[test]
    fn test_macro_exists() {
        // This is just a placeholder test to ensure the macro compiles
        let _test = true;
        assert!(_test);
    }
}
