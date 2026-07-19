//! Language adapters that implement [`crate::api::core::prebindgen::Prebindgen`].
//!
//! Each submodule is a destination-language back-end built on the
//! language-agnostic `core` pipeline.

#[cfg(feature = "unstable-cbindgen")]
pub mod cbindgen;
pub mod jnigen;
