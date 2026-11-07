//! Kotlin/JNI declarations stored on their Flat owners.
use super::*;
use prebindgen_flat::InputConversion;
use std::rc::Rc;

/// Prepared boundary roots, including typed error converters and declarations
/// whose callback interfaces are emitted without a direct exported occurrence.
pub(crate) fn nodes(flat: &Bindings) -> Vec<(bool, &kotlin::JniNode)> {
    let mut nodes = Vec::new();
    for function in flat.functions.values() {
        for param in &function.params {
            if let Some(plan) = &param.ty.slot {
                nodes.push((true, plan.input_node()));
            }
        }
        if let Some(plan) = &function.ret.slot {
            let outcomes = plan.output_outcomes();
            nodes.extend(
                std::iter::once(&outcomes.success)
                    .chain(outcomes.source_error.iter())
                    .chain(outcomes.binding_failure.iter())
                    .map(|node| (false, node)),
            );
        }
        if let Some(error) = function.metadata.as_ref().and_then(|m| m.error.as_deref()) {
            nodes.push((false, &error.output.output_outcomes().success));
        }
    }
    for input in &flat.metadata.callbacks {
        nodes.push((true, input.input_node()));
    }
    nodes
}

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
    /// Prepared callback declarations, including interfaces unused by exported functions.
    pub callbacks: Vec<JniInput>,
    /// Use bulk for materialized collection outputs with automatic transport.
    pub bulk_collection_outputs: bool,
}
/// Export and placement decisions belonging to one function.
#[derive(Clone, Debug)]
pub struct JniFunction {
    /// Fully qualified destination package.
    pub package: String,
    /// Destination function name.
    pub name: String,
    /// Declaration order used when rendering functions.
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
