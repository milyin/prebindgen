//! Custom projections through ordinary returns, retained callbacks and typed errors.
use prebindgen_conversions::{Delivery, DeliveryPolicy, InputFragment, OutputFragment};
use prebindgen_flat::{Conversion, InputConversion, OutputConversion, Result, WireNode};
use prebindgen_jni::{
    expand_return, fun, package, ptr_class, Bindings, JniBranch, JniCallback, JniChain, JniContext,
    JniGen, JniInput, JniOutput, JniWire, KotlinContext, KotlinInput, KotlinInputConversion,
    KotlinOutput, KotlinOutputConversion,
};
use proc_macro2::TokenStream;

// Reuse the native wire conversion while independently defining the Kotlin API.
struct DurationOutput(JniOutput);
impl Conversion for DurationOutput {
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
impl OutputConversion for DurationOutput {
    fn output_outcomes(
        &self,
    ) -> &prebindgen_flat::Outcomes<WireNode<JniWire, JniChain, JniBranch>> {
        self.0.output_outcomes()
    }
}
impl DurationOutput {
    fn project(mut output: KotlinOutput) -> Result<KotlinOutput> {
        for (expression, ty) in output.delivered.iter_mut().zip(&mut output.result_types) {
            *expression = match ty.as_str() {
                "Long" => format!("java.time.Duration.ofMillis({expression})"),
                "kotlin.collections.List<Long>" => {
                    format!("({expression}).map {{ java.time.Duration.ofMillis(it) }}")
                }
                "Long?" => format!("({expression})?.let {{ java.time.Duration.ofMillis(it) }}"),
                other => return Err(format!("Duration output requires Long, got {other}")),
            };
            *ty = if ty == "kotlin.collections.List<Long>" {
                "kotlin.collections.List<java.time.Duration>"
            } else if ty.ends_with('?') {
                "java.time.Duration?"
            } else {
                "java.time.Duration"
            }
            .into();
        }
        Ok(output)
    }
}
impl KotlinOutputConversion for DurationOutput {
    fn kotlin(&self, context: &KotlinContext<'_>) -> Result<KotlinOutput> {
        Self::project(self.0.kotlin(context)?)
    }
    fn kotlin_sequence_element(
        &self,
        context: &KotlinContext<'_>,
        owners: Option<&str>,
    ) -> Result<KotlinOutput> {
        Self::project(self.0.kotlin_sequence_element(context, owners)?)
    }
    fn delivery_policy(&self) -> DeliveryPolicy {
        self.0.delivery_policy()
    }
    fn prepare_return_delivery(&self, context: &JniContext) -> Result<Option<Delivery>> {
        self.0.prepare_return_delivery(context)
    }
    fn callback_arguments(
        &self,
        context: &JniContext,
        source: &syn::Ident,
    ) -> Result<Vec<(syn::Ident, TokenStream)>> {
        self.0.callback_arguments(context, source)
    }
}

struct DurationInput(JniInput);
impl Conversion for DurationInput {
    type Wire = JniWire;
    type Chain = JniChain;
    type Branch = JniBranch;
    type SlotFragment<'a> = InputFragment<'a, JniContext>;
    fn code_fragment_slot(
        &self,
        request: Self::SlotFragment<'_>,
        expected: &syn::Type,
    ) -> Result<TokenStream> {
        self.0.code_fragment_slot(request, expected)
    }
}
impl InputConversion for DurationInput {
    fn input_node(&self) -> &WireNode<JniWire, JniChain, JniBranch> {
        self.0.input_node()
    }
}
impl KotlinInputConversion for DurationInput {
    fn kotlin(&self, context: &KotlinContext<'_>) -> Result<KotlinInput> {
        let mut input = self.0.kotlin(context)?;
        for parameter in &mut input.parameters {
            let (name, ty) = parameter.rsplit_once(": ").ok_or("invalid parameter")?;
            if ty != "Long" {
                return Err("Duration input requires Long".into());
            }
            *parameter = format!("{name}: java.time.Duration");
        }
        for value in input
            .arguments
            .iter_mut()
            .chain(&mut input.prepared_arguments)
        {
            *value = format!("({value}).toMillis()");
        }
        Ok(input)
    }
    fn callback_return_fragment(
        &self,
        request: InputFragment<'_, JniContext>,
        expected: &syn::Type,
        cached_buffers: bool,
    ) -> Result<TokenStream> {
        self.0
            .callback_return_fragment(request, expected, cached_buffers)
    }
}

pub fn generate() -> Result<JniGen> {
    let items = syn::parse_file("pub struct DurationPair{pub a:i64,pub b:i64} pub fn duration_echo(milliseconds:i64)->i64{} pub fn duration_packet(milliseconds:i64)->i64{} pub fn duration_series()->Vec<i64>{} pub fn duration_pair()->DurationPair{} pub fn pair_a(pair:&DurationPair)->i64{} pub fn pair_b(pair:&DurationPair)->i64{} pub fn duration_optional(present:bool)->Option<i64>{} pub fn duration_result(milliseconds:i64)->Result<i64,i64>{} pub fn plain_result(milliseconds:i64)->Result<i64,i64>{} pub fn duration_callback(callback:impl Fn(i64)->i64)->i64{} pub fn plain_callback(callback:impl Fn(i64)->i64)->i64{}")
        .map_err(|error| error.to_string())?.items;
    let mut flat = JniGen::builder()
        .items(items.into_iter().map(|item| {
            (
                item,
                prebindgen::SourceLocation {
                    crate_name: Some("crate::output_contracts".into()),
                    ..Default::default()
                },
            )
        }))
        .set_package_prefix("io.prebindgen.covertest.outputcontracts")
        .set_jni_native_init("io.prebindgen.covertest.NativeLibrary.ensureLoaded()")
        .package(
            package!()
                .class(ptr_class!(DurationPair))
                .fun(fun!(duration_echo))
                .fun(fun!(duration_packet))
                .fun(
                    fun!(duration_pair).expand_return(
                        expand_return!(DurationPair)
                            .field(fun!(pair_a))
                            .field(fun!(pair_b)),
                    ),
                )
                .fun(fun!(duration_series).callback_return())
                .fun(fun!(duration_optional))
                .fun(fun!(duration_result))
                .fun(fun!(plain_result))
                .fun(fun!(duration_callback))
                .fun(fun!(plain_callback)),
        )
        .prepare()?;
    for name in [
        "duration_echo",
        "duration_packet",
        "duration_pair",
        "duration_series",
        "duration_series_fold",
        "duration_optional",
        "duration_result",
    ] {
        let function = flat.function_mut(name)?;
        let mut output = JniOutput::new(DurationOutput(
            function.ret.slot.take().ok_or("missing output")?,
        ));
        if name == "duration_packet" {
            output = output.packet_only();
        }
        function.ret.slot = Some(output);
    }
    // A local error override must not change plain_result's Long error handler.
    let error_output = prebindgen_declarations::compile_output_value(
        &Bindings::parse([])?,
        &prebindgen_jni::JniCodec::raw(),
        &syn::parse_quote!(i64),
        &prebindgen_jni::operations::Output::Leaf(prebindgen_jni::operations::OutputLeafOp::Value),
        &(),
    )?;
    prebindgen_jni::bind_error_output(&mut flat, "duration_result", DurationOutput(error_output))?;

    let original = flat.function("duration_callback")?.params[0]
        .ty
        .slot
        .as_ref()
        .ok_or("missing callback")?;
    let node = original.input_node();
    let callback = prebindgen_flat::wire_definitions(std::iter::once(node))
        .find_map(|wire| {
            if let JniWire::Callback { callback } = &wire.kind {
                Some(callback.clone())
            } else {
                None
            }
        })
        .ok_or("missing callback description")?;
    let mut callback: JniCallback = (*callback).clone();
    callback.set_interface_name("DurationCallback")?;
    let native_output = prebindgen_declarations::compile_output_value(
        &Bindings::parse([])?,
        &prebindgen_jni::JniCodec::raw(),
        &syn::parse_quote!(i64),
        &prebindgen_jni::operations::Output::Leaf(prebindgen_jni::operations::OutputLeafOp::Value),
        &(),
    )?;
    callback.set_argument_converter(0, DurationOutput(native_output))?;
    // Prepare an ordinary scalar input through the public codec, then give it
    // the independent Kotlin projection needed by the callback return.
    use prebindgen_declarations::Codec;
    let input =
        prebindgen_jni::JniCodec::raw().input_leaf(prebindgen_declarations::InputLeaf::new(
            &flat,
            &syn::parse_quote!(i64),
            &(),
            syn::parse_quote!(callback_duration_wire),
            syn::parse_quote!(callback_duration_value),
            &prebindgen_declarations::Condition::default(),
        ))?;
    callback.set_return_converter(DurationInput(JniInput::from(input)))?;
    flat.function_mut("duration_callback")?.params[0].ty.slot = Some(callback.into_input(
        syn::parse_quote!(custom_callback),
        syn::parse_quote!(callback_value),
    )?);
    // Keep declarations in the one Flat model; there is no side table of overrides.
    prebindgen_jni::generate(&flat)
}
