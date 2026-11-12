//! Declaration language and composition for preparing conversion slots.
//!
//! [`Declarations::resolve`] produces a request-free [`Flat<Resolved<C>>`](Resolved)
//! that owns all data needed by [`compile_resolved`]. Input and output trees keep
//! separate operation vocabularies; [`InputNode::Arrow`] and [`OutputNode::Arrow`]
//! expose reverse-direction domains structurally. [`Declarations::fill`],
//! [`bind_input`] and [`bind_output`] use the same resolver and compiler for
//! individual occurrences. Compiled slots retain wires and Rust code for emission.
//! Generation is owned by the language generator;
//! this crate owns only declaration resolution and slot preparation.
//! Each [`Wire`] carries the adapter's own description in [`Wire::kind`].
//! Reusable value and boxed-allocation leaves live in `prebindgen-conversions`.
#![warn(missing_docs)]
mod convert;
pub use convert::{prepare_value_conversions, ConvertDecl, ConvertSource};

use prebindgen_conversions::{InputPlan, OutputPlan};
use prebindgen_flat::{same_type, type_key, Flat, Result, Wire};
use prebindgen_flat::{Schema, WireNode};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Type;
mod builders;
mod callback;
pub use callback::{CallbackArgumentOwnership, CallbackDecl, CallbackOperation};
mod node;
mod request_conversion;
mod resolved;
pub use builders::{ErrorDecl, ExpandDecl, ExpandParamDecl, ExpandReturnDecl, FieldsDecl};
pub use node::*;
pub use resolved::{compile_resolved, Resolved, ResolvedModule, ResolvedSlot};
mod enums;
pub use enums::{enum_input, enum_output, EnumMapping};
mod optional;
mod variants;
pub use optional::{
    absent_output_values, optional_input, optional_output, optional_presence,
    prepare_optional_payload, retire_optional_niches, select_optional_niche, NicheReservation,
    PreparedOptionalPayload,
};
pub use prebindgen_flat::{option_inner, result_parts, sequence_parts, SequenceView};
mod sequence;
pub use sequence::{SequenceInput, SequenceOutput};
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
// The owned value deliberately constructed by InputUnary::Temporary. A Vec/String
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

/// Source-dependent choices accepted only while preparing an input tree.
#[derive(Clone, Debug)]
pub enum InputRequest {
    /// Expand enum fields, inheriting defaults for fields not explicitly overridden.
    EnumFields(
        std::collections::BTreeMap<
            String,
            std::collections::BTreeMap<String, Input<InputRequest, OutputRequest>>,
        >,
    ),
    /// Resolve only the canonical value form, independently of boundary defaults.
    Value,
    /// Select a local single-value conversion.
    Conversion(Box<ConvertDecl>),
    /// Select the configured default for this source occurrence.
    Auto,
    /// Select or override a callback signature's argument and result conversions.
    Callback(Box<CallbackDecl>),
    /// Apply a type-specific override, including inside optional/sequence sources.
    ForType {
        /// Source type targeted by the override.
        ty: Box<Type>,
        /// The requested conversion for that type.
        input: Box<Input<InputRequest, OutputRequest>>,
    },
    /// Infer a constructor's argument conversions from its source signature.
    Call(String),
}

impl Input<InputRequest, OutputRequest> {
    /// Expand the tag and selected fields. Keys are source variant and field
    /// names (or tuple indices). Unspecified fields inherit boundary defaults.
    pub fn enum_fields<'a>(
        overrides: impl IntoIterator<Item = (&'a str, Vec<(&'a str, Self)>)>,
    ) -> Self {
        Input::Request(InputRequest::EnumFields(variants::overrides(overrides)))
    }
    /// Request the canonical value representation, bypassing boundary defaults.
    pub fn value() -> Self {
        Input::Request(InputRequest::Value)
    }
    /// Request this source type's configured default during preparation.
    pub fn auto() -> Self {
        Input::Request(InputRequest::Auto)
    }
    /// Associate a conversion with a source type for a default or nested override.
    pub fn for_type(ty: Type, input: Self) -> Self {
        Input::Request(InputRequest::ForType {
            ty: Box::new(ty),
            input: Box::new(input),
        })
    }
    /// Infer a source constructor's argument conversions from registered defaults.
    pub fn call(name: impl Into<String>) -> Self {
        Input::Request(InputRequest::Call(name.into()))
    }
    /// Explicitly reconstruct an owned temporary for a shared-reference input.
    /// Slices construct a Vec, str constructs a String, and sized references
    /// construct their referent. Child recipes keep their ordinary ownership
    /// semantics; this operation authorizes the temporary's allocation/copying.
    pub fn reconstruct(self) -> Self {
        match self {
            Input::Request(InputRequest::ForType { ty, input }) => Input::for_type(
                *ty,
                Input::Unary {
                    operation: InputUnary::Temporary,
                    child: input,
                },
            ),
            other => Input::Unary {
                operation: InputUnary::Temporary,
                child: Box::new(other),
            },
        }
    }
    /// Add a constructor alternative to this conversion tree.
    pub fn variant(self, name: impl Into<String>) -> Self {
        self.add_alternative(Input::call(name))
    }
    /// Add a direct codec alternative, bypassing this type's expansion default.
    pub fn variant_self(self) -> Self {
        self.add_alternative(Input::Leaf(InputLeafOp::Value))
    }
    fn add_alternative(self, arm: Self) -> Self {
        match self {
            Input::Request(InputRequest::ForType { ty, input }) => {
                Input::for_type(*ty, input.add_alternative(arm))
            }
            Input::Branch {
                operation: InputBranch::Selector,
                children: mut arms,
            } => {
                arms.push(arm);
                Input::Branch {
                    operation: InputBranch::Selector,
                    children: arms,
                }
            }
            other => Input::Branch {
                operation: InputBranch::Selector,
                children: vec![other, arm],
            },
        }
    }
}
impl<P, Q> Input<P, Q> {
    /// Group this explicit expansion into a bulk carrier.
    pub fn bulk(self) -> Self {
        Input::Unary {
            operation: InputUnary::Bulk,
            child: Box::new(self),
        }
    }
    /// Construct a record from field recipes, or unit from an empty list.
    ///
    /// `fields` holds one recipe per field of the source record, in declaration
    /// order. Binding verifies the count against the record.
    pub fn fields(fields: impl IntoIterator<Item = Input<P, Q>>) -> Self {
        Input::Branch {
            operation: InputBranch::Fields,
            children: fields.into_iter().collect(),
        }
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
    pub fn constructor(
        name: impl Into<String>,
        args: impl IntoIterator<Item = Input<P, Q>>,
    ) -> Self {
        Input::Branch {
            operation: InputBranch::Constructor {
                function: name.into(),
            },
            children: args.into_iter().collect(),
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
    pub fn selector(arms: impl IntoIterator<Item = Input<P, Q>>) -> Self {
        Input::Branch {
            operation: InputBranch::Selector,
            children: arms.into_iter().collect(),
        }
    }
}
/// Source-dependent choices accepted only while preparing an output tree.
#[derive(Clone, Debug)]
pub enum OutputRequest {
    /// Expand enum fields, inheriting boundary defaults for unspecified fields.
    EnumFields(
        std::collections::BTreeMap<
            String,
            std::collections::BTreeMap<String, Output<OutputRequest, InputRequest>>,
        >,
    ),
    /// Resolve only the canonical value form, independently of boundary defaults.
    Value {
        /// Explicit field ownership/projection policy, if any.
        projection: Option<Projection>,
    },
    /// Select a local single-value conversion.
    Conversion(Box<ConvertDecl>),
    /// Select the configured output default for this source occurrence.
    Auto,
    /// Apply a type-specific override, including inside optional/sequence sources.
    ForType {
        /// Source type targeted by the override.
        ty: Box<Type>,
        /// Requested conversion for the matching type.
        output: Box<Output<OutputRequest, InputRequest>>,
    },
    /// Resolve source record fields by name and fill unspecified conversions.
    NamedFields {
        /// Explicit conversions keyed by source field name.
        overrides: std::collections::BTreeMap<String, Output<OutputRequest, InputRequest>>,
        /// Ownership of field decomposition.
        mode: ProjectionMode,
    },
}

impl Output<OutputRequest, InputRequest> {
    /// Expand an enum's tag and active fields using source variant/member keys.
    pub fn enum_fields<'a>(
        overrides: impl IntoIterator<Item = (&'a str, Vec<(&'a str, Self)>)>,
    ) -> Self {
        Output::Request(OutputRequest::EnumFields(variants::overrides(overrides)))
    }
    /// Request the canonical value representation, bypassing boundary defaults.
    pub fn value() -> Self {
        Output::Request(OutputRequest::Value { projection: None })
    }
    /// Request this source type's configured output default during preparation.
    pub fn auto() -> Self {
        Output::Request(OutputRequest::Auto)
    }
    /// Associate a conversion with a source type for a default or nested override.
    pub fn for_type(ty: Type, output: Self) -> Self {
        Output::Request(OutputRequest::ForType {
            ty: Box::new(ty),
            output: Box::new(output),
        })
    }
    /// Append another simultaneous output view.
    pub fn view(self, view: impl Into<Self>) -> Self {
        let view = view.into();
        match self {
            Output::Request(OutputRequest::ForType { ty, output }) => {
                Output::for_type(*ty, output.view(view))
            }
            Output::Branch {
                operation: OutputBranch::Combine,
                children: mut parts,
            } => {
                parts.push(view);
                Output::Branch {
                    operation: OutputBranch::Combine,
                    children: parts,
                }
            }
            other => Output::Branch {
                operation: OutputBranch::Combine,
                children: vec![other, view],
            },
        }
    }
    /// Append a source getter whose result uses its registered conversion default.
    pub fn field(self, name: impl Into<String>) -> Self {
        self.view(Output::getter(name, Output::auto()))
    }
    /// Append the direct codec representation, bypassing this type's expansion default.
    pub fn field_self(self) -> Self {
        self.view(Output::Leaf(OutputLeafOp::Value))
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
        self.view(Output::Leaf(OutputLeafOp::Projected(policy)))
    }
    /// Append a consuming accessor followed by owned field decomposition.
    pub fn fields_into(self, view: Self) -> Self {
        let Output::Unary {
            operation: OutputUnary::Getter { function },
            child: output,
        } = view
        else {
            panic!("fields_into requires a fields! accessor")
        };
        let Output::Request(OutputRequest::NamedFields { overrides, .. }) = *output else {
            panic!("fields_into requires named fields")
        };
        self.view(Output::Unary {
            operation: OutputUnary::Into { function },
            child: Box::new(Output::Request(OutputRequest::NamedFields {
                overrides,
                mode: ProjectionMode::Owned,
            })),
        })
    }
    /// Deprecated spelling of [`Output::fields_into`].
    #[deprecated(note = "use fields_into instead")]
    pub fn fields_self_into(self, view: Self) -> Self {
        self.fields_into(view)
    }
    /// Override one named source field, preserving defaults for its siblings.
    pub fn field_override(mut self, name: impl Into<String>, output: Self) -> Self {
        match &mut self {
            Output::Request(OutputRequest::NamedFields { overrides, .. }) => {
                assert!(
                    overrides.insert(name.into(), output).is_none(),
                    "duplicate field override"
                );
            }
            Output::Unary {
                operation: OutputUnary::Getter { .. },
                child: inner,
                ..
            }
            | Output::Unary {
                operation: OutputUnary::Into { .. },
                child: inner,
                ..
            }
            | Output::Request(OutputRequest::ForType { output: inner, .. }) => {
                let old = std::mem::replace(inner, Box::new(Output::auto()));
                **inner = old.field_override(name, output);
            }
            _ => panic!("field override requires named field decomposition"),
        }
        self
    }
}
impl<P, Q> Output<P, Q> {
    /// Group this explicit expansion into a bulk carrier.
    pub fn bulk(self) -> Self {
        Output::Unary {
            operation: OutputUnary::Bulk,
            child: Box::new(self),
        }
    }
    /// Observe record fields in declaration order; use an empty list for unit.
    ///
    /// `fields` holds one recipe per field of the source record, in declaration order.
    pub fn fields(fields: impl IntoIterator<Item = Output<P, Q>>) -> Self {
        Output::Branch {
            operation: OutputBranch::Fields,
            children: fields.into_iter().collect(),
        }
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
    pub fn getter(name: impl Into<String>, output: Output<P, Q>) -> Self {
        Output::Unary {
            operation: OutputUnary::Getter {
                function: name.into(),
            },
            child: Box::new(output),
        }
    }
    /// Produce multiple outputs from one value.
    ///
    /// `parts` holds one recipe per view of the value; their wires appear in this
    /// order. Binding rejects an empty list or more than one consuming part. The
    /// framework uses [`Codec::output_consumes`] to order observations before a move,
    /// unless [`Codec::output_owner`] first retains the final output allocation.
    pub fn combine(parts: impl IntoIterator<Item = Output<P, Q>>) -> Self {
        Output::Branch {
            operation: OutputBranch::Combine,
            children: parts.into_iter().collect(),
        }
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
            Output::Unary {
                operation: OutputUnary::Convert { conversion, .. },
                ..
            } => match &conversion.source {
                prebindgen_flat::ValueSource::Function(name) => {
                    flat.function(name).is_ok_and(|f| {
                        f.params
                            .first()
                            .is_some_and(|p| !matches!(p.ty.ty, Type::Reference(_)))
                    })
                }
                prebindgen_flat::ValueSource::Into { .. } => true,
            },
            Output::Unary {
                operation: OutputUnary::Sequence { mode, .. },
                ..
            } => match mode {
                Some(mode) => *mode == ProjectionMode::Owned,
                None => !borrowed && sequence_parts(ty).is_some_and(|(_, view)| view.is_owned()),
            },
            Output::Unary {
                operation: OutputUnary::Bulk,
                child,
            } => child.consumes(flat, ty, extra, codec, borrowed),
            Output::Unary {
                operation: OutputUnary::Named { .. },
                child,
                ..
            } => child.consumes(flat, ty, extra, codec, borrowed),
            Output::Unary {
                operation: OutputUnary::Optional,
                child,
            } => child.consumes(flat, option_inner(ty).unwrap_or(ty), extra, codec, borrowed),
            Output::Leaf(OutputLeafOp::Value) => {
                !borrowed && output_referent(ty).is_none() && codec.output_consumes(flat, ty, extra)
            }
            Output::Leaf(OutputLeafOp::Projected(p)) => p.mode == ProjectionMode::Owned,
            Output::Branch {
                operation: OutputBranch::OwnedFields,
                children: _,
            }
            | Output::Unary {
                operation: OutputUnary::Into { .. },
                ..
            }
            | Output::Unary {
                operation: OutputUnary::Result { .. },
                ..
            } => true,
            Output::Branch {
                operation: OutputBranch::Record { mode, .. },
                ..
            } => mode.map_or(!borrowed && output_referent(ty).is_none(), |m| {
                m == ProjectionMode::Owned
            }),
            Output::Branch {
                operation: OutputBranch::Enum { mode, .. },
                ..
            } => mode.map_or(!borrowed && output_referent(ty).is_none(), |m| {
                m == ProjectionMode::Owned
            }),
            Output::Request(never) => match *never {},
            Output::Branch {
                operation: OutputBranch::Combine,
                children: p,
            } => p
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
///     flat.function_mut("foo").unwrap().params[0].ty.slot = Some(Input::Leaf(prebindgen_declarations::InputLeafOp::Value));
/// }
/// ```
pub type Bindings<W, C = (), N = (), B = ()> = Flat<prebindgen_conversions::PlanSchema<W, C, N, B>>;

/// Compile a request-free description for one named source parameter.
/// The model keeps its fixed schema; this returns a converter without modifying its slot.
/// ```
/// use prebindgen_declarations::{compile_input,Codec,Input};
/// use prebindgen_flat::Flat;
/// fn resolved<C:Codec>(flat:&Flat<C::Schema>,codec:&C,extra:&C::Extra) {
///     compile_input(flat,codec,"f","x",&Input::Leaf(prebindgen_declarations::InputLeafOp::Value),extra).unwrap();
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
///     compile_output(flat,codec,"f",&Output::Leaf(prebindgen_declarations::OutputLeafOp::Value),extra).unwrap();
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

/// Compile an independent input root, such as a declared callback interface.
/// All conversion choices must already be resolved in the supplied tree and Flat.
pub fn compile_input_value<C: Codec>(
    flat: &Flat<C::Schema>,
    codec: &C,
    ty: &Type,
    recipe: &Input,
    extra: &C::Extra,
) -> Result<<C::Schema as Schema>::Input> {
    let mut plan = Compiler::new(flat, codec, "value".into()).input(
        ty,
        recipe,
        extra,
        &Condition::default(),
    )?;
    codec.finish_input(&mut plan, extra)?;
    codec.validate_input(&plan)?;
    Ok(plan.into())
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
        input_role: std::cell::Cell::new(InputRole::CallbackResult),
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
///     bind_input(flat, codec, "foo", "x", &Output::Leaf(prebindgen_declarations::OutputLeafOp::Value), extra).unwrap();
/// }
/// ```
pub fn bind_input<C: Codec>(
    flat: &mut Flat<C::Schema>,
    codec: &C,
    function: &str,
    parameter: &str,
    recipe: &Input<InputRequest, OutputRequest>,
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
///     bind_output(flat, codec, "foo", &Input::Leaf(prebindgen_declarations::InputLeafOp::Value), extra).unwrap();
/// }
/// ```
pub fn bind_output<C: Codec>(
    flat: &mut Flat<C::Schema>,
    codec: &C,
    function: &str,
    recipe: &Output<OutputRequest, InputRequest>,
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
    input_role: std::cell::Cell<InputRole>,
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
            input_role: std::cell::Cell::new(InputRole::Parameter),
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
            && !matches!(
                recipe,
                Input::Leaf(InputLeafOp::Value) | Input::Arrow { .. }
            )
        {
            return Err("source callback parameters only support leaf inputs".into());
        }
        let var = names
            .map(|(_, var)| var.clone())
            .unwrap_or_else(|| self.name());
        match recipe {
            Input::None => {
                if type_key(ty) != "()" {
                    return Err("None input requires unit".into());
                }
                Ok(InputPlan::new(
                    WireNode::Branch {
                        controls: vec![],
                        children: vec![],
                        meta: self.codec.group(GroupKind::Fields)?,
                    },
                    quote!(),
                    quote!(()),
                ))
            }
            Input::Branch {
                operation: InputBranch::Variant,
                ..
            } => Err("variant branch requires an enum parent".into()),
            Input::Branch {
                operation: InputBranch::Enum { value },
                children: variants,
            } => self.enum_input(ty, variants, *value, extra, active, names),
            Input::Unary {
                operation: InputUnary::Convert { conversion },
                child: input,
            } => self.input_conversion(ty, conversion, input, extra, active, names),
            Input::Unary {
                operation: InputUnary::Temporary,
                child: inner,
            } => {
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
            Input::Unary {
                operation: InputUnary::Bulk,
                child: inner,
            } => {
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
            Input::Arrow {
                operation: callback,
                domain,
                codomain,
            } => {
                callback.validate_for(self.flat, ty)?;
                let signature = prebindgen_flat::callback_signature(ty)?
                    .ok_or("arrow requires a callable source")?;
                if domain.argument_nodes()?.len() != signature.arguments.len() {
                    return Err("wrong arrow domain arity".into());
                }
                self.codec.input_arrow(InputArrow {
                    flat: self.flat,
                    operation: callback,
                    domain,
                    codomain,
                    extra,
                    wire: names
                        .map(|(wire, _)| wire.clone())
                        .unwrap_or_else(|| self.name()),
                    target: var,
                    active,
                })
            }
            Input::Unary {
                operation: InputUnary::Sequence,
                child: element_recipe,
            } => {
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
                    if let Input::Unary {
                        operation: InputUnary::Bulk,
                        child: inner,
                    } = element_recipe.as_ref()
                    {
                        return self.input_named(
                            ty,
                            &Input::Unary {
                                operation: InputUnary::Bulk,
                                child: Box::new(Input::Unary {
                                    operation: InputUnary::Sequence,
                                    child: inner.clone(),
                                }),
                            },
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
            Input::Unary {
                operation: InputUnary::Optional,
                child: inner,
            } => {
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
            Input::Leaf(InputLeafOp::Value) => {
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
                    role: self.input_role.get(),
                })
            }
            Input::Request(never) => match *never {},
            Input::Branch {
                operation: InputBranch::Fields,
                children,
            }
            | Input::Branch {
                operation: InputBranch::Record,
                children,
            } => {
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
                let grouped = matches!(
                    recipe,
                    Input::Branch {
                        operation: InputBranch::Record,
                        children: _
                    }
                );
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
            Input::Branch {
                operation: InputBranch::Constructor { function },
                children: args,
            } => {
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
            Input::Branch {
                operation: InputBranch::Selector,
                children: arms,
            } => {
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
        if let Output::Unary {
            operation: OutputUnary::Result { error },
            child: ok,
        } = recipe
        {
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
                if let Output::Branch {
                    operation: OutputBranch::Combine,
                    children: parts,
                } = recipe
                {
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
            if let (Type::Reference(reference), Output::Leaf(OutputLeafOp::Value)) = (ty, recipe) {
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
            if let (
                Some(payload_ty),
                Output::Unary {
                    operation: OutputUnary::Optional,
                    child: inner,
                },
            ) = (option_inner(ty), recipe)
            {
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
            if matches!(recipe, Output::Leaf(OutputLeafOp::Projected(p)) if p.mode == ProjectionMode::Owned)
            {
                return Err("owned projection cannot consume a reference payload".into());
            }
            let value = if borrowed { quote!(*(#value)) } else { value };
            return self.output_named(referent, recipe, extra, value, true, wire_name);
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
            Output::Unary {
                operation: OutputUnary::CallbackArgument { .. },
                ..
            } => return Err("callback argument ownership requires an arrow domain".into()),
            Output::None => {
                if type_key(ty) != "()" {
                    return Err("None output requires unit".into());
                }
                out.outcomes.success = WireNode::Branch {
                    controls: vec![],
                    children: vec![],
                    meta: self.codec.group(GroupKind::Fields)?,
                };
            }
            Output::Arrow { operation, .. } => match *operation {},
            Output::Branch {
                operation: OutputBranch::Variant | OutputBranch::Arguments,
                ..
            } => return Err("variant/argument branch requires its parent operation".into()),
            Output::Branch {
                operation:
                    OutputBranch::Enum {
                        value: grouped,
                        mode,
                    },
                children: variants,
            } => {
                return self.enum_output(
                    ty, variants, *grouped, *mode, extra, value, borrowed, wire_name,
                )
            }
            Output::Unary {
                operation: OutputUnary::Convert { conversion },
                child: output,
            } => return self.output_conversion(ty, conversion, output, extra, value, borrowed),
            Output::Unary {
                operation: OutputUnary::Bulk,
                child: inner,
            } => {
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
            Output::Leaf(OutputLeafOp::Discard) => return Ok(out),
            Output::Unary {
                operation: OutputUnary::Named { name },
                child: output,
            } => {
                let mut plan = self.output_named(ty, output, extra, value, borrowed, wire_name)?;
                plan.outcomes.success = WireNode::Branch {
                    meta: self.codec.group(GroupKind::NamedOutput(name))?,
                    controls: vec![],
                    children: vec![plan.outcomes.success],
                };
                return Ok(plan);
            }
            Output::Unary {
                operation: OutputUnary::Sequence { delivery, mode },
                child: element,
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
                        &Output::Unary {
                            operation: OutputUnary::Bulk,
                            child: Box::new(recipe.clone()),
                        },
                        extra,
                        value,
                        borrowed,
                        wire_name,
                    );
                }
                if !self.bulk.get() && *delivery != SequenceDelivery::Callback {
                    if let Output::Unary {
                        operation: OutputUnary::Bulk,
                        child: inner,
                    } = element.as_ref()
                    {
                        return self.output_named(
                            ty,
                            &Output::Unary {
                                operation: OutputUnary::Bulk,
                                child: Box::new(Output::Unary {
                                    operation: OutputUnary::Sequence {
                                        delivery: *delivery,
                                        mode: *mode,
                                    },
                                    child: inner.clone(),
                                }),
                            },
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
            Output::Unary {
                operation: OutputUnary::Result { .. },
                ..
            } => {
                return Err("Result error routing is supported only at the outermost return".into())
            }
            Output::Unary {
                operation: OutputUnary::Optional,
                child: inner,
            } => {
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
            Output::Leaf(OutputLeafOp::Value) => {
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
            Output::Leaf(OutputLeafOp::Projected(projection)) => {
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
            Output::Branch {
                operation: OutputBranch::Fields,
                children,
            }
            | Output::Branch {
                operation: OutputBranch::OwnedFields,
                children,
            }
            | Output::Branch {
                operation: OutputBranch::Record { .. },
                children,
                ..
            } => {
                let grouped = matches!(
                    recipe,
                    Output::Branch {
                        operation: OutputBranch::Record { .. },
                        ..
                    }
                );
                let wire = grouped.then(|| wire_name.cloned().unwrap_or_else(|| self.name()));
                let owned = match recipe {
                    Output::Branch {
                        operation: OutputBranch::OwnedFields,
                        children: _,
                    } => true,
                    Output::Branch {
                        operation: OutputBranch::Record { mode, .. },
                        ..
                    } => mode.map_or(!borrowed, |m| m == ProjectionMode::Owned),
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
                    Output::Branch {
                        operation: OutputBranch::Record {
                            mode: Some(ProjectionMode::Borrowed),
                            ..
                        },
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
            Output::Unary {
                operation: OutputUnary::Getter { function },
                child: output,
            }
            | Output::Unary {
                operation: OutputUnary::Into { function },
                child: output,
            } => {
                let consuming = matches!(
                    recipe,
                    Output::Unary {
                        operation: OutputUnary::Into { .. },
                        ..
                    }
                );
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
            Output::Branch {
                operation: OutputBranch::Combine,
                children: parts,
            } => {
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
            Output::Leaf(OutputLeafOp::Projected(p)) if p.mode == ProjectionMode::Owned && p.fields.is_empty())
                || matches!(part, Output::Leaf(OutputLeafOp::Value)) && self.codec.output_consumes(self.flat, ty, extra)
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
            .any(|part| matches!(part,Output::Leaf(OutputLeafOp::Projected(p)) if p.mode==ProjectionMode::Borrowed))
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
        if parts.iter().enumerate().any(|(i, part)| {
            i != owner
                && !matches!(
                    part,
                    Output::Unary {
                        operation: OutputUnary::Getter { .. },
                        ..
                    }
                )
        }) {
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
                let Output::Unary {
                    operation: OutputUnary::Getter { function },
                    child: output,
                } = part
                else {
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
