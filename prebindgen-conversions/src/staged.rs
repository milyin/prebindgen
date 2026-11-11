//! Optional expression protocol for stored plans used by the C and JNI adapters.
//! Scoped requests keep prepared temporaries alive while evaluating a continuation.
//! Other generators can define entirely different request enums.
use crate::{InputPlan, OutputPlan, TemplateContext};
use prebindgen_flat::{Conversion, Function, Result, WireKind};
use proc_macro2::TokenStream;
use quote::quote;

/// Convert one input, then run `next` while the value and its backing storage live.
/// Takes must be reported in wire traversal order before later failures/unwinds.
/// Incorrect access metadata or omitted reporting can make caller/callback cleanup
/// free an acquired owner twice. Custom conversion authors must uphold this safety
/// contract; arbitrary emitted Rust cannot be proved correct from another inventory.
/// Place `next` exactly once as an unlabelled tail block, without a new closure.
pub struct InputFragment<'a, C> {
    /// Concrete bindings and ownership reporting for this boundary.
    pub context: &'a C,
    /// Caller-selected Rust argument name.
    pub target: &'a syn::Ident,
    /// Already-rendered remaining call code.
    pub next: TokenStream,
}
/// Slot requests understood by stored output plans.
pub enum OutputFragment<'a, C> {
    /// Prepare output temporaries, then evaluate `next` while they remain alive.
    Prepare {
        /// Bindings used to render prepared templates.
        context: &'a C,
        /// Caller-selected source-result binding.
        source: &'a syn::Ident,
        /// Expression evaluated inside the scope introduced by this request.
        next: TokenStream,
    },
    /// Deliver output, then evaluate `next` with delivery bindings still in scope.
    Deliver {
        /// Bindings used to render prepared templates.
        context: &'a C,
        /// Caller-selected source-result binding.
        source: &'a syn::Ident,
        /// Expression evaluated inside the scope introduced by this request.
        next: TokenStream,
    },
    /// Unit expression committing ownership after successful delivery.
    Commit {
        /// Bindings used to render prepared templates.
        context: &'a C,
        /// Caller-selected source-result binding.
        source: &'a syn::Ident,
    },
    /// Wrapper's final result expression.
    Result {
        /// Bindings used to render prepared templates.
        context: &'a C,
        /// Caller-selected source-result binding.
        source: &'a syn::Ident,
    },
}
fn expression(code: TokenStream) -> Result<TokenStream> {
    syn::parse2::<syn::Expr>(code.clone())
        .map_err(|e| format!("conversion did not emit an expression: {e}"))?;
    Ok(code)
}

impl<W: WireKind, C: TemplateContext + 'static, N, B> Conversion for InputPlan<W, C, N, B> {
    type Wire = W;
    type Chain = N;
    type Branch = B;
    type SlotFragment<'a> = InputFragment<'a, C>;
    fn code_fragment_slot(
        &self,
        request: InputFragment<'_, C>,
        _: &syn::Type,
    ) -> Result<TokenStream> {
        self.emit_code(request, &self.code)
    }
}
impl<W: WireKind, C: TemplateContext + 'static, N, B> InputPlan<W, C, N, B> {
    /// Emit the complete conversion using caller-specialized carrier reads.
    pub fn emit_code(
        &self,
        request: InputFragment<'_, C>,
        code: &TokenStream,
    ) -> Result<TokenStream> {
        let code = crate::emission::bind_target(code, request.target)?;
        let code = request.context.render(&code)?;
        let storage = request.context.render(&self.storage)?;
        let body = continue_with(&code, request.next)?;
        expression(scope(storage, body)?)
    }
}
impl<W: WireKind, C: TemplateContext + 'static, N, B> Conversion for OutputPlan<W, C, N, B> {
    type Wire = W;
    type Chain = N;
    type Branch = B;
    type SlotFragment<'a> = OutputFragment<'a, C>;

    fn code_fragment_slot(
        &self,
        mode: OutputFragment<'_, C>,
        _expected_type: &syn::Type,
    ) -> Result<TokenStream> {
        self.emit_with_delivery(mode, &self.delivery)
    }
}
impl<W: WireKind, C: TemplateContext + 'static, N, B> OutputPlan<W, C, N, B> {
    /// Emit the output phases with an explicitly selected delivery protocol.
    /// Preparation and ownership commit still use this plan's original storage.
    pub fn emit_with_delivery(
        &self,
        mode: OutputFragment<'_, C>,
        delivery: &crate::Delivery,
    ) -> Result<TokenStream> {
        expression(match mode {
            OutputFragment::Prepare {
                context,
                source,
                next,
            } => {
                let code = crate::emission::render_output(context, &self.statements, source)?;
                scope(code, next)?
            }
            OutputFragment::Deliver {
                context,
                source,
                next,
            } => {
                let code = crate::emission::render_output(context, &delivery.statements, source)?;
                scope(code, next)?
            }
            OutputFragment::Commit { context, source } => {
                let code = crate::emission::render_output(context, &self.commit, source)?;
                quote!({ #code })
            }
            OutputFragment::Result { context, source } => {
                crate::emission::render_output(context, &delivery.result, source)?
            }
        })
    }
}
/// Compose complete input conversions, the source call and output delivery.
/// `result_type` is the wrapper result type inside `Result<_, String>`.
///
/// Adapters establish their boundary receipt before entering this expression.
/// Other generators may compose their own request protocols outside Flat.
pub fn emit_call<I, O, C: TemplateContext + 'static, M>(
    function: &Function<I, O, M>,
    context: &C,
    source: &syn::Ident,
    result_type: &syn::Type,
    parameter_names: &[syn::Ident],
) -> Result<TokenStream>
where
    for<'a> I: Conversion<SlotFragment<'a> = InputFragment<'a, C>>,
    for<'a> O: Conversion<SlotFragment<'a> = OutputFragment<'a, C>>,
{
    let inputs = function.input_conversions()?;
    let output = function.output_conversion()?;
    let unit = syn::parse_quote!(());
    let expected = syn::parse_quote!(::core::result::Result<#result_type, ::std::string::String>);
    let continuation = crate::template_parameter("continuation");
    let path = &function.path;
    let prepare = output.code_fragment_slot(
        OutputFragment::Prepare {
            context,
            source,
            next: continuation.clone(),
        },
        &expected,
    )?;
    let deliver = output.code_fragment_slot(
        OutputFragment::Deliver {
            context,
            source,
            next: continuation,
        },
        &expected,
    )?;
    let commit = output.code_fragment_slot(OutputFragment::Commit { context, source }, &unit)?;
    let result =
        output.code_fragment_slot(OutputFragment::Result { context, source }, result_type)?;
    let mut used = parameter_names
        .iter()
        .map(ToString::to_string)
        .collect::<std::collections::BTreeSet<_>>();
    reserve(
        &quote!(#path #source #prepare #deliver #commit #result),
        &mut used,
    );
    let mut fresh = |base: String| {
        let mut name = base;
        while !used.insert(name.clone()) {
            name.push('_');
        }
        syn::Ident::new(&name, proc_macro2::Span::call_site())
    };
    let args: Vec<_> = (0..inputs.len())
        .map(|i| fresh(format!("__prebindgen_argument_{i}")))
        .collect();
    let delivered = continue_with(&deliver, quote!({let _: () = #commit; Ok(#result)}))?;
    let mut body = continue_with(&prepare, delivered)?;
    body = scope(quote!(let #source=#path(#(#args),*);), body)?;
    for (input, target) in inputs.iter().zip(&args).rev() {
        body = input.code_fragment_slot(
            InputFragment {
                context,
                target,
                next: body,
            },
            &expected,
        )?;
    }
    Ok(body)
}
/// Replace exactly one continuation without rendering or re-scanning inserted code.
/// This operation can compose prepared templates before adapter bindings exist.
pub fn continue_with(code: &TokenStream, next: TokenStream) -> Result<TokenStream> {
    fn walk(code: &TokenStream, next: &TokenStream, count: &mut usize) -> Result<TokenStream> {
        let tokens: Vec<_> = code.clone().into_iter().collect();
        let mut out = TokenStream::new();
        let mut i = 0;
        while i < tokens.len() {
            if matches!(&tokens[i],proc_macro2::TokenTree::Ident(id) if id=="__prebindgen_emit")
                && matches!(tokens.get(i+1),Some(proc_macro2::TokenTree::Punct(p)) if p.as_char()=='!')
                && matches!(tokens.get(i+2),Some(proc_macro2::TokenTree::Group(g)) if g.stream().to_string()=="continuation")
            {
                *count += 1;
                out.extend(next.clone());
                i += 3;
                continue;
            }
            match &tokens[i] {
                proc_macro2::TokenTree::Group(g) => {
                    let mut group =
                        proc_macro2::Group::new(g.delimiter(), walk(&g.stream(), next, count)?);
                    group.set_span(g.span());
                    out.extend([proc_macro2::TokenTree::Group(group)]);
                }
                token => out.extend([token.clone()]),
            }
            i += 1;
        }
        Ok(out)
    }
    let mut count = 0;
    let out = walk(code, &next, &mut count)?;
    if count != 1 {
        return Err(format!(
            "input/output conversion must contain exactly one continuation, found {count}"
        ));
    }
    flatten_tail(out)
}
/// Place code inside an enclosing backing-storage scope.
pub(crate) fn enclose(storage: TokenStream, body: TokenStream) -> Result<TokenStream> {
    scope(storage, body)
}

// Scoped requests use a tail continuation. Merge unlabelled tail blocks to keep
// generated syntax shallow for functions with hundreds of wire parameters.
// Statements retain their order, and locals still surround the continuation.
fn scope(prefix: TokenStream, next: TokenStream) -> Result<TokenStream> {
    flatten_tail(quote!({ #prefix #next }))
}
fn flatten_tail(code: TokenStream) -> Result<TokenStream> {
    let mut expr: syn::Expr = syn::parse2(code).map_err(|e| e.to_string())?;
    if let syn::Expr::Block(outer) = &mut expr {
        while let Some(syn::Stmt::Expr(syn::Expr::Block(inner), None)) = outer.block.stmts.last() {
            if inner.label.is_some() || !inner.attrs.is_empty() {
                break;
            }
            let Some(syn::Stmt::Expr(syn::Expr::Block(inner), None)) = outer.block.stmts.pop()
            else {
                unreachable!()
            };
            outer.block.stmts.extend(inner.block.stmts);
        }
    }
    Ok(quote!(#expr))
}
fn reserve(code: &TokenStream, used: &mut std::collections::BTreeSet<String>) {
    for token in code.clone() {
        match token {
            proc_macro2::TokenTree::Ident(name) => {
                used.insert(name.to_string());
            }
            proc_macro2::TokenTree::Group(group) => reserve(&group.stream(), used),
            _ => {}
        }
    }
}

impl<W: WireKind, C: TemplateContext + 'static, N, B> prebindgen_flat::InputConversion
    for InputPlan<W, C, N, B>
{
    fn input_node(&self) -> &prebindgen_flat::WireNode<W, N, B> {
        &self.node
    }
}
impl<W: WireKind, C: TemplateContext + 'static, N, B> prebindgen_flat::OutputConversion
    for OutputPlan<W, C, N, B>
{
    fn output_outcomes(&self) -> &prebindgen_flat::Outcomes<prebindgen_flat::WireNode<W, N, B>> {
        &self.outcomes
    }
}
