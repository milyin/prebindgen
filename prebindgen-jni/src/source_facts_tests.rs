//! Emission reads declaration facts from Flat after occurrence preparation.
use super::*;
use std::rc::Rc;

#[test]
fn prepared_records_read_current_class_and_property_declarations() {
    let items = syn::parse_file("pub struct Row {pub value:i64}pub fn echo(row:Row)->Row{}")
        .unwrap()
        .items;
    let mut flat = JniGen::builder()
        .items(items.into_iter().map(|i| (i, Default::default())))
        .set_package_prefix("example")
        .package(package!().class(data_class!(Row)).fun(fun!(echo)))
        .prepare()
        .unwrap();
    let ty = flat.resolve_type(&syn::parse_quote!(Row)).unwrap();
    Rc::make_mut(flat.type_metadata_mut(&ty).unwrap().class.as_mut().unwrap()).name =
        "Renamed".into();
    let generated = generate(&flat).unwrap();
    assert!(generated.kotlin.contains_key("example/Renamed.kt"));
    let functions = &generated.kotlin["example/Functions.kt"];
    assert!(functions.contains(": example.Renamed"));
    assert!(!functions.contains("example.Row"));
    // Validation must also consult the canonical properties, rather than the
    // property-type snapshot that was available when the converter was built.
    flat.record_by_key_mut(&type_key(&ty)).unwrap().fields[0]
        .metadata
        .kotlin_type = "kotlin.String".into();
    let error = generate(&flat).err().unwrap();
    assert!(error.contains("kotlin.String"), "{error}");
}

#[test]
fn prepared_enum_callbacks_and_bulk_rows_read_current_variant_names() {
    let items = syn::parse_file("pub enum Event {Empty, Row{value:i64}}pub fn echo(event:Event)->Event{}pub fn rows()->Vec<Event>{}pub fn callback(cb:impl Fn(Event)->Event){}").unwrap().items;
    let mut flat = JniGen::builder()
        .items(items.into_iter().map(|i| (i, Default::default())))
        .set_package_prefix("example")
        .package(
            package!()
                .class(sealed_class!(Event))
                .fun(fun!(echo))
                .fun(fun!(rows))
                .fun(fun!(callback)),
        )
        .prepare()
        .unwrap();
    let ty = flat.resolve_type(&syn::parse_quote!(Event)).unwrap();
    let class = Rc::make_mut(flat.type_metadata_mut(&ty).unwrap().class.as_mut().unwrap());
    class.name = "RenamedEvent".into();
    let ClassRepresentation::Sealed { variant_names, .. } = &mut class.representation else {
        panic!()
    };
    variant_names.push(("Row".into(), "RenamedRow".into()));
    let generated = generate(&flat).unwrap();
    assert!(generated.kotlin.contains_key("example/RenamedEvent.kt"));
    assert!(generated.kotlin["example/Functions.kt"].contains("RenamedEvent.RenamedRow"));
    assert!(generated
        .kotlin
        .iter()
        .any(|(path, text)| path.contains("Callback") && text.contains("example.RenamedEvent")));
    let old_class = regex::Regex::new(r"\bexample\.Event\b").unwrap();
    assert!(!generated
        .kotlin
        .values()
        .any(|text| old_class.is_match(text)));
    let payload = flat.payload_enum_by_key_mut(&type_key(&ty)).unwrap();
    payload.variants[1].fields[0].metadata.kotlin_type = "kotlin.String".into();
    let error = generate(&flat).err().unwrap();
    assert!(error.contains("kotlin.String"), "{error}");
}

#[test]
fn changed_enum_field_shapes_are_rejected_after_preparation() {
    let items = syn::parse_file(
        "pub enum Event{Row{first:i64,second:i64}}pub fn echo(event:Event)->Event{}",
    )
    .unwrap()
    .items;
    let flat = JniGen::builder()
        .items(items.into_iter().map(|i| (i, Default::default())))
        .set_package_prefix("example")
        .package(package!().class(sealed_class!(Event)).fun(fun!(echo)))
        .prepare()
        .unwrap();
    let ty = flat.resolve_type(&syn::parse_quote!(Event)).unwrap();
    for mode in 0..3 {
        let mut edited = flat.clone();
        let fields = &mut edited
            .payload_enum_by_key_mut(&type_key(&ty))
            .unwrap()
            .variants[0]
            .fields;
        match mode {
            0 => {
                fields.pop();
            }
            1 => {
                let mut extra = fields[0].clone();
                extra.member = syn::parse_quote!(third);
                fields.push(extra);
            }
            _ => fields.swap(0, 1),
        }
        let error = generate(&edited).err().unwrap();
        assert!(error.contains("source variant"), "{error}");
    }
}

#[test]
fn changed_bulk_source_field_shapes_are_rejected_after_preparation() {
    let items = syn::parse_file("pub enum Event{Empty,Row(i64)}pub fn rows()->Vec<Event>{}")
        .unwrap()
        .items;
    let mut flat = JniGen::builder()
        .items(items.into_iter().map(|i| (i, Default::default())))
        .set_package_prefix("example")
        .package(package!().class(sealed_class!(Event)).fun(fun!(rows)))
        .prepare()
        .unwrap();
    let ty = flat.resolve_type(&syn::parse_quote!(Event)).unwrap();
    flat.payload_enum_by_key_mut(&type_key(&ty))
        .unwrap()
        .variants
        .swap(0, 1);
    let error = generate(&flat).err().unwrap();
    assert!(error.contains("source variant"), "{error}");
}
