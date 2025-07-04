//! # prebindgen-proc-macro
//!
//! Procedural macros for the prebindgen system.
//!
//! This crate provides the procedural macros used by the prebindgen system:
//! - `#[prebindgen]` or `#[prebindgen("group")]` - Attribute macro for marking FFI definitions
//! - `prebindgen_out_dir!()` - Macro that returns the prebindgen output directory path
//!
//! These macros are typically imported from the main `prebindgen` crate rather than
//! used directly from this crate.

use prebindgen::{get_prebindgen_out_dir, trace, Record, RecordKind, DEFAULT_GROUP_NAME};
use proc_macro::TokenStream;
use quote::quote;
use syn::LitStr;
use std::fs::{OpenOptions, metadata};
use std::io::Write;
use std::path::Path;
use syn::{DeriveInput, ItemFn, ItemType};

/// Helper function to generate consistent error messages for unsupported or unparseable items.
fn unsupported_item_error(item: Option<syn::Item>) -> TokenStream {
    match item {
        Some(item) => {
            let item_type = match &item {
                syn::Item::Const(_) => "Constants", 
                syn::Item::Static(_) => "Static items",
                syn::Item::Mod(_) => "Modules",
                syn::Item::Trait(_) => "Traits",
                syn::Item::Impl(_) => "Impl blocks",
                syn::Item::Use(_) => "Use statements",
                syn::Item::ExternCrate(_) => "Extern crate declarations",
                syn::Item::Macro(_) => "Macro definitions",
                syn::Item::Verbatim(_) => "Verbatim items",
                _ => "This item type",
            };
            
            syn::Error::new_spanned(
                item,
                format!(
                    "{} are not supported by #[prebindgen]",
                    item_type
                )
            )
            .to_compile_error()
            .into()
        }
        None => {
            // If we can't even parse it as an Item, return a generic error
            syn::Error::new(
                proc_macro2::Span::call_site(),
                "Invalid syntax for #[prebindgen]"
            )
            .to_compile_error()
            .into()
        }
    }
}

/// Get the full path to `{group}_{pid}_{thread_id}.jsonl` generated in OUT_DIR.
fn get_prebindgen_jsonl_path(name: &str) -> std::path::PathBuf {
    let thread_id = std::thread::current().id();
    let process_id = std::process::id();
    // Extract numeric thread ID from ThreadId debug representation
    let thread_id_str = format!("{:?}", thread_id);
    let thread_id_num = thread_id_str
        .strip_prefix("ThreadId(")
        .and_then(|s| s.strip_suffix(")"))
        .unwrap_or("0");
    get_prebindgen_out_dir().join(format!("{}_{}_{}.jsonl", name, process_id, thread_id_num))
}

/// Proc macro that returns the prebindgen output directory path as a string literal.
/// 
/// This macro generates a string literal containing the full path to the prebindgen
/// output directory. It should be used to create a public constant that can be
/// consumed by language-specific binding crates.
/// 
/// # Returns
/// 
/// A string literal with the path to the prebindgen output directory.
/// 
/// # Example
/// 
/// ```rust,ignore
/// use prebindgen_proc_macro::prebindgen_out_dir;
/// 
/// // Create a public constant for use by binding crates
/// pub const PREBINDGEN_OUT_DIR: &str = prebindgen_out_dir!();
/// ```
#[proc_macro]
pub fn prebindgen_out_dir(_input: TokenStream) -> TokenStream {
    let file_path = get_prebindgen_out_dir();
    let path_str = file_path.to_string_lossy();

    // Return just the string literal
    let expanded = quote! {
        #path_str
    };

    TokenStream::from(expanded)
}

/// Attribute macro that exports FFI definitions for use in language-specific binding crates.
/// 
/// All types and functions marked with this attribute can be made available in dependent 
/// crates as Rust source code for both binding generator processing (cbindgen, csbindgen, etc.) 
/// and for including into projects to make the compiler generate `#[no_mangle]` FFI exports 
/// for cdylib/staticlib targets.
/// 
/// # Usage
/// 
/// ```rust,ignore
/// // Use with explicit group name
/// #[prebindgen("group_name")]
/// #[repr(C)]
/// pub struct Point {
///     pub x: f64,
///     pub y: f64,
/// }
/// 
/// // Use with default group name "default"
/// #[prebindgen]
/// pub fn calculate_distance(p1: &Point, p2: &Point) -> f64 {
///     ((p2.x - p1.x).powi(2) + (p2.y - p1.y).powi(2)).sqrt()
/// }
/// 
/// // Type aliases are also supported
/// #[prebindgen]
/// pub type example_result = i8;
/// 
/// // Or specify a group name
/// #[prebindgen("functions")]
/// pub fn another_function() -> i32 {
///     42
/// }
/// ```
/// 
/// # Requirements
/// 
/// - Must call `prebindgen::init_prebindgen_out_dir()` in your crate's `build.rs`
/// - Optionally takes a string literal group name for organization (defaults to "default")
#[proc_macro_attribute]
pub fn prebindgen(args: TokenStream, input: TokenStream) -> TokenStream {
    let input_clone = input.clone();
    
    // Parse optional group name literal - use default if not provided
    let group = if args.is_empty() {
        DEFAULT_GROUP_NAME.to_string()
    } else {
        let group_lit = syn::parse::<LitStr>(args)
            .expect("`#[prebindgen]` group name must be a string literal");
        group_lit.value()
    };
    
    // Get the full path to the JSONL file
    let file_path = get_prebindgen_jsonl_path(&group);
    let dest_path = Path::new(&file_path);

    // Try to parse as different item types
    let (kind, name, content) = if let Ok(parsed) = syn::parse::<DeriveInput>(input.clone()) {
        // Handle struct, enum, union
        let kind = match &parsed.data {
            syn::Data::Struct(_) => RecordKind::Struct,
            syn::Data::Enum(_) => RecordKind::Enum,
            syn::Data::Union(_) => RecordKind::Union,
        };
        let tokens = quote! { #parsed };
        (kind, parsed.ident.to_string(), tokens.to_string())
    } else if let Ok(parsed) = syn::parse::<ItemFn>(input.clone()) {
        // Handle function
        // For functions, we want to store only the signature without the body
        let mut fn_sig = parsed.clone();
        fn_sig.block = syn::parse_quote! {{ /* placeholder */ }};
        let tokens = quote! { #fn_sig };
        (
            RecordKind::Function,
            parsed.sig.ident.to_string(),
            tokens.to_string(),
        )
    } else if let Ok(parsed) = syn::parse::<ItemType>(input.clone()) {
        // Handle type alias
        let tokens = quote! { #parsed };
        (
            RecordKind::TypeAlias,
            parsed.ident.to_string(),
            tokens.to_string(),
        )
    } else {
        // Try to parse as any item to provide better error messages
        let item = syn::parse::<syn::Item>(input.clone()).ok();
        return unsupported_item_error(item);
    };

    // Create the new record
    let new_record = Record::new(kind, name, content);

    // Convert record to JSON and append to file in JSON-lines format
    if let Ok(json_content) = serde_json::to_string(&new_record) {
        if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(dest_path) {
            // Check if file is empty (just created or was deleted)
            let is_empty = metadata(dest_path).map(|m| m.len() == 0).unwrap_or(true);

            if is_empty {
                // Create new JSONL file
                trace!("Creating jsonl file: {}", dest_path.display());
            }
            
            // Write the record as a single line (JSON-lines format)
            let _ = writeln!(file, "{}", json_content);
            let _ = file.flush();
        }
    }

    // Return the original input unchanged
    input_clone
}
