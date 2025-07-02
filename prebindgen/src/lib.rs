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
use std::env;

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
        Self { kind, name, content }
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

/// Get the full path to the file prebindgen.json 
/// generated in OUT_DIR by #[prebindgen] macro.
/// 
/// This function primarily used internally,
/// but is also available for debugging or testing purposes.
pub fn get_prebindgen_json_path() -> String {
    let out_dir = env::var("OUT_DIR")
        .expect("OUT_DIR environment variable not set. Please ensure you have a build.rs file in your project.");
    format!("{}/prebindgen.json", out_dir)
}