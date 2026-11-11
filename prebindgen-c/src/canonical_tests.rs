use super::*;
use prebindgen_declarations::{convert, fun};
fn model(source: &str) -> Bindings {
    Flat::parse(
        syn::parse_file(source)
            .unwrap()
            .items
            .into_iter()
            .map(|i| (i, Default::default())),
    )
    .unwrap()
}
#[test]
fn converted_borrowed_views_cannot_escape_a_temporary_owner() {
    for representation in ["&str", "Option<&str>"] {
        let source=format!("pub struct Owner{{pub text:String}}pub struct Child{{pub n:i64}}pub fn view(v:&Owner)->{representation}{{}}pub fn make()->Owner{{}}pub fn borrow()->&'static Owner{{}}");
        let base = || {
            Cbindgen::new(model(&source))
                .opaque_ptr("child", syn::parse_quote!(Child))
                .convert(convert!(Owner).output(fun!(view)))
        };
        let error = base().function(fun!(make)).panic().prepare().err().unwrap();
        assert!(
            error.contains("temporary owner"),
            "{representation}: {error}"
        );
        base().function(fun!(borrow)).panic().prepare().unwrap();
    }
}

#[test]
fn reference_elements_require_explicit_supported_views() {
    let source="pub struct Owner{pub n:i64}pub struct Child{pub n:i64}pub fn view(v:&Owner)->Vec<&Child>{}pub fn make()->Owner{}";
    let error = Cbindgen::new(model(source))
        .opaque_ptr("child", syn::parse_quote!(Child))
        .convert(convert!(Owner).output(fun!(view)))
        .function(fun!(make))
        .panic()
        .prepare()
        .err()
        .unwrap();
    assert!(
        error.contains("requires an explicit owning copy"),
        "{error}"
    );
}
