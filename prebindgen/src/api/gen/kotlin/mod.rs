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
pub(crate) mod expr;
pub(crate) mod file;
pub(crate) mod model;
pub(crate) mod render;
pub(crate) mod slot;
pub(crate) mod types;

#[cfg(test)]
mod tests;

pub use code::Code;
// Re-exported for #193 / #199, which are the first consumers; `ExprSlot` is
// already used by the model's mechanical bridges.
#[allow(unused_imports)]
pub use expr::{
    fill_hole, free_names, has_hole, substitute, Binder, BindingId, ExprArena, KtExpr, KtLiteral,
    KtName, KtPattern, KtStmt, Spelling,
};
pub use file::{merge_files, write_files, WriteKotlinError};
pub use model::{
    ClassKind, KtBody, KtClass, KtCtorParam, KtDecl, KtEnumEntry, KtFile, KtFun, KtFunInterface,
    KtParam, KtProperty, Vis,
};
#[allow(unused_imports)]
pub use slot::{
    AccessorTree, AnnotationSlot, ExprSlot, KtAccessor, KtAnnotation, PropertyValue,
    StaticAnnotationText,
};
pub use types::{ImportSet, KtType};
