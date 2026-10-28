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
use prebindgen_flat::{same_type, type_key, Flat, Result, Wire};
use prebindgen_flat::{Schema, WireNode};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Type;
mod builders;
mod callback;
pub use callback::{CallbackDecl, CallbackInput};
mod request_conversion;
pub use builders::{ErrorDecl, ExpandDecl, ExpandParamDecl, ExpandReturnDecl, FieldsDecl};
mod enums;
pub use enums::{enum_input, enum_output, EnumMapping};
mod optional;
pub use optional::{
    absent_output_values, optional_input, optional_output, optional_presence,
    retire_optional_niches, select_optional_niche, NicheReservation,
};
pub use prebindgen_flat::{option_inner, result_parts, sequence_parts, SequenceView};
mod sequence;
pub use sequence::{SequenceInput, SequenceOutput};
fn contains_result(ty: &Type) -> bool {
    result_parts(ty).is_some()
        || option_inner(ty).is_some_and(contains_result)
        || sequence_parts(ty).is_some_and(|(ty, _)| contains_result(ty))
        || matches!(ty, Type::Reference(r) if contains_result(&r.elem))
}
// Sized shared outputs use the referent's ordinary recipe, with borrowing
// carried by the compiler. Keep unsized string/slice views as their own types:
// adapters and sequence transport already understand those reference shapes.
fn output_referent(ty: &Type) -> Option<&Type> {
    match ty {
        Type::Reference(r)
            if r.mutability.is_none()
                && !matches!(r.elem.as_ref(), Type::Slice(_))
                && !matches!(r.elem.as_ref(), Type::Path(p) if p.path.is_ident("str")) =>
        {
            Some(&r.elem)
        }
        _ => None,
    }
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
pub type CodecInput<C> =
    InputPlan<<C as Codec>::Wire, <C as Codec>::Context, <C as Codec>::Chain, <C as Codec>::Branch>;
/// Output preparation plan, including the codec's composition metadata.
pub type CodecOutput<C> = OutputPlan<
    <C as Codec>::Wire,
    <C as Codec>::Context,
    <C as Codec>::Chain,
    <C as Codec>::Branch,
>;

/// How a sequence return reaches the foreign caller.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SequenceDelivery {
    /// Return a collection value using the language adapter's array/buffer representation.
    #[default]
    Value,
    /// Deliver each element through the function's sequence callback.
    Callback,
}

/// Resolved operations for converting boundary inputs into one source argument.
///
/// Preparation uses `Input<InputRequest>`; plain Input has no pending requests.
/// [`Declarations::resolve_input`] removes requests using source declarations and defaults.
/// An explicit subtree can be embedded in preparation with [`Input::into_preparation`].
/// Compilation lowers the resolved operations into wires and Rust code.
/// ```
/// use prebindgen_declarations::{Input,InputRequest};
/// let resolved: Input = Input::Constructor { function: "make".into(), args: vec![Input::Leaf] };
/// let preparation: Input<InputRequest> = resolved.into_preparation();
/// ```
/// Requests are impossible in resolved nodes, including nested constructor arguments:
/// ```compile_fail
/// use prebindgen_declarations::{Input,InputRequest};
/// let resolved: Input = Input::Constructor {
///     function: "make".into(),
///     args: vec![Input::Request(InputRequest::Auto)],
/// };
/// ```
#[derive(Clone, Debug)]
pub enum Input<P = std::convert::Infallible> {
    /// A pending preparation instruction. The default payload is uninhabited,
    /// so a resolved Input cannot contain this variant in safe Rust.
    Request(P),
    /// Retain a source callback with independent, fully resolved invocation conversions.
    Callback(Box<CallbackInput>),
    /// Construct a foreign record value using its configured language representation.
    Record(Vec<Input<P>>),
    /// Construct a collection by applying one recipe to every element.
    Sequence(Box<Input<P>>),
    /// Convert an optional payload, gating every conversion phase.
    Optional(Box<Input<P>>),
    /// Convert the occurrence type itself with [`Codec::input_leaf`].
    Leaf,
    /// Construct a named record from children in field order; an empty list builds unit.
    Fields(Vec<Input<P>>),
    /// Call a source function after preparing its arguments with the child recipes.
    Constructor {
        /// Unqualified source function name; its result must match the occurrence type.
        function: String,
        /// Recipes in the constructor's parameter order.
        args: Vec<Input<P>>,
    },
    /// Choose an owned input using an `i32` arm index followed by all arms' wires.
    /// Inactive arms are neither validated nor acquired; indices are zero-based.
    Selector(Vec<Input<P>>),
}
/// Source-dependent choices accepted only while preparing an input tree.
#[derive(Clone, Debug)]
pub enum InputRequest {
    /// Select the configured default for this source occurrence.
    Auto,
    /// Select or override a callback signature's argument and result conversions.
    Callback(Box<CallbackDecl>),
    /// Apply a type-specific override, including inside optional/sequence sources.
    ForType {
        /// Source type targeted by the override.
        ty: Box<Type>,
        /// The requested conversion for that type.
        input: Box<Input<InputRequest>>,
    },
    /// Infer a constructor's argument conversions from its source signature.
    Call(String),
}

impl Input<InputRequest> {
    /// Request this source type's configured default during preparation.
    pub fn auto() -> Self {
        Self::Request(InputRequest::Auto)
    }
    /// Associate a conversion with a source type for a default or nested override.
    pub fn for_type(ty: Type, input: Self) -> Self {
        Self::Request(InputRequest::ForType {
            ty: Box::new(ty),
            input: Box::new(input),
        })
    }
    /// Infer a source constructor's argument conversions from registered defaults.
    pub fn call(name: impl Into<String>) -> Self {
        Self::Request(InputRequest::Call(name.into()))
    }
    /// Add a constructor alternative to this conversion tree.
    pub fn variant(self, name: impl Into<String>) -> Self {
        self.add_alternative(Self::call(name))
    }
    /// Add a direct codec alternative, bypassing this type's expansion default.
    pub fn variant_self(self) -> Self {
        self.add_alternative(Self::Leaf)
    }
    fn add_alternative(self, arm: Self) -> Self {
        match self {
            Self::Request(InputRequest::ForType { ty, input }) => {
                Self::for_type(*ty, input.add_alternative(arm))
            }
            Self::Selector(mut arms) => {
                arms.push(arm);
                Self::Selector(arms)
            }
            other => Self::Selector(vec![other, arm]),
        }
    }
}
impl<P> Input<P> {
    /// Construct a record from field recipes, or unit from an empty list.
    ///
    /// `fields` holds one recipe per field of the source record, in declaration
    /// order. Binding verifies the count against the record.
    pub fn fields(fields: impl IntoIterator<Item = Input<P>>) -> Self {
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
    pub fn constructor(name: impl Into<String>, args: impl IntoIterator<Item = Input<P>>) -> Self {
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
    pub fn selector(arms: impl IntoIterator<Item = Input<P>>) -> Self {
        Self::Selector(arms.into_iter().collect())
    }
}
/// Resolved operations for converting a source result into boundary outputs.
/// Preparation uses `Output<OutputRequest>`; plain Output permits no pending requests.
/// [`Declarations::resolve_output`] resolves requests against source declarations and defaults.
/// Use [`Output::into_preparation`] to embed an explicit subtree in preparation;
/// source-dependent normalization still applies.
///
/// ```
/// use prebindgen_declarations::{Output,OutputRequest};
/// let resolved: Output = Output::Optional(Box::new(Output::Leaf));
/// let preparation: Output<OutputRequest> = resolved.into_preparation();
/// ```
///
/// Pending requests cannot be embedded inside a resolved output:
/// ```compile_fail
/// use prebindgen_declarations::{Output,OutputRequest};
/// let resolved: Output = Output::Optional(Box::new(Output::Request(OutputRequest::Auto)));
/// ```
///
/// `Fields` observes fields; `OwnedFields` moves them. Explicit `Projected`
/// leaves select borrowed or owned conversion independently of type contents.
#[derive(Clone, Debug)]
pub enum Output<P = std::convert::Infallible> {
    /// Omit this output value; source ownership still follows the enclosing operation.
    Discard,
    /// Give a converted view an explicit destination-language field name.
    Named {
        /// Destination name of the delivered view.
        name: String,
        /// Conversion of that view.
        output: Box<Output<P>>,
    },
    /// A pending preparation instruction. Resolved Output uses an uninhabited
    /// payload and therefore cannot contain a request.
    Request(P),
    /// Deliver a configured foreign record value with explicit field conversions.
    Record {
        /// Explicit source-field conversions in declaration order.
        fields: Vec<Output<P>>,
        /// `None` follows the occurrence: move owned fields and observe borrowed
        /// fields. `Some` explicitly selects and checks a projection policy.
        mode: Option<ProjectionMode>,
    },
    /// Convert collection elements using ordinary or explicitly selected ownership.
    Sequence {
        /// Delivery selected before conversion compilation.
        delivery: SequenceDelivery,
        /// Element conversion, interpreted independently for each item.
        element: Box<Output<P>>,
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
        ok: Box<Output<P>>,
        /// Optional unqualified source message mapper. Without one, the codec
        /// must explicitly support the source error value itself.
        error: Option<String>,
    },
    /// Convert an optional payload without preparing absent values.
    Optional(Box<Output<P>>),
    /// Convert the occurrence type itself with [`Codec::output_leaf`], or
    /// [`Codec::output_return_reference`] for a direct reference return.
    Leaf,
    /// Observe named fields in declaration order; an empty list represents unit.
    Fields(Vec<Output<P>>),
    /// Move named fields into their output conversions.
    OwnedFields(Vec<Output<P>>),
    /// Convert a leaf with an explicit ownership policy.
    Projected(Projection),
    /// Call a consuming accessor, then convert its result.
    Into {
        /// Source accessor taking exactly the owned occurrence type.
        function: String,
        /// Conversion of its result, retaining any shared-reference ownership.
        output: Box<Output<P>>,
    },
    /// A shared-borrow accessor, followed by the output conversion of its result.
    Getter {
        /// Source getter taking exactly one shared reference to the current type.
        function: String,
        /// Conversion of the getter's result. Shared results retain their borrow
        /// while being observed or cloned; they cannot satisfy an owned projection.
        /// Rust checks `Clone` when a borrowed opaque leaf becomes an owning handle.
        output: Box<Output<P>>,
    },
    /// Produce several views of one value. At most one child may consume it.
    /// Observations run before consumption, while wire order follows this list.
    Combine(Vec<Output<P>>),
}
/// Source-dependent choices accepted only while preparing an output tree.
#[derive(Clone, Debug)]
pub enum OutputRequest {
    /// Select the configured output default for this source occurrence.
    Auto,
    /// Apply a type-specific override, including inside optional/sequence sources.
    ForType {
        /// Source type targeted by the override.
        ty: Box<Type>,
        /// Requested conversion for the matching type.
        output: Box<Output<OutputRequest>>,
    },
    /// Resolve source record fields by name and fill unspecified conversions.
    NamedFields {
        /// Explicit conversions keyed by source field name.
        overrides: std::collections::BTreeMap<String, Output<OutputRequest>>,
        /// Ownership of field decomposition.
        mode: ProjectionMode,
    },
}

impl Output<OutputRequest> {
    /// Request this source type's configured output default during preparation.
    pub fn auto() -> Self {
        Self::Request(OutputRequest::Auto)
    }
    /// Associate a conversion with a source type for a default or nested override.
    pub fn for_type(ty: Type, output: Self) -> Self {
        Self::Request(OutputRequest::ForType {
            ty: Box::new(ty),
            output: Box::new(output),
        })
    }
    /// Append another simultaneous output view.
    pub fn view(self, view: impl Into<Self>) -> Self {
        let view = view.into();
        match self {
            Self::Request(OutputRequest::ForType { ty, output }) => {
                Self::for_type(*ty, output.view(view))
            }
            Self::Combine(mut parts) => {
                parts.push(view);
                Self::Combine(parts)
            }
            other => Self::Combine(vec![other, view]),
        }
    }
    /// Append a source getter whose result uses its registered conversion default.
    pub fn field(self, name: impl Into<String>) -> Self {
        self.view(Self::getter(name, Self::auto()))
    }
    /// Append the direct codec representation, bypassing this type's expansion default.
    pub fn field_self(self) -> Self {
        self.view(Self::Leaf)
    }
    /// Append an explicit borrowed projection.
    pub fn field_self_ref(self) -> Self {
        self.project_self(Projection::borrowed())
    }
    /// Append an explicit owned projection.
    pub fn field_self_into(self) -> Self {
        self.project_self(Projection::owned())
    }
    /// Append a projection with explicit ownership and nested field policies.
    pub fn project_self(self, policy: Projection) -> Self {
        self.view(Self::Projected(policy))
    }
    /// Append a consuming accessor followed by owned field decomposition.
    pub fn fields_into(self, view: Self) -> Self {
        let Self::Getter { function, output } = view else {
            panic!("fields_into requires a fields! accessor")
        };
        let Self::Request(OutputRequest::NamedFields { overrides, .. }) = *output else {
            panic!("fields_into requires named fields")
        };
        self.view(Self::Into {
            function,
            output: Box::new(Self::Request(OutputRequest::NamedFields {
                overrides,
                mode: ProjectionMode::Owned,
            })),
        })
    }
    /// Deprecated spelling of [`Self::fields_into`].
    #[deprecated(note = "use fields_into instead")]
    pub fn fields_self_into(self, view: Self) -> Self {
        self.fields_into(view)
    }
    /// Override one named source field, preserving defaults for its siblings.
    pub fn field_override(mut self, name: impl Into<String>, output: Self) -> Self {
        match &mut self {
            Self::Request(OutputRequest::NamedFields { overrides, .. }) => {
                assert!(
                    overrides.insert(name.into(), output).is_none(),
                    "duplicate field override"
                );
            }
            Self::Getter { output: inner, .. }
            | Self::Into { output: inner, .. }
            | Self::Request(OutputRequest::ForType { output: inner, .. }) => {
                let old = std::mem::replace(inner, Box::new(Self::auto()));
                **inner = old.field_override(name, output);
            }
            _ => panic!("field override requires named field decomposition"),
        }
        self
    }
}
impl<P> Output<P> {
    /// Observe record fields in declaration order; use an empty list for unit.
    ///
    /// `fields` holds one recipe per field of the source record, in declaration order.
    pub fn fields(fields: impl IntoIterator<Item = Output<P>>) -> Self {
        Self::Fields(fields.into_iter().collect())
    }
    /// Observe a value through a source getter and convert the getter's result.
    ///
    /// The getter takes `&T` for the current type `T`; adapters retaining a final
    /// output owner may also support `&mut T`. Its result uses the selected
    /// output conversion, including supported reference results.
    ///
    /// # Arguments
    ///
    /// * `name` — unqualified name of the source getter.
    /// * `output` — recipe for the getter's result.
    pub fn getter(name: impl Into<String>, output: Output<P>) -> Self {
        Self::Getter {
            function: name.into(),
            output: Box::new(output),
        }
    }
    /// Produce multiple outputs from one value.
    ///
    /// `parts` holds one recipe per view of the value; their wires appear in this
    /// order. Binding rejects an empty list or more than one consuming part. The
    /// framework uses [`Codec::consumes`] to order observations before a move,
    /// unless [`Codec::output_owner`] first retains the final output allocation.
    pub fn combine(parts: impl IntoIterator<Item = Output<P>>) -> Self {
        Self::Combine(parts.into_iter().collect())
    }
}
impl Output {
    fn consumes<C: Codec>(
        &self,
        flat: &prebindgen_flat::Flat<C::Schema>,
        ty: &Type,
        extra: &C::Extra,
        codec: &C,
        borrowed: bool,
    ) -> bool {
        match self {
            Self::Sequence { mode, .. } => match mode {
                Some(mode) => *mode == ProjectionMode::Owned,
                None => {
                    !borrowed
                        && sequence_parts(ty).is_some_and(|(_, view)| view == SequenceView::Owned)
                }
            },
            Self::Named { output: child, .. } => child.consumes(flat, ty, extra, codec, borrowed),
            Self::Optional(child) => {
                child.consumes(flat, option_inner(ty).unwrap_or(ty), extra, codec, borrowed)
            }
            Self::Leaf => {
                !borrowed && output_referent(ty).is_none() && codec.consumes(flat, ty, extra)
            }
            Self::Projected(p) => p.mode == ProjectionMode::Owned,
            Self::OwnedFields(_) | Self::Into { .. } | Self::Result { .. } => true,
            Self::Record { mode, .. } => mode
                .map_or(!borrowed && output_referent(ty).is_none(), |m| {
                    m == ProjectionMode::Owned
                }),
            Self::Request(never) => match *never {},
            Self::Combine(p) => p
                .iter()
                .any(|p| p.consumes(flat, ty, extra, codec, borrowed)),
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
/// Source-aware hooks receive the authoritative Flat model. Read configured type,
/// field, function and module metadata there instead of retaining parallel indexes
/// in the codec or passing copies through `Extra`. Pure wire-composition hooks need
/// no model access. Unresolved recipes remain temporary preparation inputs.
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
///     type Schema = prebindgen_conversions::PlanSchema<Type>;
///     type Niche = ();
///     type Chain = ();
///     type Branch = ();
///     fn group_metadata(&self) {}
///     type Extra = ();
///     type Context = ();
///     type Wire = Type; // The Rust carrier type is the complete wire description.
///
///     fn optional_metadata(&self, _: Ident, _: Ident, _: Option<()>, _: &[Wire<Type>]) -> Result<()> {Ok(())}
///     fn alternatives_metadata(&self, _: &Wire<Type>) -> Result<()> {Ok(())}
///     fn field_group(&self, _: Ident) {}
///     fn fields_group(&self) {}
///     fn optional_niche(&self, _: &prebindgen_flat::WireNode<Self::Wire>) -> Option<prebindgen_declarations::NicheReservation<()>> {None}
///     fn retire_niches(&self, _: &mut prebindgen_flat::WireNode<Self::Wire>, _: Option<&prebindgen_declarations::NicheReservation<()>>) -> Result<()> {Ok(())}
///     fn niche(&self, _: &Self::Wire) -> Option<()> { None }
///     fn set_niche(&self, _: &mut Self::Wire, _: Option<()>) {}
///     fn selector(&self, _: &()) -> Self::Wire { syn::parse_quote!(i32) }
///     fn consumes(&self, _flat: &prebindgen_flat::Flat<Self::Schema>, _: &Type, _: &()) -> bool { false }
///     fn input_leaf(
///         &self, _flat: &prebindgen_flat::Flat<Self::Schema>, ty: &Type, _: &(), wire: Ident, var: Ident, active: &Condition,
///     ) -> Result<CodecInput<Self>> {
///         let kind = value::Value::of(ty).ok_or("only values cross this codec")?;
///         let decode = quote!(#wire.clone());
///         value::input(ty, wire, var, &active.rust(), decode, carrier(kind))
///     }
///     fn output_leaf(
///         &self, _flat: &prebindgen_flat::Flat<Self::Schema>, ty: &Type, _: &(), wire: Ident, v: TokenStream, borrowed: bool,
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
/// let defaults = Declarations::new(&flat,
///     vec![expand_param!(i64).variant("from_i32").into()],
///     vec![expand_return!(i64).field("to_i32").field_self().into()],
///     vec![],
/// )?;
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
    /// Fixed schema selecting final converters and declaration metadata before parsing.
    type Schema: Schema<
        Input: prebindgen_flat::InputConversion + From<CodecInput<Self>>,
        Output: prebindgen_flat::OutputConversion + From<CodecOutput<Self>>,
    >;
    /// Compile a callback whose two conversion directions are already resolved.
    fn callback_input(
        &self,
        _flat: &Flat<Self::Schema>,
        _callback: &CallbackInput,
        _extra: &Self::Extra,
        _wire: syn::Ident,
        _var: syn::Ident,
        _active: &Condition,
    ) -> Result<CodecInput<Self>> {
        Err("configured callbacks are unsupported by this codec".into())
    }
    /// Source-aware input normalization, called only during tree preparation.
    /// Return an explicit operation tree for a language-specific representation.
    fn expand_input(
        &self,
        _flat: &Flat<Self::Schema>,
        _ty: &Type,
        _tree: &Input<InputRequest>,
        _extra: &Self::Extra,
    ) -> Result<Option<Input<InputRequest>>> {
        Ok(None)
    }
    /// Source-aware output normalization, called before conversion compilation.
    fn expand_output(
        &self,
        _flat: &Flat<Self::Schema>,
        _ty: &Type,
        _tree: &Output<OutputRequest>,
        _extra: &Self::Extra,
    ) -> Result<Option<Output<OutputRequest>>> {
        Ok(None)
    }
    /// Annotate an already compiled foreign-record input; structural expansion is complete.
    fn input_record_metadata(
        &self,
        _flat: &prebindgen_flat::Flat<Self::Schema>,
        _ty: &Type,
        _wire: &syn::Ident,
        _active: &Condition,
        _plan: &mut CodecInput<Self>,
        _extra: &Self::Extra,
    ) -> Result<()> {
        Err("foreign record inputs are unsupported by this codec".into())
    }
    /// Annotate an already compiled foreign-record output with its selected representation.
    fn output_record_metadata(
        &self,
        _flat: &prebindgen_flat::Flat<Self::Schema>,
        _ty: &Type,
        _wire: &syn::Ident,
        _plan: &mut CodecOutput<Self>,
        _extra: &Self::Extra,
    ) -> Result<()> {
        Err("foreign record outputs are unsupported by this codec".into())
    }
    /// Adapter-owned description of an absent wire representation. Use `()` if unused.
    /// The shared compiler never interprets this metadata.
    type Niche: Clone;
    /// Adapter metadata for parents with exactly one child description.
    type Chain: Clone;
    /// Adapter metadata for parents with an ordered list of child descriptions.
    type Branch: Clone;
    /// Describe an ordered set of conversion components. Constructors, output
    /// accessors and combinations retain this group with one component. Constructors
    /// retain an empty group when there are no arguments. Source
    /// record fields instead use [`Codec::fields_group`].
    fn group_metadata(&self) -> Self::Branch;
    /// Preserve an explicitly named output view in the prepared wire tree.
    fn named_output_group(&self, _name: &str) -> Result<Self::Branch> {
        Err("named output views are unsupported by this codec".into())
    }
    /// Describe an optional value in the adapter's own metadata format.
    /// `presence` is the normalized 0/1 binding used by payload conditions.
    /// `discriminator` names a descendant wire; `niche` is its reserved absent
    /// representation, or `None` when a separate flag is an inline control wire.
    /// With a flag, `presence` and `discriminator` name that same wire. The
    /// supplied controls stay in the tree. Borrow them to validate the encoding;
    /// metadata must not retain wire descriptors. Return an error when they cannot form the
    /// adapter's flag or niche representation; failed preparation preserves existing slots.
    fn optional_metadata(
        &self,
        presence: syn::Ident,
        discriminator: syn::Ident,
        niche: Option<Self::Niche>,
        controls: &[Wire<Self::Wire>],
    ) -> Result<Self::Chain>;
    /// Record the supplied selector for mutually exclusive input alternatives,
    /// preserving its niche state in the tree. Metadata must not retain the wire.
    /// Reject an unsupported tag representation.
    fn alternatives_metadata(&self, selector: &Wire<Self::Wire>) -> Result<Self::Branch>;
    /// Describe one named source field, whose subtree may contain any number of wires.
    fn field_group(&self, name: syn::Ident) -> Self::Branch;
    /// Describe a record or unit value's ordered collection of field subtrees.
    /// The shared helpers check field-name uniqueness before calling this hook.
    fn fields_group(&self) -> Self::Branch;

    /// Input data borrowed while building one occurrence's conversion; use `()` if unused.
    /// The same value is passed through nested conversions. It is not stored in Flat;
    /// hooks retain only the data needed for emission in their code and wires.
    type Extra;
    /// Context supplied by the language generator when emitting stored templates.
    type Context: prebindgen_conversions::TemplateContext + 'static;
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
    fn consumes(
        &self,
        flat: &prebindgen_flat::Flat<Self::Schema>,
        ty: &Type,
        extra: &Self::Extra,
    ) -> bool;
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
        flat: &prebindgen_flat::Flat<Self::Schema>,
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
        flat: &prebindgen_flat::Flat<Self::Schema>,
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
            flat,
            ty,
            extra,
            wire,
            value,
            projection.mode == ProjectionMode::Borrowed,
        )
    }
    /// Choose array input transport for one prepared element layout.
    /// `wire` is the unique binding stem for this occurrence, not a requirement
    /// to use a single parameter. The returned controls own the outer wire definitions.
    /// Chain metadata describes their interpretation. `open` must
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
        _element: &prebindgen_flat::WireNode<Self::Wire, Self::Chain, Self::Branch>,
        _active: &Condition,
    ) -> Result<SequenceInput<Self::Wire, Self::Chain>> {
        Err("collections are unsupported by this codec".into())
    }
    /// Implement the selected sequence delivery; stage elements without publishing owners.
    /// `begin` sees `length`; `enter` and `store` see each `index`.
    /// `wire` is a unique binding stem; metadata and `values` describe every
    /// outer value by binding name. Choose any additional names fresh
    /// against element and boundary values. `{wire}_source` and `{wire}_commits`
    /// are reserved for shared preparation locals.
    #[allow(clippy::too_many_arguments)] // Explicit occurrence bindings and delivery policy.
    fn sequence_output(
        &self,
        _extra: &Self::Extra,
        _wire: syn::Ident,
        _length: syn::Ident,
        _index: syn::Ident,
        _element: &prebindgen_flat::WireNode<Self::Wire, Self::Chain, Self::Branch>,
        _values: &[(syn::Ident, TokenStream)],
        _delivery: SequenceDelivery,
    ) -> Result<SequenceOutput<Self::Wire, Self::Chain>> {
        Err("collections are unsupported by this codec".into())
    }
    /// Route the owned source error value. `message`, when present, is an
    /// explicitly declared mapper expression returning String. Return a String
    /// expression for the active fallible boundary; an adapter may first deliver
    /// a structured error or throw an exception. The default requires a mapper
    /// and delegates to `result_error`.
    fn result_error_value(
        &self,
        flat: &prebindgen_flat::Flat<Self::Schema>,
        ty: &Type,
        _value: TokenStream,
        message: Option<TokenStream>,
        extra: &Self::Extra,
    ) -> Result<TokenStream> {
        self.result_error(
            flat,
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
        flat: &prebindgen_flat::Flat<Self::Schema>,
        ty: &Type,
        mapped: bool,
        extra: &Self::Extra,
    ) -> Result<prebindgen_flat::WireNode<Self::Wire, Self::Chain, Self::Branch>> {
        if !mapped {
            return Err(format!(
                "missing logical error projection for `{}`",
                type_key(ty)
            ));
        }
        let plan = self.output_leaf(
            flat,
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
    ) -> Result<Option<prebindgen_flat::WireNode<Self::Wire, Self::Chain, Self::Branch>>> {
        Ok(None)
    }
    /// Report a mapped String in the adapter-specific error channel.
    /// The returned expression must produce String. The compiler returns Err
    /// before preparing success outputs. The default rejects unsupported routing.
    fn result_error(
        &self,
        _flat: &prebindgen_flat::Flat<Self::Schema>,
        _ty: &Type,
        _message: TokenStream,
        _extra: &Self::Extra,
    ) -> Result<TokenStream> {
        Err("Result error routing is unsupported by this codec".into())
    }
    /// Select an excluded representation of this optional payload. The adapter
    /// chooses candidates in its value domain, including tree-owned controls.
    fn optional_niche(
        &self,
        _payload: &prebindgen_flat::WireNode<Self::Wire, Self::Chain, Self::Branch>,
    ) -> Option<NicheReservation<Self::Niche>>;
    /// Test presence using the reserved representation without decoding a payload.
    fn optional_presence(
        &self,
        _payload: &prebindgen_flat::WireNode<Self::Wire, Self::Chain, Self::Branch>,
        _reservation: &NicheReservation<Self::Niche>,
    ) -> Result<TokenStream> {
        Err("optional niche presence is unsupported".into())
    }
    /// Inert expressions for every named output of an absent payload.
    fn absent_output_values(
        &self,
        _payload: &prebindgen_flat::WireNode<Self::Wire, Self::Chain, Self::Branch>,
        _reservation: Option<&NicheReservation<Self::Niche>>,
    ) -> Result<Vec<(syn::Ident, TokenStream)>> {
        Err("optional outputs are unsupported".into())
    }
    /// Remove niche offers from inactive values. Retain only the reservation's
    /// donor with its remaining domain, when one was selected.
    fn retire_niches(
        &self,
        _payload: &mut prebindgen_flat::WireNode<Self::Wire, Self::Chain, Self::Branch>,
        _reservation: Option<&NicheReservation<Self::Niche>>,
    ) -> Result<()>;
    /// Validate a prepared input before installing it or embedding it in a sequence.
    /// The default accepts every structural shape. Adapters interpret their own
    /// metadata and enforce semantic rules, such as requiring nonempty alternatives.
    fn validate_input(&self, _plan: &CodecInput<Self>) -> Result<()> {
        Ok(())
    }
    /// Validate named success expressions against the adapter's output representation.
    fn validate_output(&self, plan: &CodecOutput<Self>) -> Result<()> {
        prebindgen_conversions::validate_value_bindings(&plan.values)
    }
    /// Prepend a semantic source path to one prepared occurrence. Override when
    /// node controls carry paths, or element definitions have a separate
    /// path context. Codecs without source-path metadata need no implementation.
    fn prefix_layout(
        &self,
        _layout: &mut prebindgen_flat::WireNode<Self::Wire, Self::Chain, Self::Branch>,
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
        flat: &prebindgen_flat::Flat<Self::Schema>,
        ty: &Type,
        extra: &Self::Extra,
        wire: syn::Ident,
        value: TokenStream,
        borrowed: bool,
    ) -> Result<CodecOutput<Self>>;
    /// Convert a reference supplied directly by the Rust return value.
    ///
    /// Unlike observing an owned value through a getter, this reference already
    /// has the source function's lifetime contract. Adapters may expose it as a
    /// non-owning foreign pointer. `reference` preserves the referent and mutability;
    /// `value` produces exactly that Rust reference.
    /// This also applies inside optional and successful Result returns, but not
    /// inside getter expansions, record decomposition or callback arguments.
    /// The foreign caller must preserve the referenced storage and Rust aliasing
    /// rules. No runtime lifetime tracking is implied.
    ///
    /// Return `None` to use ordinary borrowed leaf conversion (the default).
    fn output_return_reference(
        &self,
        _flat: &prebindgen_flat::Flat<Self::Schema>,
        _reference: &syn::TypeReference,
        _extra: &Self::Extra,
        _wire: syn::Ident,
        _value: TokenStream,
    ) -> Result<Option<CodecOutput<Self>>> {
        Ok(None)
    }
    /// Place an owned value in its final foreign allocation before getter views.
    ///
    /// Returns its owning output conversion and a Rust `*mut ty` expression from
    /// which getter receivers can be borrowed. That address must remain valid
    /// across conversion, delivery and ownership commit. Until commit, the plan
    /// must reclaim the allocation on failure. The default keeps ordinary output
    /// conversion and observation-before-move ordering.
    fn output_owner(
        &self,
        _flat: &Flat<Self::Schema>,
        _ty: &Type,
        _extra: &Self::Extra,
        _wire: syn::Ident,
        _value: TokenStream,
    ) -> Result<Option<(CodecOutput<Self>, TokenStream)>> {
        Ok(None)
    }
    /// Check whether a getter's converted outputs can outlive its temporary owner.
    /// Owner-backed views are compiled separately and already retain their owner.
    fn validate_getter_output(
        &self,
        _ty: &Type,
        _plan: &CodecOutput<Self>,
        _extra: &Self::Extra,
    ) -> Result<()> {
        Ok(())
    }
}
/// A flat model whose slots contain only conversion data for wrapper emission.
///
/// Unbound source helpers have `None` slots. Bound functions need `Some` for their
/// result and every parameter; [`prebindgen_flat::validate_bindings`] checks this before emission.
/// Record fields retain their canonical source information and metadata. Descriptions and [`Codec::Extra`]
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
pub type Bindings<W, C = (), N = (), B = ()> = Flat<prebindgen_conversions::PlanSchema<W, C, N, B>>;

/// Compile a request-free description for one named source parameter.
/// The model keeps its fixed schema; this returns a converter without modifying its slot.
/// ```
/// use prebindgen_declarations::{compile_input,Codec,Input};
/// use prebindgen_flat::Flat;
/// fn resolved<C:Codec>(flat:&Flat<C::Schema>,codec:&C,extra:&C::Extra) {
///     compile_input(flat,codec,"f","x",&Input::Leaf,extra).unwrap();
/// }
/// ```
/// ```compile_fail
/// use prebindgen_declarations::{compile_input,Codec,Input};
/// use prebindgen_flat::Flat;
/// fn pending<C:Codec>(flat:&Flat<C::Schema>,codec:&C,extra:&C::Extra) {
///     compile_input(flat,codec,"f","x",&Input::auto(),extra).unwrap();
/// }
/// ```
pub fn compile_input<C: Codec>(
    flat: &Flat<C::Schema>,
    codec: &C,
    function: &str,
    parameter: &str,
    recipe: &Input,
    extra: &C::Extra,
) -> Result<<C::Schema as Schema>::Input> {
    let f = flat.function(function)?;
    let index = f
        .params
        .iter()
        .position(|p| p.name == parameter)
        .ok_or_else(|| format!("`{function}`: unknown parameter `{parameter}`"))?;
    let mut plan = Compiler::new(flat, codec, format!("p{index}")).input(
        &f.params[index].ty.ty,
        recipe,
        extra,
        &Condition::default(),
    )?;
    codec.finish_input(&mut plan, extra)?;
    codec.validate_input(&plan)?;
    Ok(plan.into())
}

/// Compile a request-free description for one source result without modifying its slot.
/// ```
/// use prebindgen_declarations::{compile_output,Codec,Output};
/// use prebindgen_flat::Flat;
/// fn resolved<C:Codec>(flat:&Flat<C::Schema>,codec:&C,extra:&C::Extra) {
///     compile_output(flat,codec,"f",&Output::Leaf,extra).unwrap();
/// }
/// ```
/// ```compile_fail
/// use prebindgen_declarations::{compile_output,Codec,Output};
/// use prebindgen_flat::Flat;
/// fn pending<C:Codec>(flat:&Flat<C::Schema>,codec:&C,extra:&C::Extra) {
///     compile_output(flat,codec,"f",&Output::auto(),extra).unwrap();
/// }
/// ```
pub fn compile_output<C: Codec>(
    flat: &Flat<C::Schema>,
    codec: &C,
    function: &str,
    recipe: &Output,
    extra: &C::Extra,
) -> Result<<C::Schema as Schema>::Output> {
    compile_output_value(flat, codec, &flat.function(function)?.ret.ty, recipe, extra)
}

/// Compile the output conversion of a source value, such as a function result or an error.
/// The description must be request-free. Reads binding declarations from Flat without
/// introducing a synthetic source function or modifying an occurrence slot.
pub fn compile_output_value<C: Codec>(
    flat: &Flat<C::Schema>,
    codec: &C,
    ty: &Type,
    recipe: &Output,
    extra: &C::Extra,
) -> Result<<C::Schema as Schema>::Output> {
    compile_output_at(flat, codec, ty, recipe, extra, OutputPosition::Return)
}

/// Compile the foreign-to-Rust value returned by a source callback.
///
/// Uses the adapter's ordinary input conversions with an independent local namespace.
/// The request-free recipe comes from the binding's type-wide input declarations.
pub fn compile_callback_return<C: Codec>(
    flat: &Flat<C::Schema>,
    codec: &C,
    ty: &Type,
    recipe: &Input,
    extra: &C::Extra,
) -> Result<<C::Schema as Schema>::Input> {
    let mut plan = Compiler::new(flat, codec, "callback_return".into()).input(
        ty,
        recipe,
        extra,
        &Condition::default(),
    )?;
    codec.finish_input(&mut plan, extra)?;
    codec.validate_input(&plan)?;
    Ok(plan.into())
}

/// Compile one Rust-to-foreign callback argument with independent local names.
///
/// `index` is its position in the callback signature. The adapter supplies callback
/// delivery instead of the ordinary function-result packet; the same resolved
/// projection and ownership operations prepare the value in either position.
pub fn compile_callback_argument<C: Codec>(
    flat: &Flat<C::Schema>,
    codec: &C,
    ty: &Type,
    recipe: &Output,
    index: usize,
    borrowed: bool,
    extra: &C::Extra,
) -> Result<<C::Schema as Schema>::Output> {
    compile_output_at(
        flat,
        codec,
        ty,
        recipe,
        extra,
        OutputPosition::CallbackArgument { index, borrowed },
    )
}

enum OutputPosition {
    Return,
    CallbackArgument { index: usize, borrowed: bool },
}

fn compile_output_at<C: Codec>(
    flat: &Flat<C::Schema>,
    codec: &C,
    ty: &Type,
    recipe: &Output,
    extra: &C::Extra,
    position: OutputPosition,
) -> Result<<C::Schema as Schema>::Output> {
    let value = prebindgen_conversions::source_result_expr();
    let mut plan = match position {
        OutputPosition::Return => Compiler::new(flat, codec, "ret".into())
            .output_root(ty, recipe, extra, value, false, true)?,
        OutputPosition::CallbackArgument { index, borrowed } => {
            Compiler::new(flat, codec, format!("callback_arg_{index}"))
                .output_root(ty, recipe, extra, value, borrowed, false)?
        }
    };
    finalize_output_schema(codec, &mut plan, extra)?;
    codec.finish_output(&mut plan, extra)?;
    codec.validate_output(&plan)?;
    Ok(plan.into())
}

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
/// * `recipe` — preparation tree describing how to build the argument from boundary inputs.
///   To bind an already resolved [`Input`], pass `&resolved.into_preparation()`.
/// * `extra` — occurrence data passed to every codec call for this conversion.
///
/// Input binding rejects output recipes:
/// ```compile_fail
/// use prebindgen_declarations::{bind_input, Codec, Output};
/// use prebindgen_flat::Flat;
/// fn wrong<C: Codec>(flat: &mut Flat<C::Schema>, codec: &C, extra: &C::Extra) {
///     bind_input(flat, codec, "foo", "x", &Output::Leaf, extra).unwrap();
/// }
/// ```
pub fn bind_input<C: Codec>(
    flat: &mut Flat<C::Schema>,
    codec: &C,
    function: &str,
    parameter: &str,
    recipe: &Input<InputRequest>,
    extra: &C::Extra,
) -> Result<()> {
    let f = flat.function(function)?;
    let index = f
        .params
        .iter()
        .position(|p| p.name == parameter)
        .ok_or_else(|| format!("`{function}`: unknown parameter `{parameter}`"))?;
    let defaults = Declarations::new(flat, vec![], vec![], vec![])?;
    let recipe = defaults.resolve_input(flat, codec, &f.params[index].ty.ty, recipe, extra)?;
    let conversion = compile_input(flat, codec, function, parameter, &recipe, extra)?;
    flat.function_mut(function)?.params[index].ty.slot = Some(conversion);
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
/// * `recipe` — preparation tree describing how to deliver the result as boundary outputs.
///   To bind an already resolved [`Output`], pass `&resolved.into_preparation()`.
/// * `extra` — occurrence data passed to every codec call for this conversion.
///
/// Output binding rejects input recipes:
/// ```compile_fail
/// use prebindgen_declarations::{bind_output, Codec, Input};
/// use prebindgen_flat::Flat;
/// fn wrong<C: Codec>(flat: &mut Flat<C::Schema>, codec: &C, extra: &C::Extra) {
///     bind_output(flat, codec, "foo", &Input::Leaf, extra).unwrap();
/// }
/// ```
pub fn bind_output<C: Codec>(
    flat: &mut Flat<C::Schema>,
    codec: &C,
    function: &str,
    recipe: &Output<OutputRequest>,
    extra: &C::Extra,
) -> Result<()> {
    let defaults = Declarations::new(flat, vec![], vec![], vec![])?;
    let recipe =
        defaults.resolve_output(flat, codec, &flat.function(function)?.ret.ty, recipe, extra)?;
    let conversion = compile_output(flat, codec, function, &recipe, extra)?;
    flat.function_mut(function)?.ret.slot = Some(conversion);
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
    flat: &'a Flat<C::Schema>,
    codec: &'a C,
    prefix: String,
    next: std::cell::Cell<usize>,
}
impl<'a, C: Codec> Compiler<'a, C> {
    fn new(flat: &'a Flat<C::Schema>, codec: &'a C, prefix: String) -> Self {
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
        wires: &mut prebindgen_flat::WireNode<C::Wire, C::Chain, C::Branch>,
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
        self.input_named(ty, recipe, extra, active, None)
    }
    fn input_named(
        &self,
        ty: &Type,
        recipe: &Input,
        extra: &C::Extra,
        active: &Condition,
        names: Option<(&syn::Ident, &syn::Ident)>,
    ) -> Result<CodecInput<C>> {
        if prebindgen_flat::callback_signature(ty)?.is_some()
            && !matches!(recipe, Input::Leaf | Input::Callback(_))
        {
            return Err("source callback parameters only support leaf inputs".into());
        }
        if contains_result(ty) {
            return Err("Result inputs require an explicit data representation; error routing is output-only".into());
        }
        let var = names
            .map(|(_, var)| var.clone())
            .unwrap_or_else(|| self.name());
        let mut wires = Vec::new();
        let mut fallible = false;
        let mut validation = TokenStream::new();
        let mut acquire = TokenStream::new();
        let mut resources = Vec::new();
        let mut acknowledgement = None;
        let mut statements = TokenStream::new();
        let argument = quote!(#var);
        match recipe {
            Input::Callback(callback) => {
                callback.validate_for(self.flat, ty)?;
                return self.codec.callback_input(
                    self.flat,
                    callback,
                    extra,
                    names
                        .map(|(wire, _)| wire.clone())
                        .unwrap_or_else(|| self.name()),
                    var,
                    active,
                );
            }
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
                let tag = names
                    .map(|(wire, _)| format_ident!("{wire}_present"))
                    .unwrap_or_else(|| self.name());
                let child = if let Some((wire, var)) = names {
                    self.input_named(
                        ty,
                        inner,
                        extra,
                        &active.arm(tag.clone(), 1),
                        Some((&format_ident!("{wire}_some"), &format_ident!("{var}_some"))),
                    )?
                } else {
                    self.input(ty, inner, extra, &active.arm(tag.clone(), 1))?
                };
                return optional_input(self.codec, extra, tag, var, active, child);
            }
            Input::Leaf => {
                let w = names
                    .map(|(wire, _)| wire.clone())
                    .unwrap_or_else(|| self.name());
                return self.codec.input_leaf(self.flat, ty, extra, w, var, active);
            }
            Input::Request(never) => match *never {},
            Input::Fields(children) | Input::Record(children) => {
                if type_key(ty) == "()" && children.is_empty() {
                    return Ok(InputPlan {
                        fallible: false,
                        context: Default::default(),
                        acknowledgement,
                        node: WireNode::Branch {
                            controls: vec![],
                            children: vec![],
                            meta: self.codec.fields_group(),
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
                let grouped = matches!(recipe, Input::Record(_));
                let wire =
                    grouped.then(|| names.map(|(w, _)| w.clone()).unwrap_or_else(|| self.name()));
                let mut plan = record_input(
                    self.codec,
                    ty,
                    var.clone(),
                    r.fields
                        .iter()
                        .zip(children)
                        .enumerate()
                        .map(|(index, (field, child))| {
                            let plan = if let Some(wire) = &wire {
                                self.input_named(
                                    &field.ty,
                                    child,
                                    extra,
                                    active,
                                    Some((
                                        &format_ident!("{wire}_f{index}"),
                                        &format_ident!("{var}_f{index}"),
                                    )),
                                )?
                            } else {
                                self.input(&field.ty, child, extra, active)?
                            };
                            Ok((field.name.clone(), plan))
                        }),
                )?;
                if let Some(wire) = wire {
                    self.codec
                        .input_record_metadata(self.flat, ty, &wire, active, &mut plan, extra)?;
                }
                return Ok(plan);
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
                wires = vec![WireNode::Branch {
                    controls: vec![],
                    children: wires,
                    meta: self.codec.group_metadata(),
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
                let mut selector_layout = prebindgen_flat::WireNode::Leaf(vec![Wire {
                    name: tag.clone(),
                    kind: self.codec.selector(extra),
                }]);
                self.prefix(&mut selector_layout, ComponentSegment::Selector);
                let WireNode::Leaf(mut selector_values) = selector_layout else {
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
                let metadata = self.codec.alternatives_metadata(&selector)?;
                let node = WireNode::Branch {
                    controls: vec![selector],
                    meta: metadata,
                    children: alternatives,
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
        reference_return: bool,
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
            let message = self.codec.result_error_value(
                self.flat,
                error_ty,
                quote!(#failure),
                mapped,
                extra,
            )?;
            let mut child = if reference_return {
                self.output_return(success_ty, ok, extra, quote!(#payload), false)?
            } else {
                self.output(success_ty, ok, extra, quote!(#payload), false)?
            };
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
                .source_error_layout(self.flat, error_ty, mapped_error, extra)?;
            child.outcomes.source_error = Some(error);
            return Ok(child);
        }
        if reference_return {
            self.output_return(ty, recipe, extra, value, borrowed)
        } else {
            self.output(ty, recipe, extra, value, borrowed)
        }
    }
    // Preserve reference semantics for source returns and getter results.
    // The codec checks whether a getter's owner survives the generated call.
    fn output_return(
        &self,
        ty: &Type,
        recipe: &Output,
        extra: &C::Extra,
        value: TokenStream,
        borrowed: bool,
    ) -> Result<CodecOutput<C>> {
        if !borrowed {
            if matches!(ty, Type::Reference(_)) {
                if let Output::Combine(parts) = recipe {
                    if parts.is_empty() {
                        return Err("an output combination needs at least one part".into());
                    }
                    if mutable_reference(ty) && parts.len() > 1 {
                        return Err(
                            "a mutable reference cannot supply multiple output views".into()
                        );
                    }
                    let mut result = OutputPlan {
                        fallible: false,
                        context: Default::default(),
                        delivery: Default::default(),
                        commit: TokenStream::new(),
                        statements: TokenStream::new(),
                        values: vec![],
                        outcomes: prebindgen_flat::Outcomes {
                            success: WireNode::Branch {
                                controls: vec![],
                                children: vec![],
                                meta: self.codec.group_metadata(),
                            },
                            source_error: None,
                            binding_failure: None,
                        },
                    };
                    for (index, part) in parts.iter().enumerate() {
                        let mut child =
                            self.output_return(ty, part, extra, value.clone(), false)?;
                        self.prefix(&mut child.outcomes.success, ComponentSegment::View(index));
                        append(&mut result, child)?;
                    }
                    return Ok(result);
                }
            }
            if let (Type::Reference(reference), Output::Leaf) = (ty, recipe) {
                let wire = self.name();
                if let Some(plan) = self.codec.output_return_reference(
                    self.flat,
                    reference,
                    extra,
                    wire.clone(),
                    value.clone(),
                )? {
                    return Ok(plan);
                }
                return self.output_named(ty, recipe, extra, value, false, Some(&wire));
            }
            if let (Some(payload_ty), Output::Optional(inner)) = (option_inner(ty), recipe) {
                let tag = self.name();
                let payload = self.name();
                let child =
                    self.output_return(payload_ty, inner, extra, quote!(#payload), false)?;
                return optional_output(self.codec, extra, tag, payload, value, false, child);
            }
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
        self.output_named(ty, recipe, extra, value, borrowed, None)
    }
    fn output_named(
        &self,
        ty: &Type,
        recipe: &Output,
        extra: &C::Extra,
        value: TokenStream,
        borrowed: bool,
        wire_name: Option<&syn::Ident>,
    ) -> Result<CodecOutput<C>> {
        if let Some(referent) = output_referent(ty) {
            if matches!(recipe, Output::Projected(p) if p.mode == ProjectionMode::Owned) {
                return Err("owned projection cannot consume a reference payload".into());
            }
            let value = if borrowed { quote!(*(#value)) } else { value };
            return self.output_named(referent, recipe, extra, value, true, wire_name);
        }
        if contains_result(ty) {
            return Err("Result error routing is supported only at the outermost return with an explicit mapper".into());
        }
        let mut out = OutputPlan {
            fallible: false,
            context: Default::default(),
            delivery: Default::default(),
            commit: TokenStream::new(),
            outcomes: prebindgen_flat::Outcomes {
                success: WireNode::Branch {
                    controls: vec![],
                    children: vec![],
                    meta: self.codec.group_metadata(),
                },
                source_error: None,
                binding_failure: None,
            },
            statements: TokenStream::new(),
            values: Vec::new(),
        };
        match recipe {
            Output::Discard => return Ok(out),
            Output::Named { name, output } => {
                let mut plan = self.output_named(ty, output, extra, value, borrowed, wire_name)?;
                plan.outcomes.success = WireNode::Branch {
                    meta: self.codec.named_output_group(name)?,
                    controls: vec![],
                    children: vec![plan.outcomes.success],
                };
                return Ok(plan);
            }
            Output::Sequence {
                element,
                mode,
                delivery,
            } => {
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
                    child, *delivery,
                );
            }
            Output::Result { .. } => {
                return Err("Result error routing is supported only at the outermost return".into())
            }
            Output::Optional(inner) => {
                let ty = option_inner(ty).ok_or("optional recipe requires Option<T>")?;
                let tag = wire_name
                    .map(|w| format_ident!("{w}_present"))
                    .unwrap_or_else(|| self.name());
                let payload = wire_name
                    .map(|w| format_ident!("{w}_payload"))
                    .unwrap_or_else(|| self.name());
                let child_wire = wire_name.map(|w| format_ident!("{w}_some"));
                let child = self.output_named(
                    ty,
                    inner,
                    extra,
                    quote!(#payload),
                    borrowed,
                    child_wire.as_ref(),
                )?;
                return optional_output(self.codec, extra, tag, payload, value, borrowed, child);
            }
            Output::Request(never) => match *never {},
            Output::Leaf => {
                let w = wire_name.cloned().unwrap_or_else(|| self.name());
                return self
                    .codec
                    .output_leaf(self.flat, ty, extra, w, value, borrowed);
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
                return self.codec.output_projected(
                    self.flat,
                    ty,
                    extra,
                    wire_name.cloned().unwrap_or_else(|| self.name()),
                    value,
                    projection,
                );
            }
            Output::Fields(children)
            | Output::OwnedFields(children)
            | Output::Record {
                fields: children, ..
            } => {
                let grouped = matches!(recipe, Output::Record { .. });
                let wire = grouped.then(|| wire_name.cloned().unwrap_or_else(|| self.name()));
                let owned = match recipe {
                    Output::OwnedFields(_) => true,
                    Output::Record { mode, .. } => {
                        mode.map_or(!borrowed, |m| m == ProjectionMode::Owned)
                    }
                    _ => false,
                };
                if owned && borrowed {
                    return Err("owned fields cannot consume a shared-reference occurrence".into());
                }
                if type_key(ty) == "()" && children.is_empty() {
                    out.outcomes.success = WireNode::Branch {
                        controls: vec![],
                        meta: self.codec.fields_group(),
                        children: vec![],
                    };

                    return Ok(out);
                }
                let r = self.flat.record(ty)?;
                if r.fields.len() != children.len() {
                    return Err("wrong output field count".into());
                }
                let convert = |index: usize, value| {
                    let child_wire = wire.as_ref().map(|w| format_ident!("{w}_f{index}"));
                    self.output_named(
                        &r.fields[index].ty,
                        &children[index],
                        extra,
                        value,
                        !owned,
                        child_wire.as_ref(),
                    )
                };
                let var = wire
                    .as_ref()
                    .map(|w| format_ident!("{w}_record"))
                    .unwrap_or_else(|| self.name());
                let mut plan = if owned {
                    record_output_owned(
                        self.codec,
                        var,
                        value,
                        r.fields.iter().map(|f| f.name.clone()),
                        convert,
                    )?
                } else {
                    record_output(
                        self.codec,
                        var,
                        value,
                        r.fields.iter().map(|f| f.name.clone()),
                        convert,
                    )?
                };
                if let Some(wire) = wire {
                    self.codec
                        .output_record_metadata(self.flat, ty, &wire, &mut plan, extra)?;
                }
                return Ok(plan);
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
                if matches!(&f.ret.ty, Type::Reference(r) if r.mutability.is_some()) {
                    return Err("mutable-reference getter results are unsupported".into());
                }
                let v = self.name();
                let path = &f.path;
                let arg = if borrowed || consuming {
                    value
                } else {
                    quote!(&#value)
                };
                out.statements = quote!(let #v=#path(#arg););
                let mut child = self.output_return(&f.ret.ty, output, extra, quote!(#v), false)?;
                self.codec
                    .validate_getter_output(&f.ret.ty, &child, extra)?;
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
                    .filter(|p| p.consumes(self.flat, ty, extra, self.codec, borrowed))
                    .count()
                    > 1
                {
                    return Err("a combination cannot consume the same value twice; use field_self_ref() for an explicit copy view".into());
                }
                if !borrowed {
                    if let Some(plan) = self.output_owner_views(ty, parts, extra, value.clone())? {
                        return Ok(plan);
                    }
                }
                let mut bodies = Vec::new();
                for (index, child) in parts.iter().enumerate() {
                    let consuming = child.consumes(self.flat, ty, extra, self.codec, borrowed);
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

    fn output_owner_views(
        &self,
        ty: &Type,
        parts: &[Output],
        extra: &C::Extra,
        value: TokenStream,
    ) -> Result<Option<CodecOutput<C>>> {
        let owner = parts.iter().position(|part| {
            matches!(part,
            Output::Projected(p) if p.mode == ProjectionMode::Owned && p.fields.is_empty())
                || matches!(part, Output::Leaf) && self.codec.consumes(self.flat, ty, extra)
        });
        let Some(owner) = owner else {
            return Ok(None);
        };
        if parts.len() < 2 {
            return Ok(None);
        }
        // Reserve the same owner wire that the ordinary consuming leaf uses.
        let next = self.next.get();
        let wire = self.name();
        let Some((mut owned, pointer)) =
            self.codec.output_owner(self.flat, ty, extra, wire, value)?
        else {
            self.next.set(next);
            return Ok(None);
        };
        if parts
            .iter()
            .enumerate()
            .any(|(i, part)| i != owner && !matches!(part, Output::Getter { .. }))
        {
            return Err("an owner-backed output combination supports only the owning identity and getter views".into());
        }
        let prepare_owner = std::mem::take(&mut owned.statements);
        let mut result = OutputPlan {
            fallible: false,
            context: Default::default(),
            delivery: Default::default(),
            commit: TokenStream::new(),
            statements: TokenStream::new(),
            values: vec![],
            outcomes: prebindgen_flat::Outcomes {
                success: WireNode::Branch {
                    controls: vec![],
                    children: vec![],
                    meta: self.codec.group_metadata(),
                },
                source_error: None,
                binding_failure: None,
            },
        };
        let mut owner_plan = Some(owned);
        for (index, part) in parts.iter().enumerate() {
            let mut plan = if index == owner {
                owner_plan.take().unwrap()
            } else {
                let Output::Getter { function, output } = part else {
                    unreachable!()
                };
                let f = self.flat.function(function)?;
                if f.params.len() != 1 {
                    return Err(format!(
                        "getter `{function}` must borrow its returned owner"
                    ));
                }
                let receiver = &f.params[0].ty.ty;
                let mutable_receiver =
                    matches!(receiver, Type::Reference(r) if r.mutability.is_some());
                let mutable = mutable_receiver || mutable_reference(&f.ret.ty);
                if mutable && parts.len() != 2 {
                    return Err("a mutable owner getter must be the only getter view; simultaneous owner borrows could invalidate earlier views".into());
                }
                let expected: Type = if mutable_receiver {
                    syn::parse_quote!(&mut #ty)
                } else {
                    syn::parse_quote!(&#ty)
                };
                if !same_type(receiver, &expected) {
                    return Err(format!(
                        "getter `{function}` must borrow its returned owner"
                    ));
                }
                let source = self.name();
                let path = &f.path;
                let argument = if mutable_receiver {
                    quote!(unsafe { &mut *(#pointer) })
                } else {
                    quote!(unsafe { &*(#pointer) })
                };
                let mut plan =
                    self.output_return(&f.ret.ty, output, extra, quote!(#source), false)?;
                plan.statements = {
                    let body = plan.statements;
                    quote!(let #source = #path(#argument); #body)
                };
                self.prefix(
                    &mut plan.outcomes.success,
                    ComponentSegment::Getter(function.clone()),
                );
                plan
            };
            self.prefix(&mut plan.outcomes.success, ComponentSegment::View(index));
            append(&mut result, plan)?;
        }
        result.statements = {
            let views = result.statements;
            quote!(#prepare_owner #views)
        };
        Ok(Some(result))
    }
}
fn mutable_reference(ty: &Type) -> bool {
    matches!(ty, Type::Reference(r) if r.mutability.is_some())
        || option_inner(ty).is_some_and(mutable_reference)
}
fn validate_nested_output<W: prebindgen_flat::WireKind, C, N, B>(
    plan: &OutputPlan<W, C, N, B>,
) -> Result<()> {
    if plan.outcomes.source_error.is_some() || plan.outcomes.binding_failure.is_some() {
        return Err("completion channels belong to the outermost output conversion".into());
    }
    Ok(())
}
fn append<W: prebindgen_flat::WireKind, C, N, B>(
    to: &mut OutputPlan<W, C, N, B>,
    from: OutputPlan<W, C, N, B>,
) -> Result<()> {
    if !from.delivery.statements.is_empty()
        || from.delivery.result.to_string() != quote!(()).to_string()
    {
        return Err("composed output delivery must be installed by Codec::finish_output".into());
    }
    validate_nested_output(&from)?;
    let WireNode::Branch {
        children: elements, ..
    } = &mut to.outcomes.success
    else {
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
pub use declarations::{CallbackReturn, Declarations, FunctionDecl};

/// Parse one Rust identifier used by a declaration.
///
/// Returns an error for invalid syntax, including paths or multiple identifiers.
pub fn ident(name: &str) -> Result<syn::Ident> {
    syn::parse_str(name).map_err(|_| format!("invalid identifier `{name}`"))
}

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
