//! # prebindgen
//!
//! JSON structure definitions for the prebindgen system.
//!
//! This crate defines the data structures used to represent struct and enum definitions
//! in JSON format. These structures are used by the `prebindgen-proc-macro` crate
//! to serialize code definitions and by build scripts to deserialize and process them.
//!
//! The JSON format is JSON-lines where each line contains a separate record:
//! ```json
//! {"kind": "struct", "name": "MyStruct", "content": "pub struct MyStruct { ... }"}
//! {"kind": "enum", "name": "MyEnum", "content": "pub enum MyEnum { ... }"}
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

use serde::{Deserialize, Serialize};
use core::panic;
use std::env;
use std::fs;
use std::io::Write;
use std::path::Path;

/// Represents a record of a struct, enum, or union definition
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Record {
    /// The kind of definition (struct, enum, or union)
    pub kind: RecordKind,
    /// The name of the type
    pub name: String,
    /// The full source code content of the definition
    pub content: String,
}

/// The kind of record (struct, enum, or union)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum RecordKind {
    /// A struct definition
    Struct,
    /// An enum definition
    Enum,
    /// A union definition
    Union,
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
        }
    }
}

#[macro_export]
macro_rules! trace {
    ($($arg:tt)*) => {
        #[cfg(debug_assertions)]
        println!("cargo:warning=[{}:{}] {}", 
            file!().split('/').last().unwrap_or(file!()), 
            line!(), 
            format!($($arg)*)
        );
    };
}

/// Get the full path to the file prebindgen.json
/// generated in OUT_DIR by #[prebindgen] macro.
///
/// This function primarily used internally,
/// but is also available for debugging or testing purposes.
pub fn get_prebindgen_json_path() -> std::path::PathBuf {
    let out_dir = env::var("OUT_DIR")
        .expect("OUT_DIR environment variable not set. Please ensure you have a build.rs file in your project.");
    Path::new(&out_dir).join("prebindgen.json")
}

/// Initialize the prebindgen.json file by cleaning it up and adding "[" to the first line.
/// This function should be called in build.rs instead of deleting the prebindgen.json file.
///
/// This prepares the file to collect JSON records in an array format by starting with
/// an empty object, allowing the prebindgen macro to add records with leading commas.
pub fn init_prebindgen_json() {
    let path = get_prebindgen_json_path();
    let init_closure = || -> Result<(), Box<dyn std::error::Error>> {
        let mut file = fs::File::create(&path)?;
        file.write_all(b"[")?;
        file.flush()?;
        trace!("Initialized prebindgen.json at: {}", path.display());
        Ok(())
    };

    if let Err(e) = init_closure() {
        panic!("Failed to initialize {}: {e}", path.display());
    }
}

/// Process the prebindgen.json file and write ffi definitions to passed rust file in OUT_DIR.
///
/// This function:
/// - Reads the specified prebindgen.json file and adds trailing `]` to complete the JSON array
/// - Parses the result as JSON, ignoring the first empty record
/// - Deduplicates records by name (later records override earlier ones)
/// - Writes the content of all records to OUT_DIR/{ffi_rs}
pub fn prebindgen_json_to_rs<P: AsRef<Path>>(prebindgen_json_path: P, ffi_rs: &str) {
    let process_closure = || -> Result<(), Box<dyn std::error::Error>> {
        // Read the prebindgen.json file
        trace!("Reading: {}", prebindgen_json_path.as_ref().display());
        let mut content = fs::read_to_string(&prebindgen_json_path)?;

        // Replace last trailing comma to `]` to complete the JSON array
        if content.ends_with(',') {
            content.pop(); // Remove the last comma
        }
        content.push(']'); // Add trailing `]` to complete the JSON array

        // Parse as JSON array
        let records: Vec<Record> = serde_json::from_str(&content)?;

        // Skip the first empty record and deduplicate by name
        let mut unique_records = std::collections::HashMap::new();
        for record in records.into_iter().skip(1) {
            unique_records.insert(record.name.clone(), record);
        }

        // Write content to destination file
        let out_dir = env::var("OUT_DIR")?;
        let dest_path = Path::new(&out_dir).join(ffi_rs);
        trace!("Writing to: {}", dest_path.display());
        let mut dest_file = fs::File::create(dest_path)?;

        for record in unique_records.values() {
            writeln!(dest_file, "{}", record.content)?;
        }

        dest_file.flush()?;
        Ok(())
    };

    if let Err(e) = process_closure() {
        panic!("Failed to process {}: {e}", prebindgen_json_path.as_ref().to_string_lossy());
    }
}
