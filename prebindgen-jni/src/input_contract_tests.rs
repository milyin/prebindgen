use super::*;
use prebindgen_conversions::InputFragment;

use crate::{JniInputLeafOp, JniInputUnary};
use prebindgen_conversions::InputConversion;
use std::{cell::Cell, rc::Rc};

// Independent storage also records how often the emitter prepares Kotlin input.
// Forwarding native behavior makes complete output equality a useful control.
#[derive(Clone)]
struct ExternalInput {
    native: JniInputConverter,
    preparations: Rc<Cell<usize>>,
    omit_handles: bool,
    omit_consumption: bool,
}
impl Conversion for ExternalInput {
    type Wire = JniWire;
    type Chain = JniInputWireChain;
    type Branch = JniInputWireBranch;
    type SlotFragment<'a> = InputFragment<'a, JniContext>;

    fn code_fragment_slot(
        &self,
        request: Self::SlotFragment<'_>,
        expected: &syn::Type,
    ) -> Result<TokenStream> {
        self.native.code_fragment_slot(request, expected)
    }
}
impl InputConversion for ExternalInput {
    fn input_node(&self) -> &JniInputWireNode {
        self.native.input_node()
    }
}
impl KotlinInputConversion for ExternalInput {
    fn kotlin(&self, context: &KotlinContext<'_>) -> Result<KotlinInput> {
        self.preparations.set(self.preparations.get() + 1);
        let mut input = self.native.kotlin(context)?;
        if self.omit_handles {
            input.handle_inputs = None;
        }
        if self.omit_consumption {
            input.consumed.clear();
            for handle in input.handle_inputs.iter_mut().flatten() {
                handle.consumed = None;
            }
        }
        Ok(input)
    }
}

fn external(flat: JniBindings) -> JniBindings<ExternalInput> {
    flat.try_map_function_data(|_, _, bindings| {
        bindings
            .into_iter()
            .map(|binding| {
                let prebindgen_declarations::BindingState::Prepared(call) = binding.state else {
                    return Err("expected prepared export".into());
                };
                Ok(prebindgen_conversions::FunctionBinding {
                    name: binding.name,
                    metadata: binding.metadata,
                    state: prebindgen_declarations::BindingState::Prepared(
                        prebindgen_conversions::PreparedCall {
                            inputs: call
                                .inputs
                                .into_iter()
                                .map(|native| ExternalInput {
                                    native,
                                    preparations: Rc::default(),
                                    omit_handles: false,
                                    omit_consumption: false,
                                })
                                .collect(),
                            output: call.output,
                        },
                    ),
                })
            })
            .collect()
    })
    .unwrap()
}

fn handles() -> JniBindings {
    JniGen::builder()
        .items(syn::parse_file("pub type Handle=String;pub fn mixed(shared:&Handle,maybe:Option<&Handle>,owned:Handle,rows:Vec<Handle>)->i64{}").unwrap().items.into_iter().map(|item|(item,Default::default())))
        .set_package_prefix("test")
        .package(package!().class(ptr_class!(Handle)).fun(fun!(mixed).expand_param("rows", JniInputConfig::Unary { operation: JniInputUnary::Sequence, child: Box::new(JniInputConfig::Leaf(JniInputLeafOp::Value)) }.bulk())))
        .prepare().unwrap()
}

#[test]
fn external_inputs_share_the_builtin_emitter_and_prepare_kotlin_once() {
    let builtin = handles();
    let expected = generate(&builtin).unwrap();
    let flat = external(builtin);
    let actual = generate(&flat).unwrap();
    assert_eq!(actual.rust, expected.rust);
    assert_eq!(actual.kotlin, expected.kotlin);
    assert_eq!(actual.proguard_rules(), expected.proguard_rules());
    for input in flat.function("mixed").unwrap().input_conversions().unwrap() {
        assert_eq!(input.preparations.get(), 1);
    }
}

#[test]
fn callback_bulk_return_requires_cached_carrier_support() {
    let flat = external(handles());
    let input = flat.function("mixed").unwrap().input_conversions().unwrap()[3];
    let context = JniContext {
        input_transfers: prebindgen_conversions::InputTransfers::Report(syn::parse_quote!(
            __pb_input_consumed
        )),
        return_delivery: JniReturnDelivery::Packet,
        prepared_delivery: None,
        environment: syn::parse_quote!(env),
        domain_output: syn::parse_quote!(domain),
        source_error_helper: None,
        return_sink: None,
        is_domain_error: false,
        output_committed: quote!(),
    };
    let target = syn::parse_quote!(decoded);
    let request = || InputFragment {
        context: &context,
        target: &target,
        next: quote!(()),
    };
    let expected = syn::parse_quote!(Vec<Handle>);
    // An ordinary forwarding implementation must explicitly forward the cache
    // operation as well; falling back to a second JNI read is not acceptable.
    assert!(input
        .callback_return_fragment(request(), &expected, true)
        .unwrap_err()
        .contains("reuse the native buffer cache"));
    let cached = input
        .native
        .callback_return_fragment(request(), &expected, true)
        .unwrap()
        .to_string();
    assert!(cached.contains("cached_bytes"));
    assert!(!cached.contains(":: bulk :: input"));
    assert!(input
        .callback_return_fragment(request(), &expected, false)
        .is_ok());
}

#[test]
fn external_inputs_must_report_handles_and_native_ownership() {
    let mut flat = external(handles());
    flat.function_mut("mixed").unwrap().metadata[0]
        .call_mut()
        .unwrap()
        .inputs[2]
        .omit_handles = true;
    let error = generate(&flat).err().unwrap();
    assert!(error.contains("complete prepared handle inputs"), "{error}");

    let mut flat = external(handles());
    // Remove every consuming update, leaving the native acknowledgement active.
    for input in &mut flat.function_mut("mixed").unwrap().metadata[0]
        .call_mut()
        .unwrap()
        .inputs
    {
        input.omit_consumption = true;
    }
    let error = generate(&flat).err().unwrap();
    assert!(error.contains("acknowledgement does not match"), "{error}");
}

use prebindgen_conversions::FunctionConversions as _;
