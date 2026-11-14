use super::*;

fn items(source: &str) -> Vec<(syn::Item, prebindgen::SourceLocation)> {
    syn::parse_file(source)
        .unwrap()
        .items
        .into_iter()
        .map(|item| (item, Default::default()))
        .collect()
}
fn header<'a>(text: &'a str, name: &str) -> &'a str {
    text.split_once(&format!("fun {name}("))
        .unwrap()
        .1
        .split_once("): ")
        .unwrap()
        .0
}
const RECORDS: &str = "
    pub type Handle=String;
    pub struct Row { pub owned: Handle, pub observed: Handle }
    pub struct Holder { pub row: Option<Row> }
    pub fn copy_handle(value: &Handle) -> Handle { value.clone() }
    pub fn from_number(value: i64) -> Handle { value.to_string() }
    pub fn rows(values: Vec<Row>) -> i64 { 0 }
    pub fn holder(value: Holder) -> i64 { 0 }
    pub fn callback(value: impl Fn() -> Vec<Row>) -> i64 { 0 }
";
fn row() -> DeclJniExpandParam {
    expand_param!(Row).variant_record(
        param_fields!().field("observed", expand_param!(Handle).variant(fun!(copy_handle))),
    )
}

#[test]
fn grouped_owned_inputs_keep_canonical_property_types_and_explicit_copy_helpers() {
    let generated = JniGen::builder()
        .items(items(RECORDS))
        .set_package_prefix("test")
        // This boundary default must not replace a data-class Handle property with Long.
        .expand(expand_param!(Handle).variant(fun!(from_number)))
        .package(
            package!()
                .class(ptr_class!(Handle))
                .class(data_class!(Row))
                .class(data_class!(Holder))
                .fun(
                    fun!(rows)
                        .expand_param("values", expand_param!(Vec<Row>).element(row()).bulk()),
                )
                .fun(fun!(holder).expand_param(
                    "value",
                    expand_param!(Holder).variant_record(
                        param_fields!().field("row", expand_param!(Option<Row>).optional(row())),
                    ),
                ))
                .fun(
                    fun!(callback).expand_param(
                        "value",
                        callback!(impl Fn() -> Vec<Row>)
                            .expand_return(expand_param!(Vec<Row>).element(row()).bulk()),
                    ),
                ),
        )
        .build()
        .unwrap();
    assert!(generated.kotlin["test/Row.kt"].contains("owned: test.Handle"));
    assert!(
        generated.kotlin["test/Functions.kt"].contains("values: kotlin.collections.List<test.Row>")
    );
    assert!(generated.rust.contains("copy_handle"));
    assert!(!generated.rust.contains("from_number"));
}

#[test]
fn grouped_record_and_enum_fields_reject_incompatible_single_value_conversions() {
    let source = "pub type Handle=String;pub struct Row{pub handle:Handle}
        pub enum Event{Row{handle:Handle},Empty}
        pub fn from_number(value:i64)->Handle{value.to_string()}
        pub fn row(value:Row){}pub fn event(value:Event){}";
    for enumeration in [false, true] {
        let function = if enumeration {
            fun!(event).expand_param(
                "value",
                expand_param!(Event).enum_value([(
                    "Row",
                    param_fields!()
                        .field("handle", expand_param!(Handle).variant(fun!(from_number))),
                )]),
            )
        } else {
            fun!(row).expand_param(
                "value",
                expand_param!(Row).variant_record(
                    param_fields!()
                        .field("handle", expand_param!(Handle).variant(fun!(from_number))),
                ),
            )
        };
        let error = JniGen::builder()
            .items(items(source))
            .set_package_prefix("test")
            .package(
                package!()
                    .class(ptr_class!(Handle))
                    .class(data_class!(Row))
                    .class(sealed_class!(Event))
                    .fun(function),
            )
            .build()
            .err()
            .expect("incompatible property conversion must fail generation");
        assert!(error.contains("Kotlin property"), "{error}");
        assert!(error.contains("Long"), "{error}");
    }
}

#[test]
fn named_constructor_overrides_are_local_and_validate_parameter_names() {
    let source = "pub struct Point{pub flag:bool,pub count:i64}
        pub fn point_new(flag:bool,count:i64)->Point{Point{flag,count}}
        pub fn parse_flag(value:String)->bool{true}
        pub fn first(value:Point){}pub fn second(value:Point){}";
    let generated =
        JniGen::builder()
            .items(items(source))
            .set_package_prefix("test")
            .expand(expand_param!(Point).variant(fun!(point_new)))
            .package(
                package!()
                    .fun(fun!(first).expand_param(
                        "value",
                        expand_param!(Point).variant(
                            fun!(point_new).expand_param(
                                "flag",
                                expand_param!(bool).variant(fun!(parse_flag)),
                            ),
                        ),
                    ))
                    .fun(fun!(second)),
            )
            .build()
            .unwrap();
    let functions = &generated.kotlin["test/Functions.kt"];
    assert!(header(functions, "first").contains("String"));
    assert!(!header(functions, "first").contains("Boolean"));
    assert!(header(functions, "second").contains("Boolean"));
    let error =
        JniGen::builder()
            .items(items(source))
            .set_package_prefix("test")
            .package(package!().fun(fun!(first).expand_param(
                "value",
                expand_param!(Point).variant(
                    fun!(point_new).expand_param("typo", expand_param!(bool).variant_self()),
                ),
            )))
            .build()
            .err()
            .unwrap();
    assert!(
        error.contains("constructor `point_new`: unknown parameter `typo`"),
        "{error}"
    );
}

#[test]
fn getter_result_overrides_accept_borrowed_logical_types_without_changing_other_calls() {
    let source = "pub type Owner=String;pub type Leaf=String;
        pub fn leaf(owner:&Owner)->&Leaf{owner}
        pub fn length(value:&Leaf)->i64{value.len() as i64}
        pub fn first()->Owner{String::new()}pub fn second()->Owner{String::new()}";
    let generated =
        JniGen::builder()
            .items(items(source))
            .set_package_prefix("test")
            .expand(expand_return!(Leaf).field_self_cloned())
            .expand(expand_return!(Owner).field(fun!(leaf)))
            .package(
                package!()
                    .class(ptr_class!(Owner))
                    .class(ptr_class!(Leaf))
                    .fun(fun!(first).expand_return(
                        expand_return!(Owner).field(
                            fun!(leaf).expand_return(expand_return!(Leaf).field(fun!(length))),
                        ),
                    ))
                    .fun(fun!(second)),
            )
            .build()
            .unwrap();
    let functions = &generated.kotlin["test/Functions.kt"];
    assert!(functions.contains("fun first("));
    assert!(generated.rust.contains("length"));
    // Distinct aliases must remain distinct source contracts even though both name String.
    assert!(functions.contains("test.Leaf"));
}

#[test]
fn source_fields_and_enum_fields_validate_names_before_compiling() {
    let source = "pub struct Row{pub x:i64,pub y:i64}pub enum E{A{x:i64},B}
        pub fn record(value:Row)->Row{value}pub fn event(value:E)->E{value}";
    for function in [
        fun!(record).expand_param(
            "value",
            expand_param!(Row).variant_fields(
                param_fields!().field("missing", expand_param!(i64).variant_self()),
            ),
        ),
        fun!(record).expand_return(
            expand_return!(Row)
                .fields_into(fields!().field("missing", expand_return!(i64).field_self())),
        ),
        fun!(event).expand_param(
            "value",
            expand_param!(E).enum_fields([("Unknown", param_fields!())]),
        ),
        fun!(event).expand_return(expand_return!(E).enum_fields([(
            "A",
            fields!().field("missing", expand_return!(i64).field_self()),
        )])),
    ] {
        let error = JniGen::builder()
            .items(items(source))
            .set_package_prefix("test")
            .package(
                package!()
                    .class(data_class!(Row))
                    .class(sealed_class!(E))
                    .fun(function),
            )
            .build()
            .err()
            .unwrap();
        assert!(error.contains("unknown"), "{error}");
    }
}

#[test]
fn selected_element_conversions_preserve_default_bulk_and_allow_explicit_objects() {
    let source = "pub struct Row{pub x:i64,pub y:i64}pub fn defaulted()->Vec<Row>{vec![]}
        pub fn objects()->Vec<Row>{vec![]}pub fn rows()->Vec<Row>{vec![]}";
    let generated = JniGen::builder()
        .items(items(source))
        .set_package_prefix("test")
        .package(
            package!()
                .class(data_class!(Row))
                .fun(fun!(defaulted).expand_return(
                    expand_return!(Vec<Row>).element(expand_return!(Row).field_self()),
                ))
                .fun(fun!(objects).expand_return(
                    expand_return!(Vec<Row>).element_objects(expand_return!(Row).field_self()),
                ))
                .fun(fun!(rows)),
        )
        .build()
        .unwrap();
    let native = &generated.kotlin["test/JNINative.kt"];
    for (name, bulk) in [("defaulted", true), ("objects", false), ("rows", true)] {
        let signature = native
            .lines()
            .find(|line| line.contains("external fun") && line.contains(name))
            .unwrap();
        assert_eq!(signature.contains("ByteArray"), bulk, "{signature}");
    }
}

#[test]
fn grouped_output_enum_fields_reject_values_incompatible_with_the_declared_property() {
    let source = "pub type Handle=String;pub enum Event{Row{handle:Handle},Empty}
        pub fn number(value:&Handle)->i64{0} pub fn event()->Event{unimplemented!()}";
    let error = JniGen::builder()
        .items(items(source))
        .set_package_prefix("test")
        .package(
            package!()
                .class(ptr_class!(Handle))
                .class(sealed_class!(Event))
                .fun(
                    fun!(event).expand_return(expand_return!(Event).enum_value([(
                        "Row",
                        fields!().field("handle", convert!(Handle).output(fun!(number))),
                    )])),
                ),
        )
        .build()
        .err()
        .expect("invalid grouped output must fail generation");
    assert!(error.contains("Kotlin property"), "{error}");
}

#[test]
fn grouped_property_checks_preserve_nullable_list_covariance() {
    use crate::data::{check_property_input, check_property_output};
    let plain = "kotlin.collections.List<kotlin.collections.List<test.Handle>>";
    let nullable = "kotlin.collections.List<kotlin.collections.List<test.Handle?>?>?";
    assert!(check_property_input(plain, &[format!("value: {nullable}")]).is_ok());
    assert!(check_property_input(nullable, &[format!("value: {plain}")]).is_err());
    assert!(check_property_output(nullable, &[plain.into()]).is_ok());
    assert!(check_property_output(plain, &[nullable.into()]).is_err());
    assert!(check_property_input("test.kotlin.Handle", &["value: test.Handle".into()]).is_err());
    assert!(check_property_output("test.kotlin.Handle", &["test.Handle".into()]).is_err());
}

#[test]
fn local_constructor_arguments_use_the_same_raw_identifier_names_as_functions() {
    let source =
        "pub struct Point{pub flag:bool}pub fn point_new(r#type:bool)->Point{unimplemented!()}
        pub fn parse_flag(value:String)->bool{true}pub fn accept(value:Point){}";
    let generated = JniGen::builder()
        .items(items(source))
        .set_package_prefix("test")
        .package(
            package!().fun(
                fun!(accept).expand_param(
                    "value",
                    expand_param!(Point).variant(
                        fun!(point_new)
                            .expand_param("r#type", expand_param!(bool).variant(fun!(parse_flag))),
                    ),
                ),
            ),
        )
        .build()
        .unwrap();
    assert!(header(&generated.kotlin["test/Functions.kt"], "accept").contains("String"));
}
