//! Code generation utilities for transforming Rust function definitions into FFI stubs.
//!
//! This module contains all the logic for:
//! - Parsing and validating function signatures for FFI compatibility
//! - Generating `#[no_mangle] extern "C"` wrapper functions
//! - Handling type transformations and validations
//! - Creating appropriate parameter names and call arguments
//! - Processing feature flags (`#[cfg(feature="...")]`) in generated code

mod cfg_expr;
mod process_features;
mod replace_types;

// Re-export the main functions
pub(crate) use process_features::process_features;
#[allow(unused_imports)]
pub(crate) use replace_types::replace_types_in_file;
pub(crate) use replace_types::{
    convert_to_stub, generate_standard_allowed_prefixes, replace_types_in_item,
    replace_types_in_signature, generate_type_assertions,
};
