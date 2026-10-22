//! Bindings supplied by this language generator during slot emission.
use prebindgen_conversions::{render_template, TemplateContext};
use prebindgen_flat::Result;
use proc_macro2::TokenStream;
use quote::ToTokens;

/// Concrete wrapper bindings used by this language generator.
#[derive(Clone, Debug)]
pub struct JniContext {
    /// Mutable JNI environment parameter.
    pub environment: syn::Ident,
    /// Boolean-array parameter acknowledging ownership transfer.
    pub acknowledgement: syn::Ident,
    /// Local retaining a committed domain-error packet across Rust unwinding.
    pub domain_output: syn::Ident,
    /// Whether delivery marks the packet as a typed domain error.
    pub is_domain_error: bool,
    /// Infallible statements recording completed output ownership transfer.
    pub output_committed: TokenStream,
}
impl TemplateContext for JniContext {
    fn render(&self, code: &TokenStream) -> Result<TokenStream> {
        render_template(code, &|name| match name {
            "environment" => Ok(self.environment.to_token_stream()),
            "acknowledgement" => Ok(self.acknowledgement.to_token_stream()),
            "domain_output" => Ok(self.domain_output.to_token_stream()),
            "mark_output" => {
                let env = &self.environment;
                Ok(if self.is_domain_error {
                    quote::quote!(::prebindgen_jni_runtime::mark_domain_packet(&mut #env, &result)?;)
                } else {
                    TokenStream::new()
                })
            }
            "output_committed" => Ok(self.output_committed.clone()),
            _ => Err(format!("unknown emission parameter `{name}`")),
        })
    }
}
