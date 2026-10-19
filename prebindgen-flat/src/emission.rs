//! Explicit bindings between prepared conversion code and its wrapper.
use crate::Result;
use proc_macro2::{Group, TokenStream, TokenTree};
use quote::{quote, ToTokens};

/// Language-generator-owned context used when assembling a source call.
/// All other fields and their meaning are chosen by the language generator.
pub trait EmissionContext {
    /// Name of the generated variable that holds the source function's return value.
    fn source_result(&self) -> &syn::Ident;
}

/// Optional rendering support for stored [`crate::InputPlan`] and
/// [`crate::OutputPlan`] templates. Custom conversions need not implement this.
pub trait TemplateContext: EmissionContext {
    /// Resolve the template's explicit parameter markers.
    fn render(&self, code: &TokenStream) -> Result<TokenStream>;
}

impl EmissionContext for syn::Ident {
    fn source_result(&self) -> &syn::Ident {
        self
    }
}
impl TemplateContext for syn::Ident {
    fn render(&self, code: &TokenStream) -> Result<TokenStream> {
        render_template(code, &|name| match name {
            "source_result" => Ok(self.to_token_stream()),
            _ => Err(format!("unknown emission parameter `{name}`")),
        })
    }
}

/// Insert an explicit template marker. The context renderer defines its meaning.
/// Ordinary identifiers and string literals are never substituted.
pub fn template_parameter(name: &str) -> TokenStream {
    let name = syn::Ident::new(name, proc_macro2::Span::call_site());
    quote!(__prebindgen_emit!(#name))
}

/// Resolve explicit markers recursively using a language generator's bindings.
/// `__prebindgen_emit!(...)` is reserved for [`template_parameter`].
pub fn render_template(
    code: &TokenStream,
    resolve: &impl Fn(&str) -> Result<TokenStream>,
) -> Result<TokenStream> {
    let tokens: Vec<_> = code.clone().into_iter().collect();
    let mut output = TokenStream::new();
    let mut index = 0;
    while index < tokens.len() {
        if matches!(&tokens[index], TokenTree::Ident(id) if id == "__prebindgen_emit")
            && matches!(tokens.get(index + 1), Some(TokenTree::Punct(p)) if p.as_char() == '!')
        {
            let Some(TokenTree::Group(group)) = tokens.get(index + 2) else {
                return Err("invalid emission parameter marker".into());
            };
            let name = group.stream().to_string();
            let binding = resolve(&name)?;
            output.extend(binding);
            index += 3;
            continue;
        }
        match &tokens[index] {
            TokenTree::Group(group) => {
                let mut rendered = Group::new(
                    group.delimiter(),
                    render_template(&group.stream(), resolve)?,
                );
                rendered.set_span(group.span());
                output.extend([TokenTree::Group(rendered)]);
            }
            other => output.extend([other.clone()]),
        }
        index += 1;
    }
    Ok(output)
}
