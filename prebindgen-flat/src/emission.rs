//! Explicit bindings between prepared conversion code and its wrapper.
use crate::Result;
use proc_macro2::{Group, TokenStream, TokenTree};
use quote::{quote, ToTokens};

/// A wrapper-supplied binding referenced by a stored code template.
/// Markers are expanded only by [`EmissionContext::render`]; ordinary identifiers
/// and string literals are never renamed. Custom slot implementations can use
/// the context fields directly instead of storing templates.
#[derive(Clone, Copy, Debug)]
pub enum EmissionParameter {
    /// Local bound to the wrapped Rust function's return value.
    SourceResult,
    /// Mutable runtime environment, such as a JNI environment.
    Environment,
    /// Pointer receiving delivered outputs.
    OutputPointer,
    /// Concrete Rust path used to construct the wrapper result structure.
    OutputType,
    /// Parameter used to acknowledge ownership transfer to the caller.
    Acknowledgement,
    /// Rust type constructing a byte buffer carrier.
    BufferType,
    /// Rust type constructing a wide integer carrier.
    WideIntegerType,
    /// Helper function reading an input buffer.
    ReadBuffer,
}
impl EmissionParameter {
    /// Insert an explicit parameter marker into a prepared token stream.
    pub fn tokens(self) -> TokenStream {
        let name = syn::Ident::new(
            match self {
                Self::SourceResult => "source_result",
                Self::Environment => "environment",
                Self::OutputPointer => "output_pointer",
                Self::OutputType => "output_type",
                Self::Acknowledgement => "acknowledgement",
                Self::BufferType => "buffer_type",
                Self::WideIntegerType => "wide_integer_type",
                Self::ReadBuffer => "read_buffer",
            },
            proc_macro2::Span::call_site(),
        );
        quote!(__prebindgen_emit!(#name))
    }
}

/// Names selected by the wrapper writer and passed explicitly to slot emission.
/// Optional bindings are required only if a conversion uses them. Missing bindings
/// produce generation errors, rather than undeclared names in generated Rust.
#[derive(Clone, Debug)]
pub struct EmissionContext {
    /// Local in which `emit_call` binds the source function's return value.
    pub source_result: syn::Ident,
    /// Mutable runtime environment variable.
    pub environment: Option<syn::Ident>,
    /// Output pointer variable.
    pub output_pointer: Option<syn::Ident>,
    /// Result structure constructor path.
    pub output_type: Option<syn::Path>,
    /// Ownership acknowledgement parameter.
    pub acknowledgement: Option<syn::Ident>,
    /// Byte buffer carrier constructor path.
    pub buffer_type: Option<syn::Path>,
    /// Wide integer carrier constructor path.
    pub wide_integer_type: Option<syn::Path>,
    /// Input buffer reader function path.
    pub read_buffer: Option<syn::Path>,
}
impl EmissionContext {
    /// Start with an explicitly chosen source-result local and no adapter bindings.
    pub fn new(source_result: syn::Ident) -> Self {
        Self {
            source_result,
            environment: None,
            output_pointer: None,
            output_type: None,
            acknowledgement: None,
            buffer_type: None,
            wide_integer_type: None,
            read_buffer: None,
        }
    }
    /// Resolve explicit markers recursively, leaving other tokens unchanged.
    /// `__prebindgen_emit!(...)` is reserved for markers created by
    /// [`EmissionParameter::tokens`].
    pub fn render(&self, code: &TokenStream) -> Result<TokenStream> {
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
                let binding = match name.as_str() {
                    "source_result" => Some(self.source_result.to_token_stream()),
                    "environment" => self.environment.as_ref().map(ToTokens::to_token_stream),
                    "output_pointer" => self.output_pointer.as_ref().map(ToTokens::to_token_stream),
                    "output_type" => self.output_type.as_ref().map(ToTokens::to_token_stream),
                    "acknowledgement" => {
                        self.acknowledgement.as_ref().map(ToTokens::to_token_stream)
                    }
                    "buffer_type" => self.buffer_type.as_ref().map(ToTokens::to_token_stream),
                    "wide_integer_type" => self
                        .wide_integer_type
                        .as_ref()
                        .map(ToTokens::to_token_stream),
                    "read_buffer" => self.read_buffer.as_ref().map(ToTokens::to_token_stream),
                    _ => return Err(format!("unknown emission parameter `{name}`")),
                }
                .ok_or_else(|| format!("missing emission binding `{name}`"))?;
                output.extend(binding);
                index += 3;
                continue;
            }
            match &tokens[index] {
                TokenTree::Group(group) => {
                    let mut rendered = Group::new(group.delimiter(), self.render(&group.stream())?);
                    rendered.set_span(group.span());
                    output.extend([TokenTree::Group(rendered)]);
                }
                other => output.extend([other.clone()]),
            }
            index += 1;
        }
        Ok(output)
    }
}
