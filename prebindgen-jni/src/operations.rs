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
    /// Request the adapter’s packed collection/record transport.
    Bulk,
    /// Convert the payload only when present.
    Optional,
}
/// Multiple-child input operations.
#[derive(Clone, Debug)]
pub enum InputBranch {
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
pub enum OutputBranch {
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
    #[cfg(test)]
    fn bulk(self) -> Self
    where
        Self: Sized;
    fn variant_fields(&self) -> Result<&[Self]>
    where
        Self: Sized;
    fn argument_nodes(&self) -> Result<&[Self]>
    where
        Self: Sized;
    fn callback_argument_parts(&self) -> Result<(CallbackArgumentOwnership, &Self)>;
}
impl OutputParts for Output {
    #[cfg(test)]
    fn bulk(self) -> Self {
        Self::Unary {
            operation: OutputUnary::Bulk,
            child: Box::new(self),
        }
    }
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

pub(crate) fn lift_input_bulk(tree: Input) -> Input {
    match tree {
        Input::Branch {
            operation: InputBranch::Enum { value },
            children: variants,
        } => Input::Branch {
            operation: InputBranch::Enum { value },
            children: variants.into_iter().map(lift_input_bulk).collect(),
        },
        Input::Unary {
            operation: InputUnary::Convert { conversion },
            child: input,
        } => Input::Unary {
            operation: InputUnary::Convert { conversion },
            child: Box::new(lift_input_bulk(*input)),
        },
        Input::Unary {
            operation: InputUnary::Bulk,
            child: c,
        } => Input::Unary {
            operation: InputUnary::Bulk,
            child: Box::new(lift_input_bulk(*c)),
        },
        Input::Unary {
            operation: InputUnary::Temporary,
            child: c,
        } => Input::Unary {
            operation: InputUnary::Temporary,
            child: Box::new(lift_input_bulk(*c)),
        },
        Input::Unary {
            operation: InputUnary::Sequence,
            child: c,
        } => {
            let child = lift_input_bulk(*c);
            if input_has_bulk(&child) {
                Input::Unary {
                    operation: InputUnary::Bulk,
                    child: Box::new(Input::Unary {
                        operation: InputUnary::Sequence,
                        child: Box::new(child),
                    }),
                }
            } else {
                Input::Unary {
                    operation: InputUnary::Sequence,
                    child: Box::new(child),
                }
            }
        }
        Input::Unary {
            operation: InputUnary::Optional,
            child: c,
        } => match lift_input_bulk(*c) {
            Input::Unary {
                operation: InputUnary::Bulk,
                child: c,
            } => Input::Unary {
                operation: InputUnary::Bulk,
                child: Box::new(Input::Unary {
                    operation: InputUnary::Optional,
                    child: c,
                }),
            },
            c => Input::Unary {
                operation: InputUnary::Optional,
                child: Box::new(c),
            },
        },
        Input::Branch {
            operation: InputBranch::Fields,
            children: c,
        } => Input::Branch {
            operation: InputBranch::Fields,
            children: c.into_iter().map(lift_input_bulk).collect(),
        },
        Input::Branch {
            operation: InputBranch::Record,
            children: c,
        } => Input::Branch {
            operation: InputBranch::Record,
            children: c.into_iter().map(lift_input_bulk).collect(),
        },
        Input::Branch {
            operation: InputBranch::Selector,
            children: c,
        } => Input::Branch {
            operation: InputBranch::Selector,
            children: c.into_iter().map(lift_input_bulk).collect(),
        },
        Input::Branch {
            operation: InputBranch::Constructor { function },
            children: args,
        } => Input::Branch {
            operation: InputBranch::Constructor { function },
            children: args.into_iter().map(lift_input_bulk).collect(),
        },
        Input::Branch {
            operation,
            children,
        } => Input::Branch {
            operation,
            children: children.into_iter().map(lift_input_bulk).collect(),
        },
        c => c,
    }
}
pub(crate) fn lift_output_bulk(tree: Output) -> Output {
    match tree {
        Output::Branch {
            operation: OutputBranch::Enum { value, mode },
            children: variants,
        } => Output::Branch {
            operation: OutputBranch::Enum { value, mode },
            children: variants.into_iter().map(lift_output_bulk).collect(),
        },
        Output::Unary {
            operation: OutputUnary::Convert { conversion },
            child: output,
        } => Output::Unary {
            operation: OutputUnary::Convert { conversion },
            child: Box::new(lift_output_bulk(*output)),
        },
        Output::Unary {
            operation: OutputUnary::Bulk,
            child: c,
        } => Output::Unary {
            operation: OutputUnary::Bulk,
            child: Box::new(lift_output_bulk(*c)),
        },
        Output::Unary {
            operation: OutputUnary::Sequence { delivery, mode },
            child: element,
        } => {
            let child = lift_output_bulk(*element);
            if delivery != SequenceDelivery::Callback && output_has_bulk(&child) {
                Output::Unary {
                    operation: OutputUnary::Bulk,
                    child: Box::new(Output::Unary {
                        operation: OutputUnary::Sequence { delivery, mode },
                        child: Box::new(child),
                    }),
                }
            } else {
                Output::Unary {
                    operation: OutputUnary::Sequence { delivery, mode },
                    child: Box::new(child),
                }
            }
        }
        Output::Unary {
            operation: OutputUnary::Optional,
            child: c,
        } => match lift_output_bulk(*c) {
            Output::Unary {
                operation: OutputUnary::Bulk,
                child: c,
            } => Output::Unary {
                operation: OutputUnary::Bulk,
                child: Box::new(Output::Unary {
                    operation: OutputUnary::Optional,
                    child: c,
                }),
            },
            c => Output::Unary {
                operation: OutputUnary::Optional,
                child: Box::new(c),
            },
        },
        Output::Branch {
            operation: OutputBranch::Fields,
            children: c,
        } => Output::Branch {
            operation: OutputBranch::Fields,
            children: c.into_iter().map(lift_output_bulk).collect(),
        },
        Output::Branch {
            operation: OutputBranch::OwnedFields,
            children: c,
        } => Output::Branch {
            operation: OutputBranch::OwnedFields,
            children: c.into_iter().map(lift_output_bulk).collect(),
        },
        Output::Branch {
            operation: OutputBranch::Record { mode },
            children: fields,
        } => Output::Branch {
            operation: OutputBranch::Record { mode },
            children: fields.into_iter().map(lift_output_bulk).collect(),
        },
        Output::Branch {
            operation: OutputBranch::Combine,
            children: c,
        } => Output::Branch {
            operation: OutputBranch::Combine,
            children: c.into_iter().map(lift_output_bulk).collect(),
        },
        Output::Unary {
            operation: OutputUnary::Result { error },
            child: ok,
        } => Output::Unary {
            operation: OutputUnary::Result { error },
            child: Box::new(lift_output_bulk(*ok)),
        },
        Output::Unary {
            operation: OutputUnary::Getter { function },
            child: output,
        } => Output::Unary {
            operation: OutputUnary::Getter { function },
            child: Box::new(lift_output_bulk(*output)),
        },
        Output::Unary {
            operation: OutputUnary::Into { function },
            child: output,
        } => Output::Unary {
            operation: OutputUnary::Into { function },
            child: Box::new(lift_output_bulk(*output)),
        },
        Output::Unary {
            operation: OutputUnary::Named { name },
            child: output,
        } => Output::Unary {
            operation: OutputUnary::Named { name },
            child: Box::new(lift_output_bulk(*output)),
        },
        Output::Branch {
            operation,
            children,
        } => Output::Branch {
            operation,
            children: children.into_iter().map(lift_output_bulk).collect(),
        },
        c => c,
    }
}

fn input_has_bulk(tree: &Input) -> bool {
    match tree {
        Input::Branch {
            operation: InputBranch::Enum { .. },
            children: variants,
            ..
        } => variants.iter().any(input_has_bulk),
        Input::Unary {
            operation: InputUnary::Convert { .. },
            child: input,
            ..
        } => input_has_bulk(input),
        Input::Unary {
            operation: InputUnary::Bulk,
            child: _,
        } => true,
        Input::Unary {
            operation: InputUnary::Optional,
            child: c,
        }
        | Input::Unary {
            operation: InputUnary::Sequence,
            child: c,
        } => input_has_bulk(c),
        Input::Branch {
            operation: InputBranch::Fields,
            children: c,
        }
        | Input::Branch {
            operation: InputBranch::Record,
            children: c,
        }
        | Input::Branch {
            operation: InputBranch::Selector,
            children: c,
        } => c.iter().any(input_has_bulk),
        Input::Branch {
            operation: InputBranch::Constructor { .. },
            children: args,
            ..
        } => args.iter().any(input_has_bulk),
        Input::Branch { children, .. } => children.iter().any(input_has_bulk),
        _ => false,
    }
}
fn output_has_bulk(tree: &Output) -> bool {
    match tree {
        Output::Branch {
            operation: OutputBranch::Enum { .. },
            children: variants,
            ..
        } => variants.iter().any(output_has_bulk),
        // Frames inside callback elements belong to those elements, including
        // when the callback sequence is nested in another structural node.
        Output::Unary {
            operation:
                OutputUnary::Sequence {
                    delivery: SequenceDelivery::Callback,
                    ..
                },
            ..
        } => false,
        Output::Unary {
            operation: OutputUnary::Convert { .. },
            child: output,
            ..
        } => output_has_bulk(output),
        Output::Unary {
            operation: OutputUnary::Bulk,
            child: _,
        } => true,
        Output::Unary {
            operation: OutputUnary::Optional,
            child: c,
        }
        | Output::Unary {
            operation: OutputUnary::Named { .. },
            child: c,
            ..
        }
        | Output::Unary {
            operation: OutputUnary::Getter { .. },
            child: c,
            ..
        }
        | Output::Unary {
            operation: OutputUnary::Into { .. },
            child: c,
            ..
        }
        | Output::Unary {
            operation: OutputUnary::Sequence { .. },
            child: c,
            ..
        } => output_has_bulk(c),
        Output::Branch {
            operation: OutputBranch::Fields,
            children: c,
        }
        | Output::Branch {
            operation: OutputBranch::OwnedFields,
            children: c,
        }
        | Output::Branch {
            operation: OutputBranch::Combine,
            children: c,
        }
        | Output::Branch {
            operation: OutputBranch::Record { .. },
            children: c,
            ..
        } => c.iter().any(output_has_bulk),
        Output::Branch { children, .. } => children.iter().any(output_has_bulk),
        _ => false,
    }
}

pub(crate) fn sequence_callback(
    cx: &mut prebindgen_declarations::Resolve<'_, crate::JniCodec>,
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
                operation: OutputUnary::Bulk,
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
            }
            | Output::Branch {
                operation: OutputBranch::Record { .. },
                children,
                ..
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
