//! Structural preparation trees expose every explicit recipe to the common walkers.
use crate::*;
use prebindgen_declarations::{input_children, output_children, InputNode, OutputNode};

#[test]
fn nested_callback_fields_and_result_are_visible_without_request_specific_recursion() {
    let tree: CInputConfig = callback!(impl Fn(Packet) -> i64)
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
}
#[test]
fn partial_enum_branches_expose_their_field_recipes_to_generic_traversal() {
    let tree = CInputConfig::enum_fields([("First", vec![("value", CInputConfig::call("make"))])]);
    fn input(tree: &CInputConfig, requests: &mut Vec<String>) {
        requests.push(
            match tree {
                InputNode::Branch {
                    operation: CInputBranch::ConfigEnumFields(_),
                    ..
                } => "enum",
                InputNode::Branch {
                    operation: CInputBranch::ConfigVariantFields(CVariantFields { .. }),
                    ..
                } => "variant",
                CInputConfig::Leaf(CInputLeafOp::ConfigConstructor(_)) => "constructor",
                _ => "other",
            }
            .into(),
        );
        input_children(tree, requests, &mut input, &mut output);
    }
    fn output(tree: &COutputConfig, requests: &mut Vec<String>) {
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
    let tree = OutputNode::Branch {
        operation: COutputBranch::ConfigNamedFields(CNamedFields {
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
}
#[test]
fn callback_argument_indices_must_correspond_to_domain_children() {
    let flat = flat("pub fn register(cb:impl Fn(i64)->i64){}");
    let defaults = DeclCDefaults::new(&flat, vec![], vec![], vec![]).unwrap();
    let mut tree: CInputConfig = callback!(impl Fn(i64) -> i64)
        .expand_param(0, expand_return!(i64).field_self())
        .into();
    let InputNode::Unary { child, .. } = &mut tree else {
        panic!("expected typed callback")
    };
    let InputNode::Arrow {
        operation: CInputArrow::ConfigCallback(request),
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
    let input: CInputConfig = conversion().into();
    let output: COutputConfig = conversion().into();
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
            &CInputConfig::auto(),
            &(),
        )
        .unwrap();
    assert!(matches!(
        input,
        InputNode::Unary {
            operation: CInputUnary::Optional,
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
            &COutputConfig::auto(),
            &(),
        )
        .unwrap();
    assert!(matches!(
        output,
        OutputNode::Unary {
            operation: COutputUnary::Sequence { .. },
            ..
        }
    ));
    let borrowed: CInputConfig = prebindgen_declarations::convert!(&Item)
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
        InputNode::Unary {
            operation: CInputUnary::Temporary,
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
        OutputNode::Unary {
            operation: COutputUnary::Convert { .. },
            ..
        }
    ));
}

#[test]
fn implicit_callback_result_cannot_hide_an_explicit_child() {
    let flat = flat("pub fn register(cb:impl Fn()->i64){}");
    let defaults = DeclCDefaults::new(&flat, vec![], vec![], vec![]).unwrap();
    let mut tree: CInputConfig = callback!(impl Fn() -> i64).into();
    let InputNode::Unary { child, .. } = &mut tree else {
        panic!("expected typed callback")
    };
    let InputNode::Arrow { codomain, .. } = child.as_mut() else {
        panic!("expected callback arrow")
    };
    **codomain = CInputConfig::value();
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

#[test]
fn c_source_field_declarations_and_local_helper_overrides_share_the_resolver() {
    let items = syn::parse_file(
        "pub struct Pair{pub first:i64,pub second:i64}
        pub fn number(value:bool)->i64{0}
        pub fn pair_new(first:i64,second:i64)->Pair{Pair{first,second}}
        pub fn echo(value:Pair)->Pair{value}
        pub fn direct(value:Pair)->Pair{value}",
    )
    .unwrap()
    .items
    .into_iter()
    .map(|item| (item, Default::default()));
    let generated = Cbindgen::new(Flat::parse(items).unwrap())
        .function(
            fun!(echo)
                .expand_param(
                    "value",
                    expand_param!(Pair).variant(
                        fun!(pair_new)
                            .expand_param("first", expand_param!(i64).variant(fun!(number))),
                    ),
                )
                .expand_return(expand_return!(Pair).fields_into(fields!())),
        )
        .function(
            fun!(direct)
                .expand_param(
                    "value",
                    expand_param!(Pair).variant_fields(
                        param_fields!().field("first", expand_param!(i64).variant(fun!(number))),
                    ),
                )
                .expand_return(expand_return!(Pair).fields_into(fields!())),
        )
        .panic()
        .build()
        .unwrap();
    syn::parse_file(&generated.rust).unwrap();
    assert!(generated.rust.contains("pair_new"));
    assert!(generated.rust.contains("number"));
    assert!(generated.header.contains("pb_echo"));
    assert!(generated.header.contains("pb_direct"));
}
