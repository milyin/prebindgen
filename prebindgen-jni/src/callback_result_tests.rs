use super::*;

fn builder(source: &str) -> JniGenBuilder {
    let source = syn::parse_file(source).unwrap();
    JniGen::builder()
        .items(
            source
                .items
                .into_iter()
                .map(|item| (item, Default::default())),
        )
        .set_package_prefix("example")
}

#[test]
fn result_arguments_use_success_representations_and_distinct_interface_names() {
    let output = builder("pub fn scalar(cb:impl Fn(Result<i64,String>)){} pub fn unit(cb:impl Fn(Result<(),String>)){} pub fn returning(cb:impl Fn(Result<i64,String>)->i64){}")
        .package(package!().fun(fun!(scalar)).fun(fun!(unit)).fun(fun!(returning)))
        .build().unwrap();
    assert!(output.kotlin["example/Resulti64Callback.kt"].contains("Long"));
    assert!(output.kotlin["example/ResultUnitCallback.kt"].contains("public fun run()"));
    assert!(output.kotlin["example/Resulti64Toi64Callback.kt"].contains("): Long"));
    assert!(!output.rust.contains("unused_domain"));
    syn::parse_file(&output.rust).unwrap();
}

#[test]
fn result_argument_success_overrides_and_error_mappers_compose() {
    let output = builder("pub type Value=String; pub type Error=String; pub fn code(value:&Value)->i64{} pub fn message(error:&Error)->String{} pub fn f(cb:impl Fn(Result<Value,Error>)){}")
        .error_mapper(syn::parse_quote!(Error), fun!(message))
        .package(package!()
            .class(callback!(impl Fn(Result<Value,Error>)).name("Handler")
                .expand_param(0, expand_return!(Value).field(fun!(code))))
            .fun(fun!(f)))
        .build().unwrap();
    assert!(output.kotlin["example/Handler.kt"].contains("Long"));
    let rust = output.rust.split_whitespace().collect::<String>();
    assert!(rust.contains("crate::message"));
    assert!(output.rust.contains("crate::code"));
}

#[test]
fn result_arguments_require_explicit_mapping_for_non_string_errors() {
    let result = builder("pub type Error=String; pub fn f(cb:impl Fn(Result<i64,Error>)){}")
        .package(package!().fun(fun!(f)))
        .build()
        .err()
        .unwrap();
    assert!(
        result.contains("callback Result error") && result.contains("requires an error_mapper"),
        "{result}"
    );
}

#[test]
fn nested_or_borrowed_result_arguments_are_rejected() {
    for (ty, expected) in [
        ("Option<Result<i64,String>>", "Result-valued callback"),
        ("&Result<i64,String>", "requires an owned Result occurrence"),
        (
            "Result<Vec<Result<i64,String>>,String>",
            "outermost return with an explicit mapper",
        ),
    ] {
        let result = builder(&format!("pub fn f(cb:impl Fn({ty})){{}}"))
            .package(package!().fun(fun!(f)))
            .build()
            .err()
            .unwrap();
        assert!(result.contains(expected), "{result}");
    }
}

#[test]
fn c_callbacks_accept_owned_result_arguments_without_a_registration_error_packet() {
    let source = syn::parse_file("pub fn f(cb:impl Fn(Result<i64,String>)){}").unwrap();
    let flat = prebindgen_c::Bindings::parse(
        source
            .items
            .into_iter()
            .map(|item| (item, Default::default())),
    )
    .unwrap();
    let output = prebindgen_c::Cbindgen::new(flat)
        .function(fun!(f))
        .panic()
        .build()
        .unwrap();
    assert!(output
        .header
        .contains("void (*run)(void *context, int64_t arg0);"));
    syn::parse_file(&output.rust).unwrap();
}
