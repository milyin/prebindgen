//! Adapter-owned conversion metadata for source resolution and compilation.
use crate::JniOutputConfigExt;
use prebindgen_declarations::{
    CallbackArgumentOwnership, InputNode, InputOps, OutputNode, OutputOps, Projection,
    ProjectionMode, SequenceDelivery,
};
use prebindgen_flat::Result;
use std::convert::Infallible;
/// Input operations for this adapter.
#[derive(Clone, Debug)]
pub struct JniInputOps;
/// Output operations for this adapter.
#[derive(Clone, Debug)]
pub struct JniOutputOps;
impl InputOps for JniInputOps {
    type Leaf = JniInputLeafOp;
    type Unary = JniInputUnary;
    type Branch = JniInputBranch;
    type Arrow = JniInputArrow;
}
impl OutputOps for JniOutputOps {
    type Leaf = JniOutputLeafOp;
    type Unary = JniOutputUnary;
    type Branch = JniOutputBranch;
    type Arrow = Infallible;
}
/// Resolved incoming conversion. Configuration cannot enter callback domains:
/// ```compile_fail
/// use prebindgen_jni::{JniInputArrow,JniInputConfigArrow};
/// fn final_arrow(config:JniInputConfigArrow)->JniInputArrow { config }
/// ```
/// Final leaves and enum branches cannot contain configuration payloads:
/// ```compile_fail
/// use prebindgen_jni::{JniInput,JniInputLeafOp};
/// let _:JniInput=JniInput::Leaf(JniInputLeafOp::ConfigDefault(()));
/// ```
/// ```compile_fail
/// use prebindgen_jni::{JniInput,JniInputBranch};
/// let _:JniInput=JniInput::Branch {
///     operation:JniInputBranch::ConfigEnumFields(false),children:vec![],
/// };
/// ```
pub type JniInput = InputNode<JniInputOps, JniOutputOps>;
/// Resolved outgoing conversion. Partial named fields have no final payload:
/// ```compile_fail
/// use prebindgen_jni::{JniOutput,JniOutputBranch,JniNamedFields,ProjectionMode};
/// let _:JniOutput=JniOutput::Branch {
///     operation:JniOutputBranch::ConfigNamedFields(JniNamedFields {fields:vec![],mode:ProjectionMode::Owned}),
///     children:vec![],
/// };
/// ```
/// None is a configuration payload, not an inhabitant of the final payload type:
/// ```compile_fail
/// use prebindgen_jni::{JniOutput,JniOutputLeafOp};
/// let _:JniOutput=JniOutput::Leaf(JniOutputLeafOp::ConfigValue(None));
/// ```
pub type JniOutput = OutputNode<JniInputOps, JniOutputOps>;
/// Incoming conversion configuration.
/// ```
/// use prebindgen_jni::{JniInputConfig,JniOutputConfig,prelude::*};
/// let _:JniInputConfig=JniInputConfig::auto();
/// let _:JniOutputConfig=JniOutputConfig::auto();
/// ```
pub type JniInputConfig = InputNode<JniInputConfigOps, JniOutputConfigOps>;
/// Outgoing conversion configuration.
pub type JniOutputConfig = OutputNode<JniInputConfigOps, JniOutputConfigOps>;
/// Incoming operation set with concrete configuration payloads.
#[derive(Clone, Debug)]
pub struct JniInputConfigOps;
/// Outgoing operation set with concrete configuration payloads.
#[derive(Clone, Debug)]
pub struct JniOutputConfigOps;
impl InputOps for JniInputConfigOps {
    type Leaf = JniInputConfigLeafOp;
    type Unary = JniInputConfigUnary;
    type Branch = JniInputConfigBranch;
    type Arrow = JniInputConfigArrow;
}
impl OutputOps for JniOutputConfigOps {
    type Leaf = JniOutputConfigLeafOp;
    type Unary = JniOutputConfigUnary;
    type Branch = JniOutputConfigBranch;
    type Arrow = Infallible;
}
/// Source variant and field names corresponding to explicit branch children.
#[derive(Clone, Debug)]
pub struct JniVariantFields {
    /// Source variant name.
    pub variant: String,
    /// Source field names corresponding to the structural children.
    pub fields: Vec<String>,
}
/// Source record field names and projection mode for explicit branch children.
#[derive(Clone, Debug)]
pub struct JniNamedFields {
    /// Source field names corresponding to the structural children.
    pub fields: Vec<String>,
    /// Observe or move the selected fields.
    pub mode: ProjectionMode,
}

/// Source field names corresponding to explicit incoming record children.
#[derive(Clone, Debug)]
pub struct JniInputFields {
    /// Source field names, in the same order as the structural child conversions.
    pub fields: Vec<String>,
    /// Group the input as a Kotlin data-class value.
    pub record: bool,
}
/// Source constructor and names corresponding to its explicit argument children.
#[derive(Clone, Debug)]
pub struct JniConstructorArguments {
    /// Unqualified source constructor name.
    pub function: String,
    /// Source parameter names, in the same order as the structural child conversions.
    pub parameters: Vec<String>,
}

/// Incoming callback configuration and its resolved retained-callback binding.
#[derive(Clone, Debug)]
pub enum JniInputArrow<CallbackExpansion = Infallible> {
    /// Resolve explicit argument/result conversions against callback defaults.
    ConfigCallback(CallbackExpansion),
    /// Canonical callback signature and settings with resolved argument/result children.
    BoundCallback(Box<prebindgen_declarations::CallbackOperation>),
}

impl JniInputArrow {
    pub(crate) fn bound_callback(&self) -> &prebindgen_declarations::CallbackOperation {
        match self {
            Self::BoundCallback(callback) => callback,
            Self::ConfigCallback(never) => match *never {},
        }
    }
}

/// Terminal input operations.
#[derive(Clone, Debug)]
pub enum JniInputLeafOp<
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
pub enum JniInputUnary<TypeScope = Infallible> {
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
    /// Request the adapter’s packed collection/record transport.
    Bulk,
    /// Convert the payload only when present.
    Optional,
}
/// Multiple-child input operations.
#[derive(Clone, Debug)]
pub enum JniInputBranch<
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
    /// Group converted fields into the declared foreign record representation.
    Record,
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
pub enum JniOutputLeafOp<
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
pub enum JniOutputUnary<TypeScope = Infallible, SequenceSelection = Infallible> {
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
    /// Request the adapter’s packed collection/record transport.
    Bulk,
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
pub enum JniOutputBranch<
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
    /// Deliver a foreign record with explicit field conversions.
    Record {
        /// Explicit move/observation policy, or the source occurrence’s policy.
        mode: Option<ProjectionMode>,
    },
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

pub(crate) fn lift_input_bulk(tree: JniInput) -> JniInput {
    match tree {
        JniInput::Branch {
            operation: JniInputBranch::Enum { value },
            children: variants,
        } => JniInput::Branch {
            operation: JniInputBranch::Enum { value },
            children: variants.into_iter().map(lift_input_bulk).collect(),
        },
        JniInput::Unary {
            operation: JniInputUnary::Convert { conversion },
            child: input,
        } => JniInput::Unary {
            operation: JniInputUnary::Convert { conversion },
            child: Box::new(lift_input_bulk(*input)),
        },
        JniInput::Unary {
            operation: JniInputUnary::Bulk,
            child: c,
        } => JniInput::Unary {
            operation: JniInputUnary::Bulk,
            child: Box::new(lift_input_bulk(*c)),
        },
        JniInput::Unary {
            operation: JniInputUnary::Temporary,
            child: c,
        } => JniInput::Unary {
            operation: JniInputUnary::Temporary,
            child: Box::new(lift_input_bulk(*c)),
        },
        JniInput::Unary {
            operation: JniInputUnary::Sequence,
            child: c,
        } => {
            let child = lift_input_bulk(*c);
            if input_has_bulk(&child) {
                JniInput::Unary {
                    operation: JniInputUnary::Bulk,
                    child: Box::new(JniInput::Unary {
                        operation: JniInputUnary::Sequence,
                        child: Box::new(child),
                    }),
                }
            } else {
                JniInput::Unary {
                    operation: JniInputUnary::Sequence,
                    child: Box::new(child),
                }
            }
        }
        JniInput::Unary {
            operation: JniInputUnary::Optional,
            child: c,
        } => match lift_input_bulk(*c) {
            JniInput::Unary {
                operation: JniInputUnary::Bulk,
                child: c,
            } => JniInput::Unary {
                operation: JniInputUnary::Bulk,
                child: Box::new(JniInput::Unary {
                    operation: JniInputUnary::Optional,
                    child: c,
                }),
            },
            c => JniInput::Unary {
                operation: JniInputUnary::Optional,
                child: Box::new(c),
            },
        },
        JniInput::Branch {
            operation: JniInputBranch::Fields,
            children: c,
        } => JniInput::Branch {
            operation: JniInputBranch::Fields,
            children: c.into_iter().map(lift_input_bulk).collect(),
        },
        JniInput::Branch {
            operation: JniInputBranch::Record,
            children: c,
        } => JniInput::Branch {
            operation: JniInputBranch::Record,
            children: c.into_iter().map(lift_input_bulk).collect(),
        },
        JniInput::Branch {
            operation: JniInputBranch::Selector,
            children: c,
        } => JniInput::Branch {
            operation: JniInputBranch::Selector,
            children: c.into_iter().map(lift_input_bulk).collect(),
        },
        JniInput::Branch {
            operation: JniInputBranch::Constructor { function },
            children: args,
        } => JniInput::Branch {
            operation: JniInputBranch::Constructor { function },
            children: args.into_iter().map(lift_input_bulk).collect(),
        },
        JniInput::Branch {
            operation,
            children,
        } => JniInput::Branch {
            operation,
            children: children.into_iter().map(lift_input_bulk).collect(),
        },
        c => c,
    }
}
pub(crate) fn lift_output_bulk(tree: JniOutput) -> JniOutput {
    match tree {
        JniOutput::Branch {
            operation: JniOutputBranch::Enum { value, mode },
            children: variants,
        } => JniOutput::Branch {
            operation: JniOutputBranch::Enum { value, mode },
            children: variants.into_iter().map(lift_output_bulk).collect(),
        },
        JniOutput::Unary {
            operation: JniOutputUnary::Convert { conversion },
            child: output,
        } => JniOutput::Unary {
            operation: JniOutputUnary::Convert { conversion },
            child: Box::new(lift_output_bulk(*output)),
        },
        JniOutput::Unary {
            operation: JniOutputUnary::Bulk,
            child: c,
        } => JniOutput::Unary {
            operation: JniOutputUnary::Bulk,
            child: Box::new(lift_output_bulk(*c)),
        },
        JniOutput::Unary {
            operation: JniOutputUnary::Sequence { delivery, mode },
            child: element,
        } => {
            let child = lift_output_bulk(*element);
            if delivery != SequenceDelivery::Callback && output_has_bulk(&child) {
                JniOutput::Unary {
                    operation: JniOutputUnary::Bulk,
                    child: Box::new(JniOutput::Unary {
                        operation: JniOutputUnary::Sequence { delivery, mode },
                        child: Box::new(child),
                    }),
                }
            } else {
                JniOutput::Unary {
                    operation: JniOutputUnary::Sequence { delivery, mode },
                    child: Box::new(child),
                }
            }
        }
        JniOutput::Unary {
            operation: JniOutputUnary::Optional,
            child: c,
        } => match lift_output_bulk(*c) {
            JniOutput::Unary {
                operation: JniOutputUnary::Bulk,
                child: c,
            } => JniOutput::Unary {
                operation: JniOutputUnary::Bulk,
                child: Box::new(JniOutput::Unary {
                    operation: JniOutputUnary::Optional,
                    child: c,
                }),
            },
            c => JniOutput::Unary {
                operation: JniOutputUnary::Optional,
                child: Box::new(c),
            },
        },
        JniOutput::Branch {
            operation: JniOutputBranch::Fields,
            children: c,
        } => JniOutput::Branch {
            operation: JniOutputBranch::Fields,
            children: c.into_iter().map(lift_output_bulk).collect(),
        },
        JniOutput::Branch {
            operation: JniOutputBranch::OwnedFields,
            children: c,
        } => JniOutput::Branch {
            operation: JniOutputBranch::OwnedFields,
            children: c.into_iter().map(lift_output_bulk).collect(),
        },
        JniOutput::Branch {
            operation: JniOutputBranch::Record { mode },
            children: fields,
        } => JniOutput::Branch {
            operation: JniOutputBranch::Record { mode },
            children: fields.into_iter().map(lift_output_bulk).collect(),
        },
        JniOutput::Branch {
            operation: JniOutputBranch::Combine,
            children: c,
        } => JniOutput::Branch {
            operation: JniOutputBranch::Combine,
            children: c.into_iter().map(lift_output_bulk).collect(),
        },
        JniOutput::Unary {
            operation: JniOutputUnary::Result { error },
            child: ok,
        } => JniOutput::Unary {
            operation: JniOutputUnary::Result { error },
            child: Box::new(lift_output_bulk(*ok)),
        },
        JniOutput::Unary {
            operation: JniOutputUnary::Getter { function },
            child: output,
        } => JniOutput::Unary {
            operation: JniOutputUnary::Getter { function },
            child: Box::new(lift_output_bulk(*output)),
        },
        JniOutput::Unary {
            operation: JniOutputUnary::Into { function },
            child: output,
        } => JniOutput::Unary {
            operation: JniOutputUnary::Into { function },
            child: Box::new(lift_output_bulk(*output)),
        },
        JniOutput::Unary {
            operation: JniOutputUnary::Named { name },
            child: output,
        } => JniOutput::Unary {
            operation: JniOutputUnary::Named { name },
            child: Box::new(lift_output_bulk(*output)),
        },
        JniOutput::Branch {
            operation,
            children,
        } => JniOutput::Branch {
            operation,
            children: children.into_iter().map(lift_output_bulk).collect(),
        },
        c => c,
    }
}

fn input_has_bulk(tree: &JniInput) -> bool {
    match tree {
        JniInput::Branch {
            operation: JniInputBranch::Enum { .. },
            children: variants,
            ..
        } => variants.iter().any(input_has_bulk),
        JniInput::Unary {
            operation: JniInputUnary::Convert { .. },
            child: input,
            ..
        } => input_has_bulk(input),
        JniInput::Unary {
            operation: JniInputUnary::Bulk,
            child: _,
        } => true,
        JniInput::Unary {
            operation: JniInputUnary::Optional,
            child: c,
        }
        | JniInput::Unary {
            operation: JniInputUnary::Sequence,
            child: c,
        } => input_has_bulk(c),
        JniInput::Branch {
            operation: JniInputBranch::Fields,
            children: c,
        }
        | JniInput::Branch {
            operation: JniInputBranch::Record,
            children: c,
        }
        | JniInput::Branch {
            operation: JniInputBranch::Selector,
            children: c,
        } => c.iter().any(input_has_bulk),
        JniInput::Branch {
            operation: JniInputBranch::Constructor { .. },
            children: args,
            ..
        } => args.iter().any(input_has_bulk),
        JniInput::Branch { children, .. } => children.iter().any(input_has_bulk),
        _ => false,
    }
}
fn output_has_bulk(tree: &JniOutput) -> bool {
    match tree {
        JniOutput::Branch {
            operation: JniOutputBranch::Enum { .. },
            children: variants,
            ..
        } => variants.iter().any(output_has_bulk),
        // Frames inside callback elements belong to those elements, including
        // when the callback sequence is nested in another structural node.
        JniOutput::Unary {
            operation:
                JniOutputUnary::Sequence {
                    delivery: SequenceDelivery::Callback,
                    ..
                },
            ..
        } => false,
        JniOutput::Unary {
            operation: JniOutputUnary::Convert { .. },
            child: output,
            ..
        } => output_has_bulk(output),
        JniOutput::Unary {
            operation: JniOutputUnary::Bulk,
            child: _,
        } => true,
        JniOutput::Unary {
            operation: JniOutputUnary::Optional,
            child: c,
        }
        | JniOutput::Unary {
            operation: JniOutputUnary::Named { .. },
            child: c,
            ..
        }
        | JniOutput::Unary {
            operation: JniOutputUnary::Getter { .. },
            child: c,
            ..
        }
        | JniOutput::Unary {
            operation: JniOutputUnary::Into { .. },
            child: c,
            ..
        }
        | JniOutput::Unary {
            operation: JniOutputUnary::Sequence { .. },
            child: c,
            ..
        } => output_has_bulk(c),
        JniOutput::Branch {
            operation: JniOutputBranch::Fields,
            children: c,
        }
        | JniOutput::Branch {
            operation: JniOutputBranch::OwnedFields,
            children: c,
        }
        | JniOutput::Branch {
            operation: JniOutputBranch::Combine,
            children: c,
        }
        | JniOutput::Branch {
            operation: JniOutputBranch::Record { .. },
            children: c,
            ..
        } => c.iter().any(output_has_bulk),
        JniOutput::Branch { children, .. } => children.iter().any(output_has_bulk),
        _ => false,
    }
}

pub(crate) fn sequence_callback(
    cx: &mut crate::JniResolve<'_>,
    ty: &syn::Type,
    mut output: JniOutput,
    structural_default: bool,
) -> Result<JniOutput> {
    fn select(output: &mut JniOutput) -> usize {
        match output {
            JniOutput::Unary {
                operation: JniOutputUnary::Sequence { delivery, .. },
                ..
            } => {
                *delivery = SequenceDelivery::Callback;
                1
            }
            JniOutput::Unary {
                operation: JniOutputUnary::Optional,
                child,
            }
            | JniOutput::Unary {
                operation: JniOutputUnary::Bulk,
                child,
            }
            | JniOutput::Unary {
                operation: JniOutputUnary::Named { .. },
                child,
                ..
            }
            | JniOutput::Unary {
                operation: JniOutputUnary::Getter { .. },
                child,
                ..
            }
            | JniOutput::Unary {
                operation: JniOutputUnary::Into { .. },
                child,
                ..
            }
            | JniOutput::Unary {
                operation: JniOutputUnary::Convert { .. },
                child,
                ..
            } => select(child),
            JniOutput::Branch {
                operation: JniOutputBranch::Combine,
                children,
            }
            | JniOutput::Branch {
                operation: JniOutputBranch::Fields,
                children,
            }
            | JniOutput::Branch {
                operation: JniOutputBranch::OwnedFields,
                children,
            }
            | JniOutput::Branch {
                operation: JniOutputBranch::Record { .. },
                children,
                ..
            } => children.iter_mut().map(select).sum(),
            _ => 0,
        }
    }
    if select(&mut output) != 1 {
        fn structural(ty: &syn::Type, output: &JniOutput) -> Option<crate::JniOutputConfig> {
            match output {
                JniOutput::Leaf(JniOutputLeafOp::Value)
                    if prebindgen_flat::sequence_parts(ty).is_some() =>
                {
                    Some(crate::JniOutputConfig::Unary {
                        operation: crate::JniOutputUnary::Sequence {
                            delivery: SequenceDelivery::Callback,
                            mode: None,
                        },
                        child: Box::new(crate::JniOutputConfig::auto()),
                    })
                }
                JniOutput::Unary {
                    operation: JniOutputUnary::Optional,
                    child,
                } => Some(crate::JniOutputConfig::Unary {
                    operation: crate::JniOutputUnary::Optional,
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

/// Configuration specialization of [`JniInputArrow`].
pub type JniInputConfigArrow = JniInputArrow<Box<prebindgen_declarations::CallbackConfig>>;
impl From<JniInputArrow> for JniInputConfigArrow {
    fn from(operation: JniInputArrow) -> Self {
        match operation {
            JniInputArrow::ConfigCallback(never) => match never {},
            JniInputArrow::BoundCallback(field0) => Self::BoundCallback(field0),
        }
    }
}
/// Configuration specialization of [`JniInputLeafOp`].
pub type JniInputConfigLeafOp =
    JniInputLeafOp<(), (), Box<prebindgen_declarations::DeclConvert>, String>;
impl From<JniInputLeafOp> for JniInputConfigLeafOp {
    fn from(operation: JniInputLeafOp) -> Self {
        match operation {
            JniInputLeafOp::Value => Self::Value,
            JniInputLeafOp::ConfigDefault(never)
            | JniInputLeafOp::ConfigValue(never)
            | JniInputLeafOp::ConfigConversion(never) => match never {},
            JniInputLeafOp::ConfigConstructor(never) => match never {},
        }
    }
}
/// Configuration specialization of [`JniInputUnary`].
pub type JniInputConfigUnary = JniInputUnary<Box<syn::Type>>;
impl From<JniInputUnary> for JniInputConfigUnary {
    fn from(operation: JniInputUnary) -> Self {
        match operation {
            JniInputUnary::ConfigForType { ty: never } => match never {},
            JniInputUnary::Convert { conversion } => Self::Convert { conversion },
            JniInputUnary::Sequence => Self::Sequence,
            JniInputUnary::Temporary => Self::Temporary,
            JniInputUnary::Bulk => Self::Bulk,
            JniInputUnary::Optional => Self::Optional,
        }
    }
}
/// Configuration specialization of [`JniInputBranch`].
pub type JniInputConfigBranch =
    JniInputBranch<bool, JniVariantFields, JniInputFields, JniConstructorArguments>;
impl From<JniInputBranch> for JniInputConfigBranch {
    fn from(operation: JniInputBranch) -> Self {
        match operation {
            JniInputBranch::ConfigEnumFields(never) => match never {},
            JniInputBranch::ConfigVariantFields(never)
            | JniInputBranch::ConfigNamedFields(never)
            | JniInputBranch::ConfigConstructorArguments(never) => match never {},
            JniInputBranch::Fields => Self::Fields,
            JniInputBranch::Record => Self::Record,
            JniInputBranch::Selector => Self::Selector,
            JniInputBranch::Constructor { function } => Self::Constructor { function },
            JniInputBranch::Enum { value } => Self::Enum { value },
            JniInputBranch::Variant => Self::Variant,
        }
    }
}
/// Configuration specialization of [`JniOutputLeafOp`].
pub type JniOutputConfigLeafOp =
    JniOutputLeafOp<(), Option<Projection>, Box<prebindgen_declarations::DeclConvert>>;
impl From<JniOutputLeafOp> for JniOutputConfigLeafOp {
    fn from(operation: JniOutputLeafOp) -> Self {
        match operation {
            JniOutputLeafOp::Value => Self::Value,
            JniOutputLeafOp::Discard => Self::Discard,
            JniOutputLeafOp::Projected(field0) => Self::Projected(field0),
            JniOutputLeafOp::ConfigDefault(never)
            | JniOutputLeafOp::ConfigValue(never)
            | JniOutputLeafOp::ConfigConversion(never) => match never {},
        }
    }
}
/// Configuration specialization of [`JniOutputUnary`].
pub type JniOutputConfigUnary = JniOutputUnary<Box<syn::Type>, bool>;
impl From<JniOutputUnary> for JniOutputConfigUnary {
    fn from(operation: JniOutputUnary) -> Self {
        match operation {
            JniOutputUnary::ConfigForType { ty: never } => match never {},
            JniOutputUnary::ConfigSequenceCallback { structural: never } => match never {},
            JniOutputUnary::CallbackArgument { ownership } => Self::CallbackArgument { ownership },
            JniOutputUnary::Bulk => Self::Bulk,
            JniOutputUnary::Named { name } => Self::Named { name },
            JniOutputUnary::Convert { conversion } => Self::Convert { conversion },
            JniOutputUnary::Sequence { delivery, mode } => Self::Sequence { delivery, mode },
            JniOutputUnary::Result { error } => Self::Result { error },
            JniOutputUnary::Optional => Self::Optional,
            JniOutputUnary::Into { function } => Self::Into { function },
            JniOutputUnary::Getter { function } => Self::Getter { function },
        }
    }
}
/// Configuration specialization of [`JniOutputBranch`].
pub type JniOutputConfigBranch = JniOutputBranch<bool, JniVariantFields, JniNamedFields>;
impl From<JniOutputBranch> for JniOutputConfigBranch {
    fn from(operation: JniOutputBranch) -> Self {
        match operation {
            JniOutputBranch::ConfigEnumFields(never) => match never {},
            JniOutputBranch::ConfigVariantFields(never) => match never {},
            JniOutputBranch::ConfigNamedFields(never) => match never {},
            JniOutputBranch::Fields => Self::Fields,
            JniOutputBranch::OwnedFields => Self::OwnedFields,
            JniOutputBranch::Record { mode } => Self::Record { mode },
            JniOutputBranch::Combine => Self::Combine,
            JniOutputBranch::Enum { value, mode } => Self::Enum { value, mode },
            JniOutputBranch::Variant => Self::Variant,
            JniOutputBranch::Arguments => Self::Arguments,
        }
    }
}
