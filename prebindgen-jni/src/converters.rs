//! Retain heterogeneous converters in callback/error metadata, only at generation time.
use super::*;
use prebindgen_conversions::{Delivery, DeliveryPolicy, InputFragment, OutputFragment};
use prebindgen_flat::{InputConversion, Outcomes, OutputConversion};
use std::rc::Rc;

// Conversion's associated request family is intentionally not object safe.
// These private adapters erase only the already-fixed JNI request family.
trait InputProvider {
    fn node(&self) -> &kotlin::JniNode;
    fn fragment(
        &self,
        request: InputFragment<'_, JniContext>,
        expected: &syn::Type,
    ) -> Result<TokenStream>;
    fn callback_return(
        &self,
        request: InputFragment<'_, JniContext>,
        expected: &syn::Type,
        cached_buffers: bool,
    ) -> Result<TokenStream>;
    fn kotlin(&self, context: &KotlinContext<'_>) -> Result<KotlinInput>;
}
impl<T: KotlinInputConversion> InputProvider for T {
    fn node(&self) -> &kotlin::JniNode {
        self.input_node()
    }
    fn fragment(
        &self,
        request: InputFragment<'_, JniContext>,
        expected: &syn::Type,
    ) -> Result<TokenStream> {
        self.code_fragment_slot(request, expected)
    }
    fn callback_return(
        &self,
        request: InputFragment<'_, JniContext>,
        expected: &syn::Type,
        cached_buffers: bool,
    ) -> Result<TokenStream> {
        self.callback_return_fragment(request, expected, cached_buffers)
    }
    fn kotlin(&self, context: &KotlinContext<'_>) -> Result<KotlinInput> {
        KotlinInputConversion::kotlin(self, context)
    }
}

/// An owned prepared input converter, usable in default bindings and callbacks.
/// Cloning shares immutable generator data; no wrapper allocation is emitted.
#[derive(Clone)]
pub struct JniInputConverter(Rc<dyn InputProvider>);
impl JniInputConverter {
    /// Retain an independent converter through the same contracts as built-in inputs.
    pub fn new(input: impl KotlinInputConversion + 'static) -> Self {
        Self(Rc::new(input))
    }
}
impl std::fmt::Debug for JniInputConverter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("JniInputConverter")
            .field("node", self.input_node())
            .finish()
    }
}
impl From<JniInputPlan<JniWire>> for JniInputConverter {
    fn from(rust: JniInputPlan<JniWire>) -> Self {
        Self::new(kotlin::PlanInput { rust })
    }
}
impl Conversion for JniInputConverter {
    type Wire = JniWire;
    type Chain = JniChain;
    type Branch = JniBranch;
    type SlotFragment<'a> = InputFragment<'a, JniContext>;
    fn code_fragment_slot(
        &self,
        request: Self::SlotFragment<'_>,
        expected: &syn::Type,
    ) -> Result<TokenStream> {
        self.0.fragment(request, expected)
    }
}
impl InputConversion for JniInputConverter {
    fn input_node(&self) -> &kotlin::JniNode {
        self.0.node()
    }
}
impl KotlinInputConversion for JniInputConverter {
    fn kotlin(&self, context: &KotlinContext<'_>) -> Result<KotlinInput> {
        self.0.kotlin(context)
    }
    fn callback_return_fragment(
        &self,
        request: InputFragment<'_, JniContext>,
        expected: &syn::Type,
        cached_buffers: bool,
    ) -> Result<TokenStream> {
        self.0.callback_return(request, expected, cached_buffers)
    }
}

trait OutputProvider {
    fn outcomes(&self) -> &Outcomes<kotlin::JniNode>;
    fn fragment(
        &self,
        request: OutputFragment<'_, JniContext>,
        expected: &syn::Type,
    ) -> Result<TokenStream>;
    fn kotlin(&self, context: &KotlinContext<'_>) -> Result<KotlinOutput>;
    fn policy(&self) -> DeliveryPolicy;
    fn prepare_return(&self, context: &JniContext) -> Result<Option<Delivery>>;
    fn callback_arguments(
        &self,
        context: &JniContext,
        source: &syn::Ident,
    ) -> Result<Vec<(syn::Ident, TokenStream)>>;
    fn sequence_element(
        &self,
        context: &KotlinContext<'_>,
        owners: Option<&str>,
    ) -> Result<KotlinOutput>;
    fn kotlin_return(&self, context: &KotlinReturnContext<'_>) -> Result<KotlinOutput>;
}
impl<T: KotlinOutputConversion> OutputProvider for T {
    fn outcomes(&self) -> &Outcomes<kotlin::JniNode> {
        self.output_outcomes()
    }
    fn fragment(
        &self,
        request: OutputFragment<'_, JniContext>,
        expected: &syn::Type,
    ) -> Result<TokenStream> {
        self.code_fragment_slot(request, expected)
    }
    fn kotlin(&self, context: &KotlinContext<'_>) -> Result<KotlinOutput> {
        KotlinOutputConversion::kotlin(self, context)
    }
    fn policy(&self) -> DeliveryPolicy {
        self.delivery_policy()
    }
    fn prepare_return(&self, context: &JniContext) -> Result<Option<Delivery>> {
        self.prepare_return_delivery(context)
    }
    fn callback_arguments(
        &self,
        context: &JniContext,
        source: &syn::Ident,
    ) -> Result<Vec<(syn::Ident, TokenStream)>> {
        KotlinOutputConversion::callback_arguments(self, context, source)
    }
    fn sequence_element(
        &self,
        context: &KotlinContext<'_>,
        owners: Option<&str>,
    ) -> Result<KotlinOutput> {
        self.kotlin_sequence_element(context, owners)
    }
    fn kotlin_return(&self, context: &KotlinReturnContext<'_>) -> Result<KotlinOutput> {
        KotlinOutputConversion::kotlin_return(self, context)
    }
}

/// An owned prepared output converter, usable in default bindings and callbacks.
#[derive(Clone)]
pub struct JniOutputConverter {
    provider: Rc<dyn OutputProvider>,
    packet_only: bool,
}
impl JniOutputConverter {
    /// Retain an independent converter, including its delivery and Kotlin projections.
    pub fn new(output: impl KotlinOutputConversion + 'static) -> Self {
        Self {
            provider: Rc::new(output),
            packet_only: false,
        }
    }
    /// Preserve packet delivery even when this converter supports direct/capture delivery.
    pub fn packet_only(mut self) -> Self {
        self.packet_only = true;
        self
    }
    /// Successful packet fields in native order; excludes status/error framing.
    pub fn result_packet_fields(
        &self,
    ) -> Result<std::vec::IntoIter<&prebindgen_flat::Wire<JniWire>>> {
        group::validate_layout(&self.output_outcomes().success)?;
        Ok(group::result_packet_fields(&self.output_outcomes().success))
    }
}
impl std::fmt::Debug for JniOutputConverter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("JniOutputConverter")
            .field("outcomes", self.output_outcomes())
            .field("packet_only", &self.packet_only)
            .finish()
    }
}
impl From<JniOutputPlan<JniWire>> for JniOutputConverter {
    fn from(rust: JniOutputPlan<JniWire>) -> Self {
        Self::new(kotlin::PlanOutput { rust })
    }
}
impl Conversion for JniOutputConverter {
    type Wire = JniWire;
    type Chain = JniChain;
    type Branch = JniBranch;
    type SlotFragment<'a> = OutputFragment<'a, JniContext>;
    fn code_fragment_slot(
        &self,
        request: Self::SlotFragment<'_>,
        expected: &syn::Type,
    ) -> Result<TokenStream> {
        self.provider.fragment(request, expected)
    }
}
impl OutputConversion for JniOutputConverter {
    fn output_outcomes(&self) -> &Outcomes<kotlin::JniNode> {
        self.provider.outcomes()
    }
}
impl KotlinOutputConversion for JniOutputConverter {
    fn kotlin(&self, context: &KotlinContext<'_>) -> Result<KotlinOutput> {
        self.provider.kotlin(context)
    }
    fn delivery_policy(&self) -> DeliveryPolicy {
        if self.packet_only {
            DeliveryPolicy::Fixed
        } else {
            self.provider.policy()
        }
    }
    fn prepare_return_delivery(&self, context: &JniContext) -> Result<Option<Delivery>> {
        if self.packet_only {
            Ok(None)
        } else {
            self.provider.prepare_return(context)
        }
    }
    fn callback_arguments(
        &self,
        context: &JniContext,
        source: &syn::Ident,
    ) -> Result<Vec<(syn::Ident, TokenStream)>> {
        self.provider.callback_arguments(context, source)
    }
    fn kotlin_sequence_element(
        &self,
        context: &KotlinContext<'_>,
        owners: Option<&str>,
    ) -> Result<KotlinOutput> {
        self.provider.sequence_element(context, owners)
    }
    fn kotlin_return(&self, context: &KotlinReturnContext<'_>) -> Result<KotlinOutput> {
        self.provider.kotlin_return(context)
    }
}
