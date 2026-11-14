use super::*;
/// Collect source names; all recursive traversal follows structural edges.
pub fn source_items<'a>(
    inputs: &[JniInputConfig],
    outputs: &[JniOutputConfig],
    errors: &[(syn::Type, String)],
    functions: impl IntoIterator<Item = &'a DeclJniFunction>,
) -> Vec<String> {
    fn input(tree: &JniInputConfig, names: &mut Vec<String>) {
        match tree {
            JniInputConfig::Leaf(JniInputLeafOp::ConfigConversion(declaration)) => {
                prebindgen_declarations::conversion_source_items(declaration, names)
            }
            JniInputConfig::Leaf(JniInputLeafOp::ConfigConstructor(name)) => {
                names.push(name.clone())
            }
            _ => {}
        }
        match tree {
            JniInputConfig::Unary {
                operation: JniInputUnary::Convert { conversion },
                ..
            } => {
                if let prebindgen_flat::ValueSource::Function(name) = &conversion.source {
                    names.push(name.clone())
                }
            }
            JniInputConfig::Branch {
                operation: JniInputBranch::Constructor { function },
                ..
            } => names.push(function.clone()),
            _ => {}
        }
        prebindgen_declarations::input_children(tree, names, &mut input, &mut output);
    }
    fn output(tree: &JniOutputConfig, names: &mut Vec<String>) {
        if let JniOutputConfig::Leaf(JniOutputLeafOp::ConfigConversion(declaration)) = tree {
            prebindgen_declarations::conversion_source_items(declaration, names)
        }
        match tree {
            JniOutputConfig::Unary {
                operation: JniOutputUnary::Convert { conversion },
                ..
            } => {
                if let prebindgen_flat::ValueSource::Function(name) = &conversion.source {
                    names.push(name.clone())
                }
            }
            JniOutputConfig::Unary {
                operation: JniOutputUnary::Getter { function } | JniOutputUnary::Into { function },
                ..
            } => names.push(function.clone()),
            JniOutputConfig::Unary {
                operation: JniOutputUnary::Result { error: Some(name) },
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
