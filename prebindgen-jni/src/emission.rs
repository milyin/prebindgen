//! Bindings supplied by this language generator during slot emission.
use prebindgen_flat::{render_template, EmissionContext, Result, TemplateContext};
use proc_macro2::TokenStream;
use quote::ToTokens;

/// Concrete wrapper bindings used by this language generator.
#[derive(Clone, Debug)]
pub struct JniContext {
    /// Name of the generated variable that holds the source function's return value.
    pub source_result: syn::Ident,
    /// Mutable JNI environment parameter.
    pub environment: syn::Ident,
    /// Boolean-array parameter acknowledging ownership transfer.
    pub acknowledgement: syn::Ident,
}
impl EmissionContext for JniContext {
    fn source_result(&self) -> &syn::Ident {
        &self.source_result
    }
}
impl TemplateContext for JniContext {
    fn render(&self, code: &TokenStream) -> Result<TokenStream> {
        render_template(code, &|name| match name {
            "source_result" => Ok(self.source_result.to_token_stream()),
            "environment" => Ok(self.environment.to_token_stream()),
            "acknowledgement" => Ok(self.acknowledgement.to_token_stream()),
            _ => Err(format!("unknown emission parameter `{name}`")),
        })
    }
}
