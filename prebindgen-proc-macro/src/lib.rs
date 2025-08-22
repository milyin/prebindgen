//! # prebindgen-proc-macro
//!
//! Procedural macros for the prebindgen system.
//!
//! This crate provides the procedural macros used by the prebindgen system:
//! - `#[prebindgen]` or `#[prebindgen("group")]` - Attribute macro for marking FFI definitions
//! - `prebindgen_out_dir!()` - Macro that returns the prebindgen output directory path
//! - `features!()` - Macro that returns the list of features enabled for the crate
//!
//! See also: [`prebindgen`](https://docs.rs/prebindgen) for the main processing library.
//!
use prebindgen::{get_prebindgen_out_dir, Record, RecordKind, SourceLocation, DEFAULT_GROUP_NAME};
use proc_macro::TokenStream;
use quote::quote;
use std::collections::HashMap;
use std::fs::OpenOptions;
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;
use syn::{DeriveInput, ItemConst, ItemFn, ItemType};
use syn::{Ident, LitStr};
use syn::{Result, Token};

/// Helper function to generate consistent error messages for unsupported or unparseable items.
fn unsupported_item_error(item: Option<syn::Item>) -> TokenStream {
    match item {
        Some(item) => {
            let item_type = match &item {
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
                format!("{item_type} are not supported by #[prebindgen]"),
            )
            .to_compile_error()
            .into()
        }
        None => {
            // If we can't even parse it as an Item, return a generic error
            syn::Error::new(
                proc_macro2::Span::call_site(),
                "Invalid syntax for #[prebindgen]",
            )
            .to_compile_error()
            .into()
        }
    }
}

/// Arguments for the prebindgen macro
struct PrebindgenArgs {
    group: String,
    cfg: Option<String>,
}

impl Parse for PrebindgenArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut group = DEFAULT_GROUP_NAME.to_string();
        let mut cfg = None;

        if input.is_empty() {
            return Ok(PrebindgenArgs { group, cfg });
        }

        // Parse arguments in any order
        while !input.is_empty() {
            if input.peek(LitStr) {
                // String literal - could be group name
                let lit: LitStr = input.parse()?;
                group = lit.value();
            } else if input.peek(Ident) {
                let ident: Ident = input.parse()?;
                input.parse::<Token![=]>()?;

                match ident.to_string().as_str() {
                    "cfg" => {
                        let cfg_lit: LitStr = input.parse()?;
                        cfg = Some(cfg_lit.value());
                    }
                    _ => {
                        return Err(syn::Error::new_spanned(ident, "Expected 'cfg'"));
                    }
                }
            } else {
                return Err(syn::Error::new(input.span(), "Invalid argument format"));
            }

            // Parse optional comma
            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            } else if !input.is_empty() {
                return Err(syn::Error::new(
                    input.span(),
                    "Expected comma between arguments",
                ));
            }
        }

        Ok(PrebindgenArgs { group, cfg })
    }
}

thread_local! {
    static THREAD_ID: std::cell::RefCell<Option<u64>> = std::cell::RefCell::new(None);
    static JSONL_PATHS: std::cell::RefCell<HashMap<String, std::path::PathBuf>> = std::cell::RefCell::new(HashMap::new());
}

/// Get the full path to `{group}_{pid}_{thread_id}.jsonl` generated in OUT_DIR.
fn get_prebindgen_jsonl_path(group: &str) -> std::path::PathBuf {
    if let Some(p) = JSONL_PATHS.with(|path| path.borrow().get(group).cloned()) {
        return p;
    }
    let process_id = std::process::id();
    let thread_id = if let Some(in_thread_id) = THREAD_ID.with(|id| *id.borrow()) {
        in_thread_id
    } else {
        let new_id = rand::random::<u64>();
        THREAD_ID.with(|id| *id.borrow_mut() = Some(new_id));
        new_id
    };
    let mut random_value = None;
    // Try to really create file and repeat until success
    // to avoid collisions in extremely rare case when two threads got 
    // the same random value
    let new_path = loop {
        let postfix = if let Some(rv) = random_value {
            format!("_{rv}")
        } else {
            "".to_string()
        };
        let path = get_prebindgen_out_dir()
            .join(format!("{group}_{process_id}_{thread_id}{postfix}.jsonl"));
        if OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&path)
            .is_ok()
        {
            break path;
        }
        random_value = Some(rand::random::<u32>());
    };
    JSONL_PATHS.with(|path| {
        path.borrow_mut()
            .insert(group.to_string(), new_path.clone());
    });
    new_path
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
/// // Add cfg attribute to generated code
/// #[prebindgen(cfg = "feature = \"experimental\"")]
/// pub fn experimental_function() -> i32 {
///     42
/// }
///
/// // Combine group name with cfg
/// #[prebindgen("functions", cfg = "unix")]
/// pub fn another_function() -> i32 {
///     42
/// }
/// ```
///
/// # Requirements
///
/// - Must call `prebindgen::init_prebindgen_out_dir()` in your crate's `build.rs`
/// - Optionally takes a string literal group name for organization (defaults to "default")
/// - Optionally takes `cfg = "condition"` to add `#[cfg(condition)]` to generated code
#[proc_macro_attribute]
pub fn prebindgen(args: TokenStream, input: TokenStream) -> TokenStream {
    let input_clone = input.clone();

    // Parse arguments
    let parsed_args = syn::parse::<PrebindgenArgs>(args).expect("Invalid #[prebindgen] arguments");

    let group = parsed_args.group;

    // Try to parse as different item types
    let (kind, name, content, span) = if let Ok(parsed) = syn::parse::<DeriveInput>(input.clone()) {
        // Handle struct, enum, union
        let kind = match &parsed.data {
            syn::Data::Struct(_) => RecordKind::Struct,
            syn::Data::Enum(_) => RecordKind::Enum,
            syn::Data::Union(_) => RecordKind::Union,
        };
        let tokens = quote! { #parsed };
        (
            kind,
            parsed.ident.to_string(),
            tokens.to_string(),
            parsed.span(),
        )
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
            parsed.sig.span(),
        )
    } else if let Ok(parsed) = syn::parse::<ItemType>(input.clone()) {
        // Handle type alias
        let tokens = quote! { #parsed };
        (
            RecordKind::TypeAlias,
            parsed.ident.to_string(),
            tokens.to_string(),
            parsed.ident.span(),
        )
    } else if let Ok(parsed) = syn::parse::<ItemConst>(input.clone()) {
        // Handle constant
        let tokens = quote! { #parsed };
        (
            RecordKind::Const,
            parsed.ident.to_string(),
            tokens.to_string(),
            parsed.ident.span(),
        )
    } else {
        // Try to parse as any item to provide better error messages
        let item = syn::parse::<syn::Item>(input.clone()).ok();
        return unsupported_item_error(item);
    };

    // Extract basic source location information available during compilation
    let source_location = SourceLocation::from_span(&span);

    // Create the new record
    let new_record = Record::new(
        kind,
        name,
        content,
        source_location,
        parsed_args.cfg.clone(),
    );

    // Get the full path to the JSONL file
    let file_path = get_prebindgen_jsonl_path(&group);
    if let Err(_) = prebindgen::write_to_jsonl_file(&file_path, &[&new_record]) {
        return TokenStream::from(quote! {
            compile_error!("Failed to write prebindgen record");
        });
    }

    // Apply cfg attribute to the original code if specified
    if let Some(cfg_value) = &parsed_args.cfg {
        let cfg_tokens: proc_macro2::TokenStream = cfg_value
            .parse()
            .unwrap_or_else(|_| panic!("Invalid cfg condition: {}", cfg_value));
        let cfg_attr = quote! { #[cfg(#cfg_tokens)] };
        let original_tokens: proc_macro2::TokenStream = input_clone.into();
        let result = quote! {
            #cfg_attr
            #original_tokens
        };
        result.into()
    } else {
        // Otherwise return the original input unchanged
        input_clone
    }
}

/// Proc macro that returns the prebindgen output directory path as a string literal.
///
/// This macro generates a string literal containing the full path to the prebindgen
/// output directory. It should be used to create a public constant that can be
/// consumed by language-specific binding crates.
///
/// # Panics
///
/// Panics if OUT_DIR environment variable is not set. This indicates that the macro
/// is being used outside of a build.rs context.
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
    let out_dir = std::env::var("OUT_DIR")
        .expect("OUT_DIR environment variable not set. Please ensure you have a build.rs file in your project.");
    let file_path = std::path::Path::new(&out_dir).join("prebindgen");
    let path_str = file_path.to_string_lossy();

    let expanded = quote! {
        #path_str
    };

    TokenStream::from(expanded)
}

/// Proc macro that returns the enabled features, joined by commas, as a string literal.
///
/// The value is sourced from the `PREBINDGEN_FEATURES` compile-time environment variable,
/// which is set by calling `prebindgen::init_prebindgen_out_dir()` in your crate's `build.rs`.
///
/// # Panics
///
/// Emits a compile-time error if `PREBINDGEN_FEATURES` is not set, which typically means
/// `prebindgen::init_prebindgen_out_dir()` wasn't called in `build.rs`.
///
/// # Returns
///
/// A string literal containing the comma-separated list of enabled features.
/// The string may be empty if no features are enabled.
///
/// # Example
///
/// ```rust,ignore
/// use prebindgen_proc_macro::features;
///
/// pub const ENABLED_FEATURES: &str = features!();
/// ```
#[proc_macro]
pub fn features(_input: TokenStream) -> TokenStream {
    let features = std::env::var("PREBINDGEN_FEATURES").expect(
        "PREBINDGEN_FEATURES environment variable not set. Ensure prebindgen::init_prebindgen_out_dir() is called in build.rs",
    );
    let lit = syn::LitStr::new(&features, proc_macro2::Span::call_site());
    TokenStream::from(quote! { #lit })
}
