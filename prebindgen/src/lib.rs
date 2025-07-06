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
use core::panic;
use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::{env, fs};
use roxygen::roxygen;

mod codegen;
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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Record {
    /// The kind of definition (struct, enum, union, or function)
    pub kind: RecordKind,
    /// The name of the type or function
    pub name: String,
    /// The full source code content of the definition
    pub content: String,
    /// Source location information
    pub source_location: SourceLocation,
}

/// Source location information for tracking where code originated
#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct SourceLocation {
    /// The source file path
    pub file: String,
    /// The line number where the item starts (1-based)
    pub line: usize,
    /// The column number where the item starts (1-based)
    pub column: usize,
}

/// The kind of record (struct, enum, union, or function).
///
/// **Internal API**: This type is public only for interaction with the proc-macro crate.
/// It should not be used directly by end users.
#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum RecordKind {
    /// Unknown or unrecognized record type
    #[default]
    Unknown,
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
    /// Create a new record with the specified kind, name, content, and source location.
    ///
    /// **Internal API**: This method is public only for interaction with the proc-macro crate.
    #[doc(hidden)]
    pub fn new(kind: RecordKind, name: String, content: String, source_location: SourceLocation) -> Self {
        Self {
            kind,
            name,
            content,
            source_location,
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
            RecordKind::Unknown => write!(f, "unknown"),
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
    /// Functions, constants, and unknown types are not considered type definitions.
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
    builder: Builder,
    records: HashMap<String, Vec<Record>>,
    exported_types: HashSet<String>,
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
///     .allowed_prefix("libc")
///     .allowed_prefix("core")
///     .strip_transparent_wrapper("std::mem::MaybeUninit")
///     .select_group("structs")
///     .select_group("functions")
///     .disable_feature("experimental")
///     .enable_feature("std")
///     .match_feature("unstable", "unstable")
///     .build();
/// ```
pub struct Builder {
    input_dir: std::path::PathBuf,
    crate_name: String, // Empty string by default, read from file if empty
    edition: String,
    selected_groups: HashSet<String>,
    allowed_prefixes: Vec<syn::Path>,
    disabled_features: HashSet<String>,
    enabled_features: HashSet<String>,
    feature_mappings: HashMap<String, String>,
    transparent_wrappers: Vec<syn::Path>,
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
        let pattern = format!("{group}_");
        let mut record_map = HashMap::new();

        // Read the directory and find all matching files
        if let Ok(entries) = fs::read_dir(&self.builder.input_dir) {
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
        let mut groups = HashSet::new();

        // Discover all available groups
        if let Ok(entries) = fs::read_dir(&self.builder.input_dir) {
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

    /// Select a specific group for file generation
    ///
    /// Returns a `FileBuilder` that can be used to write the specified group
    /// to a file, optionally combined with other groups.
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
    #[roxygen]
    pub fn group(
        &self, 
        /// Name of the group to select
        group_name: &str
    ) -> FileBuilder<'_> {
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
        use std::collections::HashSet;
        
        // Collect assertion pairs across all functions in this group
        let mut global_assertion_type_pairs: HashSet<(String, String)> = HashSet::new();
        
        if let Some(group_records) = self.records.get(group) {
            for record in group_records {
                // Parse the content as a syntax tree for feature processing
                let content = syn::parse_file(&record.content)
                    .map_err(|e| format!("Failed to parse content for {}: {}", record.name, e))?;

                // Apply feature processing to the syntax tree
                let content = codegen::process_features(
                    content,
                    &self.builder.disabled_features,
                    &self.builder.enabled_features,
                    &self.builder.feature_mappings,
                );

                // Skip if content is empty after feature processing
                if content.items.is_empty() {
                    continue;
                }

                // Generate FFI stub for function
                let content = if record.kind == RecordKind::Function {
                    codegen::transform_function_to_stub(
                        content,
                        &self.builder.crate_name,
                        &self.exported_types,
                        &self.builder.allowed_prefixes,
                        &self.builder.transparent_wrappers,
                        &self.builder.edition,
                        &mut global_assertion_type_pairs,
                        &record.source_location,
                    )?
                } else {
                    content
                };
                writeln!(dest, "{}", prettyplease::unparse(&content))?;
            }
            
            // Generate all collected assertions at the end of the file
            if !global_assertion_type_pairs.is_empty() {
                let type_assertions = codegen::generate_type_assertions(&global_assertion_type_pairs);
                for assertion in type_assertions {
                    let assertion_file = syn::File {
                        shebang: None,
                        attrs: vec![],
                        items: vec![assertion],
                    };
                    writeln!(dest, "{}", prettyplease::unparse(&assertion_file))?;
                }
            }
        }
        dest.flush()?;
        Ok(())
    }
}

impl Builder {
    /// Create a new builder with the specified input directory
    ///
    /// The input directory should contain the prebindgen data files generated
    /// by the common FFI crate. This is typically obtained from the
    /// `PREBINDGEN_OUT_DIR` constant exported by the common FFI crate.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let builder = prebindgen::Builder::new(common_ffi::PREBINDGEN_OUT_DIR);
    /// ```
    #[roxygen]
    pub fn new<P: AsRef<Path>>(
        /// Path to the directory containing prebindgen data files
        input_dir: P
    ) -> Self {
        // Generate comprehensive allowed prefixes including standard prelude
        let allowed_prefixes = codegen::generate_standard_allowed_prefixes();

        Self {
            input_dir: input_dir.as_ref().to_path_buf(),
            crate_name: String::new(), // Empty string by default, read from file if empty
            edition: "2024".to_string(), // Default edition
            selected_groups: HashSet::new(),
            allowed_prefixes,
            disabled_features: HashSet::new(),
            enabled_features: HashSet::new(),
            feature_mappings: HashMap::new(),
            transparent_wrappers: Vec::new(),
        }
    }

    /// Override the source crate name used in generated extern "C" functions
    ///
    /// By default, the crate name is read from the prebindgen data files.
    /// This method allows you to override it, which can be useful when
    /// the crate has been renamed or when you want to use a different
    /// module path in the generated calls.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let builder = prebindgen::Builder::new(path)
    ///     .crate_name("my_renamed_crate");
    /// ```
    #[roxygen]
    pub fn crate_name<S: Into<String>>(
        mut self, 
        /// The crate name to use in generated function calls
        crate_name: S
    ) -> Self {
        self.crate_name = crate_name.into();
        self
    }

    /// Set the Rust edition to use for generated code
    ///
    /// This affects how the `#[no_mangle]` attribute is generated:
    /// - For edition "2024": `#[unsafe(no_mangle)]`
    /// - For other editions: `#[no_mangle]`
    ///
    /// Default is "2024" if not specified.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let builder = prebindgen::Builder::new(path)
    ///     .edition("2021");
    /// ```
    #[roxygen]
    pub fn edition<E: Into<String>>(
        mut self, 
        /// The Rust edition ("2021", "2024", etc.)
        edition: E
    ) -> Self {
        self.edition = edition.into();
        self
    }

    /// Select a specific group to include in the final Prebindgen instance
    ///
    /// This method can be called multiple times to select multiple groups.
    /// If no groups are selected, all available groups will be included.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let builder = prebindgen::Builder::new(path)
    ///     .select_group("structs")
    ///     .select_group("core_functions");
    /// ```
    #[roxygen]
    pub fn select_group<S: Into<String>>(
        mut self, 
        /// Name of the group to include
        group_name: S
    ) -> Self {
        self.selected_groups.insert(group_name.into());
        self
    }

    /// Add an allowed type prefix for FFI validation
    ///
    /// This method allows you to specify additional type prefixes that should be
    /// considered valid for FFI functions, beyond the comprehensive set of default
    /// allowed prefixes that includes the standard prelude, core library types,
    /// primitive types, and common FFI types.
    ///
    /// # Default Allowed Prefixes
    ///
    /// The builder automatically includes prefixes for:
    /// - Standard library modules (`std`, `core`, `alloc`)
    /// - Standard prelude types (`Option`, `Result`, `Vec`, `String`, etc.)
    /// - Core library modules (`core::mem`, `core::ptr`, etc.)
    /// - Primitive types (`bool`, `i32`, `u64`, etc.)
    /// - Common FFI types (`libc`, `c_char`, `c_int`, etc.)
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let builder = prebindgen::Builder::new(path)
    ///     .allowed_prefix("libc")
    ///     .allowed_prefix("core");
    /// ```
    #[roxygen]
    pub fn allowed_prefix<S: Into<String>>(
        mut self, 
        /// The additional type prefix to allow
        prefix: S
    ) -> Self {
        let prefix_str = prefix.into();
        if let Ok(path) = syn::parse_str::<syn::Path>(&prefix_str) {
            self.allowed_prefixes.push(path);
        } else {
            panic!("Invalid path prefix: '{}'", prefix_str);
        }
        self
    }

    /// Add a transparent wrapper type to be stripped from FFI function parameters
    ///
    /// Transparent wrappers are types that wrap other types but have the same
    /// memory layout (like `std::mem::MaybeUninit<T>`). When generating FFI stubs,
    /// these wrappers will be stripped from parameter types to create simpler
    /// C-compatible function signatures.
    ///
    /// For example, if you add `std::mem::MaybeUninit` as a transparent wrapper:
    /// - `&mut std::mem::MaybeUninit<Foo>` becomes `*mut Foo` in the FFI signature
    /// - `&std::mem::MaybeUninit<Bar>` becomes `*const Bar` in the FFI signature
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let builder = prebindgen::Builder::new(path)
    ///     .strip_transparent_wrapper("std::mem::MaybeUninit")
    ///     .strip_transparent_wrapper("std::mem::ManuallyDrop");
    /// ```
    #[roxygen]
    pub fn strip_transparent_wrapper<S: Into<String>>(
        mut self, 
        /// The transparent wrapper type to strip (e.g., "std::mem::MaybeUninit")
        wrapper_type: S
    ) -> Self {
        let wrapper_str = wrapper_type.into();
        if let Ok(path) = syn::parse_str::<syn::Path>(&wrapper_str) {
            self.transparent_wrappers.push(path);
        } else {
            panic!("Invalid transparent wrapper type: '{}'", wrapper_str);
        }
        self
    }

    /// Disable a feature in the generated code
    ///
    /// When processing code with `#[cfg(feature="...")]` attributes, code blocks
    /// guarded by disabled features will be completely skipped in the output.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let builder = prebindgen::Builder::new(path)
    ///     .disable_feature("experimental")
    ///     .disable_feature("deprecated");
    /// ```
    #[roxygen]
    pub fn disable_feature<S: Into<String>>(
        mut self, 
        /// The name of the feature to disable
        feature_name: S
    ) -> Self {
        self.disabled_features.insert(feature_name.into());
        self
    }

    /// Enable a feature in the generated code
    ///
    /// When processing code with `#[cfg(feature="...")]` attributes, code blocks
    /// guarded by enabled features will be included in the output with the
    /// `#[cfg(...)]` attribute removed.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let builder = prebindgen::Builder::new(path)
    ///     .enable_feature("experimental");
    /// ```
    #[roxygen]
    pub fn enable_feature<S: Into<String>>(
        mut self, 
        /// The name of the feature to enable
        feature_name: S
    ) -> Self {
        self.enabled_features.insert(feature_name.into());
        self
    }

    /// Map a feature name to a different name in the generated code
    ///
    /// When processing code with `#[cfg(feature="...")]` attributes, features
    /// that match the mapping will have their names replaced with the target
    /// feature name in the output.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let builder = prebindgen::Builder::new(path)
    ///     .match_feature("unstable", "unstable")
    ///     .match_feature("internal", "unstable");
    /// ```
    #[roxygen]
    pub fn match_feature<S1: Into<String>, S2: Into<String>>(
        mut self,
        /// The original feature name to match
        from_feature: S1,
        /// The new feature name to use in the output
        to_feature: S2,
    ) -> Self {
        self.feature_mappings
            .insert(from_feature.into(), to_feature.into());
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
    pub fn build(mut self) -> Prebindgen {
        // Determine the crate name: use provided one, or read from stored file, or panic if not initialized
        let original_crate_name = read_stored_crate_name(&self.input_dir).unwrap_or_else(|| {
            panic!(
                "The directory {} was not initialized with init_prebindgen_out_dir(). \
                Please ensure that init_prebindgen_out_dir() is called in the build.rs of the source crate.",
                self.input_dir.display()
            )
        });
        if self.crate_name.is_empty() {
            self.crate_name = original_crate_name;
        }

        let mut pb = Prebindgen {
            builder: self,
            records: HashMap::new(),
            exported_types: HashSet::new(),
        };

        // Read the groups based on selection
        if pb.builder.selected_groups.is_empty() {
            // Read all available groups
            pb.read_all_groups_internal();
        } else {
            // Read only selected groups
            for group in &pb.builder.selected_groups.clone() {
                pb.read_group_internal(group);
            }
        }

        pb
    }
}

impl<'a> FileBuilder<'a> {
    /// Add another group to the selection
    ///
    /// This allows you to combine multiple groups into a single output file.
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
    #[roxygen]
    pub fn group<S: Into<String>>(
        mut self, 
        /// Name of the additional group to include
        group_name: S
    ) -> Self {
        self.groups.push(group_name.into());
        self
    }

    /// Write the selected groups to a file
    ///
    /// Generates the Rust source code for all selected groups and writes it
    /// to the specified file. For functions, this generates `#[no_mangle] extern "C"`
    /// wrapper functions that call the original functions from the source crate.
    /// For types (structs, enums, unions), this copies the original definitions.
    ///
    /// If the file path is relative, it will be created relative to `OUT_DIR`.
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
    #[roxygen]
    pub fn write_to_file<P: AsRef<Path>>(
        self, 
        /// Path where the generated code should be written
        file_name: P
    ) -> std::path::PathBuf {
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
        use tempfile::NamedTempFile;

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

        // Try to build prebindgen with the invalid data
        let prebindgen = Builder::new(&prebindgen_dir).build();

        // Try to write the group, which should fail with a parsing error
        let mut temp_file = NamedTempFile::new().unwrap();
        let result = prebindgen.write_internal(temp_file.as_file_mut(), "structs");

        // Should get an error about parsing failure
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Failed to parse content for InvalidStruct"));
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
    fn test_error_reporting_with_source_location() {
        use std::collections::HashSet;
        
    // Parse a function with invalid FFI types - using a custom type that's not in allowed prefixes
    let function_content = r#"
pub fn invalid_ffi_function(param: mycrate::CustomType) -> othercrate::AnotherType {
    Default::default()
}
"#;
        
        let file = syn::parse_file(function_content).unwrap();
        let exported_types = HashSet::new();
        let allowed_prefixes = codegen::generate_standard_allowed_prefixes();
        let transparent_wrappers = Vec::new();
        let mut assertion_type_pairs = HashSet::new();
        
        let source_location = SourceLocation {
            file: "test_file.rs".to_string(),
            line: 42,  // Example line number
            column: 5, // Example column number
        };
        
        // This should trigger an FFI validation error with source location
        match codegen::transform_function_to_stub(
            file,
            "test-crate",
            &exported_types,
            &allowed_prefixes,
            &transparent_wrappers,
            "2021",
            &mut assertion_type_pairs,
            &source_location,
        ) {
            Ok(_) => std::panic!("Expected error but got success"),
            Err(error) => {
                println!("Error with location info: {error}");
                // Check that the error includes source location information
                assert!(error.contains("test_file.rs"));
                assert!(error.contains("42"));
                assert!(error.contains("5"));
            }
        }
    }

    #[test]
    fn test_default_implementations() {
        // Test that SourceLocation has sensible defaults
        let default_location: SourceLocation = Default::default();
        assert_eq!(default_location.file, "");
        assert_eq!(default_location.line, 0);
        assert_eq!(default_location.column, 0);

        // Test that Record has sensible defaults
        let default_record: Record = Default::default();
        assert_eq!(default_record.kind, RecordKind::Unknown); // Should be the #[default] variant
        assert_eq!(default_record.name, "");
        assert_eq!(default_record.content, "");
        assert_eq!(default_record.source_location, default_location);

        // Test that RecordKind has sensible default
        let default_kind: RecordKind = Default::default();
        assert_eq!(default_kind, RecordKind::Unknown);
    }

    #[test]
    fn test_unknown_record_kind() {
        // Test that Unknown is not considered a type
        assert!(!RecordKind::Unknown.is_type());
        
        // Test that Unknown displays correctly
        assert_eq!(format!("{}", RecordKind::Unknown), "unknown");
        
        // Test that Unknown is the default
        let default_kind: RecordKind = Default::default();
        assert_eq!(default_kind, RecordKind::Unknown);
    }
}
