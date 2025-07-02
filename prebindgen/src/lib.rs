//! # prebindgen
//!
//! JSON structure definitions for the prebindgen system.
//!
//! This crate defines the data structures used to represent struct, enum, union, and function definitions
//! in JSON format. These structures are used by the `prebindgen-proc-macro` crate
//! to serialize code definitions and by build scripts to deserialize and process them.
//!
//! The JSON format is JSON-lines where each line contains a separate record:
//! ```json
//! {"kind": "struct", "name": "MyStruct", "content": "pub struct MyStruct { ... }"}
//! {"kind": "enum", "name": "MyEnum", "content": "pub enum MyEnum { ... }"}
//! {"kind": "function", "name": "my_function", "content": "pub fn my_function() { ... }"}
//! ```
//!
//! ## Usage
//!
//! ```rust
//! use prebindgen::{Record, RecordKind};
//! use serde_json;
//!
//! // Parse a JSON line into a Record
//! let json_line = r#"{"kind":"struct","name":"MyStruct","content":"pub struct MyStruct { ... }"}"#;
//! let record: Record = serde_json::from_str(json_line)?;
//!
//! assert_eq!(record.kind, RecordKind::Struct);
//! assert_eq!(record.name, "MyStruct");
//! # Ok::<(), serde_json::Error>(())
//! ```

use core::panic;
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::{env, fs};

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
/// Get the full path to `<name>.json` generated in OUT_DIR.
pub fn get_prebindgen_out_dir() -> std::path::PathBuf {
    let out_dir = std::env::var("OUT_DIR")
        .expect("OUT_DIR environment variable not set. Please ensure you have a build.rs file in your project.");
    std::path::Path::new(&out_dir).join("prebindgen")
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

/// Helper for reading JSON records and generating Rust bindings per group
pub struct Prebindgen {
    records: std::collections::HashMap<String, Vec<Record>>,
    defined_types: std::collections::HashSet<String>,
    input_dir: std::path::PathBuf,
    crate_name: String,
}

impl Prebindgen {
    /// Create a new Prebindgen context with specified directory and crate name
    pub fn new<P: AsRef<Path>>(input_dir: P, crate_name: String) -> Self {
        Self {
            records: std::collections::HashMap::new(),
            defined_types: std::collections::HashSet::new(),
            input_dir: input_dir.as_ref().to_path_buf(),
            crate_name,
        }
    }

    /// Read all JSON files matching the group name pattern `<group>_*.json`, panicking on error with detailed path info
    pub fn read(&mut self, group: &str) {
        let pattern = format!("{}_", group);
        let mut record_map = std::collections::HashMap::new();
        
        // Read the directory and find all matching files
        if let Ok(entries) = fs::read_dir(&self.input_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                    if file_name.starts_with(&pattern) && file_name.ends_with(".json") {
                        trace!("Reading file: {}", path.display());
                        let path_clone = path.clone();
                        (|| -> Result<(), Box<dyn std::error::Error>> {
                            let mut content = fs::read_to_string(&path)?;
                            if content.ends_with(',') {
                                content.pop();
                            }
                            content.push(']');
                            let parsed: Vec<Record> = serde_json::from_str(&content)?;
                            for record in parsed {
                                // Use HashMap to deduplicate records by name
                                record_map.insert(record.name.clone(), record);
                            }
                            Ok(())
                        })()
                        .unwrap_or_else(|e| {
                            panic!("Failed to read {}: {}", path_clone.display(), e);
                        });
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

    /// Generate `<group>.rs`, panicking on error with detailed path info
    pub fn make_rs<P: AsRef<Path>>(&self, group: &str, file_name: P) -> std::path::PathBuf {
        let out_dir = env::var("OUT_DIR").expect("OUT_DIR environment variable not set. Please ensure you have a build.rs file in your project.");
        let dest_path = PathBuf::from(&out_dir).join(&file_name);
        (|| -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
            let mut dest = fs::File::create(&dest_path)?;
            if let Some(group_records) = self.records.get(group) {
                for record in group_records {
                    match record.kind {
                        RecordKind::Function => {
                            let stub = transform_function_to_stub(
                                &record.content,
                                &self.crate_name,
                                &self.defined_types,
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

    // Build the no_mangle extern "C" function
    let stub = quote::quote! {
        #[unsafe(no_mangle)]
        #vis unsafe extern "C" fn #fn_name(#(#extern_inputs),*) #output {
            #function_body
        }
    };

    Ok(stub.to_string())
}
