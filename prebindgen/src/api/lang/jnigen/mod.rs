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
//!      exception classes, the centralized `JNINative` holder).

pub mod jni;
pub(crate) mod util;

pub use jni::{
    box_jboolean, box_jbyte, box_jchar, box_jdouble, box_jfloat, box_jint, box_jlong, box_jshort,
    decode_byte_array, decode_string, encode_byte_array, encode_string, null_byte_array,
    null_string, CachedIfaceMethod, ClassDecl, ConstDecl, DataClassDecl, EnumClassDecl,
    FunctionDecl, GenericTypeWrapperDecl, JniBindingError, JniGen, PackageDecl, PtrClassDecl,
    ScalarTypeWrapperDecl, ValueClassDecl, WireBody,
};

// Kotlin emission types now live in the standalone generator module
// (`api::gen::kotlin`); re-exported here so the public `lang::` surface is
// unchanged (`KotlinFile` aliases the model's `KtFile`).
pub use crate::api::gen::kotlin::KtFile as KotlinFile;
pub use crate::api::gen::kotlin::WriteKotlinError;
