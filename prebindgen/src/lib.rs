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
use core::panic;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::{env, fs};

mod jsonl;
/// **Internal API**: JSONL file utilities. Public only for proc-macro crate interaction.
#[doc(hidden)]
pub use jsonl::{read_jsonl_file, write_jsonl_file};

/// File extension for data files
const JSONL_EXTENSION: &str = ".jsonl";
/// Name of the prebindgen output directory
const PREBINDGEN_DIR: &str = "prebindgen";
/// File name for storing the crate name
const CRATE_NAME_FILE: &str = "crate_name.txt";
/// Default group name for prebindgen when no group is specified
pub const DEFAULT_GROUP_NAME: &str = "default";

/// Represents a record of a struct, enum, union, or function definition.
/// 
/// **Internal API**: This type is public only for interaction with the proc-macro crate.
/// It should not be used directly by end users.
#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Record {
    /// The kind of definition (struct, enum, union, or function)
    pub kind: RecordKind,
    /// The name of the type or function
    pub name: String,
    /// The full source code content of the definition
    pub content: String,
}

/// The kind of record (struct, enum, union, or function).
/// 
/// **Internal API**: This type is public only for interaction with the proc-macro crate.
/// It should not be used directly by end users.
#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum RecordKind {
    /// A struct definition with named or unnamed fields
    Struct,
    /// An enum definition with variants
    Enum,
    /// A union definition (C-style union)
    Union,
    /// A function definition (signature only, body is replaced)
    Function,
    /// A type alias definition
    TypeAlias,
    /// A constant definition
    Const,
}

impl Record {
    /// Create a new record with the specified kind, name, and content.
    /// 
    /// **Internal API**: This method is public only for interaction with the proc-macro crate.
    #[doc(hidden)]
    pub fn new(kind: RecordKind, name: String, content: String) -> Self {
        Self {
            kind,
            name,
            content,
        }
    }

    /// Serialize this record to a JSON-lines compatible string.
    /// 
    /// **Internal API**: This method is public only for interaction with the proc-macro crate.
    #[doc(hidden)]
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
            RecordKind::TypeAlias => write!(f, "type"),
            RecordKind::Const => write!(f, "const"),
        }
    }
}

impl RecordKind {
    /// Returns true if this record kind represents a type definition.
    /// 
    /// Type definitions include structs, enums, unions, and type aliases.
    /// Functions and constants are not considered type definitions.
    pub fn is_type(&self) -> bool {
        matches!(
            self,
            RecordKind::Struct | RecordKind::Enum | RecordKind::Union | RecordKind::TypeAlias
        )
    }
}

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
fn read_stored_crate_name(input_dir: &Path) -> Option<String> {
    let crate_name_path = input_dir.join(CRATE_NAME_FILE);
    fs::read_to_string(&crate_name_path)
        .ok()
        .map(|s| s.trim().to_string())
}

/// Initialize the prebindgen output directory for the current crate.
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
    records: std::collections::HashMap<String, Vec<Record>>,
    exported_types: std::collections::HashSet<String>,
    input_dir: std::path::PathBuf,
    crate_name: String,
    edition: String,
}

/// Builder for configuring Prebindgen with optional parameters.
/// 
/// This builder allows you to configure how prebindgen reads and processes
/// the exported FFI definitions before building the final `Prebindgen` instance.
/// 
/// # Example
/// 
/// ```rust,ignore
/// let prebindgen = prebindgen::Builder::new("/path/to/prebindgen/data")
///     .crate_name("my_custom_crate")
///     .edition("2024")
///     .select_group("structs")
///     .select_group("functions") 
///     .build();
/// ```
pub struct Builder {
    input_dir: std::path::PathBuf,
    crate_name: Option<String>,
    edition: Option<String>,
    selected_groups: Option<std::collections::HashSet<String>>,
}

/// Builder for writing groups to files with append capability.
/// 
/// This builder is returned by `Prebindgen::group()` and `Prebindgen::all()` methods
/// and allows you to select multiple groups and write them to a single output file.
/// 
/// # Example
/// 
/// ```rust,ignore
/// // Write multiple groups to one file
/// let combined = prebindgen
///     .group("structs")
///     .group("enums")
///     .group("functions")
///     .write_to_file("combined_ffi.rs");
/// ```
pub struct FileBuilder<'a> {
    prebindgen: &'a Prebindgen,
    groups: Vec<String>,
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

        // Update exported_types with type names from all groups
        for record in &all_records {
            if record.kind.is_type() {
                self.exported_types.insert(record.name.clone());
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

    /// Select a specific group for file generation.
    /// 
    /// Returns a `FileBuilder` that can be used to write the specified group
    /// to a file, optionally combined with other groups.
    /// 
    /// # Arguments
    /// 
    /// * `group_name` - Name of the group to select
    /// 
    /// # Returns
    /// 
    /// A `FileBuilder` configured to write the specified group.
    /// 
    /// # Example
    /// 
    /// ```rust,ignore
    /// let structs_file = prebindgen.group("structs").write_to_file("structs.rs");
    /// ```
    pub fn group(&self, group_name: &str) -> FileBuilder<'_> {
        FileBuilder {
            prebindgen: self,
            groups: vec![group_name.to_string()],
        }
    }

    /// Select all available groups for file generation.
    /// 
    /// Returns a `FileBuilder` that can be used to write all available groups
    /// to a file. This is equivalent to calling `group()` for each available group.
    /// 
    /// # Returns
    /// 
    /// A `FileBuilder` configured to write all available groups.
    /// 
    /// # Example
    /// 
    /// ```rust,ignore
    /// let all_file = prebindgen.all().write_to_file("all_bindings.rs");
    /// ```
    pub fn all(&self) -> FileBuilder<'_> {
        FileBuilder {
            prebindgen: self,
            groups: self.records.keys().cloned().collect(),
        }
    }

    /// Internal method to write records with optional append mode
    fn write_internal(
        &self,
        dest: &mut File,
        group: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(group_records) = self.records.get(group) {
            for record in group_records {
                match record.kind {
                    RecordKind::Function => {
                        let stub = transform_function_to_stub(
                            &record.content,
                            &self.crate_name,
                            &self.exported_types,
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
        Ok(())
    }
}

impl Builder {
    /// Create a new builder with the specified input directory.
    /// 
    /// The input directory should contain the prebindgen data files generated
    /// by the common FFI crate. This is typically obtained from the
    /// `PREBINDGEN_OUT_DIR` constant exported by the common FFI crate.
    /// 
    /// # Arguments
    /// 
    /// * `input_dir` - Path to the directory containing prebindgen data files
    /// 
    /// # Example
    /// 
    /// ```rust,ignore
    /// let builder = prebindgen::Builder::new(common_ffi::PREBINDGEN_OUT_DIR);
    /// ```
    pub fn new<P: AsRef<Path>>(input_dir: P) -> Self {
        Self {
            input_dir: input_dir.as_ref().to_path_buf(),
            crate_name: None,
            edition: None,
            selected_groups: None,
        }
    }
    
    /// Override the source crate name used in generated extern "C" functions.
    /// 
    /// By default, the crate name is read from the prebindgen data files.
    /// This method allows you to override it, which can be useful when
    /// the crate has been renamed or when you want to use a different
    /// module path in the generated calls.
    /// 
    /// # Arguments
    /// 
    /// * `crate_name` - The crate name to use in generated function calls
    /// 
    /// # Example
    /// 
    /// ```rust,ignore
    /// let builder = prebindgen::Builder::new(path)
    ///     .crate_name("my_renamed_crate");
    /// ```
    pub fn crate_name<S: Into<String>>(mut self, crate_name: S) -> Self {
        self.crate_name = Some(crate_name.into());
        self
    }
    
    /// Set the Rust edition to use for generated code.
    /// 
    /// This affects how the `#[no_mangle]` attribute is generated:
    /// - For edition "2024": `#[unsafe(no_mangle)]`
    /// - For other editions: `#[no_mangle]`
    /// 
    /// # Arguments
    /// 
    /// * `edition` - The Rust edition ("2021", "2024", etc.)
    /// 
    /// Default is "2024" if not specified.
    /// 
    /// # Example
    /// 
    /// ```rust,ignore
    /// let builder = prebindgen::Builder::new(path)
    ///     .edition("2021");
    /// ```
    pub fn edition<E: Into<String>>(mut self, edition: E) -> Self {
        self.edition = Some(edition.into());
        self
    }
    
    /// Select a specific group to include in the final Prebindgen instance.
    /// 
    /// This method can be called multiple times to select multiple groups.
    /// If no groups are selected, all available groups will be included.
    /// 
    /// # Arguments
    /// 
    /// * `group_name` - Name of the group to include
    /// 
    /// # Example
    /// 
    /// ```rust,ignore
    /// let builder = prebindgen::Builder::new(path)
    ///     .select_group("structs")
    ///     .select_group("core_functions");
    /// ```
    pub fn select_group<S: Into<String>>(mut self, group_name: S) -> Self {
        if self.selected_groups.is_none() {
            self.selected_groups = Some(std::collections::HashSet::new());
        }
        if let Some(ref mut groups) = self.selected_groups {
            groups.insert(group_name.into());
        }
        self
    }
    
    /// Build the configured Prebindgen instance.
    /// 
    /// This method reads the prebindgen data files from the input directory
    /// and creates a `Prebindgen` instance ready for generating FFI bindings.
    /// 
    /// # Panics
    /// 
    /// Panics if the input directory was not properly initialized with
    /// `init_prebindgen_out_dir()` in the source crate's build.rs.
    /// 
    /// # Returns
    /// 
    /// A configured `Prebindgen` instance ready for use.
    /// 
    /// # Example
    /// 
    /// ```rust,ignore
    /// let prebindgen = prebindgen::Builder::new(path)
    ///     .edition("2021")
    ///     .build();
    /// ```
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
            exported_types: std::collections::HashSet::new(),
            input_dir: self.input_dir,
            crate_name,
            edition: self.edition.unwrap_or_else(|| "2024".to_string()),
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
    /// Add another group to the selection.
    /// 
    /// This allows you to combine multiple groups into a single output file.
    /// 
    /// # Arguments
    /// 
    /// * `group_name` - Name of the additional group to include
    /// 
    /// # Returns
    /// 
    /// The same `FileBuilder` with the additional group added.
    /// 
    /// # Example
    /// 
    /// ```rust,ignore
    /// let combined = prebindgen
    ///     .group("structs")
    ///     .group("enums")
    ///     .group("functions")
    ///     .write_to_file("combined.rs");
    /// ```
    pub fn group<S: Into<String>>(mut self, group_name: S) -> Self {
        self.groups.push(group_name.into());
        self
    }

    /// Write the selected groups to a file.
    /// 
    /// Generates the Rust source code for all selected groups and writes it
    /// to the specified file. For functions, this generates `#[no_mangle] extern "C"`
    /// wrapper functions that call the original functions from the source crate.
    /// For types (structs, enums, unions), this copies the original definitions.
    /// 
    /// If the file path is relative, it will be created relative to `OUT_DIR`.
    /// 
    /// # Arguments
    /// 
    /// * `file_name` - Path where the generated code should be written
    /// 
    /// # Returns
    /// 
    /// The absolute path to the generated file.
    /// 
    /// # Panics
    /// 
    /// Panics if:
    /// - `OUT_DIR` environment variable is not set
    /// - File creation fails
    /// - Writing to the file fails
    /// 
    /// # Example
    /// 
    /// ```rust,ignore
    /// let output_file = prebindgen.all().write_to_file("ffi_bindings.rs");
    /// println!("Generated FFI bindings at: {}", output_file.display());
    /// ```
    pub fn write_to_file<P: AsRef<Path>>(self, file_name: P) -> std::path::PathBuf {
        // Prepend with OUT_DIR if file_name is relative
        let file_name = if file_name.as_ref().is_relative() {
            let out_dir = env::var("OUT_DIR").expect("OUT_DIR environment variable not set. Please ensure you have a build.rs file in your project.");
            PathBuf::from(out_dir).join(file_name)
        } else {
            file_name.as_ref().to_path_buf()
        };
        let out_dir = env::var("OUT_DIR").expect("OUT_DIR environment variable not set. Please ensure you have a build.rs file in your project.");
        let dest_path = PathBuf::from(&out_dir).join(&file_name);
        let mut dest = fs::File::create(&dest_path).unwrap_or_else(|e| {
            panic!("Failed to create {}: {}", dest_path.display(), e);
        });
        for group in &self.groups {
            // Write the records for each group
            self.prebindgen
                .write_internal(&mut dest, group)
                .unwrap_or_else(|e| {
                    panic!(
                        "Failed to write records for group {} to {}: {}",
                        group,
                        dest_path.display(),
                        e
                    )
                });
        }
        trace!(
            "Generated bindings for groups [{}] written to: {}",
            self.groups.join(", "),
            dest_path.display()
        );
        dest_path
    }
}

/// Helper function to check if a type contains any of the exported types
fn contains_exported_type(
    ty: &syn::Type,
    exported_types: &std::collections::HashSet<String>,
) -> bool {
    match ty {
        syn::Type::Path(type_path) => {
            // Check if the type itself is defined
            if let Some(segment) = type_path.path.segments.last() {
                let type_name = segment.ident.to_string();
                if exported_types.contains(&type_name) {
                    return true;
                }

                // Check generic arguments recursively
                if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                    for arg in &args.args {
                        if let syn::GenericArgument::Type(inner_ty) = arg {
                            if contains_exported_type(inner_ty, exported_types) {
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
            contains_exported_type(&type_ref.elem, exported_types)
        }
        syn::Type::Ptr(type_ptr) => {
            // Check the pointed-to type
            contains_exported_type(&type_ptr.elem, exported_types)
        }
        syn::Type::Slice(type_slice) => {
            // Check the slice element type
            contains_exported_type(&type_slice.elem, exported_types)
        }
        syn::Type::Array(type_array) => {
            // Check the array element type
            contains_exported_type(&type_array.elem, exported_types)
        }
        syn::Type::Tuple(type_tuple) => {
            // Check all tuple element types
            type_tuple
                .elems
                .iter()
                .any(|elem_ty| contains_exported_type(elem_ty, exported_types))
        }
        _ => false,
    }
}

/// Helper function to validate that a type is either absolute (starting with ::) or defined in exported types
fn validate_type_for_ffi(
    ty: &syn::Type,
    exported_types: &std::collections::HashSet<String>,
    context: &str,
) -> Result<(), String> {
    match ty {
        syn::Type::Path(type_path) => {
            // Check if the path is absolute (starts with ::)
            if type_path.path.leading_colon.is_some() {
                return Ok(()); // Absolute path is valid
            }
            
            // Check if it's a single identifier that's in exported types
            if type_path.path.segments.len() == 1 {
                if let Some(segment) = type_path.path.segments.first() {
                    let type_name = segment.ident.to_string();
                    if exported_types.contains(&type_name) {
                        // Recursively validate generic arguments
                        if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                            for arg in &args.args {
                                if let syn::GenericArgument::Type(inner_ty) = arg {
                                    validate_type_for_ffi(inner_ty, exported_types, &format!("{} (generic argument)", context))?;
                                }
                            }
                        }
                        return Ok(());
                    }
                }
            }
            
            // If we get here, it's a relative path that's not in exported types
            Err(format!(
                "Type '{}' in {} is not valid for FFI: must be either absolute (starting with '::') or defined in exported types",
                quote::quote! { #ty }, context
            ))
        }
        syn::Type::Reference(type_ref) => {
            // Validate the referenced type
            validate_type_for_ffi(&type_ref.elem, exported_types, &format!("{} (reference)", context))
        }
        syn::Type::Ptr(type_ptr) => {
            // Validate the pointed-to type
            validate_type_for_ffi(&type_ptr.elem, exported_types, &format!("{} (pointer)", context))
        }
        syn::Type::Slice(type_slice) => {
            // Validate the slice element type
            validate_type_for_ffi(&type_slice.elem, exported_types, &format!("{} (slice element)", context))
        }
        syn::Type::Array(type_array) => {
            // Validate the array element type
            validate_type_for_ffi(&type_array.elem, exported_types, &format!("{} (array element)", context))
        }
        syn::Type::Tuple(type_tuple) => {
            // Validate all tuple element types
            for (i, elem_ty) in type_tuple.elems.iter().enumerate() {
                validate_type_for_ffi(elem_ty, exported_types, &format!("{} (tuple element {})", context, i))?;
            }
            Ok(())
        }
        _ => {
            // For other types, we'll be conservative and reject them
            Err(format!(
                "Unsupported type '{}' in {}: only path types, references, pointers, slices, arrays, and tuples are supported for FFI",
                quote::quote! { #ty }, context
            ))
        }
    }
}

/// Transform a function prototype to a no_mangle extern "C" function that calls the original function
fn transform_function_to_stub(
    function_content: &str,
    source_crate: &str,
    exported_types: &std::collections::HashSet<String>,
    edition: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    // Helper function to check if a type needs transmute
    let needs_transmute = |ty: &syn::Type| -> bool { contains_exported_type(ty, exported_types) };
    // Parse the function using syn
    let parsed: syn::ItemFn = syn::parse_str(function_content)?;

    // Validate parameter types
    for (i, input) in parsed.sig.inputs.iter().enumerate() {
        if let syn::FnArg::Typed(pat_type) = input {
            validate_type_for_ffi(
                &pat_type.ty,
                exported_types,
                &format!("parameter {} of function '{}'", i + 1, parsed.sig.ident),
            ).map_err(|e| format!("Invalid FFI function parameter: {}", e))?;
        }
    }

    // Validate return type
    if let syn::ReturnType::Type(_, return_type) = &parsed.sig.output {
        validate_type_for_ffi(
            return_type,
            exported_types,
            &format!("return type of function '{}'", parsed.sig.ident),
        ).map_err(|e| format!("Invalid FFI function return type: {}", e))?;
    }

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
