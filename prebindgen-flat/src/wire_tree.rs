//! Logical values exchanged by a conversion; transport belongs to its generator.
use crate::{Result, Wire, WireKind};

/// Logical structure of one value. Each node kind has its own metadata type;
/// adapters choose that metadata and interpret it when emitting bindings.
///
/// Metadata belongs to its specific parent variant:
///
/// ```compile_fail,E0308
/// use prebindgen_flat::WireNode;
/// struct OptionalInfo;
/// struct SequenceInfo;
/// let node: WireNode<syn::Type, OptionalInfo, SequenceInfo> = WireNode::Optional {
///     element: Box::new(WireNode::default()), metadata: SequenceInfo,
/// };
/// ```
/// Output completion channels cannot be nested in a value tree:
///
/// ```compile_fail,E0308
/// use prebindgen_flat::{Outcomes, WireNode};
/// let node: WireNode<syn::Type> = WireNode::Sequence {
///     element: Box::new(Outcomes { success: WireNode::<syn::Type>::default(),
///         source_error: None, binding_failure: None }), metadata: (),
/// };
/// ```
#[derive(Clone, Debug)]
pub enum WireNode<W: WireKind, O = (), S = (), G = (), A = ()> {
    /// A terminal bundle of independent values. An empty bundle carries no values.
    Wire(Vec<Wire<W>>),
    /// Zero or one occurrence of the payload node.
    Optional {
        /// Description of one present payload.
        element: Box<WireNode<W, O, S, G, A>>,
        /// Adapter-specific information for this optional value.
        metadata: O,
    },
    /// Zero or more occurrences of the element node.
    Sequence {
        /// Description of one sequence element.
        element: Box<WireNode<W, O, S, G, A>>,
        /// Adapter-specific information for this sequence.
        metadata: S,
    },
    /// Related child values, such as fields or ordered conversion components.
    Group {
        /// Child descriptions in declaration order; may be empty.
        elements: Vec<WireNode<W, O, S, G, A>>,
        /// Adapter-specific information about this group.
        metadata: G,
    },
    /// Exactly one complete alternative is selected. A payload-free choice uses
    /// an empty bundle or group. At least one alternative must be declared.
    Alternatives {
        /// Mutually exclusive descriptions in declaration order.
        elements: Vec<WireNode<W, O, S, G, A>>,
        /// Adapter-specific information for selecting an alternative.
        metadata: A,
    },
}
impl<W: WireKind, O, S, G, A> Default for WireNode<W, O, S, G, A> {
    fn default() -> Self {
        Self::Wire(vec![])
    }
}

/// Completion channels of an output conversion. These surround value trees;
/// they cannot appear inside an optional, sequence, group or alternative.
#[derive(Clone, Debug)]
pub struct Outcomes<N> {
    /// Description delivered on successful completion, including unit.
    pub success: N,
    /// Projected `Err(E)`. `Some` with an empty bundle or group is a payload-free error.
    pub source_error: Option<N>,
    /// Conversion or invocation failure, separate from the source's `Err(E)`.
    pub binding_failure: Option<N>,
}
impl<N: Default> Default for Outcomes<N> {
    fn default() -> Self {
        Self {
            success: N::default(),
            source_error: None,
            binding_failure: None,
        }
    }
}

/// Visit each logical wire definition once. Repetitions are not expanded;
/// adapter metadata is opaque. Roots may be filtered or chained explicitly.
pub fn wire_definitions<'a, W: WireKind + 'a, O: 'a, S: 'a, G: 'a, A: 'a>(
    nodes: impl IntoIterator<Item = &'a WireNode<W, O, S, G, A>>,
) -> impl std::iter::FusedIterator<Item = &'a Wire<W>> {
    enum Frame<'a, W: WireKind, O, S, G, A> {
        Nodes(std::slice::Iter<'a, WireNode<W, O, S, G, A>>),
        Wires(std::slice::Iter<'a, Wire<W>>),
    }
    let mut roots = nodes.into_iter().fuse();
    let mut stack: Vec<Frame<'a, W, O, S, G, A>> = vec![];
    std::iter::from_fn(move || loop {
        let node = match stack.last_mut() {
            Some(Frame::Wires(wires)) => {
                if let Some(wire) = wires.next() {
                    return Some(wire);
                }
                stack.pop();
                continue;
            }
            Some(Frame::Nodes(nodes)) => {
                if let Some(node) = nodes.next() {
                    node
                } else {
                    stack.pop();
                    continue;
                }
            }
            None => roots.next()?,
        };
        match node {
            WireNode::Wire(wires) => stack.push(Frame::Wires(wires.iter())),
            WireNode::Optional { element, .. } => {
                stack.push(Frame::Nodes(std::slice::from_ref(element.as_ref()).iter()))
            }
            WireNode::Sequence { element, .. } => {
                stack.push(Frame::Nodes(std::slice::from_ref(element.as_ref()).iter()))
            }
            WireNode::Group { elements, .. } | WireNode::Alternatives { elements, .. } => {
                stack.push(Frame::Nodes(elements.iter()))
            }
        }
    })
    .fuse()
}

/// Mutate logical wire definitions without cloning metadata or changing nodes.
pub fn wire_definitions_mut<'a, W: WireKind + 'a, O: 'a, S: 'a, G: 'a, A: 'a>(
    nodes: impl IntoIterator<Item = &'a mut WireNode<W, O, S, G, A>>,
) -> impl std::iter::FusedIterator<Item = &'a mut Wire<W>> {
    enum Frame<'a, W: WireKind, O, S, G, A> {
        Nodes(std::slice::IterMut<'a, WireNode<W, O, S, G, A>>),
        Wires(std::slice::IterMut<'a, Wire<W>>),
    }
    let mut roots = nodes.into_iter().fuse();
    let mut stack: Vec<Frame<'a, W, O, S, G, A>> = vec![];
    std::iter::from_fn(move || loop {
        let node = match stack.last_mut() {
            Some(Frame::Wires(wires)) => {
                if let Some(wire) = wires.next() {
                    return Some(wire);
                }
                stack.pop();
                continue;
            }
            Some(Frame::Nodes(nodes)) => {
                if let Some(node) = nodes.next() {
                    node
                } else {
                    stack.pop();
                    continue;
                }
            }
            None => roots.next()?,
        };
        match node {
            WireNode::Wire(wires) => stack.push(Frame::Wires(wires.iter_mut())),
            WireNode::Optional { element, .. } => stack.push(Frame::Nodes(
                std::slice::from_mut(element.as_mut()).iter_mut(),
            )),
            WireNode::Sequence { element, .. } => stack.push(Frame::Nodes(
                std::slice::from_mut(element.as_mut()).iter_mut(),
            )),
            WireNode::Group { elements, .. } | WireNode::Alternatives { elements, .. } => {
                stack.push(Frame::Nodes(elements.iter_mut()))
            }
        }
    })
    .fuse()
}

/// Address relative to the selected roots. The first component selects a root;
/// following components select a child or a terminal wire within its bundle.
/// Optional/sequence payloads use child position zero. Outcomes are not part of
/// a value path: choose the completion channel before traversing its tree.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct WirePath(pub Vec<usize>);
/// A borrowed wire definition and its address within the selected trees.
pub struct WireEntry<'a, W: WireKind> {
    /// Structural address, rebuilt after edits or changing the selected roots.
    pub path: WirePath,
    /// Borrowed terminal value.
    pub wire: &'a Wire<W>,
}
/// Visit wire definitions with paths, including their terminal bundle positions.
pub fn wire_definitions_with_paths<'a, W: WireKind + 'a, O: 'a, S: 'a, G: 'a, A: 'a>(
    nodes: impl IntoIterator<Item = &'a WireNode<W, O, S, G, A>>,
) -> impl std::iter::FusedIterator<Item = WireEntry<'a, W>> {
    fn visit<'a, W: WireKind, O, S, G, A>(
        node: &'a WireNode<W, O, S, G, A>,
        path: &mut Vec<usize>,
        out: &mut Vec<WireEntry<'a, W>>,
    ) {
        match node {
            WireNode::Wire(wires) => {
                for (i, wire) in wires.iter().enumerate() {
                    path.push(i);
                    out.push(WireEntry {
                        path: WirePath(path.clone()),
                        wire,
                    });
                    path.pop();
                }
            }
            WireNode::Optional { element, .. } => {
                path.push(0);
                visit(element, path, out);
                path.pop();
            }
            WireNode::Sequence { element, .. } => {
                path.push(0);
                visit(element, path, out);
                path.pop();
            }
            WireNode::Group { elements, .. } | WireNode::Alternatives { elements, .. } => {
                for (i, child) in elements.iter().enumerate() {
                    path.push(i);
                    visit(child, path, out);
                    path.pop();
                }
            }
        }
    }
    let mut out = vec![];
    for (i, node) in nodes.into_iter().enumerate() {
        visit(node, &mut vec![i], &mut out);
    }
    out.into_iter()
}
/// Resolve a path produced for the same roots and selected completion channel.
pub fn wire_at_path<'a, W: WireKind + 'a, O: 'a, S: 'a, G: 'a, A: 'a>(
    nodes: impl IntoIterator<Item = &'a WireNode<W, O, S, G, A>>,
    path: &WirePath,
) -> Option<&'a Wire<W>> {
    fn get<'a, W: WireKind, O, S, G, A>(
        node: &'a WireNode<W, O, S, G, A>,
        path: &[usize],
    ) -> Option<&'a Wire<W>> {
        let (&index, rest) = path.split_first()?;
        match node {
            WireNode::Wire(wires) => rest.is_empty().then(|| wires.get(index)).flatten(),
            WireNode::Optional { element, .. } => {
                if index == 0 {
                    get(element, rest)
                } else {
                    None
                }
            }
            WireNode::Sequence { element, .. } => {
                if index == 0 {
                    get(element, rest)
                } else {
                    None
                }
            }
            WireNode::Group { elements, .. } | WireNode::Alternatives { elements, .. } => {
                get(elements.get(index)?, rest)
            }
        }
    }
    let (&root, rest) = path.0.split_first()?;
    get(nodes.into_iter().nth(root)?, rest)
}
/// Validate structural requirements independent of adapter metadata.
pub fn validate_wire_nodes<'a, W: WireKind + 'a, O: 'a, S: 'a, G: 'a, A: 'a>(
    nodes: impl IntoIterator<Item = &'a WireNode<W, O, S, G, A>>,
) -> Result<()> {
    fn check<W: WireKind, O, S, G, A>(node: &WireNode<W, O, S, G, A>) -> Result<()> {
        match node {
            WireNode::Wire(_) => Ok(()),
            WireNode::Optional { element, .. } => check(element),
            WireNode::Sequence { element, .. } => check(element),
            WireNode::Group { elements, .. } => {
                for child in elements {
                    check(child)?;
                }
                Ok(())
            }
            WireNode::Alternatives { elements, .. } => {
                if elements.is_empty() {
                    return Err("selection has no alternatives".into());
                }
                for child in elements {
                    check(child)?;
                }
                Ok(())
            }
        }
    }
    for node in nodes {
        check(node)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn wire(name: &str) -> Wire<syn::Type> {
        Wire {
            name: syn::parse_str(name).unwrap(),
            kind: syn::parse_quote!(i64),
        }
    }
    fn leaf(name: &str) -> WireNode<syn::Type> {
        WireNode::Wire(vec![wire(name)])
    }
    #[test]
    fn terminal_bundles_preserve_zero_one_and_multiple_values() {
        let nodes = [
            WireNode::Wire(vec![]),
            leaf("one"),
            WireNode::Wire(vec![wire("two"), wire("three")]),
        ];
        assert_eq!(
            wire_definitions(nodes.iter())
                .map(|w| w.name.to_string())
                .collect::<Vec<_>>(),
            ["one", "two", "three"]
        );
        let entries = wire_definitions_with_paths(nodes.iter()).collect::<Vec<_>>();
        assert_eq!(entries[2].path, WirePath(vec![2, 1]));
        for entry in entries {
            assert!(std::ptr::eq(
                entry.wire,
                wire_at_path(nodes.iter(), &entry.path).unwrap()
            ));
        }
    }
    #[test]
    fn paths_cover_group_choice_and_single_container_payloads() {
        let nodes = [WireNode::Group {
            metadata: (),
            elements: vec![WireNode::Alternatives {
                metadata: (),
                elements: vec![
                    WireNode::default(),
                    WireNode::Sequence {
                        metadata: (),
                        element: Box::new(WireNode::Optional {
                            metadata: (),
                            element: Box::new(leaf("value")),
                        }),
                    },
                ],
            }],
        }];
        let entry = wire_definitions_with_paths(nodes.iter()).next().unwrap();
        assert_eq!(entry.path, WirePath(vec![0, 0, 1, 0, 0, 0]));
        assert!(std::ptr::eq(
            entry.wire,
            wire_at_path(nodes.iter(), &entry.path).unwrap()
        ));
        assert!(wire_at_path(nodes.iter(), &WirePath(vec![0, 0, 99])).is_none());
        validate_wire_nodes(nodes.iter()).unwrap();
    }
    struct Kind(u32);
    impl WireKind for Kind {
        fn rust_type(&self) -> syn::Type {
            syn::parse_quote!(u32)
        }
    }
    struct OptionalInfo {
        ignored: Wire<Kind>,
    }
    struct SequenceInfo;
    struct GroupInfo;
    struct AlternativeInfo;
    fn typed_leaf(
        name: &str,
        value: u32,
    ) -> WireNode<Kind, OptionalInfo, SequenceInfo, GroupInfo, AlternativeInfo> {
        WireNode::Wire(vec![Wire {
            name: syn::parse_str(name).unwrap(),
            kind: Kind(value),
        }])
    }
    #[test]
    fn typed_metadata_and_mutable_traversal_need_no_clone() {
        let mut nodes = [
            typed_leaf("untouched", 1),
            WireNode::Optional {
                metadata: OptionalInfo {
                    ignored: Wire {
                        name: syn::parse_quote!(metadata),
                        kind: Kind(99),
                    },
                },
                element: Box::new(WireNode::Sequence {
                    metadata: SequenceInfo,
                    element: Box::new(typed_leaf("changed", 2)),
                }),
            },
        ];
        for wire in wire_definitions_mut(nodes.iter_mut().skip(1)) {
            wire.kind.0 += 10;
        }
        assert_eq!(
            wire_definitions(nodes.iter())
                .map(|w| w.kind.0)
                .collect::<Vec<_>>(),
            [1, 12]
        );
        let WireNode::Optional { metadata, .. } = &nodes[1] else {
            panic!()
        };
        assert_eq!(metadata.ignored.kind.0, 99);
        let selected = || nodes.iter().skip(1).chain(nodes.iter().take(1));
        for entry in wire_definitions_with_paths(selected()) {
            assert!(std::ptr::eq(
                entry.wire,
                wire_at_path(selected(), &entry.path).unwrap()
            ));
        }
    }
    #[test]
    fn wire_traversal_is_fused_when_roots_are_not() {
        let nodes = [leaf("first"), leaf("later")];
        let mut step = 0;
        let roots = std::iter::from_fn(|| {
            step += 1;
            match step {
                1 => Some(&nodes[0]),
                2 => None,
                3 => Some(&nodes[1]),
                _ => None,
            }
        });
        let mut iter = wire_definitions(roots);
        assert_eq!(iter.next().unwrap().name, "first");
        assert!(iter.next().is_none());
        assert!(iter.next().is_none());
    }
    #[test]
    fn empty_payloads_preserve_optional_and_group_structure() {
        let nodes: [WireNode<syn::Type>; 2] = [
            WireNode::Optional {
                metadata: (),
                element: Box::default(),
            },
            WireNode::Group {
                metadata: (),
                elements: vec![],
            },
        ];
        assert!(wire_definitions(nodes.iter()).next().is_none());
        assert!(matches!(nodes[0], WireNode::Optional { .. }));
        assert!(matches!(nodes[1], WireNode::Group { .. }));
        validate_wire_nodes(nodes.iter()).unwrap();
    }
    #[test]
    fn output_channels_are_selected_before_traversing_value_nodes() {
        for source in [false, true] {
            for binding in [false, true] {
                let mut outcomes = Outcomes {
                    success: leaf("ok"),
                    source_error: source.then(|| leaf("error")),
                    binding_failure: binding.then(|| leaf("failure")),
                };
                let success_path = wire_definitions_with_paths(std::iter::once(&outcomes.success))
                    .next()
                    .unwrap()
                    .path;
                assert_eq!(success_path, WirePath(vec![0, 0]));
                let count = wire_definitions(
                    std::iter::once(&outcomes.success)
                        .chain(outcomes.source_error.iter())
                        .chain(outcomes.binding_failure.iter()),
                )
                .count();
                assert_eq!(count, 1 + usize::from(source) + usize::from(binding));
                for wire in wire_definitions_mut(
                    std::iter::once(&mut outcomes.success)
                        .chain(outcomes.source_error.iter_mut())
                        .chain(outcomes.binding_failure.iter_mut()),
                ) {
                    wire.kind = syn::parse_quote!(u32);
                }
                assert_eq!(
                    crate::type_key(
                        &wire_at_path(std::iter::once(&outcomes.success), &success_path)
                            .unwrap()
                            .kind
                    ),
                    "u32"
                );
            }
        }
    }
    #[test]
    fn empty_error_payload_is_distinct_from_absence() {
        let outcomes = Outcomes {
            success: leaf("ok"),
            source_error: Some(WireNode::default()),
            binding_failure: None,
        };
        assert!(wire_definitions(outcomes.source_error.iter())
            .next()
            .is_none());
        assert!(outcomes.source_error.is_some());
        assert!(outcomes.binding_failure.is_none());
    }
    #[test]
    fn alternatives_require_a_choice_but_choices_can_be_empty() {
        let mut node: WireNode<syn::Type> = WireNode::Alternatives {
            metadata: (),
            elements: vec![],
        };
        assert!(validate_wire_nodes(std::iter::once(&node))
            .unwrap_err()
            .contains("no alternatives"));
        let WireNode::Alternatives { elements, .. } = &mut node else {
            panic!()
        };
        elements.push(WireNode::default());
        validate_wire_nodes(std::iter::once(&node)).unwrap();
    }
}
