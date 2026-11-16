//! The public preparation stages retain declarations on their original Flat owners.
use super::*;

fn builder(source: &str) -> JniGenBuilder {
    JniGen::builder()
        .items(
            syn::parse_file(source)
                .unwrap()
                .items
                .into_iter()
                .map(|item| (item, Default::default())),
        )
        .set_package_prefix("example")
}

#[test]
fn class_only_models_resolve_before_compilation_or_emission() {
    for (source, declaration, name) in [
        (
            "pub struct Row { pub number:i64 }",
            DeclJniClass::from(data_class!(Row)),
            "Row",
        ),
        (
            "pub enum Code { One=1, Two=2 }",
            enum_class!(Code).into(),
            "Code",
        ),
        (
            "pub enum Event { Empty, Data{number:i64} }",
            sealed_class!(Event).into(),
            "Event",
        ),
    ] {
        let configured = builder(source)
            .package(package!().class(declaration))
            .configured()
            .unwrap();
        assert!(configured.functions.is_empty());
        let ty = configured
            .resolve_type(&syn::parse_str(name).unwrap())
            .unwrap();
        let codec = JniCodec::declared();
        let resolver = JniSourceResolver::default();
        let compiler = prebindgen_declarations::Compiler::new(&configured, &codec, "value".into());
        for error in [
            compile(configured.clone()).err().unwrap(),
            generate(&configured).err().unwrap(),
            resolver
                .resolve_input(&configured, &codec, &ty, &JniInputConfig::auto(), &())
                .err()
                .unwrap(),
            resolver
                .resolve_output(&configured, &codec, &ty, &JniOutputConfig::auto(), &())
                .err()
                .unwrap(),
            compiler
                .input(
                    &ty,
                    &JniInput::Leaf(JniInputLeafOp::Value),
                    &(),
                    &Condition::default(),
                )
                .err()
                .unwrap(),
            compiler
                .output_root(
                    &ty,
                    &JniOutput::Leaf(JniOutputLeafOp::Value),
                    &(),
                    quote::quote!(value),
                    false,
                    false,
                )
                .err()
                .unwrap(),
        ] {
            assert!(
                error.contains(name) && error.contains("unresolved"),
                "{error}"
            );
        }
        // Classes require semantic resolution, but have no Rust converter to compile.
        let resolved = resolve(configured).unwrap();
        let input = resolver
            .resolve_input(&resolved, &codec, &ty, &JniInputConfig::auto(), &())
            .unwrap();
        let output = resolver
            .resolve_output(&resolved, &codec, &ty, &JniOutputConfig::auto(), &())
            .unwrap();
        prebindgen_declarations::compile_input_value(&resolved, &codec, &ty, &input, &()).unwrap();
        prebindgen_declarations::compile_output_value(&resolved, &codec, &ty, &output, &())
            .unwrap();
        let from_resolved = generate(&resolved).unwrap();
        let prepared = compile(resolved).unwrap();
        let from_prepared = generate(&prepared).unwrap();
        assert!(from_prepared
            .kotlin
            .contains_key(&format!("example/{name}.kt")));
        assert_eq!(from_resolved.rust, from_prepared.rust);
        assert_eq!(from_resolved.kotlin, from_prepared.kotlin);
    }
}

#[test]
fn standalone_callback_default_must_resolve_to_a_callback() {
    let mut configured = builder("pub fn accept(cb:impl Fn(i64)){}")
        .package(package!().class(callback!(impl Fn(i64)).name("Handler")))
        .configured()
        .unwrap();
    let ty = prebindgen_declarations::callback::input_type(
        &configured,
        &syn::parse_quote!(impl Fn(i64)),
    )
    .unwrap();
    configured.type_metadata_mut(&ty).unwrap().defaults.input = Some(JniInputConfig::selector([
        callback!(impl Fn(i64)).into(),
        callback!(impl Fn(i64)).into(),
    ]));
    let error = resolve(configured).err().unwrap();
    assert!(
        error.contains("callback declaration requires a callback conversion"),
        "{error}"
    );
}

#[test]
fn resolution_reads_source_fields_after_configuration() {
    let mut configured = builder("pub struct Row { pub number:i64 }")
        .package(package!().class(data_class!(Row)))
        .configured()
        .unwrap();
    let ty = configured.resolve_type(&syn::parse_quote!(Row)).unwrap();
    let key = type_key(&ty);
    let field = &mut configured.record_by_key_mut(&key).unwrap().fields[0];
    assert!(field.metadata.kotlin_type.is_empty());
    field.ty = syn::parse_quote!(String);
    let resolved = resolve(configured).unwrap();
    assert_eq!(
        resolved.record_by_key(&key).unwrap().fields[0]
            .metadata
            .kotlin_type,
        "kotlin.String"
    );
    assert!(generate(&resolved).unwrap().kotlin["example/Row.kt"].contains("number: kotlin.String"));
}

#[test]
fn class_representation_errors_belong_to_resolution() {
    for (source, declaration, expected) in [
        (
            "pub struct Row { pub number:Option<Option<i64>> }",
            DeclJniClass::from(data_class!(Row)),
            "nested Option",
        ),
        (
            "pub enum Code { Value=2147483648 }",
            enum_class!(Code).into(),
            "does not fit",
        ),
        (
            "pub enum Event { Data { number:Option<Option<i64>> } }",
            sealed_class!(Event).into(),
            "nested Option",
        ),
    ] {
        let configured = builder(source)
            .package(package!().class(declaration))
            .configured()
            .unwrap();
        let error = resolve(configured).err().unwrap();
        assert!(error.contains(expected), "{error}");
    }
}

#[test]
fn nested_class_ownership_is_resolved_before_function_recipes() {
    let configured = builder("pub type Handle=String;pub struct Child{pub handle:Handle}pub struct Parent{pub child:Child}pub fn send(value:Parent){}")
        .package(package!()
            .class(data_class!(Parent))
            .class(data_class!(Child))
            .class(ptr_class!(Handle))
            .fun(fun!(send)))
        .configured()
        .unwrap();
    let error = resolve(configured).err().unwrap();
    assert!(
        error.contains("Parent") && error.contains("contains handles"),
        "{error}"
    );
}

#[test]
fn standalone_callback_recipes_wait_for_class_resolution() {
    let configured = builder("pub enum Code{One,Two}pub enum Event{Empty,Data{number:i64}}pub struct Row{pub code:Code,pub event:Event}pub fn accept(cb:impl Fn(Row)->Event){}")
        .package(package!()
            .class(callback!(impl Fn(Row)->Event).name("Handler"))
            .class(data_class!(Row))
            .class(enum_class!(Code))
            .class(sealed_class!(Event)))
        .configured()
        .unwrap();
    let callback = configured
        .resolve_parameter_type(&syn::parse_quote!(impl Fn(Row) -> Event))
        .unwrap();
    assert!(matches!(
        configured.type_info(&callback).unwrap().metadata.callback,
        Some(prebindgen_conversions::BindingState::Configured(()))
    ));
    let resolved = resolve(configured).unwrap();
    let prepared = compile(resolved).unwrap();
    let generated = generate(&prepared).unwrap();
    assert!(generated.kotlin["example/Handler.kt"].contains("example.Row"));
    assert!(generated.kotlin["example/Handler.kt"].contains("example.Event"));
    // Matching an unselected source occurrence must not add a foreign export.
    assert!(prepared.functions.is_empty());
}

#[test]
fn repeated_resolution_preserves_enum_guards_and_existing_calls() {
    let prepared =
        builder("pub enum Code{One,Two}pub struct Row{pub code:Code}pub fn echo(value:Row)->Row{}")
            .package(
                package!()
                    .class(data_class!(Row))
                    .class(enum_class!(Code).variant_name("One", "First"))
                    .fun(fun!(echo)),
            )
            .prepare()
            .unwrap();
    let before = generate(&prepared).unwrap();
    let guard_count = prepared.guards.len();
    let prepared = prepare(prepared).unwrap();
    let after = generate(&prepared).unwrap();
    assert_eq!(guard_count, prepared.guards.len());
    assert_eq!(before.rust, after.rust);
    assert_eq!(before.kotlin, after.kotlin);
}
