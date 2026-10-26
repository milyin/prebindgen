use super::*;

const SOURCE: &str = "
pub type Value=String;
pub type Handler=String;
pub fn number(value:&Value)->i64{}
pub fn positive(value:&Value)->bool{}
pub fn from_number(number:i64)->Value{}
pub fn from_flag(flag:bool)->Value{}
pub fn first(cb:impl Fn(Value,i64)->Value+Send+Sync+'static)->i64{}
pub fn second(cb:impl Fn(Value,i64)->Value)->i64{}
pub fn make(cb:impl Fn(Value,i64)->Value)->Handler{}
pub fn nested(handlers:Vec<Handler>)->i64{}
pub fn ordinary(value:Value)->i64{}
pub fn unselected(value:*const i64){}
";
fn builder() -> JniGenBuilder {
    let source = syn::parse_file(SOURCE).unwrap();
    JniGen::builder()
        .items(
            source
                .items
                .into_iter()
                .map(|item| (item, Default::default())),
        )
        .set_package_prefix("example")
}
fn declaration() -> CallbackDecl {
    callback!(impl Fn(Value, i64) -> Value)
        .name("Handler")
        .expand_param(0, expand_return!(Value).field(fun!(number)))
        .expand_return(expand_param!(Value).variant(fun!(from_flag)))
}
fn error(builder: JniGenBuilder) -> String {
    builder.build().err().expect("expected invalid declaration")
}

#[test]
fn signature_defaults_and_local_overrides_are_independent() {
    let output = builder()
        .expand(expand_param!(Value).variant(fun!(from_number)))
        .package(package!("handlers").class(declaration()))
        .package(
            package!()
                .fun(fun!(first))
                .fun(
                    fun!(second).expand_param(
                        "cb",
                        callback!(impl Fn(Value, i64) -> Value)
                            .name("Local")
                            .expand_param(0, expand_return!(Value).field(fun!(positive))),
                    ),
                )
                .fun(fun!(ordinary)),
        )
        .build()
        .unwrap();
    let default = &output.kotlin["example/handlers/Handler.kt"];
    let local = &output.kotlin["example/handlers/Local.kt"];
    assert!(
        default.contains("Long, `arg1_0`: Long): Boolean"),
        "{default}"
    );
    assert!(
        local.contains("Boolean, `arg1_0`: Long): Boolean"),
        "{local}"
    );
    assert!(output.kotlin["example/Functions.kt"].contains("example.handlers.Local?"));
    assert!(!output.rust.contains("fn unselected"));
    assert!(output.rust.contains("crate::positive"));
    assert!(output.rust.contains("crate::from_flag"));
    syn::parse_file(&output.rust).unwrap();
}

#[test]
fn local_result_and_nested_constructor_overrides_inherit_arguments() {
    let output = builder()
        .package(package!("handlers").class(declaration()))
        .package(
            package!()
                .class(ptr_class!(Handler).name("NativeHandler"))
                .fun(
                    fun!(first).expand_param(
                        "cb",
                        callback!(impl Fn(Value, i64) -> Value)
                            .name("NumberResult")
                            .expand_return(expand_param!(Value).variant(fun!(from_number))),
                    ),
                )
                .fun(
                    fun!(nested).expand_param(
                        "handlers",
                        Input::for_type(
                            syn::parse_quote!(Handler),
                            Input::constructor(
                                "make",
                                [callback!(impl Fn(Value, i64) -> Value)
                                    .name("Nested")
                                    .expand_param(0, expand_return!(Value).field(fun!(positive)))
                                    .into()],
                            ),
                        ),
                    ),
                ),
        )
        .build()
        .unwrap();
    let number = &output.kotlin["example/handlers/NumberResult.kt"];
    assert!(number.contains("Long, `arg1_0`: Long): Long"), "{number}");
    let nested = &output.kotlin["example/handlers/Nested.kt"];
    assert!(
        nested.contains("Boolean, `arg1_0`: Long): Boolean"),
        "{nested}"
    );
    assert!(output.kotlin["example/Functions.kt"].contains("List<example.handlers.Nested?"));
    assert!(output.rust.contains("crate::make"));
}

#[test]
fn callback_only_declarations_select_helpers_and_emit_interfaces() {
    let output = builder()
        .package(package!("handlers").class(declaration()))
        .build()
        .unwrap();
    assert!(output.kotlin.contains_key("example/handlers/Handler.kt"));
    assert!(!output.kotlin.keys().any(|name| name.ends_with("Value.kt")));
}

#[test]
fn canonical_signatures_ignore_marker_bounds_and_omitted_unit_returns() {
    let error = error(
        builder().package(package!().class(declaration()).class(callback!(
            impl Fn(Value, i64) -> Value + Send + Sync + 'static
        ))),
    );
    assert!(error.contains("duplicate input expansion"), "{error}");
    let error = error_from_unit_defaults();
    assert!(error.contains("duplicate input expansion"), "{error}");
}
fn error_from_unit_defaults() -> String {
    error(
        builder().package(
            package!()
                .class(callback!(impl Fn(i64)))
                .class(callback!(impl Fn(i64) -> ())),
        ),
    )
}

#[test]
fn callback_overrides_validate_positions_types_and_foreign_names() {
    for (local, expected) in [
        (
            callback!(impl Fn(Value, i64) -> Value).expand_param(2, Output::Leaf),
            "out of range",
        ),
        (
            callback!(impl Fn(Value, i64) -> Value)
                .expand_param(0, expand_return!(bool).field_self()),
            "does not match",
        ),
        (
            callback!(impl Fn(Value, i64) -> Value)
                .expand_return(expand_param!(bool).variant_self()),
            "does not match",
        ),
        (callback!(impl Fn(bool)), "does not match"),
    ] {
        let result = error(
            builder()
                .package(package!("handlers").class(declaration()))
                .package(package!().fun(fun!(first).expand_param("cb", local))),
        );
        assert!(result.contains(expected), "{result}");
    }
    for name in ["JNINative", "invalid-name"] {
        let result = error(builder().package(package!().class(declaration().name(name))));
        assert!(result.contains("name"), "{result}");
    }
}

#[test]
fn incompatible_local_shapes_require_distinct_interface_names() {
    let result = error(
        builder()
            .package(package!("handlers").class(declaration()))
            .package(
                package!().fun(
                    fun!(first).expand_param(
                        "cb",
                        callback!(impl Fn(Value, i64) -> Value)
                            .expand_param(0, expand_return!(Value).field(fun!(positive))),
                    ),
                ),
            ),
    );
    assert!(
        result.contains("callback interface signature collision"),
        "{result}"
    );
    assert!(result.contains("name(...)"), "{result}");
}

#[test]
fn equal_kotlin_types_with_different_handle_ownership_require_distinct_names() {
    let source = syn::parse_file("pub type Value=String; pub fn f(cb:impl Fn(Value)){}").unwrap();
    let result = error(
        JniGen::builder()
            .items(
                source
                    .items
                    .into_iter()
                    .map(|item| (item, Default::default())),
            )
            .set_package_prefix("example")
            .package(
                package!()
                    .class(ptr_class!(Value))
                    .class(callback!(impl Fn(Value)).name("Handler"))
                    .fun(
                        fun!(f).expand_param(
                            "cb",
                            callback!(impl Fn(Value))
                                .expand_param(0, expand_return!(Value).field_self_into()),
                        ),
                    ),
            ),
    );
    assert!(
        result.contains("callback interface ownership collision"),
        "{result}"
    );
}

#[test]
fn recursive_callback_construction_is_a_preparation_error() {
    let source =
        syn::parse_file("pub type Value=String; pub fn make(cb:impl Fn()->Value)->Value{}")
            .unwrap();
    let result = error(
        JniGen::builder()
            .items(
                source
                    .items
                    .into_iter()
                    .map(|item| (item, Default::default())),
            )
            .set_package_prefix("example")
            .package(
                package!().class(
                    callback!(impl Fn() -> Value)
                        .expand_return(expand_param!(Value).variant(fun!(make))),
                ),
            ),
    );
    assert!(result.contains("recursive"), "{result}");
}

#[test]
fn derived_names_and_local_only_overrides_work_with_fold_variants() {
    let source = syn::parse_file("pub fn values(cb:impl Fn(i64)->i64)->Vec<i64>{}").unwrap();
    let output = JniGen::builder()
        .items(
            source
                .items
                .into_iter()
                .map(|item| (item, Default::default())),
        )
        .set_package_prefix("example")
        .package(package!("handlers").class(callback!(impl Fn(i64) -> i64)))
        .package(
            package!().fun(
                fun!(values).callback_return().expand_param(
                    "cb",
                    callback!(impl Fn(i64) -> i64)
                        .expand_param(0, expand_return!(i64).field_self())
                        .expand_return(expand_param!(i64).variant_self()),
                ),
            ),
        )
        .build()
        .unwrap();
    assert!(output
        .kotlin
        .contains_key("example/handlers/i64Toi64Callback.kt"));
    assert!(output.kotlin["example/Functions.kt"].contains("valuesFold"));
    assert_eq!(
        output.kotlin["example/Functions.kt"]
            .matches("example.handlers.i64Toi64Callback?")
            .count(),
        2
    );

    let source = syn::parse_file("pub fn f(cb:impl Fn(i64)->i64){}").unwrap();
    let output = JniGen::builder()
        .items(
            source
                .items
                .into_iter()
                .map(|item| (item, Default::default())),
        )
        .set_package_prefix("example")
        .package(package!().fun(fun!(f).expand_param("cb", callback!(impl Fn(i64) -> i64))))
        .build()
        .unwrap();
    assert!(output.kotlin.contains_key("example/i64Toi64Callback.kt"));
}

#[test]
fn callback_interfaces_cannot_shadow_handle_classes() {
    let result = error(
        builder().package(
            package!()
                .class(ptr_class!(Value).name("Handler"))
                .class(declaration()),
        ),
    );
    assert!(
        result.contains("duplicate foreign callback interface name"),
        "{result}"
    );
}

#[test]
fn c_preserves_automatic_callbacks_and_rejects_callback_specific_settings() {
    for configured in [false, true] {
        let source = syn::parse_file("pub fn f(cb:impl Fn(i64)->i64)->i64{}").unwrap();
        let flat = prebindgen_c::Bindings::parse(
            source
                .items
                .into_iter()
                .map(|item| (item, Default::default())),
        )
        .unwrap();
        let declaration = if configured {
            fun!(f).expand_param(
                "cb",
                callback!(impl Fn(i64) -> i64).expand_return(expand_param!(i64).variant_self()),
            )
        } else {
            fun!(f)
        };
        let result = prebindgen_c::Cbindgen::new(flat)
            .function(declaration)
            .panic()
            .build();
        if configured {
            assert!(result
                .err()
                .unwrap()
                .contains("configured callbacks are unsupported"));
        } else {
            let generated = result.unwrap();
            assert!(generated.header.contains("(*run)"));
            syn::parse_file(&generated.rust).unwrap();
        }
    }
}

#[test]
fn unit_callback_return_conversions_cannot_be_silently_ignored() {
    let source = syn::parse_file("pub fn make(value:i64){} pub fn f(cb:impl Fn()){}").unwrap();
    let result = error(
        JniGen::builder()
            .items(
                source
                    .items
                    .into_iter()
                    .map(|item| (item, Default::default())),
            )
            .set_package_prefix("example")
            .package(package!().fun(fun!(f).expand_param(
                "cb",
                callback!(impl Fn()).expand_return(expand_param!(()).variant(fun!(make))),
            ))),
    );
    assert!(result.contains("unit callback returns require"), "{result}");
}
