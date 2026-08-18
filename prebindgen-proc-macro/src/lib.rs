//! # prebindgen-proc-macro
//!
//! Procedural macros for the prebindgen system.
//!
//! This crate provides the procedural macros used by the prebindgen system:
//! - `#[prebindgen]` or `#[prebindgen("group")]` - Attribute macro for marking FFI definitions
//! - `prebindgen_out_dir!()` - Macro that returns the prebindgen output directory path
//! - `features!()` - Macro that returns the list of features enabled for the crate
//!
//! # Crate features
//!
//! - **`inline`** *(off by default)* — inject `#[inline]` onto every function
//!   marked with `#[prebindgen]` (types/consts are unaffected).
//!
//!   prebindgen wrappers are usually thin shims that forward to a native API. A
//!   non-generic `pub fn` in one crate is **not** inlined into a Rust caller in
//!   another crate unless the function is `#[inline]` *or* the final binary is
//!   built with link-time optimization. Without inlining, every wrapper call
//!   costs an extra cross-crate call (measurable on hot paths — e.g. a per-message
//!   publish loop).
//!
//!   Two ways to make the wrappers zero-cost:
//!   1. Build the final artifact with **LTO** (`[profile.release] lto = "fat"`,
//!      `codegen-units = 1`). Cross-crate inlining then happens automatically and
//!      this feature is redundant. This is the recommended setup for an FFI
//!      `cdylib`/`staticlib` and matches how upstream zenoh builds its release
//!      profile.
//!   2. Enable **`inline`** when you cannot rely on LTO — e.g. the wrapper crate is
//!      consumed as a normal Rust dependency by crates that build *without* LTO.
//!
//!   It is opt-in because prebindgen can also wrap non-trivial functions where
//!   forcing `#[inline]` would only bloat the consumer; enable it only for genuine
//!   thin-wrapper libraries. The feature affects **only** the Rust function emitted
//!   into the wrapper crate — the recorded definition used for binding generation
//!   (and the resulting C ABI) is unchanged.
//!
//! See also: [`prebindgen`](https://docs.rs/prebindgen) for the main processing library.
//!
use std::sync::OnceLock;

use prebindgen::{
    get_prebindgen_out_dir,
    layout::{capture_file_name, group_dir_name, MAX_COMPONENT_LEN},
    output::check_writer,
    utils::publish_file,
    Record, RecordKind, SourceLocation, DEFAULT_GROUP_NAME,
};
use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    spanned::Spanned,
    DeriveInput, Ident, ItemConst, ItemFn, ItemType, LitStr, Result, Token,
};

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

/// Publish `record` under the path its own contents determine.
///
/// The path is derived from the **record**, never from the process or the
/// compilation that produced it: `{OUT_DIR}/prebindgen/g_{group}/
/// {name}_{digest(record)}.jsonl` (see `prebindgen::layout`). Every compiler
/// that captures this item computes this same path and writes these same
/// bytes, so repeated compilations — `cargo check`, `build`, `test`, `clippy`,
/// and the doctest rustdoc run Cargo never caches — rewrite one file instead of
/// accumulating a copy each (#201).
///
/// That also makes the layout loss-proof by construction rather than by careful
/// key derivation: a name determines its contents, so two writers either write
/// identical bytes to one path or different bytes to different paths. There is
/// no third case in which one compilation overwrites another's records.
fn publish_record(
    group: &str,
    record: &Record,
    serialized: &str,
) -> std::result::Result<(), String> {
    writer_format_checked()?;

    let group_dir = group_dir_name(group);
    if group_dir.len() > MAX_COMPONENT_LEN {
        return Err(format!(
            "#[prebindgen] group name {group:?} is too long: it encodes to {} bytes, \
             and a directory name may hold {MAX_COMPONENT_LEN}",
            group_dir.len()
        ));
    }

    publish_file(
        get_prebindgen_out_dir()
            .join(group_dir)
            .join(capture_file_name(&record.name, serialized)),
        &format!("{serialized}\n"),
    )
    .map_err(|error| format!("prebindgen: {error}"))
}

/// Whether the capture directory is described in the format this macro writes,
/// decided once per compilation.
///
/// This macro and the `init_prebindgen_out_dir()` that prepared the directory
/// are two packages — `prebindgen-proc-macro` and `prebindgen` — and a manifest
/// may name versions of them that lay captures out differently. Nothing else
/// compares those two, so the check happens here, where the disagreement can
/// still be reported against the source crate that declared both. One rustc
/// process compiles one crate, so once is enough.
fn writer_format_checked() -> std::result::Result<(), String> {
    static CHECKED: OnceLock<std::result::Result<(), String>> = OnceLock::new();
    CHECKED
        .get_or_init(|| check_writer(&get_prebindgen_out_dir()))
        .clone()
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
/// ```rust
/// # use prebindgen_proc_macro::prebindgen;
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
///
/// # The `inline` feature
///
/// With the crate `inline` feature enabled, this macro prepends `#[inline]` to the
/// emitted function so thin wrappers stay zero-cost for Rust consumers that do not
/// build with LTO. Off by default; see the crate-level documentation.
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

    // The `inline` feature adds `#[inline]` to function wrappers only (not to
    // structs/enums/types/consts). Captured here before `kind` is moved below.
    let is_function = matches!(kind, RecordKind::Function);

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

    // Publish the record under the path its own contents determine. Failures
    // are reported at the item, not swallowed: a capture that is silently short
    // makes the consumer generate incomplete bindings.
    let serialized = match new_record.to_jsonl_string() {
        Ok(serialized) => serialized,
        Err(error) => {
            return syn::Error::new(span, format!("prebindgen: {error}"))
                .to_compile_error()
                .into()
        }
    };
    if let Err(error) = publish_record(&group, &new_record, &serialized) {
        return syn::Error::new(span, error).to_compile_error().into();
    }

    // Re-emit the original item, optionally prepending `#[cfg(...)]` (from the
    // macro argument) and `#[inline]` (when the `inline` feature is on, functions
    // only). When neither applies, the original tokens are returned unchanged.
    let add_inline = cfg!(feature = "inline") && is_function;
    if parsed_args.cfg.is_none() && !add_inline {
        return input_clone;
    }
    let inline_attr = if add_inline {
        quote! { #[inline] }
    } else {
        quote! {}
    };
    let cfg_attr = if let Some(cfg_value) = &parsed_args.cfg {
        let cfg_tokens: proc_macro2::TokenStream = cfg_value
            .parse()
            .unwrap_or_else(|_| panic!("Invalid cfg condition: {}", cfg_value));
        quote! { #[cfg(#cfg_tokens)] }
    } else {
        quote! {}
    };
    let original_tokens: proc_macro2::TokenStream = input_clone.into();
    quote! {
        #cfg_attr
        #inline_attr
        #original_tokens
    }
    .into()
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
/// ```rust
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
/// ```rust
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

/// Proc macro that returns the **source crate's** manifest directory as a string
/// literal (its `CARGO_MANIFEST_DIR` at compile time).
///
/// Exposing this lets a downstream binding crate locate the marked source crate
/// *wherever it lives* (a path/git/registry dependency) without guessing layout —
/// e.g. to compile a size/alignment probe against it. This complements
/// [`prebindgen_out_dir!`](macro@prebindgen_out_dir) and [`features!`](macro@features).
///
/// # Returns
///
/// A string literal with the absolute path to the source crate's manifest dir.
///
/// # Example
///
/// ```rust
/// use prebindgen_proc_macro::manifest_dir;
///
/// // Create a public constant for use by binding crates
/// pub const MANIFEST_DIR: &str = manifest_dir!();
/// ```
#[proc_macro]
pub fn manifest_dir(_input: TokenStream) -> TokenStream {
    let dir = std::env::var("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR environment variable not set");
    let lit = syn::LitStr::new(&dir, proc_macro2::Span::call_site());
    TokenStream::from(quote! { #lit })
}
