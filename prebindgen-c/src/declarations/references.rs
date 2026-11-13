use super::*;
/// Collect source names; all recursive traversal follows structural edges.
pub fn source_items<'a>(
    inputs: &[CInput<CInputRequest, COutputRequest>],
    outputs: &[COutput<CInputRequest, COutputRequest>],
    errors: &[(syn::Type, String)],
    functions: impl IntoIterator<Item = &'a DeclCFunction>,
) -> Vec<String> {
    fn input(tree: &CInput<CInputRequest, COutputRequest>, names: &mut Vec<String>) {
        match tree.pending_request() {
            Some(CInputRequest::Conversion(declaration)) => {
                prebindgen_declarations::conversion_source_items(declaration, names)
            }
            Some(CInputRequest::Call(name)) => names.push(name.clone()),
            _ => {}
        }
        match tree {
            CInput::Unary {
                operation:
                    prebindgen_declarations::Operation::Resolved(CInputUnary::Convert { conversion }),
                ..
            } => {
                if let prebindgen_flat::ValueSource::Function(name) = &conversion.source {
                    names.push(name.clone())
                }
            }
            CInput::Branch {
                operation:
                    prebindgen_declarations::Operation::Resolved(CInputBranch::Constructor { function }),
                ..
            } => names.push(function.clone()),
            _ => {}
        }
        prebindgen_declarations::input_children(tree, names, &mut input, &mut output);
    }
    fn output(tree: &COutput<CInputRequest, COutputRequest>, names: &mut Vec<String>) {
        if let Some(COutputRequest::Conversion(declaration)) = tree.pending_request() {
            prebindgen_declarations::conversion_source_items(declaration, names)
        }
        match tree {
            COutput::Unary {
                operation:
                    prebindgen_declarations::Operation::Resolved(COutputUnary::Convert { conversion }),
                ..
            } => {
                if let prebindgen_flat::ValueSource::Function(name) = &conversion.source {
                    names.push(name.clone())
                }
            }
            COutput::Unary {
                operation:
                    prebindgen_declarations::Operation::Resolved(
                        COutputUnary::Getter { function } | COutputUnary::Into { function },
                    ),
                ..
            } => names.push(function.clone()),
            COutput::Unary {
                operation:
                    prebindgen_declarations::Operation::Resolved(COutputUnary::Result {
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
