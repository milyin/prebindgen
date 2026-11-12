//! Adapter-owned conversion operations, after declaration requests have been resolved.
use prebindgen_declarations::{
    CallbackArgumentOwnership, CallbackOperation, InputNode, Operations, OutputNode, Projection,
    ProjectionMode, SequenceDelivery,
};
use prebindgen_flat::Result;
use std::convert::Infallible;
/// Input operations for this adapter.
#[derive(Clone, Debug)]
pub struct InputOperations;
/// Output operations for this adapter.
#[derive(Clone, Debug)]
pub struct OutputOperations;
impl Operations for InputOperations {
    type Leaf = InputLeafOp;
    type Unary = InputUnary;
    type Branch = InputBranch;
    type Arrow = Box<CallbackOperation>;
}
impl Operations for OutputOperations {
    type Leaf = OutputLeafOp;
    type Unary = OutputUnary;
    type Branch = OutputBranch;
    type Arrow = Infallible;
}
/// Resolved input conversion for this adapter.
pub type Input = InputNode<InputOperations, OutputOperations>;
/// Resolved output conversion for this adapter.
pub type Output = OutputNode<InputOperations, OutputOperations>;

/// Terminal input operations.
#[derive(Clone, Debug)]
pub enum InputLeafOp {
    /// Convert the occurrence using its declared value representation.
    Value,
}
/// Single-child input operations.
#[derive(Clone, Debug)]
pub enum InputUnary {
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
pub enum InputBranch {
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
pub enum OutputLeafOp {
    /// Convert the occurrence using its declared value representation.
    Value,
    /// Omit a value while preserving its ordinary Rust destruction.
    Discard,
    /// Apply an explicit observation or move policy to a terminal value.
    Projected(Projection),
}
/// Single-child output operations.
#[derive(Clone, Debug)]
pub enum OutputUnary {
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
pub enum OutputBranch {
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

pub(crate) trait InputParts {
    fn variant_fields(&self) -> Result<&[Self]>
    where
        Self: Sized;
}
impl InputParts for Input {
    fn variant_fields(&self) -> Result<&[Self]> {
        match self {
            Self::Branch {
                operation: InputBranch::Variant,
                children,
            } => Ok(children),
            _ => Err("enum children must be variant branches".into()),
        }
    }
}
pub(crate) trait OutputParts {
    fn variant_fields(&self) -> Result<&[Self]>
    where
        Self: Sized;
    fn argument_nodes(&self) -> Result<&[Self]>
    where
        Self: Sized;
    fn callback_argument_parts(&self) -> Result<(CallbackArgumentOwnership, &Self)>;
}
impl OutputParts for Output {
    fn variant_fields(&self) -> Result<&[Self]> {
        match self {
            Self::Branch {
                operation: OutputBranch::Variant,
                children,
            } => Ok(children),
            _ => Err("enum children must be variant branches".into()),
        }
    }
    fn argument_nodes(&self) -> Result<&[Self]> {
        match self {
            Self::None => Ok(&[]),
            Self::Branch {
                operation: OutputBranch::Arguments,
                children,
            } => Ok(children),
            _ => Err("callback domain requires an argument branch or None".into()),
        }
    }
    fn callback_argument_parts(&self) -> Result<(CallbackArgumentOwnership, &Self)> {
        match self {
            Self::Unary {
                operation: OutputUnary::CallbackArgument { ownership },
                child,
            } => Ok((*ownership, child)),
            _ => Err("callback argument requires an ownership contract".into()),
        }
    }
}

pub(crate) fn sequence_callback(
    cx: &mut prebindgen_declarations::Resolve<'_, crate::CCodec>,
    ty: &syn::Type,
    mut output: Output,
    structural_default: bool,
) -> Result<Output> {
    fn select(output: &mut Output) -> usize {
        match output {
            Output::Unary {
                operation: OutputUnary::Sequence { delivery, .. },
                ..
            } => {
                *delivery = SequenceDelivery::Callback;
                1
            }
            Output::Unary {
                operation: OutputUnary::Optional,
                child,
            }
            | Output::Unary {
                operation: OutputUnary::Named { .. },
                child,
                ..
            }
            | Output::Unary {
                operation: OutputUnary::Getter { .. },
                child,
                ..
            }
            | Output::Unary {
                operation: OutputUnary::Into { .. },
                child,
                ..
            }
            | Output::Unary {
                operation: OutputUnary::Convert { .. },
                child,
                ..
            } => select(child),
            Output::Branch {
                operation: OutputBranch::Combine,
                children,
            }
            | Output::Branch {
                operation: OutputBranch::Fields,
                children,
            }
            | Output::Branch {
                operation: OutputBranch::OwnedFields,
                children,
            } => children.iter_mut().map(select).sum(),
            _ => 0,
        }
    }
    if select(&mut output) != 1 {
        fn structural(
            ty: &syn::Type,
            output: &Output,
        ) -> Option<
            prebindgen_declarations::DeclarationOutput<
                prebindgen_declarations::OutputRequest,
                prebindgen_declarations::InputRequest,
            >,
        > {
            match output {
                Output::Leaf(OutputLeafOp::Value)
                    if prebindgen_flat::sequence_parts(ty).is_some() =>
                {
                    Some(prebindgen_declarations::DeclarationOutput::Unary {
                        operation: prebindgen_declarations::frontend::OutputUnary::Sequence {
                            delivery: SequenceDelivery::Callback,
                            mode: None,
                        },
                        child: Box::new(prebindgen_declarations::DeclarationOutput::auto()),
                    })
                }
                Output::Unary {
                    operation: OutputUnary::Optional,
                    child,
                } => Some(prebindgen_declarations::DeclarationOutput::Unary {
                    operation: prebindgen_declarations::frontend::OutputUnary::Optional,
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
