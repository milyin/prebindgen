//! Reusable boundary conversions and their representation metadata.
//!
//! These helpers build [`InputPlan`] and
//! [`OutputPlan`] values. They do not interpret declarations or
//! select functions to export. Adapters may use the helpers during preparation
//! and their wire metadata during generation.
//!
//! Generated code calls `prebindgen-runtime` at runtime; this crate itself is
//! used only during binding generation.
#![warn(missing_docs)]

pub mod boxed;
pub mod value;

mod emission;
pub use emission::{render_template, template_parameter, TemplateContext};
mod plan;
pub use plan::*;
pub mod staged;
pub use staged::{InputFragment, InputWireFragment, OutputFragment, OutputWireFragment};

#[cfg(test)]
mod generation_tests;
