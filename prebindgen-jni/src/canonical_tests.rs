use super::*;
use prebindgen_declarations::{convert, from, into, try_from, try_into};

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
fn builder(source: &str) -> JniGenBuilder {
    JniGen::builder()
        .flat(model(source))
        .set_package_prefix("test")
}
const VALUE: &str = "pub struct Value{pub n:i64} pub fn make(n:i64)->Value{} pub fn read(v:&Value)->i64{} pub fn echo(v:Value)->Value{}";
fn conversion() -> prebindgen_declarations::ConvertDecl {
    convert!(Value).input(fun!(make)).output(fun!(read))
}
#[test]
fn canonical_values_share_function_local_and_global_compilation() {
    let global = builder(VALUE)
        .convert(conversion())
        .package(package!().fun(fun!(echo)))
        .build()
        .unwrap();
    let local = builder(VALUE)
        .package(
            package!().fun(
                fun!(echo)
                    .expand_param("v", conversion())
                    .expand_return(conversion()),
            ),
        )
        .build()
        .unwrap();
    assert_eq!(global.rust, local.rust);
    assert_eq!(global.kotlin, local.kotlin);
    assert!(global.kotlin["test/Functions.kt"].contains("v: Long"));
    assert!(!global.kotlin["test/JNINative.kt"].contains("call_make_"));
}
#[test]
fn value_properties_ignore_boundary_defaults_and_accept_local_overrides() {
    let source = format!("{VALUE} pub struct Row{{pub value:Value,pub items:Vec<Option<Value>>}} pub fn rows(v:Vec<Row>)->Vec<Row>{{}} pub fn narrow(v:i32)->Value{{}} pub fn small(v:&Value)->i32{{}}pub fn local(v:Value)->Value{{}}");
    let generated = builder(&source)
        .convert(conversion())
        .expand(expand_param!(Value).variant(fun!(narrow)))
        .expand(expand_return!(Value).field(fun!(small)))
        .package(
            package!()
                .class(data_class!(Row))
                .fun(fun!(echo))
                .fun(fun!(rows))
                .fun(
                    fun!(local)
                        .expand_param("v", convert!(Value).input(from!(i16)))
                        .expand_return(convert!(Value).output(into!(i16))),
                ),
        )
        .build()
        .unwrap();
    assert!(generated.kotlin["test/Row.kt"].contains("kotlin.Long"));
    assert!(generated.kotlin["test/Row.kt"].contains("List<kotlin.Long?>"));
    let functions = &generated.kotlin["test/Functions.kt"];
    assert!(functions.contains("fun echo(v: Int"));
    assert!(functions.contains("fun local(v: Short"));
}
#[test]
fn canonical_trait_directions_and_chains_are_compiled_statically() {
    let generated=builder("pub struct Value{pub n:i64} pub struct Intermediate{pub n:i64} pub fn echo(v:Value)->Value{}")
        .convert(convert!(Value).input(from!(Intermediate)).output(into!(Intermediate)))
        .convert(convert!(Intermediate).input(try_from!(i64)).output(try_into!(i64)))
        .package(package!().fun(fun!(echo))).build().unwrap();
    assert!(generated.kotlin["test/Functions.kt"].contains("v: Long"));
    assert!(generated.rust.contains("TryInto"));
    assert!(generated.rust.contains("ToString"));
}
#[test]
fn invalid_and_unused_canonical_declarations_are_diagnosed() {
    for declaration in [
        convert!(Value),
        convert!(Value).input(from!(i64)).input(from!(i32)),
        convert!(Value).input(into!(i64)),
        convert!(i64).input(from!(i32)),
        convert!(str).input(from!(String)),
        convert!(Result<Value,String>).input(from!(i64)),
        convert!(Box<Value>).input(from!(i64)),
    ] {
        assert!(builder(VALUE).convert(declaration).build().is_err());
    }
    assert!(builder(VALUE)
        .convert(conversion())
        .convert(conversion())
        .build()
        .is_err());
    for repr in [
        syn::parse_quote!(Value),
        syn::parse_quote!(Option<Value>),
        syn::parse_quote!(Vec<Value>),
    ] {
        let error = builder(VALUE)
            .convert(
                convert!(Value).output(prebindgen_declarations::ConvertSource::output(repr, false)),
            )
            .build()
            .err()
            .unwrap();
        assert!(
            error.contains("recursive") || error.contains("different representation"),
            "{error}"
        );
    }
    let error = builder(
        "pub struct Value{pub n:i64} pub struct Other{pub n:i64} pub fn observe(v:&Other)->Value{}",
    )
    .convert(convert!(Value).output(into!(Other)))
    .convert(convert!(Other).output(fun!(observe)))
    .build()
    .err()
    .unwrap();
    assert!(error.contains("recursive"), "{error}");
}
#[test]
fn canonical_property_directions_must_agree_on_public_type() {
    let source="pub struct Value{pub n:i64}pub struct Row{pub n:Option<Vec<Value>>}pub fn row(v:Row)->Row{}";
    let error = builder(source)
        .convert(convert!(Value).input(from!(i32)).output(into!(String)))
        .package(package!().class(data_class!(Row)).fun(fun!(row)))
        .build()
        .err()
        .unwrap();
    assert!(error.contains("incompatible canonical"), "{error}");
}
#[test]
fn one_way_values_work_for_one_way_records_and_reject_missing_directions() {
    let source="pub struct Value{pub n:i64}pub struct Row{pub n:Value}pub fn give()->Row{}pub fn take(v:Row){}";
    builder(source)
        .convert(convert!(Value).output(into!(i64)))
        .package(package!().class(data_class!(Row)).fun(fun!(give)))
        .build()
        .unwrap();
    builder(source)
        .convert(convert!(Value).input(from!(i64)))
        .package(package!().class(data_class!(Row)).fun(fun!(take)))
        .build()
        .unwrap();
    for input in [false, true] {
        let declaration = if input {
            convert!(Value).input(from!(i64))
        } else {
            convert!(Value).output(into!(i64))
        };
        let function = if input { fun!(give) } else { fun!(take) };
        let error = builder(source)
            .convert(declaration)
            .package(package!().class(data_class!(Row)).fun(function))
            .build()
            .err()
            .unwrap();
        assert!(error.contains("no canonical"), "{error}");
    }
}
#[test]
fn borrowed_sources_require_shared_output_and_explicit_input_reconstruction() {
    let source =
        format!("{VALUE}pub fn borrow(v:&Value)->i64{{}}pub fn static_value()->&'static Value{{}}");
    let error = builder(&source)
        .convert(conversion())
        .package(package!().fun(fun!(borrow)))
        .build()
        .err()
        .unwrap();
    assert!(
        error.contains("borrow") || error.contains("no handle class"),
        "{error}"
    );
    builder(&source)
        .convert(conversion())
        .package(
            package!()
                .fun(fun!(borrow).expand_param("v", expand_param!(&Value).reconstruct()))
                .fun(fun!(static_value)),
        )
        .build()
        .unwrap();
    let error = builder(&source)
        .convert(convert!(Value).output(into!(i64)))
        .package(package!().fun(fun!(static_value)))
        .build()
        .err()
        .unwrap();
    assert!(error.contains("borrowed"), "{error}");
}
#[test]
fn canonical_helpers_validate_source_signatures() {
    for helper in [
        "pub fn wrong()->Value{}",
        "pub fn wrong(a:i64,b:i64)->Value{}",
        "pub fn wrong(a:&mut i64)->Value{}",
        "pub fn wrong(a:i64)->i64{}",
    ] {
        assert!(builder(&format!("{VALUE}{helper}"))
            .convert(convert!(Value).input(fun!(wrong)))
            .build()
            .is_err());
    }
}
#[test]
fn canonical_conversions_use_existing_flat_type_entries() {
    let mut flat = model(VALUE);
    prebindgen_declarations::prepare_value_conversions(&mut flat, vec![conversion()]).unwrap();
    assert!(flat
        .type_info(&flat.resolve_type(&syn::parse_quote!(Value)).unwrap())
        .unwrap()
        .conversion
        .input
        .is_some());
    let generated = JniGen::builder()
        .flat(flat)
        .set_package_prefix("test")
        .package(package!().fun(fun!(echo)))
        .build()
        .unwrap();
    assert!(generated.kotlin["test/Functions.kt"].contains("v: Long"));
}

#[test]
fn source_selection_includes_local_trait_representations() {
    let items = syn::parse_file(
        "pub struct Value{pub n:i64}pub struct Repr{pub n:i64}pub fn echo(v:Value)->Value{}",
    )
    .unwrap()
    .items
    .into_iter()
    .map(|i| (i, Default::default()))
    .collect::<Vec<_>>();
    let function = fun!(echo).expand_param("v", convert!(Value).input(from!(crate::Repr)));
    let names = prebindgen_declarations::declarations::source_items(&[], &[], &[], [&function]);
    let selected = Bindings::parse_selected(items, &names, &[]).unwrap();
    assert!(selected.type_info(&syn::parse_quote!(crate::Repr)).is_ok());
}
