//! Independent input/output entry-point controls for the completed contracts.
use prebindgen_conversions::{InputFragment, OutputFragment};
use prebindgen_declarations::{Codec, Condition};
use prebindgen_flat::{Conversion, InputConversion, OutputConversion, Result, Schema};
use prebindgen_jni::{
    fun, package, JniBranch, JniChain, JniContext, JniGen, JniInputConverter, JniOutputConverter,
    JniSchema, JniWire, KotlinContext, KotlinInput, KotlinInputConversion, KotlinOutput,
    KotlinOutputConversion,
};
use proc_macro2::TokenStream;
use quote::quote;

// Forward exactly the existing behavior: rejection cannot be attributed to a
// different wire representation, native protocol or Kotlin implementation.
struct IndependentInput(JniInputConverter);
impl Conversion for IndependentInput {
    type Wire = JniWire;
    type Chain = JniChain;
    type Branch = JniBranch;
    type SlotFragment<'a> = InputFragment<'a, JniContext>;

    fn code_fragment_slot(
        &self,
        mode: Self::SlotFragment<'_>,
        expected: &syn::Type,
    ) -> Result<TokenStream> {
        self.0.code_fragment_slot(mode, expected)
    }
}
impl InputConversion for IndependentInput {
    fn input_node(&self) -> &prebindgen_flat::WireNode<JniWire, JniChain, JniBranch> {
        self.0.input_node()
    }
}
impl KotlinInputConversion for IndependentInput {
    fn kotlin(&self, context: &KotlinContext<'_>) -> Result<KotlinInput> {
        self.0.kotlin(context)
    }
}

struct IndependentOutput(JniOutputConverter);
impl Conversion for IndependentOutput {
    type Wire = JniWire;
    type Chain = JniChain;
    type Branch = JniBranch;
    type SlotFragment<'a> = OutputFragment<'a, JniContext>;

    fn code_fragment_slot(
        &self,
        mode: Self::SlotFragment<'_>,
        expected: &syn::Type,
    ) -> Result<TokenStream> {
        self.0.code_fragment_slot(mode, expected)
    }
}
impl OutputConversion for IndependentOutput {
    fn output_outcomes(
        &self,
    ) -> &prebindgen_flat::Outcomes<prebindgen_flat::WireNode<JniWire, JniChain, JniBranch>> {
        self.0.output_outcomes()
    }
}
impl KotlinOutputConversion for IndependentOutput {
    fn kotlin(&self, context: &KotlinContext<'_>) -> Result<KotlinOutput> {
        self.0.kotlin(context)
    }
}

// Keep all adapter metadata identical; vary only the occurrence converter types.
struct IndependentSchema<I, O>(std::marker::PhantomData<(I, O)>);
impl<I, O> Schema for IndependentSchema<I, O> {
    type Input = I;
    type Output = O;
    type Module = <JniSchema as Schema>::Module;
    type Function = <JniSchema as Schema>::Function;
    type Type = <JniSchema as Schema>::Type;
    type Field = <JniSchema as Schema>::Field;
    type Variant = <JniSchema as Schema>::Variant;
}

#[cfg(feature = "independent-input")]
fn independent_input(flat: &prebindgen_flat::Flat<JniSchema<IndependentInput>>) -> Result<JniGen> {
    prebindgen_jni::generate(flat)
}

#[cfg(feature = "independent-output")]
fn independent_output(
    flat: &prebindgen_flat::Flat<JniSchema<JniInputConverter, IndependentOutput>>,
) -> Result<JniGen> {
    prebindgen_jni::generate(flat)
}

fn main() -> Result<()> {
    #[cfg(feature = "independent-input")]
    let _ = independent_input
        as fn(&prebindgen_flat::Flat<JniSchema<IndependentInput>>) -> Result<JniGen>;
    #[cfg(feature = "independent-output")]
    let _ = independent_output
        as fn(
            &prebindgen_flat::Flat<JniSchema<JniInputConverter, IndependentOutput>>,
        ) -> Result<JniGen>;
    fn input_contract<T: InputConversion + KotlinInputConversion>() {}
    fn output_contract<T: OutputConversion + KotlinOutputConversion>() {}
    input_contract::<IndependentInput>();
    output_contract::<IndependentOutput>();
    // The custom-schema model itself is supported by Flat.
    let _: prebindgen_flat::Flat<IndependentSchema<IndependentInput, IndependentOutput>> =
        prebindgen_flat::Flat::parse([])?;
    println!("independent input/output satisfy the public contracts; Flat accepts their schema");

    let source =
        syn::parse_file("pub fn echo(x: i64) -> i64 { x }").map_err(|error| error.to_string())?;
    let flat = JniGen::builder()
        .items(
            source
                .items
                .into_iter()
                .map(|item| (item, Default::default())),
        )
        .set_package_prefix("probe")
        .configure(|flat| {
            let ty = flat.function("echo")?.params[0].ty.ty.clone();
            let mut plan = prebindgen_jni::JniCodec::raw().input_leaf(
                prebindgen_declarations::InputLeaf::new(
                    flat,
                    &ty,
                    &(),
                    syn::parse_quote!(custom_wire),
                    syn::parse_quote!(custom_value),
                    &Condition::default(),
                ),
            )?;
            plan.storage
                .extend(quote!(let __pb_contract_probe_marker = ();));
            flat.function_mut("echo")?.params[0].ty.slot = Some(plan.into());
            Ok(())
        })
        .package(package!().fun(fun!(echo)))
        .prepare()?;
    let generated = prebindgen_jni::generate(&flat)?;
    assert!(generated.rust.contains("__pb_contract_probe_marker"));
    syn::parse_file(&generated.rust).map_err(|error| error.to_string())?;
    assert!(generated
        .kotlin
        .values()
        .any(|file| file.contains("fun echo(")));
    println!("customized built-in input plan generates Rust and Kotlin; Rust syntax parses");
    Ok(())
}
