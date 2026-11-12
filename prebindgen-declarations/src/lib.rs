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
pub mod frontend;
mod node;
use frontend::*;
pub use frontend::{Input as DeclarationInput, Output as DeclarationOutput};
mod request_conversion;
mod resolution;
mod resolved;
pub use builders::{ErrorDecl, ExpandDecl, ExpandParamDecl, ExpandReturnDecl, FieldsDecl};
pub use node::*;
pub use resolution::Resolve;
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
pub mod sequence;
pub use sequence::{SequenceInput, SequenceOutput};
// Sized shared outputs use the referent's ordinary recipe, with borrowing
// carried by the compiler. Keep unsized string/slice views as their own types:
// adapters and sequence transport already understand those reference shapes.
/// Referent used when observing a sized shared reference.
pub fn output_referent(ty: &Type) -> Option<&Type> {
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
/// Owned backing type reconstructed for an explicitly requested temporary borrow.
pub fn temporary_input_type(ty: &Type) -> Result<Type> {
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
    /// Select per-element delivery after resolving defaults and local expansion.
    SequenceCallback {
        /// Expansion whose outer sequence supplies callback elements.
        output: Box<Output<OutputRequest, InputRequest>>,
        /// Allow structural decomposition of a default collection identity.
        structural: bool,
    },
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

mod codec;
#[cfg(test)]
#[macro_use]
mod test_codec;
mod compilation;
pub use codec::{prepare_mapped_source_error, Codec, DeclarationCodec};
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
/// use prebindgen_declarations::{Bindings, DeclarationInput as Input};
/// fn configure(flat: &mut Bindings<syn::Type>) {
///     flat.function_mut("foo").unwrap().params[0].ty.slot = Some(Input::Leaf(prebindgen_declarations::frontend::InputLeafOp::Value));
/// }
/// ```
pub type Bindings<W, C = (), N = (), B = ()> = Flat<prebindgen_conversions::PlanSchema<W, C, N, B>>;

/// Compile a request-free description for one named source parameter.
/// The model keeps its fixed schema; this returns a converter without modifying its slot.
/// ```
/// use prebindgen_declarations::{compile_input,Codec,ResolvedInput};
/// use prebindgen_flat::Flat;
/// fn resolved<C:Codec>(flat:&Flat<C::Schema>,codec:&C,tree:&ResolvedInput<C>,extra:&C::Extra) {
///     compile_input(flat,codec,"f","x",tree,extra).unwrap();
/// }
/// ```
/// ```compile_fail
/// use prebindgen_declarations::{compile_input,Codec,DeclarationInput as Input};
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
    recipe: &ResolvedInput<C>,
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
/// use prebindgen_declarations::{compile_output,Codec,ResolvedOutput};
/// use prebindgen_flat::Flat;
/// fn resolved<C:Codec>(flat:&Flat<C::Schema>,codec:&C,tree:&ResolvedOutput<C>,extra:&C::Extra) {
///     compile_output(flat,codec,"f",tree,extra).unwrap();
/// }
/// ```
/// ```compile_fail
/// use prebindgen_declarations::{compile_output,Codec,DeclarationOutput as Output};
/// use prebindgen_flat::Flat;
/// fn pending<C:Codec>(flat:&Flat<C::Schema>,codec:&C,extra:&C::Extra) {
///     compile_output(flat,codec,"f",&Output::auto(),extra).unwrap();
/// }
/// ```
pub fn compile_output<C: Codec>(
    flat: &Flat<C::Schema>,
    codec: &C,
    function: &str,
    recipe: &ResolvedOutput<C>,
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
    recipe: &ResolvedOutput<C>,
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
    recipe: &ResolvedInput<C>,
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
    recipe: &ResolvedInput<C>,
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
    recipe: &ResolvedOutput<C>,
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
    recipe: &ResolvedOutput<C>,
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
///   Adapter-resolved trees go directly to [`compile_input`].
/// * `extra` — occurrence data passed to every codec call for this conversion.
///
/// Input binding rejects output recipes:
/// ```compile_fail
/// use prebindgen_declarations::{bind_input, DeclarationCodec, DeclarationOutput};
/// use prebindgen_flat::Flat;
/// fn wrong<C: DeclarationCodec>(flat: &mut Flat<C::Schema>, codec: &C, extra: &C::Extra) {
///     bind_input(flat, codec, "foo", "x", &DeclarationOutput::value(), extra).unwrap();
/// }
/// ```
pub fn bind_input<C: DeclarationCodec>(
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
///   Adapter-resolved trees go directly to [`compile_output`].
/// * `extra` — occurrence data passed to every codec call for this conversion.
///
/// Output binding rejects input recipes:
/// ```compile_fail
/// use prebindgen_declarations::{bind_output, DeclarationCodec, DeclarationInput};
/// use prebindgen_flat::Flat;
/// fn wrong<C: DeclarationCodec>(flat: &mut Flat<C::Schema>, codec: &C, extra: &C::Extra) {
///     bind_output(flat, codec, "foo", &DeclarationInput::value(), extra).unwrap();
/// }
/// ```
pub fn bind_output<C: DeclarationCodec>(
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

/// Generation context shared by nested adapter conversion operations.
/// It holds source metadata and a deterministic name scope, not an operation vocabulary.
pub struct Compiler<'a, C: Codec> {
    /// Whether the input supplies a function parameter or callback result.
    pub input_role: std::cell::Cell<InputRole>,
    copy_borrowed: std::cell::Cell<bool>,
    temporary_receiver: std::cell::Cell<bool>,
    /// Source declarations and adapter metadata.
    pub flat: &'a Flat<C::Schema>,
    /// Interpreter for this adapter’s operations.
    pub codec: &'a C,
    prefix: String,
    next: std::cell::Cell<usize>,
    /// Adapter-specific state for this compilation scope.
    pub state: C::CompileState,
}
#[allow(missing_docs, clippy::too_many_arguments)]
impl<'a, C: Codec> Compiler<'a, C> {
    pub fn new(flat: &'a Flat<C::Schema>, codec: &'a C, prefix: String) -> Self {
        Self {
            input_role: std::cell::Cell::new(InputRole::Parameter),
            copy_borrowed: std::cell::Cell::new(false),
            temporary_receiver: std::cell::Cell::new(false),
            flat,
            codec,
            prefix,
            next: std::cell::Cell::new(0),
            state: Default::default(),
        }
    }
    pub fn input(
        &self,
        ty: &Type,
        recipe: &ResolvedInput<C>,
        extra: &C::Extra,
        active: &Condition,
    ) -> Result<CodecInput<C>> {
        self.input_named(ty, recipe, extra, active, None)
    }
    pub fn input_named(
        &self,
        ty: &Type,
        recipe: &ResolvedInput<C>,
        extra: &C::Extra,
        active: &Condition,
        names: Option<(&syn::Ident, &syn::Ident)>,
    ) -> Result<CodecInput<C>> {
        self.codec
            .compile_input_tree(self, ty, recipe, extra, active, names)
    }
    pub fn output_root(
        &self,
        ty: &Type,
        recipe: &ResolvedOutput<C>,
        extra: &C::Extra,
        value: TokenStream,
        borrowed: bool,
        reference_return: bool,
    ) -> Result<CodecOutput<C>> {
        self.codec
            .compile_output_root(self, ty, recipe, extra, value, borrowed, reference_return)
    }
    pub fn output_return(
        &self,
        ty: &Type,
        recipe: &ResolvedOutput<C>,
        extra: &C::Extra,
        value: TokenStream,
        borrowed: bool,
    ) -> Result<CodecOutput<C>> {
        self.codec
            .compile_output_return(self, ty, recipe, extra, value, borrowed)
    }
    pub fn output(
        &self,
        ty: &Type,
        recipe: &ResolvedOutput<C>,
        extra: &C::Extra,
        value: TokenStream,
        borrowed: bool,
    ) -> Result<CodecOutput<C>> {
        self.output_named(ty, recipe, extra, value, borrowed, None)
    }
    pub fn output_named(
        &self,
        ty: &Type,
        recipe: &ResolvedOutput<C>,
        extra: &C::Extra,
        value: TokenStream,
        borrowed: bool,
        wire: Option<&syn::Ident>,
    ) -> Result<CodecOutput<C>> {
        self.codec
            .compile_output_tree(self, ty, recipe, extra, value, borrowed, wire)
    }
}
/// Adapter-specific input operations stored in a resolved Flat.
pub type ResolvedInput<C> =
    InputNode<<C as Codec>::InputOperations, <C as Codec>::OutputOperations>;
/// Adapter-specific output operations stored in a resolved Flat.
pub type ResolvedOutput<C> =
    OutputNode<<C as Codec>::InputOperations, <C as Codec>::OutputOperations>;
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
/// Append one output view while preserving its preparation and ownership commit.
pub fn append<W: prebindgen_flat::WireKind, C, N, B>(
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
