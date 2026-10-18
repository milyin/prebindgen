//! Reusable boundary conversions and their representation metadata.
//!
//! These helpers build [`prebindgen_flat::InputPlan`] and
//! [`prebindgen_flat::OutputPlan`] values. They do not interpret declarations or
//! select functions to export. Adapters may use the helpers during preparation
//! and their wire metadata during generation.
//!
//! Generated code calls `prebindgen-runtime` at runtime; this crate itself is
//! used only during binding generation.
#![warn(missing_docs)]

pub mod boxed;
pub mod value;
