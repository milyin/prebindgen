//! Kotlin/JNI declarations stored on their Flat owners.
use super::*;
use std::rc::Rc;

/// Metadata schema for a Kotlin/JNI binding model.
pub struct JniSchema;
impl prebindgen_flat::Schema for JniSchema {
    type Input = JniInput;
    type Output = JniOutput;
    type Module = JniModule;
    type Function = Option<JniFunction>;
    type Type = JniType;
    type Field = data::JniField;
    type Variant = ();
}
/// Settings shared by the generated Kotlin/JNI module.
#[derive(Clone, Debug, Default)]
pub struct JniModule {
    /// Root Kotlin package containing JNINative and shared infrastructure.
    pub package: String,
    /// Kotlin initialization statements in JNINative.
    pub native_init: String,
}
/// Export and placement decisions belonging to one function.
#[derive(Clone, Debug)]
pub struct JniFunction {
    /// Fully qualified destination package.
    pub package: String,
    /// Destination function name.
    pub name: String,
    /// Stable declaration order used for native entry-point numbering.
    pub order: usize,
    pub(crate) error: Option<Rc<errors::ErrorBinding>>,
}
/// Kotlin/JNI declaration information shared by occurrences of a Rust type.
#[derive(Clone, Debug, Default)]
pub struct JniType {
    pub(crate) class: Option<Rc<Class>>,
    pub(crate) enumeration: Option<Rc<JniEnum>>,
    pub(crate) errors: BTreeMap<bool, Rc<errors::ErrorBinding>>,
    pub(crate) error_class: Option<String>,
}
