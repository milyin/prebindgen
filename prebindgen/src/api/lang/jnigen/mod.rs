//! JNI / Kotlin language adapter — the [`JniGen`] back-end.
//!
//! Sibling of [`crate::api::lang::cbindgen`]: it implements the
//! language-agnostic [`crate::api::core::prebindgen::Prebindgen`] trait to
//! turn a flat `#[prebindgen]` library into a Rust file of JNI `extern "C"`
//! wrappers plus a fan-out of generated Kotlin sources.
//!
//! Pipeline:
//!   1. [`crate::api::core::registry::Registry::from_items`] scans a stream of
//!      `(syn::Item, SourceLocation)` (typically `source.items_all()`).
//!   2. [`crate::api::core::registry::Registry::write_rust`] resolves every
//!      required type via a configured [`JniGen`] and writes the generated
//!      Rust bindings file.
//!   3. [`jni::JniGen::write_kotlin`] walks the resolved registry to emit the
//!      secondary Kotlin artifacts (typed-handle classes, data/enum classes,
//!      exception classes, callback fun-interfaces, the centralized
//!      `JNINative` holder).

pub mod jni;
pub(crate) mod kotlin;
pub(crate) mod util;

pub use jni::{
    decode_byte_array, decode_string, encode_byte_array, encode_string, null_byte_array,
    null_string, JniBindingError, JniGen,
};
pub use kotlin::kotlin_ext::{KotlinFile, WriteKotlinError};
