use super::*;
/// Collect source helper and representation-type names needed by `Flat::parse_selected`.
pub fn source_items<'a>(
    inputs: &[JniInput<JniInputRequest, JniOutputRequest>],
    outputs: &[JniOutput<JniInputRequest, JniOutputRequest>],
    errors: &[(syn::Type, String)],
    functions: impl IntoIterator<Item = &'a DeclJniFunction>,
) -> Vec<String> {
    fn input(tree: &JniInput<JniInputRequest, JniOutputRequest>, names: &mut Vec<String>) {
        match tree {
            JniInput::Request(JniInputRequest::EnumFields(variants)) => {
                for child in variants.values().flat_map(|v| v.values()) {
                    input(child, names)
                }
            }
            JniInput::Request(JniInputRequest::Conversion(d)) => {
                prebindgen_declarations::conversion_source_items(d, names)
            }
            JniInput::Unary {
                operation: JniInputUnary::Convert { conversion },
                ..
            } => {
                if let prebindgen_flat::ValueSource::Function(name) = &conversion.source {
                    names.push(name.clone())
                }
            }
            JniInput::Request(JniInputRequest::Callback(declaration)) => {
                for tree in declaration.params.values() {
                    output(tree, names)
                }
                if let Some(tree) = &declaration.ret {
                    input(tree, names)
                }
            }
            JniInput::Request(JniInputRequest::Call(name)) => names.push(name.clone()),
            JniInput::Branch {
                operation: JniInputBranch::Constructor { function },
                ..
            } => names.push(function.clone()),
            JniInput::Request(JniInputRequest::ForType { input: child, .. }) => input(child, names),
            _ => {}
        }
        prebindgen_declarations::input_children(tree, names, &mut input, &mut output);
    }
    fn output(tree: &JniOutput<JniInputRequest, JniOutputRequest>, names: &mut Vec<String>) {
        match tree {
            JniOutput::Request(JniOutputRequest::SequenceCallback { output: child, .. })
            | JniOutput::Request(JniOutputRequest::ForType { output: child, .. }) => {
                output(child, names)
            }
            JniOutput::Request(JniOutputRequest::EnumFields(variants)) => {
                for child in variants.values().flat_map(|v| v.values()) {
                    output(child, names)
                }
            }
            JniOutput::Request(JniOutputRequest::Conversion(d)) => {
                prebindgen_declarations::conversion_source_items(d, names)
            }
            JniOutput::Unary {
                operation: JniOutputUnary::Convert { conversion },
                ..
            } => {
                if let prebindgen_flat::ValueSource::Function(name) = &conversion.source {
                    names.push(name.clone())
                }
            }
            JniOutput::Unary {
                operation: JniOutputUnary::Getter { function },
                ..
            }
            | JniOutput::Unary {
                operation: JniOutputUnary::Into { function },
                ..
            } => names.push(function.clone()),
            JniOutput::Unary {
                operation: JniOutputUnary::Result { error: Some(name) },
                ..
            } => names.push(name.clone()),
            JniOutput::Request(JniOutputRequest::NamedFields { overrides, .. }) => {
                for child in overrides.values() {
                    output(child, names)
                }
            }
            _ => {}
        }
        prebindgen_declarations::output_children(tree, names, &mut input, &mut output);
    }
    prebindgen_declarations::collect_source_items(
        inputs,
        outputs,
        errors,
        functions,
        &mut input,
        &mut output,
    )
}
