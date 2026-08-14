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
use std::{
    collections::HashMap,
    ffi::{OsStr, OsString},
    hash::{Hash, Hasher},
    path::{Path, PathBuf},
    sync::{Arc, Mutex, OnceLock},
};

use prebindgen::{get_prebindgen_out_dir, Record, RecordKind, SourceLocation, DEFAULT_GROUP_NAME};
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

#[derive(Default)]
struct CaptureState {
    initialized: bool,
}

type SharedCaptureState = Arc<Mutex<CaptureState>>;

/// One lock per capture file. Macro expansions can run on several rustc
/// threads, but unrelated groups do not need to block one another.
static CAPTURE_STATES: OnceLock<Mutex<HashMap<PathBuf, SharedCaptureState>>> = OnceLock::new();

/// Options that describe how the compiled tests are *run*, or how diagnostics
/// are *rendered*, and take a value (`--opt value` or `--opt=value`).
///
/// None of them changes what is compiled, and Cargo varies them from one
/// invocation to the next: `cargo test --doc <filter>` forwards the filter (and
/// everything after `--`) to rustdoc as `--test-args`. Feeding them to the
/// fallback id would mint a fresh capture file per invocation — exactly the
/// leak this is fixing (#201).
///
/// The list is deliberately a deny-list of provably run-time-only options
/// rather than an allow-list of compile-identifying ones: dropping an
/// argument that *does* identify a unit would let two distinct units share one
/// capture file and reset each other's records, which loses items silently.
/// Everything not listed here is kept, so an unknown option can only ever cost
/// an extra file, never a lost record.
const RUNTIME_ONLY_VALUE_OPTIONS: &[&str] = &[
    "--test-args",
    "--test-run-directory",
    "--runtool",
    "--runtool-arg",
    "--color",
    "--error-format",
    "--json",
    "--diagnostic-width",
];

/// Run-time-only options that take no value; see [`RUNTIME_ONLY_VALUE_OPTIONS`].
const RUNTIME_ONLY_FLAGS: &[&str] = &["--nocapture"];

/// Identify the compilation unit this expansion belongs to.
///
/// `build.rs` clears the prebindgen directory, but only when Cargo re-runs the
/// build script — plain `cargo check` / `build` / `test` / `clippy` cycles
/// re-run *rustc* against an untouched directory. A per-process file name would
/// therefore leave one stale JSONL file behind per rustc invocation, forever
/// (#201). The id has to be stable across rebuilds of the same unit (so the
/// file is overwritten) yet differ between units built concurrently, e.g. lib
/// vs. test: rustc's own `-C metadata` is exactly that. rustdoc, which compiles
/// doctests without `-C metadata`, gets a hash of the compile-identifying part
/// of its command line instead — equally stable per unit.
///
/// Both halves err towards *more* ids: an id that is finer than the unit only
/// leaves a stale file behind, while an id shared by two units would let one
/// unit reset the other's capture and silently drop its items.
fn unit_id() -> &'static str {
    static UNIT_ID: OnceLock<String> = OnceLock::new();
    UNIT_ID.get_or_init(|| unit_id_from_args(&std::env::args_os().collect::<Vec<_>>()))
}

fn unit_id_from_args(args: &[OsString]) -> String {
    let mut metadata = metadata_components(args);
    // rustc sorts the list before folding it into the crate id, so argument
    // order alone does not make a different unit.
    metadata.sort_unstable();
    if !metadata.is_empty() {
        // rustc accepts arbitrary strings here, including slashes and
        // backslashes. Hash the components before using them as a path
        // component so a caller's metadata cannot introduce directories or
        // platform-specific names.
        return hash_unit_id(metadata.as_slice());
    }

    hash_unit_id(compile_identifying_args(args).as_slice())
}

/// Collect every `-C metadata` component.
///
/// rustc declares this option as `metadata: Vec<String>` parsed by `parse_list`,
/// so repeated occurrences **accumulate** (they are not last-value-wins) and each
/// occurrence contributes its whitespace-separated words; `StableCrateId::new`
/// then sorts them. `-Cmetadata=a b`, `-Cmetadata=a -Cmetadata=b` and
/// `-C metadata=b --codegen=metadata=a` therefore all describe the same unit,
/// while `-Cmetadata=b` alone describes a different one, and an empty value
/// contributes nothing. Verified on rustc 1.85.0 and 1.97.1 by comparing the
/// symbol hash / crate disambiguator of a probe crate.
///
/// Hashing only the last occurrence — as this did before — would map
/// `unit_a shared` and `unit_b shared` onto one capture file, letting one unit
/// reset the other's records.
fn metadata_components(args: &[OsString]) -> Vec<&str> {
    let mut components = Vec::new();
    let mut index = 0;
    while index < args.len() {
        let arg = args[index].to_str();
        index += 1;
        let Some(arg) = arg else { continue };
        // rustc accepts the short and long spelling of the codegen flag, each
        // with the value joined or in the following argument.
        let value = match arg {
            "-C" | "--codegen" => {
                let value = args.get(index).and_then(|arg| arg.to_str());
                // The next argument is this option's value whatever it holds.
                index += 1;
                value.and_then(|value| value.strip_prefix("metadata="))
            }
            _ => arg
                .strip_prefix("-C")
                .and_then(|arg| arg.strip_prefix("metadata="))
                .or_else(|| arg.strip_prefix("--codegen=metadata=")),
        };
        if let Some(value) = value {
            components.extend(value.split_whitespace());
        }
    }
    components
}

/// The arguments that identify *what* is compiled: everything except the
/// run-time-only options above.
fn compile_identifying_args(args: &[OsString]) -> Vec<&OsStr> {
    let mut kept = Vec::with_capacity(args.len());
    let mut index = 0;
    while index < args.len() {
        let arg = &args[index];
        index += 1;
        // A non-UTF-8 argument is none of the options below, so it is kept.
        let Some(text) = arg.to_str() else {
            kept.push(arg.as_os_str());
            continue;
        };
        if RUNTIME_ONLY_FLAGS.contains(&text) {
            continue;
        }
        if RUNTIME_ONLY_VALUE_OPTIONS.contains(&text) {
            // Drop the value carried by the following argument as well.
            index += 1;
            continue;
        }
        let joined_value = RUNTIME_ONLY_VALUE_OPTIONS.iter().any(|option| {
            text.strip_prefix(option)
                .is_some_and(|rest| rest.starts_with('='))
        });
        if joined_value {
            continue;
        }
        kept.push(arg.as_os_str());
    }
    kept
}

fn hash_unit_id(value: &(impl Hash + ?Sized)) -> String {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    value.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

/// Get the full path to `{group}_{unit_id}.jsonl` in OUT_DIR.
fn get_prebindgen_jsonl_path(group: &str) -> std::path::PathBuf {
    get_prebindgen_out_dir().join(format!("{group}_{}.jsonl", unit_id()))
}

fn capture_state(file_path: &Path) -> std::result::Result<SharedCaptureState, String> {
    let mut states = CAPTURE_STATES
        .get_or_init(Mutex::default)
        .lock()
        .map_err(|_| "prebindgen capture-state registry lock was poisoned".to_string())?;
    Ok(Arc::clone(
        states.entry(file_path.to_path_buf()).or_default(),
    ))
}

/// Reset a compilation unit's capture on its first record, then serialize all
/// initialization and appends to that file. Keeping both operations under the
/// same per-file lock prevents a first writer from unlinking a record that a
/// second rustc thread has already appended.
fn write_capture_record(file_path: &Path, record: &Record) -> std::result::Result<(), String> {
    let state = capture_state(file_path)?;
    let mut state = state.lock().map_err(|_| {
        format!(
            "prebindgen capture lock for {} was poisoned",
            file_path.display()
        )
    })?;

    if !state.initialized {
        match std::fs::remove_file(file_path) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(format!(
                    "failed to reset prebindgen capture {}: {error}",
                    file_path.display()
                ));
            }
        }
    }

    prebindgen::utils::write_to_jsonl_file(file_path, &[record])
        .map_err(|error| format!("failed to append {}: {error}", file_path.display()))?;
    state.initialized = true;
    Ok(())
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

    let file_path = get_prebindgen_jsonl_path(&group);
    if let Err(error) = write_capture_record(&file_path, &new_record) {
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

#[cfg(test)]
mod tests {
    use std::{
        collections::BTreeSet,
        sync::{Arc, Barrier},
    };

    use super::{
        compile_identifying_args, hash_unit_id, unit_id_from_args, write_capture_record, Record,
        RecordKind,
    };

    fn record(name: impl Into<String>) -> Record {
        let name = name.into();
        Record::new(
            RecordKind::Struct,
            name.clone(),
            format!("pub struct {name};"),
            Default::default(),
            None,
        )
    }

    fn test_dir(name: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!(
            "prebindgen-proc-macro-{}-{name}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }

    #[test]
    fn unit_id_accepts_every_rustc_codegen_spelling_and_hashes_the_value() {
        let args = [
            "rustc",
            "-Cmetadata=first",
            "--codegen=metadata=second",
            "-C",
            "metadata=third/path\\unit",
        ]
        .map(Into::into);
        let unit_id = unit_id_from_args(&args);
        assert_eq!(
            unit_id,
            hash_unit_id(&["first", "second", "third/path\\unit"][..])
        );
        assert_eq!(unit_id.len(), 16);
        assert!(unit_id.bytes().all(|byte| byte.is_ascii_hexdigit()));
        assert!(!unit_id.contains(['/', '\\']));

        let args = ["rustdoc", "--codegen", "metadata=long-form"].map(Into::into);
        assert_eq!(unit_id_from_args(&args), hash_unit_id(&["long-form"][..]));
    }

    #[test]
    fn unit_id_hashes_every_metadata_component_not_only_the_last() {
        // rustc's `-C metadata` is a list: every occurrence contributes to the
        // unit, so two units sharing only their trailing component are still
        // distinct and must not share a capture file.
        let unit_a = ["rustc", "-Cmetadata=unit_a", "-Cmetadata=shared"].map(Into::into);
        let unit_b = ["rustc", "-Cmetadata=unit_b", "-Cmetadata=shared"].map(Into::into);
        assert_ne!(unit_id_from_args(&unit_a), unit_id_from_args(&unit_b));
        assert_ne!(
            unit_id_from_args(&unit_a),
            unit_id_from_args(&["rustc", "-Cmetadata=shared"].map(Into::into))
        );

        // Argument order alone is not a different unit: rustc sorts the list
        // before folding it into the crate id (verified against the symbol
        // hashes of a probe crate on 1.85.0 and 1.97.1).
        assert_eq!(
            unit_id_from_args(&["rustc", "-Cmetadata=a", "-Cmetadata=b"].map(Into::into)),
            unit_id_from_args(&["rustc", "-Cmetadata=b", "-Cmetadata=a"].map(Into::into))
        );
    }

    #[test]
    fn unit_id_follows_rustc_list_parsing_of_metadata() {
        // `parse_list` splits each value on whitespace and appends it, so these
        // spellings describe one and the same unit.
        assert_eq!(
            unit_id_from_args(&["rustc", "-Cmetadata=a b"].map(Into::into)),
            unit_id_from_args(&["rustc", "-Cmetadata=a", "-Cmetadata=b"].map(Into::into))
        );
        // An empty value contributes no component.
        assert_eq!(
            unit_id_from_args(&["rustc", "-Cmetadata=first", "-Cmetadata="].map(Into::into)),
            unit_id_from_args(&["rustc", "-Cmetadata=first"].map(Into::into))
        );
        // ... and on its own it leaves nothing to key the unit on, so the
        // argument fallback takes over.
        let only_empty = ["rustc", "-Cmetadata=", "src/lib.rs"].map(Into::into);
        assert_eq!(
            unit_id_from_args(&only_empty),
            hash_unit_id(compile_identifying_args(&only_empty).as_slice())
        );
    }

    #[test]
    fn unit_id_fallback_is_stable_without_metadata() {
        let args = ["rustdoc", "--test", "src/lib.rs"].map(Into::into);
        assert_eq!(unit_id_from_args(&args), unit_id_from_args(&args));
        assert_ne!(
            unit_id_from_args(&args),
            unit_id_from_args(&["rustdoc", "src/lib.rs"].map(Into::into))
        );
    }

    #[test]
    fn unit_id_fallback_ignores_arguments_that_only_run_the_tests() {
        // Cargo forwards a doctest filter (and anything after `--`) as
        // `--test-args`; rustdoc compiles the same unit either way.
        let doctest = |extra: &[&str]| {
            let mut args = vec![
                "rustdoc".to_string(),
                "--edition=2021".to_string(),
                "--crate-type".to_string(),
                "lib".to_string(),
                "--crate-name".to_string(),
                "example_flat".to_string(),
                "--test".to_string(),
                "examples/example-flat/src/lib.rs".to_string(),
            ];
            args.extend(extra.iter().map(|arg| arg.to_string()));
            unit_id_from_args(&args.into_iter().map(Into::into).collect::<Vec<_>>())
        };

        let plain = doctest(&[]);
        assert_eq!(plain, doctest(&["--test-args", "alpha"]));
        assert_eq!(plain, doctest(&["--test-args", "beta"]));
        assert_eq!(plain, doctest(&["--test-args=beta"]));
        assert_eq!(
            plain,
            doctest(&["--test-args", "beta", "--test-args", "--nocapture"])
        );
        assert_eq!(plain, doctest(&["--nocapture"]));
        assert_eq!(
            plain,
            doctest(&[
                "--test-run-directory",
                "examples/example-flat",
                "--color",
                "always",
                "--error-format=json",
            ])
        );

        // What is compiled still separates units.
        assert_ne!(plain, doctest(&["--cfg", "feature=\"unstable\""]));
        assert_ne!(
            plain,
            unit_id_from_args(
                &["rustdoc", "--crate-name", "other", "--test", "src/lib.rs"].map(Into::into)
            )
        );
        // An unknown option is kept: an extra file is acceptable, a lost
        // record is not.
        assert_ne!(plain, doctest(&["--never-heard-of-it", "value"]));
    }

    #[test]
    fn concurrent_first_writes_reset_once_without_losing_records() {
        const WRITERS: usize = 32;

        let dir = test_dir("concurrent-first-writes");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("group_unit.jsonl");
        prebindgen::utils::write_to_jsonl_file(&path, &[record("Stale")]).unwrap();

        let barrier = Arc::new(Barrier::new(WRITERS));
        let handles = (0..WRITERS)
            .map(|index| {
                let barrier = Arc::clone(&barrier);
                let path = path.clone();
                std::thread::spawn(move || {
                    let record = record(format!("Record{index}"));
                    barrier.wait();
                    write_capture_record(&path, &record).unwrap();
                })
            })
            .collect::<Vec<_>>();
        for handle in handles {
            handle.join().unwrap();
        }

        // A later expansion in the same process must append, not reset again.
        write_capture_record(&path, &record("LastRecord")).unwrap();

        let records = prebindgen::utils::read_jsonl_file(&path).unwrap();
        assert_eq!(records.len(), WRITERS + 1);
        let names = records
            .into_iter()
            .map(|record| record.name)
            .collect::<BTreeSet<_>>();
        let expected = (0..WRITERS)
            .map(|index| format!("Record{index}"))
            .chain(["LastRecord".to_string()])
            .collect::<BTreeSet<_>>();
        assert_eq!(names, expected);

        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn unexpected_reset_error_is_reported() {
        let dir = test_dir("reset-error");
        let path = dir.join("capture-is-a-directory");
        std::fs::create_dir_all(&path).unwrap();

        let error = write_capture_record(&path, &record("Record")).unwrap_err();
        assert!(
            error.contains("failed to reset prebindgen capture"),
            "{error}"
        );
        assert!(error.contains(&path.display().to_string()), "{error}");

        std::fs::remove_dir_all(dir).unwrap();
    }
}
