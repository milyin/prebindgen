//! # prebindgen
//! 
//! A proc-macro crate that provides the `#[prebindgen]` attribute macro for copying 
//! struct and enum definitions to a file during compilation.
//!
//! ## Usage
//!
//! ```rust
//! use prebindgen::prebindgen;
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

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};
use std::env;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::process;

/// Attribute macro that copies the annotated struct or enum definition to prebindgen.rs in OUT_DIR
#[proc_macro_attribute]
pub fn prebindgen(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input_clone = input.clone();
    let parsed = parse_macro_input!(input as DeriveInput);
    
    // Get the destination directory - prefer OUT_DIR, fallback to temp with unique path
    let dest_dir = if let Ok(out_dir) = env::var("OUT_DIR") {
        out_dir
    } else {
        // Create a unique directory in temp for this session
        create_unique_temp_dir()
    };
    
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
    if !existing_content.contains(&definition_name) {
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

/// Creates a unique temporary directory for prebindgen when OUT_DIR is not available
fn create_unique_temp_dir() -> String {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    
    let temp_dir = env::temp_dir();
    let pid = process::id();
    let counter = COUNTER.fetch_add(1, Ordering::SeqCst);
    let unique_name = format!("prebindgen_{}_{}_{}", pid, counter, 
                             std::time::SystemTime::now()
                                 .duration_since(std::time::UNIX_EPOCH)
                                 .unwrap_or_default()
                                 .as_millis());
    
    let unique_dir = temp_dir.join(unique_name);
    
    // Create the directory if it doesn't exist
    if std::fs::create_dir_all(&unique_dir).is_err() {
        // Fallback to just temp dir if we can't create the unique one
        return temp_dir.to_string_lossy().to_string();
    }
    
    unique_dir.to_string_lossy().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;
    
    // Static mutex to ensure tests don't interfere with each other when modifying env vars
    static ENV_MUTEX: Mutex<()> = Mutex::new(());
    
    #[test]
    fn test_macro_exists() {
        // This is just a placeholder test to ensure the macro compiles
        let _test = true;
        assert!(_test);
    }
    
    #[test]
    fn test_unique_temp_dir_creation() {
        let _lock = ENV_MUTEX.lock().unwrap();
        
        // Test that create_unique_temp_dir creates different paths
        let dir1 = create_unique_temp_dir();
        let dir2 = create_unique_temp_dir();
        
        assert_ne!(dir1, dir2, "Unique temp directories should be different");
        assert!(dir1.contains("prebindgen_"), "Directory should contain prebindgen prefix");
        assert!(dir2.contains("prebindgen_"), "Directory should contain prebindgen prefix");
        
        // Verify the directories exist or can be created
        let path1 = std::path::Path::new(&dir1);
        let path2 = std::path::Path::new(&dir2);
        
        assert!(path1.exists() || std::fs::create_dir_all(path1).is_ok());
        assert!(path2.exists() || std::fs::create_dir_all(path2).is_ok());
    }
}
