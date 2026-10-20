use crate::template_parameter;
use prebindgen_flat::{Wire, WireKind};
use proc_macro2::TokenStream;
use quote::quote;
/// An input conversion split into call-wide validation, acquisition and construction.
///
/// Optional storage for the staged request protocol in this crate.
/// Generators may use this protocol or implement their own request types.
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
/// Optional concrete storage implementing [`prebindgen_flat::Conversion`].
/// Delivery and ownership commit are separate requests.
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
