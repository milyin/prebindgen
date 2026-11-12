use super::*;
use prebindgen_conversions::InputFragment;
use prebindgen_declarations::{frontend::InputLeafOp, frontend::InputUnary};
use prebindgen_flat::{Function, InputConversion, Param, TypeUse};
use std::{cell::Cell, rc::Rc};

// Independent storage also records how often the emitter prepares Kotlin input.
// Forwarding native behavior makes complete output equality a useful control.
#[derive(Clone)]
struct ExternalInput {
    native: JniInput,
    preparations: Rc<Cell<usize>>,
    omit_handles: bool,
    omit_consumption: bool,
}
impl Conversion for ExternalInput {
    type Wire = JniWire;
    type Chain = JniChain;
    type Branch = JniBranch;
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
    fn input_node(&self) -> &prebindgen_flat::WireNode<JniWire, JniChain, JniBranch> {
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

fn external(flat: Bindings) -> Flat<JniSchema<ExternalInput>> {
    // Test fixture: preserve every declaration and metadata value while changing
    // just the input implementation. Production preparation chooses its schema
    // before filling slots; this does not add a schema-changing API.
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
                let params = f.params.into_iter().map(|p| Param {
                    name: p.name,
                    ty: TypeUse {
                        ty: p.ty.ty,
                        slot: p.ty.slot.map(|native| ExternalInput {
                            native,
                            preparations: Rc::default(),
                            omit_handles: false,
                            omit_consumption: false,
                        }),
                    },
                });
                (
                    key,
                    Function {
                        metadata: f.metadata,
                        name: f.name,
                        path: f.path,
                        location: f.location,
                        ret: f.ret,
                        params: params.collect(),
                    },
                )
            })
            .collect(),
    }
}

fn handles() -> Bindings {
    JniGen::builder()
        .items(syn::parse_file("pub type Handle=String;pub fn mixed(shared:&Handle,maybe:Option<&Handle>,owned:Handle,rows:Vec<Handle>)->i64{}").unwrap().items.into_iter().map(|item|(item,Default::default())))
        .set_package_prefix("test")
        .package(package!().class(ptr_class!(Handle)).fun(fun!(mixed).expand_param("rows", DeclarationInput::Unary { operation: InputUnary::Sequence, child: Box::new(DeclarationInput::Leaf(InputLeafOp::Value)) }.bulk())))
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
    let input = flat.function("mixed").unwrap().params[3]
        .ty
        .slot
        .as_ref()
        .unwrap();
    let context = JniContext {
        input_transfers: prebindgen_conversions::InputTransfers::Report(syn::parse_quote!(
            __pb_input_consumed
        )),
        return_delivery: JniReturnDelivery::Packet,
        prepared_delivery: None,
        environment: syn::parse_quote!(env),
        domain_output: syn::parse_quote!(domain),
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
    flat.function_mut("mixed").unwrap().params[2]
        .ty
        .slot
        .as_mut()
        .unwrap()
        .omit_handles = true;
    let error = generate(&flat).err().unwrap();
    assert!(error.contains("complete prepared handle inputs"), "{error}");

    let mut flat = external(handles());
    // Remove every consuming update, leaving the native acknowledgement active.
    for parameter in &mut flat.function_mut("mixed").unwrap().params {
        parameter.ty.slot.as_mut().unwrap().omit_consumption = true;
    }
    let error = generate(&flat).err().unwrap();
    assert!(error.contains("acknowledgement does not match"), "{error}");
}
