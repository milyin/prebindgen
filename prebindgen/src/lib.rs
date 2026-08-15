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
//! `prebindgen` solves this by generating language-specific proxy code from a common
//! Rust library crate. This crate is the base of that pipeline: it reads what
//! `#[prebindgen]` captured and hands out `(syn::Item, [`SourceLocation`])`
//! pairs through [`Source`] — nothing more. The flat model built over that
//! stream ships in the separate
//! [`prebindgen-flat`](https://docs.rs/prebindgen-flat) crate; the
//! language-neutral registry pipeline over the flat model — type resolution,
//! boundary expansion, Rust emission — ships in the separate
//! [`prebindgen-registry`](https://docs.rs/prebindgen-registry) crate. The
//! supported 0.5 surface is that pipeline plus the JNI/Kotlin `JniGenBuilder`
//! adapter, which ships in the separate
//! [`prebindgen-jni`](https://docs.rs/prebindgen-jni) crate.
//!
//! The C / cbindgen adapter is an experimental proof of concept, ships in the
//! separate `prebindgen-c` crate, and is not covered by the 0.5 semver
//! guarantee.
//!
//! ## Usage example
//!
//! See also example projects on <https://github.com/milyin/prebindgen/tree/main/examples>
//!
//! See also the prebindgen-proc-macro documentation for details on how to use the `#[prebindgen]` macro:
//! <https://docs.rs/prebindgen-proc-macro/latest/prebindgen_proc_macro/>
//!
//! ### Stable core and JNI/Kotlin path
//!
//! The supported workflow reads captured items with [`Source`], resolves them
//! through the separate `prebindgen-registry` crate's `Registry`, and
//! configures the separate `prebindgen-jni` crate's `JniGenBuilder` to emit
//! Rust JNI wrappers plus Kotlin sources. The `covertest-kotlin` and
//! `perftest-kotlin` workspace examples are the maintained references for
//! that path; see `prebindgen-jni`'s own crate docs for the full JNI/Kotlin
//! workflow and macro surface.
//!
//! ### 1. In the Common FFI Library Crate (e.g., `example_flat`)
//!
//! Mark the types and functions that form your FFI surface with the `prebindgen`
//! macro. The source crate stays **plain idiomatic Rust** — opaque handles are
//! ordinary types returned by value, fallible calls return `Result<T, E>`; the
//! language adapter does the C-ABI lowering, so there is no `#[repr(C)]` here.
//!
//! ```rust
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
//! ### 2. Experimental C binding crate
//!
//! Depend on the common FFI library (as both a normal and a build dependency) and
//! drive the experimental `prebindgen-c` crate's `CbindgenBuilder` adapter from
//! `build.rs`:
//!
//! ```toml
//! # example-cbindgen/Cargo.toml
//! [dependencies]
//! example-flat = { path = "../example-flat" }
//! prebindgen = "0.5"
//! prebindgen-c-runtime = "0.5"   # the generated converters reference its traits
//! konst = "0.3"      # the generated file emits a konst feature guard
//!
//! [build-dependencies]
//! example-flat = { path = "../example-flat" }
//! prebindgen = "0.5"
//! prebindgen-registry = "0.5"
//! prebindgen-c = "0.5"
//! cbindgen = "0.29"
//! syn = { version = "2", features = ["full"] }
//! ```
//! ```rust,ignore
//! // example-cbindgen/build.rs
//! use syn::parse_quote as pq;
//!
//! fn main() {
//!     // Configure the C adapter: point it at the items captured from the
//!     // common FFI crate, and declare which ones to export and how to name them.
//!     let cbindgen = prebindgen_c::Cbindgen::builder()
//!         .source(example_flat::PREBINDGEN_OUT_DIR)
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
//!     let bindings_file = cbindgen.build().unwrap().write_rust("example_flat.rs").unwrap();
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
//! ## Macros
//!
//! The declaration surface is built almost entirely from exported macros. The
//! language-neutral ones — the domain vocabulary shared by every adapter, plus
//! the syntax helpers they're built from — now live in the separate
//! `prebindgen-registry` crate:
//!
//! - Members & constants: `fun!`
//! - Conversions: `convert!`, `from!`, `try_from!`, `into!`, `try_into!`
//! - Boundary expansion: `expand_param!`, `expand_return!`, `fields!`
//!
//! **Syntax helpers** produce a bare `syn` node — `Type` / `Path` / `Expr` /
//! `Signature` / `Ident` — to hand to a declaration method that requires one.
//! They exist only to sidestep `syn::parse_quote!`'s type-inference ambiguity
//! (E0283) in a generic argument position, not to express a domain concept:
//! `ty!`, `path!`, `expr!`, `sig!`, `ident!`.
//!
//! The JNI/Kotlin-specific declaration macros — `package!`, `ptr_class!`,
//! `data_class!`, `enum_class!`, `sealed_class!`, `variant!`, `constant!` —
//! construct a typed `*Decl` for the Kotlin surface and live in the separate
//! `prebindgen-jni` crate, which hands the result to its `JniGenBuilder`.
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

// The flat model (`flat`, `shape`, `types_util`, `Element`, `Flat`, `TypeKey`,
// `Emit`, …) moved to the separate `prebindgen-flat` crate, which depends on
// this crate for [`Source`] / [`SourceLocation`] and parses the
// `(syn::Item, SourceLocation)` pairs [`Source`] hands out into one flat
// namespace — see that crate's docs for the model.
//
// The registry pipeline (`Registry`, `Prebindgen`, `decl`, `write`, `expand`,
// `unfold`, …) moved to the separate `prebindgen-registry` crate, which
// depends on this crate and `prebindgen-flat`, and re-exports the flat model
// (`flat`, `shape`, `types_util`) at its own root — see that crate's docs for
// the pipeline.
//
// The JNI / Kotlin adapter (`matching`, the `lang` module, `JniGenBuilder`, …)
// moved to the separate `prebindgen-jni` crate. It depends on this crate and
// `prebindgen-registry`, and re-exports its own decl macros' support types at
// its own root — see `prebindgen-jni`'s docs for the JNI / Kotlin workflow.

#[doc(hidden)]
pub mod layout {
    //! Capture-directory layout, shared by the proc-macro (writing) and
    //! [`Source`](crate::Source) (reading).
    pub use crate::api::layout::{
        capture_file_name, decode_group_dir_name, group_dir_name, MAX_COMPONENT_LEN,
    };
}

pub mod utils {
    #[doc(hidden)]
    pub use crate::api::utils::jsonl::{publish_file, read_jsonl_file, write_to_jsonl_file};
    pub use crate::api::utils::target_triple::TargetTriple;
}

#[doc(hidden)]
pub use crate::api::record::Record;
#[doc(hidden)]
pub use crate::api::record::RecordKind;
