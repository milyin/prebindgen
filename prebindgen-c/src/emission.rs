//! Bindings supplied by this language generator during slot emission.
use prebindgen_flat::{render_template, EmissionContext, Result, TemplateContext};
use proc_macro2::TokenStream;
use quote::ToTokens;

/// Concrete wrapper bindings used by this language generator.
#[derive(Clone, Debug)]
pub struct CContext {
    /// Local holding the source function result.
    pub source_result: syn::Ident,
    /// Wrapper parameter receiving the result structure.
    pub output_pointer: syn::Ident,
    /// Concrete result structure constructor.
    pub output_type: syn::Path,
    /// Byte buffer carrier constructor.
    pub buffer_type: syn::Path,
    /// Wide integer carrier constructor.
    pub wide_integer_type: syn::Path,
    /// Helper that reads an input byte buffer.
    pub read_buffer: syn::Path,
}
impl EmissionContext for CContext {
    fn source_result(&self) -> &syn::Ident {
        &self.source_result
    }
}
impl TemplateContext for CContext {
    fn render(&self, code: &TokenStream) -> Result<TokenStream> {
        render_template(code, &|name| match name {
            "source_result" => Ok(self.source_result.to_token_stream()),
            "output_pointer" => Ok(self.output_pointer.to_token_stream()),
            "output_type" => Ok(self.output_type.to_token_stream()),
            "buffer_type" => Ok(self.buffer_type.to_token_stream()),
            "wide_integer_type" => Ok(self.wide_integer_type.to_token_stream()),
            "read_buffer" => Ok(self.read_buffer.to_token_stream()),
            _ => Err(format!("unknown emission parameter `{name}`")),
        })
    }
}
