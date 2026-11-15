//! Bulk is an inherited encoding, not a second sequence delivery policy.
use super::*;
use kotlin::JniNode;
use prebindgen_conversions::{InputConversion, WireNode};
use prebindgen_declarations::SequenceDelivery;

fn sequence(child: JniOutput) -> JniOutput {
    JniOutput::Unary {
        operation: JniOutputUnary::Sequence {
            delivery: SequenceDelivery::Value,
            mode: None,
        },
        child: Box::new(child),
    }
}
fn prepared() -> (JniBindings, JniOutputPlan<JniWire>) {
    let source = syn::parse_file("pub fn rows()->Vec<Vec<i64>>{}").unwrap();
    let model = JniGen::builder()
        .items(
            source
                .items
                .into_iter()
                .map(|item| (item, Default::default())),
        )
        .set_package_prefix("test")
        .package(package!().fun(fun!(rows)))
        .prepare()
        .unwrap();
    let recipe = sequence(sequence(JniOutput::Leaf(JniOutputLeafOp::Value))).bulk();
    let codec = JniCodec::raw();
    let mut plan = prebindgen_declarations::Compiler::new(&model, &codec, "ret".into())
        .output_root(
            &model.function("rows").unwrap().ret.ty,
            &recipe,
            &(),
            prebindgen_conversions::source_result_expr(),
            false,
            true,
        )
        .unwrap();
    plan.outcomes.binding_failure = codec.binding_failure_layout(&()).unwrap();
    codec.finish_output(&mut plan, &()).unwrap();
    (model, plan)
}
fn prefer_jvm_rows(node: &mut JniNode) {
    match node {
        WireNode::Chain { meta, child, .. } => {
            if let JniChain::Sequence(delivery) = meta {
                *delivery = JniSequence::TypedBuffered;
            }
            prefer_jvm_rows(child);
        }
        WireNode::Branch { children, .. } => children.iter_mut().for_each(prefer_jvm_rows),
        WireNode::Leaf(_) => {}
    }
}

#[test]
fn one_bulk_scope_controls_nested_rows_and_overrides_ordinary_jvm_preferences() {
    let (mut model, mut plan) = prepared();
    assert_eq!(
        prebindgen_conversions::wire_definitions(std::iter::once(&plan.outcomes.success))
            .filter(|wire| matches!(wire.kind, JniWire::NativeRows { .. }))
            .count(),
        2
    );
    bindings::set_output(&mut model, "rows", plan.clone().into()).unwrap();
    let original = generate(&model).unwrap();
    let shape = group::error_wire_shape(&plan.outcomes.success);
    // Changing an ordinary JVM preference must not add JVM row classes or alter
    // decoding, cleanup or native code inside the inherited bulk scope.
    prefer_jvm_rows(&mut plan.outcomes.success);
    assert_eq!(shape, group::error_wire_shape(&plan.outcomes.success));
    bindings::set_output(&mut model, "rows", plan.into()).unwrap();
    let changed = generate(&model).unwrap();
    assert_eq!(original.rust, changed.rust);
    assert_eq!(original.kotlin, changed.kotlin);
}

#[test]
fn complete_boundaries_reject_native_rows_without_a_scope_and_object_rows_inside_it() {
    let (_, plan) = prepared();
    let mut root = plan.outcomes.success;
    let WireNode::Chain {
        meta: JniChain::Bulk(_),
        child,
        ..
    } = &mut root
    else {
        panic!()
    };
    // A partial conversion can be validated before the compiler attaches Bulk.
    group::validate_fragment(child).unwrap();
    assert!(group::validate_layout(child)
        .unwrap_err()
        .contains("incompatible"));
    let WireNode::Chain { controls, .. } = child.as_mut() else {
        panic!()
    };
    controls[0].kind = JniWire::Rows {
        condition: "true".into(),
        null_niche_available: true,
    };
    assert!(group::validate_layout(&root)
        .unwrap_err()
        .contains("incompatible"));
}

#[test]
fn bulk_scope_does_not_leak_to_the_next_function_parameter() {
    let source = syn::parse_file("pub fn mixed(packed:Vec<i64>,ordinary:Vec<i64>){}").unwrap();
    let model = JniGen::builder()
        .items(
            source
                .items
                .into_iter()
                .map(|item| (item, Default::default())),
        )
        .set_package_prefix("test")
        .package(package!().fun(fun!(mixed).expand_param("packed", JniInputConfig::auto().bulk())))
        .prepare()
        .unwrap();
    let call = model.function("mixed").unwrap().metadata[0].call().unwrap();
    let packed = group::argument_parameters(call.inputs[0].input_node()).collect::<Vec<_>>();
    let ordinary = group::argument_parameters(call.inputs[1].input_node()).collect::<Vec<_>>();
    assert!(matches!(packed[0].kind, JniWire::BulkBuffer { .. }));
    assert!(matches!(ordinary.as_slice(), [wire] if matches!(wire.kind, JniWire::Rows { .. })));
    generate(&model).unwrap();
}
