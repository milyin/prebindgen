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
/// Conversion implementers provide these phases. [`PreparedCall::emit`] orders
/// them across all parameters; wrapper writers do not assemble them themselves.
#[derive(Clone, Debug)]
pub struct InputPlan<W> {
    /// Explicit resolved representation, if this conversion exposes a named shape.
    pub representation: Option<crate::RepresentationId>,
    /// Flattened boundary parameters in signature order.
    pub wires: Vec<Wire<W>>,
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
/// Conversion implementers provide preparation and deferred commit code.
/// [`PreparedCall::emit`] commits only after adapter delivery succeeds.
#[derive(Clone, Debug)]
pub struct OutputPlan<W> {
    /// Explicit resolved representation, if this conversion exposes a named shape.
    pub representation: Option<crate::RepresentationId>,
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
pub fn validate_bindings<I: InputConversion, O: OutputConversion, R>(
    flat: &crate::Flat<I, O, R>,
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

/// Identifier bound to the source function result by [`PreparedCall::emit`].
/// Output conversion builders must use this identifier when constructing result
/// expressions. The name is reserved within the generated call body.
pub fn source_result() -> syn::Ident {
    syn::parse_quote!(__result)
}

/// Expression referring to the source result bound by [`PreparedCall::emit`].
/// Use [`source_result`] instead when an identifier is needed for interpolation.
pub fn source_result_expr() -> TokenStream {
    let result = source_result();
    quote!(#result)
}

/// Opaque input conversion stored in a parameter slot.
/// Implementations describe phases; wrapper writers use [`Flat::prepare_call`](crate::Flat::prepare_call).
pub trait InputConversion {
    /// Adapter-specific boundary description.
    type Wire;
    /// Resolve code and boundary inputs for this occurrence.
    fn emit(&self) -> Result<InputPlan<Self::Wire>>;
}

/// Opaque output conversion stored in a result slot.
/// Build source-value expressions with [`source_result`], the same identifier
/// bound by the call emitter.
pub trait OutputConversion {
    /// Adapter-specific boundary description.
    type Wire;
    /// Resolve code and boundary outputs for this occurrence.
    fn emit(&self) -> Result<OutputPlan<Self::Wire>>;
}
impl<W: Clone> InputConversion for InputPlan<W> {
    type Wire = W;
    fn emit(&self) -> Result<InputPlan<W>> {
        Ok(self.clone())
    }
}
impl<W: Clone> OutputConversion for OutputPlan<W> {
    type Wire = W;
    fn emit(&self) -> Result<OutputPlan<W>> {
        Ok(self.clone())
    }
}

/// Boundary signature and representation for one source parameter.
pub struct InputBoundary<'a, W> {
    /// Explicit shape selected during preparation.
    pub representation: Option<&'a crate::RepresentationId>,
    /// Boundary components belonging to this parameter, in signature order.
    pub wires: &'a [Wire<W>],
}

/// An output expression paired with its boundary description.
pub struct OutputValue<'a, W> {
    /// Resolved boundary component and logical mapping.
    pub wire: &'a Wire<W>,
    /// Expression ready for adapter-specific delivery.
    pub value: &'a TokenStream,
}

/// Adapter code inserted at fixed execution points by [`PreparedCall::emit`].
pub struct CallHooks {
    /// Fallible acknowledgement after validation, before infallible acquisition.
    pub acknowledge: TokenStream,
    /// Deliver all outputs. May fail with `?`; ownership has not been committed.
    pub deliver: TokenStream,
    /// Wrapper result expression, evaluated after commit; must be infallible.
    pub result: TokenStream,
}

impl Default for CallHooks {
    fn default() -> Self {
        Self {
            acknowledge: TokenStream::new(),
            deliver: TokenStream::new(),
            result: quote!(()),
        }
    }
}

/// Resolved call with opaque conversion phases and inspectable boundary mappings.
/// Created by [`Flat::prepare_call`](crate::Flat::prepare_call).
pub struct PreparedCall<I, O> {
    path: syn::Path,
    inputs: Vec<InputPlan<I>>,
    output: OutputPlan<O>,
}
impl<I, O> PreparedCall<I, O> {
    /// Preserve parameter grouping when mapping components to representation fields.
    pub fn parameters(&self) -> impl ExactSizeIterator<Item = InputBoundary<'_, I>> {
        self.inputs.iter().map(|input| InputBoundary {
            representation: input.representation.as_ref(),
            wires: &input.wires,
        })
    }
    /// Boundary inputs in function and component order.
    pub fn inputs(&self) -> impl Iterator<Item = &Wire<I>> {
        self.inputs.iter().flat_map(|input| &input.wires)
    }
    /// Boundary outputs paired with their delivery expressions.
    pub fn outputs(&self) -> impl ExactSizeIterator<Item = OutputValue<'_, O>> {
        self.output
            .wires
            .iter()
            .zip(&self.output.values)
            .map(|(wire, value)| OutputValue { wire, value })
    }
    /// Assemble validation, alias checks, acquisition, construction, source call,
    /// output preparation, delivery and commit in their required order.
    /// The returned statements belong inside the adapter's fallible boundary closure.
    pub fn emit(&self, hooks: CallHooks) -> TokenStream {
        let validation = self.inputs.iter().map(|i| &i.validation);
        let resources: Vec<_> = self
            .inputs
            .iter()
            .flat_map(|i| i.resources.iter().cloned())
            .collect();
        let aliases = validate_resources(&resources);
        let acquire = self.inputs.iter().map(|i| &i.acquire);
        let construct = self.inputs.iter().map(|i| &i.statements);
        let args = self.inputs.iter().map(|i| &i.argument);
        let path = &self.path;
        let prepare = &self.output.statements;
        let commit = &self.output.commit;
        let source = source_result();
        let CallHooks {
            acknowledge,
            deliver,
            result,
        } = hooks;
        quote! {
            #(#validation)* #aliases #acknowledge #(#acquire)* #(#construct)*
            let #source = #path(#(#args),*);
            #prepare #deliver #commit Ok(#result)
        }
    }
}
impl<I: InputConversion, O: OutputConversion> crate::Function<I, O> {
    /// Resolve all slots once. Missing slots and inconsistent output arity fail
    /// before a writer can emit a partial signature or call body.
    fn resolve_call(&self) -> Result<PreparedCall<I::Wire, O::Wire>> {
        let inputs = self
            .params
            .iter()
            .map(|p| {
                p.ty.slot
                    .as_ref()
                    .ok_or_else(|| {
                        format!("`{}`: parameter `{}` has no conversion", self.name, p.name)
                    })?
                    .emit()
            })
            .collect::<Result<Vec<_>>>()?;
        let output = self
            .ret
            .slot
            .as_ref()
            .ok_or_else(|| format!("`{}`: result has no conversion", self.name))?
            .emit()?;
        if output.wires.len() != output.values.len() {
            return Err(format!("`{}`: output wire/value count differs", self.name));
        }
        Ok(PreparedCall {
            path: self.path.clone(),
            inputs,
            output,
        })
    }
}

impl<I: InputConversion, O: OutputConversion, R> crate::Flat<I, O, R> {
    /// Prepare a selected function and validate its explicit representation mappings.
    /// Call [`Self::validate_representations`] once before a generation pass.
    /// This method checks only the selected call's mappings. Canonical defaults
    /// are never substituted during generation.
    pub fn prepare_call(&self, name: &str) -> Result<PreparedCall<I::Wire, O::Wire>> {
        let function = self.function(name)?;
        let call = function.resolve_call()?;
        for (parameter, input) in function.params.iter().zip(&call.inputs) {
            self.check_mapping(
                &parameter.ty.ty,
                input.representation.as_ref(),
                &input.wires,
            )?;
        }
        self.check_mapping(
            &function.ret.ty,
            call.output.representation.as_ref(),
            &call.output.wires,
        )?;
        Ok(call)
    }
    /// Validate all registered type shapes, including schemas not used by a function.
    pub fn validate_representations(&self) -> Result<()> {
        self.representations.validate()?;
        for (id, shape) in self.representations.iter() {
            self.resolve_type(&shape.source_type)?;
            for component in &shape.components {
                if let Some(fields) = &component.source_fields {
                    let mut ty = &shape.source_type;
                    for field in fields {
                        ty = &self
                            .record(ty)?
                            .fields
                            .iter()
                            .find(|f| f.name == field.as_str())
                            .ok_or_else(|| {
                                format!(
                                    "unknown source field `{field}` in representation `{}`",
                                    id.0
                                )
                            })?
                            .ty;
                    }
                    if let Some(child) = &component.representation {
                        if !crate::same_type(ty, &self.representations.get(child)?.source_type) {
                            return Err(format!(
                                "component `{}` has an incompatible source type",
                                component.name
                            ));
                        }
                    }
                }
            }
        }
        Ok(())
    }
    fn check_mapping<W>(
        &self,
        ty: &syn::Type,
        id: Option<&crate::RepresentationId>,
        wires: &[Wire<W>],
    ) -> Result<()> {
        let Some(id) = id else {
            return Ok(());
        };
        let shape = self.representations.get(id)?;
        if !crate::same_type(ty, &shape.source_type) {
            return Err(format!(
                "representation `{}` does not match occurrence type",
                id.0
            ));
        }
        for wire in wires {
            if !shape.components.iter().any(|c| c.path == wire.component) {
                return Err(format!(
                    "wire `{}` has no component in representation `{}`",
                    wire.name, id.0
                ));
            }
        }
        for component in &shape.components {
            if !wires.iter().any(|wire| wire.component == component.path) {
                return Err(format!(
                    "component `{}` has no wire in representation `{}`",
                    component.name, id.0
                ));
            }
        }
        Ok(())
    }
}
impl<I, O> PreparedCall<I, O> {
    /// Resolved representation of a source parameter, by its declaration index.
    pub fn input_representation(&self, index: usize) -> Option<&crate::RepresentationId> {
        self.inputs
            .get(index)
            .and_then(|input| input.representation.as_ref())
    }
    /// Resolved result representation.
    pub fn output_representation(&self) -> Option<&crate::RepresentationId> {
        self.output.representation.as_ref()
    }
}
