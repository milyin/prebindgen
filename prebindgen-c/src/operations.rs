//! Adapter-owned conversion metadata for source resolution and compilation.
use crate::COutputConfigExt;
use prebindgen_declarations::{
    CallbackArgumentOwnership, InputNode, InputOps, OutputNode, OutputOps, Projection,
    ProjectionMode, SequenceDelivery,
};
use prebindgen_flat::Result;
use std::convert::Infallible;
/// Input operations for this adapter.
#[derive(Clone, Debug)]
pub struct CInputOps;
/// Output operations for this adapter.
#[derive(Clone, Debug)]
pub struct COutputOps;
impl InputOps for CInputOps {
    type Leaf = CInputLeafOp;
    type Unary = CInputUnary;
    type Branch = CInputBranch;
    type Arrow = CInputArrow;
}
impl OutputOps for COutputOps {
    type Leaf = COutputLeafOp;
    type Unary = COutputUnary;
    type Branch = COutputBranch;
    type Arrow = Infallible;
}
/// Resolved incoming conversion. Configuration-only payloads are uninhabited.
/// A configuration tree cannot be passed as a final tree:
/// ```compile_fail
/// use prebindgen_c::{CInput,CInputConfig,prelude::*};
/// let _:CInput=CInputConfig::auto();
/// ```
/// Even a nested child must have the final operation sets:
/// ```compile_fail
/// use prebindgen_c::{CInput,CInputConfig,CInputUnary,prelude::*};
/// let _:CInput=CInput::Unary {
///     operation:CInputUnary::Sequence,
///     child:Box::new(CInputConfig::auto()),
/// };
/// ```
/// Configuration-only leaf payloads are directly Infallible:
/// ```compile_fail
/// use prebindgen_c::{CInput,CInputLeafOp};
/// let _:CInput=CInput::Leaf(CInputLeafOp::ConfigDefault(()));
/// ```
/// The same applies to unary scopes and both partial enum operations:
/// ```compile_fail
/// use prebindgen_c::{CInput,CInputUnary};
/// let _:CInput=CInput::Unary {
///     operation:CInputUnary::ConfigForType {ty:Box::new(syn::parse_quote!(i64))},
///     child:Box::new(CInput::None),
/// };
/// ```
/// ```compile_fail
/// use prebindgen_c::{CInput,CInputBranch};
/// let _:CInput=CInput::Branch {
///     operation:CInputBranch::ConfigEnumFields(()),children:vec![],
/// };
/// ```
/// ```compile_fail
/// use prebindgen_c::{CInput,CInputBranch,CVariantFields};
/// let _:CInput=CInput::Branch {
///     operation:CInputBranch::ConfigVariantFields(CVariantFields {variant:"Row".into(),fields:vec![]}),
///     children:vec![],
/// };
/// ```
pub type CInput = InputNode<CInputOps, COutputOps>;
/// Resolved outgoing conversion. An optional configuration payload cannot
/// leave a constructible None in the final leaf type:
/// ```compile_fail
/// use prebindgen_c::{COutput,COutputLeafOp};
/// let _:COutput=COutput::Leaf(COutputLeafOp::ConfigValue(None));
/// ```
pub type COutput = OutputNode<CInputOps, COutputOps>;
/// Incoming conversion configuration.
pub type CInputConfig = InputNode<CInputConfigOps, COutputConfigOps>;
/// Outgoing conversion configuration.
pub type COutputConfig = OutputNode<CInputConfigOps, COutputConfigOps>;
/// Incoming operation set with concrete configuration payloads.
#[derive(Clone, Debug)]
pub struct CInputConfigOps;
/// Outgoing operation set with concrete configuration payloads.
#[derive(Clone, Debug)]
pub struct COutputConfigOps;
impl InputOps for CInputConfigOps {
    type Leaf = CInputConfigLeafOp;
    type Unary = CInputConfigUnary;
    type Branch = CInputConfigBranch;
    type Arrow = CInputConfigArrow;
}
impl OutputOps for COutputConfigOps {
    type Leaf = COutputConfigLeafOp;
    type Unary = COutputConfigUnary;
    type Branch = COutputConfigBranch;
    type Arrow = Infallible;
}
/// Source variant and field names corresponding to explicit branch children.
#[derive(Clone, Debug)]
pub struct CVariantFields {
    /// Source variant name.
    pub variant: String,
    /// Source field names corresponding to the structural children.
    pub fields: Vec<String>,
}
/// Source record field names and projection mode for explicit branch children.
#[derive(Clone, Debug)]
pub struct CNamedFields {
    /// Source field names corresponding to the structural children.
    pub fields: Vec<String>,
    /// Observe or move the selected fields.
    pub mode: ProjectionMode,
}

/// Source field names corresponding to explicit incoming record children.
#[derive(Clone, Debug)]
pub struct CInputFields {
    /// Source field names, in the same order as the structural child conversions.
    pub fields: Vec<String>,
}
/// Source constructor and names corresponding to its explicit argument children.
#[derive(Clone, Debug)]
pub struct CConstructorArguments {
    /// Unqualified source constructor name.
    pub function: String,
    /// Source parameter names, in the same order as the structural child conversions.
    pub parameters: Vec<String>,
}

/// Incoming callback configuration and its resolved retained-callback binding.
#[derive(Clone, Debug)]
pub enum CInputArrow<CallbackExpansion = Infallible> {
    /// Resolve explicit argument/result conversions against callback defaults.
    ConfigCallback(CallbackExpansion),
    /// Canonical callback signature and settings with resolved argument/result children.
    BoundCallback(Box<prebindgen_declarations::CallbackOperation>),
}

impl CInputArrow {
    pub(crate) fn bound_callback(&self) -> &prebindgen_declarations::CallbackOperation {
        match self {
            Self::BoundCallback(callback) => callback,
            Self::ConfigCallback(never) => match *never {},
        }
    }
}

/// Terminal input operations.
#[derive(Clone, Debug)]
pub enum CInputLeafOp<
    DefaultChoice = Infallible,
    ValueChoice = Infallible,
    ConversionChoice = Infallible,
    ConstructorChoice = Infallible,
> {
    /// Resolve this occurrence's configured default.
    ConfigDefault(DefaultChoice),
    /// Select the canonical value representation.
    ConfigValue(ValueChoice),
    /// Select a local Rust value conversion.
    ConfigConversion(ConversionChoice),
    /// Infer the source constructor's argument conversions.
    ConfigConstructor(ConstructorChoice),

    /// Convert the occurrence using its declared value representation.
    Value,
}
/// Single-child input operations.
#[derive(Clone, Debug)]
pub enum CInputUnary<TypeScope = Infallible> {
    /// Apply the child conversion at this source type, including nested occurrences.
    ConfigForType {
        /// Source type at which the child applies.
        ty: TypeScope,
    },

    /// Apply an explicitly selected Rust value conversion.
    Convert {
        /// Checked source/representation conversion.
        conversion: Box<prebindgen_flat::ValueConversion>,
    },
    /// Construct a collection from converted elements.
    Sequence,
    /// Construct an owned temporary and lend it for this call.
    Temporary,
    /// Convert the payload only when present.
    Optional,
}
/// Multiple-child input operations.
#[derive(Clone, Debug)]
pub enum CInputBranch<
    EnumExpansion = Infallible,
    VariantExpansion = Infallible,
    RecordExpansion = Infallible,
    ConstructorExpansion = Infallible,
> {
    /// Partially specified enum fields; children are variant branches.
    ConfigEnumFields(EnumExpansion),
    /// Partially specified fields of one source variant.
    ConfigVariantFields(VariantExpansion),
    /// Source record fields with named occurrence-local conversion overrides.
    ConfigNamedFields(RecordExpansion),
    /// Source constructor with named occurrence-local argument overrides.
    ConfigConstructorArguments(ConstructorExpansion),

    /// Convert source record fields in declaration order.
    Fields,
    /// Select one constructor alternative by its incoming tag.
    Selector,
    /// Construct a value by calling a source function.
    Constructor {
        /// Unqualified source function name.
        function: String,
    },
    /// Construct a source enum from its selected variant branch.
    Enum {
        /// Group the foreign value instead of exposing tag/field arguments.
        value: bool,
    },
    /// Fields of one source enum variant; interpreted by its enum parent.
    Variant,
}
/// Terminal output operations.
#[derive(Clone, Debug)]
pub enum COutputLeafOp<
    DefaultChoice = Infallible,
    ValueChoice = Infallible,
    ConversionChoice = Infallible,
> {
    /// Resolve this occurrence's configured default.
    ConfigDefault(DefaultChoice),
    /// Select the canonical value representation.
    ConfigValue(ValueChoice),
    /// Select a local Rust value conversion.
    ConfigConversion(ConversionChoice),

    /// Convert the occurrence using its declared value representation.
    Value,
    /// Omit a value while preserving its ordinary Rust destruction.
    Discard,
    /// Apply an explicit observation or move policy to a terminal value.
    Projected(Projection),
}
/// Single-child output operations.
#[derive(Clone, Debug)]
pub enum COutputUnary<TypeScope = Infallible, SequenceSelection = Infallible> {
    /// Apply the child conversion at this source type, including nested occurrences.
    ConfigForType {
        /// Source type at which the child applies.
        ty: TypeScope,
    },
    /// Select per-element callback delivery of the child's sequence.
    ConfigSequenceCallback {
        /// Permit decomposition of an ordinary collection identity.
        structural: SequenceSelection,
    },

    /// Ownership contract of one callable argument; interpreted by its arrow.
    CallbackArgument {
        /// Whether produced foreign owners expire at the end of invocation.
        ownership: CallbackArgumentOwnership,
    },
    /// Give one delivered view an explicit foreign field name.
    Named {
        /// Foreign field name.
        name: String,
    },
    /// Apply an explicitly selected Rust value conversion.
    Convert {
        /// Checked source/representation conversion.
        conversion: Box<prebindgen_flat::ValueConversion>,
    },
    /// Convert and deliver each collection element.
    Sequence {
        /// Materialized or per-element callback delivery.
        delivery: SequenceDelivery,
        /// Explicit move/observation policy, or the source occurrence’s policy.
        mode: Option<ProjectionMode>,
    },
    /// Convert success and route a source error through the adapter.
    Result {
        /// Explicit source message mapper, if selected.
        error: Option<String>,
    },
    /// Convert the payload only when present.
    Optional,
    /// Call a consuming source accessor and convert its result.
    Into {
        /// Unqualified source function name.
        function: String,
    },
    /// Observe through a source getter and convert its result.
    Getter {
        /// Unqualified source function name.
        function: String,
    },
}
/// Multiple-child output operations.
#[derive(Clone, Debug)]
pub enum COutputBranch<
    EnumExpansion = Infallible,
    VariantExpansion = Infallible,
    RecordExpansion = Infallible,
> {
    /// Partially specified enum fields; children are variant branches.
    ConfigEnumFields(EnumExpansion),
    /// Partially specified fields of one source variant.
    ConfigVariantFields(VariantExpansion),
    /// Partially specified record fields, in the same order as the child conversions.
    ConfigNamedFields(RecordExpansion),

    /// Convert source record fields in declaration order.
    Fields,
    /// Move source record fields into their conversions.
    OwnedFields,
    /// Deliver multiple views, observing before a possible consuming view.
    Combine,
    /// Match a source enum and convert its active variant branch.
    Enum {
        /// Group the foreign value instead of exposing tag/field arguments.
        value: bool,
        /// Explicit move/observation policy, or the source occurrence’s policy.
        mode: Option<ProjectionMode>,
    },
    /// Fields of one source enum variant; interpreted by its enum parent.
    Variant,
    /// A callable domain in source parameter order.
    Arguments,
}

pub(crate) fn sequence_callback(
    cx: &mut crate::CResolve<'_>,
    ty: &syn::Type,
    mut output: COutput,
    structural_default: bool,
) -> Result<COutput> {
    fn select(output: &mut COutput) -> usize {
        match output {
            COutput::Unary {
                operation: COutputUnary::Sequence { delivery, .. },
                ..
            } => {
                *delivery = SequenceDelivery::Callback;
                1
            }
            COutput::Unary {
                operation: COutputUnary::Optional,
                child,
            }
            | COutput::Unary {
                operation: COutputUnary::Named { .. },
                child,
                ..
            }
            | COutput::Unary {
                operation: COutputUnary::Getter { .. },
                child,
                ..
            }
            | COutput::Unary {
                operation: COutputUnary::Into { .. },
                child,
                ..
            }
            | COutput::Unary {
                operation: COutputUnary::Convert { .. },
                child,
                ..
            } => select(child),
            COutput::Branch {
                operation: COutputBranch::Combine,
                children,
            }
            | COutput::Branch {
                operation: COutputBranch::Fields,
                children,
            }
            | COutput::Branch {
                operation: COutputBranch::OwnedFields,
                children,
            } => children.iter_mut().map(select).sum(),
            _ => 0,
        }
    }
    if select(&mut output) != 1 {
        fn structural(ty: &syn::Type, output: &COutput) -> Option<crate::COutputConfig> {
            match output {
                COutput::Leaf(COutputLeafOp::Value)
                    if prebindgen_flat::sequence_parts(ty).is_some() =>
                {
                    Some(crate::COutputConfig::Unary {
                        operation: crate::COutputUnary::Sequence {
                            delivery: SequenceDelivery::Callback,
                            mode: None,
                        },
                        child: Box::new(crate::COutputConfig::auto()),
                    })
                }
                COutput::Unary {
                    operation: COutputUnary::Optional,
                    child,
                } => Some(crate::COutputConfig::Unary {
                    operation: crate::COutputUnary::Optional,
                    child: Box::new(structural(prebindgen_flat::option_inner(ty)?, child)?),
                }),
                _ => None,
            }
        }
        if structural_default {
            if let Some(tree) = structural(ty, &output) {
                output = cx.output(ty, &tree)?;
            } else {
                return Err("callback_return requires exactly one outer sequence".into());
            }
        } else {
            return Err("callback_return requires exactly one outer sequence".into());
        }
    }
    Ok(output)
}

/// Configuration specialization of [`CInputArrow`].
pub type CInputConfigArrow = CInputArrow<Box<prebindgen_declarations::CallbackConfig>>;
impl From<CInputArrow> for CInputConfigArrow {
    fn from(operation: CInputArrow) -> Self {
        match operation {
            CInputArrow::ConfigCallback(never) => match never {},
            CInputArrow::BoundCallback(field0) => Self::BoundCallback(field0),
        }
    }
}
/// Configuration specialization of [`CInputLeafOp`].
pub type CInputConfigLeafOp =
    CInputLeafOp<(), (), Box<prebindgen_declarations::DeclConvert>, String>;
impl From<CInputLeafOp> for CInputConfigLeafOp {
    fn from(operation: CInputLeafOp) -> Self {
        match operation {
            CInputLeafOp::Value => Self::Value,
            CInputLeafOp::ConfigDefault(never)
            | CInputLeafOp::ConfigValue(never)
            | CInputLeafOp::ConfigConversion(never) => match never {},
            CInputLeafOp::ConfigConstructor(never) => match never {},
        }
    }
}
/// Configuration specialization of [`CInputUnary`].
pub type CInputConfigUnary = CInputUnary<Box<syn::Type>>;
impl From<CInputUnary> for CInputConfigUnary {
    fn from(operation: CInputUnary) -> Self {
        match operation {
            CInputUnary::ConfigForType { ty: never } => match never {},
            CInputUnary::Convert { conversion } => Self::Convert { conversion },
            CInputUnary::Sequence => Self::Sequence,
            CInputUnary::Temporary => Self::Temporary,
            CInputUnary::Optional => Self::Optional,
        }
    }
}
/// Configuration specialization of [`CInputBranch`].
pub type CInputConfigBranch = CInputBranch<(), CVariantFields, CInputFields, CConstructorArguments>;
impl From<CInputBranch> for CInputConfigBranch {
    fn from(operation: CInputBranch) -> Self {
        match operation {
            CInputBranch::ConfigEnumFields(never) => match never {},
            CInputBranch::ConfigVariantFields(never)
            | CInputBranch::ConfigNamedFields(never)
            | CInputBranch::ConfigConstructorArguments(never) => match never {},
            CInputBranch::Fields => Self::Fields,
            CInputBranch::Selector => Self::Selector,
            CInputBranch::Constructor { function } => Self::Constructor { function },
            CInputBranch::Enum { value } => Self::Enum { value },
            CInputBranch::Variant => Self::Variant,
        }
    }
}
/// Configuration specialization of [`COutputLeafOp`].
pub type COutputConfigLeafOp =
    COutputLeafOp<(), Option<Projection>, Box<prebindgen_declarations::DeclConvert>>;
impl From<COutputLeafOp> for COutputConfigLeafOp {
    fn from(operation: COutputLeafOp) -> Self {
        match operation {
            COutputLeafOp::Value => Self::Value,
            COutputLeafOp::Discard => Self::Discard,
            COutputLeafOp::Projected(field0) => Self::Projected(field0),
            COutputLeafOp::ConfigDefault(never)
            | COutputLeafOp::ConfigValue(never)
            | COutputLeafOp::ConfigConversion(never) => match never {},
        }
    }
}
/// Configuration specialization of [`COutputUnary`].
pub type COutputConfigUnary = COutputUnary<Box<syn::Type>, bool>;
impl From<COutputUnary> for COutputConfigUnary {
    fn from(operation: COutputUnary) -> Self {
        match operation {
            COutputUnary::ConfigForType { ty: never } => match never {},
            COutputUnary::ConfigSequenceCallback { structural: never } => match never {},
            COutputUnary::CallbackArgument { ownership } => Self::CallbackArgument { ownership },
            COutputUnary::Named { name } => Self::Named { name },
            COutputUnary::Convert { conversion } => Self::Convert { conversion },
            COutputUnary::Sequence { delivery, mode } => Self::Sequence { delivery, mode },
            COutputUnary::Result { error } => Self::Result { error },
            COutputUnary::Optional => Self::Optional,
            COutputUnary::Into { function } => Self::Into { function },
            COutputUnary::Getter { function } => Self::Getter { function },
        }
    }
}
/// Configuration specialization of [`COutputBranch`].
pub type COutputConfigBranch = COutputBranch<(), CVariantFields, CNamedFields>;
impl From<COutputBranch> for COutputConfigBranch {
    fn from(operation: COutputBranch) -> Self {
        match operation {
            COutputBranch::ConfigEnumFields(never) => match never {},
            COutputBranch::ConfigVariantFields(never) => match never {},
            COutputBranch::ConfigNamedFields(never) => match never {},
            COutputBranch::Fields => Self::Fields,
            COutputBranch::OwnedFields => Self::OwnedFields,
            COutputBranch::Combine => Self::Combine,
            COutputBranch::Enum { value, mode } => Self::Enum { value, mode },
            COutputBranch::Variant => Self::Variant,
            COutputBranch::Arguments => Self::Arguments,
        }
    }
}
