//! Adapter-owned conversion metadata for source resolution and compilation.
use crate::COutputRequests;
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
/// Input conversion for this adapter. Leaf requests default to Infallible.
/// Metadata from one structural shape cannot be used at another:
/// ```compile_fail
/// use prebindgen_c::{CInput,CInputUnary,CInputBranch};
/// let _:CInput=CInput::Unary {
///     operation:CInputBranch::EnumFields,
///     child:Box::new(CInput::None),
/// };
/// ```
pub type CInput<InRequest = Infallible, OutRequest = Infallible> =
    InputNode<CInputOps, COutputOps, InRequest, OutRequest>;
/// Output conversion for this adapter. Leaf requests default to Infallible.
pub type COutput<InRequest = Infallible, OutRequest = Infallible> =
    OutputNode<CInputOps, COutputOps, InRequest, OutRequest>;

/// Incoming callback configuration and its resolved retained-callback binding.
#[derive(Clone, Debug)]
pub enum CInputArrow {
    /// Resolve explicit argument/result conversions against callback defaults.
    Callback(Box<prebindgen_declarations::CallbackRequest>),
    /// Canonical callback signature and settings with resolved argument/result children.
    BoundCallback(Box<prebindgen_declarations::CallbackOperation>),
}

impl CInputArrow {
    pub(crate) fn bound_callback(&self) -> Result<&prebindgen_declarations::CallbackOperation> {
        match self {
            Self::BoundCallback(callback) => Ok(callback),
            Self::Callback(_) => Err("resolve callback defaults before compilation".into()),
        }
    }
}

/// Terminal input operations.
#[derive(Clone, Debug)]
pub enum CInputLeafOp {
    /// Convert the occurrence using its declared value representation.
    Value,
}
/// Single-child input operations.
#[derive(Clone, Debug)]
pub enum CInputUnary {
    /// Apply the child conversion at this source type, including nested occurrences.
    ForType {
        /// Source type at which the child applies.
        ty: Box<syn::Type>,
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
pub enum CInputBranch {
    /// Partially specified enum fields; children are variant branches.
    EnumFields,
    /// Partially specified fields of one source variant.
    VariantFields {
        /// Source variant name.
        variant: String,
        /// Source field names corresponding to the child conversions.
        fields: Vec<String>,
    },

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
pub enum COutputLeafOp {
    /// Convert the occurrence using its declared value representation.
    Value,
    /// Omit a value while preserving its ordinary Rust destruction.
    Discard,
    /// Apply an explicit observation or move policy to a terminal value.
    Projected(Projection),
}
/// Single-child output operations.
#[derive(Clone, Debug)]
pub enum COutputUnary {
    /// Apply the child conversion at this source type, including nested occurrences.
    ForType {
        /// Source type at which the child applies.
        ty: Box<syn::Type>,
    },
    /// Select per-element callback delivery of the child's sequence.
    SequenceCallback {
        /// Permit decomposition of an ordinary collection identity.
        structural: bool,
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
pub enum COutputBranch {
    /// Partially specified enum fields; children are variant branches.
    EnumFields,
    /// Partially specified fields of one source variant.
    VariantFields {
        /// Source variant name.
        variant: String,
        /// Source field names corresponding to the child conversions.
        fields: Vec<String>,
    },
    /// Partially specified record fields, in the same order as the child conversions.
    NamedFields {
        /// Source field names corresponding to the child conversions.
        fields: Vec<String>,
        /// Observe or move the selected fields.
        mode: ProjectionMode,
    },

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
        fn structural(
            ty: &syn::Type,
            output: &COutput,
        ) -> Option<crate::COutput<crate::CInputRequest, crate::COutputRequest>> {
            match output {
                COutput::Leaf(COutputLeafOp::Value)
                    if prebindgen_flat::sequence_parts(ty).is_some() =>
                {
                    Some(crate::COutput::Unary {
                        operation: crate::COutputUnary::Sequence {
                            delivery: SequenceDelivery::Callback,
                            mode: None,
                        },
                        child: Box::new(crate::COutput::auto()),
                    })
                }
                COutput::Unary {
                    operation: COutputUnary::Optional,
                    child,
                } => Some(crate::COutput::Unary {
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
