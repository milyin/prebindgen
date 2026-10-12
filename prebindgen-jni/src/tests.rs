use super::*;
fn flat(source: &str) -> Flat<Slot> {
    Flat::parse(
        syn::parse_file(source)
            .unwrap()
            .items
            .into_iter()
            .map(|item| (item, Default::default())),
    )
    .unwrap()
}
fn fail(builder: JniGenBuilder) -> String {
    match builder.build() {
        Ok(_) => panic!("expected generation error"),
        Err(e) => e,
    }
}
#[test]
fn rejects_unknown_parameter_override() {
    let b = JniGen::builder()
        .flat(flat("pub type Key=String;pub fn f(k:Key)->Key{}"))
        .set_package_prefix("test")
        .package(
            package!()
                .class(ptr_class!(Key))
                .fun(fun!(f).expand_param("missing", expand_param!(Key).variant_self())),
        );
    assert!(fail(b).contains("unknown parameter"));
}
#[test]
fn rejects_recursive_expansion() {
    let b = JniGen::builder()
        .flat(flat("pub type Key=String;pub fn f(k:Key)->Key{}"))
        .set_package_prefix("test")
        .expand(expand_param!(Key).variant(fun!(f)))
        .package(package!().class(ptr_class!(Key)).fun(fun!(f)));
    assert!(fail(b).contains("recursive input"));
}
#[test]
fn rejects_missing_handle_class() {
    let b = JniGen::builder()
        .flat(flat("pub type Key=String;pub fn f(k:Key)->Key{}"))
        .set_package_prefix("test")
        .package(package!().fun(fun!(f)));
    assert!(fail(b).contains("no handle class"));
}
#[test]
fn rejects_jvm_signature_overflow() {
    let params = (0..128)
        .map(|i| format!("k{i}:Key"))
        .collect::<Vec<_>>()
        .join(",");
    let source = format!("pub type Key=String;pub fn f({params})->Key{{}}");
    let b = JniGen::builder()
        .flat(flat(&source))
        .set_package_prefix("test")
        .package(package!().class(ptr_class!(Key)).fun(fun!(f)));
    assert!(fail(b).contains("limit is 255"));
}

#[test]
fn scalar_jvm_slots_include_wide_numeric_carriers() {
    for ty in ["i64", "u32", "u64", "isize", "usize", "f64"] {
        for (count, accepted) in [(127, true), (128, false)] {
            let params = (0..count)
                .map(|i| format!("p{i}:{ty}"))
                .collect::<Vec<_>>()
                .join(",");
            let source = format!("pub fn f({params}){{}}");
            let result = JniGen::builder()
                .flat(flat(&source))
                .set_package_prefix("test")
                .package(package!().fun(fun!(f)))
                .build();
            assert_eq!(result.is_ok(), accepted, "{ty}: {count}");
        }
    }
}
