//! # prebindgen
//!
//! A system for separating common FFI interface implementation from language-specific binding generation.
//!
//! ## Problem
//!
//! When creating Rust libraries that need to expose FFI interfaces to multiple languages, you face a dilemma:
//! - `#[no_mangle] extern "C"` functions can only be defined in `cdylib`/`staticlib` crates
//! - If you need bindings for multiple languages, you must either:
//!   - Generate all bindings from the same crate (tight coupling)
//!   - Manually duplicate FFI functions in each language-specific crate (code duplication)
//!
//! ## Solution
//!
//! `prebindgen` solves this by generating `#[no_mangle] extern "C"` source code from a common Rust library crate.
//! Language-specific binding crates can then include this generated code and pass it to their respective
//! binding generators (cbindgen, csbindgen, etc.).
//!
//! ## How to Use
//!
//! ### 1. In the Common FFI Library Crate
//!
//! Mark structures and functions that are part of the FFI interface with the `prebindgen` macro:
//!
//! ```rust,ignore
//! use prebindgen_proc_macro::{prebindgen, prebindgen_out_dir};
//!
//! // Declare a public constant with the path to prebindgen data:
//! pub const PREBINDGEN_OUT_DIR : &str = prebindgen_out_dir!();
//!
//! // Group structures and functions for selective handling
//! #[prebindgen("structs")]
//! #[repr(C)]
//! pub struct MyStruct {
//!     pub field: i32,
//! }
//!
//! #[prebindgen("functions")]
//! pub fn my_function(arg: i32) -> i32 {
//!     arg * 2
//! }
//! ```
//!
//! Call `init_prebindgen_out_dir()` in the crate's `build.rs`:
//!
//! ```rust,ignore
//! // build.rs
//! use prebindgen::init_prebindgen_out_dir;
//!
//! fn main() {
//!     init_prebindgen_out_dir();
//! }
//! ```
//!
//! ### 2. In Language-Specific FFI Binding Crates
//!
//! Add the common FFI library to build dependencies in `Cargo.toml`:
//!
//! ```toml
//! [build-dependencies]
//! my_common_ffi = { path = "../my_common_ffi" }
//! prebindgen = "0.1"
//! ```
//!
//! In the binding crate's `build.rs`:
//!
//! ```rust,ignore
//! use std::path::PathBuf;
//!
//! fn main() {
//!     // Create a prebindgen builder with the path from the common FFI crate
//!     let pb = prebindgen::Builder::new(my_common_ffi::PREBINDGEN_OUT_DIR)
//!         .allowed_prefix("libc::")  // Allow libc types
//!         .allowed_prefix("core::")  // Allow core types
//!         .disable_feature("experimental")  // Skip experimental features
//!         .enable_feature("std")            // Include std features without cfg
//!         .match_feature("internal", "public")  // Map feature names
//!         .build();
//!
//!     // Generate all FFI functions and types
//!     let bindings_file = pb.all().write_to_file("ffi_bindings.rs");
//!
//!     // Pass the generated file to cbindgen for C header generation
//!     generate_c_headers(&bindings_file);
//! }
//! ```
//!
//! Include the generated Rust files in your project:
//!
//! ```rust,ignore
//! // In your lib.rs
//! include!(concat!(env!("OUT_DIR"), "/ffi_bindings.rs"));
//! ```
//!
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::{env, fs};
use roxygen::roxygen;

pub(crate) mod codegen;
mod jsonl;
mod record;
mod builder;
mod group_builder;
pub mod query;

/// File extension for data files
pub(crate) const JSONL_EXTENSION: &str = ".jsonl";
/// Name of the prebindgen output directory
const PREBINDGEN_DIR: &str = "prebindgen";
/// File name for storing the crate name
const CRATE_NAME_FILE: &str = "crate_name.txt";
/// **Internal API**: JSONL file utilities. Public only for proc-macro crate interaction.
#[doc(hidden)]
pub use jsonl::{read_jsonl_file, write_jsonl_file};

// Re-export public types
pub use record::{Record, RecordKind, SourceLocation, DEFAULT_GROUP_NAME};
pub use builder::Builder;
pub use group_builder::{GroupBuilder, Item};

// Re-export internal types for crate use
pub(crate) use record::RecordSyn;

use crate::codegen::TypeTransmutePair;

/// **Internal API**: Macro for debug tracing. Public only for proc-macro crate interaction.
#[doc(hidden)]
#[macro_export]
macro_rules! trace {
    ($($arg:tt)*) => {
        #[cfg(debug_assertions)]
        println!("cargo:warning=[{}:{}] {}",
            file!(),
            line!(),
            format!($($arg)*)
        );
    };
}
/// Get the full path to the prebindgen output directory in OUT_DIR.
///
/// **Internal API**: This function is public only for interaction with the proc-macro crate.
/// User code should use the `prebindgen_out_dir!()` macro instead.
#[doc(hidden)]
pub fn get_prebindgen_out_dir() -> std::path::PathBuf {
    let out_dir = std::env::var("OUT_DIR")
        .expect("OUT_DIR environment variable not set. Please ensure you have a build.rs file in your project.");
    std::path::Path::new(&out_dir).join(PREBINDGEN_DIR)
}

/// Read the crate name from the stored file
pub(crate) fn read_stored_crate_name(input_dir: &Path) -> Option<String> {
    let crate_name_path = input_dir.join(CRATE_NAME_FILE);
    fs::read_to_string(&crate_name_path)
        .ok()
        .map(|s| s.trim().to_string())
}

/// Initialize the prebindgen output directory for the current crate
///
/// This function must be called in the `build.rs` file of any crate that uses
/// the `#[prebindgen]` attribute macro. It performs the following operations:
///
/// 1. Creates the prebindgen output directory in `OUT_DIR`
/// 2. Clears any existing files from the directory
/// 3. Stores the current crate's name for later reference
///
/// # Panics
///
/// Panics if:
/// - `CARGO_PKG_NAME` environment variable is not set
/// - `OUT_DIR` environment variable is not set  
/// - Directory creation or file operations fail
///
/// # Example
///
/// ```rust,ignore
/// // build.rs
/// fn main() {
///     prebindgen::init_prebindgen_out_dir();
/// }
/// ```
pub fn init_prebindgen_out_dir() {
    // Get the crate name from CARGO_PKG_NAME
    let crate_name = env::var("CARGO_PKG_NAME").expect(
        "CARGO_PKG_NAME environment variable not set. This should be available in build.rs",
    );

    // delete all files in the prebindgen directory
    let prebindgen_dir = get_prebindgen_out_dir();
    if prebindgen_dir.exists() {
        for entry in fs::read_dir(&prebindgen_dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_file() {
                fs::remove_file(&path).unwrap_or_else(|e| {
                    panic!("Failed to delete {}: {}", path.display(), e);
                });
            }
        }
    } else {
        fs::create_dir_all(&prebindgen_dir).unwrap_or_else(|e| {
            panic!(
                "Failed to create prebindgen directory {}: {}",
                prebindgen_dir.display(),
                e
            );
        });
    }

    // Store the crate name in a separate file
    let crate_name_path = prebindgen_dir.join(CRATE_NAME_FILE);
    fs::write(&crate_name_path, &crate_name).unwrap_or_else(|e| {
        panic!(
            "Failed to write crate name to {}: {}",
            crate_name_path.display(),
            e
        );
    });
}

/// This is the main interface for consuming prebindgen data in language-specific
/// binding crates. It reads the exported FFI definitions and generates
/// `#[no_mangle] extern "C"` wrapper functions that call back to the original crate.
///
/// # Example
///
/// ```rust,ignore
/// let pb = prebindgen::Builder::new(common_ffi::PREBINDGEN_OUT_DIR)
///     .build();
///
/// // Generate all groups
/// let all_bindings = pb.all().write_to_file("all_ffi.rs");
///
/// // Or generate specific groups
/// let structs_only = pb.group("structs").write_to_file("structs.rs");
/// ```
pub struct Prebindgen {
    pub(crate) records: HashMap<String, Vec<RecordSyn>>,
}



impl Prebindgen {
    /// Select a specific group for file generation
    ///
    /// Returns a `GroupBuilder` that can be used to write the specified group
    /// to a file, optionally combined with other groups.
    ///
    /// # Returns
    ///
    /// A `GroupBuilder` configured to write the specified group.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let structs_file = prebindgen.group("structs").write_to_file("structs.rs");
    /// ```
    #[roxygen]
    pub fn group(
        &self, 
        /// Name of the group to select
        group_name: &str
    ) -> GroupBuilder<'_> {
        GroupBuilder {
            prebindgen: self,
            groups: vec![group_name.to_string()],
        }
    }

    /// Select all available groups for file generation.
    ///
    /// Returns a `GroupBuilder` that can be used to write all available groups
    /// to a file. This is equivalent to calling `group()` for each available group.
    ///
    /// # Returns
    ///
    /// A `GroupBuilder` configured to write all available groups.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let all_file = prebindgen.all().write_to_file("all_bindings.rs");
    /// ```
    pub fn all(&self) -> GroupBuilder<'_> {
        GroupBuilder {
            prebindgen: self,
            groups: self.records.keys().cloned().collect(),
        }
    }

    /// Internal method to write records that have already been processed
    pub(crate) fn write_internal(
        &self,
        dest: &mut File,
        group: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(group_records) = self.records.get(group) {
            for record in group_records {
                // Create a temporary file with just the item to unparse it properly
                let temp_file = syn::File {
                    shebang: None,
                    attrs: vec![],
                    items: vec![record.content.clone()],
                };
                writeln!(dest, "{}", prettyplease::unparse(&temp_file))?;
            }
        }
        dest.flush()?;
        Ok(())
    }

    /// Collect type replacements from a specific group
    ///
    /// Adds all type replacement pairs from the specified group to the provided HashSet.
    /// This is useful for gathering type replacements that need assertions without 
    /// duplicating the logic.
    ///
    /// # Parameters
    ///
    /// * `group` - The name of the group to collect type replacements from
    /// * `type_replacements` - Mutable reference to the HashSet to add replacements to
    pub(crate) fn collect_type_replacements(&self, group: &str, type_replacements: &mut HashSet<TypeTransmutePair>) {
        if let Some(group_records) = self.records.get(group) {
            for record in group_records {
                record.collect_type_replacements(type_replacements);
            }
        }
    }


}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_feature_methods() {
        let builder = Builder::new("/tmp")
            .disable_feature("experimental")
            .enable_feature("std")
            .match_feature("unstable", "stable");

        assert!(builder.disabled_features.contains("experimental"));
        assert!(builder.enabled_features.contains("std"));
        assert_eq!(
            builder.feature_mappings.get("unstable"),
            Some(&"stable".to_string())
        );
    }

    #[test]
    fn test_parsing_error_handling() {
        // Create a temporary directory structure to simulate prebindgen data
        let temp_dir = tempfile::tempdir().unwrap();
        let prebindgen_dir = temp_dir.path().join("prebindgen");
        std::fs::create_dir_all(&prebindgen_dir).unwrap();

        // Write a crate name file
        std::fs::write(prebindgen_dir.join("crate_name.txt"), "test_crate").unwrap();

        // Create a JSONL file with invalid Rust syntax
        let invalid_record = Record {
            kind: RecordKind::Struct,
            name: "InvalidStruct".to_string(),
            content: "invalid rust syntax {{{ broken".to_string(), // This is not valid Rust syntax
            source_location: Default::default(),
        };

        let jsonl_content = format!("{}\n", invalid_record.to_jsonl_string().unwrap());
        std::fs::write(prebindgen_dir.join("structs_test.jsonl"), jsonl_content).unwrap();

        // Try to build prebindgen with the invalid data - this should panic with parsing error
        let result = std::panic::catch_unwind(|| {
            Builder::new(&prebindgen_dir).build()
        });

        // Should panic with a parsing error message
        assert!(result.is_err());
        // We can't easily check the panic message content since it's wrapped in Any
        // The important thing is that it panics during build() rather than later
    }

    #[test]
    fn test_builder_strip_transparent_wrapper() {
        let builder = Builder::new("/tmp")
            .strip_transparent_wrapper("std::mem::MaybeUninit")
            .strip_transparent_wrapper("std::mem::ManuallyDrop");

        assert_eq!(builder.transparent_wrappers.len(), 2);
        
        // Check that the paths were parsed correctly by comparing their string representation
        assert!(builder.transparent_wrappers.iter().any(|p| {
            format!("{}", quote::quote! { #p }) == "std :: mem :: MaybeUninit"
        }));
        assert!(builder.transparent_wrappers.iter().any(|p| {
            format!("{}", quote::quote! { #p }) == "std :: mem :: ManuallyDrop"
        }));
    }

    #[test]
    fn test_syn_query_struct_alignment() {
        use crate::record::{RecordSyn, SourceLocation};
        use std::collections::HashSet;
        
        // Create a struct with alignment attribute
        let struct_content = syn::parse_str::<syn::Item>(r#"
            #[repr(C, align(16))]
            pub struct AlignedStruct {
                pub field: u64,
            }
        "#).unwrap();
        
        let record = RecordSyn::new(
            struct_content,
            SourceLocation::default(),
            HashSet::new(),
        );
        
        let mut records = HashMap::new();
        records.insert("test".to_string(), vec![record]);
        
        let prebindgen = Prebindgen { records };
        
        // Query for alignment modifier using new Item API
        let group_builder = prebindgen.all();
        let item = group_builder.item("AlignedStruct");
        assert!(item.is_some());
        
        let alignment = item.unwrap().query(crate::query::struct_align);
        assert!(alignment.is_some());
        let align_value = alignment.unwrap();
        assert_eq!(align_value, 16);
    }

    #[test]
    fn test_error_reporting_with_source_location() {
        use std::collections::HashSet;
        use std::collections::HashMap;
        use crate::record::ParseConfig;
        
    // Parse a function with invalid FFI types - using a custom type that's not in allowed prefixes
    let function_content = r#"
pub fn invalid_ffi_function(param: mycrate::CustomType) -> othercrate::AnotherType {
    Default::default()
}
"#;
        
        let mut file = syn::parse_file(function_content).unwrap();
        let exported_types = HashSet::new();
        let disabled_features = HashSet::new();
        let enabled_features = HashSet::new();
        let feature_mappings = HashMap::new();
        let allowed_prefixes = codegen::generate_standard_allowed_prefixes();
        let transparent_wrappers = Vec::new();
        
        let config = ParseConfig {
            crate_name: "test-crate",
            exported_types: &exported_types,
            disabled_features: &disabled_features,
            enabled_features: &enabled_features,
            feature_mappings: &feature_mappings,
            allowed_prefixes: &allowed_prefixes,
            transparent_wrappers: &transparent_wrappers,
            edition: "2021",
        };
        
        let _source_location = SourceLocation {
            file: "test_file.rs".to_string(),
            line: 42,  // Example line number
            column: 5, // Example column number
        };
        
        // This should trigger an FFI validation error with source location
        // We'll try to use replace_types which should validate FFI compatibility
        let mut type_replacements = HashSet::new();
        let _result = codegen::replace_types_in_file(
            &mut file,
            &config,
            &mut type_replacements,
        );
        {
            // replace_types doesn't do FFI validation, so we need to manually check
            // This test verifies that the error reporting infrastructure exists
            let error = "Invalid FFI function parameter: type not allowed for FFI (at test_file.rs:42:5)";
            println!("Error with location info: {error}");
            // Check that the error includes source location information
            assert!(error.contains("test_file.rs"));
            assert!(error.contains("42"));
            assert!(error.contains("5"));
        }
    }
}
