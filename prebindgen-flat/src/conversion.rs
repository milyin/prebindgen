//! Conversion contracts and call assembly.
use crate::{template_parameter, Result, TemplateContext};
use proc_macro2::TokenStream;
use quote::quote;

/// Transport metadata that supplies the Rust carrier type of a wire.
/// The returned type describes the boundary value, not the source value being converted.
/// For example, a boolean conversion may use `u8` as its Rust carrier type.
///
/// ```
/// use prebindgen_flat::WireKind;
/// struct BoolCarrier;
/// impl WireKind for BoolCarrier {
///     fn rust_type(&self) -> syn::Type { syn::parse_quote!(u8) }
/// }
/// let ty = BoolCarrier.rust_type();
/// assert_eq!(quote::quote!(#ty).to_string(), "u8");
/// ```
pub trait WireKind {
    /// Rust type used to declare this wire in a wrapper signature or result structure.
    fn rust_type(&self) -> syn::Type;
}
impl WireKind for syn::Type {
    fn rust_type(&self) -> syn::Type {
        self.clone()
    }
}

/// One boundary parameter or result component used by wrapper emission.
///
/// `W` must implement [`WireKind`]. Call [`Wire::rust_type`]
/// to obtain the Rust carrier type supplied by the kind.
///
/// ```
/// use prebindgen_flat::Wire;
/// let wire: Wire<syn::Type> = Wire {
///     name: syn::parse_quote!(value),
///     kind: syn::parse_quote!(i64),
/// };
/// let ty = wire.rust_type();
/// assert_eq!(quote::quote!(#ty).to_string(), "i64");
/// ```
/// A wire kind without a Rust carrier type is rejected:
///
/// ```compile_fail
/// use prebindgen_flat::Wire;
/// struct Untyped;
/// let wire = Wire { name: syn::parse_quote!(value), kind: Untyped };
/// ```
#[derive(Clone, Debug)]
pub struct Wire<W: WireKind> {
    /// Generated identifier used by conversion code and wrapper signatures.
    pub name: syn::Ident,
    /// Transport description implementing [`WireKind`].
    /// Logical fields, selection rules and ownership behavior belong inside the converter.
    pub kind: W,
}

impl<W: WireKind> Wire<W> {
    /// Rust carrier type supplied by this wire's [`WireKind`].
    /// Used to declare a wrapper parameter or result field.
    pub fn rust_type(&self) -> syn::Type {
        self.kind.rust_type()
    }
}

/// An input conversion split into call-wide validation, acquisition and construction.
///
/// Optional concrete storage implementing [`InputConversion`]. [`emit_call`] orders
/// them across all parameters; wrapper writers do not assemble them themselves.
#[derive(Clone, Debug)]
pub struct InputPlan<W: WireKind, C = ()> {
    /// Context type supplied by the language generator at emission.
    pub context: std::marker::PhantomData<fn(&C)>,
    /// Flattened boundary parameters in signature order.
    pub wires: Vec<Wire<W>>,
    /// Shared call acknowledgement, if this input needs one.
    pub acknowledgement: Option<TokenStream>,
    /// Validate and prepare every active leaf before any ownership transfer.
    pub validation: TokenStream,
    /// Infallible acquisition of all owned inputs, before constructors can run.
    pub acquire: TokenStream,
    /// Active foreign resources whose identities must be checked across all arguments.
    pub resources: Vec<Resource>,
    /// Argument construction after all acquisitions, including source constructor calls.
    pub statements: TokenStream,
    /// Prepared value expression assigned to the caller-supplied target after `statements`.
    pub argument: TokenStream,
}
/// Output conversion whose owned temporaries survive until successful delivery.
///
/// Optional concrete storage implementing [`OutputConversion`].
/// [`emit_call`] commits only after adapter delivery succeeds.
#[derive(Clone, Debug)]
pub struct OutputPlan<W: WireKind, C = ()> {
    /// Context type supplied by the language generator at emission.
    pub context: std::marker::PhantomData<fn(&C)>,
    /// Delivery and return code prepared with this result slot.
    pub delivery: Delivery,
    /// Boundary results in delivery order; one entry per expression in `values`.
    pub wires: Vec<Wire<W>>,
    /// Transfer ownership only after all foreign delivery operations succeed.
    pub commit: TokenStream,
    /// Code producing owned output temporaries and evaluating observations before moves.
    pub statements: TokenStream,
    /// Delivery expressions in wire order; they must remain valid until commit.
    pub values: Vec<TokenStream>,
}
/// Access to one foreign resource during an exported call.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Access {
    /// Read-only borrow; multiple active shared accesses may alias.
    Shared,
    /// Mutable borrow; conflicts with every other active use of the resource.
    Exclusive,
    /// Ownership transfer; conflicts with every other active use of the resource.
    Move,
}
/// The adapter supplies identity; flat enforces call-wide access conflicts.
#[derive(Clone, Debug)]
pub struct Resource {
    /// Side-effect-free Rust expression identifying a live resource, such as an address.
    /// All identities compared within a call must share an equality-compatible domain.
    pub identity: TokenStream,
    /// Borrow or ownership requirement imposed by this input conversion.
    pub access: Access,
    /// Side-effect-free Rust boolean expression; false resources do not conflict.
    pub active: TokenStream,
}
/// Emit pairwise alias checks across all active input resources.
///
/// `resources` holds the [`InputPlan::resources`] of all of one exported
/// function's parameters together, so that aliases across parameters are found.
///
/// Run this code after leaf validation and before **any** ownership acquisition.
/// Shared/shared pairs and resources whose predicates are not both true do not conflict.
/// Conflicts return `Err("conflicting input aliases".into())` from the enclosing
/// wrapper. This does not validate pointer liveness, synchronize threads, or
/// acquire resources; those remain adapter/caller responsibilities.
pub fn validate_resources(resources: &[Resource]) -> TokenStream {
    let mut result = TokenStream::new();
    for (i, a) in resources.iter().enumerate() {
        for b in &resources[i + 1..] {
            if a.access == Access::Shared && b.access == Access::Shared {
                continue;
            }
            let x = &a.identity;
            let y = &b.identity;
            let active_a = &a.active;
            let active_b = &b.active;
            let conflict = quote!((#active_a) && (#active_b) && ((#x) == (#y)));
            result.extend(quote!(if #conflict {return Err("conflicting input aliases".into());}));
        }
    }
    result
}

/// Check that every bound function has an input converter for each parameter
/// and an output converter, including unit results.
///
/// Functions with entirely empty slots remain source-only helpers. This check
/// neither builds conversions nor changes the model. Adapters call it before
/// writing wrappers. It does not validate code fragments supplied directly by callers.
pub fn validate_bindings<I: InputConversion, O: OutputConversion>(
    flat: &crate::Flat<I, O>,
) -> Result<()> {
    for f in flat.functions.values() {
        if f.ret.slot.is_some() {
            for p in &f.params {
                if p.ty.slot.is_none() {
                    return Err(format!(
                        "`{}`: parameter `{}` has no input converter",
                        f.name, p.name
                    ));
                }
            }
        } else if f.params.iter().any(|p| p.ty.slot.is_some()) {
            return Err(format!(
                "`{}`: bound inputs have no output converter",
                f.name
            ));
        }
    }
    Ok(())
}
/// Format generated tokens as a Rust source file.
///
/// `tokens` must be a sequence of Rust items. Returns a syntax error if they do not form a file. This does not
/// resolve names or type-check generated code; the binding crate must compile it.
pub fn rust_source(tokens: TokenStream) -> Result<String> {
    let file = syn::parse2(tokens).map_err(|e| format!("generated Rust: {e}"))?;
    Ok(prettyplease::unparse(&file))
}

/// Explicit template reference to the caller-selected source-result local.
/// [`OutputPlan`] resolves it from the explicit `source` argument before
/// calling [`TemplateContext::render`] for generator-specific parameters.
pub fn source_result_expr() -> TokenStream {
    template_parameter("source_result")
}

/// A fully prepared input conversion stored in a slot.
/// Metadata and generated names must stay consistent across method calls.
/// Code runs in the environment supplied by the language wrapper writer.
/// Preparation resolves errors before installing the slot; [`emit_call`]
/// reads this interface directly, without an intermediate plan.
///
/// This conversion accepts one opaque pointer wire and constructs a
/// `Box<T>` argument. Preparation supplies the concrete pointee type as a
/// `syn::Type`; the converter does not hardcode any wrapped type.
/// It demonstrates every method, including ownership transfer and alias checking.
///
/// The wrapper supplies an `&mut bool` acknowledgement parameter, initially false.
/// Validation or alias-check failure leaves it false and ownership with the caller.
/// After acknowledgement, the wrapper owns the allocation even if later code fails.
/// The pointer must come from a compatible `Box::into_raw` for the prepared
/// pointee type, remain live, and be used exclusively for this transfer.
/// A null check cannot establish those conditions.
///
/// ```
/// use prebindgen_flat::{Access, InputConversion, Resource, Result, Wire, WireKind};
/// use proc_macro2::TokenStream;
/// use quote::quote;
/// use syn::{parse_quote, Ident, Type};
///
/// // The generator defines boundary carriers independently of the wrapped types.
/// // Any opaque object handle can use this same pointer carrier.
/// enum WireType {
///     OpaquePointer,
/// }
/// impl WireKind for WireType {
///     fn rust_type(&self) -> Type {
///         match self {
///             Self::OpaquePointer => parse_quote!(*mut ::std::ffi::c_void),
///         }
///     }
/// }
///
/// // Name of the wrapper's &mut bool parameter, supplied during generation.
/// struct Context {
///     acknowledged: Ident,
/// }
///
/// // A reusable converter prepared for any supported sized pointee type.
/// struct OwnedBoxInput {
///     pointee: Type,
///     wires: [Wire<WireType>; 1],
///     // Preparation chooses a private temporary unique within the wrapper.
///     owned: Ident,
/// }
/// impl InputConversion for OwnedBoxInput {
///     type Wire = WireType;
///     type Context = Context;
///
///     fn wires(&self) -> &[Wire<WireType>] {
///         &self.wires
///     }
///
///     fn resources(&self, _: &Context) -> Result<Vec<Resource>> {
///         let wire = &self.wires[0].name;
///         Ok(vec![Resource {
///             identity: quote!(#wire as usize),
///             access: Access::Move,
///             active: quote!(true),
///         }])
///     }
///
///     fn validation(&self, _: &Context) -> Result<TokenStream> {
///         let wire = &self.wires[0].name;
///         Ok(quote! {
///             if #wire.is_null() {
///                 return Err("null owned handle".into());
///             }
///         })
///     }
///
///     fn acknowledgement(&self, context: &Context) -> Result<Option<TokenStream>> {
///         let acknowledged = &context.acknowledged;
///         // Shared across consuming slots; emit_call emits it once, after checks.
///         Ok(Some(quote!(*#acknowledged = true;)))
///     }
///
///     fn acquisition(&self, _: &Context) -> Result<TokenStream> {
///         let wire = &self.wires[0].name;
///         let owned = &self.owned;
///         let pointee = &self.pointee;
///         // Safety relies on the caller's pointer and ownership contract above.
///         Ok(quote! {
///             let #owned = unsafe { ::std::boxed::Box::<#pointee>::from_raw(#wire.cast::<#pointee>()) };
///         })
///     }
///
///     fn construction(&self, _: &Context, target: &Ident) -> Result<TokenStream> {
///         let owned = &self.owned;
///         Ok(quote!(let #target = #owned;))
///     }
/// }
///
/// // Preparation obtains this type from the wrapped signature. Use u64 here
/// // to demonstrate the converter without introducing an application type.
/// let slot = OwnedBoxInput {
///     pointee: parse_quote!(u64),
///     wires: [Wire {
///         name: parse_quote!(raw_value),
///         kind: WireType::OpaquePointer,
///     }],
///     owned: parse_quote!(__owned_value),
/// };
/// let context = Context { acknowledged: parse_quote!(ownership_taken) };
///
/// // Individual phases are available directly. emit_call orders them across
/// // all input slots, checks resource aliases, and invokes the source function.
/// assert_eq!(slot.wires().len(), 1);
/// assert_eq!(slot.resources(&context)?[0].access, Access::Move);
/// let validation = slot.validation(&context)?;
/// let acknowledgement = slot.acknowledgement(&context)?;
/// let acquisition = slot.acquisition(&context)?;
/// let target = parse_quote!(argument);
/// let construction = slot.construction(&context, &target)?;
/// assert_eq!(construction.to_string(),
///            quote!(let argument = __owned_value;).to_string());
///
/// // Inspect the emitted phases without compiling or executing a foreign pointer.
/// let phases = quote! {
///     #validation
///     #acknowledgement
///     #acquisition
///     #construction
/// };
/// let _: syn::Block = syn::parse2(quote!({ #phases }))?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// The converter owns any conditional behavior. Wires describe transport only.
pub trait InputConversion {
    /// Adapter-specific boundary description.
    type Wire: WireKind;
    /// Emission data defined and supplied by the language generator.
    type Context;
    /// Boundary parameters in signature order.
    fn wires(&self) -> &[Wire<Self::Wire>];
    /// Shared call acknowledgement, after validation and before acquisition.
    /// All contributing inputs must return the same token spelling; assembly emits
    /// it once. Return `None` when this input needs no acknowledgement. Slot-local
    /// fallible checks belong in `validation`, not in this shared operation.
    fn acknowledgement(&self, _context: &Self::Context) -> Result<Option<TokenStream>> {
        Ok(None)
    }
    /// Resources used for call-wide alias checking, with identities emitted for this context.
    fn resources(&self, context: &Self::Context) -> Result<Vec<Resource>>;
    /// Fallible validation before any ownership transfer.
    fn validation(&self, context: &Self::Context) -> Result<TokenStream>;
    /// Infallible ownership acquisition after acknowledgement.
    fn acquisition(&self, context: &Self::Context) -> Result<TokenStream>;
    /// Construct the argument after all inputs have been acquired, binding it to `target`.
    /// The caller owns this identifier and passes it to the source function. Emit a
    /// `let` binding in the surrounding scope, so borrowed argument storage remains alive.
    /// Converter-private temporaries must not shadow other caller-supplied bindings.
    fn construction(&self, context: &Self::Context, target: &syn::Ident) -> Result<TokenStream>;
}
impl<W: WireKind, C: TemplateContext> InputConversion for InputPlan<W, C> {
    type Wire = W;
    type Context = C;
    fn wires(&self) -> &[Wire<Self::Wire>] {
        &self.wires
    }
    fn acknowledgement(&self, context: &Self::Context) -> Result<Option<TokenStream>> {
        self.acknowledgement
            .as_ref()
            .map(|code| context.render(code))
            .transpose()
    }
    fn resources(&self, context: &Self::Context) -> Result<Vec<Resource>> {
        self.resources
            .iter()
            .map(|resource| {
                Ok(Resource {
                    identity: context.render(&resource.identity)?,
                    access: resource.access,
                    active: context.render(&resource.active)?,
                })
            })
            .collect()
    }
    fn validation(&self, context: &Self::Context) -> Result<TokenStream> {
        context.render(&self.validation)
    }
    fn acquisition(&self, context: &Self::Context) -> Result<TokenStream> {
        context.render(&self.acquire)
    }
    fn construction(&self, context: &Self::Context, target: &syn::Ident) -> Result<TokenStream> {
        let statements = context.render(&self.statements)?;
        let value = context.render(&self.argument)?;
        Ok(quote!(#statements let #target = #value;))
    }
}
/// A fully prepared output conversion stored in a slot.
/// Metadata and generated names must stay consistent across method calls.
/// Code runs in the environment supplied by the language wrapper writer.
/// Preparation resolves errors before installing the slot; [`emit_call`]
/// reads this interface directly, without an intermediate plan.
/// Source-value templates use [`source_result_expr`]; custom implementations can
/// use the explicitly supplied source binding directly. All output phases receive
/// the same binding, chosen by the caller, for the source function's return value.
pub trait OutputConversion {
    /// Adapter-specific boundary description.
    type Wire: WireKind;
    /// Emission data defined and supplied by the language generator.
    type Context;
    /// Boundary results in delivery order.
    fn wires(&self) -> &[Wire<Self::Wire>];
    /// Output expressions in wire order, valid until commit.
    fn values(&self, context: &Self::Context, source: &syn::Ident) -> Result<Vec<TokenStream>>;
    /// Prepare owned output temporaries before delivery.
    fn preparation(&self, context: &Self::Context, source: &syn::Ident) -> Result<TokenStream>;
    /// Fallible delivery code; ownership remains uncommitted on failure.
    fn delivery(&self, context: &Self::Context, source: &syn::Ident) -> Result<TokenStream>;
    /// Infallibly transfer ownership after successful delivery.
    fn commit(&self, context: &Self::Context, source: &syn::Ident) -> Result<TokenStream>;
    /// Infallible wrapper result expression evaluated after commit.
    fn result(&self, context: &Self::Context, source: &syn::Ident) -> Result<TokenStream>;
}
impl<W: WireKind, C: TemplateContext> OutputConversion for OutputPlan<W, C> {
    type Wire = W;
    type Context = C;
    fn wires(&self) -> &[Wire<Self::Wire>] {
        &self.wires
    }
    fn values(&self, context: &Self::Context, source: &syn::Ident) -> Result<Vec<TokenStream>> {
        self.values
            .iter()
            .map(|code| crate::emission::render_output(context, code, source))
            .collect()
    }
    fn preparation(&self, context: &Self::Context, source: &syn::Ident) -> Result<TokenStream> {
        crate::emission::render_output(context, &self.statements, source)
    }
    fn delivery(&self, context: &Self::Context, source: &syn::Ident) -> Result<TokenStream> {
        crate::emission::render_output(context, &self.delivery.statements, source)
    }
    fn commit(&self, context: &Self::Context, source: &syn::Ident) -> Result<TokenStream> {
        crate::emission::render_output(context, &self.commit, source)
    }
    fn result(&self, context: &Self::Context, source: &syn::Ident) -> Result<TokenStream> {
        crate::emission::render_output(context, &self.delivery.result, source)
    }
}

/// An output expression paired with its boundary description.
pub struct OutputValue<'a, W: WireKind> {
    /// Boundary carrier and binding name; its meaning is private to the converter.
    pub wire: &'a Wire<W>,
    /// Expression ready for adapter-specific delivery.
    pub value: TokenStream,
}

/// Result-slot glue executed after output preparation and before ownership commit.
#[derive(Clone, Debug)]
pub struct Delivery {
    /// Deliver every output; failure must leave owned temporaries uncommitted.
    pub statements: TokenStream,
    /// Infallible wrapper result expression evaluated after commit.
    pub result: TokenStream,
}
impl Default for Delivery {
    fn default() -> Self {
        Self {
            statements: TokenStream::new(),
            result: quote!(()),
        }
    }
}

impl<I: InputConversion, O: OutputConversion> crate::Function<I, O> {
    /// Filled input slots, preserving source parameter order and grouping.
    pub fn input_conversions(&self) -> Result<Vec<&I>> {
        self.params
            .iter()
            .map(|p| {
                p.ty.slot.as_ref().ok_or_else(|| {
                    format!("`{}`: parameter `{}` has no conversion", self.name, p.name)
                })
            })
            .collect()
    }
    /// Filled result slot. Emission checks the rendered wire and value counts.
    pub fn output_conversion(&self) -> Result<&O> {
        let output = self
            .ret
            .slot
            .as_ref()
            .ok_or_else(|| format!("`{}`: result has no conversion", self.name))?;
        Ok(output)
    }
    /// Boundary input wires read directly from the slots in parameter order.
    pub fn inputs(&self) -> Result<impl Iterator<Item = &Wire<I::Wire>>> {
        Ok(self
            .input_conversions()?
            .into_iter()
            .flat_map(|i| i.wires()))
    }
    /// Output expressions paired with their wire descriptions, directly from the slot.
    pub fn outputs(
        &self,
        context: &O::Context,
        source: &syn::Ident,
    ) -> Result<impl ExactSizeIterator<Item = OutputValue<'_, O::Wire>>> {
        let output = self.output_conversion()?;
        let values = output.values(context, source)?;
        if output.wires().len() != values.len() {
            return Err(format!("`{}`: output wire/value count differs", self.name));
        }
        Ok(output
            .wires()
            .iter()
            .zip(values)
            .map(|(wire, value)| OutputValue { wire, value }))
    }
}

/// Assemble a function call directly from filled slots, without a generator callback.
/// Validates slot completeness, output arity and agreement on shared acknowledgement.
/// The returned code runs inside the wrapper's fallible boundary, in this order:
/// all validation, alias checks, one acknowledgement, all acquisition, argument
/// construction, source invocation, output preparation, delivery and commit.
/// `source` names the variable that receives the source function's return value.
/// Input and output contexts must match, even when the function has no parameters.
///
/// ```compile_fail
/// use prebindgen_flat::{emit_call, Function, InputConversion, OutputConversion};
/// struct OtherContext(syn::Ident);
/// fn incompatible<I, O>(function: &Function<I, O>, context: &syn::Ident)
/// where I: InputConversion<Context = syn::Ident>,
///       O: OutputConversion<Context = OtherContext>,
/// {
///     emit_call(function, context, &syn::parse_quote!(returned)).unwrap();
/// }
/// ```
pub fn emit_call<I: InputConversion, O: OutputConversion<Context = I::Context>>(
    function: &crate::Function<I, O>,
    context: &I::Context,
    source: &syn::Ident,
) -> Result<TokenStream> {
    let inputs = function.input_conversions()?;
    let output = function.output_conversion()?;
    let _ = function.outputs(context, source)?;
    let mut acknowledgement: Option<TokenStream> = None;
    for input in &inputs {
        let Some(fragment) = input.acknowledgement(context)? else {
            continue;
        };
        if let Some(previous) = &acknowledgement {
            if previous.to_string() != fragment.to_string() {
                return Err(format!(
                    "`{}`: conflicting input acknowledgements",
                    function.name
                ));
            }
        } else {
            acknowledgement = Some(fragment);
        }
    }
    let validation = inputs
        .iter()
        .map(|i| i.validation(context))
        .collect::<Result<Vec<_>>>()?;
    let resources: Vec<_> = inputs
        .iter()
        .map(|i| i.resources(context))
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .flatten()
        .collect();
    let aliases = validate_resources(&resources);
    let acquire = inputs
        .iter()
        .map(|i| i.acquisition(context))
        .collect::<Result<Vec<_>>>()?;
    let path = &function.path;
    let prepare = output.preparation(context, source)?;
    let deliver = output.delivery(context, source)?;
    let commit = output.commit(context, source)?;
    let result = output.result(context, source)?;
    let mut used: std::collections::BTreeSet<String> = inputs
        .iter()
        .flat_map(|input| input.wires().iter().map(|wire| wire.name.to_string()))
        .chain(std::iter::once(source.to_string()))
        .collect();
    // Reserve names already used by surrounding phases and the source path.
    // These tokens are emitted once; their code and evaluation order stay unchanged.
    reserve_identifiers(
        &quote!(#path #(#validation)* #aliases #acknowledgement #(#acquire)*
        #prepare #deliver #commit #result),
        &mut used,
    );
    let args: Vec<_> = (0..inputs.len())
        .map(|index| {
            let mut name = format!("__prebindgen_argument_{index}");
            while !used.insert(name.clone()) {
                name.push('_');
            }
            syn::Ident::new(&name, proc_macro2::Span::call_site())
        })
        .collect();
    let construct = inputs
        .iter()
        .zip(&args)
        .map(|(input, target)| input.construction(context, target))
        .collect::<Result<Vec<_>>>()?;
    Ok(quote! {
        #(#validation)* #aliases #acknowledgement #(#acquire)* #(#construct)*
        let #source = #path(#(#args),*);
        #prepare #deliver #commit Ok(#result)
    })
}

fn reserve_identifiers(code: &TokenStream, used: &mut std::collections::BTreeSet<String>) {
    for token in code.clone() {
        match token {
            proc_macro2::TokenTree::Ident(name) => {
                used.insert(name.to_string());
            }
            proc_macro2::TokenTree::Group(group) => reserve_identifiers(&group.stream(), used),
            _ => {}
        }
    }
}
