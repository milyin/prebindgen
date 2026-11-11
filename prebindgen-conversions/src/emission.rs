//! Explicit bindings between prepared conversion code and its wrapper.
use prebindgen_flat::Result;
use proc_macro2::{Group, TokenStream, TokenTree};
use quote::{quote, ToTokens};

/// Optional rendering support for stored [`crate::InputPlan`] and
/// [`crate::OutputPlan`] templates. Custom conversions need not implement this.
pub trait TemplateContext {
    /// Resolve local binding and transfer markers before inserting the rendered
    /// continuation. Already-rendered Rust remains unchanged.
    fn render(&self, code: &TokenStream) -> Result<TokenStream>;
}

/// Ownership reporting required by the generated boundary.
#[derive(Clone, Debug, Default)]
pub enum InputTransfers {
    /// No ownership transfer is allowed at this boundary.
    #[default]
    Forbidden,
    /// Record completed takes in this boundary's local `Cell<usize>`.
    Report(syn::Ident),
    /// Errors abort the call; no incoming-owner receipt is observable.
    Abort,
}
impl InputTransfers {
    /// Emit code for completed ownership takes according to the boundary contract.
    pub fn render(&self, count: &TokenStream) -> Result<TokenStream> {
        match self {
            Self::Report(counter) => Ok(quote!({#counter.set(#counter.get()+(#count));})),
            Self::Abort => Ok(quote!({})),
            Self::Forbidden => Err(
                "input transfer requires an ownership receipt at this recoverable boundary".into(),
            ),
        }
    }
}
impl TemplateContext for () {
    fn render(&self, code: &TokenStream) -> Result<TokenStream> {
        render_template(
            code,
            &|name| Err(format!("unknown emission parameter `{name}`")),
            &|count| InputTransfers::Forbidden.render(count),
        )
    }
}
/// Record successful incoming ownership takes, before later fallible/unwinding work.
/// `count` must be a side-effect-free `usize` expression. An aborting boundary may
/// omit its evaluation. Never report a take which has not happened.
pub fn record_input_transfer(count: TokenStream) -> TokenStream {
    quote!(__prebindgen_input_transfer! {#count})
}
/// Bind a prepared conversion's target while preserving adapter markers.
pub(crate) fn bind_target(code: &TokenStream, target: &syn::Ident) -> Result<TokenStream> {
    render_template(
        code,
        &|name| {
            Ok(if name == "input_target" {
                target.to_token_stream()
            } else {
                template_parameter(name)
            })
        },
        &|count| Ok(record_input_transfer(count.clone())),
    )
}

/// Resolve the call's source binding before generator-specific template parameters.
pub fn render_output(
    context: &impl TemplateContext,
    code: &TokenStream,
    source: &syn::Ident,
) -> Result<TokenStream> {
    let code = render_template(
        code,
        &|name| {
            Ok(if name == "source_result" {
                source.to_token_stream()
            } else {
                template_parameter(name)
            })
        },
        &|count| Ok(record_input_transfer(count.clone())),
    )?;
    context.render(&code)
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
    transfer: &impl Fn(&TokenStream) -> Result<TokenStream>,
) -> Result<TokenStream> {
    let tokens: Vec<_> = code.clone().into_iter().collect();
    let mut output = TokenStream::new();
    let mut index = 0;
    while index < tokens.len() {
        if matches!(&tokens[index], TokenTree::Ident(id) if id == "__prebindgen_emit" || id == "__prebindgen_input_transfer")
            && matches!(tokens.get(index + 1), Some(TokenTree::Punct(p)) if p.as_char() == '!')
        {
            let Some(TokenTree::Group(group)) = tokens.get(index + 2) else {
                return Err("invalid emission parameter marker".into());
            };
            let binding = if matches!(&tokens[index],TokenTree::Ident(id) if id == "__prebindgen_input_transfer")
            {
                let count = render_template(&group.stream(), resolve, transfer)?;
                transfer(&count)?
            } else {
                let name = group.stream().to_string();
                if name == "continuation" {
                    template_parameter(&name)
                } else {
                    resolve(&name)?
                }
            };
            output.extend(binding);
            index += 3;
            continue;
        }
        match &tokens[index] {
            TokenTree::Group(group) => {
                let mut rendered = Group::new(
                    group.delimiter(),
                    render_template(&group.stream(), resolve, transfer)?,
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
