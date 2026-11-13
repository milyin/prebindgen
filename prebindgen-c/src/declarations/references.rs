use super::*;
/// Collect source helper and representation-type names needed by `Flat::parse_selected`.
pub fn source_items<'a>(
    inputs: &[CInput<CInputRequest, COutputRequest>],
    outputs: &[COutput<CInputRequest, COutputRequest>],
    errors: &[(syn::Type, String)],
    functions: impl IntoIterator<Item = &'a DeclCFunction>,
) -> Vec<String> {
    fn input(tree: &CInput<CInputRequest, COutputRequest>, names: &mut Vec<String>) {
        match tree {
            CInput::Request(CInputRequest::EnumFields(variants)) => {
                for child in variants.values().flat_map(|v| v.values()) {
                    input(child, names)
                }
            }
            CInput::Request(CInputRequest::Conversion(d)) => {
                prebindgen_declarations::conversion_source_items(d, names)
            }
            CInput::Unary {
                operation: CInputUnary::Convert { conversion },
                ..
            } => {
                if let prebindgen_flat::ValueSource::Function(name) = &conversion.source {
                    names.push(name.clone())
                }
            }
            CInput::Request(CInputRequest::Callback(declaration)) => {
                for tree in declaration.params.values() {
                    output(tree, names)
                }
                if let Some(tree) = &declaration.ret {
                    input(tree, names)
                }
            }
            CInput::Request(CInputRequest::Call(name)) => names.push(name.clone()),
            CInput::Branch {
                operation: CInputBranch::Constructor { function },
                ..
            } => names.push(function.clone()),
            CInput::Request(CInputRequest::ForType { input: child, .. }) => input(child, names),
            _ => {}
        }
        prebindgen_declarations::input_children(tree, names, &mut input, &mut output);
    }
    fn output(tree: &COutput<CInputRequest, COutputRequest>, names: &mut Vec<String>) {
        match tree {
            COutput::Request(COutputRequest::SequenceCallback { output: child, .. })
            | COutput::Request(COutputRequest::ForType { output: child, .. }) => {
                output(child, names)
            }
            COutput::Request(COutputRequest::EnumFields(variants)) => {
                for child in variants.values().flat_map(|v| v.values()) {
                    output(child, names)
                }
            }
            COutput::Request(COutputRequest::Conversion(d)) => {
                prebindgen_declarations::conversion_source_items(d, names)
            }
            COutput::Unary {
                operation: COutputUnary::Convert { conversion },
                ..
            } => {
                if let prebindgen_flat::ValueSource::Function(name) = &conversion.source {
                    names.push(name.clone())
                }
            }
            COutput::Unary {
                operation: COutputUnary::Getter { function },
                ..
            }
            | COutput::Unary {
                operation: COutputUnary::Into { function },
                ..
            } => names.push(function.clone()),
            COutput::Unary {
                operation: COutputUnary::Result { error: Some(name) },
                ..
            } => names.push(name.clone()),
            COutput::Request(COutputRequest::NamedFields { overrides, .. }) => {
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
