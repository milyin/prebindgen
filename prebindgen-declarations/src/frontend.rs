//! Optional expansion/conversion declaration syntax.
//!
//! This is binding-author configuration, not the operation vocabulary stored in
//! a resolved Flat. DeclarationCodec lowers it into the adapter's own operations.
use crate::{InputNode, Operations, OutputNode, Projection, ProjectionMode, SequenceDelivery};
use std::convert::Infallible;
/// Input syntax accepted by the optional declaration frontend.
#[derive(Clone, Debug)]
pub struct InputOperations;
/// Output syntax accepted by the optional declaration frontend.
#[derive(Clone, Debug)]
pub struct OutputOperations;
impl Operations for InputOperations {
    type Leaf = InputLeafOp;
    type Unary = InputUnary;
    type Branch = InputBranch;
    type Arrow = Box<crate::CallbackOperation>;
}
impl Operations for OutputOperations {
    type Leaf = OutputLeafOp;
    type Unary = OutputUnary;
    type Branch = OutputBranch;
    type Arrow = Infallible;
}
/// Input syntax before adapter lowering.
///
/// Preparation uses `DeclarationInput<InputRequest, OutputRequest>`.
/// Plain `DeclarationInput` uses `Infallible` for both directions, including
/// reverse-direction arrow domains.
/// ```compile_fail
/// use prebindgen_declarations::{DeclarationInput, frontend::InputBranch, InputRequest};
/// let resolved: DeclarationInput = DeclarationInput::Branch {
///     operation: InputBranch::Constructor { function: "make".into() },
///     children: vec![DeclarationInput::Request(InputRequest::Auto)],
/// };
/// ```
pub type DeclarationInput<P = Infallible, Q = Infallible> =
    InputNode<InputOperations, OutputOperations, P, Q>;
/// Output declaration tree; both request parameters must be uninhabited after resolution.
pub type DeclarationOutput<P = Infallible, Q = Infallible> =
    OutputNode<InputOperations, OutputOperations, Q, P>;

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

impl<P, Q> DeclarationOutput<P, Q> {
    /// Group a callable's domain in source parameter order.
    pub fn arguments(children: Vec<Self>) -> Self {
        if children.is_empty() {
            Self::None
        } else {
            Self::Branch {
                operation: OutputBranch::Arguments,
                children,
            }
        }
    }
    /// Read the source-ordered components of a callable domain.
    pub fn argument_nodes(&self) -> crate::Result<&[Self]> {
        match self {
            Self::None => Ok(&[]),
            Self::Branch {
                operation: OutputBranch::Arguments,
                children,
            } => Ok(children),
            _ => Err("callback domain requires an argument branch or None".into()),
        }
    }
}

impl<P, Q> DeclarationInput<P, Q> {
    /// Describe source variants, each with an ordinary branch of field conversions.
    pub fn enum_variants(variants: Vec<Vec<Self>>, value: bool) -> Self {
        Self::Branch {
            operation: InputBranch::Enum { value },
            children: variants
                .into_iter()
                .map(|children| Self::Branch {
                    operation: InputBranch::Variant,
                    children,
                })
                .collect(),
        }
    }
    /// Read an enum variant's field conversions.
    pub fn variant_fields(&self) -> crate::Result<&[Self]> {
        match self {
            Self::Branch {
                operation: InputBranch::Variant,
                children,
            } => Ok(children),
            _ => Err("enum children must be variant branches".into()),
        }
    }
}

impl<P, Q> DeclarationOutput<P, Q> {
    /// Describe source variants, each with an ordinary branch of field conversions.
    pub fn enum_variants(
        variants: Vec<Vec<Self>>,
        value: bool,
        mode: Option<ProjectionMode>,
    ) -> Self {
        Self::Branch {
            operation: OutputBranch::Enum { value, mode },
            children: variants
                .into_iter()
                .map(|children| Self::Branch {
                    operation: OutputBranch::Variant,
                    children,
                })
                .collect(),
        }
    }
    /// Read an enum variant's field conversions.
    pub fn variant_fields(&self) -> crate::Result<&[Self]> {
        match self {
            Self::Branch {
                operation: OutputBranch::Variant,
                children,
            } => Ok(children),
            _ => Err("enum children must be variant branches".into()),
        }
    }
}

impl<P, Q> DeclarationInput<P, Q> {
    /// Convert each incoming collection element using the supplied subtree.
    pub fn sequence(element: Self) -> Self {
        Self::Unary {
            operation: InputUnary::Sequence,
            child: Box::new(element),
        }
    }
}

impl<P, Q> DeclarationOutput<P, Q> {
    /// Attach the lifetime of produced foreign owners to this callable argument.
    pub fn callback_argument(self, ownership: crate::CallbackArgumentOwnership) -> Self {
        Self::Unary {
            operation: OutputUnary::CallbackArgument { ownership },
            child: Box::new(self),
        }
    }
    /// Read an argument's conversion and ownership contract from its own subtree.
    pub fn callback_argument_parts(
        &self,
    ) -> crate::Result<(crate::CallbackArgumentOwnership, &Self)> {
        match self {
            Self::Unary {
                operation: OutputUnary::CallbackArgument { ownership },
                child,
            } => Ok((*ownership, child)),
            _ => Err("callback argument requires an explicit ownership contract".into()),
        }
    }
}
