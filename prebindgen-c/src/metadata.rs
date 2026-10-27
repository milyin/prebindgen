//! C binding decisions owned by the corresponding Flat declarations.
use super::*;
use std::rc::Rc;

/// Metadata schema for a C binding model.
pub struct CSchema;
impl prebindgen_flat::Schema for CSchema {
    type Input = InputPlan<CWire>;
    type Output = OutputPlan<CWire>;
    type Module = CModule;
    type Function = Option<CFunction>;
    type Type = CType;
    type Field = ();
    type Variant = ();
}
/// Settings applying to the whole generated C module.
#[derive(Clone, Debug)]
pub struct CModule {
    /// Prefix of generated symbols and declarations.
    pub prefix: String,
}
impl Default for CModule {
    fn default() -> Self {
        Self {
            prefix: "pb".into(),
        }
    }
}
/// Export decisions belonging to one function binding.
#[derive(Clone, Debug, Default)]
pub struct CFunction {
    /// Whether conversion failure deliberately aborts this non-Result binding.
    pub abort_on_failure: bool,
    /// Whether these conversions can fail outside a Result error channel.
    /// Custom converters must record this property when preparing metadata.
    pub requires_abort: bool,
}
/// C representation information belonging to one named Rust type.
#[derive(Clone, Debug, Default)]
pub struct CType {
    /// Unprefixed opaque C typedef name, when this type has a handle representation.
    pub pointer: Option<String>,
    pub(crate) pointer_order: usize,
    pub(crate) error_order: usize,
    /// Checked fieldless-enum declaration shared by all occurrences.
    pub enumeration: Option<Rc<CEnum>>,
    pub(crate) error: Option<Rc<errors::ErrorBinding>>,
}
/// One checked C enum declaration.
#[derive(Clone, Debug)]
pub struct CEnum {
    /// Unprefixed C typedef and constant prefix.
    pub name: String,
    /// Source variants and their checked wire values.
    pub mapping: Rc<prebindgen_declarations::EnumMapping>,
}
