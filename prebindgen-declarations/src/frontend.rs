//! Optional expansion/conversion declaration syntax.
//!
//! This is binding-author configuration, not the operation vocabulary stored in
//! a resolved Flat. DeclCodec lowers it into the adapter's own operations.
use crate::{
    InputNode, InputOps, OutputNode, OutputOps, Projection, ProjectionMode, SequenceDelivery,
};
use std::convert::Infallible;
/// Input syntax accepted by the optional declaration frontend.
#[derive(Clone, Debug)]
pub struct DeclInputOps;
/// Output syntax accepted by the optional declaration frontend.
#[derive(Clone, Debug)]
pub struct DeclOutputOps;
impl InputOps for DeclInputOps {
    type Leaf = DeclInputLeafOp;
    type Unary = DeclInputUnary;
    type Branch = DeclInputBranch;
    type Arrow = Box<crate::CallbackOperation>;
}
impl OutputOps for DeclOutputOps {
    type Leaf = DeclOutputLeafOp;
    type Unary = DeclOutputUnary;
    type Branch = DeclOutputBranch;
    type Arrow = Infallible;
}
/// Input syntax before adapter lowering.
///
/// Preparation uses `DeclInput<DeclInputRequest, DeclOutputRequest>`.
/// Plain `DeclInput` uses `Infallible` for both directions, including
/// reverse-direction arrow domains.
/// ```compile_fail
/// use prebindgen_declarations::{DeclInput, frontend::DeclInputBranch, DeclInputRequest};
/// let resolved: DeclInput = DeclInput::Branch {
///     operation: DeclInputBranch::Constructor { function: "make".into() },
///     children: vec![DeclInput::Request(DeclInputRequest::Auto)],
/// };
/// ```
pub type DeclInput<InRequest = Infallible, OutRequest = Infallible> =
    InputNode<DeclInputOps, DeclOutputOps, InRequest, OutRequest>;
/// Output declaration tree, with input requests first and output requests second.
/// ```
/// use prebindgen_declarations::{DeclInput, DeclOutput, DeclInputRequest, DeclOutputRequest};
/// let _: DeclInput<DeclInputRequest, DeclOutputRequest> = DeclInput::auto();
/// let _: DeclOutput<DeclInputRequest, DeclOutputRequest> = DeclOutput::auto();
/// ```
pub type DeclOutput<InRequest = Infallible, OutRequest = Infallible> =
    OutputNode<DeclInputOps, DeclOutputOps, InRequest, OutRequest>;

/// Terminal input operations.
#[derive(Clone, Debug)]
pub enum DeclInputLeafOp {
    /// Convert the occurrence using its declared value representation.
    Value,
}
/// Single-child input operations.
#[derive(Clone, Debug)]
pub enum DeclInputUnary {
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
pub enum DeclInputBranch {
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
pub enum DeclOutputLeafOp {
    /// Convert the occurrence using its declared value representation.
    Value,
    /// Omit a value while preserving its ordinary Rust destruction.
    Discard,
    /// Apply an explicit observation or move policy to a terminal value.
    Projected(Projection),
}
/// Single-child output operations.
#[derive(Clone, Debug)]
pub enum DeclOutputUnary {
    /// Ownership contract of one callable argument; interpreted by its arrow.
    CallbackArgument {
        /// Whether produced foreign owners expire at the end of invocation.
        ownership: crate::CallbackArgumentOwnership,
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
pub enum DeclOutputBranch {
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

impl<InRequest, OutRequest> DeclOutput<InRequest, OutRequest> {
    /// Group a callable's domain in source parameter order.
    pub fn arguments(children: Vec<Self>) -> Self {
        if children.is_empty() {
            Self::None
        } else {
            Self::Branch {
                operation: DeclOutputBranch::Arguments,
                children,
            }
        }
    }
    /// Read the source-ordered components of a callable domain.
    pub fn argument_nodes(&self) -> crate::Result<&[Self]> {
        match self {
            Self::None => Ok(&[]),
            Self::Branch {
                operation: DeclOutputBranch::Arguments,
                children,
            } => Ok(children),
            _ => Err("callback domain requires an argument branch or None".into()),
        }
    }
}

impl<InRequest, OutRequest> DeclInput<InRequest, OutRequest> {
    /// Describe source variants, each with an ordinary branch of field conversions.
    pub fn enum_variants(variants: Vec<Vec<Self>>, value: bool) -> Self {
        Self::Branch {
            operation: DeclInputBranch::Enum { value },
            children: variants
                .into_iter()
                .map(|children| Self::Branch {
                    operation: DeclInputBranch::Variant,
                    children,
                })
                .collect(),
        }
    }
    /// Read an enum variant's field conversions.
    pub fn variant_fields(&self) -> crate::Result<&[Self]> {
        match self {
            Self::Branch {
                operation: DeclInputBranch::Variant,
                children,
            } => Ok(children),
            _ => Err("enum children must be variant branches".into()),
        }
    }
}

impl<InRequest, OutRequest> DeclOutput<InRequest, OutRequest> {
    /// Describe source variants, each with an ordinary branch of field conversions.
    pub fn enum_variants(
        variants: Vec<Vec<Self>>,
        value: bool,
        mode: Option<ProjectionMode>,
    ) -> Self {
        Self::Branch {
            operation: DeclOutputBranch::Enum { value, mode },
            children: variants
                .into_iter()
                .map(|children| Self::Branch {
                    operation: DeclOutputBranch::Variant,
                    children,
                })
                .collect(),
        }
    }
    /// Read an enum variant's field conversions.
    pub fn variant_fields(&self) -> crate::Result<&[Self]> {
        match self {
            Self::Branch {
                operation: DeclOutputBranch::Variant,
                children,
            } => Ok(children),
            _ => Err("enum children must be variant branches".into()),
        }
    }
}

impl<InRequest, OutRequest> DeclInput<InRequest, OutRequest> {
    /// Convert each incoming collection element using the supplied subtree.
    pub fn sequence(element: Self) -> Self {
        Self::Unary {
            operation: DeclInputUnary::Sequence,
            child: Box::new(element),
        }
    }
}

impl<InRequest, OutRequest> DeclOutput<InRequest, OutRequest> {
    /// Attach the lifetime of produced foreign owners to this callable argument.
    pub fn callback_argument(self, ownership: crate::CallbackArgumentOwnership) -> Self {
        Self::Unary {
            operation: DeclOutputUnary::CallbackArgument { ownership },
            child: Box::new(self),
        }
    }
    /// Read an argument's conversion and ownership contract from its own subtree.
    pub fn callback_argument_parts(
        &self,
    ) -> crate::Result<(crate::CallbackArgumentOwnership, &Self)> {
        match self {
            Self::Unary {
                operation: DeclOutputUnary::CallbackArgument { ownership },
                child,
            } => Ok((*ownership, child)),
            _ => Err("callback argument requires an explicit ownership contract".into()),
        }
    }
}
