//! Adapter-owned conversion operations, after declaration requests have been resolved.
use crate::JniOutputRequests;
use prebindgen_declarations::{
    CallbackArgumentOwnership, CallbackOperation, InputNode, InputOps, OutputNode, OutputOps,
    Projection, ProjectionMode, SequenceDelivery,
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
    type Arrow = Box<CallbackOperation>;
}
impl OutputOps for JniOutputOps {
    type Leaf = JniOutputLeafOp;
    type Unary = JniOutputUnary;
    type Branch = JniOutputBranch;
    type Arrow = Infallible;
}
/// Input conversion, optionally carrying this adapter's unresolved requests.
/// ```
/// use prebindgen_jni::{JniInput,JniOutput,JniInputRequest,JniOutputRequest,prelude::*};
/// let _: JniInput<JniInputRequest,JniOutputRequest> = JniInput::auto();
/// let _: JniOutput<JniInputRequest,JniOutputRequest> = JniOutput::auto();
/// ```
pub type JniInput<InRequest = Infallible, OutRequest = Infallible> =
    InputNode<JniInputOps, JniOutputOps, InRequest, OutRequest>;
/// Resolved output conversion for this adapter.
pub type JniOutput<InRequest = Infallible, OutRequest = Infallible> =
    OutputNode<JniInputOps, JniOutputOps, InRequest, OutRequest>;

/// Terminal input operations.
#[derive(Clone, Debug)]
pub enum JniInputLeafOp {
    /// Convert the occurrence using its declared value representation.
    Value,
}
/// Single-child input operations.
#[derive(Clone, Debug)]
pub enum JniInputUnary {
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
pub enum JniInputBranch {
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
pub enum JniOutputLeafOp {
    /// Convert the occurrence using its declared value representation.
    Value,
    /// Omit a value while preserving its ordinary Rust destruction.
    Discard,
    /// Apply an explicit observation or move policy to a terminal value.
    Projected(Projection),
}
/// Single-child output operations.
#[derive(Clone, Debug)]
pub enum JniOutputUnary {
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
pub enum JniOutputBranch {
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
        fn structural(
            ty: &syn::Type,
            output: &JniOutput,
        ) -> Option<crate::JniOutput<crate::JniInputRequest, crate::JniOutputRequest>> {
            match output {
                JniOutput::Leaf(JniOutputLeafOp::Value)
                    if prebindgen_flat::sequence_parts(ty).is_some() =>
                {
                    Some(crate::JniOutput::Unary {
                        operation: crate::JniOutputUnary::Sequence {
                            delivery: SequenceDelivery::Callback,
                            mode: None,
                        },
                        child: Box::new(crate::JniOutput::auto()),
                    })
                }
                JniOutput::Unary {
                    operation: JniOutputUnary::Optional,
                    child,
                } => Some(crate::JniOutput::Unary {
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
