//! # prebindgen
//!
//! A tool for separating the implementation of FFI interfaces from language-specific binding generation,
//! allowing each to reside in different crates.
//!
//! See also: [`prebindgen-proc-macro`](https://docs.rs/prebindgen-proc-macro) for the procedural macros.
//!
//! ## Problem
//!
//! When creating Rust libraries that need to expose FFI interfaces to multiple languages,
//! it may be preferable to create separate `cdylib` or `staticlib` crates for each language-specific binding.
//! This allows you to tailor each crate to the requirements and quirks of its binding generator and to specifisc of the
//! destination language.
//! However, `#[no_mangle] extern "C"` functions can only be defined in a `cdylib` or `staticlib` crate, and cannot be
//! exported from a `lib` crate. As a result, these functions must be duplicated in each language-specific
//! binding crate. This duplication is inconvenient for large projects with many FFI functions and types.
//!
//! ## Solution
//!
//! `prebindgen` solves this by generating `#[no_mangle] extern "C"` Rust proxy source code from a common
//! Rust library crate.
//! Language-specific binding crates can then compile this generated code and pass it to their respective
//! binding generators (such as cbindgen, csbindgen, etc.).
//!
//! ## Usage example
//!
//! See also example projects on <https://github.com/milyin/prebindgen/tree/main/examples>
//!
//! See also the prebindgen-proc-macro documentation for details on how to use the `#[prebindgen]` macro:
//! <https://docs.rs/prebindgen-proc-macro/latest/prebindgen_proc_macro/>
//!
//! ### 1. In the Common FFI Library Crate (e.g., `example_flat`)
//!
//! Mark the types and functions that form your FFI surface with the `prebindgen`
//! macro. The source crate stays **plain idiomatic Rust** — opaque handles are
//! ordinary types returned by value, fallible calls return `Result<T, E>`; the
//! language adapter does the C-ABI lowering, so there is no `#[repr(C)]` here.
//!
//! ```rust,ignore
//! // example-flat/src/lib.rs
//! use prebindgen_proc_macro::{features, prebindgen, prebindgen_out_dir};
//!
//! // Export the prebindgen output directory and the enabled features.
//! pub const PREBINDGEN_OUT_DIR: &str = prebindgen_out_dir!();
//! pub const FEATURES: &str = features!();
//!
//! // An opaque handle — a plain Rust type, returned by value.
//! pub struct Calculator { value: f64 }
//!
//! #[prebindgen]
//! pub fn calculator_new() -> Calculator { Calculator { value: 0.0 } }
//!
//! #[prebindgen]
//! pub fn calculator_get_value(c: &Calculator) -> f64 { c.value }
//! ```
//!
//! Call [`init_prebindgen_out_dir`] in the crate's `build.rs`:
//!
//! ```rust,no_run
//! // example-flat/build.rs
//! prebindgen::init_prebindgen_out_dir();
//! ```
//!
//! ### 2. In a Language-Specific Binding Crate (e.g. `example-cbindgen`)
//!
//! Depend on the common FFI library (as both a normal and a build dependency) and
//! drive the [`lang::Cbindgen`] adapter from `build.rs`:
//!
//! ```toml
//! # example-cbindgen/Cargo.toml
//! [dependencies]
//! example-flat = { path = "../example-flat" }
//! prebindgen = "0.5"
//! konst = "0.3"      # the generated file emits a konst feature guard
//!
//! [build-dependencies]
//! example-flat = { path = "../example-flat" }
//! prebindgen = "0.5"
//! cbindgen = "0.29"
//! syn = { version = "2", features = ["full"] }
//! ```
//! ```rust,ignore
//! // example-cbindgen/build.rs
//! use syn::parse_quote as pq;
//!
//! fn main() {
//!     // Read the items captured from the common FFI crate.
//!     let source = prebindgen::Source::new(example_flat::PREBINDGEN_OUT_DIR);
//!
//!     // Configure the C adapter: declare which items to export and how to name them.
//!     let cbindgen = prebindgen::lang::Cbindgen::new()
//!         .source_module(pq!(example_flat))
//!         .free_memory_function("example_free")
//!         .mangle_type_name(|base| format!("{base}_t"))
//!         .mangle_destructor(|base| format!("{base}_drop"))
//!         .mangle_function(|n| n.to_string())
//!         .opaque_ptr(pq!(Calculator))
//!         .function(pq!(calculator_new))
//!         .function(pq!(calculator_get_value)).panic();
//!
//!     // Resolve types, then write the Rust file of `extern "C"` wrappers.
//!     let generation = prebindgen::core::Registry::from_items(source.items_all())
//!         .unwrap()
//!         .resolve(cbindgen)
//!         .unwrap();
//!     let bindings_file = generation.write_rust("example_flat.rs").unwrap();
//!
//!     // Pass the generated file to cbindgen for C header generation.
//!     generate_c_headers(&bindings_file);
//! }
//! ```
//!
//! Include the generated Rust file in your crate to build the static or dynamic library:
//!
//! ```rust,ignore
//! // lib.rs
//! include!(concat!(env!("OUT_DIR"), "/example_flat.rs"));
//! ```
//!

/// File name for storing the crate name
const CRATE_NAME_FILE: &str = "crate_name.txt";

/// File name for storing enabled Cargo features collected in build.rs
const FEATURES_FILE: &str = "features.txt";

/// Default group name for items without explicit group name
pub const DEFAULT_GROUP_NAME: &str = "default";

pub(crate) mod api;
pub(crate) mod codegen;

pub use crate::api::{
    buildrs::{
        get_all_features, get_enabled_features, get_prebindgen_out_dir, init_prebindgen_out_dir,
        is_feature_enabled,
    },
    record::SourceLocation,
    source::Source,
    utils::{edition::RustEdition, target_triple::TargetTriple},
};

/// Not part of the public API — referenced by the [`ident!`] macro expansion
/// so callers don't need their own `proc-macro2` dependency just to build a
/// `Span`, and by the `lang::jnigen` decl macros (`ptr_class!`, `fun!`, …) to
/// parse a bare type token into a concrete `syn::Type`.
#[doc(hidden)]
pub mod __macro_support {
    pub use proc_macro2;

    pub fn parse_type(s: &str) -> ::syn::Type {
        ::syn::parse_str(s).unwrap_or_else(|e| panic!("prebindgen: invalid type `{s}`: {e}"))
    }

    pub fn parse_path(s: &str) -> ::syn::Path {
        ::syn::parse_str(s).unwrap_or_else(|e| panic!("prebindgen: invalid path `{s}`: {e}"))
    }

    pub fn parse_expr(s: &str) -> ::syn::Expr {
        ::syn::parse_str(s).unwrap_or_else(|e| panic!("prebindgen: invalid expression `{s}`: {e}"))
    }

    /// Parse a `sig!((params) -> Ret)` body: `s` is the token text between
    /// the macro's outer parens plus the optional `-> Ret` tail, e.g.
    /// `"(s: & Summary, verbose: bool) -> String"`. Wrapped into a full fn
    /// item signature under a placeholder name (replaced by the declaring
    /// decl's fn ident at synthesis time).
    pub fn parse_signature(s: &str) -> ::syn::Signature {
        let full = format!("fn __sig {s}");
        ::syn::parse_str::<::syn::ItemFn>(&format!("{full} {{ unimplemented!() }}"))
            .map(|f| f.sig)
            .unwrap_or_else(|e| panic!("prebindgen: invalid signature `sig!({s})`: {e}"))
    }
}

/// Build a `syn::Ident` from a bare identifier token. Unlike
/// `syn::parse_quote!`, this always yields the concrete type `syn::Ident` —
/// there's no external context needed to infer it — so it can be passed
/// directly into a generic `impl Into<T>` parameter without hitting rustc's
/// "type annotations needed" ambiguity. `syn::parse_quote!`'s output type
/// has to be pinned by a *concrete* parameter type to infer successfully; a
/// generic `impl Into<T>` bound doesn't give it anything to unify against.
///
/// This is what powers the `lang::jnigen` [`fun!`](crate::fun) decl macro —
/// see that macro (and `ptr_class!`/`enum_class!`/`data_class!`/`value_class!`,
/// which apply the same trick to `syn::Type`) for the primary way this
/// crate's builders are fed bare Rust names today.
///
/// ```
/// let _: syn::Ident = prebindgen::ident!(z_thing_name);
/// ```
#[macro_export]
macro_rules! ident {
    ($name:ident) => {
        ::syn::Ident::new(
            stringify!($name),
            $crate::__macro_support::proc_macro2::Span::call_site(),
        )
    };
}

/// Registry-based, **language-agnostic** converter pipeline.
///
/// This module is the language-neutral core of prebindgen: it turns a stream of
/// `#[prebindgen]` items into generated Rust FFI bindings plus a fully resolved
/// table of type converters. It has no knowledge of any particular destination
/// language — C, JNI/Kotlin, Swift, Python, etc. all plug in the same way.
///
/// # The plug-in point
///
/// Implement the [`Prebindgen`](core::Prebindgen) trait once per destination
/// language. The trait teaches the pipeline two things:
///
/// * **How the language represents Rust types on the wire** — the
///   `on_input_type_rank_0..3` / `on_output_type_rank_0..3` methods return a
///   [`ConverterImpl`](core::ConverterImpl) (a generated converter fn plus its
///   wire type) for each required type.
/// * **What wrapper code to emit per item** — `on_function` / `on_struct` /
///   `on_enum` / `on_const`.
///
/// Everything language-specific that must travel through the pipeline rides in
/// the back-end's chosen [`Metadata`](core::Prebindgen::Metadata) type (a JNI
/// back-end's Kotlin class names and exception info, a C back-end's header
/// names, …). It is set on each converter, propagated into the registry's
/// [`TypeEntry`](core::TypeEntry), and read back by the back-end's own emitter —
/// no side channels. Back-ends needing no extras leave it at the default `()`.
///
/// # Flow
///
/// 1. [`Registry::from_items`](core::Registry::from_items) indexes the
///    `(syn::Item, SourceLocation)` stream (typically [`Source::items_all`]).
/// 2. [`Registry::resolve`](core::Registry::resolve) resolves every required
///    type via your back-end, yielding a [`Generation`](core::Generation);
///    its `write_rust` (and adapter-specific `write_*`) methods emit the
///    artifacts.
/// 3. The back-end produces any secondary artifacts (C headers, Kotlin sources,
///    …) by walking the resolved [`Registry`](core::Registry).
///
/// # Universality, by example
///
/// The same machinery serves very different languages:
///
/// * **C / cbindgen back-end:** wire types are raw pointers and primitive C
///   types; converters are thin transmutes; `pre_stages` are usually empty
///   (errors surface as return codes).
/// * **JNI / Kotlin back-end:** wire types are JNI handles (`jlong`,
///   `JObject`); converters marshal across the JVM boundary; `pre_stages`
///   carry fallible steps whose `Err` arms throw JVM exceptions (the exception
///   info lives in that back-end's `Metadata`).
///
/// Both adapters ship in [`mod@lang`]: the C / cbindgen back-end
/// ([`lang::Cbindgen`]) and the JNI / Kotlin back-end ([`lang::JniGen`]).
pub mod core {
    pub use crate::api::core::{
        ConverterImpl, Direction, Generation, Gravestone, NicheSlot, Niches, Prebindgen, Registry,
        ScanError, Stage, Transmute, TypeEntry, TypeKey, WriteRustError,
    };
}

/// Runtime traits implemented by inline-opaque (`value_opaque`) FFI counterpart
/// types; see [`core::Transmute`] / [`core::Gravestone`]. Re-exported at the crate
/// root because the `extern "C"` converters emitted by [`lang::Cbindgen`]
/// reference them as `::prebindgen::Transmute` / `::prebindgen::Gravestone`.
pub use crate::api::core::gravestone::{Gravestone, Transmute};
/// Root re-export of [`lang::matching`] so the ignore-predicate constructor
/// sits next to the decl macros it composes with
/// (`.ignore(matching(|n| …))`, like `.ignore(fun!(…))`).
pub use crate::api::lang::jnigen::matching;

/// Destination-language adapters implementing [`core::Prebindgen`].
///
/// [`lang::Cbindgen`] is the C / cbindgen adapter: it turns a flat
/// `#[prebindgen]` library into a Rust file ready for `cbindgen` to parse into
/// a C header plus a static / dynamic library. Items are opt-in via its
/// declaration builder.
///
/// [`lang::JniGen`] is the JNI / Kotlin adapter: it turns a flat
/// `#[prebindgen]` library into a Rust file of JNI `extern "C"` wrappers plus
/// a fan-out of generated Kotlin sources (typed-handle classes, data/enum
/// classes, exception classes).
pub mod lang {
    pub use crate::api::lang::{
        cbindgen::{snake_case, Cbindgen},
        jnigen::{
            box_jboolean, box_jbyte, box_jchar, box_jdouble, box_jfloat, box_jint, box_jlong,
            box_jshort, decode_byte_array, decode_string, encode_byte_array, encode_string,
            matching, null_byte_array, null_string, CachedIfaceMethod, ClassDecl, ConstDecl,
            ConvertDecl, ConvertSourceDecl, DataClassDecl, EnumClassDecl, ExpandDecl,
            ExpandParamDecl, ExpandReturnDecl, FieldDecl, FunctionDecl, IgnoreDecl,
            JniBindingError, JniGen, KotlinFile, PackageDecl, PtrClassDecl, ValueClassDecl,
            WriteKotlinError,
        },
    };
}

pub mod utils {
    #[doc(hidden)]
    pub use crate::api::utils::jsonl::{read_jsonl_file, write_to_jsonl_file};
    pub use crate::api::utils::target_triple::TargetTriple;
}

#[doc(hidden)]
pub use crate::api::record::Record;
#[doc(hidden)]
pub use crate::api::record::RecordKind;
