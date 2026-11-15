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
    type Record = ();
    type Enum = ();
    type PayloadEnum = ();
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
    /// Input/output defaults stored on this type, including callback signatures.
    pub defaults:
        prebindgen_declarations::DeclTypeDefaults<crate::CInputConfig, crate::COutputConfig>,
    /// Canonical value conversions selected for this source type.
    pub conversion: prebindgen_conversions::ValueConversions,
    /// Selected foreign representation, independent of the Rust definition kind.
    pub representation: Option<CRepresentation>,
    pub(crate) error_order: usize,
    pub(crate) error: Option<Rc<errors::ErrorBinding>>,
}
/// Mutually exclusive C representations of one Rust type.
#[derive(Clone, Debug)]
pub enum CRepresentation {
    /// An opaque pointer and its typed destructor.
    Pointer {
        /// Unprefixed C typedef name.
        name: String,
        /// Declaration order used for stable emission.
        order: usize,
    },
    /// Inline owning storage, including its measured target layout once prepared.
    Owned {
        /// Rust value, storage recipe and accessors.
        owner: Rc<COwned>,
        /// Target size/alignment, filled during preparation.
        target_layout: Option<prebindgen_reprc::SizeAlign>,
    },
    /// A checked fieldless enum.
    Enum(Rc<CEnum>),
    /// A tagged carrier of a payload enum, identified by its C typedef name.
    Tagged(String),
}
impl CType {
    pub(crate) fn pointer(&self) -> Option<&String> {
        match &self.representation {
            Some(CRepresentation::Pointer { name, .. }) => Some(name),
            _ => None,
        }
    }
    pub(crate) fn owned(&self) -> Option<&Rc<COwned>> {
        match &self.representation {
            Some(CRepresentation::Owned { owner, .. }) => Some(owner),
            _ => None,
        }
    }
    pub(crate) fn enumeration(&self) -> Option<&Rc<CEnum>> {
        match &self.representation {
            Some(CRepresentation::Enum(e)) => Some(e),
            _ => None,
        }
    }
    pub(crate) fn tagged(&self) -> Option<&String> {
        match &self.representation {
            Some(CRepresentation::Tagged(name)) => Some(name),
            _ => None,
        }
    }
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
