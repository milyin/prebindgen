//! Optional expression protocol for stored plans used by the C and JNI adapters.
//! Scoped requests keep prepared temporaries alive while evaluating a continuation.
//! Other generators can define entirely different request enums.
use crate::{Access, InputPlan, OutputPlan, TemplateContext};
use prebindgen_flat::{Conversion, Function, Result, Wire, WireKind};
use proc_macro2::TokenStream;
use quote::quote;

/// Slot requests understood by the stored input plans.
pub enum InputFragment<'a, C> {
    /// Validate and prepare storage, then evaluate `next` in that storage's scope.
    Validate {
        /// Bindings used to render prepared templates.
        context: &'a C,
        /// Expression evaluated inside the scope introduced by this request.
        next: TokenStream,
    },
    /// Acquire ownership, then evaluate `next` while acquired owners remain alive.
    Acquire {
        /// Bindings used to render prepared templates.
        context: &'a C,
        /// Expression evaluated inside the scope introduced by this request.
        next: TokenStream,
    },
    /// Construct the argument as `target`, then evaluate `next` in its scope.
    Construct {
        /// Bindings used to render prepared templates.
        context: &'a C,
        /// Caller-selected argument binding.
        target: &'a syn::Ident,
        /// Expression evaluated inside the scope introduced by this request.
        next: TokenStream,
    },
    /// Unit expression acknowledging ownership, or exactly `()` if unnecessary.
    Acknowledge {
        /// Bindings used to render prepared templates.
        context: &'a C,
    },
    /// Check addresses against `seen: Vec<(usize, bool)>`, then record them.
    /// The boolean means exclusive access (including ownership transfer).
    CheckAliases {
        /// Bindings used to render prepared templates.
        context: &'a C,
        /// Caller-owned collection of previously checked allocation accesses.
        seen: &'a syn::Ident,
    },
}
/// Per-wire requests understood by stored input plans.
pub enum InputWireFragment {
    /// Read the boundary parameter as an expression.
    Value,
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
/// Per-wire output expression requests.
pub enum OutputWireFragment<'a, C> {
    /// Read this wire's prepared value before ownership commit.
    Value {
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

impl<W: WireKind, C: TemplateContext + 'static> Conversion for InputPlan<W, C> {
    type Wire = W;
    type SlotFragment<'a> = InputFragment<'a, C>;
    type WireFragment<'a> = InputWireFragment;
    fn wires(&self) -> &[Wire<W>] {
        &self.wires
    }
    fn code_fragment_slot(
        &self,
        mode: InputFragment<'_, C>,
        _expected_type: &syn::Type,
    ) -> Result<TokenStream> {
        expression(match mode {
            InputFragment::Validate { context, next } => {
                let code = context.render(&self.validation)?;
                scope(code, next)?
            }
            InputFragment::Acquire { context, next } => {
                let code = context.render(&self.acquire)?;
                scope(code, next)?
            }
            InputFragment::Construct {
                context,
                target,
                next,
            } => {
                let code = context.render(&self.statements)?;
                let value = context.render(&self.argument)?;
                scope(quote!(#code let #target = #value;), next)?
            }
            InputFragment::Acknowledge { context } => match &self.acknowledgement {
                Some(code) => {
                    let code = context.render(code)?;
                    quote!({ #code })
                }
                None => quote!(()),
            },
            InputFragment::CheckAliases { context, seen } => {
                let mut checks = TokenStream::new();
                for resource in &self.resources {
                    let identity = context.render(&resource.identity)?;
                    let active = context.render(&resource.active)?;
                    let exclusive = resource.access != Access::Shared;
                    let conflict = if exclusive {
                        quote!(identity == other)
                    } else {
                        quote!(identity == other && exclusive)
                    };
                    let prior_access = if exclusive {
                        quote!(_)
                    } else {
                        quote!(exclusive)
                    };
                    checks.extend(quote! {
                        if #active {
                            let identity = (#identity) as usize;
                            if #seen.iter().any(|&(other, #prior_access)| #conflict) {
                                return Err("conflicting input aliases".into());
                            }
                            #seen.push((identity, #exclusive));
                        }
                    });
                }
                quote!({ #checks })
            }
        })
    }
    fn code_fragment_wire(
        &self,
        pos: usize,
        _mode: InputWireFragment,
        _expected_type: &syn::Type,
    ) -> Result<TokenStream> {
        let wire = self
            .wires
            .get(pos)
            .ok_or_else(|| format!("invalid input wire position {pos}"))?;
        let name = &wire.name;
        Ok(quote!(#name))
    }
}

impl<W: WireKind, C: TemplateContext + 'static> Conversion for OutputPlan<W, C> {
    type Wire = W;
    type SlotFragment<'a> = OutputFragment<'a, C>;
    type WireFragment<'a> = OutputWireFragment<'a, C>;
    fn wires(&self) -> &[Wire<W>] {
        &self.wires
    }
    fn code_fragment_slot(
        &self,
        mode: OutputFragment<'_, C>,
        _expected_type: &syn::Type,
    ) -> Result<TokenStream> {
        self.check_arity()?;
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
                let code =
                    crate::emission::render_output(context, &self.delivery.statements, source)?;
                scope(code, next)?
            }
            OutputFragment::Commit { context, source } => {
                let code = crate::emission::render_output(context, &self.commit, source)?;
                quote!({ #code })
            }
            OutputFragment::Result { context, source } => {
                crate::emission::render_output(context, &self.delivery.result, source)?
            }
        })
    }
    fn code_fragment_wire(
        &self,
        pos: usize,
        mode: OutputWireFragment<'_, C>,
        _expected_type: &syn::Type,
    ) -> Result<TokenStream> {
        self.check_arity()?;
        let code = self
            .values
            .get(pos)
            .ok_or_else(|| format!("invalid output wire position {pos}"))?;
        let OutputWireFragment::Value { context, source } = mode;
        expression(crate::emission::render_output(context, code, source)?)
    }
}
impl<W: WireKind, C> OutputPlan<W, C> {
    fn check_arity(&self) -> Result<()> {
        if self.wires.len() != self.values.len() {
            return Err("output wire/value count differs".into());
        }
        Ok(())
    }
}

/// An output expression paired with its wire description.
pub struct OutputValue<'a, W: WireKind> {
    /// Boundary carrier description.
    pub wire: &'a Wire<W>,
    /// Expression requested using the wire's Rust carrier type.
    pub value: TokenStream,
}
/// Convenience access to output values for generators choosing this protocol.
pub trait StagedFunction<C, W: WireKind> {
    /// Request each wire value in delivery order.
    fn outputs(
        &self,
        context: &C,
        source: &syn::Ident,
    ) -> Result<std::vec::IntoIter<OutputValue<'_, W>>>;
}
impl<I, O, C: 'static> StagedFunction<C, O::Wire> for Function<I, O>
where
    I: Conversion,
    O: Conversion,
    for<'a> O: Conversion<WireFragment<'a> = OutputWireFragment<'a, C>>,
{
    fn outputs(
        &self,
        context: &C,
        source: &syn::Ident,
    ) -> Result<std::vec::IntoIter<OutputValue<'_, O::Wire>>> {
        let output = self.output_conversion()?;
        output
            .wires()
            .iter()
            .enumerate()
            .map(|(pos, wire)| {
                let value = output.code_fragment_wire(
                    pos,
                    OutputWireFragment::Value { context, source },
                    &wire.rust_type(),
                )?;
                Ok(OutputValue { wire, value })
            })
            .collect::<Result<Vec<_>>>()
            .map(Vec::into_iter)
    }
}

/// Compose the opt-in staged protocol as a fallible expression.
/// `result_type` is the wrapper result type inside `Result<_, String>`.
///
/// Generators choosing this helper use address alias checks and one shared
/// acknowledgement. Other generators can compose their own request protocols.
/// This is deliberately outside Flat: none of these phases are required by its traits.
pub fn emit_call<I, O, C: 'static>(
    function: &Function<I, O>,
    context: &C,
    source: &syn::Ident,
    result_type: &syn::Type,
) -> Result<TokenStream>
where
    I: Conversion,
    O: Conversion,
    for<'a> I: Conversion<SlotFragment<'a> = InputFragment<'a, C>>,
    for<'a> O: Conversion<
        SlotFragment<'a> = OutputFragment<'a, C>,
        WireFragment<'a> = OutputWireFragment<'a, C>,
    >,
{
    let inputs = function.input_conversions()?;
    let output = function.output_conversion()?;
    let unit = syn::parse_quote!(());
    let expected = syn::parse_quote!(::core::result::Result<#result_type, ::std::string::String>);
    let continuation = crate::template_parameter("continuation");
    let path = &function.path;
    let mut acknowledge: Option<TokenStream> = None;
    for input in &inputs {
        let code = input.code_fragment_slot(InputFragment::Acknowledge { context }, &unit)?;
        if code.to_string() == quote!(()).to_string() {
            continue;
        }
        if acknowledge
            .as_ref()
            .is_some_and(|previous| previous.to_string() != code.to_string())
        {
            return Err(format!(
                "`{}`: conflicting input acknowledgements",
                function.name
            ));
        }
        acknowledge = Some(code);
    }
    let validation = inputs
        .iter()
        .map(|input| {
            input.code_fragment_slot(
                InputFragment::Validate {
                    context,
                    next: continuation.clone(),
                },
                &expected,
            )
        })
        .collect::<Result<Vec<_>>>()?;
    let acquire = inputs
        .iter()
        .map(|input| {
            input.code_fragment_slot(
                InputFragment::Acquire {
                    context,
                    next: continuation.clone(),
                },
                &expected,
            )
        })
        .collect::<Result<Vec<_>>>()?;
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
            next: continuation.clone(),
        },
        &expected,
    )?;
    let commit = output.code_fragment_slot(OutputFragment::Commit { context, source }, &unit)?;
    let result =
        output.code_fragment_slot(OutputFragment::Result { context, source }, result_type)?;
    let _ = function.outputs(context, source)?;
    let mut used = std::collections::BTreeSet::new();
    for input in &inputs {
        for wire in input.wires() {
            used.insert(wire.name.to_string());
        }
    }
    reserve(
        &quote!(#path #source #(#validation)* #(#acquire)* #prepare #deliver #commit #result #acknowledge),
        &mut used,
    );
    let mut fresh = |base: String| {
        let mut name = base;
        while !used.insert(name.clone()) {
            name.push('_');
        }
        syn::Ident::new(&name, proc_macro2::Span::call_site())
    };
    let seen = fresh("__prebindgen_resources".into());
    let args: Vec<_> = (0..inputs.len())
        .map(|i| fresh(format!("__prebindgen_argument_{i}")))
        .collect();
    let checks = inputs
        .iter()
        .map(|input| {
            input.code_fragment_slot(
                InputFragment::CheckAliases {
                    context,
                    seen: &seen,
                },
                &unit,
            )
        })
        .collect::<Result<Vec<_>>>()?;
    let delivered = plug(&deliver, quote!({ let _: () = #commit; Ok(#result) }))?;
    let mut body = plug(&prepare, delivered)?;
    body = scope(quote!(let #source = #path(#(#args),*);), body)?;
    for (input, target) in inputs.iter().zip(&args).rev() {
        body = input.code_fragment_slot(
            InputFragment::Construct {
                context,
                target,
                next: body,
            },
            &expected,
        )?;
    }
    for scope in acquire.iter().rev() {
        body = plug(scope, body)?;
    }
    body = if let Some(ack) = acknowledge {
        scope(quote!(let _: () = #ack;), body)?
    } else {
        body
    };
    body = scope(
        quote!(
            let mut #seen: Vec<(usize, bool)> = Vec::new();
            #(let _: () = #checks;)*
        ),
        body,
    )?;
    for scope in validation.iter().rev() {
        body = plug(scope, body)?;
    }
    Ok(body)
}
fn plug(scope: &TokenStream, next: TokenStream) -> Result<TokenStream> {
    let code = crate::render_template(scope, &|name| {
        if name == "continuation" {
            Ok(next.clone())
        } else {
            Err(format!("unknown composition parameter `{name}`"))
        }
    })?;
    flatten_tail(code)
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
