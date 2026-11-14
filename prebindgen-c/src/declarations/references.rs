use super::*;
/// Collect source names; all recursive traversal follows structural edges.
pub fn source_items<'a>(
    inputs: &[CInputConfig],
    outputs: &[COutputConfig],
    errors: &[(syn::Type, String)],
    functions: impl IntoIterator<Item = &'a DeclCFunction>,
) -> Vec<String> {
    fn input(tree: &CInputConfig, names: &mut Vec<String>) {
        match tree {
            CInputConfig::Leaf(CInputLeafOp::ConfigConversion(declaration)) => {
                prebindgen_declarations::conversion_source_items(declaration, names)
            }
            CInputConfig::Leaf(CInputLeafOp::ConfigConstructor(name)) => names.push(name.clone()),
            _ => {}
        }
        match tree {
            CInputConfig::Unary {
                operation: CInputUnary::Convert { conversion },
                ..
            } => {
                if let prebindgen_flat::ValueSource::Function(name) = &conversion.source {
                    names.push(name.clone())
                }
            }
            CInputConfig::Branch {
                operation: CInputBranch::Constructor { function },
                ..
            } => names.push(function.clone()),
            _ => {}
        }
        prebindgen_declarations::input_children(tree, names, &mut input, &mut output);
    }
    fn output(tree: &COutputConfig, names: &mut Vec<String>) {
        if let COutputConfig::Leaf(COutputLeafOp::ConfigConversion(declaration)) = tree {
            prebindgen_declarations::conversion_source_items(declaration, names)
        }
        match tree {
            COutputConfig::Unary {
                operation: COutputUnary::Convert { conversion },
                ..
            } => {
                if let prebindgen_flat::ValueSource::Function(name) = &conversion.source {
                    names.push(name.clone())
                }
            }
            COutputConfig::Unary {
                operation: COutputUnary::Getter { function } | COutputUnary::Into { function },
                ..
            } => names.push(function.clone()),
            COutputConfig::Unary {
                operation: COutputUnary::Result { error: Some(name) },
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
