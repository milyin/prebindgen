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
pub use emission::{render_template, template_parameter, TemplateContext};

mod plan;
pub use plan::*;
pub mod staged;
pub use staged::{InputFragment, OutputFragment};

/// Schema for consumers storing the standard input/output plans and unit declaration metadata.
/// Language generators with declaration metadata define their own fixed schema.
#[allow(clippy::type_complexity)] // Keep wire, context and node metadata roles explicit.
pub struct PlanSchema<W, C = (), N = (), B = ()>(std::marker::PhantomData<fn() -> (W, C, N, B)>);
impl<W: prebindgen_flat::WireKind, C, N, B> prebindgen_flat::Schema for PlanSchema<W, C, N, B> {
    type Input = InputPlan<W, C, N, B>;
    type Output = OutputPlan<W, C, N, B>;
    type Module = ();
    type Function = ();
    type Type = ();
    type Field = ();
    type Variant = ();
}

/// Check one adapter-selected namespace. The caller decides which values share
/// a namespace; this helper knows nothing about tree shape or ABI placement.
pub fn validate_wire_names<'a, W: prebindgen_flat::WireKind + 'a>(
    wires: impl IntoIterator<Item = &'a prebindgen_flat::Wire<W>>,
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
