use super::*;
/// Collect source names; all recursive traversal follows structural edges.
pub fn source_items<'a>(
    inputs: &[JniInput<JniInputRequest, JniOutputRequest>],
    outputs: &[JniOutput<JniInputRequest, JniOutputRequest>],
    errors: &[(syn::Type, String)],
    functions: impl IntoIterator<Item = &'a DeclJniFunction>,
) -> Vec<String> {
    fn input(tree: &JniInput<JniInputRequest, JniOutputRequest>, names: &mut Vec<String>) {
        match tree.pending_request() {
            Some(JniInputRequest::Conversion(declaration)) => {
                prebindgen_declarations::conversion_source_items(declaration, names)
            }
            Some(JniInputRequest::Call(name)) => names.push(name.clone()),
            _ => {}
        }
        match tree {
            JniInput::Unary {
                operation:
                    prebindgen_declarations::Operation::Resolved(JniInputUnary::Convert { conversion }),
                ..
            } => {
                if let prebindgen_flat::ValueSource::Function(name) = &conversion.source {
                    names.push(name.clone())
                }
            }
            JniInput::Branch {
                operation:
                    prebindgen_declarations::Operation::Resolved(JniInputBranch::Constructor {
                        function,
                    }),
                ..
            } => names.push(function.clone()),
            _ => {}
        }
        prebindgen_declarations::input_children(tree, names, &mut input, &mut output);
    }
    fn output(tree: &JniOutput<JniInputRequest, JniOutputRequest>, names: &mut Vec<String>) {
        if let Some(JniOutputRequest::Conversion(declaration)) = tree.pending_request() {
            prebindgen_declarations::conversion_source_items(declaration, names)
        }
        match tree {
            JniOutput::Unary {
                operation:
                    prebindgen_declarations::Operation::Resolved(JniOutputUnary::Convert { conversion }),
                ..
            } => {
                if let prebindgen_flat::ValueSource::Function(name) = &conversion.source {
                    names.push(name.clone())
                }
            }
            JniOutput::Unary {
                operation:
                    prebindgen_declarations::Operation::Resolved(
                        JniOutputUnary::Getter { function } | JniOutputUnary::Into { function },
                    ),
                ..
            } => names.push(function.clone()),
            JniOutput::Unary {
                operation:
                    prebindgen_declarations::Operation::Resolved(JniOutputUnary::Result {
                        error: Some(name),
                    }),
                ..
            } => names.push(name.clone()),
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
