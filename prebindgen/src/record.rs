use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

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
    pub _type_replacements: HashSet<(String, String)>,
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

impl std::fmt::Debug for RecordSyn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RecordSyn")
            .field("kind", &self.kind())
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

impl RecordSyn {
    /// Create a new RecordSyn with the given components
    pub(crate) fn new(
        name: String,
        content: syn::Item,
        source_location: SourceLocation,
        type_replacements: HashSet<(String, String)>,
    ) -> Self {
        Self {
            name,
            content,
            source_location,
            _type_replacements: type_replacements,
        }
    }

    /// Parse a Record into a RecordSyn with feature processing and FFI transformation
    pub(crate) fn from_record(
        record: Record,
        config: &ParseConfig<'_>,
    ) -> Result<Self, String> {
        // Destructure record fields
        let Record {
            kind,
            name,
            content: record_content,
            source_location,
        } = record;

        // Parse the raw content into a syntax tree
        let parsed = syn::parse_file(&record_content).map_err(|e| e.to_string())?;

        // Apply feature processing
        let processed = crate::codegen::process_features(
            parsed,
            config.disabled_features,
            config.enabled_features,
            config.feature_mappings,
            &source_location,
        );

        // Skip records that become empty
        if processed.items.is_empty() {
            return Err(format!(
                "Record {name} of kind {kind} became empty after feature processing"
            ));
        }

        // Transform functions to FFI stubs and collect type replacements
        let (final_item, type_replacements) = if kind == RecordKind::Function {
            // Extract the function from the processed file
            if processed.items.len() != 1 {
                return Err(format!(
                    "Expected exactly one item in file, found {}",
                    processed.items.len()
                ));
            }

            let function_item = match &processed.items[0] {
                syn::Item::Fn(item_fn) => item_fn.clone(),
                item => {
                    return Err(format!(
                        "Expected function item, found {:?}",
                        std::mem::discriminant(item)
                    ));
                }
            };

            // Step 1: Strip function body using trim_implementation
            let trimmed_function = crate::codegen::trim_implementation(function_item);

            // Step 2: Replace types in stripped function with replace_types
            let trimmed_file = syn::File {
                shebang: processed.shebang.clone(),
                attrs: processed.attrs.clone(),
                items: vec![syn::Item::Fn(trimmed_function)],
            };

            let (processed_file, type_replacements) = crate::codegen::replace_types(
                trimmed_file,
                config.crate_name,
                config.exported_types,
                config.allowed_prefixes,
                config.transparent_wrappers,
            );

            // Extract the processed function again
            let processed_function = match &processed_file.items[0] {
                syn::Item::Fn(item_fn) => item_fn.clone(),
                _ => return Err("Expected function item after type replacement".to_string()),
            };

            // Step 3: Generate new body with create_stub_implementation
            let source_crate_name = config.crate_name.replace('-', "_");
            let source_crate_ident =
                syn::Ident::new(&source_crate_name, proc_macro2::Span::call_site());

            let final_function = crate::codegen::create_stub_implementation(
                processed_function,
                &source_crate_ident,
            )?;

            // Determine the appropriate no_mangle attribute based on Rust edition
            let no_mangle_attr: syn::Attribute = if config.edition == "2024" {
                syn::parse_quote! { #[unsafe(no_mangle)] }
            } else {
                syn::parse_quote! { #[no_mangle] }
            };

            // Add the no_mangle attribute and make it extern "C"
            let mut extern_function = final_function;
            extern_function.attrs.insert(0, no_mangle_attr);
            extern_function.sig.unsafety =
                Some(syn::Token![unsafe](proc_macro2::Span::call_site()));
            extern_function.sig.abi = Some(syn::Abi {
                extern_token: syn::Token![extern](proc_macro2::Span::call_site()),
                name: Some(syn::LitStr::new("C", proc_macro2::Span::call_site())),
            });
            extern_function.vis =
                syn::Visibility::Public(syn::Token![pub](proc_macro2::Span::call_site()));

            (syn::Item::Fn(extern_function), type_replacements)
        } else {
            // For non-function items, extract the first (and should be only) item
            if processed.items.len() != 1 {
                return Err(format!(
                    "Expected exactly one item in file, found {}",
                    processed.items.len()
                ));
            }
            (processed.items.into_iter().next().unwrap(), HashSet::new())
        };

        // Construct the RecordSyn with type replacements
        Ok(RecordSyn::new(
            name.clone(),
            final_item,
            source_location.clone(),
            type_replacements,
        ))
    }

    /// Derive the record kind from the syn::Item content
    pub(crate) fn kind(&self) -> RecordKind {
        match &self.content {
            syn::Item::Struct(_) => RecordKind::Struct,
            syn::Item::Enum(_) => RecordKind::Enum,
            syn::Item::Union(_) => RecordKind::Union,
            syn::Item::Fn(_) => RecordKind::Function,
            syn::Item::Type(_) => RecordKind::TypeAlias,
            syn::Item::Const(_) => RecordKind::Const,
            _ => RecordKind::Unknown,
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
