//! Structural conversion trees. Operations describe meaning; children describe composition.
use std::convert::Infallible;

/// Operations used when foreign values enter Rust.
pub trait InputOps: Clone + std::fmt::Debug {
    /// Terminal input conversion.
    type Leaf: Clone + std::fmt::Debug;
    /// Operation on one input conversion.
    type Unary: Clone + std::fmt::Debug;
    /// Operation combining input conversions.
    type Branch: Clone + std::fmt::Debug;
    /// Incoming callable representation; its domain reverses direction.
    type Arrow: Clone + std::fmt::Debug;
}
/// Operations used when Rust values leave Rust.
pub trait OutputOps: Clone + std::fmt::Debug {
    /// Terminal output conversion.
    type Leaf: Clone + std::fmt::Debug;
    /// Operation on one output conversion.
    type Unary: Clone + std::fmt::Debug;
    /// Operation combining output conversions.
    type Branch: Clone + std::fmt::Debug;
    /// Outgoing callable representation; its domain reverses direction.
    type Arrow: Clone + std::fmt::Debug;
}

/// A foreign-to-Rust conversion, with structurally visible reverse-direction domains.
/// The first operation set must implement InputOps:
/// ```compile_fail
/// use std::convert::Infallible;
/// use prebindgen_declarations::{InputNode, OutputNode, InputOps, OutputOps};
/// #[derive(Clone, Debug)] struct Incoming;
/// #[derive(Clone, Debug)] struct Outgoing;
/// impl InputOps for Incoming {
///     type Leaf = (); type Unary = Infallible;
///     type Branch = (); type Arrow = Infallible;
/// }
/// impl OutputOps for Outgoing {
///     type Leaf = (); type Unary = Infallible;
///     type Branch = (); type Arrow = Infallible;
/// }
/// let _: InputNode<Outgoing, Outgoing> = InputNode::None;
/// ```
/// The second operation set must implement OutputOps:
/// ```compile_fail
/// use std::convert::Infallible;
/// use prebindgen_declarations::{InputNode, OutputNode, InputOps, OutputOps};
/// #[derive(Clone, Debug)] struct Incoming;
/// #[derive(Clone, Debug)] struct Outgoing;
/// impl InputOps for Incoming {
///     type Leaf = (); type Unary = Infallible;
///     type Branch = (); type Arrow = Infallible;
/// }
/// impl OutputOps for Outgoing {
///     type Leaf = (); type Unary = Infallible;
///     type Branch = (); type Arrow = Infallible;
/// }
/// let _: InputNode<Incoming, Incoming> = InputNode::None;
/// ```
#[derive(Clone, Debug)]
pub enum InputNode<
    InOps: InputOps,
    OutOps: OutputOps,
    InRequest = Infallible,
    OutRequest = Infallible,
> {
    /// Unit: no values cross the boundary.
    None,
    /// A choice to resolve before compilation. Resolved trees use Infallible.
    Request(InRequest),
    /// An adapter-defined terminal conversion.
    Leaf(InOps::Leaf),
    /// One conversion in the same direction.
    Unary {
        /// Interpretation of the child conversion.
        operation: InOps::Unary,
        /// Operand, in the enclosing conversion direction.
        child: Box<Self>,
    },
    /// Multiple conversions in the same direction.
    Branch {
        /// Interpretation of the child group.
        operation: InOps::Branch,
        /// Components in operation-defined order.
        children: Vec<Self>,
    },
    /// A callable whose arguments leave Rust and whose result enters Rust.
    Arrow {
        /// Callable representation and invocation policy.
        operation: InOps::Arrow,
        /// Argument conversion, in the opposite direction.
        domain: Box<OutputNode<InOps, OutOps, InRequest, OutRequest>>,
        /// Result conversion, in the enclosing direction.
        codomain: Box<Self>,
    },
}

/// A Rust-to-foreign conversion, with structurally visible reverse-direction domains.
/// The first operation set must implement InputOps:
/// ```compile_fail
/// use std::convert::Infallible;
/// use prebindgen_declarations::{InputNode, OutputNode, InputOps, OutputOps};
/// #[derive(Clone, Debug)] struct Incoming;
/// #[derive(Clone, Debug)] struct Outgoing;
/// impl InputOps for Incoming {
///     type Leaf = (); type Unary = Infallible;
///     type Branch = (); type Arrow = Infallible;
/// }
/// impl OutputOps for Outgoing {
///     type Leaf = (); type Unary = Infallible;
///     type Branch = (); type Arrow = Infallible;
/// }
/// let _: OutputNode<Outgoing, Outgoing> = OutputNode::None;
/// ```
/// The second operation set must implement OutputOps:
/// ```compile_fail
/// use std::convert::Infallible;
/// use prebindgen_declarations::{InputNode, OutputNode, InputOps, OutputOps};
/// #[derive(Clone, Debug)] struct Incoming;
/// #[derive(Clone, Debug)] struct Outgoing;
/// impl InputOps for Incoming {
///     type Leaf = (); type Unary = Infallible;
///     type Branch = (); type Arrow = Infallible;
/// }
/// impl OutputOps for Outgoing {
///     type Leaf = (); type Unary = Infallible;
///     type Branch = (); type Arrow = Infallible;
/// }
/// let _: OutputNode<Incoming, Incoming> = OutputNode::None;
/// ```
#[derive(Clone, Debug)]
pub enum OutputNode<
    InOps: InputOps,
    OutOps: OutputOps,
    InRequest = Infallible,
    OutRequest = Infallible,
> {
    /// Unit: no values cross the boundary.
    None,
    /// A choice to resolve before compilation. Resolved trees use Infallible.
    Request(OutRequest),
    /// An adapter-defined terminal conversion.
    Leaf(OutOps::Leaf),
    /// One conversion in the same direction.
    Unary {
        /// Interpretation of the child conversion.
        operation: OutOps::Unary,
        /// Operand, in the enclosing conversion direction.
        child: Box<Self>,
    },
    /// Multiple conversions in the same direction.
    Branch {
        /// Interpretation of the child group.
        operation: OutOps::Branch,
        /// Components in operation-defined order.
        children: Vec<Self>,
    },
    /// A callable whose arguments enter Rust and whose result leaves Rust.
    Arrow {
        /// Callable representation and invocation policy.
        operation: OutOps::Arrow,
        /// Argument conversion, in the opposite direction.
        domain: Box<InputNode<InOps, OutOps, InRequest, OutRequest>>,
        /// Result conversion, in the enclosing direction.
        codomain: Box<Self>,
    },
}

impl<InOps: InputOps, OutOps: OutputOps, InRequest, OutRequest>
    InputNode<InOps, OutOps, InRequest, OutRequest>
{
    /// Transform requests throughout the tree, including reverse-direction domains.
    pub fn map_requests<InRequest2, OutRequest2>(
        self,
        input: &mut impl FnMut(InRequest) -> InRequest2,
        output: &mut impl FnMut(OutRequest) -> OutRequest2,
    ) -> InputNode<InOps, OutOps, InRequest2, OutRequest2> {
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

impl<InOps: InputOps, OutOps: OutputOps, InRequest, OutRequest>
    OutputNode<InOps, OutOps, InRequest, OutRequest>
{
    /// Transform requests throughout the tree, including reverse-direction domains.
    pub fn map_requests<InRequest2, OutRequest2>(
        self,
        input: &mut impl FnMut(InRequest) -> InRequest2,
        output: &mut impl FnMut(OutRequest) -> OutRequest2,
    ) -> OutputNode<InOps, OutOps, InRequest2, OutRequest2> {
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

impl<I: InputOps, O: OutputOps> InputNode<I, O> {
    /// Embed a request-free tree in a preparation tree without changing its operations.
    pub fn into_preparation<IR, OR>(self) -> InputNode<I, O, IR, OR> {
        self.map_requests(&mut |never| match never {}, &mut |never| match never {})
    }
}
impl<I: InputOps, O: OutputOps> OutputNode<I, O> {
    /// Embed a request-free tree in a preparation tree without changing its operations.
    pub fn into_preparation<IR, OR>(self) -> OutputNode<I, O, IR, OR> {
        self.map_requests(&mut |never| match never {}, &mut |never| match never {})
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[derive(Clone, Debug)]
    struct Incoming;
    #[derive(Clone, Debug)]
    struct Outgoing;
    impl InputOps for Incoming {
        type Leaf = u8;
        type Unary = bool;
        type Branch = usize;
        type Arrow = &'static str;
    }
    impl OutputOps for Outgoing {
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
