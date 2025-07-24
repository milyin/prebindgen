use serde::{Deserialize, Serialize};
use std::collections::HashSet;

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

impl std::fmt::Display for SourceLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}:{}", self.file, self.line, self.column)
    }
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

impl From<&(syn::Item,SourceLocation)> for RecordKind {
    fn from((item, source_location): &(syn::Item, SourceLocation)) -> Self {
        match item {
            syn::Item::Struct(_) => RecordKind::Struct,
            syn::Item::Enum(_) => RecordKind::Enum,
            syn::Item::Union(_) => RecordKind::Union,
            syn::Item::Fn(_) => RecordKind::Function,
            syn::Item::Type(_) => RecordKind::TypeAlias,
            syn::Item::Const(_) => RecordKind::Const,
            _ => panic!("Unknown syn::Item variant for RecordKind at {source_location}"),
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
            RecordKind::TypeAlias => write!(f, "type"),
            RecordKind::Const => write!(f, "const"),
        }
    }
}


impl Record {
    /// Create a new record with the specified kind, name, content, and source location.
    ///
    /// **Internal API**: This method is public only for interaction with the proc-macro crate.
    #[doc(hidden)]
    pub fn new(
        kind: RecordKind,
        name: String,
        content: String,
        source_location: SourceLocation,
    ) -> Self {
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

    pub(crate) fn parse(self: &Record) -> (syn::Item, SourceLocation) {
        // Parse the raw content into a syntax tree
        let parsed = syn::parse_file(&self.content).map_err(|e| {
            panic!(
                "Failed to parse record content at {}: {}",
                self.source_location, e
            )
        }).unwrap();

        // Check that we have exactly one item
        let mut items = parsed.items.into_iter();
        let item = items.next().unwrap_or_else(|| {
            panic!(
                "Expected exactly one item in record, found 0 at {}",
                self.source_location
            )
        });

        if items.next().is_some() {
            panic!(
                "Expected exactly one item in record, found more than 1 at {}",
                self.source_location
            );
        }

        // Create RecordSyn first
        let record_syn = (item, self.source_location.clone());

        // Check that the item type matches the record kind
        let actual_kind: RecordKind = (&record_syn).into();
        if actual_kind != self.kind {
            panic!(
                "Record kind mismatch at {}: expected {}, found {}",
                self.source_location, self.kind, actual_kind
            );
        }
        record_syn
    }
}

/// Configuration parameters for parsing records
pub(crate) struct ParseConfig<'a> {
    pub crate_name: &'a str,
    pub exported_types: &'a HashSet<String>,
    pub allowed_prefixes: &'a [syn::Path],
    pub transparent_wrappers: &'a [syn::Path],
    pub edition: &'a str,
}

impl<'a> ParseConfig<'a> {
    pub fn crate_ident(&self) -> syn::Ident {
        // Convert crate name to identifier (replace dashes with underscores)
        let source_crate_name = self.crate_name.replace('-', "_");
        syn::Ident::new(&source_crate_name, proc_macro2::Span::call_site())
    }
}