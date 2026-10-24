//! Declaration language and composition for preparing conversion slots.
//!
//! [`Declarations::fill`], [`bind_input`] and [`bind_output`] build conversions
//! directly in a [`Bindings`] model. Recipes and codec input data are used only
//! during construction; slots retain wires and code needed by wrapper writers.
//! Generation is owned by the language generator;
//! this crate owns only declaration resolution and slot preparation.
//! Each [`Wire`] carries the adapter's own description in [`Wire::kind`].
//! Reusable value and boxed-allocation leaves live in `prebindgen-conversions`.
#![warn(missing_docs)]
use prebindgen_conversions::{InputPlan, OutputPlan};
use prebindgen_flat::WireNode;
use prebindgen_flat::{same_type, type_key, Flat, Result, Wire};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Type;
mod enums;
pub use enums::{enum_input, enum_output, EnumMapping};
mod optional;
pub use optional::{
    absent_output_values, optional_input, optional_output, optional_presence,
    retire_optional_niches, select_optional_niche, NicheReservation,
};
pub use prebindgen_flat::{option_inner, result_parts, sequence_parts, SequenceView};
mod sequence;
pub use sequence::{outer_sequences, SequenceInput, SequenceOutput};
fn contains_result(ty: &Type) -> bool {
    result_parts(ty).is_some()
        || option_inner(ty).is_some_and(contains_result)
        || sequence_parts(ty).is_some_and(|(ty, _)| contains_result(ty))
        || matches!(ty, Type::Reference(r) if contains_result(&r.elem))
}
mod projection;
pub use projection::{Projection, ProjectionMode};
mod record;
pub use record::{record_input, record_output, record_output_owned};
mod condition;
pub use condition::Condition;
mod component;
pub use component::{ComponentPath, ComponentSegment};

/// Input preparation plan, including the codec's composition metadata.
pub type CodecInput<C> = InputPlan<
    <C as Codec>::Wire,
    <C as Codec>::Context,
    <C as Codec>::Optional,
    <C as Codec>::Sequence,
    <C as Codec>::Group,
    <C as Codec>::Alternatives,
>;
/// Output preparation plan, including the codec's composition metadata.
pub type CodecOutput<C> = OutputPlan<
    <C as Codec>::Wire,
    <C as Codec>::Context,
    <C as Codec>::Optional,
    <C as Codec>::Sequence,
    <C as Codec>::Group,
    <C as Codec>::Alternatives,
>;

/// A recipe for converting boundary inputs into one source argument.
///
/// Binding expands the recipe into wires and Rust conversion code.
#[derive(Clone, Debug)]
pub enum Input {
    /// Construct a collection by applying one recipe to every element.
    Sequence(Box<Input>),
    /// Convert an optional payload, gating every conversion phase.
    Optional(Box<Input>),
    /// Convert the occurrence type itself with [`Codec::input_leaf`].
    Leaf,
    /// Construct a named record from children in field order; an empty list builds unit.
    Fields(Vec<Input>),
    /// Call a source function after preparing its arguments with the child recipes.
    Constructor {
        /// Unqualified source function name; its result must match the occurrence type.
        function: String,
        /// Recipes in the constructor's parameter order.
        args: Vec<Input>,
    },
    /// Choose an owned input using an `i32` arm index followed by all arms' wires.
    /// Inactive arms are neither validated nor acquired; indices are zero-based.
    Selector(Vec<Input>),
}
impl Input {
    /// Construct a record from field recipes, or unit from an empty list.
    ///
    /// `fields` holds one recipe per field of the source record, in declaration
    /// order. Binding verifies the count against the record.
    pub fn fields(fields: impl IntoIterator<Item = Input>) -> Self {
        Self::Fields(fields.into_iter().collect())
    }
    /// Call a named source constructor with the supplied argument recipes.
    ///
    /// Binding checks its result type and argument count. Argument validation and
    /// ownership acquisition precede the source call.
    ///
    /// # Arguments
    ///
    /// * `name` — unqualified name of a source function whose result type is the
    ///   input's type.
    /// * `args` — one recipe per parameter of that function, in order.
    pub fn constructor(name: impl Into<String>, args: impl IntoIterator<Item = Input>) -> Self {
        Self::Constructor {
            function: name.into(),
            args: args.into_iter().collect(),
        }
    }
    /// Choose between owned representations of the same input type.
    ///
    /// `arms` holds one recipe per alternative; each must produce the input's
    /// type. The caller passes the zero-based index of the chosen alternative in an
    /// extra wire ahead of all arms' wires.
    ///
    /// Requires at least one arm. Each arm retains its own wires; this does not
    /// merge equivalent wire parameters or generate destination-language overloads.
    pub fn selector(arms: impl IntoIterator<Item = Input>) -> Self {
        Self::Selector(arms.into_iter().collect())
    }
}
/// A result conversion recipe, interpreted together with its source type and codec input data.
///
/// `Fields` observes fields; `OwnedFields` moves them. Explicit `Projected`
/// leaves select borrowed or owned conversion independently of type contents.
#[derive(Clone, Debug)]
pub enum Output {
    /// Convert collection elements using ordinary or explicitly selected ownership.
    Sequence {
        /// Element conversion, interpreted independently for each item.
        element: Box<Output>,
        /// `None` follows the occurrence's ownership: move owned values and observe
        /// borrowed values. `Some` explicitly selects and checks a projection policy.
        mode: Option<ProjectionMode>,
    },
    /// Deliver `Ok` through its recipe and route an owned `Err` through the codec.
    /// An optional mapper takes exactly `E` or `&E` and returns `String`.
    /// Without one, the codec must explicitly support preserving the error value.
    /// The codec selects how that message reaches the destination's error channel.
    Result {
        /// Conversion of the owned success value.
        ok: Box<Output>,
        /// Optional unqualified source message mapper. Without one, the codec
        /// must explicitly support the source error value itself.
        error: Option<String>,
    },
    /// Convert an optional payload without preparing absent values.
    Optional(Box<Output>),
    /// Convert the occurrence type itself with [`Codec::output_leaf`].
    Leaf,
    /// Observe named fields in declaration order; an empty list represents unit.
    Fields(Vec<Output>),
    /// Move named fields into their output conversions.
    OwnedFields(Vec<Output>),
    /// Convert a leaf with an explicit ownership policy.
    Projected(Projection),
    /// Call a consuming accessor, then convert its owned result.
    Into {
        /// Source accessor taking exactly the owned occurrence type.
        function: String,
        /// Conversion of its owned result.
        output: Box<Output>,
    },
    /// A shared-borrow accessor, followed by the output conversion of its result.
    Getter {
        /// Source getter taking exactly one shared reference to the current type.
        function: String,
        /// Conversion of the getter's owned result; borrowed results are unsupported.
        output: Box<Output>,
    },
    /// Produce several views of one value. At most one child may consume it.
    /// Observations run before consumption, while wire order follows this list.
    Combine(Vec<Output>),
}
impl Output {
    /// Observe record fields in declaration order; use an empty list for unit.
    ///
    /// `fields` holds one recipe per field of the source record, in declaration order.
    pub fn fields(fields: impl IntoIterator<Item = Output>) -> Self {
        Self::Fields(fields.into_iter().collect())
    }
    /// Observe a value through a source getter and convert the getter's result.
    ///
    /// The getter must take exactly `&T` for the current type `T`, and return an
    /// owned value. Binding rejects mismatched signatures.
    ///
    /// # Arguments
    ///
    /// * `name` — unqualified name of the source getter.
    /// * `output` — recipe for the getter's result.
    pub fn getter(name: impl Into<String>, output: Output) -> Self {
        Self::Getter {
            function: name.into(),
            output: Box::new(output),
        }
    }
    /// Produce multiple outputs from one value.
    ///
    /// `parts` holds one recipe per view of the value; their wires appear in this
    /// order. Binding rejects an empty list or more than one consuming part. The
    /// framework uses [`Codec::consumes`] to order observations before a move.
    pub fn combine(parts: impl IntoIterator<Item = Output>) -> Self {
        Self::Combine(parts.into_iter().collect())
    }
    fn consumes<C: Codec>(&self, ty: &Type, extra: &C::Extra, codec: &C, borrowed: bool) -> bool {
        match self {
            Self::Sequence { mode, .. } => match mode {
                Some(mode) => *mode == ProjectionMode::Owned,
                None => {
                    !borrowed
                        && sequence_parts(ty).is_some_and(|(_, view)| view == SequenceView::Owned)
                }
            },
            Self::Optional(child) => {
                child.consumes(option_inner(ty).unwrap_or(ty), extra, codec, borrowed)
            }
            Self::Leaf => codec.consumes(ty, extra),
            Self::Projected(p) => p.mode == ProjectionMode::Owned,
            Self::OwnedFields(_) | Self::Into { .. } | Self::Result { .. } => true,
            Self::Combine(p) => p.iter().any(|p| p.consumes(ty, extra, codec, borrowed)),
            _ => false,
        }
    }
}

/// Language-specific leaf conversions. Recursive composition stays in this declaration compiler.
///
/// A leaf is a source type the adapter converts itself: a value copied across
/// the boundary or an object kept in Rust. `prebindgen-conversions` provides helpers
/// for both representations. The
/// adapter chooses by type; a recipe only says where the leaves are.
///
/// Input leaves must validate without consuming in `validation`, then
/// acquire ownership without failure or source calls in `acquire`. Each acquired
/// value needs an ordinary Rust owner so later failures drop it. `statements`
/// can construct arguments from those owners. Every input wire must retain
/// the supplied activation condition. Output leaves keep ownership in Rust
/// locals until their infallible `commit` runs at the selected protocol's
/// transfer point. Buffered delivery commits after all foreign delivery succeeds;
/// an explicit visitor protocol can transfer each prepared element at callback entry.
///
/// # Implementing a codec
///
/// This minimal codec uses Rust values as its carriers. It illustrates converter
/// construction without a foreign ABI or object storage. A language adapter
/// additionally writes wrappers and chooses native and destination-language
/// carrier types for its wire kinds.
///
/// ```
/// use prebindgen_flat::{Result, Wire, validate_bindings};
/// use prebindgen_conversions::{value};
/// use prebindgen_declarations::{
///     Codec, CodecInput, CodecOutput, Declarations,
///     Bindings, fun, expand_param, expand_return, bind_input, Input,
/// };
/// use proc_macro2::TokenStream;
/// use quote::quote;
/// use syn::{Ident, Type};
///
/// use prebindgen_declarations::Condition;
/// fn carrier(kind: value::Value) -> Type {
///     match kind {
///         value::Value::Scalar(p) => p.rust_type(),
///         value::Value::Text => syn::parse_quote!(String),
///         value::Value::Bytes => syn::parse_quote!(Vec<u8>),
///     }
/// }
/// struct RustCarriers;
/// impl Codec for RustCarriers {
///     type Niche = ();
///     type Optional = ();
///     type Sequence = ();
///     type Group = ();
///     type Alternatives = ();
///     fn group_metadata(&self) {}
///     type Extra = ();
///     type Context = ();
///     type Input = CodecInput<Self>;
///     type Output = CodecOutput<Self>;
///     type Wire = Type; // The Rust carrier type is the complete wire description.
///
///     fn optional_metadata(&self, _: Ident, _: Ident, _: Option<()>, _: Vec<Wire<Type>>) -> Result<()> {Ok(())}
///     fn alternatives_metadata(&self, _: Wire<Type>) -> Result<()> {Ok(())}
///     fn field_group(&self, _: Ident) {}
///     fn fields_group(&self) {}
///     fn optional_niche(&self, _: &prebindgen_flat::WireNode<Self::Wire>) -> Option<prebindgen_declarations::NicheReservation<()>> {None}
///     fn retire_niches(&self, _: &mut prebindgen_flat::WireNode<Self::Wire>, _: Option<&prebindgen_declarations::NicheReservation<()>>) -> Result<()> {Ok(())}
///     fn niche(&self, _: &Self::Wire) -> Option<()> { None }
///     fn set_niche(&self, _: &mut Self::Wire, _: Option<()>) {}
///     fn selector(&self, _: &()) -> Self::Wire { syn::parse_quote!(i32) }
///     fn consumes(&self, _: &Type, _: &()) -> bool { false }
///     fn input_leaf(
///         &self, ty: &Type, _: &(), wire: Ident, var: Ident, active: &Condition,
///     ) -> Result<CodecInput<Self>> {
///         let kind = value::Value::of(ty).ok_or("only values cross this codec")?;
///         let decode = quote!(#wire.clone());
///         value::input(ty, wire, var, &active.rust(), decode, carrier(kind))
///     }
///     fn output_leaf(
///         &self, ty: &Type, _: &(), wire: Ident, v: TokenStream, borrowed: bool,
///     ) -> Result<CodecOutput<Self>> {
///         let kind = value::Value::of(ty).ok_or("only values cross this codec")?;
///         value::output(ty, wire, v, borrowed, |v| v, carrier(kind))
///     }
/// }
///
/// let source = syn::parse_file(r#"
///     pub fn from_i32(value: i32) -> i64 { value as i64 }
///     pub fn to_i32(value: &i64) -> i32 { *value as i32 }
///     pub fn echo(value: i64) -> i64 { value }
/// "#)?;
/// let mut flat = Bindings::<Type>::parse(
///     source.items.into_iter().map(|item| (item, Default::default())),
/// )?;
/// let defaults = Declarations::new(&flat, vec![
///     expand_param!(i64).variant(fun!(from_i32)).into(),
///     expand_return!(i64).field(fun!(to_i32)).field_self().into(),
/// ])?;
/// defaults.fill(&mut flat, &RustCarriers, &fun!(echo), &())?;
/// let function = flat.function("echo")?;
/// // Input: one constructor argument. Output: two projected values.
/// assert_eq!(prebindgen_flat::wire_definitions(std::iter::once(&function.params[0].ty.slot.as_ref().unwrap().node)).count(), 1);
/// assert_eq!(prebindgen_flat::wire_definitions(std::iter::once(&function.ret.slot.as_ref().unwrap().outcomes.success)).count(), 2);
/// // Replace just this parameter's converter; later default filling preserves it.
/// bind_input(&mut flat, &RustCarriers, "echo", "value", &Input::Leaf, &())?;
/// defaults.fill(&mut flat, &RustCarriers, &fun!(echo), &())?;
/// validate_bindings(&flat)?;
/// let input = flat.function("echo")?.params[0].ty.slot.as_ref().unwrap();
/// assert_eq!(prebindgen_flat::wire_definitions(std::iter::once(&input.node)).count(), 1);
/// assert_eq!(prebindgen_flat::type_key(&prebindgen_flat::wire_definitions(std::iter::once(&input.node)).next().unwrap().kind), "i64");
/// assert!(input.resources.is_empty()); // No foreign ownership in this codec.
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[allow(clippy::type_complexity)] // Explicit, independently typed metadata for each parent variant.
pub trait Codec {
    /// Adapter-owned description of an absent wire representation. Use `()` if unused.
    /// The shared compiler never interprets this metadata.
    type Niche: Clone;
    /// Metadata for one optional occurrence, including its presence representation.
    type Optional: Clone;
    /// Metadata for sequence encoding.
    type Sequence: Clone;
    /// Metadata for related values and source fields.
    type Group: Clone;
    /// Metadata for selecting an alternative.
    type Alternatives: Clone;
    /// Describe an ordered set of conversion components. Constructors, output
    /// accessors and combinations retain this group with one component. Constructors
    /// retain an empty group when there are no arguments. Source
    /// record fields instead use [`Codec::fields_group`].
    fn group_metadata(&self) -> Self::Group;
    /// Describe an optional value in the adapter's own metadata format.
    /// `presence` is the normalized 0/1 binding used by payload conditions.
    /// `discriminator` names a descendant wire; `niche` is its reserved absent
    /// representation, or `None` when a separate flag is an inline control wire.
    /// With a flag, `presence` and `discriminator` name that same wire. Retain the
    /// supplied controls unchanged. Return an error when they cannot form the
    /// adapter's flag or niche representation; failed preparation preserves existing slots.
    fn optional_metadata(
        &self,
        presence: syn::Ident,
        discriminator: syn::Ident,
        niche: Option<Self::Niche>,
        controls: Vec<Wire<Self::Wire>>,
    ) -> Result<Self::Optional>;
    /// Record the supplied selector for mutually exclusive input alternatives,
    /// preserving its niche state. Reject an unsupported tag representation.
    fn alternatives_metadata(&self, selector: Wire<Self::Wire>) -> Result<Self::Alternatives>;
    /// Describe one named source field, whose subtree may contain any number of wires.
    fn field_group(&self, name: syn::Ident) -> Self::Group;
    /// Describe a record or unit value's ordered collection of field subtrees.
    /// The shared helpers check field-name uniqueness before calling this hook.
    fn fields_group(&self) -> Self::Group;

    /// Input data borrowed while building one occurrence's conversion; use `()` if unused.
    /// The same value is passed through nested conversions. It is not stored in Flat;
    /// hooks retain only the data needed for emission in their code and wires.
    type Extra;
    /// Context supplied by the language generator when emitting stored templates.
    type Context: prebindgen_conversions::TemplateContext + 'static;
    /// Final self-contained input converter installed after preparation.
    type Input: prebindgen_flat::InputConversion + From<CodecInput<Self>>;
    /// Final self-contained output converter installed after preparation.
    type Output: prebindgen_flat::OutputConversion + From<CodecOutput<Self>>;
    /// Complete description of one wire, including its type and adapter metadata.
    /// Grouping belongs to parent nodes; no wire kind contains child wires.
    type Wire: prebindgen_flat::WireKind + Clone + std::fmt::Debug;
    /// Describe an `i32` selector tag or optional presence flag.
    ///
    /// Selector arm indices start at zero; optional flags use 0 and 1. A codec
    /// may advertise excluded negative values as niches for enclosing options.
    /// A newly added flag retains its own niches; inactive payload wires do not.
    /// `extra` is the occurrence data of the conversion that contains the selector.
    fn selector(&self, extra: &Self::Extra) -> Self::Wire;
    /// Whether a leaf output consumes the current source value.
    ///
    /// This must agree with [`Self::output_leaf`]. It governs move ordering and
    /// double-consumption rejection in [`Output::Combine`], not input ownership.
    /// A non-consuming leaf receives a shared reference with `borrowed` set.
    ///
    /// # Arguments
    ///
    /// * `ty` — owned source type of the value being delivered.
    /// * `extra` — occurrence data of the conversion, as passed to [`bind_output`].
    fn consumes(&self, ty: &Type, extra: &Self::Extra) -> bool;
    /// Prepare a leaf input of source type `ty` using occurrence data `extra`.
    ///
    /// All fallible checks belong in [`InputPlan::validation`]; do not
    /// consume values there. Return a generation error when this type or access
    /// mode is unsupported.
    ///
    /// # Arguments
    ///
    /// * `ty` — the source parameter type at this position, as written: `T`,
    ///   `&T` or `&mut T`.
    /// * `extra` — occurrence data of the conversion, as passed to [`bind_input`]
    ///   or [`Declarations::fill`].
    /// * `wire` — fresh identifier for the boundary parameter this leaf adds.
    ///   A leaf that needs more wires or locals derives their names from it.
    /// * `var` — fresh identifier for the Rust local that holds the converted value.
    ///   [`InputPlan::argument`] normally refers to it.
    /// * `active` — the selector arms under which this leaf is used. Copy it to every
    ///   returned wire and resource, and guard validation and acquisition with it.
    fn input_leaf(
        &self,
        ty: &Type,
        extra: &Self::Extra,
        wire: syn::Ident,
        var: syn::Ident,
        active: &Condition,
    ) -> Result<CodecInput<Self>>;
    /// Convert a leaf with explicit ownership, independently of its usual policy.
    ///
    /// The compiler supplies either an owned value or a shared reference according
    /// to `projection.mode`. The default supports leaves without field overrides.
    fn output_projected(
        &self,
        ty: &Type,
        extra: &Self::Extra,
        wire: syn::Ident,
        value: TokenStream,
        projection: &Projection,
    ) -> Result<CodecOutput<Self>> {
        if !projection.fields.is_empty() {
            return Err("this output converter does not support field projection overrides".into());
        }
        let mut payload = ty;
        while let Some(inner) = option_inner(payload) {
            payload = inner;
        }
        if projection.mode == ProjectionMode::Owned && matches!(payload, Type::Reference(_)) {
            return Err("owned projection cannot consume a reference payload".into());
        }
        self.output_leaf(
            ty,
            extra,
            wire,
            value,
            projection.mode == ProjectionMode::Borrowed,
        )
    }
    /// Choose array input transport for one prepared element layout.
    /// `wire` is the unique binding stem for this occurrence, not a requirement
    /// to use a single parameter. Metadata owns all outer values. `open` must
    /// make `length` available as a parameter or local; `bind` runs for each
    /// `index`. Raw reads may defer errors until the active conversion.
    ///
    /// Choose additional names fresh against the element and outer boundary
    /// values. The shared helper reserves `{wire}_staged`, `{wire}_acquired` and
    /// `{wire}_identities_N` (for each resource index N) for its own locals.
    fn sequence_input(
        &self,
        _extra: &Self::Extra,
        _wire: syn::Ident,
        _length: syn::Ident,
        _index: syn::Ident,
        _element: &prebindgen_flat::WireNode<
            Self::Wire,
            Self::Optional,
            Self::Sequence,
            Self::Group,
            Self::Alternatives,
        >,
        _active: &Condition,
    ) -> Result<SequenceInput<Self::Sequence>> {
        Err("collections are unsupported by this codec".into())
    }
    /// Choose array output transport; store element values without publishing owners.
    /// `begin` sees `length`; `enter` and `store` see each `index`.
    /// `wire` is a unique binding stem; metadata and `values` describe every
    /// outer value by binding name. Choose any additional names fresh
    /// against element and boundary values. `{wire}_source` and `{wire}_commits`
    /// are reserved for shared preparation locals.
    fn sequence_output(
        &self,
        _extra: &Self::Extra,
        _wire: syn::Ident,
        _length: syn::Ident,
        _index: syn::Ident,
        _element: &prebindgen_flat::WireNode<
            Self::Wire,
            Self::Optional,
            Self::Sequence,
            Self::Group,
            Self::Alternatives,
        >,
        _values: &[(syn::Ident, TokenStream)],
    ) -> Result<SequenceOutput<Self::Sequence>> {
        Err("collections are unsupported by this codec".into())
    }
    /// Route the owned source error value. `message`, when present, is an
    /// explicitly declared mapper expression returning String. Return a String
    /// expression for the active fallible boundary; an adapter may first deliver
    /// a structured error or throw an exception. The default requires a mapper
    /// and delegates to `result_error`.
    fn result_error_value(
        &self,
        ty: &Type,
        _value: TokenStream,
        message: Option<TokenStream>,
        extra: &Self::Extra,
    ) -> Result<TokenStream> {
        self.result_error(
            ty,
            message.ok_or_else(|| {
                format!(
                    "Result error `{}` requires an explicit error_mapper or on_error mapper",
                    type_key(ty)
                )
            })?,
            extra,
        )
    }
    /// Logical payload of the source Err(E) channel. A mapped error explicitly
    /// uses String; raw typed errors must provide their own declared projection.
    fn source_error_layout(
        &self,
        ty: &Type,
        mapped: bool,
        extra: &Self::Extra,
    ) -> Result<
        prebindgen_flat::WireNode<
            Self::Wire,
            Self::Optional,
            Self::Sequence,
            Self::Group,
            Self::Alternatives,
        >,
    > {
        if !mapped {
            return Err(format!(
                "missing logical error projection for `{}`",
                type_key(ty)
            ));
        }
        let plan = self.output_leaf(
            &syn::parse_quote!(String),
            extra,
            format_ident!("source_error_message"),
            quote!(source_error_message),
            false,
        )?;
        Ok(plan.outcomes.success)
    }
    /// Logical binding-failure payload, independently of how the adapter delivers
    /// it. None means this codec declares no binding-failure channel.
    fn binding_failure_layout(
        &self,
        _extra: &Self::Extra,
    ) -> Result<
        Option<
            prebindgen_flat::WireNode<
                Self::Wire,
                Self::Optional,
                Self::Sequence,
                Self::Group,
                Self::Alternatives,
            >,
        >,
    > {
        Ok(None)
    }
    /// Report a mapped String in the adapter-specific error channel.
    /// The returned expression must produce String. The compiler returns Err
    /// before preparing success outputs. The default rejects unsupported routing.
    fn result_error(
        &self,
        _ty: &Type,
        _message: TokenStream,
        _extra: &Self::Extra,
    ) -> Result<TokenStream> {
        Err("Result error routing is unsupported by this codec".into())
    }
    /// Select an excluded representation of this optional payload. The adapter
    /// chooses candidates in its value domain, including metadata-owned controls.
    fn optional_niche(
        &self,
        _payload: &prebindgen_flat::WireNode<
            Self::Wire,
            Self::Optional,
            Self::Sequence,
            Self::Group,
            Self::Alternatives,
        >,
    ) -> Option<NicheReservation<Self::Niche>>;
    /// Test presence using the reserved representation without decoding a payload.
    fn optional_presence(
        &self,
        _payload: &prebindgen_flat::WireNode<
            Self::Wire,
            Self::Optional,
            Self::Sequence,
            Self::Group,
            Self::Alternatives,
        >,
        _reservation: &NicheReservation<Self::Niche>,
    ) -> Result<TokenStream> {
        Err("optional niche presence is unsupported".into())
    }
    /// Inert expressions for every named output of an absent payload.
    fn absent_output_values(
        &self,
        _payload: &prebindgen_flat::WireNode<
            Self::Wire,
            Self::Optional,
            Self::Sequence,
            Self::Group,
            Self::Alternatives,
        >,
        _reservation: Option<&NicheReservation<Self::Niche>>,
    ) -> Result<Vec<(syn::Ident, TokenStream)>> {
        Err("optional outputs are unsupported".into())
    }
    /// Remove niche offers from inactive values. Retain only the reservation's
    /// donor with its remaining domain, when one was selected.
    fn retire_niches(
        &self,
        _payload: &mut prebindgen_flat::WireNode<
            Self::Wire,
            Self::Optional,
            Self::Sequence,
            Self::Group,
            Self::Alternatives,
        >,
        _reservation: Option<&NicheReservation<Self::Niche>>,
    ) -> Result<()>;
    /// Validate a prepared input before installing it or embedding it in a sequence.
    fn validate_input(&self, plan: &CodecInput<Self>) -> Result<()> {
        prebindgen_flat::validate_wire_nodes(std::iter::once(&plan.node))
    }
    /// Validate named success expressions against the adapter's output representation.
    fn validate_output(&self, plan: &CodecOutput<Self>) -> Result<()> {
        prebindgen_flat::validate_wire_nodes(
            std::iter::once(&plan.outcomes.success)
                .chain(plan.outcomes.source_error.iter())
                .chain(plan.outcomes.binding_failure.iter()),
        )?;
        prebindgen_conversions::validate_value_bindings(&plan.values)
    }
    /// Prepend a semantic source path to one prepared occurrence. Override when
    /// metadata-owned values carry paths, or element definitions have a separate
    /// path context. Codecs without source-path metadata need no implementation.
    fn prefix_layout(
        &self,
        _layout: &mut prebindgen_flat::WireNode<
            Self::Wire,
            Self::Optional,
            Self::Sequence,
            Self::Group,
            Self::Alternatives,
        >,
        _segment: ComponentSegment,
    ) {
    }
    /// The unused representation currently available on this wire.
    /// Initialize it when creating the wire and return the state last written by
    /// `set_niche`. Never offer a representation already used by an inner option
    /// or accepted as a present leaf payload. It may describe one value or a set.
    fn niche(&self, _wire: &Self::Wire) -> Option<Self::Niche> {
        None
    }
    /// Update the unused representation stored in a wire's own description.
    /// Optional encoding helpers call this after reserving or disabling a niche.
    /// Codecs must preserve this update in their wire description. A codec with
    /// no niches can implement this as a no-op and always return `None` from `niche`.
    fn set_niche(&self, _wire: &mut Self::Wire, _niche: Option<Self::Niche>) {}
    /// Recognize presence without decoding the payload. Called on original inner metadata.
    /// The expression must be total and side-effect-free for every initialized wire
    /// value, including inactive placeholders; it must not dereference raw pointers.
    /// It returns false for every representation reserved by this niche.
    fn is_present(
        &self,
        _wire: &Self::Wire,
        _niche: &Self::Niche,
        _value: TokenStream,
    ) -> Result<TokenStream> {
        Err("niche presence testing is unsupported by this codec".into())
    }
    /// Remaining excluded representation after reserving `niche` for one optional layer.
    /// The compiler stores the result in the wire through `set_niche`. Returning
    /// `None` exhausts this domain; enclosing options then use another wire or a flag.
    fn remaining_niche(&self, _wire: &Self::Wire, _niche: &Self::Niche) -> Option<Self::Niche> {
        None
    }
    /// Inert wire expression used for an absent payload, optionally using a niche.
    fn absent_value(
        &self,
        _wire: &Self::Wire,
        _niche: Option<&Self::Niche>,
        _input: bool,
    ) -> Result<TokenStream> {
        Err("optional values are unsupported by this codec".into())
    }
    /// Complete a composed parameter plan before it is stored in a slot.
    /// Called once per bound parameter, including records and selectors. The
    /// tree is complete when this hook runs. Edits to it or its metadata are
    /// immediately visible to adapter operations; no secondary layout needs refreshing.
    /// The compiler validates the tree before installing the finished slot.
    fn finish_input(&self, _plan: &mut CodecInput<Self>, _extra: &Self::Extra) -> Result<()> {
        Ok(())
    }
    /// Complete a composed result plan, including unit and multi-wire results.
    /// Install call-level delivery here, after all output expressions are known.
    /// The logical schema already includes the composed payload and error channels.
    /// Edits to the tree or its adapter metadata are preserved and validated.
    fn finish_output(&self, _plan: &mut CodecOutput<Self>, _extra: &Self::Extra) -> Result<()> {
        Ok(())
    }
    /// Prepare delivery of the Rust expression `value` as one or more wires.
    ///
    /// Keep unpublished allocations under Rust ownership and transfer them only in
    /// [`OutputPlan::commit`]. Return a generation error for unsupported
    /// conversions. Install final delivery in [`Codec::finish_output`], rather
    /// than on nested leaf plans; recursive composition rejects leaf delivery.
    ///
    /// # Arguments
    ///
    /// * `ty` — the owned source type of the value.
    /// * `extra` — occurrence data of the conversion, as passed to [`bind_output`]
    ///   or [`Declarations::fill`].
    /// * `wire` — fresh identifier for the delivered value. A leaf that needs more
    ///   wires or locals derives their names from it.
    /// * `value` — Rust expression producing the value to deliver.
    /// * `borrowed` — `true` when `value` is a shared reference to `ty`, such as a
    ///   record field observed in place; do not move out of it. `false` when
    ///   `value` is owned.
    fn output_leaf(
        &self,
        ty: &Type,
        extra: &Self::Extra,
        wire: syn::Ident,
        value: TokenStream,
        borrowed: bool,
    ) -> Result<CodecOutput<Self>>;
}
/// A flat model whose slots contain only conversion data for wrapper emission.
///
/// Unbound source helpers have `None` slots. Bound functions need `Some` for their
/// result and every parameter; [`prebindgen_flat::validate_bindings`] checks this before emission.
/// Record fields retain only names and source types. Recipes and [`Codec::Extra`]
/// are construction inputs, not stored in this model.
///
/// Output conversion data cannot be attached to an input position:
/// ```compile_fail
/// use prebindgen_declarations::Bindings;
/// fn swap(flat: &mut Bindings<syn::Type>) {
///     let function = flat.function_mut("foo").unwrap();
///     function.params[0].ty.slot = function.ret.slot.clone();
/// }
/// ```
/// Nor can a recipe be stored in a slot:
/// ```compile_fail
/// use prebindgen_declarations::{Bindings, Input};
/// fn configure(flat: &mut Bindings<syn::Type>) {
///     flat.function_mut("foo").unwrap().params[0].ty.slot = Some(Input::Leaf);
/// }
/// ```
pub type Bindings<W, C = (), O = (), S = (), G = (), A = ()> =
    Flat<InputPlan<W, C, O, S, G, A>, OutputPlan<W, C, O, S, G, A>>;

/// Build and attach a converter to a named function parameter.
///
/// Resolves the occurrence's source type from `flat`, checks recipe compatibility,
/// and recursively composes wires and code. Nested conversions use the supplied
/// `extra`, not data from referenced constructors or getters. Recipes and `extra`
/// are borrowed only for this call. A failure leaves the existing slot unchanged.
/// Rebinding an occurrence replaces its converter without changing other slots.
/// Generated names are scoped by parameter position, so separate calls cannot
/// collide within the wrapper. Keep the function signature fixed after binding.
/// Check completeness with [`prebindgen_flat::validate_bindings`].
///
/// # Arguments
///
/// * `flat` — the model holding the function; the parameter's slot is replaced.
/// * `codec` — the adapter's leaf conversions.
/// * `function` — unqualified name of the source function.
/// * `parameter` — name of the parameter, as written in the source.
/// * `recipe` — how to build the argument from boundary inputs.
/// * `extra` — occurrence data passed to every codec call for this conversion.
///
/// Input binding rejects output recipes:
/// ```compile_fail
/// use prebindgen_declarations::{bind_input, Codec, Output};
/// use prebindgen_flat::Flat;
/// fn wrong<C: Codec>(flat: &mut Flat<C::Input, C::Output>, codec: &C, extra: &C::Extra) {
///     bind_input(flat, codec, "foo", "x", &Output::Leaf, extra).unwrap();
/// }
/// ```
pub fn bind_input<C: Codec>(
    flat: &mut Flat<C::Input, C::Output>,
    codec: &C,
    function: &str,
    parameter: &str,
    recipe: &Input,
    extra: &C::Extra,
) -> Result<()> {
    let f = flat.function(function)?;
    let index = f
        .params
        .iter()
        .position(|p| p.name == parameter)
        .ok_or_else(|| format!("`{function}`: unknown parameter `{parameter}`"))?;
    let mut conversion = Compiler::new(flat, codec, format!("p{index}")).input(
        &f.params[index].ty.ty,
        recipe,
        extra,
        &Condition::default(),
    )?;

    codec.finish_input(&mut conversion, extra)?;
    prebindgen_flat::validate_wire_nodes(std::iter::once(&conversion.node))?;
    codec.validate_input(&conversion)?;
    flat.function_mut(function)?.params[index].ty.slot = Some(conversion.into());
    Ok(())
}

/// Build and attach a converter to a function result, replacing any existing one.
///
/// Uses the same construction and failure rules as [`bind_input`]. Output code
/// uses [`prebindgen_conversions::source_result_expr`] for the source call's result local.
/// The recipe is checked for compatible types and signatures, and for repeated
/// consumption of the same value. Unit results use `Output::fields([])`.
///
/// # Arguments
///
/// * `flat` — the model holding the function; the result's slot is replaced.
/// * `codec` — the adapter's leaf conversions.
/// * `function` — unqualified name of the source function.
/// * `recipe` — how to deliver the result as boundary outputs.
/// * `extra` — occurrence data passed to every codec call for this conversion.
///
/// Output binding rejects input recipes:
/// ```compile_fail
/// use prebindgen_declarations::{bind_output, Codec, Input};
/// use prebindgen_flat::Flat;
/// fn wrong<C: Codec>(flat: &mut Flat<C::Input, C::Output>, codec: &C, extra: &C::Extra) {
///     bind_output(flat, codec, "foo", &Input::Leaf, extra).unwrap();
/// }
/// ```
pub fn bind_output<C: Codec>(
    flat: &mut Flat<C::Input, C::Output>,
    codec: &C,
    function: &str,
    recipe: &Output,
    extra: &C::Extra,
) -> Result<()> {
    let mut conversion = Compiler::new(flat, codec, "ret".into()).output_root(
        &flat.function(function)?.ret.ty,
        recipe,
        extra,
        prebindgen_conversions::source_result_expr(),
        false,
    )?;
    finalize_output_schema(codec, &mut conversion, extra)?;
    codec.finish_output(&mut conversion, extra)?;
    prebindgen_flat::validate_wire_nodes(
        std::iter::once(&conversion.outcomes.success)
            .chain(conversion.outcomes.source_error.iter())
            .chain(conversion.outcomes.binding_failure.iter()),
    )?;
    codec.validate_output(&conversion)?;
    flat.function_mut(function)?.ret.slot = Some(conversion.into());
    Ok(())
}

fn finalize_output_schema<C: Codec>(
    codec: &C,
    plan: &mut CodecOutput<C>,
    extra: &C::Extra,
) -> Result<()> {
    if let Some(error) = codec.binding_failure_layout(extra)? {
        plan.outcomes.binding_failure = Some(error);
    }
    codec.validate_output(plan)
}

struct Compiler<'a, C: Codec> {
    flat: &'a Flat<C::Input, C::Output>,
    codec: &'a C,
    prefix: String,
    next: std::cell::Cell<usize>,
}
impl<'a, C: Codec> Compiler<'a, C> {
    fn new(flat: &'a Flat<C::Input, C::Output>, codec: &'a C, prefix: String) -> Self {
        Self {
            flat,
            codec,
            prefix,
            next: std::cell::Cell::new(0),
        }
    }
    fn name(&self) -> syn::Ident {
        let next = self.next.get();
        let n = format_ident!("{}_w{next}", self.prefix);
        self.next.set(next + 1);
        n
    }
    #[allow(clippy::type_complexity)] // Preserve the codec metadata types while walking its tree.
    fn prefix(
        &self,
        wires: &mut prebindgen_flat::WireNode<
            C::Wire,
            C::Optional,
            C::Sequence,
            C::Group,
            C::Alternatives,
        >,
        segment: ComponentSegment,
    ) {
        self.codec.prefix_layout(wires, segment);
    }
    fn input(
        &self,
        ty: &Type,
        recipe: &Input,
        extra: &C::Extra,
        active: &Condition,
    ) -> Result<CodecInput<C>> {
        if contains_result(ty) {
            return Err("Result inputs require an explicit data representation; error routing is output-only".into());
        }
        let var = self.name();
        let mut wires = Vec::new();
        let mut fallible = false;
        let mut validation = TokenStream::new();
        let mut acquire = TokenStream::new();
        let mut resources = Vec::new();
        let mut acknowledgement = None;
        let mut statements = TokenStream::new();
        let argument = quote!(#var);
        match recipe {
            Input::Sequence(element_recipe) => {
                let (element, _) =
                    sequence_parts(ty).ok_or("sequence input requires Vec<T>, &[T] or &Vec<T>")?;
                let wire = self.name();
                let length = self.name();
                let index = self.name();
                let child = self.input(element, element_recipe, extra, &Condition::default())?;
                return sequence::input(
                    self.codec, ty, extra, wire, var, length, index, active, child,
                );
            }
            Input::Optional(inner) => {
                let ty = option_inner(ty).ok_or("optional recipe requires Option<T>")?;
                let tag = self.name();
                let child = self.input(ty, inner, extra, &active.arm(tag.clone(), 1))?;
                return optional_input(self.codec, extra, tag, var, active, child);
            }
            Input::Leaf => {
                let w = self.name();
                return self.codec.input_leaf(ty, extra, w, var, active);
            }
            Input::Fields(children) => {
                if type_key(ty) == "()" && children.is_empty() {
                    return Ok(InputPlan {
                        fallible: false,
                        context: Default::default(),
                        acknowledgement,
                        node: WireNode::Group {
                            elements: vec![],
                            metadata: self.codec.fields_group(),
                        },
                        validation,
                        acquire,
                        resources,
                        statements,
                        argument: quote!(()),
                    });
                }
                let r = self.flat.record(ty)?;
                if r.fields.len() != children.len() {
                    return Err(format!(
                        "`{}`: expected {} field converters, got {}",
                        type_key(ty),
                        r.fields.len(),
                        children.len()
                    ));
                }
                return record_input(
                    self.codec,
                    ty,
                    var,
                    r.fields.iter().zip(children).map(|(field, child)| {
                        Ok((
                            field.name.clone(),
                            self.input(&field.ty, child, extra, active)?,
                        ))
                    }),
                );
            }
            Input::Constructor { function, args } => {
                let f = self.flat.function(function)?;
                if !same_type(&f.ret.ty, ty) {
                    return Err(format!(
                        "constructor `{function}` returns `{}`, expected `{}`",
                        type_key(&f.ret.ty),
                        type_key(ty)
                    ));
                }
                if f.params.len() != args.len() {
                    return Err(format!("constructor `{function}`: wrong argument count"));
                }
                let mut call_args = Vec::new();
                for (param, child) in f.params.iter().zip(args) {
                    let mut p = self.input(&param.ty.ty, child, extra, active)?;
                    self.prefix(
                        &mut p.node,
                        ComponentSegment::Argument(param.name.to_string()),
                    );
                    fallible |= p.fallible;
                    validation.extend(p.validation);
                    acquire.extend(p.acquire);
                    resources.extend(p.resources);
                    merge_acknowledgement(&mut acknowledgement, p.acknowledgement)?;
                    statements.extend(p.statements);
                    wires.push(p.node);
                    call_args.push(p.argument);
                }
                let path = &f.path;
                statements.extend(quote!(let #var:#ty=#path(#(#call_args),*);));
                wires = vec![WireNode::Group {
                    elements: wires,
                    metadata: self.codec.group_metadata(),
                }];
            }
            Input::Selector(arms) => {
                fallible = true;
                if arms.is_empty() {
                    return Err("an input selector needs at least one alternative".into());
                }
                if matches!(ty, Type::Reference(_)) {
                    return Err("selector results must be owned in this subset".into());
                }
                let tag = self.name();
                let mut selector_layout = prebindgen_flat::WireNode::Wire(vec![Wire {
                    name: tag.clone(),
                    kind: self.codec.selector(extra),
                }]);
                self.prefix(&mut selector_layout, ComponentSegment::Selector);
                let WireNode::Wire(mut selector_values) = selector_layout else {
                    return Err("selector prefixing must retain its value".into());
                };
                if selector_values.len() != 1 {
                    return Err("selector prefixing must retain its value".into());
                }
                let selector = selector_values.remove(0);
                let mut alternatives = Vec::new();
                let count = arms.len() as i32;
                let invalid = active.and(quote!(!(0..#count).contains(&#tag)));
                validation.extend(quote!(if #invalid {return Err("invalid selector".into());}));
                let mut cases = Vec::new();
                for (i, arm) in arms.iter().enumerate() {
                    let mut p = self.input(ty, arm, extra, &active.arm(tag.clone(), i as i32))?;
                    self.prefix(&mut p.node, ComponentSegment::Alternative(i));
                    self.codec.retire_niches(&mut p.node, None)?;
                    fallible |= p.fallible;
                    validation.extend(p.validation);
                    acquire.extend(p.acquire);
                    resources.extend(p.resources);
                    merge_acknowledgement(&mut acknowledgement, p.acknowledgement)?;
                    let body = p.statements;
                    let value = p.argument;
                    alternatives.push(p.node);
                    let i = i as i32;
                    cases.push(quote!(#i=>{#body #value}));
                }
                let node = WireNode::Alternatives {
                    metadata: self.codec.alternatives_metadata(selector)?,
                    elements: alternatives,
                };
                wires = vec![node];
                statements = quote!(let #var:#ty=match #tag{#(#cases,)* _=>unreachable!("validated selector")};);
            }
        }
        Ok(InputPlan {
            fallible,
            context: Default::default(),
            acknowledgement,
            node: wires.remove(0),
            validation,
            acquire,
            resources,
            statements,
            argument,
        })
    }
    fn output_root(
        &self,
        ty: &Type,
        recipe: &Output,
        extra: &C::Extra,
        value: TokenStream,
        borrowed: bool,
    ) -> Result<CodecOutput<C>> {
        if let Output::Result { ok, error } = recipe {
            if borrowed {
                return Err("Result error routing requires an owned Result occurrence".into());
            }
            let (success_ty, error_ty) =
                result_parts(ty).ok_or("Result recipe requires Result<T, E>")?;
            let payload = self.name();
            let failure = self.name();
            let mapped = if let Some(error) = error {
                let mapper = self.flat.function(error)?;
                let mapper_arg = mapper.params.first().map(|p| &p.ty.ty);
                let shared = matches!(mapper_arg, Some(Type::Reference(r)) if r.mutability.is_none() && same_type(&r.elem, error_ty));
                if mapper.params.len() != 1
                    || !(mapper_arg.is_some_and(|ty| same_type(ty, error_ty)) || shared)
                    || type_key(&mapper.ret.ty) != "String"
                {
                    return Err(
                        "Result error mapper must take exactly E or &E and return String".into(),
                    );
                }
                let path = &mapper.path;
                let argument = if shared {
                    quote!(&#failure)
                } else {
                    quote!(#failure)
                };
                Some(quote!(#path(#argument)))
            } else {
                None
            };
            let mapped_error = mapped.is_some();
            let message =
                self.codec
                    .result_error_value(error_ty, quote!(#failure), mapped, extra)?;
            let mut child = self.output(success_ty, ok, extra, quote!(#payload), false)?;
            let prepare = child.statements;
            child.statements = quote! {
                let #payload = match #value {
                    ::core::result::Result::Ok(value) => value,
                    ::core::result::Result::Err(#failure) => return Err(#message),
                };
                #prepare
            };

            let error = self
                .codec
                .source_error_layout(error_ty, mapped_error, extra)?;
            child.outcomes.source_error = Some(error);
            return Ok(child);
        }
        self.output(ty, recipe, extra, value, borrowed)
    }
    fn output(
        &self,
        ty: &Type,
        recipe: &Output,
        extra: &C::Extra,
        value: TokenStream,
        borrowed: bool,
    ) -> Result<CodecOutput<C>> {
        if contains_result(ty) {
            return Err("Result error routing is supported only at the outermost return with an explicit mapper".into());
        }
        let mut out = OutputPlan {
            fallible: false,
            context: Default::default(),
            delivery: Default::default(),
            commit: TokenStream::new(),
            outcomes: prebindgen_flat::Outcomes {
                success: WireNode::Group {
                    elements: vec![],
                    metadata: self.codec.group_metadata(),
                },
                source_error: None,
                binding_failure: None,
            },
            statements: TokenStream::new(),
            values: Vec::new(),
        };
        match recipe {
            Output::Sequence { element, mode } => {
                let (element_type, _) =
                    sequence_parts(ty).ok_or("sequence output requires a collection")?;
                let mode = mode.unwrap_or_else(|| {
                    if borrowed || sequence_parts(ty).unwrap().1 != SequenceView::Owned {
                        ProjectionMode::Borrowed
                    } else {
                        ProjectionMode::Owned
                    }
                });
                let wire = self.name();
                let payload = self.name();
                let length = self.name();
                let index = self.name();
                let child = self.output(
                    element_type,
                    element,
                    extra,
                    quote!(#payload),
                    mode == ProjectionMode::Borrowed,
                )?;
                return sequence::output(
                    self.codec, ty, extra, wire, payload, length, index, value, borrowed, mode,
                    child,
                );
            }
            Output::Result { .. } => {
                return Err("Result error routing is supported only at the outermost return".into())
            }
            Output::Optional(inner) => {
                let ty = option_inner(ty).ok_or("optional recipe requires Option<T>")?;
                let tag = self.name();
                let payload = self.name();
                let child = self.output(ty, inner, extra, quote!(#payload), borrowed)?;
                return optional_output(self.codec, extra, tag, payload, value, borrowed, child);
            }
            Output::Leaf => {
                let w = self.name();
                return self.codec.output_leaf(ty, extra, w, value, borrowed);
            }
            Output::Projected(projection) => {
                if borrowed && projection.mode == ProjectionMode::Owned {
                    return Err(
                        "owned projection cannot consume a shared-reference occurrence".into(),
                    );
                }
                let value = if !borrowed && projection.mode == ProjectionMode::Borrowed {
                    quote!(&(#value))
                } else {
                    value
                };
                return self
                    .codec
                    .output_projected(ty, extra, self.name(), value, projection);
            }
            Output::Fields(children) | Output::OwnedFields(children) => {
                let owned = matches!(recipe, Output::OwnedFields(_));
                if owned && borrowed {
                    return Err("owned fields cannot consume a shared-reference occurrence".into());
                }
                if type_key(ty) == "()" && children.is_empty() {
                    out.outcomes.success = WireNode::Group {
                        metadata: self.codec.fields_group(),
                        elements: vec![],
                    };

                    return Ok(out);
                }
                let r = self.flat.record(ty)?;
                if r.fields.len() != children.len() {
                    return Err("wrong output field count".into());
                }
                let convert = |index: usize, value| {
                    self.output(&r.fields[index].ty, &children[index], extra, value, !owned)
                };
                return if owned {
                    record_output_owned(
                        self.codec,
                        self.name(),
                        value,
                        r.fields.iter().map(|f| f.name.clone()),
                        convert,
                    )
                } else {
                    record_output(
                        self.codec,
                        self.name(),
                        value,
                        r.fields.iter().map(|f| f.name.clone()),
                        convert,
                    )
                };
            }
            Output::Getter { function, output } | Output::Into { function, output } => {
                let consuming = matches!(recipe, Output::Into { .. });
                if consuming && borrowed {
                    return Err(
                        "consuming accessor cannot consume a shared-reference occurrence".into(),
                    );
                }
                let f = self.flat.function(function)?;
                let expected: Type = if consuming {
                    ty.clone()
                } else {
                    syn::parse_quote!(&#ty)
                };
                if f.params.len() != 1 || !same_type(&f.params[0].ty.ty, &expected) {
                    return Err(format!(
                        "getter `{function}` must take exactly `{}`",
                        type_key(&expected)
                    ));
                }
                if matches!(f.ret.ty, Type::Reference(_)) {
                    return Err("borrowed getter results are outside this subset".into());
                }
                let v = self.name();
                let path = &f.path;
                let arg = if borrowed || consuming {
                    value
                } else {
                    quote!(&#value)
                };
                out.statements = quote!(let #v=#path(#arg););
                let mut child = self.output(&f.ret.ty, output, extra, quote!(#v), false)?;
                self.prefix(
                    &mut child.outcomes.success,
                    ComponentSegment::Getter(function.clone()),
                );
                append(&mut out, child)?;
            }
            Output::Combine(parts) => {
                if parts.is_empty() {
                    return Err("an output combination needs at least one part".into());
                }
                if parts
                    .iter()
                    .filter(|p| p.consumes(ty, extra, self.codec, borrowed))
                    .count()
                    > 1
                {
                    return Err("a combination cannot consume the same value twice".into());
                }
                let mut bodies = Vec::new();
                for (index, child) in parts.iter().enumerate() {
                    let consuming = child.consumes(ty, extra, self.codec, borrowed);
                    let arg = if borrowed || consuming {
                        value.clone()
                    } else {
                        quote!(&#value)
                    };
                    let mut p = self.output(ty, child, extra, arg, borrowed || !consuming)?;
                    self.prefix(&mut p.outcomes.success, ComponentSegment::View(index));
                    bodies.push((consuming, std::mem::take(&mut p.statements)));
                    append(&mut out, p)?;
                }
                // Wire order follows the declaration. Observations execute before the move.
                bodies.sort_by_key(|(consumes, _)| *consumes);
                for (_, body) in bodies {
                    out.statements.extend(body);
                }
            }
        }

        Ok(out)
    }
}
fn validate_nested_output<W: prebindgen_flat::WireKind, C, O, S, G, A>(
    plan: &OutputPlan<W, C, O, S, G, A>,
) -> Result<()> {
    if plan.outcomes.source_error.is_some() || plan.outcomes.binding_failure.is_some() {
        return Err("completion channels belong to the outermost output conversion".into());
    }
    Ok(())
}
fn append<W: prebindgen_flat::WireKind, C, O, S, G, A>(
    to: &mut OutputPlan<W, C, O, S, G, A>,
    from: OutputPlan<W, C, O, S, G, A>,
) -> Result<()> {
    if !from.delivery.statements.is_empty()
        || from.delivery.result.to_string() != quote!(()).to_string()
    {
        return Err("composed output delivery must be installed by Codec::finish_output".into());
    }
    validate_nested_output(&from)?;
    let WireNode::Group { elements, .. } = &mut to.outcomes.success else {
        return Err("output components require an ordered group".into());
    };
    elements.push(from.outcomes.success);
    to.fallible |= from.fallible;
    to.commit.extend(from.commit);
    to.statements.extend(from.statements);
    to.values.extend(from.values);
    prebindgen_conversions::validate_value_bindings(&to.values)
}

#[cfg(test)]
mod packet_tests;
#[cfg(test)]
mod tests;

pub mod declarations;
pub use declarations::{
    CallbackReturn, Declarations, ErrorDecl, ExpandDecl, ExpandParamDecl, ExpandReturnDecl,
    FieldsDecl, FunctionDecl,
};

/// Parse one Rust identifier used by a declaration.
///
/// Returns an error for invalid syntax, including paths or multiple identifiers.
pub fn ident(name: &str) -> Result<syn::Ident> {
    syn::parse_str(name).map_err(|_| format!("invalid identifier `{name}`"))
}

mod representation;
pub use representation::{Component, Representation, RepresentationId, Representations};

fn merge_acknowledgement(
    target: &mut Option<TokenStream>,
    incoming: Option<TokenStream>,
) -> Result<()> {
    if let Some(code) = incoming {
        if let Some(previous) = target {
            if previous.to_string() != code.to_string() {
                return Err("conflicting input acknowledgements".into());
            }
        } else {
            *target = Some(code);
        }
    }
    Ok(())
}

#[cfg(test)]
mod niche_tests;
