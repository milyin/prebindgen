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
    prepare_optional_payload, retire_optional_niches, select_optional_niche, NicheReservation,
    PreparedOptionalPayload,
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
// The owned value deliberately constructed by Input::Temporary. A Vec/String
// descriptor must never be fabricated over memory merely borrowed from C.
fn temporary_input_type(ty: &Type) -> Result<Type> {
    let Type::Reference(reference) = ty else {
        return Err("reconstruct() requires a shared-reference input; apply it to the optional payload if necessary".into());
    };
    if reference.mutability.is_some()
        || reference
            .lifetime
            .as_ref()
            .is_some_and(|l| l.ident == "static")
    {
        return Err("reconstruct() cannot supply a mutable or 'static reference".into());
    }
    Ok(match reference.elem.as_ref() {
        Type::Slice(slice) => {
            let element = &slice.elem;
            syn::parse_quote!(Vec<#element>)
        }
        Type::Path(path) if path.path.is_ident("str") => syn::parse_quote!(String),
        other => other.clone(),
    })
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
    /// Return a collection using the adapter's default materialized transport.
    Auto,
    /// Explicitly return ordinary expanded rows, without applying a transport default.
    #[default]
    Value,
    /// Deliver each element through the function's sequence callback.
    /// Bulk packing inside an element stays inside this delivery boundary.
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
    /// Explicitly construct an owned temporary, then lend it as a shared input.
    /// The child describes the owned value, including any transferred resources.
    /// The temporary is dropped after the source call; it cannot back an escaping result.
    Temporary(Box<Input<P>>),
    /// Group the expanded fields into an adapter-selected C-representation bulk carrier.
    Bulk(Box<Input<P>>),
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
    /// Explicitly reconstruct an owned temporary for a shared-reference input.
    /// Slices construct a Vec, str constructs a String, and sized references
    /// construct their referent. Child recipes keep their ordinary ownership
    /// semantics; this operation authorizes the temporary's allocation/copying.
    pub fn reconstruct(self) -> Self {
        match self {
            Self::Request(InputRequest::ForType { ty, input }) => {
                Self::for_type(*ty, Self::Temporary(input))
            }
            other => Self::Temporary(Box::new(other)),
        }
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
    /// Group this explicit expansion into a bulk carrier.
    pub fn bulk(self) -> Self {
        Self::Bulk(Box::new(self))
    }
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
    /// Group the expanded fields into an adapter-selected C-representation bulk carrier.
    Bulk(Box<Output<P>>),
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
        /// An owning opaque copy requires an explicit shared projection or clone
        /// accessor; Rust then checks `Clone`.
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
    /// Group this explicit expansion into a bulk carrier.
    pub fn bulk(self) -> Self {
        Self::Bulk(Box::new(self))
    }
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
    /// framework uses [`Codec::output_consumes`] to order observations before a move,
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
                None => !borrowed && sequence_parts(ty).is_some_and(|(_, view)| view.is_owned()),
            },
            Self::Bulk(child) => child.consumes(flat, ty, extra, codec, borrowed),
            Self::Named { output: child, .. } => child.consumes(flat, ty, extra, codec, borrowed),
            Self::Optional(child) => {
                child.consumes(flat, option_inner(ty).unwrap_or(ty), extra, codec, borrowed)
            }
            Self::Leaf => {
                !borrowed && output_referent(ty).is_none() && codec.output_consumes(flat, ty, extra)
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

mod codec;
pub use codec::{prepare_mapped_source_error, Codec};
mod operations;
pub use operations::*;

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
    let compiler = Compiler {
        input_role: InputRole::CallbackResult,
        ..Compiler::new(flat, codec, "callback_return".into())
    };
    let mut plan = compiler.input(ty, recipe, extra, &Condition::default())?;
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
                .output_root(ty, recipe, extra, value, borrowed, true)?
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
    input_role: InputRole,
    bulk: std::cell::Cell<bool>,
    // Explicit borrowed collection/record projections authorize owning copies
    // of borrowed leaves, without changing conversions of owned getter results.
    copy_borrowed: std::cell::Cell<bool>,
    // Row temporaries may borrow only the enclosing sequence's cached frame,
    // never a separate bulk field decoded inside an ordinary JVM object row.
    ordinary_collections: std::cell::Cell<bool>,
    // A Combine observation borrows a Rust temporary, not persistent foreign storage.
    temporary_receiver: std::cell::Cell<bool>,
    flat: &'a Flat<C::Schema>,
    codec: &'a C,
    prefix: String,
    next: std::cell::Cell<usize>,
}
impl<'a, C: Codec> Compiler<'a, C> {
    fn new(flat: &'a Flat<C::Schema>, codec: &'a C, prefix: String) -> Self {
        Self {
            input_role: InputRole::Parameter,
            bulk: std::cell::Cell::new(false),
            copy_borrowed: std::cell::Cell::new(false),
            ordinary_collections: std::cell::Cell::new(false),
            temporary_receiver: std::cell::Cell::new(false),
            flat,
            codec,
            prefix,
            next: std::cell::Cell::new(0),
        }
    }
    fn with_temporary_receiver<T>(
        &self,
        temporary: bool,
        prepare: impl FnOnce() -> Result<T>,
    ) -> Result<T> {
        let previous = self.temporary_receiver.get();
        self.temporary_receiver.set(previous || temporary);
        let result = prepare();
        self.temporary_receiver.set(previous);
        result
    }
    fn with_copy_permission<T>(
        &self,
        explicit: bool,
        prepare: impl FnOnce() -> Result<T>,
    ) -> Result<T> {
        let previous = self
            .copy_borrowed
            .replace(self.copy_borrowed.get() || explicit);
        let result = prepare();
        self.copy_borrowed.set(previous);
        result
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
        self.input_unprepared_named(ty, recipe, extra, active, names)
    }

    fn input_unprepared_named(
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
        match recipe {
            Input::Temporary(inner) => {
                let owned = temporary_input_type(ty)?;
                let mut child = self.input_named(&owned, inner, extra, active, names)?;
                let storage = self.name();
                let target = format_ident!("{storage}_owned");
                let mut backing = std::mem::take(&mut child.storage);
                backing.extend(quote!(#[allow(unused_mut,unused_assignments)] let mut #storage:Option<#owned>=None;));
                let build = child.bind(&target, quote!(#storage=Some(#target);))?;
                let Type::Reference(reference) = ty else {
                    unreachable!()
                };
                let elem = &reference.elem;
                let value = if matches!(elem.as_ref(), Type::Slice(_))
                    || matches!(elem.as_ref(),Type::Path(p) if p.path.is_ident("str"))
                {
                    quote!(::core::convert::AsRef::<#elem>::as_ref(#storage.as_ref().expect("active temporary")))
                } else {
                    quote!(#storage.as_ref().expect("active temporary"))
                };
                let mut plan = InputPlan::new(
                    WireNode::Branch {
                        controls: vec![],
                        children: vec![child.node],
                        meta: self.codec.group(GroupKind::Values)?,
                    },
                    quote!(#build;),
                    value,
                );
                plan.storage = backing;
                plan.fallible = child.fallible;
                Ok(plan)
            }
            Input::Bulk(inner) => {
                require_bulk(self.codec, Direction::Input)?;
                if self.bulk.get() {
                    return self.input_named(ty, inner, extra, active, names);
                }
                let wire = self.name();
                self.bulk.set(true);
                let child = self.input_named(ty, inner, extra, active, names);
                self.bulk.set(false);
                self.codec
                    .bulk_input(self.flat, ty, extra, wire, child?, active)
            }
            Input::Callback(callback) => {
                callback.validate_for(self.flat, ty)?;
                self.codec.callback_input(
                    self.flat,
                    callback,
                    extra,
                    names
                        .map(|(wire, _)| wire.clone())
                        .unwrap_or_else(|| self.name()),
                    var,
                    active,
                )
            }
            Input::Sequence(element_recipe) => {
                if matches!(ty, Type::Reference(_)) {
                    let next = self.next.get();
                    let wire = names
                        .map(|(wire, _)| wire.clone())
                        .unwrap_or_else(|| self.name());
                    if let Some(plan) = self.codec.input_borrowed_sequence(
                        self.flat,
                        ty,
                        extra,
                        wire,
                        var.clone(),
                        element_recipe,
                        active,
                    )? {
                        return Ok(plan);
                    }
                    self.next.set(next);
                }
                if !self.bulk.get() {
                    if let Input::Bulk(inner) = element_recipe.as_ref() {
                        return self.input_named(
                            ty,
                            &Input::Bulk(Box::new(Input::Sequence(inner.clone()))),
                            extra,
                            active,
                            names,
                        );
                    }
                }
                let (element, _) =
                    sequence_parts(ty).ok_or("sequence input requires Vec<T>, &[T] or &Vec<T>")?;
                let wire = self.name();
                let length = self.name();
                let index = self.name();
                let child = self.input(element, element_recipe, extra, &Condition::default());
                let child = child?;
                sequence::input(
                    self.codec,
                    ty,
                    extra,
                    wire,
                    var,
                    length,
                    index,
                    active,
                    child,
                    self.bulk.get(),
                )
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
                optional_input(self.codec, extra, tag, var, active, child)
            }
            Input::Leaf => {
                let w = names
                    .map(|(wire, _)| wire.clone())
                    .unwrap_or_else(|| self.name());
                let representation = if self.bulk.get() {
                    Representation::Bulk
                } else {
                    Representation::Ordinary
                };
                if representation == Representation::Bulk {
                    require_bulk(self.codec, Direction::Input)?;
                }
                self.codec.input_leaf(InputLeaf {
                    flat: self.flat,
                    ty,
                    extra,
                    wire: w,
                    target: var,
                    active,
                    representation,
                    role: self.input_role,
                })
            }
            Input::Request(never) => match *never {},
            Input::Fields(children) | Input::Record(children) => {
                if type_key(ty) == "()" && children.is_empty() {
                    return Ok(InputPlan::new(
                        WireNode::Branch {
                            controls: vec![],
                            children: vec![],
                            meta: self.codec.group(GroupKind::Fields)?,
                        },
                        quote!(),
                        quote!(()),
                    ));
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
                Ok(plan)
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
                let mut plans = Vec::new();
                let mut call_args = Vec::new();
                let mut wires = Vec::new();
                let mut storage = TokenStream::new();
                let mut fallible = false;
                for (i, (param, child)) in f.params.iter().zip(args).enumerate() {
                    let mut p = self.input(&param.ty.ty, child, extra, active)?;
                    self.prefix(
                        &mut p.node,
                        ComponentSegment::Argument(param.name.to_string()),
                    );
                    fallible |= p.fallible;
                    storage.extend(std::mem::take(&mut p.storage));
                    let target = format_ident!("{var}_argument_{i}");
                    call_args.push(target.clone());
                    plans.push((target, p));
                }
                let path = &f.path;
                let mut plan = InputPlan::new(
                    WireNode::default(),
                    quote!(let #var:#ty=#path(#(#call_args),*);),
                    quote!(#var),
                );
                for (target, p) in plans.iter().rev() {
                    plan.code = p.bind(target, plan.code)?;
                }
                for (_, p) in plans {
                    wires.push(p.node);
                }
                plan.node = WireNode::Branch {
                    controls: vec![],
                    children: wires,
                    meta: self.codec.group(GroupKind::Values)?,
                };
                plan.storage = storage;
                plan.fallible = fallible;
                Ok(plan)
            }
            Input::Selector(arms) => {
                let mut fallible = self.codec.validates_inputs();
                let mut storage = TokenStream::new();
                let mut validation = TokenStream::new();
                if arms.is_empty() {
                    return Err("an input selector needs at least one alternative".into());
                }
                if matches!(ty, Type::Reference(_)) {
                    return Err("selector results must be owned in this subset".into());
                }
                let tag = self.name();
                let mut selector_layout = prebindgen_flat::WireNode::Leaf(vec![Wire {
                    name: tag.clone(),
                    kind: self.codec.selector(extra)?,
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
                if self.codec.validates_inputs() {
                    validation.extend(quote!(if #invalid {return Err("invalid selector".into());}));
                }
                let mut cases = Vec::new();
                for (i, arm) in arms.iter().enumerate() {
                    let mut p = self.input(ty, arm, extra, &active.arm(tag.clone(), i as i32))?;
                    self.prefix(&mut p.node, ComponentSegment::Alternative(i));
                    fallible |= p.fallible;
                    storage.extend(std::mem::take(&mut p.storage));
                    let target = format_ident!("{var}_alternative_{i}");
                    let body = p.bind(&target, quote!(#target))?;
                    alternatives.push(p.node);
                    let i = i as i32;
                    cases.push(quote!(#i=>#body));
                }
                let metadata = self.codec.alternatives(Alternatives {
                    selector: &selector,
                    arms: &mut alternatives,
                })?;
                let node = WireNode::Branch {
                    controls: vec![selector],
                    meta: metadata,
                    children: alternatives,
                };

                let invalid = if self.codec.validates_inputs() {
                    quote!(unreachable!("validated selector"))
                } else {
                    quote!(unsafe { ::core::hint::unreachable_unchecked() })
                };
                let mut plan = InputPlan::new(
                    node,
                    quote!(#validation let #var:#ty=match #tag {#(#cases,)* _=>#invalid};),
                    quote!(#var),
                );
                plan.storage = storage;
                plan.fallible = fallible;
                Ok(plan)
            }
        }
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
            let error = self.codec.source_error(SourceError {
                flat: self.flat,
                ty: error_ty,
                value: quote!(#failure),
                message: mapped,
                extra,
            })?;
            let message = error.propagate;
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

            child.outcomes.source_error = Some(error.payload);
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
                                meta: self.codec.group(GroupKind::Values)?,
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
                    meta: self.codec.group(GroupKind::Values)?,
                },
                source_error: None,
                binding_failure: None,
            },
            statements: TokenStream::new(),
            values: Vec::new(),
        };
        match recipe {
            Output::Bulk(inner) => {
                require_bulk(self.codec, Direction::Output)?;
                if self.bulk.get() {
                    return self.output_named(ty, inner, extra, value, borrowed, wire_name);
                }
                let wire = self.name();
                self.bulk.set(true);
                let child = self.output_named(ty, inner, extra, value, borrowed, wire_name);
                self.bulk.set(false);
                return self.codec.bulk_output(self.flat, ty, extra, wire, child?);
            }
            Output::Discard => return Ok(out),
            Output::Named { name, output } => {
                let mut plan = self.output_named(ty, output, extra, value, borrowed, wire_name)?;
                plan.outcomes.success = WireNode::Branch {
                    meta: self.codec.group(GroupKind::NamedOutput(name))?,
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
                if self.bulk.get() && *delivery == SequenceDelivery::Callback {
                    return Err("bulk cannot enclose callback delivery; apply bulk to the callback element expansion instead".into());
                }
                if *delivery == SequenceDelivery::Auto
                    && !self.bulk.get()
                    && !self.ordinary_collections.get()
                    && self.codec.default_output_sequence_is_bulk(self.flat, extra)
                {
                    return self.output_named(
                        ty,
                        &Output::Bulk(Box::new(recipe.clone())),
                        extra,
                        value,
                        borrowed,
                        wire_name,
                    );
                }
                if !self.bulk.get() && *delivery != SequenceDelivery::Callback {
                    if let Output::Bulk(inner) = element.as_ref() {
                        return self.output_named(
                            ty,
                            &Output::Bulk(Box::new(Output::Sequence {
                                element: inner.clone(),
                                mode: *mode,
                                delivery: *delivery,
                            })),
                            extra,
                            value,
                            borrowed,
                            wire_name,
                        );
                    }
                }
                let (element_type, _) =
                    sequence_parts(ty).ok_or("sequence output requires a collection")?;
                let explicit_copy = *mode == Some(ProjectionMode::Borrowed);
                let mode = mode.unwrap_or_else(|| {
                    if borrowed || !sequence_parts(ty).unwrap().1.is_owned() {
                        ProjectionMode::Borrowed
                    } else {
                        ProjectionMode::Owned
                    }
                });
                let wire = self.name();
                if mode == ProjectionMode::Borrowed && !explicit_copy && !self.copy_borrowed.get() {
                    if let Some(plan) = self.codec.output_borrowed_sequence(
                        self.flat,
                        ty,
                        extra,
                        wire.clone(),
                        value.clone(),
                        element,
                        *delivery,
                        self.temporary_receiver.get(),
                    )? {
                        return Ok(plan);
                    }
                }
                let payload = self.name();
                let length = self.name();
                let index = self.name();
                // An explicit ordinary collection (or a callback row) keeps
                // nested automatic sequences ordinary. Explicit Bulk still wins.
                let ordinary = self.ordinary_collections.replace(
                    self.ordinary_collections.get() || *delivery != SequenceDelivery::Auto,
                );
                let child = self.with_copy_permission(explicit_copy, || {
                    self.with_temporary_receiver(
                        mode == ProjectionMode::Borrowed
                            && !borrowed
                            && sequence_parts(ty).unwrap().1.is_owned(),
                        || {
                            self.output(
                                element_type,
                                element,
                                extra,
                                quote!(#payload),
                                mode == ProjectionMode::Borrowed,
                            )
                        },
                    )
                });
                self.ordinary_collections.set(ordinary);
                let child = child?;
                return sequence::output(
                    self.codec,
                    ty,
                    extra,
                    wire,
                    payload,
                    length,
                    index,
                    value,
                    borrowed,
                    mode,
                    child,
                    if *delivery == SequenceDelivery::Auto {
                        SequenceDelivery::Value
                    } else {
                        *delivery
                    },
                    self.bulk.get(),
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
                if self.bulk.get() {
                    require_bulk(self.codec, Direction::Output)?;
                }
                let request = OutputLeaf {
                    flat: self.flat,
                    ty,
                    extra,
                    wire: wire_name.cloned().unwrap_or_else(|| self.name()),
                    value,
                    access: if borrowed {
                        ValueAccess::Shared
                    } else {
                        ValueAccess::Owned
                    },
                    representation: if self.bulk.get() {
                        Representation::Bulk
                    } else {
                        Representation::Ordinary
                    },
                };
                return if borrowed && self.copy_borrowed.get() {
                    self.codec
                        .output_projected(request, &Projection::borrowed())
                } else {
                    self.codec.output_leaf(request)
                };
            }
            Output::Projected(projection) => {
                if self.bulk.get() {
                    require_bulk(self.codec, Direction::Output)?;
                }
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
                    OutputLeaf {
                        flat: self.flat,
                        ty,
                        extra,
                        wire: wire_name.cloned().unwrap_or_else(|| self.name()),
                        value,
                        access: if projection.mode == ProjectionMode::Borrowed {
                            ValueAccess::Shared
                        } else {
                            ValueAccess::Owned
                        },
                        representation: if self.bulk.get() {
                            Representation::Bulk
                        } else {
                            Representation::Ordinary
                        },
                    },
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
                        meta: self.codec.group(GroupKind::Fields)?,
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
                let explicit_copy = matches!(
                    recipe,
                    Output::Record {
                        mode: Some(ProjectionMode::Borrowed),
                        ..
                    }
                );
                let mut plan = self.with_copy_permission(explicit_copy, || {
                    self.with_temporary_receiver(!owned && !borrowed, || {
                        if owned {
                            record_output_owned(
                                self.codec,
                                var,
                                value,
                                r.fields.iter().map(|f| f.name.clone()),
                                convert,
                            )
                        } else {
                            record_output(
                                self.codec,
                                var,
                                value,
                                r.fields.iter().map(|f| f.name.clone()),
                                convert,
                            )
                        }
                    })
                })?;
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
                self.codec.validate_getter_output(
                    self.flat,
                    ty,
                    borrowed && !self.temporary_receiver.get(),
                    &f.ret.ty,
                    &child,
                    extra,
                )?;
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
                    let mut p = self.with_temporary_receiver(!borrowed && !consuming, || {
                        self.output(ty, child, extra, arg, borrowed || !consuming)
                    })?;
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
                || matches!(part, Output::Leaf) && self.codec.output_consumes(self.flat, ty, extra)
        });
        let Some(owner) = owner else {
            return Ok(None);
        };
        if parts.len() < 2 {
            return Ok(None);
        }
        // Explicit copy projections use observation-before-move ordering. They
        // do not require an address-preserving loan from the delivered owner.
        if parts
            .iter()
            .any(|part| matches!(part,Output::Projected(p) if p.mode==ProjectionMode::Borrowed))
        {
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
                    meta: self.codec.group(GroupKind::Values)?,
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
                self.codec
                    .validate_owner_getter(self.flat, ty, function, &plan)?;
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

fn require_bulk<C: Codec + ?Sized>(codec: &C, direction: Direction) -> Result<()> {
    if codec.supports_bulk(direction) {
        Ok(())
    } else {
        Err(match direction {
            Direction::Input => "bulk inputs are unsupported by this codec",
            Direction::Output => "bulk outputs are unsupported by this codec",
        }
        .into())
    }
}

#[cfg(test)]
mod operation_tests;
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

#[cfg(test)]
mod niche_tests;
