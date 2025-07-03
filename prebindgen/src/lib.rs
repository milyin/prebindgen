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
//! Call `init_prebindgen()` in the crate's `build.rs`:
//!
//! ```rust,ignore
//! // build.rs
//! use prebindgen::init_prebindgen;
//!
//! fn main() {
//!     init_prebindgen();
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
//! use prebindgen::Prebindgen;
//! use std::path::Path;
//!
//! fn main() {
//!     // Create prebindgen context with path to generated data
//!     let mut pb = Prebindgen::new(Path::new(my_common_ffi::PREBINDGEN_OUT_DIR))
//!         .crate_name("my_common_ffi")
//!         .edition("2024")
//!         .build();
//!
//!     // Read the prebindgen data by group
//!     pb.read("structs");
//!     pb.read("functions");
//!     
//!     // Or read all available groups at once
//!     // pb.read_all();
//!
//!     // Generate Rust FFI code files
//!     let structs_file = pb.create("ffi_structs.rs").append("structs");
//!     let functions_file = pb.create("ffi_functions.rs").append("functions");
//!
//!     // Alternative: Write all groups to a single file
//!     // let combined_file = pb.create("ffi_bindings.rs").append_all();
//!     // println!("Generated file: {}", combined_file.get_path().display());
//!
//!     // Now pass the generated files to your binding generator:
//!     // - cbindgen for C/C++
//!     // - csbindgen for C#
//!     // - etc.
//! }
//! ```
//!
//! Include the generated Rust files in your project:
//!
//! ```rust,ignore
//! // In src/lib.rs or src/main.rs
//! include!(concat!(env!("OUT_DIR"), "/ffi_structs.rs"));
//! include!(concat!(env!("OUT_DIR"), "/ffi_functions.rs"));
//!
//! // Or if using a single combined file:
//! // include!(concat!(env!("OUT_DIR"), "/ffi_bindings.rs"));
//! ```
//!
//! ## Benefits
//!
//! - **Separation of concerns**: Common FFI interface logic stays in one place
//! - **Multiple language support**: Generate bindings for different languages from separate crates
//! - **Code reuse**: No duplication of FFI function implementations
//! - **Flexibility**: Group definitions by functionality for selective inclusion
//! - **Tool integration**: Generated code works with existing binding generators
//!
//! ## Core API
//!
//! - [`Prebindgen`]: Main struct for reading exported definitions and generating FFI code
//!   - [`Prebindgen::new()`]: Create a new PrebindgenBuilder with the input directory path
//!   - [`Prebindgen::read()`]: Read exported definitions for a group
//!   - [`Prebindgen::read_all()`]: Read exported definitions for all available groups
//!   - [`Prebindgen::create()`]: Create a new file for writing groups, returns a FileBuilder
//! - [`PrebindgenBuilder`]: Builder for configuring Prebindgen with optional parameters
//!   - [`PrebindgenBuilder::crate_name()`]: Set the source crate name (optional)
//!   - [`PrebindgenBuilder::edition()`]: Set the Rust edition (optional, defaults to "2021")
//!   - [`PrebindgenBuilder::build()`]: Build the configured Prebindgen instance
//! - [`FileBuilder`]: Builder for appending groups to a file
//!   - [`FileBuilder::append()`]: Append a specific group to the file
//!   - [`FileBuilder::append_all()`]: Append all loaded groups to the file
//!   - [`FileBuilder::get_path()`]: Get the absolute path to the generated file
//!   - [`FileBuilder::into_path()`]: Convert the builder to a PathBuf
//! - [`Record`]: Represents a single exported definition (struct, enum, union, or function)
//! - [`RecordKind`]: Enum indicating the type of definition
//! - [`init_prebindgen()`]: Utility to initialize the prebindgen system in build scripts
//!
//! ## Basic API Example
//!
//! ```rust
//! use prebindgen::{Record, RecordKind};
//!
//! // Create a record representing a struct definition
//! let record = Record::new(
//!     RecordKind::Struct,
//!     "MyStruct".to_string(),
//!     "#[repr(C)] pub struct MyStruct { pub field: i32 }".to_string()
//! );
//!
//! assert_eq!(record.kind, RecordKind::Struct);
//! assert_eq!(record.name, "MyStruct");
//! ```

use core::panic;
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::{env, fs};

mod jsonl;
pub use jsonl::{read_jsonl_file, write_jsonl_file};

/// File extension for data files
const JSONL_EXTENSION: &str = ".jsonl";
/// Name of the prebindgen output directory
const PREBINDGEN_DIR: &str = "prebindgen";

/// Represents a record of a struct, enum, union, or function definition
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Record {
    /// The kind of definition (struct, enum, or union)
    pub kind: RecordKind,
    /// The name of the type
    pub name: String,
    /// The full source code content of the definition
    pub content: String,
}

/// The kind of record (struct, enum, union, or function)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum RecordKind {
    /// A struct definition
    Struct,
    /// An enum definition
    Enum,
    /// A union definition
    Union,
    /// A function definition
    Function,
}

impl Record {
    /// Create a new record
    pub fn new(kind: RecordKind, name: String, content: String) -> Self {
        Self {
            kind,
            name,
            content,
        }
    }

    /// Serialize this record to a JSON-lines compatible string
    pub fn to_jsonl_string(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

impl std::fmt::Display for RecordKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RecordKind::Struct => write!(f, "struct"),
            RecordKind::Enum => write!(f, "enum"),
            RecordKind::Union => write!(f, "union"),
            RecordKind::Function => write!(f, "function"),
        }
    }
}

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
/// This directory contains generated data files with exported definitions.
pub fn get_prebindgen_out_dir() -> std::path::PathBuf {
    let out_dir = std::env::var("OUT_DIR")
        .expect("OUT_DIR environment variable not set. Please ensure you have a build.rs file in your project.");
    std::path::Path::new(&out_dir).join(PREBINDGEN_DIR)
}

pub fn init_prebindgen() {
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
}

/// Helper for reading exported records and generating Rust bindings per group
pub struct Prebindgen {
    records: std::collections::HashMap<String, Vec<Record>>,
    defined_types: std::collections::HashSet<String>,
    input_dir: std::path::PathBuf,
    crate_name: String,
    edition: String,
}

/// Builder for configuring Prebindgen with optional parameters
pub struct PrebindgenBuilder {
    input_dir: std::path::PathBuf,
    crate_name: Option<String>,
    edition: Option<String>,
}

/// Builder for writing groups to files with append capability
pub struct FileBuilder<'a> {
    prebindgen: &'a Prebindgen,
    file_path: std::path::PathBuf,
}

impl Prebindgen {
    /// Create a new Prebindgen context with the specified input directory
    /// 
    /// # Parameters
    /// - `input_dir`: Path to the directory containing prebindgen data files
    /// 
    /// # Returns
    /// A PrebindgenBuilder for configuring optional parameters
    /// 
    /// # Example
    /// ```rust,ignore
    /// let pb = Prebindgen::new(Path::new("/path/to/prebindgen/data"))
    ///     .crate_name("my_crate")
    ///     .edition("2024")
    ///     .build();
    /// ```
    pub fn new<P: AsRef<Path>>(input_dir: P) -> PrebindgenBuilder {
        PrebindgenBuilder {
            input_dir: input_dir.as_ref().to_path_buf(),
            crate_name: None,
            edition: None,
        }
    }

    /// Read all exported files matching the group name pattern `<group>_*`, panicking on error with detailed path info
    pub fn read(&mut self, group: &str) {
        let pattern = format!("{}_", group);
        let mut record_map = std::collections::HashMap::new();
        
        // Read the directory and find all matching files
        if let Ok(entries) = fs::read_dir(&self.input_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                    if file_name.starts_with(&pattern) && file_name.ends_with(JSONL_EXTENSION) {
                        trace!("Reading exported file: {}", path.display());
                        let path_clone = path.clone();
                        
                        match jsonl::read_jsonl_file(&path) {
                            Ok(records) => {
                                for record in records {
                                    // Use HashMap to deduplicate records by name
                                    record_map.insert(record.name.clone(), record);
                                }
                            }
                            Err(e) => {
                                panic!("Failed to read {}: {}", path_clone.display(), e);
                            }
                        }
                    }
                }
            }
        }
        
        // Convert map values to vector
        let all_records: Vec<Record> = record_map.values().cloned().collect();
        
        // Store the deduplicated records for this group
        self.records.insert(group.to_string(), all_records.clone());
        
        // Update defined_types with all types from all groups
        for record in &all_records {
            if matches!(record.kind, RecordKind::Struct | RecordKind::Enum | RecordKind::Union) {
                self.defined_types.insert(record.name.clone());
            }
        }
    }

    /// Read all exported files for all available groups
    /// 
    /// This method automatically discovers all available groups by scanning for 
    /// `.jsonl` files in the input directory and reads all of them.
    pub fn read_all(&mut self) {
        let mut groups = std::collections::HashSet::new();
        
        // Discover all available groups
        if let Ok(entries) = fs::read_dir(&self.input_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                    if file_name.ends_with(JSONL_EXTENSION) {
                        // Extract group name from filename (everything before the first underscore)
                        if let Some(underscore_pos) = file_name.find('_') {
                            let group_name = &file_name[..underscore_pos];
                            groups.insert(group_name.to_string());
                        }
                    }
                }
            }
        }
        
        // Read all discovered groups
        for group in groups {
            self.read(&group);
        }
    }

    /// Create a new file for writing groups
    /// 
    /// This method creates a new file (or overwrites an existing one) and returns
    /// a FileBuilder for writing groups to it.
    /// 
    /// # Parameters
    /// - `file_name`: The name of the file to create in OUT_DIR
    /// 
    /// # Returns
    /// A FileBuilder that allows appending groups to the file
    pub fn create<P: AsRef<Path>>(&self, file_name: P) -> FileBuilder<'_> {
        let out_dir = env::var("OUT_DIR").expect("OUT_DIR environment variable not set. Please ensure you have a build.rs file in your project.");
        let dest_path = PathBuf::from(&out_dir).join(&file_name);
        
        // Create an empty file
        fs::File::create(&dest_path).unwrap_or_else(|e| {
            panic!("Failed to create {}: {}", dest_path.display(), e);
        });
        
        FileBuilder {
            prebindgen: self,
            file_path: dest_path,
        }
    }

    /// Internal method to write records with optional append mode
    fn write_internal<P: AsRef<Path>>(&self, group: &str, file_name: P, append: bool) -> std::path::PathBuf {
        let out_dir = env::var("OUT_DIR").expect("OUT_DIR environment variable not set. Please ensure you have a build.rs file in your project.");
        let dest_path = PathBuf::from(&out_dir).join(&file_name);
        (|| -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
            let mut dest = if append {
                fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&dest_path)?
            } else {
                fs::File::create(&dest_path)?
            };
            
            if let Some(group_records) = self.records.get(group) {
                for record in group_records {
                    match record.kind {
                        RecordKind::Function => {
                            let stub = transform_function_to_stub(
                                &record.content,
                                &self.crate_name,
                                &self.defined_types,
                                &self.edition,
                            )?;
                            writeln!(dest, "{}", stub)?;
                        }
                        _ => {
                            writeln!(dest, "{}", record.content)?;
                        }
                    }
                }
            }
            dest.flush()?;
            Ok(dest_path.clone())
        })()
        .unwrap_or_else(|e| {
            panic!("Failed to generate {}: {}", dest_path.display(), e);
        })
    }
}

impl PrebindgenBuilder {
    /// Set the crate name for the Prebindgen context
    /// 
    /// This is used when generating function stubs to call the original functions.
    /// If not set, defaults to "unknown_crate".
    /// 
    /// # Parameters
    /// - `crate_name`: The name of the source crate containing the original functions
    pub fn crate_name<S: Into<String>>(mut self, crate_name: S) -> Self {
        self.crate_name = Some(crate_name.into());
        self
    }

    /// Set the Rust edition for the Prebindgen context
    /// 
    /// This affects how certain attributes are generated (e.g., #[unsafe(no_mangle)] for 2024 edition).
    /// If not set, defaults to "2021".
    /// 
    /// # Parameters
    /// - `edition`: The Rust edition as a string (e.g., "2021", "2024")
    pub fn edition<E: Into<String>>(mut self, edition: E) -> Self {
        self.edition = Some(edition.into());
        self
    }

    /// Build the Prebindgen instance with the configured parameters
    /// 
    /// # Returns
    /// A configured Prebindgen instance ready for use
    pub fn build(self) -> Prebindgen {
        Prebindgen {
            records: std::collections::HashMap::new(),
            defined_types: std::collections::HashSet::new(),
            input_dir: self.input_dir,
            crate_name: self.crate_name.unwrap_or_else(|| "unknown_crate".to_string()),
            edition: self.edition.unwrap_or_else(|| "2021".to_string()),
        }
    }
}

impl<'a> FileBuilder<'a> {
    /// Append records for a group to the file
    /// 
    /// This method appends records from the specified group to the file
    /// that was created by the `create` method.
    /// 
    /// # Parameters
    /// - `group`: The name of the group to append
    /// 
    /// # Returns
    /// Self for method chaining
    pub fn append(self, group: &str) -> Self {
        // Extract just the filename from the full path
        if let Some(file_name) = self.file_path.file_name() {
            self.prebindgen.write_internal(group, file_name, true);
        }
        self
    }

    /// Append all loaded groups to the file
    /// 
    /// This method appends records from all groups that have been loaded
    /// via `read()` or `read_all()` calls.
    /// 
    /// # Returns
    /// Self for method chaining
    pub fn append_all(self) -> Self {
        if let Some(file_name) = self.file_path.file_name() {
            for group_name in self.prebindgen.records.keys() {
                self.prebindgen.write_internal(group_name, file_name, true);
            }
        }
        self
    }

    /// Get the absolute path to the generated file
    /// 
    /// # Returns
    /// The absolute path to the file that was created
    pub fn get_path(&self) -> &std::path::Path {
        &self.file_path
    }

    /// Converts the FileBuilder to a string representation of the file path
    /// 
    /// # Returns
    /// A path object representing the file path
    pub fn into_path(self) -> std::path::PathBuf {
        self.file_path
    }
}

/// Helper function to check if a type contains any of the defined types
fn contains_defined_type(
    ty: &syn::Type,
    defined_types: &std::collections::HashSet<String>,
) -> bool {
    match ty {
        syn::Type::Path(type_path) => {
            // Check if the type itself is defined
            if let Some(segment) = type_path.path.segments.last() {
                let type_name = segment.ident.to_string();
                if defined_types.contains(&type_name) {
                    return true;
                }

                // Check generic arguments recursively
                if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                    for arg in &args.args {
                        if let syn::GenericArgument::Type(inner_ty) = arg {
                            if contains_defined_type(inner_ty, defined_types) {
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
            contains_defined_type(&type_ref.elem, defined_types)
        }
        syn::Type::Ptr(type_ptr) => {
            // Check the pointed-to type
            contains_defined_type(&type_ptr.elem, defined_types)
        }
        syn::Type::Slice(type_slice) => {
            // Check the slice element type
            contains_defined_type(&type_slice.elem, defined_types)
        }
        syn::Type::Array(type_array) => {
            // Check the array element type
            contains_defined_type(&type_array.elem, defined_types)
        }
        syn::Type::Tuple(type_tuple) => {
            // Check all tuple element types
            type_tuple
                .elems
                .iter()
                .any(|elem_ty| contains_defined_type(elem_ty, defined_types))
        }
        _ => false,
    }
}

/// Transform a function prototype to a no_mangle extern "C" function that calls the original function
fn transform_function_to_stub(
    function_content: &str,
    source_crate: &str,
    defined_types: &std::collections::HashSet<String>,
    edition: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    // Helper function to check if a type needs transmute
    let needs_transmute = |ty: &syn::Type| -> bool { contains_defined_type(ty, defined_types) };
    // Parse the function using syn
    let parsed: syn::ItemFn = syn::parse_str(function_content)?;

    // Extract function signature parts
    let fn_name = &parsed.sig.ident;
    let output = &parsed.sig.output;
    let vis = &parsed.vis;

    // Create parameter names for the extern "C" function (with underscore prefix)
    let extern_inputs = parsed.sig.inputs.iter().map(|input| match input {
        syn::FnArg::Typed(pat_type) => {
            if let syn::Pat::Ident(pat_ident) = &*pat_type.pat {
                let new_ident =
                    syn::Ident::new(&format!("_{}", pat_ident.ident), pat_ident.ident.span());
                let mut new_pat_ident = pat_ident.clone();
                new_pat_ident.ident = new_ident;
                let mut new_pat_type = pat_type.clone();
                new_pat_type.pat = Box::new(syn::Pat::Ident(new_pat_ident));
                syn::FnArg::Typed(new_pat_type)
            } else {
                input.clone()
            }
        }
        _ => input.clone(),
    });

    // Extract parameter names for calling the original function
    let call_args = parsed.sig.inputs.iter().filter_map(|input| match input {
        syn::FnArg::Typed(pat_type) => {
            if let syn::Pat::Ident(pat_ident) = &*pat_type.pat {
                let param_name =
                    syn::Ident::new(&format!("_{}", pat_ident.ident), pat_ident.ident.span());
                if needs_transmute(&pat_type.ty) {
                    Some(quote::quote! { unsafe { std::mem::transmute(#param_name) } })
                } else {
                    Some(quote::quote! { #param_name })
                }
            } else {
                None
            }
        }
        _ => None,
    });

    // Create the source crate identifier
    let source_crate_ident = syn::Ident::new(source_crate, proc_macro2::Span::call_site());

    // Generate the function body that calls the original function
    let function_body = match &parsed.sig.output {
        syn::ReturnType::Default => {
            // Void function
            quote::quote! {
                #source_crate_ident::#fn_name(#(#call_args),*);
            }
        }
        syn::ReturnType::Type(_, return_type) => {
            // Function with return value
            if needs_transmute(return_type) {
                quote::quote! {
                    let result = #source_crate_ident::#fn_name(#(#call_args),*);
                    unsafe { std::mem::transmute(result) }
                }
            } else {
                quote::quote! {
                    #source_crate_ident::#fn_name(#(#call_args),*)
                }
            }
        }
    };

    // Determine the correct no_mangle attribute depending on the Rust edition
    let no_mangle_attr = match edition {
        "2024" => "#[unsafe(no_mangle)]",
        _ => "#[no_mangle]",
    };

    // Build the no_mangle extern "C" function
    let stub = format!(
        "{}\n{} unsafe extern \"C\" fn {}({}) {} {{\n{}\n}}",
        no_mangle_attr,
        quote::quote! { #vis },
        fn_name,
        extern_inputs.map(|arg| quote::quote! { #arg }.to_string()).collect::<Vec<_>>().join(", "),
        quote::quote! { #output },
        function_body.to_string().trim()
    );

    Ok(stub)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_builder_pattern_minimal() {
        let pb = Prebindgen::new("/tmp").build();
        assert_eq!(pb.input_dir, Path::new("/tmp"));
        assert_eq!(pb.crate_name, "unknown_crate");
        assert_eq!(pb.edition, "2021");
    }

    #[test]
    fn test_builder_pattern_with_crate_name() {
        let pb = Prebindgen::new("/tmp")
            .crate_name("my_crate")
            .build();
        assert_eq!(pb.input_dir, Path::new("/tmp"));
        assert_eq!(pb.crate_name, "my_crate");
        assert_eq!(pb.edition, "2021");
    }

    #[test]
    fn test_builder_pattern_full_config() {
        let pb = Prebindgen::new("/tmp")
            .crate_name("my_crate")
            .edition("2024")
            .build();
        assert_eq!(pb.input_dir, Path::new("/tmp"));
        assert_eq!(pb.crate_name, "my_crate");
        assert_eq!(pb.edition, "2024");
    }

    #[test]
    fn test_builder_pattern_with_path_ref() {
        let pb = Prebindgen::new(Path::new("/tmp"))
            .crate_name("my_crate")
            .edition("2021")
            .build();
        assert_eq!(pb.input_dir, Path::new("/tmp"));
        assert_eq!(pb.crate_name, "my_crate");
        assert_eq!(pb.edition, "2021");
    }

    #[test]
    fn test_builder_pattern_chaining() {
        let pb = Prebindgen::new("/tmp")
            .crate_name("crate1")
            .crate_name("crate2") // Should override
            .edition("2018")
            .edition("2024") // Should override
            .build();
        assert_eq!(pb.crate_name, "crate2");
        assert_eq!(pb.edition, "2024");
    }
}
