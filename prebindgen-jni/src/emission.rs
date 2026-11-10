//! Bindings supplied by this language generator during slot emission.
use prebindgen_conversions::{render_template, TemplateContext};
use prebindgen_flat::Result;
use proc_macro2::TokenStream;
use quote::ToTokens;

/// Concrete wrapper bindings used by this language generator.
#[derive(Clone, Debug)]
pub struct JniContext {
    /// Explicit function-return transport. Source callback argument scopes use Packet.
    pub return_delivery: crate::JniReturnDelivery,
    /// Delivery prepared once for the active result converter, if adapter-selected.
    pub prepared_delivery: Option<prebindgen_conversions::Delivery>,
    /// Mutable JNI environment parameter.
    pub environment: syn::Ident,
    /// Long-array parameter recording completed input transfers.
    pub acknowledgement: syn::Ident,
    /// Local retaining a committed domain-error packet or delivery marker across Rust unwinding.
    pub domain_output: syn::Ident,
    /// Borrowed synchronous function-return capture. None preserves custom
    /// packet delivery and source-callback argument conversion.
    pub return_sink: Option<syn::Ident>,
    /// Whether this delivery belongs to the typed domain-error channel.
    pub is_domain_error: bool,
    /// Infallible statements recording completed output ownership transfer.
    pub output_committed: TokenStream,
}
impl TemplateContext for JniContext {
    fn render(&self, code: &TokenStream) -> Result<TokenStream> {
        // Generated Kotlin validates aliases while acquiring handle locks.
        render_template(code, &|name| match name {
            "input_consumed" => Ok(quote::quote!(&__pb_input_consumed)),
            "environment" => Ok(self.environment.to_token_stream()),
            "acknowledgement" => Ok(self.acknowledgement.to_token_stream()),
            "domain_output" => Ok(self.domain_output.to_token_stream()),
            "return_sink" => Ok(self
                .return_sink
                .as_ref()
                .map_or_else(|| quote::quote!(None), |sink| quote::quote!(Some(&#sink)))),
            "mark_output" => {
                let env = &self.environment;
                Ok(if self.is_domain_error {
                    quote::quote!(::prebindgen_jni_runtime::mark_domain_packet(&mut #env, &result)?;)
                } else {
                    TokenStream::new()
                })
            }
            "output_committed" => Ok(self.output_committed.clone()),
            name if name.starts_with("bulk_read_") => {
                Ok(quote::quote!(::prebindgen_jni_runtime::bulk::input))
            }
            _ => Err(format!("unknown emission parameter `{name}`")),
        })
    }
}

impl JniContext {
    /// Use the shared callback boundary's prepared bulk carriers in native input
    /// validation. The code uses the standard bulk-read template markers and
    /// the same wire tree as the callback's input converter.
    pub fn cached_callback_validation(
        &self,
        validation: &TokenStream,
        node: &prebindgen_flat::WireNode<crate::JniWire, crate::JniChain, crate::JniBranch>,
    ) -> Result<TokenStream> {
        crate::bulk::cached_reads(validation, node)
    }
    /// Prepare native result delivery from the successful wire expressions.
    /// This is the same operation used by built-in output plans. Values may
    /// refer to source_result_expr; the output fragment resolves that binding.
    pub fn output_delivery(
        &self,
        outcomes: &prebindgen_flat::Outcomes<
            prebindgen_flat::WireNode<crate::JniWire, crate::JniChain, crate::JniBranch>,
        >,
        values: &[(syn::Ident, TokenStream)],
    ) -> Result<prebindgen_conversions::Delivery> {
        let node = &outcomes.success;
        match self.return_delivery {
            crate::JniReturnDelivery::Packet => crate::returns::packet_delivery(node, values),
            crate::JniReturnDelivery::Capture => crate::returns::callback_delivery(
                node,
                values,
                self.return_sink
                    .as_ref()
                    .ok_or("capture delivery requires a return sink")?,
                self.is_domain_error,
            ),
            crate::JniReturnDelivery::Direct => {
                if self.is_domain_error {
                    return Err("domain errors require capture or packet delivery".into());
                }
                if crate::group::result_packet_fields(node).len() > 1 {
                    return Err("direct delivery requires at most one wire".into());
                }
                crate::returns::delivery(
                    node,
                    values,
                    self.return_sink
                        .as_ref()
                        .ok_or("direct delivery requires an error capture")?,
                    false,
                )
            }
        }
    }
}
