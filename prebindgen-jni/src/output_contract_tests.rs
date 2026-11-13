use super::*;
use prebindgen_conversions::{Delivery, DeliveryPolicy, OutputFragment};
use prebindgen_flat::{Function, InputConversion, TypeUse};
use std::{cell::Cell, rc::Rc};

struct ExternalOutput {
    native: JniOutputConverter,
    preparations: Rc<Cell<usize>>,
    projections: Rc<Cell<usize>>,
    omit_delivery: bool,
    mark_delivery: bool,
}
impl Conversion for ExternalOutput {
    type Wire = JniWire;
    type Chain = JniChain;
    type Branch = JniBranch;
    type SlotFragment<'a> = OutputFragment<'a, JniContext>;
    fn code_fragment_slot(
        &self,
        request: Self::SlotFragment<'_>,
        expected: &syn::Type,
    ) -> Result<TokenStream> {
        let delivery = matches!(&request, OutputFragment::Deliver { .. });
        let code = self.native.code_fragment_slot(request, expected)?;
        if delivery && self.mark_delivery {
            Ok(quote!({ let __pb_custom_packet_marker = (); #code }))
        } else {
            Ok(code)
        }
    }
}
impl OutputConversion for ExternalOutput {
    fn output_outcomes(&self) -> &prebindgen_flat::Outcomes<kotlin::JniNode> {
        self.native.output_outcomes()
    }
}
impl KotlinOutputConversion for ExternalOutput {
    fn kotlin(&self, context: &KotlinContext<'_>) -> Result<KotlinOutput> {
        self.native.kotlin(context)
    }
    fn delivery_policy(&self) -> DeliveryPolicy {
        self.native.delivery_policy()
    }
    fn prepare_return_delivery(&self, context: &JniContext) -> Result<Option<Delivery>> {
        self.preparations.set(self.preparations.get() + 1);
        if self.omit_delivery {
            Ok(None)
        } else {
            self.native.prepare_return_delivery(context)
        }
    }
    fn callback_arguments(
        &self,
        context: &JniContext,
        source: &syn::Ident,
    ) -> Result<Vec<(syn::Ident, TokenStream)>> {
        self.native.callback_arguments(context, source)
    }
    fn kotlin_return(&self, context: &KotlinReturnContext<'_>) -> Result<KotlinOutput> {
        self.projections.set(self.projections.get() + 1);
        self.native.kotlin_return(context)
    }
}

fn prepared() -> JniBindings {
    let source = syn::parse_file("pub type Handle=String;pub struct Pair{pub a:i64,pub b:i64}pub fn scalar()->i64{} pub fn unit(){} pub fn text()->String{} pub fn handle()->Handle{} pub fn pair()->Pair{} pub fn rows()->Vec<Pair>{} pub fn failed()->Result<i64,i64>{} pub fn also_failed()->Result<i64,i64>{} pub fn invoke(callback:impl Fn(i64)->i64)->i64{} pub fn also_invoke(callback:impl Fn(i64)->i64)->i64{} pub fn overloaded_i64(n:i64)->Result<i64,i64>{} pub fn overloaded_string(n:String)->Result<i64,i64>{}").unwrap();
    JniGen::builder()
        .items(
            source
                .items
                .into_iter()
                .map(|item| (item, Default::default())),
        )
        .set_package_prefix("test")
        .package(
            package!()
                .class(ptr_class!(Handle))
                .class(data_class!(Pair))
                .fun(fun!(scalar))
                .fun(fun!(unit))
                .fun(fun!(text))
                .fun(fun!(handle))
                .fun(fun!(pair))
                .fun(fun!(rows).callback_return())
                .fun(fun!(failed))
                .fun(fun!(also_failed))
                .fun(fun!(invoke))
                .fun(fun!(also_invoke))
                .fun(fun!(overloaded_i64))
                .fun(fun!(overloaded_string)),
        )
        .prepare()
        .unwrap()
}

fn external(flat: JniBindings) -> JniBindings<JniInputConverter, ExternalOutput> {
    Flat {
        metadata: flat.metadata,
        records: flat.records,
        enums: flat.enums,
        payload_enums: flat.payload_enums,
        types: flat.types,
        guards: flat.guards,
        functions: flat
            .functions
            .into_iter()
            .map(|(key, f)| {
                (
                    key,
                    Function {
                        metadata: f.metadata,
                        name: f.name,
                        path: f.path,
                        location: f.location,
                        params: f.params,
                        ret: TypeUse {
                            ty: f.ret.ty,
                            slot: f.ret.slot.map(|native| ExternalOutput {
                                native,
                                preparations: Rc::default(),
                                projections: Rc::default(),
                                omit_delivery: false,
                                mark_delivery: false,
                            }),
                        },
                    },
                )
            })
            .collect(),
    }
}

#[test]
fn independent_outputs_use_the_same_emitter_and_prepare_delivery_once() {
    let flat = prepared();
    let expected = generate(&flat).unwrap();
    let flat = external(flat);
    let actual = generate(&flat).unwrap();
    assert_eq!(actual.rust, expected.rust);
    assert_eq!(actual.kotlin, expected.kotlin);
    for function in flat.functions.values() {
        if let Some(output) = &function.ret.slot {
            assert_eq!(output.preparations.get(), 1, "{}", function.name);
            assert_eq!(output.projections.get(), 1, "{}", function.name);
        }
    }
}

#[test]
fn a_missing_selected_delivery_is_a_generation_error() {
    let mut flat = external(prepared());
    flat.function_mut("scalar")
        .unwrap()
        .ret
        .slot
        .as_mut()
        .unwrap()
        .omit_delivery = true;
    assert!(generate(&flat)
        .err()
        .unwrap()
        .contains("did not prepare return delivery"));
}

#[test]
fn packet_policy_is_explicit_and_does_not_require_token_recognition() {
    let mut flat = prepared();
    let function = flat.function_mut("scalar").unwrap();
    function.ret.slot = Some(function.ret.slot.take().unwrap().packet_only());
    let mut flat = external(flat);
    flat.function_mut("scalar")
        .unwrap()
        .ret
        .slot
        .as_mut()
        .unwrap()
        .mark_delivery = true;
    let actual = generate(&flat).unwrap();
    assert!(actual.rust.contains("__pb_custom_packet_marker"));
    assert!(actual.kotlin["test/JNINative.kt"].contains("): Array<Any?>"));
    assert_eq!(
        flat.function("scalar")
            .unwrap()
            .ret
            .slot
            .as_ref()
            .unwrap()
            .preparations
            .get(),
        0
    );
}

#[test]
fn local_error_override_rejects_an_incompatible_wire_description() {
    use crate::operations::{JniOutput, JniOutputLeafOp};

    let mut flat = prepared();
    let output = prebindgen_declarations::compile_output_value(
        &JniBindings::parse([]).unwrap(),
        &JniCodec::raw(),
        &syn::parse_quote!(String),
        &JniOutput::Leaf(JniOutputLeafOp::Value),
        &(),
    )
    .unwrap();
    bind_error_output(&mut flat, "failed", output).unwrap();
    let error = generate(&flat).err().unwrap();
    assert!(error.contains("domain-error converter differs"), "{error}");
}

#[test]
fn one_prepared_output_supports_packet_direct_and_capture_without_mutation() {
    let flat = prepared();
    let output = flat.function("scalar").unwrap().ret.slot.as_ref().unwrap();
    let mut context = JniContext {
        input_transfers: prebindgen_conversions::InputTransfers::Forbidden,
        return_delivery: JniReturnDelivery::Packet,
        prepared_delivery: None,
        environment: syn::parse_quote!(env),
        domain_output: syn::parse_quote!(unused_domain),
        return_sink: Some(syn::parse_quote!(capture)),
        is_domain_error: false,
        output_committed: quote!(),
    };
    let mut codes = Vec::new();
    for delivery in [
        JniReturnDelivery::Packet,
        JniReturnDelivery::Direct,
        JniReturnDelivery::Capture,
        JniReturnDelivery::Packet,
    ] {
        context.return_delivery = delivery;
        returns::prepare(output, &mut context).unwrap();
        codes.push(
            context
                .prepared_delivery
                .as_ref()
                .unwrap()
                .statements
                .to_string(),
        );
    }
    assert!(codes[0].contains("array"));
    assert!(codes[1].contains("__pb_direct_result"));
    assert!(codes[2].contains("__PB_RETURN_METHOD_0"));
    assert_eq!(codes[0], codes[3]);
    assert_eq!(output.delivery_policy(), DeliveryPolicy::AdapterSelected);
}

struct TextOutput(JniOutputConverter);
impl Conversion for TextOutput {
    type Wire = JniWire;
    type Chain = JniChain;
    type Branch = JniBranch;
    type SlotFragment<'a> = OutputFragment<'a, JniContext>;
    fn code_fragment_slot(
        &self,
        request: Self::SlotFragment<'_>,
        expected: &syn::Type,
    ) -> Result<TokenStream> {
        self.0.code_fragment_slot(request, expected)
    }
}
impl OutputConversion for TextOutput {
    fn output_outcomes(&self) -> &prebindgen_flat::Outcomes<kotlin::JniNode> {
        self.0.output_outcomes()
    }
}
impl KotlinOutputConversion for TextOutput {
    fn kotlin(&self, context: &KotlinContext<'_>) -> Result<KotlinOutput> {
        let mut output = self.0.kotlin(context)?;
        for (value, ty) in output.delivered.iter_mut().zip(&mut output.result_types) {
            *value = format!("({value}).toString()");
            *ty = "String".into();
        }
        Ok(output)
    }
    fn callback_arguments(
        &self,
        context: &JniContext,
        source: &syn::Ident,
    ) -> Result<Vec<(syn::Ident, TokenStream)>> {
        self.0.callback_arguments(context, source)
    }
}
fn callback(flat: &JniBindings, name: &str) -> JniCallback {
    let input = flat.function(name).unwrap().params[0]
        .ty
        .slot
        .as_ref()
        .unwrap();
    prebindgen_flat::wire_definitions(std::iter::once(input.input_node()))
        .find_map(|wire| {
            if let JniWire::Callback { callback } = &wire.kind {
                Some(callback.as_ref().clone())
            } else {
                None
            }
        })
        .unwrap()
}
#[test]
fn a_local_callback_projection_needs_a_distinct_shared_interface_name() {
    let mut flat = prepared();
    let mut custom = callback(&flat, "invoke");
    let native = flat
        .function("scalar")
        .unwrap()
        .ret
        .slot
        .as_ref()
        .unwrap()
        .clone();
    custom
        .set_argument_converter(0, TextOutput(native))
        .unwrap();
    flat.function_mut("invoke").unwrap().params[0].ty.slot = Some(
        custom
            .clone()
            .into_input(syn::parse_quote!(wire), syn::parse_quote!(value))
            .unwrap(),
    );
    assert!(generate(&flat)
        .err()
        .unwrap()
        .contains("signature collision"));
    custom.set_interface_name("TextCallback").unwrap();
    flat.function_mut("invoke").unwrap().params[0].ty.slot = Some(
        custom
            .into_input(syn::parse_quote!(wire), syn::parse_quote!(value))
            .unwrap(),
    );
    let output = generate(&flat).unwrap();
    assert!(output.kotlin["test/TextCallback.kt"].contains("arg0_0: String"));
    assert!(output.kotlin["test/i64Toi64Callback.kt"].contains("arg0_0: Long"));
}
#[test]
fn local_error_handlers_are_independent_for_overloads() {
    let mut flat = prepared();
    for name in ["overloaded_i64", "overloaded_string"] {
        flat.function_mut(name)
            .unwrap()
            .metadata
            .as_mut()
            .unwrap()
            .name = "overloaded".into();
        let native = flat
            .function(name)
            .unwrap()
            .metadata
            .as_ref()
            .unwrap()
            .error
            .as_ref()
            .unwrap()
            .output()
            .unwrap()
            .clone();
        bind_error_output(&mut flat, name, native).unwrap();
    }
    let first = flat
        .function("overloaded_i64")
        .unwrap()
        .metadata
        .as_ref()
        .unwrap()
        .error
        .as_ref()
        .unwrap();
    let second = flat
        .function("overloaded_string")
        .unwrap()
        .metadata
        .as_ref()
        .unwrap()
        .error
        .as_ref()
        .unwrap();
    assert_ne!(first.helper, second.helper);
    assert_ne!(first.handler, second.handler);
    let output = generate(&flat).unwrap();
    assert_eq!(
        output.kotlin["test/Functions.kt"]
            .matches(" fun overloaded(")
            .count(),
        2
    );
}
#[test]
fn nested_error_layouts_must_match_even_with_the_same_outer_byte_array() {
    use crate::operations::{JniOutput, JniOutputLeafOp, JniOutputUnary};

    let source = syn::parse_file("pub fn failed()->Result<i64,Vec<i64>>{}").unwrap();
    let mut flat = JniGen::builder()
        .items(
            source
                .items
                .into_iter()
                .map(|item| (item, Default::default())),
        )
        .set_package_prefix("test")
        .package(package!().fun(fun!(failed)))
        .prepare()
        .unwrap();
    let mut catalogue = JniBindings::parse([]).unwrap();
    catalogue.metadata.bulk_collection_outputs = true;
    let output = prebindgen_declarations::compile_output_value(
        &catalogue,
        &JniCodec::raw(),
        &syn::parse_quote!(Vec<i32>),
        &JniOutput::Unary {
            operation: (JniOutputUnary::Sequence {
                delivery: prebindgen_declarations::SequenceDelivery::Value,
                mode: None,
            })
            .into(),
            child: Box::new(JniOutput::Leaf(JniOutputLeafOp::Value)),
        }
        .bulk(),
        &(),
    )
    .unwrap();
    let old = flat
        .function("failed")
        .unwrap()
        .ret
        .slot
        .as_ref()
        .unwrap()
        .output_outcomes()
        .source_error
        .as_ref()
        .unwrap();
    let top = |node: &kotlin::JniNode| {
        group::result_packet_fields(node)
            .map(|wire| wire.kind.jvm_descriptor())
            .collect::<Vec<_>>()
    };
    assert_eq!(top(old), top(&output.output_outcomes().success));
    bind_error_output(&mut flat, "failed", output).unwrap();
    assert!(generate(&flat)
        .err()
        .unwrap()
        .contains("domain-error converter differs"));
}

#[test]
fn explicit_plan_delivery_edits_are_preserved_with_fixed_policy() {
    use prebindgen_declarations::Codec;
    let mut flat = prepared();
    let mut plan = JniCodec::raw()
        .output_leaf(prebindgen_declarations::OutputLeaf::new(
            &flat,
            &syn::parse_quote!(i64),
            &(),
            syn::parse_quote!(custom_wire),
            prebindgen_conversions::source_result_expr(),
            prebindgen_declarations::ValueAccess::Owned,
        ))
        .unwrap();
    plan.outcomes.binding_failure = JniCodec::raw().binding_failure_layout(&()).unwrap();
    JniCodec::raw().finish_output(&mut plan, &()).unwrap();
    plan.delivery
        .statements
        .extend(quote!(let __pb_plan_delivery_hook=();));
    plan.delivery.policy = DeliveryPolicy::Fixed;
    flat.function_mut("scalar").unwrap().ret.slot = Some(plan.into());
    let output = generate(&flat).unwrap();
    assert!(output.rust.contains("__pb_plan_delivery_hook"));
    assert!(output.kotlin["test/JNINative.kt"].contains("): Array<Any?>"));
}
