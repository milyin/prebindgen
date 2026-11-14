use super::*;
use crate::fields;
use prebindgen_declarations::convert;

use crate::{JniInputUnary, JniOutputBranch, JniOutputLeafOp};

fn build(source: &str) -> JniGenBuilder {
    JniGen::builder()
        .items(
            syn::parse_file(source)
                .unwrap()
                .items
                .into_iter()
                .map(|i| (i, Default::default())),
        )
        .set_package_prefix("test")
}
#[test]
fn payload_enum_round_trip_uses_sealed_properties() {
    let generated=build("pub enum Event {Idle,Count(u64),Packet{id:u64,bytes:Vec<u8>}}pub fn echo(value:Event)->Event{value}")
        .package(package!().class(sealed_class!(Event)).fun(fun!(echo))).build().unwrap();
    syn::parse_file(&generated.rust).unwrap();
    let class = &generated.kotlin["test/Event.kt"];
    assert!(class.contains("sealed interface Event"), "{class}");
    assert!(class.contains("data object Idle"), "{class}");
    assert!(class.contains("kotlin.ULong"), "{class}");
    assert!(class.contains("contentEquals"), "{class}");
    let functions = &generated.kotlin["test/Functions.kt"];
    assert!(functions.contains(": test.Event"), "{functions}");
}
#[test]
fn recovery_config_nests_optional_enum_with_canonical_duration() {
    let source="pub struct Duration{pub ms:u64}pub fn duration(ms:u64)->Duration{}pub fn millis(d:Duration)->u64{}pub enum RecoveryMode{PeriodicQueries(Duration),Heartbeat}pub struct RecoveryConfig{pub mode:Option<RecoveryMode>,pub retention_period:Option<Duration>}pub fn echo(value:RecoveryConfig)->RecoveryConfig{}";
    let generated = build(source)
        .convert(
            convert!(Duration)
                .input(fun!(duration))
                .output(fun!(millis)),
        )
        .package(
            package!()
                .class(sealed_class!(RecoveryMode))
                .class(data_class!(RecoveryConfig))
                .fun(fun!(echo)),
        )
        .build()
        .unwrap();
    syn::parse_file(&generated.rust).unwrap();
    assert!(generated.kotlin["test/RecoveryConfig.kt"].contains("test.RecoveryMode?"));
    assert!(generated.kotlin["test/RecoveryMode.kt"].contains("kotlin.ULong"));
}
#[test]
fn fields_only_callback_inherits_owned_payload_expansion_without_classes() {
    let source="pub struct Sample{pub n:i64}pub enum ReplyResult{Sample(Sample),Error(String)}pub struct ReplyStruct{pub result:ReplyResult,pub replier_id:Option<i64>}pub type Reply=String;pub fn reply_into_struct(value:Reply)->ReplyStruct{}pub fn receive(cb:impl Fn(Reply)){}";
    let generated = build(source)
        .expand(expand_return!(Sample).view(JniOutputConfig::Branch {
            operation: JniOutputBranch::OwnedFields,
            children: vec![JniOutputConfig::auto()],
        }))
        .expand(expand_return!(ReplyResult).view(JniOutputConfig::enum_fields([])))
        .expand(expand_return!(Reply).fields_into(fields!(reply_into_struct)))
        .package(package!().fun(fun!(receive)))
        .build()
        .unwrap();
    syn::parse_file(&generated.rust).unwrap();
    assert!(!generated.kotlin.contains_key("test/ReplyResult.kt"));
}
#[test]
fn standard_result_is_callback_data_with_unit_payload() {
    let generated = build("pub fn receive(cb:impl Fn(Result<(),String>)->Result<i64,String>){}")
        .package(
            package!()
                .class(sealed_class!(Result<(),String>).name("Notice"))
                .class(sealed_class!(Result<i64,String>).name("Decision"))
                .fun(fun!(receive)),
        )
        .build()
        .unwrap();
    syn::parse_file(&generated.rust).unwrap();
    assert!(generated.kotlin["test/Notice.kt"].contains("kotlin.Unit"));
}

#[test]
fn ordinary_enum_requires_a_declared_value_class() {
    let error = build("pub enum Event{Idle,Count(i64)}pub fn echo(value:Event)->Event{value}")
        .package(package!().fun(fun!(echo)))
        .build()
        .err()
        .unwrap();
    assert!(error.contains("sealed_class"), "{error}");
}
#[test]
fn local_field_expansion_needs_no_value_class_and_publishes_source_tags() {
    let generated = build("pub enum Event{Idle,Count(i64)}pub fn echo(value:Event)->Event{value}")
        .package(
            package!().fun(
                fun!(echo)
                    .expand_param("value", JniInputConfig::enum_fields([]))
                    .expand_return(JniOutputConfig::enum_fields([])),
            ),
        )
        .build()
        .unwrap();
    assert!(!generated.kotlin.contains_key("test/Event.kt"));
    assert!(generated.kotlin["test/PayloadTags.kt"].contains("EVENT_COUNT_TAG: Int = 1"));
    assert!(generated.kotlin["test/JNINative.kt"].contains("incompatible payload enum tag mapping"));
}
#[test]
fn generic_value_classes_need_explicit_names() {
    let error = build("pub fn echo(value:Result<i64,String>)->Result<i64,String>{value}")
        .package(
            package!()
                .class(sealed_class!(Result<i64,String>))
                .fun(fun!(echo).result_as_value()),
        )
        .build()
        .err()
        .unwrap();
    assert!(error.contains("explicit .name"), "{error}");
}
#[test]
fn result_data_is_not_also_a_function_error_channel() {
    let error =
        build("pub fn echo()->Result<i64,String>{}pub fn message(value:String)->String{value}")
            .package(
                package!()
                    .class(sealed_class!(Result<i64,String>).name("Decision"))
                    .fun(fun!(echo).result_as_value().on_error(fun!(message))),
            )
            .build()
            .err()
            .unwrap();
    assert!(error.contains("result_as_value"), "{error}");
}
#[test]
fn enum_override_keys_are_checked_against_source_members() {
    let error = build("pub enum Event{Idle,Count(i64)}pub fn echo(value:Event)->Event{value}")
        .package(package!().fun(fun!(echo).expand_param(
            "value",
            JniInputConfig::enum_fields([("Count", vec![("missing", JniInputConfig::auto())])]),
        )))
        .build()
        .err()
        .unwrap();
    assert!(
        error.contains("unknown enum field `Count.missing`"),
        "{error}"
    );
}
#[test]
fn exact_result_constructor_is_data_and_not_a_fallible_helper() {
    let generated=build("pub fn decide(flag:bool)->Result<i64,String>{}pub fn use_decision(value:Result<i64,String>)->i64{}")
        .convert(convert!(Result<i64,String>).input(fun!(decide)))
        .package(package!().fun(fun!(use_decision))).build().unwrap();
    syn::parse_file(&generated.rust).unwrap();
    assert!(generated.kotlin["test/Functions.kt"].contains(": Boolean"));
}
#[test]
fn canonical_output_helper_can_return_result_as_data() {
    let generated=build("pub struct Domain{pub n:i64}pub fn as_decision(value:Domain)->Result<i64,String>{}pub fn make()->Domain{}")
        .convert(convert!(Domain).output(fun!(as_decision).result_as_value()))
        .package(package!().class(sealed_class!(Result<i64,String>).name("Decision")).fun(fun!(make))).build().unwrap();
    syn::parse_file(&generated.rust).unwrap();
    assert!(generated.kotlin["test/Functions.kt"].contains("test.Decision"));
}
#[test]
fn bulk_enums_use_target_union_offsets() {
    let generated=build("pub enum Event{Idle,Count(u64),Packet{id:u64,bytes:Vec<u8>}}pub fn echo(values:Vec<Event>)->Vec<Event>{values}")
        .package(package!().class(sealed_class!(Event)).fun(fun!(echo).expand_param("values",JniInputConfig::Unary { operation: JniInputUnary::Sequence, child: Box::new(JniInputConfig::value()) }.bulk()))).build().unwrap();
    syn::parse_file(&generated.rust).unwrap();
    assert!(generated.rust.contains("pub union __PbBulk_"));
    assert!(generated.rust.contains("offset_of!"));
    assert!(generated.kotlin["test/Functions.kt"].contains("BulkWriter"));
}

#[test]
fn enum_used_only_for_a_domain_error_is_part_of_the_tag_catalogue() {
    let generated=build("pub enum DomainError{Code(i64),Message(String)}pub fn operation()->Result<(),DomainError>{}")
        .package(package!().class(sealed_class!(DomainError)).fun(fun!(operation))).build().unwrap();
    assert!(generated.kotlin["test/PayloadTags.kt"].contains("DOMAIN_ERROR_CODE_TAG"));
}
#[test]
fn a_variant_can_have_the_same_name_as_its_interface() {
    let generated = build("pub enum Event{Event(i64),Empty}pub fn echo(value:Event)->Event{value}")
        .package(package!().class(sealed_class!(Event)).fun(fun!(echo)))
        .build()
        .unwrap();
    assert!(generated.kotlin["test/Event.kt"].contains(": test.Event"));
}
#[test]
fn grouped_enum_fields_reject_multiple_logical_output_values() {
    let generated = build("pub enum Event{One(i64),Empty}pub fn echo()->Event{}")
        .package(
            package!()
                .class(sealed_class!(Event))
                .fun(fun!(echo).expand_return(JniOutputConfig::enum_variants(
                    vec![
                        vec![JniOutputConfig::Branch {
                            operation: JniOutputBranch::Combine,
                            children: vec![
                                JniOutputConfig::Leaf(JniOutputLeafOp::Value),
                                JniOutputConfig::Leaf(JniOutputLeafOp::Value),
                            ],
                        }],
                        vec![],
                    ],
                    true,
                    None,
                ))),
        )
        .build();
    let error = generated.err().unwrap();
    assert!(error.contains("one Kotlin value"), "{error}");
}
