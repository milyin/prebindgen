//! Optional expression protocol for stored plans used by the C and JNI adapters.
//! Scoped requests keep prepared temporaries alive while evaluating a continuation.
//! Other generators can define entirely different request enums.
use crate::{Access, InputPlan, OutputPlan, Resource, ResourceIdentity, TemplateContext};
use prebindgen_flat::{Conversion, Function, Result, WireKind};
use proc_macro2::TokenStream;
use quote::quote;

/// Check whether function parameters conflict in their access to foreign allocations.
///
/// Include every parameter's access requirements during generation. Compiled
/// eager plans use [`Self::initialize`] before preparation and check each access
/// before acquiring it. Deferred custom plans use [`Self::emit`] after validation.
/// Shared-only calls need no bookkeeping; fixed sets use stack storage and
/// dynamic exclusive sets use native address history.
#[derive(Default)]
pub struct ParameterAliasCheck {
    resources: Vec<Resource>,
}
impl ParameterAliasCheck {
    /// Include the access requirements of one source parameter.
    ///
    /// Render its active conditions and identities using this parameter's
    /// generation context. These expressions remain guarded at runtime: an
    /// inactive resource's identity is never evaluated.
    pub fn include_parameter<C: TemplateContext>(
        &mut self,
        context: &C,
        resources: &[Resource],
    ) -> Result<()> {
        let rendered = resources
            .iter()
            .map(|resource| {
                Ok(Resource {
                    registered: resource.registered,
                    active: context.render(&resource.active)?,
                    access: resource.access,
                    identity: if resource.registered {
                        resource.identity.clone()
                    } else {
                        match &resource.identity {
                            ResourceIdentity::Single(code) => {
                                ResourceIdentity::Single(context.render(code)?)
                            }
                            ResourceIdentity::Many(code) => {
                                ResourceIdentity::Many(context.render(code)?)
                            }
                        }
                    },
                })
            })
            .collect::<Result<Vec<_>>>()?;
        self.resources.extend(rendered);
        Ok(())
    }

    /// Initialize the alias history before evaluating eager input validation.
    /// The caller also supplies `__prebindgen_input_consumed`, a shared Cell<usize>,
    /// for ownership receipts. Deferred resources require the separate emit path.
    pub fn initialize(&self) -> Result<TokenStream> {
        if self.resources.iter().any(|resource| !resource.registered) {
            return Err("eager input history requires eagerly prepared resources".into());
        }
        if self.resources.is_empty() {
            return Ok(TokenStream::new());
        }
        let fixed = self
            .resources
            .iter()
            .all(|r| matches!(r.identity, ResourceIdentity::Single(_)));
        let count = self.resources.len();
        Ok(
            if self.resources.iter().all(|r| r.access == Access::Shared) || fixed && count <= 1 {
                quote!(let mut __prebindgen_input_aliases=::prebindgen_runtime::SharedInputs;)
            } else if fixed {
                quote!(let mut __prebindgen_input_aliases=::prebindgen_runtime::FixedInputs::<#count>::default();)
            } else {
                quote!(let mut __prebindgen_input_aliases=::prebindgen_runtime::DynamicInputs::default();)
            },
        )
    }
    /// Emit a deferred plan's unit alias check; eager accesses are already checked.
    ///
    /// Fixed sets of at most eight addresses use pairwise comparisons, omitting
    /// shared/shared pairs. Larger fixed sets sort a stack array. A dynamic set
    /// with an exclusive access uses a vector and one sort. No helper state or
    /// configuration lookup is introduced into the generated runtime.
    pub fn emit(&self) -> Result<TokenStream> {
        if self.resources.iter().any(|r| r.registered)
            && self.resources.iter().any(|r| !r.registered)
        {
            return Err(
                "cannot mix deferred resource conversions with eagerly constructed inputs".into(),
            );
        }
        if self
            .resources
            .iter()
            .all(|r| r.registered || r.access == Access::Shared)
        {
            return Ok(quote!(()));
        }
        let fixed = self
            .resources
            .iter()
            .all(|r| matches!(r.identity, ResourceIdentity::Single(_)));
        if fixed && self.resources.len() < 2 {
            return Ok(quote!(()));
        }
        // Check locals live only inside the emitted expression. Reserve all
        // resource identifiers so they cannot shadow validation locals used by
        // another resource, including those belonging to a different parameter.
        let mut used = std::collections::BTreeSet::new();
        for resource in &self.resources {
            reserve(&resource.active, &mut used);
            match &resource.identity {
                ResourceIdentity::Single(code) | ResourceIdentity::Many(code) => {
                    reserve(code, &mut used)
                }
            }
        }
        let mut fresh = |base: String| {
            let mut name = base;
            while !used.insert(name.clone()) {
                name.push('_');
            }
            syn::Ident::new(&name, proc_macro2::Span::call_site())
        };
        let error = quote!(return Err("conflicting input aliases".into()););
        if fixed {
            let addresses = self
                .resources
                .iter()
                .map(|resource| {
                    let ResourceIdentity::Single(identity) = &resource.identity else {
                        unreachable!()
                    };
                    let active = &resource.active;
                    quote!(if #active { Some((#identity) as usize) } else { None })
                })
                .collect::<Vec<_>>();
            if self.resources.len() <= 8 {
                let names = (0..self.resources.len())
                    .map(|i| fresh(format!("__prebindgen_alias_{i}")))
                    .collect::<Vec<_>>();
                let bindings = names
                    .iter()
                    .zip(addresses)
                    .map(|(name, address)| quote!(let #name = #address;));
                let mut conflicts = Vec::new();
                for (i, a) in self.resources.iter().enumerate() {
                    for (j, b) in self.resources.iter().enumerate().skip(i + 1) {
                        if a.access == Access::Shared && b.access == Access::Shared {
                            continue;
                        }
                        let left = &names[i];
                        let right = &names[j];
                        conflicts.push(quote!(#left.is_some() && #left == #right));
                    }
                }
                return expression(quote!({ #(#bindings)* if #(#conflicts)||* { #error } }));
            }
            let entries = addresses
                .into_iter()
                .zip(&self.resources)
                .map(|(address, resource)| {
                    let exclusive = resource.access != Access::Shared;
                    quote!((#address, #exclusive))
                });
            let seen = fresh("__prebindgen_resources".into());
            return expression(quote!({
                let mut #seen = [#(#entries),*];
                #seen.sort_unstable();
                if #seen.windows(2).any(|pair| pair[0].0.is_some()
                    && pair[0].0 == pair[1].0 && (pair[0].1 || pair[1].1)) {
                    #error
                }
            }));
        }
        let seen = fresh("__prebindgen_resources".into());
        let mut collect = TokenStream::new();
        for resource in &self.resources {
            let active = &resource.active;
            let exclusive = resource.access != Access::Shared;
            let identities = match &resource.identity {
                ResourceIdentity::Single(code) => quote!(::core::iter::once((#code) as usize)),
                ResourceIdentity::Many(code) => code.clone(),
            };
            collect.extend(quote!(if #active { for identity in #identities { #seen.push((identity, #exclusive)); } }));
        }
        expression(quote!({
            let mut #seen: Vec<(usize, bool)> = Vec::new();
            #collect
            #seen.sort_unstable();
            if #seen.windows(2).any(|pair| pair[0].0 == pair[1].0 && (pair[0].1 || pair[1].1)) {
                #error
            }
        }))
    }
}

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
    /// Include this parameter in a call-wide alias check during generation.
    /// The standard plan returns `()`; emit `check.emit()` after validation and
    /// before acknowledgement/acquisition. Custom conversions may also return
    /// additional runtime checks as a unit expression.
    CheckAliases {
        /// Bindings used to render prepared templates.
        context: &'a C,
        /// Checks shared by all parameters of this call.
        check: &'a mut ParameterAliasCheck,
    },
    /// Check aliases within an isolated input conversion, such as a callback result.
    /// Deferred plans emit their isolated check. Eager plans have already checked
    /// their accesses during validation: initialize their history and consumed
    /// counter before validation, as the callback return emitters do.
    CheckInternalAliases {
        /// Bindings used to render prepared templates.
        context: &'a C,
    },
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
            InputFragment::CheckAliases { context, check } => {
                check.include_parameter(context, &self.resources)?;
                quote!(())
            }
            InputFragment::CheckInternalAliases { context } => {
                let mut check = ParameterAliasCheck::default();
                check.include_parameter(context, &self.resources)?;
                check.emit()?
            }
        })
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
}
/// Compose the opt-in staged protocol as a fallible expression.
/// `result_type` is the wrapper result type inside `Result<_, String>`.
///
/// Generators choosing this helper use address alias checks and one shared
/// acknowledgement. Other generators can compose their own request protocols.
/// This is deliberately outside Flat: none of these phases are required by its traits.
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
    let mut used = parameter_names
        .iter()
        .map(ToString::to_string)
        .collect::<std::collections::BTreeSet<_>>();
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
    let args: Vec<_> = (0..inputs.len())
        .map(|i| fresh(format!("__prebindgen_argument_{i}")))
        .collect();
    let mut check = ParameterAliasCheck::default();
    let mut checks = inputs
        .iter()
        .map(|input| {
            input.code_fragment_slot(
                InputFragment::CheckAliases {
                    context,
                    check: &mut check,
                },
                &unit,
            )
        })
        .collect::<Result<Vec<_>>>()?;
    let eager = check.resources.iter().any(|r| r.registered);
    if eager && check.resources.iter().any(|r| !r.registered) {
        return Err("cannot mix deferred resource conversions with eagerly constructed inputs; prepare all resource-bearing inputs through the declaration compiler".into());
    }
    let early_receipt = eager
        || (acknowledge.is_some() && check.resources.iter().any(|r| r.access == Access::Move));
    let setup = if early_receipt {
        let counter = context_counter(context)?;
        let aliases = if eager {
            check.initialize()?
        } else {
            TokenStream::new()
        };
        quote!(let __prebindgen_input_consumed=#counter; #aliases)
    } else {
        TokenStream::new()
    };
    if !eager {
        checks.push(check.emit()?);
    }
    checks.retain(|code| code.to_string() != quote!(()).to_string());
    let alias_checks = quote!(#(let _: () = #checks;)*);
    let legacy_count = if !eager && early_receipt {
        let counts = check
            .resources
            .iter()
            .filter(|r| r.access == Access::Move)
            .map(|r| {
                let active = &r.active;
                let count = match &r.identity {
                    ResourceIdentity::Single(_) => quote!(1usize),
                    ResourceIdentity::Many(ids) => quote!((#ids).count()),
                };
                quote!(if #active {#count} else {0usize})
            });
        quote!(__prebindgen_input_consumed.set(__prebindgen_input_consumed.get() #(+ (#counts))*);)
    } else {
        TokenStream::new()
    };
    let delivered = plug(&deliver, quote!({let _: () = #commit; Ok(#result)}))?;
    let mut body = plug(&prepare, delivered)?;
    body = scope(quote!(let #source=#path(#(#args),*);), body)?;
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
    for fragment in acquire.iter().rev() {
        body = plug(fragment, body)?;
    }
    body = scope(legacy_count, body)?;
    if !eager {
        if let Some(ack) = &acknowledge {
            body = scope(quote!(let _: () = #ack;), body)?;
        }
    }
    body = scope(alias_checks, body)?;
    for fragment in validation.iter().rev() {
        body = plug(fragment, body)?;
    }
    if eager {
        if let Some(ack) = acknowledge {
            body = scope(quote!(let _: () = #ack;), body)?;
        }
    }
    body = scope(setup, body)?;
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

fn context_counter(context: &impl TemplateContext) -> Result<TokenStream> {
    context.render(&crate::template_parameter("input_consumed"))
}
