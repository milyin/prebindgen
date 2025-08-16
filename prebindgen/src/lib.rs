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
//! ### 1. In the Common FFI Library Crate (e.g., `example_ffi`)
//!
//! Mark structures and functions that are part of the FFI interface with the `prebindgen` macro:
//!
//! ```rust,ignore
//! // example-ffi/src/lib.rs
//! use prebindgen_proc_macro::prebindgen;
//!
//! // Export path to prebindgen output directory
//! const PREBINDGEN_OUT_DIR: &str = prebindgen_proc_macro::prebindgen_out_dir!();
//!
//! // Group structures and functions for selective handling
//! #[prebindgen]
//! #[repr(C)]
//! pub struct MyStruct {
//!     pub field: i32,
//! }
//!
//! #[prebindgen]
//! pub fn my_function(arg: i32) -> i32 {
//!     arg * 2
//! }
//! ```
//!
//! Call [`init_prebindgen_out_dir`] in the crate's `build.rs`:
//!
//! ```rust,no_run
//! // example-ffi/build.rs
//! prebindgen::init_prebindgen_out_dir();
//! ```
//!
//! ### 2. In Language-Specific FFI Binding Crate (named e.g. `example-cbindgen`)
//!
//! Add the common FFI library to build dependencies
//!
//! ```toml
//! # example-cbindgen/Cargo.toml
//! [dependencies]
//! example_ffi = { path = "../example_ffi" }
//!
//! [build-dependencies]
//! example_ffi = { path = "../example_ffi" }
//! prebindgen = "0.2"
//! cbindgen = "0.24"
//! ```
//! ```rust,ignore
//! // example-cbindgen/build.rs
//! use prebindgen::{Source, batching::ffi_converter, collect::Destination};
//! use itertools::Itertools;
//!
//! fn main() {
//!     // Create a source from the common FFI crate's prebindgen data
//!     let source = Source::new(example_ffi::PREBINDGEN_OUT_DIR);
//!
//!     // Process items with filtering and conversion
//!     let destination = source
//!         .items_all()
//!         .batching(crate::batching::feature_filter::Builder::new()
//!             .disable_feature("experimental")
//!             .enable_feature("std")
//!             .build()
//!             .into_closure())
//!         .batching(ffi_converter::Builder::new(source.crate_name())
//!             .edition(prebindgen::Edition::Edition2024)
//!             .strip_transparent_wrapper("std::mem::MaybeUninit")
//!             .build()
//!             .into_closure())
//!         .collect::<Destination>();
//!
//!     // Write generated FFI code to file
//!     let bindings_file = destination.write("ffi_bindings.rs");
//!
//!     // Pass the generated file to cbindgen for C header generation
//!     generate_c_headers(&bindings_file);
//! }
//! ```
//!
//! Include the generated Rust files in your project to build the static or dynamic library itself:
//!
//! ```rust,ignore
//! // lib.rs
//! include!(concat!(env!("OUT_DIR"), "/ffi_bindings.rs"));
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
pub(crate) mod utils;

pub use crate::api::buildrs::get_prebindgen_out_dir;
pub use crate::api::buildrs::init_prebindgen_out_dir;
pub use crate::api::record::SourceLocation;
pub use crate::api::source::Source;
pub use crate::utils::edition::RustEdition;
pub use konst; // Re-export konst for compile-time comparisons in generated code

/// Filters for sequences of (syn::Item, SourceLocation) called by `itertools::batching`
pub mod batching {
    pub mod ffi_converter {
        pub use crate::api::batching::ffi_converter::Builder;
    }
    pub use crate::api::batching::feature_filter::FeatureFilter;
    pub use crate::api::batching::ffi_converter::FfiConverter;
    pub mod feature_filter {
        pub use crate::api::batching::feature_filter::Builder;
    }
}

/// Filters for sequences of (syn::Item, SourceLocation) called by `filter_map`
pub mod filter_map {
    pub use crate::api::filter_map::struct_align::struct_align;
}

/// Filters for sequences of (syn::Item, SourceLocation) called by `map`
pub mod map {
    pub use crate::api::map::strip_derive::StripDerives;
    pub mod strip_derive {
        pub use crate::api::map::strip_derive::Builder;
    }
    pub use crate::api::map::strip_macro::StripMacros;
    pub mod strip_macro {
        pub use crate::api::map::strip_macro::Builder;
    }
    pub use crate::api::map::replace_types::ReplaceTypes;
    pub mod replace_types {
        pub use crate::api::map::replace_types::Builder;
    }
}

/// Collectors for sequences of (syn::Item, SourceLocation) produced by `collect`
pub mod collect {
    pub use crate::api::collect::destination::Destination;
}

#[doc(hidden)]
pub use crate::api::record::Record;
#[doc(hidden)]
pub use crate::api::record::RecordKind;
