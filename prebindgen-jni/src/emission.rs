//! Bindings supplied by this language generator during slot emission.
use prebindgen_flat::Result;
use prebindgen_conversions::{render_template, TemplateContext};
use proc_macro2::TokenStream;
use quote::ToTokens;

/// Concrete wrapper bindings used by this language generator.
#[derive(Clone, Debug)]
pub struct JniContext {
    /// Mutable JNI environment parameter.
    pub environment: syn::Ident,
    /// Boolean-array parameter acknowledging ownership transfer.
    pub acknowledgement: syn::Ident,
}
impl TemplateContext for JniContext {
    fn render(&self, code: &TokenStream) -> Result<TokenStream> {
        render_template(code, &|name| match name {
            "environment" => Ok(self.environment.to_token_stream()),
            "acknowledgement" => Ok(self.acknowledgement.to_token_stream()),
            _ => Err(format!("unknown emission parameter `{name}`")),
        })
    }
}
