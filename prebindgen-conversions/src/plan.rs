use crate::template_parameter;
use prebindgen_flat::WireKind;

use proc_macro2::TokenStream;
use quote::quote;
/// A complete input conversion and the backing storage its borrows require.
///
/// `code` binds the caller's target and evaluates its continuation exactly once.
/// `storage` contains only enclosing declarations with inert initializers. Recipe
/// composition can hoist those declarations outside optional/selector branches.
pub struct InputPlan<Wire: WireKind, Context = (), Chain = (), Branch = ()> {
    /// Whether conversion can return a binding error, excluding panics/aborts.
    pub fallible: bool,
    /// Context supplied by the language adapter during emission.
    pub context: std::marker::PhantomData<fn(&Context)>,
    /// Authoritative incoming representation, including adapter access metadata.
    pub node: prebindgen_flat::WireNode<Wire, Chain, Branch>,
    /// Backing-storage declarations, without wire reads or ownership takes.
    pub storage: TokenStream,
    /// Complete scoped conversion with target and continuation markers.
    pub code: TokenStream,
}
impl<Wire: WireKind, Context, Chain, Branch> InputPlan<Wire, Context, Chain, Branch> {
    /// Prepare conversion statements and their value as one scoped operation.
    /// Supporting owners needed outside conditional branches belong in `storage`.
    pub fn new(
        node: prebindgen_flat::WireNode<Wire, Chain, Branch>,
        code: TokenStream,
        value: TokenStream,
    ) -> Self {
        let target = template_parameter("input_target");
        let next = template_parameter("continuation");
        Self {
            fallible: false,
            context: Default::default(),
            node,
            storage: TokenStream::new(),
            code: quote!({#code let #target = #value; #next}),
        }
    }
    /// Compose a child conversion during preparation, retaining unresolved adapter
    /// bindings. Move its `storage` to the enclosing scope before conditional use.
    pub fn bind(
        &self,
        target: &syn::Ident,
        next: TokenStream,
    ) -> prebindgen_flat::Result<TokenStream> {
        let code = crate::emission::bind_target(&self.code, target)?;
        let code = crate::staged::continue_with(&code, next)?;
        crate::staged::enclose(self.storage.clone(), code)
    }
}
/// Output conversion whose owned temporaries survive until successful delivery.
///
/// Optional concrete storage implementing [`prebindgen_flat::Conversion`].
/// Delivery and ownership commit are separate requests.
pub struct OutputPlan<Wire: WireKind, Context = (), Chain = (), Branch = ()> {
    /// Whether preparation or delivery can return a binding error, excluding
    /// panics and allocation aborts. Adapters must mark custom fallible code.
    pub fallible: bool,
    /// Context type supplied by the language generator at emission.
    pub context: std::marker::PhantomData<fn(&Context)>,
    /// Delivery and return code prepared with this result slot.
    pub delivery: Delivery,
    /// Authoritative success/error value tree. Adapter metadata on its nodes
    /// records explicit native encodings without duplicating payload subtrees.
    pub outcomes: prebindgen_flat::Outcomes<prebindgen_flat::WireNode<Wire, Chain, Branch>>,
    /// Complete transfers for the selected protocol. Buffered outputs commit after
    /// foreign delivery succeeds; explicit visitor protocols can commit each
    /// element at callback entry during preparation.
    /// Commit code must be infallible and must not unwind after foreign delivery.
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
/// Whether the adapter may choose another supported result delivery protocol.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DeliveryPolicy {
    /// Preserve the converter's explicit delivery code and native result ABI.
    #[default]
    Fixed,
    /// Allow the adapter to choose its standard packet, direct or capture delivery.
    AdapterSelected,
}
/// Result-slot glue executed after output preparation and before ownership commit.
#[derive(Clone, Debug)]
pub struct Delivery {
    /// Permission to select an adapter delivery protocol; custom code stays fixed.
    pub policy: DeliveryPolicy,
    /// Deliver every output; failure must leave owned temporaries uncommitted.
    pub statements: TokenStream,
    /// Infallible wrapper result expression evaluated after commit.
    pub result: TokenStream,
}
impl Default for Delivery {
    fn default() -> Self {
        Self {
            policy: DeliveryPolicy::Fixed,
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
        impl<Wire: WireKind + Clone, Context, Chain: Clone, Branch: Clone> Clone for $plan<Wire, Context, Chain, Branch> {
            fn clone(&self) -> Self {
                Self {
                    context: self.context,
                    $($field: self.$field.clone(),)*
                }
            }
        }
        impl<Wire: WireKind + std::fmt::Debug, Context, Chain: std::fmt::Debug, Branch: std::fmt::Debug> std::fmt::Debug
            for $plan<Wire, Context, Chain, Branch>
        {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_struct(stringify!($plan))
                    $(.field(stringify!($field), &self.$field))*
                    .finish()
            }
        }
    }
}
plan_traits!(InputPlan; fallible, node, storage, code);
plan_traits!(OutputPlan; fallible, outcomes, delivery, commit, statements, values);

/// Match prepared expressions to the fields of a concrete result packet or row.
/// The caller supplies destination fields in ABI order. Names, rather than the
/// expression list's order, establish correspondence; missing/extra names fail.
pub fn match_output_fields<'a, Wire: WireKind + 'a>(
    fields: impl IntoIterator<Item = &'a prebindgen_flat::Wire<Wire>>,
    bindings: &'a [(syn::Ident, TokenStream)],
) -> prebindgen_flat::Result<Vec<(&'a prebindgen_flat::Wire<Wire>, &'a TokenStream)>> {
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
