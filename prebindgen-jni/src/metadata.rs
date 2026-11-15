//! Kotlin/JNI declarations stored on their Flat owners.
use super::*;
use prebindgen_conversions::InputConversion;
use std::rc::Rc;

/// Prepared boundary roots, including typed error converters and declarations
/// whose callback interfaces are emitted without a direct exported occurrence.
pub(crate) fn nodes<I: KotlinInputConversion, O: KotlinOutputConversion>(
    flat: &Flat<JniSchema<I, O>>,
) -> Result<Vec<(bool, &kotlin::JniNode)>> {
    let mut nodes = Vec::new();
    for source in flat.functions.values() {
        for binding in &source.metadata {
            let call = binding.call()?;
            for plan in &call.inputs {
                nodes.push((true, plan.input_node()));
            }
            let outcomes = call.output.output_outcomes();
            nodes.extend(
                std::iter::once(&outcomes.success)
                    .chain(outcomes.source_error.iter())
                    .chain(outcomes.binding_failure.iter())
                    .map(|node| (false, node)),
            );
            if let Some(output) = binding
                .metadata
                .error
                .as_deref()
                .and_then(|e| e.output.as_ref())
            {
                nodes.push((false, &output.output_outcomes().success));
            }
        }
    }
    for input in crate::callback::declarations(flat)? {
        nodes.push((true, input.input_node()));
    }
    Ok(nodes)
}

/// Metadata schema for a Kotlin/JNI binding model.
/// `InConverter` and `OutConverter` select the converters stored in each export.
/// Source parameter/return attachments are unit; type metadata keeps the JNI
/// representation. Default containers also accept custom converters through their
/// owned containers. No schema value is needed to construct Flat.
pub struct JniSchema<InConverter = JniInputConverter, OutConverter = JniOutputConverter>(
    std::marker::PhantomData<fn() -> (InConverter, OutConverter)>,
);
impl<InConverter, OutConverter> prebindgen_flat::Schema for JniSchema<InConverter, OutConverter> {
    type Parameter = ();
    type Return = ();
    type Module = JniModule;
    type Function = Vec<JniBinding<InConverter, OutConverter>>;
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
    /// Use bulk for materialized collection outputs with automatic transport.
    pub bulk_collection_outputs: bool,
}
/// Export and placement decisions belonging to one function.
#[derive(Clone, Debug, Default)]
pub struct JniFunction {
    /// This export delivers a returned sequence through a visitor/folder.
    pub callback_return: bool,
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
    /// Standalone callback interface attached to its canonical callback type.
    /// Configured selects this type's input default; later stages retain its recipe/converter.
    pub callback:
        Option<prebindgen_conversions::BindingState<(), crate::JniInput, crate::JniInputConverter>>,
    /// Input/output defaults stored on this type, including callback signatures.
    pub defaults:
        prebindgen_declarations::DeclTypeDefaults<crate::JniInputConfig, crate::JniOutputConfig>,
    /// Canonical value conversions selected for this source type.
    pub conversion: prebindgen_conversions::ValueConversions,
    pub(crate) class: Option<Rc<Class>>,
    pub(crate) enumeration: Option<Rc<JniEnum>>,
    pub(crate) errors: BTreeMap<bool, Rc<errors::ErrorBinding>>,
    pub(crate) error_class: Option<String>,
}

/// One JNI export attached to its original source function.
pub type JniBinding<I = JniInputConverter, O = JniOutputConverter> =
    prebindgen_declarations::DeclBinding<
        crate::JniInputConfig,
        crate::JniOutputConfig,
        crate::JniInput,
        crate::JniOutput,
        I,
        O,
        JniFunction,
    >;
