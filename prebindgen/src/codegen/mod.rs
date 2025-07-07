//! Code generation utilities for transforming Rust function definitions into FFI stubs.
//!
//! This module contains all the logic for:
//! - Parsing and validating function signatures for FFI compatibility
//! - Generating `#[no_mangle] extern "C"` wrapper functions
//! - Handling type transformations and validations
//! - Creating appropriate parameter names and call arguments
//! - Processing feature flags (`#[cfg(feature="...")]`) in generated code

pub mod transform_function;
pub mod process_features;
pub mod replace_types;
pub mod cfg_expr;

// Re-export the main functions
pub use transform_function::{trim_implementation, create_stub_implementation};
pub use process_features::process_features;
pub use replace_types::{replace_types, generate_standard_allowed_prefixes};