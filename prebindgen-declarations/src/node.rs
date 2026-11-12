//! Structural conversion trees. Operations describe meaning; children describe composition.
use crate::{Projection, ProjectionMode, SequenceDelivery};
use std::convert::Infallible;

/// Adapter-selected meanings for the structural nodes of one conversion direction.
pub trait Operations: Clone + std::fmt::Debug {
    /// A terminal conversion.
    type Leaf: Clone + std::fmt::Debug;
    /// An operation on one conversion.
    type Unary: Clone + std::fmt::Debug;
    /// An operation combining conversions.
    type Branch: Clone + std::fmt::Debug;
    /// A callable representation. Its domain reverses conversion direction.
    type Arrow: Clone + std::fmt::Debug;
}

/// A foreign-to-Rust conversion, with structurally visible reverse-direction domains.
#[derive(Clone, Debug)]
pub enum InputNode<I: Operations, O: Operations, IR = Infallible, OR = Infallible> {
    /// Unit: no values cross the boundary.
    None,
    /// A choice to resolve before compilation. Resolved trees use Infallible.
    Request(IR),
    /// An adapter-defined terminal conversion.
    Leaf(I::Leaf),
    /// One conversion in the same direction.
    Unary {
        /// Interpretation of the child conversion.
        operation: I::Unary,
        /// Operand, in the enclosing conversion direction.
        child: Box<Self>,
    },
    /// Multiple conversions in the same direction.
    Branch {
        /// Interpretation of the child group.
        operation: I::Branch,
        /// Components in operation-defined order.
        children: Vec<Self>,
    },
    /// A callable whose arguments leave Rust and whose result enters Rust.
    Arrow {
        /// Callable representation and invocation policy.
        operation: I::Arrow,
        /// Argument conversion, in the opposite direction.
        domain: Box<OutputNode<I, O, IR, OR>>,
        /// Result conversion, in the enclosing direction.
        codomain: Box<Self>,
    },
}

/// A Rust-to-foreign conversion, with structurally visible reverse-direction domains.
#[derive(Clone, Debug)]
pub enum OutputNode<I: Operations, O: Operations, IR = Infallible, OR = Infallible> {
    /// Unit: no values cross the boundary.
    None,
    /// A choice to resolve before compilation. Resolved trees use Infallible.
    Request(OR),
    /// An adapter-defined terminal conversion.
    Leaf(O::Leaf),
    /// One conversion in the same direction.
    Unary {
        /// Interpretation of the child conversion.
        operation: O::Unary,
        /// Operand, in the enclosing conversion direction.
        child: Box<Self>,
    },
    /// Multiple conversions in the same direction.
    Branch {
        /// Interpretation of the child group.
        operation: O::Branch,
        /// Components in operation-defined order.
        children: Vec<Self>,
    },
    /// A callable whose arguments enter Rust and whose result leaves Rust.
    Arrow {
        /// Callable representation and invocation policy.
        operation: O::Arrow,
        /// Argument conversion, in the opposite direction.
        domain: Box<InputNode<I, O, IR, OR>>,
        /// Result conversion, in the enclosing direction.
        codomain: Box<Self>,
    },
}

/// Input operations implemented by the shared declaration compiler.
#[derive(Clone, Debug)]
pub struct InputOperations;
/// Output operations implemented by the shared declaration compiler.
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
/// Input operations supported by the shared compiler.
///
/// Preparation uses `Input<InputRequest, OutputRequest>`. Plain `Input` uses
/// `Infallible` for both directions, including reverse-direction arrow domains.
/// ```compile_fail
/// use prebindgen_declarations::{Input, InputBranch, InputRequest};
/// let resolved: Input = Input::Branch {
///     operation: InputBranch::Constructor { function: "make".into() },
///     children: vec![Input::Request(InputRequest::Auto)],
/// };
/// ```
pub type Input<P = Infallible, Q = Infallible> = InputNode<InputOperations, OutputOperations, P, Q>;
/// Output declaration tree; both request parameters must be uninhabited after resolution.
pub type Output<P = Infallible, Q = Infallible> =
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

impl<I: Operations, O: Operations, IR, OR> InputNode<I, O, IR, OR> {
    /// Transform requests throughout the tree, including reverse-direction domains.
    pub fn map_requests<IR2, OR2>(
        self,
        input: &mut impl FnMut(IR) -> IR2,
        output: &mut impl FnMut(OR) -> OR2,
    ) -> InputNode<I, O, IR2, OR2> {
        match self {
            Self::None => InputNode::None,
            Self::Request(r) => InputNode::Request(input(r)),
            Self::Leaf(op) => InputNode::Leaf(op),
            Self::Unary { operation, child } => InputNode::Unary {
                operation,
                child: Box::new(child.map_requests(input, output)),
            },
            Self::Branch {
                operation,
                children,
            } => InputNode::Branch {
                operation,
                children: children
                    .into_iter()
                    .map(|child| child.map_requests(input, output))
                    .collect(),
            },
            Self::Arrow {
                operation,
                domain,
                codomain,
            } => InputNode::Arrow {
                operation,
                domain: Box::new(domain.map_requests(input, output)),
                codomain: Box::new(codomain.map_requests(input, output)),
            },
        }
    }
}

impl<I: Operations, O: Operations, IR, OR> OutputNode<I, O, IR, OR> {
    /// Transform requests throughout the tree, including reverse-direction domains.
    pub fn map_requests<IR2, OR2>(
        self,
        input: &mut impl FnMut(IR) -> IR2,
        output: &mut impl FnMut(OR) -> OR2,
    ) -> OutputNode<I, O, IR2, OR2> {
        match self {
            Self::None => OutputNode::None,
            Self::Request(r) => OutputNode::Request(output(r)),
            Self::Leaf(op) => OutputNode::Leaf(op),
            Self::Unary { operation, child } => OutputNode::Unary {
                operation,
                child: Box::new(child.map_requests(input, output)),
            },
            Self::Branch {
                operation,
                children,
            } => OutputNode::Branch {
                operation,
                children: children
                    .into_iter()
                    .map(|child| child.map_requests(input, output))
                    .collect(),
            },
            Self::Arrow {
                operation,
                domain,
                codomain,
            } => OutputNode::Arrow {
                operation,
                domain: Box::new(domain.map_requests(input, output)),
                codomain: Box::new(codomain.map_requests(input, output)),
            },
        }
    }
}

impl<P, Q> Output<P, Q> {
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

impl<P, Q> Input<P, Q> {
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

impl<P, Q> Output<P, Q> {
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

impl<P, Q> Input<P, Q> {
    /// Convert each incoming collection element using the supplied subtree.
    pub fn sequence(element: Self) -> Self {
        Self::Unary {
            operation: InputUnary::Sequence,
            child: Box::new(element),
        }
    }
}

impl<P, Q> Output<P, Q> {
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

#[cfg(test)]
mod tests {
    use super::*;
    #[derive(Clone, Debug)]
    struct Incoming;
    #[derive(Clone, Debug)]
    struct Outgoing;
    impl Operations for Incoming {
        type Leaf = u8;
        type Unary = bool;
        type Branch = usize;
        type Arrow = &'static str;
    }
    impl Operations for Outgoing {
        type Leaf = &'static str;
        type Unary = usize;
        type Branch = bool;
        type Arrow = u8;
    }
    #[test]
    fn nested_arrows_keep_distinct_operations_and_resolve_both_request_directions() {
        let tree: InputNode<Incoming, Outgoing, u8, &'static str> = InputNode::Arrow {
            operation: "outer",
            domain: Box::new(OutputNode::Branch {
                operation: true,
                children: vec![
                    OutputNode::Request("out"),
                    OutputNode::Arrow {
                        operation: 7,
                        domain: Box::new(InputNode::Request(42)),
                        codomain: Box::new(OutputNode::Leaf("already resolved")),
                    },
                ],
            }),
            codomain: Box::new(InputNode::Request(5)),
        };
        let mut incoming = Vec::new();
        let mut outgoing = Vec::new();
        let mapped = tree.map_requests(
            &mut |r| {
                incoming.push(r);
                r + 1
            },
            &mut |r| {
                outgoing.push(r);
                r.len()
            },
        );
        assert_eq!(incoming, [42, 5]);
        assert_eq!(outgoing, ["out"]);
        let InputNode::Arrow {
            domain, codomain, ..
        } = mapped
        else {
            panic!()
        };
        assert!(matches!(*codomain, InputNode::Request(6)));
        let OutputNode::Branch { children, .. } = *domain else {
            panic!()
        };
        assert!(matches!(children[0], OutputNode::Request(3)));
        let OutputNode::Arrow { domain, .. } = &children[1] else {
            panic!()
        };
        assert!(matches!(**domain, InputNode::Request(43)));
    }
}
