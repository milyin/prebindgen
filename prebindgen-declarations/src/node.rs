//! Structural conversion trees. Operations describe meaning; children describe composition.

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
pub enum InputNode<InOps: InputOps, OutOps: OutputOps> {
    /// Unit: no values cross the boundary.
    None,
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
        domain: Box<OutputNode<InOps, OutOps>>,
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
pub enum OutputNode<InOps: InputOps, OutOps: OutputOps> {
    /// Unit: no values cross the boundary.
    None,
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
        domain: Box<InputNode<InOps, OutOps>>,
        /// Result conversion, in the enclosing direction.
        codomain: Box<Self>,
    },
}

impl<InOps: InputOps, OutOps: OutputOps> InputNode<InOps, OutOps> {
    /// Convert every metadata value using From, preserving all structural children.
    /// This can embed resolved adapter trees in a configuration specialization.
    pub fn map_into<NewInOps: InputOps, NewOutOps: OutputOps>(
        self,
    ) -> InputNode<NewInOps, NewOutOps>
    where
        NewInOps::Leaf: From<InOps::Leaf>,
        NewInOps::Unary: From<InOps::Unary>,
        NewInOps::Branch: From<InOps::Branch>,
        NewInOps::Arrow: From<InOps::Arrow>,
        NewOutOps::Leaf: From<OutOps::Leaf>,
        NewOutOps::Unary: From<OutOps::Unary>,
        NewOutOps::Branch: From<OutOps::Branch>,
        NewOutOps::Arrow: From<OutOps::Arrow>,
    {
        match self {
            Self::None => InputNode::None,
            Self::Leaf(operation) => InputNode::Leaf(operation.into()),
            Self::Unary { operation, child } => InputNode::Unary {
                operation: operation.into(),
                child: Box::new(child.map_into()),
            },
            Self::Branch {
                operation,
                children,
            } => InputNode::Branch {
                operation: operation.into(),
                children: children.into_iter().map(|child| child.map_into()).collect(),
            },
            Self::Arrow {
                operation,
                domain,
                codomain,
            } => InputNode::Arrow {
                operation: operation.into(),
                domain: Box::new(domain.map_into()),
                codomain: Box::new(codomain.map_into()),
            },
        }
    }
}

impl<InOps: InputOps, OutOps: OutputOps> OutputNode<InOps, OutOps> {
    /// Convert every metadata value using From, preserving all structural children.
    /// This can embed resolved adapter trees in a configuration specialization.
    pub fn map_into<NewInOps: InputOps, NewOutOps: OutputOps>(
        self,
    ) -> OutputNode<NewInOps, NewOutOps>
    where
        NewInOps::Leaf: From<InOps::Leaf>,
        NewInOps::Unary: From<InOps::Unary>,
        NewInOps::Branch: From<InOps::Branch>,
        NewInOps::Arrow: From<InOps::Arrow>,
        NewOutOps::Leaf: From<OutOps::Leaf>,
        NewOutOps::Unary: From<OutOps::Unary>,
        NewOutOps::Branch: From<OutOps::Branch>,
        NewOutOps::Arrow: From<OutOps::Arrow>,
    {
        match self {
            Self::None => OutputNode::None,
            Self::Leaf(operation) => OutputNode::Leaf(operation.into()),
            Self::Unary { operation, child } => OutputNode::Unary {
                operation: operation.into(),
                child: Box::new(child.map_into()),
            },
            Self::Branch {
                operation,
                children,
            } => OutputNode::Branch {
                operation: operation.into(),
                children: children.into_iter().map(|child| child.map_into()).collect(),
            },
            Self::Arrow {
                operation,
                domain,
                codomain,
            } => OutputNode::Arrow {
                operation: operation.into(),
                domain: Box::new(domain.map_into()),
                codomain: Box::new(codomain.map_into()),
            },
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
    #[derive(Clone, Debug)]
    struct ExpandedIncoming;
    #[derive(Clone, Debug)]
    struct ExpandedOutgoing;
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
    impl InputOps for ExpandedIncoming {
        type Leaf = u16;
        type Unary = bool;
        type Branch = usize;
        type Arrow = String;
    }
    impl OutputOps for ExpandedOutgoing {
        type Leaf = String;
        type Unary = usize;
        type Branch = bool;
        type Arrow = u16;
    }
    #[test]
    fn metadata_conversion_preserves_every_shape_and_both_arrow_directions() {
        let tree: InputNode<Incoming, Outgoing> = InputNode::Arrow {
            operation: "outer",
            domain: Box::new(OutputNode::Branch {
                operation: true,
                children: vec![
                    OutputNode::Unary {
                        operation: 9,
                        child: Box::new(OutputNode::Leaf("argument")),
                    },
                    OutputNode::Arrow {
                        operation: 7,
                        domain: Box::new(InputNode::Leaf(42)),
                        codomain: Box::new(OutputNode::None),
                    },
                ],
            }),
            codomain: Box::new(InputNode::Branch {
                operation: 2,
                children: vec![InputNode::Unary {
                    operation: true,
                    child: Box::new(InputNode::Leaf(5)),
                }],
            }),
        };
        let mapped: InputNode<ExpandedIncoming, ExpandedOutgoing> = tree.map_into();
        let InputNode::Arrow {
            operation,
            domain,
            codomain,
        } = mapped
        else {
            panic!("expected arrow")
        };
        assert_eq!(operation, "outer");
        let OutputNode::Branch {
            operation,
            children,
        } = *domain
        else {
            panic!("expected domain group")
        };
        assert!(operation);
        assert!(
            matches!(&children[0],OutputNode::Unary{operation:9,child} if matches!(child.as_ref(),OutputNode::Leaf(value) if value=="argument"))
        );
        assert!(
            matches!(&children[1],OutputNode::Arrow{operation:7,domain,codomain} if matches!(domain.as_ref(),InputNode::Leaf(42))&&matches!(codomain.as_ref(),OutputNode::None))
        );
        assert!(
            matches!(*codomain,InputNode::Branch{operation:2,children} if matches!(&children[0],InputNode::Unary{operation:true,child} if matches!(child.as_ref(),InputNode::Leaf(5))))
        );
    }
}
