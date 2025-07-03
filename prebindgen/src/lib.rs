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
//! use prebindgen::Builder;
//! use std::path::Path;
//!
//! fn main() {
//!     // Create prebindgen context with selected groups
//!     let pb = Builder::new(Path::new(my_common_ffi::PREBINDGEN_OUT_DIR))
//!         .crate_name("my_common_ffi")
//!         .edition("2024")
//!         .with_group("structs")
//!         .with_group("functions")
//!         .build();
//!
//!     // Or create with all groups (if no with_group calls)
//!     // let pb = Builder::new(Path::new(my_common_ffi::PREBINDGEN_OUT_DIR))
//!     //     .crate_name("my_common_ffi")
//!     //     .edition("2024")
//!     //     .build();
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
/// Name of the file storing the crate name
const CRATE_NAME_FILE: &str = "crate_name.txt";

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

/// Read the crate name from the stored file
fn read_stored_crate_name(input_dir: &Path) -> Option<String> {
    let crate_name_path = input_dir.join(CRATE_NAME_FILE);
    fs::read_to_string(&crate_name_path)
        .ok()
        .map(|s| s.trim().to_string())
}

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

/// Helper for reading exported records and generating Rust bindings per group
pub struct Prebindgen {
    records: std::collections::HashMap<String, Vec<Record>>,
    defined_types: std::collections::HashSet<String>,
    input_dir: std::path::PathBuf,
    crate_name: String,
    edition: String,
}

/// Builder for configuring Prebindgen with optional parameters
pub struct Builder {
    input_dir: std::path::PathBuf,
    crate_name: Option<String>,
    edition: Option<String>,
    selected_groups: Option<std::collections::HashSet<String>>,
}

/// Builder for writing groups to files with append capability
pub struct FileBuilder<'a> {
    prebindgen: &'a Prebindgen,
    file_path: std::path::PathBuf,
}

impl Prebindgen {
    /// Internal method to read all exported files matching the group name pattern `<group>_*`
    fn read_group_internal(&mut self, group: &str) {
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
            if matches!(
                record.kind,
                RecordKind::Struct | RecordKind::Enum | RecordKind::Union
            ) {
                self.defined_types.insert(record.name.clone());
            }
        }
    }

    /// Internal method to read all exported files for all available groups
    fn read_all_groups_internal(&mut self) {
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
            self.read_group_internal(&group);
        }
    }

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
    fn write_internal<P: AsRef<Path>>(
        &self,
        group: &str,
        file_name: P,
        append: bool,
    ) -> std::path::PathBuf {
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

impl Builder {
    pub fn new<P: AsRef<Path>>(input_dir: P) -> Self {
        Self {
            input_dir: input_dir.as_ref().to_path_buf(),
            crate_name: None,
            edition: None,
            selected_groups: None,
        }
    }
    pub fn crate_name<S: Into<String>>(mut self, crate_name: S) -> Self {
        self.crate_name = Some(crate_name.into());
        self
    }
    pub fn edition<E: Into<String>>(mut self, edition: E) -> Self {
        self.edition = Some(edition.into());
        self
    }
    pub fn with_group<S: Into<String>>(mut self, group_name: S) -> Self {
        if self.selected_groups.is_none() {
            self.selected_groups = Some(std::collections::HashSet::new());
        }
        if let Some(ref mut groups) = self.selected_groups {
            groups.insert(group_name.into());
        }
        self
    }
    pub fn build(self) -> Prebindgen {
        // Determine the crate name: use provided one, or read from stored file, or panic if not initialized
        let original_crate_name = read_stored_crate_name(&self.input_dir).unwrap_or_else(|| {
            panic!(
                "The directory {} was not initialized with init_prebindgen_out_dir(). \
                Please ensure that init_prebindgen_out_dir() is called in the build.rs of the source crate.",
                self.input_dir.display()
            )
        });
        let crate_name = self.crate_name.unwrap_or(original_crate_name);

        let mut pb = Prebindgen {
            records: std::collections::HashMap::new(),
            defined_types: std::collections::HashSet::new(),
            input_dir: self.input_dir,
            crate_name,
            edition: self.edition.unwrap_or_else(|| "2021".to_string()),
        };

        // Read the groups based on selection
        if let Some(selected_groups) = self.selected_groups {
            // Read only selected groups
            for group in selected_groups {
                pb.read_group_internal(&group);
            }
        } else {
            // Read all available groups
            pb.read_all_groups_internal();
        }

        pb
    }
}

impl<'a> FileBuilder<'a> {
    pub fn append(self, group: &str) -> Self {
        // Extract just the filename from the full path
        if let Some(file_name) = self.file_path.file_name() {
            self.prebindgen.write_internal(group, file_name, true);
        }
        self
    }
    pub fn append_all(self) -> Self {
        if let Some(file_name) = self.file_path.file_name() {
            for group_name in self.prebindgen.records.keys() {
                self.prebindgen.write_internal(group_name, file_name, true);
            }
        }
        self
    }
    pub fn get_path(&self) -> &std::path::Path {
        &self.file_path
    }
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

    // Create the source crate identifier (convert hyphens to underscores for valid identifier)
    let source_crate_name = source_crate.replace('-', "_");
    let source_crate_ident = syn::Ident::new(&source_crate_name, proc_macro2::Span::call_site());

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
        extern_inputs
            .map(|arg| quote::quote! { #arg }.to_string())
            .collect::<Vec<_>>()
            .join(", "),
        quote::quote! { #output },
        function_body.to_string().trim()
    );

    Ok(stub)
}
