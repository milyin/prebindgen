//! # prebindgen
//!
//! A system for separating common FFI interface implementation from language-specific binding generation.
//!
//! ## Problem
//!
//! When creating Rust libraries that need to expose FFI interfaces to multiple languages, you face a dilemma:
//! - `#[no_mangle] extern "C"` functions can only be defined in `cdylib`/`staticlib` crates
//! - If you need bindings for multiple languages, you must either:
//!   - Generate all bindings from the same crate (tight coupling)
//!   - Manually duplicate FFI functions in each language-specific crate (code duplication)
//!
//! ## Solution
//!
//! `prebindgen` solves this by generating `#[no_mangle] extern "C"` source code from a common Rust library crate.
//! Language-specific binding crates can then include this generated code and pass it to their respective
//! binding generators (cbindgen, csbindgen, etc.).
//!
//! ## How to Use
//!
//! ### 1. In the Common FFI Library Crate
//!
//! Mark structures and functions that are part of the FFI interface with the `prebindgen` macro:
//!
//! ```rust
//! use prebindgen_proc_macro::{prebindgen, prebindgen_out_dir};
//!
//! // Declare a public constant with the path to prebindgen data:
//! pub const PREBINDGEN_OUT_DIR: &str = prebindgen_out_dir!();
//!
//! // Group structures and functions for selective handling
//! #[prebindgen("structs")]
//! #[repr(C)]
//! pub struct MyStruct {
//!     pub field: i32,
//! }
//!
//! #[prebindgen("functions")]
//! pub fn my_function(arg: i32) -> i32 {
//!     arg * 2
//! }
//! ```
//!
//! Call [`init_prebindgen_out_dir`] in the crate's `build.rs`:
//!
//! ```rust,no_run
//! use prebindgen::init_prebindgen_out_dir;
//!
//! fn main() {
//!     init_prebindgen_out_dir();
//! }
//! ```
//!
//! ### 2. In Language-Specific FFI Binding Crates
//!
//! Add the common FFI library to build dependencies in `Cargo.toml`:
//!
//! ```toml
//! [build-dependencies]
//! my_common_ffi = { path = "../my_common_ffi" }
//! prebindgen = "0.1"
//! ```
//!
//! In the binding crate's `build.rs`:
//!
//! ```rust,ignore
//! use prebindgen::{Source, batching::ffi_converter, filter_map::feature_filter, collect::Destination};
//! use itertools::Itertools;
//!
//! fn main() {
//!     // Create a source from the common FFI crate's prebindgen data
//!     let source = Source::new(my_common_ffi::PREBINDGEN_OUT_DIR);
//!
//!     // Process items with filtering and conversion
//!     let destination = source
//!         .items_all()
//!         .filter_map(feature_filter::Builder::new()
//!             .disable_feature("experimental")
//!             .enable_feature("std")
//!             .build()
//!             .into_closure())
//!         .batching(ffi_converter::Builder::new(source.crate_name())
//!             .allowed_prefix("libc::")
//!             .allowed_prefix("core::")
//!             .build()
//!             .into_closure())
//!         .collect::<Destination>();
//!
//!     // Write generated FFI code to file
//!     let bindings_file = destination.write("ffi_bindings.rs");
//!
//!     // Pass the generated file to cbindgen for C header generation
//!     // generate_c_headers(&bindings_file);
//! }
//! ```
//!
//! Include the generated Rust files in your project:
//!
//! ```rust,ignore
//! // In your lib.rs
//! include!(concat!(env!("OUT_DIR"), "/ffi_bindings.rs"));
//! ```
//!

/// File name for storing the crate name
const CRATE_NAME_FILE: &str = "crate_name.txt";

/// Default group name for items without explicit group name
pub const DEFAULT_GROUP_NAME: &str = "default";

pub(crate) mod api;
pub(crate) mod codegen;
pub(crate) mod utils;

pub use crate::api::buildrs::init_prebindgen_out_dir;
pub use crate::api::source::Source;
pub use crate::api::record::SourceLocation;

/// Filters for sequences of (syn::Item, SourceLocation) called by `itertools::batching`
pub mod batching {
    pub mod ffi_converter {
        pub use crate::api::batching::ffi_converter::Builder;
    }
    pub use crate::api::batching::ffi_converter::FfiConverter;
}

/// Filters for sequences of (syn::Item, SourceLocation) called by `filter_map`
pub mod filter_map {
    pub use crate::api::filter_map::struct_align::struct_align;
    pub use crate::api::filter_map::feature_filter::FeatureFilter;
    pub mod feature_filter {
        pub use crate::api::filter_map::feature_filter::Builder;
    }
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

/// Collectors for sequences of (syn::Item, SourceLocation) called by `collect`
pub mod collect {
    pub use crate::api::collect::destination::Destination;
}

#[doc(hidden)]
pub use crate::api::record::Record;
#[doc(hidden)]
pub use crate::api::record::RecordKind;
#[doc(hidden)]
pub use crate::api::buildrs::get_prebindgen_out_dir;

/// Macro for setting up doctest environment with source_ffi module
#[doc(hidden)]
#[macro_export]
macro_rules! doctest_setup {
    () => {
        use prebindgen_proc_macro::prebindgen_out_dir;
        let fallback_dir = std::env::temp_dir().join("prebindgen_fallback");
        std::fs::create_dir_all(&fallback_dir).unwrap();
        std::fs::write(fallback_dir.join("crate_name.txt"), "source_ffi").unwrap();
        // Set the OUT_DIR environment variable to point to our fallback directory
        unsafe { std::env::set_var("OUT_DIR", &fallback_dir); }
        mod source_ffi {
            use prebindgen_proc_macro::prebindgen_out_dir;
            pub const PREBINDGEN_OUT_DIR: &str = prebindgen_out_dir!();
        }
    };
}