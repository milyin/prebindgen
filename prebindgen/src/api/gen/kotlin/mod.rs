//! Kotlin code generator — a self-contained declaration model + renderer
//! (KotlinPoet-style "spec model with raw code bodies").
//!
//! Declarations (files, classes, functions, properties, parameters, types,
//! annotations) are typed model values; imports and formatting derive from
//! the model. Statement **bodies** stay raw Kotlin text structured through
//! the indentation-aware [`Code`] builder.
//!
//! This module is deliberately independent: it must not import anything
//! from `api::lang` or `api::core` — it receives model values and strings,
//! and produces source text and file paths. Language back-ends (jnigen)
//! build the model; this module renders it.

pub(crate) mod code;
pub(crate) mod file;
pub(crate) mod model;
pub(crate) mod render;
pub(crate) mod types;

#[cfg(test)]
mod tests;

pub use code::Code;
pub use file::{merge_files, write_files, WriteKotlinError};
pub use model::{
    ClassKind, KtClass, KtCtorParam, KtDecl, KtEnumEntry, KtFile, KtFun, KtParam, KtProperty, Vis,
};
pub use types::KtType;
