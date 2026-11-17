//! An independently implemented input, without a JniInputConverter or InputPlan.
use prebindgen_conversions::InputFragment;
use prebindgen_conversions::{Conversion, InputConversion, Wire, WireNode};
use prebindgen_flat::{Flat, Primitive, Result};
use prebindgen_jni::{
    JniBindings, JniContext, JniFunction, JniGen, JniInputWireBranch, JniInputWireChain, JniSchema,
    JniWire, KotlinContext, KotlinInput, KotlinInputConversion,
};
use proc_macro2::TokenStream;
use quote::quote;

struct DurationInput(
    WireNode<
        JniWire,
        JniInputWireChain,
        JniInputWireBranch,
        prebindgen_jni::JniInputWireAnnotation,
        prebindgen_jni::JniInputWireContainer,
    >,
);

impl Conversion for DurationInput {
    type Wire = JniWire;
    type Chain = JniInputWireChain;
    type Branch = JniInputWireBranch;
    type Annotation = prebindgen_jni::JniInputWireAnnotation;
    type Container = prebindgen_jni::JniInputWireContainer;
    type SlotFragment<'a> = InputFragment<'a, JniContext>;

    fn code_fragment_slot(
        &self,
        request: Self::SlotFragment<'_>,
        _: &syn::Type,
    ) -> Result<TokenStream> {
        let InputFragment { target, next, .. } = request;
        Ok(quote!({let #target=millis_wire;#next}))
    }
}

impl InputConversion for DurationInput {
    fn input_node(
        &self,
    ) -> &WireNode<
        JniWire,
        JniInputWireChain,
        JniInputWireBranch,
        prebindgen_jni::JniInputWireAnnotation,
        prebindgen_jni::JniInputWireContainer,
    > {
        &self.0
    }
}

impl KotlinInputConversion for DurationInput {
    fn kotlin(&self, _: &KotlinContext<'_>) -> Result<KotlinInput> {
        Ok(KotlinInput {
            parameters: vec!["millis_wire: java.time.Duration".into()],
            arguments: vec!["millis_wire.toMillis()".into()],
            ..Default::default()
        })
    }
}

pub fn generate() -> Result<JniGen> {
    let items = syn::parse_file("pub fn duration_millis(milliseconds: i64) -> i64 {}")
        .map_err(|error| error.to_string())?
        .items;
    let mut flat = Flat::<JniSchema<DurationInput>>::parse(
        items.into_iter().map(|item| (item, Default::default())),
    )?;
    flat.metadata.package = "io.prebindgen.covertest.inputcontracts".into();
    flat.metadata.native_init = "io.prebindgen.covertest.NativeLibrary.ensureLoaded()".into();
    let placement = JniFunction {
        package: flat.metadata.package.clone(),
        name: "durationMillis".into(),
        ..Default::default()
    };
    // Output conversion remains the standard implementation in this stage.
    let output = prebindgen_declarations::compile_output_value(
        &JniBindings::parse([])?,
        &prebindgen_jni::JniCodec::raw(),
        &syn::parse_quote!(i64),
        &prebindgen_jni::operations::JniOutput::Leaf(
            prebindgen_jni::operations::JniOutputLeafOp::Value,
        ),
        &(),
    )?;
    let function = flat.function_mut("duration_millis")?;
    function.path = syn::parse_quote!(crate::input_contracts::duration_millis);
    function.metadata.push(prebindgen_jni::JniBinding {
        name: syn::parse_quote!(duration_millis),
        metadata: placement,
        state: prebindgen_conversions::BindingState::Prepared(
            prebindgen_conversions::PreparedCall {
                inputs: vec![DurationInput(WireNode::Leaf(vec![Wire {
                    name: syn::parse_quote!(millis_wire),
                    kind: JniWire::scalar(Primitive::I64),
                }]))],
                output,
            },
        ),
    });
    prebindgen_jni::generate(&flat)
}
