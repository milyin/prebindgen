//! C binding decisions owned by the corresponding Flat declarations.
use super::*;
use std::rc::Rc;

/// Metadata schema for a C binding model.
pub struct CSchema;
impl prebindgen_flat::Schema for CSchema {
    type Parameter = ();
    type Return = ();
    type Module = CModule;
    type Function = CFunctionData;
    type Type = CType;
    type Field = ();
    type Variant = ();
}
/// Settings applying to the whole generated C module.
#[derive(Clone, Debug)]
pub struct CModule {
    /// Prefix of generated symbols and declarations.
    pub prefix: String,
    pub(crate) target: Option<(String, u32, TokenStream)>,
    pub(crate) layout_artifact: Option<crate::OpaqueLayouts>,
}
impl Default for CModule {
    fn default() -> Self {
        Self {
            prefix: "pb".into(),
            layout_artifact: None,
            target: None,
        }
    }
}
/// Export decisions belonging to one function binding.
#[derive(Clone, Debug, Default)]
pub struct CFunction {
    /// This export delivers a returned sequence through a visitor/folder.
    pub callback_return: bool,
    /// Whether conversion failure deliberately aborts this non-Result binding.
    pub abort_on_failure: bool,
    /// Whether these conversions can fail outside a Result error channel.
    /// Custom converters must record this property when preparing metadata.
    pub requires_abort: bool,
}
/// C representation information belonging to one named Rust type.
#[derive(Clone, Debug, Default)]
pub struct CType {
    pub(crate) target_layout: Option<crate::layouts::StorageLayout>,
    /// Input/output defaults stored on this type, including callback signatures.
    pub defaults:
        prebindgen_declarations::DeclTypeDefaults<crate::CInputConfig, crate::COutputConfig>,
    /// Canonical value conversions selected for this source type.
    pub conversion: prebindgen_conversions::ValueConversions,
    /// Ordinary tagged carrier of a source payload enum.
    pub tagged: Option<String>,
    /// Unprefixed opaque C typedef name, when this type has a handle representation.
    pub pointer: Option<String>,
    /// Inline owning representation, when configured or selected for Vec/String outputs.
    pub owned: Option<Rc<COwned>>,
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

/// One C export attached to its original source function.
pub type CBinding<I = CInputConverter, O = COutputConverter> = prebindgen_declarations::DeclBinding<
    crate::CInputConfig,
    crate::COutputConfig,
    crate::CInput,
    crate::COutput,
    I,
    O,
    CFunction,
>;

/// C-specific data attached to one source callable.
/// Export conversions and generated helper bodies share the source signature owner.
#[derive(Clone, Debug)]
pub struct CFunctionData<I = CInputConverter, O = COutputConverter> {
    /// Foreign uses of this callable, each with independent conversions.
    pub bindings: Vec<CBinding<I, O>>,
    pub(crate) helper: Option<RustHelper>,
}
impl<I, O> Default for CFunctionData<I, O> {
    fn default() -> Self {
        Self {
            bindings: Vec::new(),
            helper: None,
        }
    }
}
#[derive(Clone, Debug)]
pub(crate) struct RustHelper {
    pub attributes: Vec<syn::Attribute>,
    pub body: Box<syn::Block>,
}
