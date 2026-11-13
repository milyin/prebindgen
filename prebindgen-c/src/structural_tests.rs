//! Structural preparation trees expose every explicit recipe to the common walkers.
use crate::*;
use prebindgen_declarations::{input_children, output_children, Operation};

#[test]
fn nested_callback_fields_and_result_are_visible_without_request_specific_recursion() {
    let tree: CInput<CInputRequest, COutputRequest> = callback!(impl Fn(Packet) -> i64)
        .expand_param(
            0,
            expand_return!(Packet).fields(
                fields!(packet_view).field("value", expand_return!(i64).field(fun!(observe))),
            ),
        )
        .expand_return(expand_param!(i64).variant(fun!(construct)))
        .into();
    let names = crate::declarations::source_items(
        std::slice::from_ref(&tree),
        &[],
        &[],
        std::iter::empty(),
    );
    assert_eq!(names, ["packet_view", "observe", "construct"]);
    let mut calls = vec![];
    let mapped = tree.map_requests(
        &mut |request| match request {
            CInputRequest::Call(name) => {
                calls.push(name);
                CInputRequest::Call("replacement".into())
            }
            other => other,
        },
        &mut |request| request,
    );
    assert_eq!(calls, ["construct"]);
    let names = crate::declarations::source_items(&[mapped], &[], &[], std::iter::empty());
    assert_eq!(names, ["packet_view", "observe", "replacement"]);
}
#[test]
fn partial_enum_branches_expose_their_field_recipes_to_generic_traversal() {
    let tree = CInput::enum_fields([("First", vec![("value", CInput::call("make"))])]);
    fn input(tree: &CInput<CInputRequest, COutputRequest>, requests: &mut Vec<String>) {
        if let Some(request) = tree.pending_request() {
            requests.push(
                match request {
                    CInputRequest::EnumFields => "enum",
                    CInputRequest::VariantFields { .. } => "variant",
                    CInputRequest::Call(_) => "constructor",
                    _ => "other",
                }
                .into(),
            );
        }
        input_children(tree, requests, &mut input, &mut output);
    }
    fn output(tree: &COutput<CInputRequest, COutputRequest>, requests: &mut Vec<String>) {
        output_children(tree, requests, &mut input, &mut output);
    }
    let mut requests = vec![];
    input(&tree, &mut requests);
    assert_eq!(requests, ["enum", "variant", "constructor"]);
}
fn flat(source: &str) -> CBindings {
    prebindgen_flat::Flat::parse(
        syn::parse_file(source)
            .unwrap()
            .items
            .into_iter()
            .map(|item| (item, Default::default())),
    )
    .unwrap()
}
#[test]
fn mismatched_names_and_children_are_rejected_before_conversion() {
    let flat = flat("pub struct Row {pub value:i64}pub fn rows()->Row{}");
    let defaults = DeclCDefaults::new(&flat, vec![], vec![], vec![]).unwrap();
    let tree = COutput::Branch {
        operation: Operation::Request(COutputRequest::NamedFields {
            fields: vec!["value".into()],
            mode: prebindgen_declarations::ProjectionMode::Owned,
        }),
        children: vec![],
    };
    let error = defaults
        .resolve_output(
            &flat,
            &CCodec::declared(),
            &flat.function("rows").unwrap().ret.ty,
            &tree,
            &(),
        )
        .unwrap_err();
    assert!(error.contains("field name and child counts"), "{error}");
    let malformed = CInput::Unary {
        operation: Operation::Request(CInputRequest::EnumFields),
        child: Box::new(CInput::auto()),
    };
    let error = defaults
        .resolve_input(
            &flat,
            &CCodec::declared(),
            &syn::parse_quote!(i64),
            &malformed,
            &(),
        )
        .unwrap_err();
    assert!(error.contains("structural node shape"), "{error}");
}
#[test]
fn callback_argument_indices_must_correspond_to_domain_children() {
    let flat = flat("pub fn register(cb:impl Fn(i64)->i64){}");
    let defaults = DeclCDefaults::new(&flat, vec![], vec![], vec![]).unwrap();
    let mut tree: CInput<CInputRequest, COutputRequest> = callback!(impl Fn(i64) -> i64)
        .expand_param(0, expand_return!(i64).field_self())
        .into();
    let CInput::Unary { child, .. } = &mut tree else {
        panic!("expected typed callback")
    };
    let CInput::Arrow {
        operation: Operation::Request(CInputRequest::Callback(request)),
        ..
    } = child.as_mut()
    else {
        panic!("expected callback arrow")
    };
    request.arguments.clear();
    let error = defaults
        .resolve_input(
            &flat,
            &CCodec::declared(),
            &flat.function("register").unwrap().params[0].ty.ty,
            &tree,
            &(),
        )
        .unwrap_err();
    assert!(
        error.contains("callback argument index and child counts"),
        "{error}"
    );
}

#[test]
fn local_conversion_combinators_preserve_type_scope_and_container_defaults() {
    let flat = flat("pub struct Item{pub value:i64} pub fn make(value:i64)->Item{} pub fn read(item:&Item)->i64{}");
    let conversion = || {
        prebindgen_declarations::convert!(Item)
            .input(fun!(make))
            .output(fun!(read))
    };
    let input: CInput<CInputRequest, COutputRequest> = conversion().into();
    let output: COutput<CInputRequest, COutputRequest> = conversion().into();
    let defaults = DeclCDefaults::new(
        &flat,
        vec![input.clone().variant("make")],
        vec![output.field("read")],
        vec![],
    )
    .unwrap();
    // Both selector arms must resolve at Item, rather than at Option<Item>.
    let input = defaults
        .resolve_input(
            &flat,
            &CCodec::declared(),
            &flat.resolve_type(&syn::parse_quote!(Option<Item>)).unwrap(),
            &CInput::auto(),
            &(),
        )
        .unwrap();
    assert!(matches!(
        input,
        CInput::Unary {
            operation: Operation::Resolved(CInputUnary::Optional),
            ..
        }
    ));
    // The appended getter observes Item even for a borrowed collection element.
    let output = defaults
        .resolve_output(
            &flat,
            &CCodec::declared(),
            &flat
                .resolve_type(&syn::parse_quote!(Vec<Option<&Item>>))
                .unwrap(),
            &COutput::auto(),
            &(),
        )
        .unwrap();
    assert!(matches!(
        output,
        COutput::Unary {
            operation: Operation::Resolved(COutputUnary::Sequence { .. }),
            ..
        }
    ));
    let borrowed: CInput<CInputRequest, COutputRequest> = prebindgen_declarations::convert!(&Item)
        .input(fun!(make))
        .into();
    let reconstructed = defaults
        .resolve_input(
            &flat,
            &CCodec::declared(),
            &flat.resolve_type(&syn::parse_quote!(&Item)).unwrap(),
            &borrowed.reconstruct(),
            &(),
        )
        .unwrap();
    assert!(matches!(
        reconstructed,
        CInput::Unary {
            operation: Operation::Resolved(CInputUnary::Temporary),
            ..
        }
    ));
}

#[test]
fn reference_type_local_conversions_keep_their_declared_scope() {
    let flat = flat("pub struct Item{pub value:i64}");
    let defaults = DeclCDefaults::new(&flat, vec![], vec![], vec![]).unwrap();
    let output = prebindgen_declarations::convert!(&Item)
        .output(prebindgen_declarations::into!(i64))
        .into();
    let output = defaults
        .resolve_output(
            &flat,
            &CCodec::declared(),
            &flat.resolve_type(&syn::parse_quote!(&Item)).unwrap(),
            &output,
            &(),
        )
        .unwrap();
    assert!(matches!(
        output,
        COutput::Unary {
            operation: Operation::Resolved(COutputUnary::Convert { .. }),
            ..
        }
    ));
}

#[test]
fn implicit_callback_result_cannot_hide_an_explicit_child() {
    let flat = flat("pub fn register(cb:impl Fn()->i64){}");
    let defaults = DeclCDefaults::new(&flat, vec![], vec![], vec![]).unwrap();
    let mut tree: CInput<CInputRequest, COutputRequest> = callback!(impl Fn() -> i64).into();
    let CInput::Unary { child, .. } = &mut tree else {
        panic!("expected typed callback")
    };
    let CInput::Arrow { codomain, .. } = child.as_mut() else {
        panic!("expected callback arrow")
    };
    **codomain = CInput::value();
    let error = defaults
        .resolve_input(
            &flat,
            &CCodec::declared(),
            &flat.function("register").unwrap().params[0].ty.ty,
            &tree,
            &(),
        )
        .unwrap_err();
    assert!(error.contains("Auto codomain"), "{error}");
}
