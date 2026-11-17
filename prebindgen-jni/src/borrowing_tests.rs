use super::*;

fn items(source: &str) -> Vec<(syn::Item, prebindgen::SourceLocation)> {
    syn::parse_file(source)
        .unwrap()
        .items
        .into_iter()
        .map(|item| (item, Default::default()))
        .collect()
}

#[test]
fn cow_byte_outputs_use_the_scoped_byte_encoder() {
    let generated = JniGen::builder().items(items("pub type Owner=String;pub fn bytes(owner:&Owner)->Cow<'_,[u8]>{}pub fn optional(owner:&Owner)->Option<Cow<'_,[u8]>>{}pub fn register(callback:impl Fn(Cow<'_,[u8]>)+Send+Sync+'static){}"))
        .set_package_prefix("test")
        .package(package!().class(ptr_class!(Owner)).fun(fun!(bytes)).fun(fun!(optional)).fun(fun!(register)))
        .build().unwrap();
    syn::parse_file(&generated.rust).unwrap();
    let functions = &generated.kotlin["test/Functions.kt"];
    assert!(functions.contains("ByteArray"));
    assert!(generated.rust.contains("bytes_object"));
    assert!(!generated.rust.contains("into_owned"));
    assert!(!generated.rust.contains("to_vec"));
    assert!(!generated.rust.contains("Clone::clone"));
}

#[test]
fn cow_rows_use_byte_fields_in_both_collection_transports() {
    for bulk in [false, true] {
        let generated = JniGen::builder()
            .items(items("pub fn rows()->Vec<Cow<'static,[u8]>>{}"))
            .set_package_prefix("test")
            .set_bulk_collection_outputs(bulk)
            .package(package!().fun(fun!(rows)))
            .build()
            .unwrap();
        syn::parse_file(&generated.rust).unwrap();
        assert!(
            generated.kotlin["test/Functions.kt"].contains("List<kotlin.ByteArray>")
                || generated.kotlin["test/Functions.kt"].contains("List<ByteArray>")
        );
        assert_eq!(generated.rust.contains("append_buffer"), bulk);
        assert!(!generated.rust.contains("into_owned"));
    }
}

#[test]
fn cow_input_requires_an_explicit_constructor() {
    let error = JniGen::builder()
        .items(items("pub fn read(value:Cow<'_,[u8]>){}"))
        .set_package_prefix("test")
        .package(package!().fun(fun!(read)))
        .build()
        .err()
        .unwrap();
    assert!(error.contains("Cow byte inputs"), "{error}");
    assert!(error.contains("explicit source constructor"), "{error}");
    JniGen::builder().items(items("pub fn from_bytes(value:&[u8])->Cow<'_,[u8]>{}pub fn read(value:Cow<'_,[u8]>)->usize{}"))
        .set_package_prefix("test")
        .package(package!().fun(fun!(read).expand_param("value",expand_param!(std::borrow::Cow<'_,[u8]>)
            .variant(fun!(from_bytes)))))
        .build().unwrap();
}

#[test]
fn placeholder_signatures_share_callback_configuration_and_type_defaults() {
    let generated=JniGen::builder().items(items("pub type Owner=String;pub fn read(value:&'_ Owner)->&'_ str{}pub fn register(callback:impl Fn(&'_ Owner)+Send+Sync+'static){}"))
        .set_package_prefix("test")
        .expand(expand_param!(&Owner).variant_self())
        .package(package!().class(ptr_class!(Owner))
            .class(callback!(impl Fn(&Owner)).name("Observer").expand_param(0,expand_return!(Owner).field_self_cloned()))
            .fun(fun!(read)).fun(fun!(register)))
        .build().unwrap();
    syn::parse_file(&generated.rust).unwrap();
    assert!(generated.kotlin.contains_key("test/Observer.kt"));
    assert!(!generated.rust.contains("&'_"));
}

#[test]
fn mutable_value_inputs_report_the_missing_mutation_contract() {
    for ty in ["i64", "[u8]", "String"] {
        let error = JniGen::builder()
            .items(items(&format!("pub fn update(value:&mut {ty}){{}}")))
            .set_package_prefix("test")
            .package(package!().fun(fun!(update)))
            .build()
            .err()
            .unwrap();
        assert!(error.contains("mutable JNI input"), "{ty}: {error}");
        assert!(
            error.contains("native handle class") && error.contains("copy-back"),
            "{ty}: {error}"
        );
        assert!(!error.contains("no handle class"), "{ty}: {error}");
    }
    JniGen::builder()
        .items(items(
            "pub type Owner=String;pub fn update(value:&mut Owner){}",
        ))
        .set_package_prefix("test")
        .package(package!().class(ptr_class!(Owner)).fun(fun!(update)))
        .build()
        .unwrap();
}
