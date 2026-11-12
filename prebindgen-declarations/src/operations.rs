//! Inputs and results of declaration preparation operations.
use super::*;

/// Wire tree prepared by one codec.
pub type CodecNode<C> = WireNode<<C as Codec>::Wire, <C as Codec>::Chain, <C as Codec>::Branch>;
/// Prepared expressions keyed by their wire binding names.
pub type WireValues = Vec<(syn::Ident, TokenStream)>;

/// Direction in which a source value crosses the boundary.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    /// Foreign values become a Rust argument or callback result.
    Input,
    /// A Rust value becomes foreign result or callback argument values.
    Output,
}
/// Physical representation selected by the declaration compiler.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Representation {
    /// The adapter's normal representation, also inside ordinary sequence rows.
    Ordinary,
    /// Native fields inside an explicitly selected bulk frame.
    Bulk,
}
/// Source operation for which an input is being prepared.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputRole {
    /// One ordinary function parameter, including its nested fields.
    Parameter,
    /// An owned value returned by a retained source callback.
    CallbackResult,
    /// A payload member stored inside an ordinary enum carrier.
    EnumField,
}
/// Access to the Rust expression supplied to an output leaf.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ValueAccess {
    /// The expression has the declared type T; a reference T does not own its pointee.
    Owned,
    /// The expression observes the declared source value through &T.
    Shared,
}
impl ValueAccess {
    /// Whether the supplied expression is a shared observation.
    pub fn is_shared(self) -> bool {
        self == Self::Shared
    }
}

/// Prepare one source input leaf in an explicit representation and scope.
pub struct InputLeaf<'a, C: Codec + ?Sized> {
    /// Authoritative source and binding declarations.
    pub flat: &'a Flat<C::Schema>,
    /// Source Rust type at this position.
    pub ty: &'a Type,
    /// Borrowed occurrence configuration.
    pub extra: &'a C::Extra,
    /// Unique boundary binding stem.
    pub wire: syn::Ident,
    /// Rust binding for the constructed argument.
    pub target: syn::Ident,
    /// Branches under which this value is active.
    pub active: &'a Condition,
    /// Ordinary values or native bulk fields.
    pub representation: Representation,
    /// Ordinary parameter or source callback result.
    pub role: InputRole,
}
impl<'a, C: Codec + ?Sized> InputLeaf<'a, C> {
    /// Request an ordinary leaf for an enclosing function-parameter conversion.
    pub fn new(
        flat: &'a Flat<C::Schema>,
        ty: &'a Type,
        extra: &'a C::Extra,
        wire: syn::Ident,
        target: syn::Ident,
        active: &'a Condition,
    ) -> Self {
        Self {
            flat,
            ty,
            extra,
            wire,
            target,
            active,
            representation: Representation::Ordinary,
            role: InputRole::Parameter,
        }
    }
}

/// Prepare a Rust output leaf; explicit copying uses [`Codec::output_projected`].
pub struct OutputLeaf<'a, C: Codec + ?Sized> {
    /// Authoritative source and binding declarations.
    pub flat: &'a Flat<C::Schema>,
    /// Declared source type, possibly itself a reference.
    pub ty: &'a Type,
    /// Borrowed occurrence configuration.
    pub extra: &'a C::Extra,
    /// Unique output binding stem.
    pub wire: syn::Ident,
    /// Rust expression producing the source value or its shared observation.
    pub value: TokenStream,
    /// Actual access supplied by value, independently of explicit projection policy.
    pub access: ValueAccess,
    /// Ordinary values or native bulk fields.
    pub representation: Representation,
}
impl<'a, C: Codec + ?Sized> OutputLeaf<'a, C> {
    /// Request an ordinary output leaf with explicit source access.
    pub fn new(
        flat: &'a Flat<C::Schema>,
        ty: &'a Type,
        extra: &'a C::Extra,
        wire: syn::Ident,
        value: TokenStream,
        access: ValueAccess,
    ) -> Self {
        Self {
            flat,
            ty,
            extra,
            wire,
            value,
            access,
            representation: Representation::Ordinary,
        }
    }
}

/// Meaning of a group in the adapter's wire tree.
#[derive(Clone, Copy, Debug)]
pub enum GroupKind<'a> {
    /// Ordered constructor arguments or combined output observations.
    Values,
    /// Source fields, including unit's empty group.
    Fields,
    /// One named source field and its entire subtree.
    Field(&'a syn::Ident),
    /// An explicitly named output view.
    NamedOutput(&'a str),
}

/// Choose an optional encoding and update its payload's remaining niche offers.
pub struct Optional<'a, C: Codec + ?Sized> {
    /// Borrowed occurrence configuration.
    pub extra: &'a C::Extra,
    /// Temporary payload tree; its native scope determines available discriminators.
    pub payload: &'a mut CodecNode<C>,
    /// Requested normalized 0/1 presence binding.
    pub presence: syn::Ident,
    /// Input needs a presence test; output needs inactive expressions.
    pub direction: Direction,
}
/// Presence encoding selected by one optional preparation operation.
pub enum Presence<W: prebindgen_flat::WireKind> {
    /// One i32-compatible 0/1 control, named by the requested presence binding.
    Flag(Wire<W>),
    /// No added wire. Input requires a total, side-effect-free boolean test;
    /// output requires None because it branches on the source Rust Option.
    Niche {
        /// Test supplied only in input direction.
        input_test: Option<TokenStream>,
    },
}
/// Complete physical optional encoding, without duplicating its payload tree.
pub struct OptionalEncoding<C: Codec + ?Sized> {
    /// Separate flag or a reserved representation in an existing payload wire.
    pub presence: Presence<C::Wire>,
    /// Adapter metadata retaining the selected discriminator/reservation.
    pub metadata: C::Chain,
    /// Output expressions for every inactive payload wire. The niche donor must
    /// produce its reserved value. Empty for input.
    pub absent_values: WireValues,
}
/// Prepare metadata and inactive arm offers after the compiler names the selector.
pub struct Alternatives<'a, C: Codec + ?Sized> {
    /// The selector after its source path has already been prefixed.
    pub selector: &'a Wire<C::Wire>,
    /// Prepared arm trees; retire offers in inactive values within each native scope.
    pub arms: &'a mut [CodecNode<C>],
}

/// Physical grouping requested for a source enum occurrence.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EnumForm {
    /// One public foreign enum value.
    Value,
    /// An expanded tag and variant fields.
    Fields,
}
/// Prepare the physical interpretation of already converted enum payloads.
pub struct EnumVariants<'a, C: Codec + ?Sized> {
    /// Source enum and adapter declarations.
    pub flat: &'a Flat<C::Schema>,
    /// Qualified source type.
    pub ty: &'a Type,
    /// Input/output occurrence settings.
    pub extra: &'a C::Extra,
    /// Tag wire; the adapter can reserve its remaining domain for options.
    pub selector: &'a mut Wire<C::Wire>,
    /// One active-arm description per source variant.
    pub arms: &'a mut [CodecNode<C>],
    /// Ordinary enum value or explicitly expanded fields.
    pub form: EnumForm,
    /// Input or output conversion.
    pub direction: Direction,
}
/// Enum grouping metadata and safe raw values for inactive output slots.
pub struct EnumEncoding<C: Codec + ?Sized> {
    /// Adapter enum grouping, without duplicated child descriptions.
    pub metadata: C::Branch,
    /// One inactive-value list per output arm; empty on input.
    pub inactive: Vec<WireValues>,
}

/// Choose the physical input transport for a prepared sequence element.
pub struct SequenceInputRequest<'a, C: Codec + ?Sized> {
    /// Source collection ownership/view semantics.
    pub view: prebindgen_flat::SequenceView,
    /// Borrowed occurrence configuration.
    pub extra: &'a C::Extra,
    /// Unique carrier binding stem.
    pub wire: syn::Ident,
    /// Binding for the number of elements.
    pub length: syn::Ident,
    /// Binding for the current element index.
    pub index: syn::Ident,
    /// Prepared element tree.
    pub element: &'a CodecNode<C>,
    /// Active outer branches.
    pub active: &'a Condition,
    /// Ordinary rows or native bulk storage.
    pub representation: Representation,
}
/// Choose physical storage/delivery for a prepared sequence output.
pub struct SequenceOutputRequest<'a, C: Codec + ?Sized> {
    /// Borrowed occurrence configuration.
    pub extra: &'a C::Extra,
    /// Unique carrier binding stem.
    pub wire: syn::Ident,
    /// Binding for the number of elements.
    pub length: syn::Ident,
    /// Binding for the current element index.
    pub index: syn::Ident,
    /// Prepared element tree.
    pub element: &'a CodecNode<C>,
    /// Element expressions keyed by wire name.
    pub values: &'a [(syn::Ident, TokenStream)],
    /// Materialization or per-element callback, after resolving Auto.
    pub delivery: SequenceDelivery,
    /// Ordinary rows or native bulk storage.
    pub representation: Representation,
}

/// Prepare the layout and propagation of an owned source Result error together.
pub struct SourceError<'a, C: Codec + ?Sized> {
    /// Authoritative source/error declarations.
    pub flat: &'a Flat<C::Schema>,
    /// Original source error type E.
    pub ty: &'a Type,
    /// Borrowed occurrence configuration.
    pub extra: &'a C::Extra,
    /// Expression owning the caught E.
    pub value: TokenStream,
    /// Explicit mapper expression, evaluated at most once instead of value.
    pub message: Option<TokenStream>,
}
/// Matching error payload and expression for the active fallible boundary.
pub struct SourceErrorPlan<C: Codec + ?Sized> {
    /// Logical error wires, matching the declared delivery.
    pub payload: CodecNode<C>,
    /// String expression returned as Err after typed delivery, if any.
    pub propagate: TokenStream,
}
