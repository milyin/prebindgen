//! An independently implemented input, without a JniInput or InputPlan.
use prebindgen_conversions::InputFragment;
use prebindgen_flat::{Conversion, Flat, InputConversion, Primitive, Result, Wire, WireNode};
use prebindgen_jni::{
    Bindings, JniBranch, JniChain, JniContext, JniFunction, JniGen, JniSchema, JniWire,
    KotlinContext, KotlinInput, KotlinInputConversion,
};
use proc_macro2::TokenStream;
use quote::quote;

struct DurationInput(WireNode<JniWire, JniChain, JniBranch>);

impl Conversion for DurationInput {
    type Wire = JniWire;
    type Chain = JniChain;
    type Branch = JniBranch;
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
    fn input_node(&self) -> &WireNode<JniWire, JniChain, JniBranch> {
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
    let mut placement = JniFunction::default();
    placement.package = flat.metadata.package.clone();
    placement.name = "durationMillis".into();
    // Output conversion remains the standard implementation in this stage.
    let output = prebindgen_declarations::compile_output_value(
        &Bindings::parse([])?,
        &prebindgen_jni::JniCodec::raw(),
        &syn::parse_quote!(i64),
        &prebindgen_declarations::Output::Leaf,
        &(),
    )?;
    let function = flat.function_mut("duration_millis")?;
    function.path = syn::parse_quote!(crate::input_contracts::duration_millis);
    function.metadata = Some(placement);
    function.params[0].ty.slot = Some(DurationInput(WireNode::Leaf(vec![Wire {
        name: syn::parse_quote!(millis_wire),
        kind: JniWire::scalar(Primitive::I64),
    }])));
    function.ret.slot = Some(output);
    prebindgen_jni::generate(&flat)
}
