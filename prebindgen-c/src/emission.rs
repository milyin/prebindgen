//! Bindings supplied by this language generator during slot emission.
use prebindgen_conversions::{render_template, TemplateContext};
use prebindgen_flat::Result;
use proc_macro2::TokenStream;
use quote::ToTokens;

/// Concrete wrapper bindings used by this language generator.
#[derive(Clone, Debug)]
pub struct CContext {
    /// Generated ABI row types keyed by sequence carrier name in this function.
    pub row_types: std::collections::BTreeMap<String, syn::Ident>,
    /// Wrapper-local flag recording acceptance of owned input allocations.
    pub acknowledgement: syn::Ident,
    /// Wrapper parameter receiving the result structure.
    pub output_pointer: syn::Ident,
    /// Concrete result structure constructor.
    pub output_type: syn::Path,
    /// Optional explicit Result error output parameter.
    pub error_pointer: syn::Ident,
    /// Records that a source error has already been delivered.
    pub error_reported: syn::Ident,
    /// Infallible statements appended after output ownership commits.
    pub output_committed: TokenStream,
    /// Byte buffer carrier constructor.
    pub buffer_type: syn::Path,
    /// Wide integer carrier constructor.
    pub wide_integer_type: syn::Path,
    /// Helper that reads an input byte buffer.
    pub read_buffer: syn::Path,
}
impl TemplateContext for CContext {
    fn render(&self, code: &TokenStream) -> Result<TokenStream> {
        render_template(code, &|name| match name {
            "output_committed" => Ok(self.output_committed.clone()),
            "error_pointer" => Ok(self.error_pointer.to_token_stream()),
            "error_reported" => Ok(self.error_reported.to_token_stream()),
            "acknowledgement" => Ok(self.acknowledgement.to_token_stream()),
            "output_pointer" => Ok(self.output_pointer.to_token_stream()),
            "output_type" => Ok(self.output_type.to_token_stream()),
            "buffer_type" => Ok(self.buffer_type.to_token_stream()),
            "wide_integer_type" => Ok(self.wide_integer_type.to_token_stream()),
            "read_buffer" => Ok(self.read_buffer.to_token_stream()),
            _ if name.starts_with("row_") => self
                .row_types
                .get(&name[4..])
                .map(ToTokens::to_token_stream)
                .ok_or_else(|| format!("unknown collection row `{name}`")),
            _ => Err(format!("unknown emission parameter `{name}`")),
        })
    }
}
