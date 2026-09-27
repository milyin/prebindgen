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
//! stream ships in [`prebindgen-flat`](https://docs.rs/prebindgen-flat); the
//! language adapters — [`prebindgen-c`](https://docs.rs/prebindgen-c) and
//! [`prebindgen-jni`](https://docs.rs/prebindgen-jni) — read the model and
//! generate the binding, writing through
//! [`prebindgen-tools`](https://docs.rs/prebindgen-tools).
//!
//! ## Usage example
//!
//! See also example projects on <https://github.com/milyin/prebindgen/tree/main/examples>
//!
//! See also the prebindgen-proc-macro documentation for details on how to use the `#[prebindgen]` macro:
//! <https://docs.rs/prebindgen-proc-macro/latest/prebindgen_proc_macro/>
//!
//! The `covertest-kotlin` and `perftest-kotlin` workspace examples are the
//! references for the JNI/Kotlin path; `example-cbindgen` and `perftest-c`
//! for C.
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
//! ### 2. C binding crate
//!
//! Depend on the common FFI library (as both a normal and a build dependency) and
//! drive the `prebindgen-c` crate's `CbindgenBuilder` adapter from `build.rs`:
//!
//! ```rust,ignore
//! // example-cbindgen/build.rs
//! use syn::parse_quote as pq;
//!
//! fn main() {
//!     let cbindgen = prebindgen_c::Cbindgen::builder()
//!         .source(example_flat::PREBINDGEN_OUT_DIR)
//!         .free_memory_function("example_free")
//!         .mangle_type_name(|base| format!("{base}_t"))
//!         .opaque_ptr(pq!(Calculator))
//!         .function(pq!(calculator_new))
//!         .function(pq!(calculator_get_value)).panic();
//!
//!     // Generate, then write the Rust file of `extern "C"` wrappers.
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

pub mod utils {
    #[doc(hidden)]
    pub use crate::api::utils::jsonl::{read_jsonl_file, write_to_jsonl_file};
    pub use crate::api::utils::target_triple::TargetTriple;
}

#[doc(hidden)]
pub use crate::api::record::Record;
#[doc(hidden)]
pub use crate::api::record::RecordKind;
