//! Conversion contracts and call assembly.
use crate::Result;
use proc_macro2::TokenStream;
use quote::quote;

/// One boundary parameter or result component used by wrapper emission.
///
/// ```
/// use prebindgen_flat::{Condition, Wire};
/// let wire = Wire {
///     component: Default::default(),
///     name: syn::parse_quote!(value),
///     kind: "i64",
///     active: Condition::default(),
/// };
/// assert_eq!(wire.kind, "i64");
/// ```
#[derive(Clone, Debug)]
pub struct Wire<W> {
    /// Logical component supplied by this wire; several wires may share a path.
    pub component: crate::ComponentPath,
    /// Generated identifier used by conversion code and wrapper signatures.
    pub name: syn::Ident,
    /// Adapter description of the carrier, supplied by a conversion.
    /// The framework carries it into the conversion without interpreting it.
    pub kind: W,
    /// Input selector path; outputs normally use the always-active default.
    pub active: Condition,
}

/// An input conversion split into call-wide validation, acquisition and construction.
///
/// Optional concrete storage implementing [`InputConversion`]. [`emit_call`] orders
/// them across all parameters; wrapper writers do not assemble them themselves.
#[derive(Clone, Debug)]
pub struct InputPlan<W> {
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
    /// Rust expression passed to the source function after `statements`.
    pub argument: TokenStream,
}
/// Output conversion whose owned temporaries survive until successful delivery.
///
/// Optional concrete storage implementing [`OutputConversion`].
/// [`emit_call`] commits only after adapter delivery succeeds.
#[derive(Clone, Debug)]
pub struct OutputPlan<W> {
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
/// A leaf is active only on its selected path. Adapters use the same condition
/// for foreign-side resource locking and native conversion code.
#[derive(Clone, Debug, Default)]
pub struct Condition(
    /// Conjunction of `(selector identifier, required arm index)` terms.
    /// An empty vector is always active.
    pub Vec<(syn::Ident, i32)>,
);
impl Condition {
    /// Return a new condition additionally requiring one selector arm.
    ///
    /// Does not modify the original condition; no additional runtime flag is generated.
    ///
    /// # Arguments
    ///
    /// * `tag` — identifier of the selector's index wire.
    /// * `index` — zero-based index of the required arm.
    pub fn arm(&self, tag: syn::Ident, index: i32) -> Self {
        let mut result = self.clone();
        result.0.push((tag, index));
        result
    }
    /// Emit this condition AND a Rust predicate, short-circuiting the predicate
    /// when the selector path is inactive. `predicate` is a Rust `bool` expression.
    pub fn and(&self, predicate: TokenStream) -> TokenStream {
        if self.0.is_empty() {
            predicate
        } else {
            let condition = self.rust();
            quote!((#condition) && (#predicate))
        }
    }
    /// Emit the conjunction as a Rust boolean expression; an empty condition emits `true`.
    pub fn rust(&self) -> TokenStream {
        self.0
            .iter()
            .map(|(tag, i)| quote!(#tag == #i))
            .reduce(|a, b| quote!((#a) && (#b)))
            .unwrap_or_else(|| quote!(true))
    }
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
    /// Path on which this resource is accessed; inactive resources do not conflict.
    pub active: Condition,
}
/// Emit pairwise alias checks across all active input resources.
///
/// `resources` holds the [`InputPlan::resources`] of all of one exported
/// function's parameters together, so that aliases across parameters are found.
///
/// Run this code after leaf validation and before **any** ownership acquisition.
/// Shared/shared pairs and mutually exclusive selector arms do not conflict.
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
            // Different alternatives of the same selector cannot conflict.
            if a.active
                .0
                .iter()
                .any(|(tag, arm)| b.active.0.iter().any(|(t, i)| t == tag && i != arm))
            {
                continue;
            }
            let mut active = a.active.clone();
            for term in &b.active.0 {
                if !active.0.contains(term) {
                    active.0.push(term.clone());
                }
            }
            let conflict = active.and(quote!((#x) == (#y)));
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

/// Identifier bound to the source function result by [`emit_call`].
/// Output conversion builders must use this identifier when constructing result
/// expressions. The name is reserved within the generated call body.
pub fn source_result() -> syn::Ident {
    syn::parse_quote!(__result)
}

/// Expression referring to the source result bound by [`emit_call`].
/// Use [`source_result`] instead when an identifier is needed for interpolation.
pub fn source_result_expr() -> TokenStream {
    let result = source_result();
    quote!(#result)
}

/// A fully prepared input conversion stored in a slot.
/// Metadata and generated names must stay consistent across method calls.
/// Code runs in the environment supplied by the language wrapper writer.
/// Preparation resolves errors before installing the slot; [`emit_call`]
/// reads this interface directly, without an intermediate plan.
pub trait InputConversion {
    /// Adapter-specific boundary description.
    type Wire;
    /// Boundary parameters in signature order.
    fn wires(&self) -> &[Wire<Self::Wire>];
    /// Shared call acknowledgement, after validation and before acquisition.
    /// All contributing inputs must return the same token spelling; assembly emits
    /// it once. Return `None` when this input needs no acknowledgement. Slot-local
    /// fallible checks belong in `validation`, not in this shared operation.
    fn acknowledgement(&self) -> Option<&TokenStream> {
        None
    }
    /// Resources used for call-wide alias checking.
    fn resources(&self) -> &[Resource];
    /// Fallible validation before any ownership transfer.
    fn validation(&self) -> TokenStream;
    /// Infallible ownership acquisition after acknowledgement.
    fn acquisition(&self) -> TokenStream;
    /// Construct the argument after all inputs have been acquired.
    fn construction(&self) -> TokenStream;
    /// Expression passed to the source function.
    fn argument(&self) -> TokenStream;
}
impl<W> InputConversion for InputPlan<W> {
    type Wire = W;
    fn wires(&self) -> &[Wire<Self::Wire>] {
        &self.wires
    }
    fn acknowledgement(&self) -> Option<&TokenStream> {
        self.acknowledgement.as_ref()
    }
    fn resources(&self) -> &[Resource] {
        &self.resources
    }
    fn validation(&self) -> TokenStream {
        self.validation.clone()
    }
    fn acquisition(&self) -> TokenStream {
        self.acquire.clone()
    }
    fn construction(&self) -> TokenStream {
        self.statements.clone()
    }
    fn argument(&self) -> TokenStream {
        self.argument.clone()
    }
}
/// A fully prepared output conversion stored in a slot.
/// Metadata and generated names must stay consistent across method calls.
/// Code runs in the environment supplied by the language wrapper writer.
/// Preparation resolves errors before installing the slot; [`emit_call`]
/// reads this interface directly, without an intermediate plan.
/// Source-value expressions use [`source_result`].
pub trait OutputConversion {
    /// Adapter-specific boundary description.
    type Wire;
    /// Boundary results in delivery order.
    fn wires(&self) -> &[Wire<Self::Wire>];
    /// Output expressions in wire order, valid until commit.
    fn values(&self) -> &[TokenStream];
    /// Prepare owned output temporaries before delivery.
    fn preparation(&self) -> TokenStream;
    /// Fallible delivery code; ownership remains uncommitted on failure.
    fn delivery(&self) -> TokenStream;
    /// Infallibly transfer ownership after successful delivery.
    fn commit(&self) -> TokenStream;
    /// Infallible wrapper result expression evaluated after commit.
    fn result(&self) -> TokenStream;
}
impl<W> OutputConversion for OutputPlan<W> {
    type Wire = W;
    fn wires(&self) -> &[Wire<Self::Wire>] {
        &self.wires
    }
    fn values(&self) -> &[TokenStream] {
        &self.values
    }
    fn preparation(&self) -> TokenStream {
        self.statements.clone()
    }
    fn delivery(&self) -> TokenStream {
        self.delivery.statements.clone()
    }
    fn commit(&self) -> TokenStream {
        self.commit.clone()
    }
    fn result(&self) -> TokenStream {
        self.delivery.result.clone()
    }
}

/// An output expression paired with its boundary description.
pub struct OutputValue<'a, W> {
    /// Resolved boundary component and logical mapping.
    pub wire: &'a Wire<W>,
    /// Expression ready for adapter-specific delivery.
    pub value: &'a TokenStream,
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
    /// Filled result slot, checked for matching wire and value counts.
    pub fn output_conversion(&self) -> Result<&O> {
        let output = self
            .ret
            .slot
            .as_ref()
            .ok_or_else(|| format!("`{}`: result has no conversion", self.name))?;
        if output.wires().len() != output.values().len() {
            return Err(format!("`{}`: output wire/value count differs", self.name));
        }
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
    pub fn outputs(&self) -> Result<impl ExactSizeIterator<Item = OutputValue<'_, O::Wire>>> {
        let output = self.output_conversion()?;
        Ok(output
            .wires()
            .iter()
            .zip(output.values())
            .map(|(wire, value)| OutputValue { wire, value }))
    }
}

/// Assemble a function call directly from filled slots, without a generator callback.
/// Validates slot completeness, output arity and agreement on shared acknowledgement.
/// The returned code runs inside the wrapper's fallible boundary, in this order:
/// all validation, alias checks, one acknowledgement, all acquisition, argument
/// construction, source invocation, output preparation, delivery and commit.
pub fn emit_call<I: InputConversion, O: OutputConversion>(
    function: &crate::Function<I, O>,
) -> Result<TokenStream> {
    let inputs = function.input_conversions()?;
    let output = function.output_conversion()?;
    let mut acknowledgement: Option<&TokenStream> = None;
    for fragment in inputs.iter().filter_map(|i| i.acknowledgement()) {
        if let Some(previous) = acknowledgement {
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
    let validation = inputs.iter().map(|i| i.validation());
    let resources: Vec<_> = inputs
        .iter()
        .flat_map(|i| i.resources().iter().cloned())
        .collect();
    let aliases = validate_resources(&resources);
    let acquire = inputs.iter().map(|i| i.acquisition());
    let construct = inputs.iter().map(|i| i.construction());
    let args = inputs.iter().map(|i| i.argument());
    let path = &function.path;
    let prepare = output.preparation();
    let deliver = output.delivery();
    let commit = output.commit();
    let result = output.result();
    let source = source_result();
    Ok(quote! {
        #(#validation)* #aliases #acknowledgement #(#acquire)* #(#construct)*
        let #source = #path(#(#args),*);
        #prepare #deliver #commit Ok(#result)
    })
}
