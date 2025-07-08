use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
};

use crate::codegen::TypeTransmutePair;

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

/// Represents a record with parsed syntax tree content.
///
/// This is the internal representation used by Prebindgen after deduplication
/// and initial parsing. Unlike `Record`, this stores the parsed `syn::Item`
/// instead of raw string content for more efficient processing.
#[derive(Clone)]
pub(crate) struct RecordSyn {
    /// The name of the type or function
    pub name: String,
    /// The parsed syntax tree content of the definition (after feature processing and stub generation)
    pub content: syn::Item,
    /// Source location information
    pub source_location: SourceLocation,
    /// Type replacement pairs for this record only (local_type, origin_type)
    pub type_replacements: HashSet<TypeTransmutePair>,
}

impl Default for RecordSyn {
    fn default() -> Self {
        Self {
            name: String::new(),
            content: syn::Item::Verbatim(proc_macro2::TokenStream::new()),
            source_location: SourceLocation::default(),
            type_replacements: HashSet::new(),
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
}

impl RecordSyn {
    /// Create a new RecordSyn with the given components
    pub(crate) fn new(
        name: String,
        content: syn::Item,
        source_location: SourceLocation,
        type_replacements: HashSet<TypeTransmutePair>,
    ) -> Self {
        Self {
            name,
            content,
            source_location,
            type_replacements,
        }
    }

    /// Get the identifier from the parsed syntax tree item
    pub(crate) fn ident(&self) -> Result<&syn::Ident, String> {
        match &self.content {
            syn::Item::Struct(item) => Ok(&item.ident),
            syn::Item::Enum(item) => Ok(&item.ident),
            syn::Item::Union(item) => Ok(&item.ident),
            syn::Item::Fn(item) => Ok(&item.sig.ident),
            syn::Item::Type(item) => Ok(&item.ident),
            syn::Item::Const(item) => Ok(&item.ident),
            _ => Err(format!(
                "unexpected item type '{}' at {} (RecordSyn::ident)",
                std::any::type_name::<&syn::Item>(),
                self.source_location
            )),
        }
    }

    /// Process the features in the content according to the provided configuration.
    /// Returns true if the content is not empty after processing.
    fn process_features(&mut self, config: &ParseConfig<'_>) -> bool {
        let processed = crate::codegen::process_features(
            syn::File {
                shebang: None,
                attrs: vec![],
                items: vec![self.content.clone()],
            },
            config.disabled_features,
            config.enabled_features,
            config.feature_mappings,
            &self.source_location,
        );
        // Update the content and kind with processed items
        if let Some(content) = processed.items.into_iter().next() {
            self.content = content;
            true
        } else {
            self.content = syn::Item::Verbatim(proc_macro2::TokenStream::new());
            false
        }
    }

    /// Parse a Record into a RecordSyn with feature processing and FFI transformation
    pub(crate) fn parse_record(record: Record, config: &ParseConfig<'_>) -> Result<Self, String> {
        let mut record_syn = Self::try_from(record)?;

        // Apply feature processing
        if !record_syn.process_features(config) {
            // If processing results in no content, return an empty RecordSyn
            return Ok(record_syn);
        }

        if let syn::Item::Fn(function) = &mut record_syn.content {
            // Transform functions to FFI stubs (including type replacement and collection)
            crate::codegen::convert_to_stub(function, config, &mut record_syn.type_replacements)?;
        } else {
            // Replace types in non-function items and collect type replacements
            crate::codegen::replace_types_in_item(
                &mut record_syn.content,
                config,
                &mut record_syn.type_replacements,
            );
        }
        Ok(record_syn)
    }

    /// Derive the record kind from the syn::Item content
    pub(crate) fn kind(&self) -> Result<RecordKind, String> {
        match &self.content {
            syn::Item::Struct(_) => Ok(RecordKind::Struct),
            syn::Item::Enum(_) => Ok(RecordKind::Enum),
            syn::Item::Union(_) => Ok(RecordKind::Union),
            syn::Item::Fn(_) => Ok(RecordKind::Function),
            syn::Item::Type(_) => Ok(RecordKind::TypeAlias),
            syn::Item::Const(_) => Ok(RecordKind::Const),
            _ => Err("Unknown syn::Item variant for RecordSyn::kind".to_string()),
        }
    }

    /// Collect type replacements from this record
    ///
    /// Adds all type replacement pairs from this record to the provided HashSet.
    /// This is useful for gathering type replacements that need assertions.
    ///
    /// # Parameters
    ///
    /// * `type_replacements` - Mutable reference to the HashSet to add replacements to
    pub(crate) fn collect_type_replacements(&self, type_replacements: &mut HashSet<TypeTransmutePair>) {
        type_replacements.extend(self.type_replacements.iter().cloned());
    }
}

impl std::fmt::Debug for RecordSyn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RecordSyn")
            .field("kind", &self.kind().as_ref().map(|k| k.to_string()).unwrap_or_else(|e| e.clone()))
            .field("name", &self.name)
            .field("content", &"<syn::Item>")
            .field("source_location", &self.source_location)
            .finish()
    }
}

/// Configuration parameters for parsing records
pub(crate) struct ParseConfig<'a> {
    pub crate_name: &'a str,
    pub exported_types: &'a HashSet<String>,
    pub disabled_features: &'a HashSet<String>,
    pub enabled_features: &'a HashSet<String>,
    pub feature_mappings: &'a HashMap<String, String>,
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

impl TryFrom<Record> for RecordSyn {
    type Error = String;

    fn try_from(record: Record) -> Result<Self, Self::Error> {
        // Parse the raw content into a syntax tree
        let parsed = syn::parse_file(&record.content).map_err(|e| {
            format!(
                "Failed to parse record content at {}: {}",
                record.source_location, e
            )
        })?;

        // Check that we have exactly one item
        let mut items = parsed.items.into_iter();
        let item = items.next().ok_or_else(|| {
            format!(
                "Expected exactly one item in record, found 0 at {}",
                record.source_location
            )
        })?;

        if items.next().is_some() {
            return Err(format!(
                "Expected exactly one item in record, found more than 1 at {}",
                record.source_location
            ));
        }

        // Create RecordSyn first
        let record_syn = RecordSyn::new(
            record.name,
            item,
            record.source_location.clone(),
            HashSet::new(),
        );

        // Check that the item type matches the record kind
        let actual_kind = record_syn.kind()?;
        if actual_kind != record.kind {
            return Err(format!(
                "Record kind mismatch at {}: expected {}, found {}",
                record.source_location,
                record.kind,
                actual_kind
            ));
        }

        Ok(record_syn)
    }
}
