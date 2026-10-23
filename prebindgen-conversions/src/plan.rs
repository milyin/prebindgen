use crate::template_parameter;
use prebindgen_flat::WireKind;
use prebindgen_flat::WireLayout;
use proc_macro2::TokenStream;
use quote::quote;
/// An input conversion split into call-wide validation, acquisition and construction.
///
/// Optional storage for the staged request protocol in this crate.
/// Generators may use this protocol or implement their own request types.
pub struct InputPlan<W: WireKind, C = (), G = ()> {
    /// Whether validation or construction can return a binding error. This
    /// excludes panics and allocation aborts. Composition preserves child flags.
    pub fallible: bool,
    /// Context type supplied by the language generator at emission.
    pub context: std::marker::PhantomData<fn(&C)>,
    /// Authoritative value tree. Adapter metadata records native encoding choices;
    /// physical parameters are derived from this tree when the generator emits its signature.
    pub layout: WireLayout<W, G>,
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
/// Optional concrete storage implementing [`prebindgen_flat::Conversion`].
/// Delivery and ownership commit are separate requests.
pub struct OutputPlan<W: WireKind, C = (), G = ()> {
    /// Whether preparation or delivery can return a binding error, excluding
    /// panics and allocation aborts. Adapters must mark custom fallible code.
    pub fallible: bool,
    /// Context type supplied by the language generator at emission.
    pub context: std::marker::PhantomData<fn(&C)>,
    /// Delivery and return code prepared with this result slot.
    pub delivery: Delivery,
    /// Authoritative success/error value tree. Adapter metadata on its nodes
    /// records explicit native encodings without duplicating payload subtrees.
    pub layout: WireLayout<W, G>,
    /// Complete transfers for the selected protocol. Buffered outputs commit after
    /// foreign delivery succeeds; explicit visitor protocols can commit each
    /// element at callback entry during preparation.
    pub commit: TokenStream,
    /// Code producing owned output temporaries and evaluating observations before moves.
    pub statements: TokenStream,
    /// Prepared success expressions keyed by their destination binding names.
    /// Adapters choose packet/row order from the tree and match these by name.
    pub values: Vec<(syn::Ident, TokenStream)>,
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
/// One allocation address, or a runtime-sized set of addresses with the same access mode.
#[derive(Clone, Debug)]
pub enum ResourceIdentity {
    /// Expression evaluating to one allocation address.
    Single(TokenStream),
    /// Expression implementing IntoIterator<Item = usize>.
    Many(TokenStream),
}
/// The adapter supplies identity; flat enforces call-wide access conflicts.
#[derive(Clone, Debug)]
pub struct Resource {
    /// Side-effect-free Rust expression identifying a live resource, such as an address.
    /// All identities compared within a call must share an equality-compatible domain.
    pub identity: ResourceIdentity,
    /// Borrow or ownership requirement imposed by this input conversion.
    pub access: Access,
    /// Side-effect-free Rust boolean expression; false resources do not conflict.
    pub active: TokenStream,
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

/// Explicit template reference to the caller-selected source-result local.
/// [`OutputPlan`] resolves it from the explicit `source` argument before
/// calling [`crate::TemplateContext::render`] for generator-specific parameters.
pub fn source_result_expr() -> TokenStream {
    template_parameter("source_result")
}

macro_rules! plan_traits {
    ($plan:ident; $($field:ident),* $(,)?) => {
        impl<W: WireKind + Clone, C, G: Clone> Clone for $plan<W, C, G> {
            fn clone(&self) -> Self {
                Self {
                    context: self.context,
                    $($field: self.$field.clone(),)*
                }
            }
        }
        impl<W: WireKind + std::fmt::Debug, C, G: std::fmt::Debug> std::fmt::Debug
            for $plan<W, C, G>
        {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_struct(stringify!($plan))
                    $(.field(stringify!($field), &self.$field))*
                    .finish()
            }
        }
    }
}
plan_traits!(InputPlan; fallible, layout, acknowledgement, validation, acquire, resources, statements, argument);
plan_traits!(OutputPlan; fallible, layout, delivery, commit, statements, values);

/// Match prepared expressions to the fields of a concrete result packet or row.
/// The caller supplies destination fields in ABI order. Names, rather than the
/// expression list's order, establish correspondence; missing/extra names fail.
pub fn match_output_fields<'a, W: WireKind + 'a>(
    fields: impl IntoIterator<Item = &'a prebindgen_flat::Wire<W>>,
    bindings: &'a [(syn::Ident, TokenStream)],
) -> prebindgen_flat::Result<Vec<(&'a prebindgen_flat::Wire<W>, &'a TokenStream)>> {
    validate_value_bindings(bindings)?;
    let mut remaining = bindings
        .iter()
        .map(|(name, value)| (name.to_string(), value))
        .collect::<std::collections::BTreeMap<_, _>>();
    let mut out = vec![];
    for field in fields {
        let value = remaining
            .remove(&field.name.to_string())
            .ok_or_else(|| format!("missing output binding `{}`", field.name))?;
        out.push((field, value));
    }
    if let Some(name) = remaining.keys().next() {
        return Err(format!("unexpected output binding `{name}`"));
    }
    Ok(out)
}
/// Check that each prepared output name is defined once.
pub fn validate_value_bindings(
    bindings: &[(syn::Ident, TokenStream)],
) -> prebindgen_flat::Result<()> {
    let mut names = std::collections::BTreeSet::new();
    for (name, _) in bindings {
        if !names.insert(name.to_string()) {
            return Err(format!("duplicate output binding `{name}`"));
        }
    }
    Ok(())
}
