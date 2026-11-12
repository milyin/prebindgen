//! Kotlin/JNI declarations stored on their Flat owners.
use super::*;
use prebindgen_flat::InputConversion;
use std::rc::Rc;

/// Prepared boundary roots, including typed error converters and declarations
/// whose callback interfaces are emitted without a direct exported occurrence.
pub(crate) fn nodes<I: KotlinInputConversion, O: KotlinOutputConversion>(
    flat: &Flat<JniSchema<I, O>>,
) -> Vec<(bool, &kotlin::JniNode)> {
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
            if let Some(output) = &error.output {
                nodes.push((false, &output.output_outcomes().success));
            }
        }
    }
    for input in &flat.metadata.callbacks {
        nodes.push((true, input.input_node()));
    }
    nodes
}

/// Metadata schema for a Kotlin/JNI binding model.
/// `I` and `O` select occurrence converter types while declaration metadata keeps
/// the JNI representation. Defaults also accept custom converters through their
/// owned containers. No schema value is needed to construct Flat.
pub struct JniSchema<I = JniInput, O = JniOutput>(std::marker::PhantomData<fn() -> (I, O)>);
impl<I, O> prebindgen_flat::Schema for JniSchema<I, O> {
    type Input = I;
    type Output = O;
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
    /// Resolved standalone callback roots, consumed when plans are compiled.
    pub callback_recipes: Vec<crate::operations::Input>,
    /// Use bulk for materialized collection outputs with automatic transport.
    pub bulk_collection_outputs: bool,
}
/// Export and placement decisions belonging to one function.
#[derive(Clone, Debug, Default)]
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
