//! Reusable boundary conversions and their representation metadata.
//!
//! These helpers build [`InputPlan`] and
//! [`OutputPlan`] values. They do not interpret declarations or
//! select functions to export. Adapters may use the helpers during preparation
//! and their wire metadata during generation.
//!
//! Generated code calls `prebindgen-runtime` at runtime; this crate itself is
//! used only during binding generation.
#![warn(missing_docs)]

pub mod boxed;
pub mod value;

mod emission;
pub use emission::{
    record_input_transfer, render_output, render_template, template_parameter, InputTransfers,
    TemplateContext,
};

mod plan;
pub use plan::*;
pub mod staged;
pub use staged::{InputFragment, OutputFragment};

/// Schema for consumers storing the standard input/output plans and unit declaration metadata.
/// Language generators with declaration metadata define their own fixed schema.
#[allow(clippy::type_complexity)] // Keep wire, context and node metadata roles explicit.
pub struct PlanSchema<WireType, Context = (), Chain = (), Branch = ()>(
    std::marker::PhantomData<fn() -> (WireType, Context, Chain, Branch)>,
);
impl<WireType: crate::WireKind, Context, Chain, Branch> prebindgen_flat::Schema
    for PlanSchema<WireType, Context, Chain, Branch>
{
    type Parameter = Option<InputPlan<WireType, Context, Chain, Branch>>;
    type Return = Option<OutputPlan<WireType, Context, Chain, Branch>>;
    type Module = ();
    type Function = ();
    type Type = ();
    type Field = ();
    type Variant = ();
}

/// Check one adapter-selected namespace. The caller decides which values share
/// a namespace; this helper knows nothing about tree shape or ABI placement.
pub fn validate_wire_names<'a, W: crate::WireKind + 'a>(
    wires: impl IntoIterator<Item = &'a crate::Wire<W>>,
) -> prebindgen_flat::Result<()> {
    let mut names = std::collections::BTreeSet::new();
    for wire in wires {
        if !names.insert(wire.name.to_string()) {
            return Err(format!("duplicate wire identifier `{}`", wire.name));
        }
    }
    Ok(())
}

#[cfg(test)]
mod generation_tests;

mod canonical;
pub use canonical::{ValueConversion, ValueConversions, ValueSource};
mod contract;
pub use contract::*;
mod wire_tree;
pub use wire_tree::{
    wire_at_path, wire_definitions, wire_definitions_mut, wire_definitions_with_paths, Outcomes,
    WireEntry, WireNode, WirePath,
};
#[cfg(test)]
mod request_tests;

mod binding;
pub use binding::{
    BindingState, BoundFunction, BoundParam, BoundTypeUse, FunctionBinding, PreparedCall,
};
