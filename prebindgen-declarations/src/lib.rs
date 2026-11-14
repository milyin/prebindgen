//! Generic source-aware helpers and compilation of adapter-owned conversion trees.
//!
//! Language adapters own their operations, configuration and source-representation policies.
//! Shared [`DeclFunction`] and [`DeclCallback`] containers hold the caller's tree types.
//! [`DeclDefaults`] stores typed defaults; source helpers resolve constructors, getters,
//! fields, callbacks and local overrides without a shared operation vocabulary.
//! [`resolve_bindings`] produces a complete [`Flat<ResolvedSchema<C>>`](ResolvedSchema).
//! [`compile_resolved`] compiles that model using the adapter's [`Codec`].
//! All generation settings and resolved conversions belong to Flat.
#![warn(missing_docs)]
mod builders;
#[cfg(test)]
mod extension_tests;
pub use builders::{
    getter_view, DeclError, DeclExpand, DeclExpandParam, DeclExpandReturn, DeclFields,
    DeclInputAlternatives,
};
mod convert;
pub use convert::{
    conversion_borrows, prepare_value_conversions, validate_conversion, DeclConvert,
    DeclConvertSource, LocalConversion,
};

use prebindgen_conversions::{InputPlan, OutputPlan};
use prebindgen_flat::{same_type, type_key, Flat, Result, Wire};
use prebindgen_flat::{Schema, WireNode};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Type;

pub mod callback;
pub use callback::{
    callback_input, CallbackArgumentOwnership, CallbackConfig, CallbackInput, CallbackOperation,
    DeclCallback,
};

mod node;

mod resolved;

pub use node::*;

pub use resolved::{
    compile_function, compile_resolved, resolve_bindings, FunctionRecipe, ResolvedModule,
    ResolvedSchema, ResolvedSlot,
};
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
pub use variants::{overrides as enum_overrides, validate_overrides as validate_enum_overrides};
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
// The owned backing value deliberately requested by reconstruct(). A Vec/String
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
pub type CodecInputPlan<C> =
    InputPlan<<C as Codec>::Wire, <C as Codec>::Context, <C as Codec>::Chain, <C as Codec>::Branch>;
/// Output preparation plan, including the codec's composition metadata.
pub type CodecOutputPlan<C> = OutputPlan<
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

mod codec;
#[cfg(test)]
#[macro_use]
mod test_codec;
#[cfg(test)]
#[macro_use]
mod test_language;
#[cfg(test)]
use test_language::*;
mod compilation;
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
/// use prebindgen_declarations::PlanBindings;
/// fn swap(flat: &mut PlanBindings<syn::Type>) {
///     let function = flat.function_mut("foo").unwrap();
///     function.params[0].ty.slot = function.ret.slot.clone();
/// }
/// ```
/// Nor can a recipe be stored in a slot:
pub type PlanBindings<WireType, Context = (), Chain = (), Branch = ()> =
    Flat<prebindgen_conversions::PlanSchema<WireType, Context, Chain, Branch>>;

/// Compile a resolved description for one named source parameter.
/// The model keeps its fixed schema; this returns a converter without modifying its slot.
/// ```
/// use prebindgen_declarations::{compile_input,Codec,ResolvedInput};
/// use prebindgen_flat::Flat;
/// fn resolved<C:Codec>(flat:&Flat<C::Schema>,codec:&C,tree:&ResolvedInput<C>,extra:&C::Extra) {
///     compile_input(flat,codec,"f","x",tree,extra).unwrap();
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

/// Compile a resolved description for one source result without modifying its slot.
/// ```
/// use prebindgen_declarations::{compile_output,Codec,ResolvedOutput};
/// use prebindgen_flat::Flat;
/// fn resolved<C:Codec>(flat:&Flat<C::Schema>,codec:&C,tree:&ResolvedOutput<C>,extra:&C::Extra) {
///     compile_output(flat,codec,"f",tree,extra).unwrap();
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
/// The description must be resolved. Reads binding declarations from Flat without
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
/// The resolved recipe comes from the binding's type-wide input declarations.
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

/// Compile and install a resolved adapter input tree.
pub fn bind_input<C: Codec>(
    flat: &mut Flat<C::Schema>,
    codec: &C,
    function: &str,
    parameter: &str,
    tree: &ResolvedInput<C>,
    extra: &C::Extra,
) -> Result<()> {
    let input = compile_input(flat, codec, function, parameter, tree, extra)?;
    flat.function_mut(function)?
        .params
        .iter_mut()
        .find(|p| p.name == parameter)
        .ok_or("unknown parameter")?
        .ty
        .slot = Some(input);
    Ok(())
}
/// Compile and install a resolved adapter output conversion.
/// Input and output trees cannot be interchanged:
/// ```compile_fail
/// use prebindgen_declarations::{bind_output,Codec,ResolvedInput};
/// use prebindgen_flat::Flat;
/// fn wrong<C:Codec>(flat:&mut Flat<C::Schema>,codec:&C,input:&ResolvedInput<C>,extra:&C::Extra) {
///     bind_output(flat,codec,"function",input,extra).unwrap();
/// }
/// ```
pub fn bind_output<C: Codec>(
    flat: &mut Flat<C::Schema>,
    codec: &C,
    function: &str,
    tree: &ResolvedOutput<C>,
    extra: &C::Extra,
) -> Result<()> {
    let output = compile_output(flat, codec, function, tree, extra)?;
    flat.function_mut(function)?.ret.slot = Some(output);
    Ok(())
}

fn finalize_output_schema<C: Codec>(
    codec: &C,
    plan: &mut CodecOutputPlan<C>,
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
    ) -> Result<CodecInputPlan<C>> {
        self.input_named(ty, recipe, extra, active, None)
    }
    pub fn input_named(
        &self,
        ty: &Type,
        recipe: &ResolvedInput<C>,
        extra: &C::Extra,
        active: &Condition,
        names: Option<(&syn::Ident, &syn::Ident)>,
    ) -> Result<CodecInputPlan<C>> {
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
    ) -> Result<CodecOutputPlan<C>> {
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
    ) -> Result<CodecOutputPlan<C>> {
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
    ) -> Result<CodecOutputPlan<C>> {
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
    ) -> Result<CodecOutputPlan<C>> {
        self.codec
            .compile_output_tree(self, ty, recipe, extra, value, borrowed, wire)
    }
}
/// Adapter-specific input operations stored in a resolved Flat.
pub type ResolvedInput<C> = InputNode<<C as Codec>::InOps, <C as Codec>::OutOps>;
/// Adapter-specific output operations stored in a resolved Flat.
pub type ResolvedOutput<C> = OutputNode<<C as Codec>::InOps, <C as Codec>::OutOps>;
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
pub use declarations::{CallbackReturn, DeclFunction};

/// Parse one Rust identifier used by a declaration.
///
/// Returns an error for invalid syntax, including paths or multiple identifiers.
pub fn ident(name: &str) -> Result<syn::Ident> {
    syn::parse_str(name).map_err(|_| format!("invalid identifier `{name}`"))
}

#[cfg(test)]
mod niche_tests;

mod source;
pub use source::*;
