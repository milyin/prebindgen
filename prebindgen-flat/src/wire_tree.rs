//! Logical values exchanged by a conversion; transport belongs to its generator.
use crate::{Result, Wire, WireKind};

/// Logical structure. Common nodes impose no ABI positions or delivery rules;
/// adapters may retain explicit encoding decisions in their opaque metadata.
#[derive(Clone, Debug)]
pub enum WireNode<W: WireKind, G = ()> {
    /// Definition of one typed value, wherever the generator chooses to pass it.
    Wire(Wire<W>),
    /// Zero or one occurrence of the element schema.
    Optional {
        /// Definition of one present payload.
        element: Vec<WireNode<W, G>>,
        /// Adapter-specific information about optional encoding.
        metadata: Option<G>,
    },
    /// Zero or more occurrences of the element schema.
    Sequence {
        /// Definition of one sequence element.
        element: Vec<WireNode<W, G>>,
        /// Adapter-specific information about sequence encoding.
        metadata: Option<G>,
    },
    /// Adapter-described relationships, such as records and named fields.
    Group {
        /// Adapter-specific information about the children.
        metadata: G,
        /// Logical child definitions in declared order.
        children: Vec<WireNode<W, G>>,
    },
    /// Exactly one alternative is selected; selector tags belong to the generator.
    /// Empty alternatives represent choices carrying no values.
    Alternatives {
        /// Adapter-specific selection information, without common placement rules.
        metadata: Option<G>,
        /// Mutually exclusive payload schemas in declaration order.
        alternatives: Vec<Vec<WireNode<W, G>>>,
    },
    /// One of the declared completion channels is selected. Success is always
    /// declared; each error channel is independently optional. Empty payloads retain
    /// their outcome identity; Result<(), E> is distinct from an ordinary unit return.
    Outcomes {
        /// Values delivered on successful completion; empty for a unit result.
        success: Vec<WireNode<W, G>>,
        /// Projected source error, present when this conversion routes `Err(E)`.
        /// `Some(vec![])` declares a payload-free error; `None` omits the channel.
        source_error: Option<Vec<WireNode<W, G>>>,
        /// Conversion or invocation failure, independent of a source `Err(E)`.
        /// `Some(vec![])` declares a payload-free failure; `None` omits the channel.
        binding_failure: Option<Vec<WireNode<W, G>>>,
    },
}

/// Visit logical wire definitions recursively. Each element/alternative schema
/// is visited once, including every outcome. Metadata is left to the adapter.
/// Accepts node iterators, including filtered or chained roots; vector iteration
/// itself always means immediate nodes.
pub fn wire_definitions<'a, W: WireKind + 'a, G: 'a>(
    nodes: impl IntoIterator<Item = &'a WireNode<W, G>>,
) -> impl std::iter::FusedIterator<Item = &'a Wire<W>> {
    WireDefinitions {
        roots: nodes.into_iter().fuse(),
        stack: vec![],
    }
}
struct WireDefinitions<'a, W: WireKind, G, I> {
    roots: I,
    stack: Vec<std::slice::Iter<'a, WireNode<W, G>>>,
}
impl<'a, W: WireKind, G, I: Iterator<Item = &'a WireNode<W, G>>> Iterator
    for WireDefinitions<'a, W, G, I>
{
    type Item = &'a Wire<W>;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let node = if let Some(children) = self.stack.last_mut() {
                if let Some(node) = children.next() {
                    node
                } else {
                    self.stack.pop();
                    continue;
                }
            } else {
                self.roots.next()?
            };
            match node {
                WireNode::Wire(w) => return Some(w),
                WireNode::Optional { element, .. } | WireNode::Sequence { element, .. } => {
                    self.stack.push(element.iter())
                }
                WireNode::Group { children, .. } => self.stack.push(children.iter()),
                WireNode::Alternatives { alternatives, .. } => {
                    for arm in alternatives.iter().rev() {
                        self.stack.push(arm.iter());
                    }
                }
                WireNode::Outcomes {
                    success,
                    source_error,
                    binding_failure,
                } => {
                    if let Some(nodes) = binding_failure {
                        self.stack.push(nodes.iter());
                    }
                    if let Some(nodes) = source_error {
                        self.stack.push(nodes.iter());
                    }
                    self.stack.push(success.iter());
                }
            }
        }
    }
}
impl<'a, W: WireKind, G, I: std::iter::FusedIterator<Item = &'a WireNode<W, G>>>
    std::iter::FusedIterator for WireDefinitions<'a, W, G, I>
{
}

/// Edit logical wire definitions recursively, including element and outcome
/// schemas. Node structure and adapter metadata are unchanged by traversal.
pub fn wire_definitions_mut<'a, W: WireKind + 'a, G: 'a>(
    nodes: impl IntoIterator<Item = &'a mut WireNode<W, G>>,
) -> impl std::iter::FusedIterator<Item = &'a mut Wire<W>> {
    WireDefinitionsMut {
        roots: nodes.into_iter().fuse(),
        stack: vec![],
    }
}
struct WireDefinitionsMut<'a, W: WireKind, G, I> {
    roots: I,
    stack: Vec<std::slice::IterMut<'a, WireNode<W, G>>>,
}
impl<'a, W: WireKind, G, I: Iterator<Item = &'a mut WireNode<W, G>>> Iterator
    for WireDefinitionsMut<'a, W, G, I>
{
    type Item = &'a mut Wire<W>;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let node = if let Some(children) = self.stack.last_mut() {
                if let Some(node) = children.next() {
                    node
                } else {
                    self.stack.pop();
                    continue;
                }
            } else {
                self.roots.next()?
            };
            match node {
                WireNode::Wire(w) => return Some(w),
                WireNode::Optional { element, .. } | WireNode::Sequence { element, .. } => {
                    self.stack.push(element.iter_mut())
                }
                WireNode::Group { children, .. } => self.stack.push(children.iter_mut()),
                WireNode::Alternatives { alternatives, .. } => {
                    for arm in alternatives.iter_mut().rev() {
                        self.stack.push(arm.iter_mut());
                    }
                }
                WireNode::Outcomes {
                    success,
                    source_error,
                    binding_failure,
                } => {
                    if let Some(nodes) = binding_failure {
                        self.stack.push(nodes.iter_mut());
                    }
                    if let Some(nodes) = source_error {
                        self.stack.push(nodes.iter_mut());
                    }
                    self.stack.push(success.iter_mut());
                }
            }
        }
    }
}
impl<'a, W: WireKind, G, I: std::iter::FusedIterator<Item = &'a mut WireNode<W, G>>>
    std::iter::FusedIterator for WireDefinitionsMut<'a, W, G, I>
{
}

/// Address of a definition relative to the supplied roots. Includes element and
/// alternative boundaries. Outcome branch positions are fixed: 0 for success,
/// 1 for source error and 2 for binding failure, including when a channel is absent.
/// Rebuild paths after structural edits or choosing different root iterators.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct WirePath(pub Vec<usize>);
/// A logical definition with its structural address.
pub struct WireEntry<'a, W: WireKind> {
    /// Address relative to the supplied roots.
    pub path: WirePath,
    /// Borrowed leaf definition.
    pub wire: &'a Wire<W>,
}
/// Visit logical wire definitions with structural paths. Paths are relative to
/// this root iterator, so filtering roots also changes their root indices.
pub fn wire_definitions_with_paths<'a, W: WireKind + 'a, G: 'a>(
    nodes: impl IntoIterator<Item = &'a WireNode<W, G>>,
) -> impl std::iter::FusedIterator<Item = WireEntry<'a, W>> {
    fn visit<'a, W: WireKind, G>(
        node: &'a WireNode<W, G>,
        path: &mut Vec<usize>,
        out: &mut Vec<WireEntry<'a, W>>,
    ) {
        fn children<'a, W: WireKind, G>(
            nodes: &'a [WireNode<W, G>],
            path: &mut Vec<usize>,
            out: &mut Vec<WireEntry<'a, W>>,
        ) {
            for (i, node) in nodes.iter().enumerate() {
                path.push(i);
                visit(node, path, out);
                path.pop();
            }
        }
        match node {
            WireNode::Wire(wire) => out.push(WireEntry {
                path: WirePath(path.clone()),
                wire,
            }),
            WireNode::Optional { element, .. } | WireNode::Sequence { element, .. } => {
                children(element, path, out)
            }
            WireNode::Group {
                children: nodes, ..
            } => children(nodes, path, out),
            WireNode::Alternatives { alternatives, .. } => {
                for (i, arm) in alternatives.iter().enumerate() {
                    path.push(i);
                    children(arm, path, out);
                    path.pop();
                }
            }
            WireNode::Outcomes {
                success,
                source_error,
                binding_failure,
            } => {
                for (branch, nodes) in [
                    Some(success.as_slice()),
                    source_error.as_deref(),
                    binding_failure.as_deref(),
                ]
                .into_iter()
                .enumerate()
                {
                    if let Some(nodes) = nodes {
                        path.push(branch);
                        children(nodes, path, out);
                        path.pop();
                    }
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
/// Resolve a definition by a path produced for the same root selection.
pub fn wire_at_path<'a, W: WireKind + 'a, G: 'a>(
    nodes: impl IntoIterator<Item = &'a WireNode<W, G>>,
    path: &WirePath,
) -> Option<&'a Wire<W>> {
    fn get<'a, W: WireKind, G>(node: &'a WireNode<W, G>, rest: &[usize]) -> Option<&'a Wire<W>> {
        fn child<'a, W: WireKind, G>(
            nodes: &'a [WireNode<W, G>],
            path: &[usize],
        ) -> Option<&'a Wire<W>> {
            let (&index, rest) = path.split_first()?;
            get(nodes.get(index)?, rest)
        }
        match node {
            WireNode::Wire(w) => rest.is_empty().then_some(w),
            WireNode::Optional { element, .. } | WireNode::Sequence { element, .. } => {
                child(element, rest)
            }
            WireNode::Group { children, .. } => child(children, rest),
            WireNode::Alternatives { alternatives, .. } => {
                let (&arm, rest) = rest.split_first()?;
                child(alternatives.get(arm)?, rest)
            }
            WireNode::Outcomes {
                success,
                source_error,
                binding_failure,
            } => {
                let (&branch, rest) = rest.split_first()?;
                let nodes = match branch {
                    0 => success.as_slice(),
                    1 => source_error.as_deref()?,
                    2 => binding_failure.as_deref()?,
                    _ => return None,
                };
                child(nodes, rest)
            }
        }
    }
    let (&index, rest) = path.0.split_first()?;
    get(nodes.into_iter().nth(index)?, rest)
}
/// Check logical branch structure. An `Outcomes` node must be the sole root;
/// its payloads cannot contain further `Outcomes` nodes. Named fields guarantee
/// a success schema and at most one schema for each error channel. Names may
/// repeat across contexts.
pub fn validate_wire_nodes<'a, W: WireKind + 'a, G: 'a>(
    nodes: impl IntoIterator<Item = &'a WireNode<W, G>>,
) -> Result<()> {
    fn check<W: WireKind, G>(node: &WireNode<W, G>, allow_outcomes: bool) -> Result<()> {
        fn nested<W: WireKind, G>(nodes: &[WireNode<W, G>]) -> Result<()> {
            for node in nodes {
                check(node, false)?;
            }
            Ok(())
        }
        match node {
            WireNode::Wire(_) => {}
            WireNode::Optional { element, .. } | WireNode::Sequence { element, .. } => {
                nested(element)?
            }
            WireNode::Group { children, .. } => nested(children)?,
            WireNode::Alternatives { alternatives, .. } => {
                if alternatives.is_empty() {
                    return Err("selection has no alternatives".into());
                }
                for arm in alternatives {
                    nested(arm)?;
                }
            }
            WireNode::Outcomes {
                success,
                source_error,
                binding_failure,
            } => {
                if !allow_outcomes {
                    return Err("outcomes must be the sole root node".into());
                }
                nested(success)?;
                if let Some(nodes) = source_error {
                    nested(nodes)?;
                }
                if let Some(nodes) = binding_failure {
                    nested(nodes)?;
                }
            }
        }
        Ok(())
    }
    let mut roots = nodes.into_iter().enumerate().peekable();
    while let Some((i, node)) = roots.next() {
        check(node, i == 0 && roots.peek().is_none())?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn leaf(name: &str) -> WireNode<syn::Type> {
        WireNode::Wire(Wire {
            name: syn::parse_str(name).unwrap(),
            kind: syn::parse_quote!(i64),
        })
    }
    #[test]
    fn vector_iteration_and_indexing_access_immediate_nodes() {
        let mut nodes = vec![WireNode::Group {
            metadata: (),
            children: vec![leaf("first"), leaf("second")],
        }];
        assert_eq!(nodes.len(), 1);
        assert!(matches!(nodes[0], WireNode::Group { .. }));
        assert_eq!(
            wire_definitions(nodes.iter())
                .map(|w| w.name.to_string())
                .collect::<Vec<_>>(),
            ["first", "second"]
        );
        nodes.insert(0, leaf("prefix"));
        assert_eq!(nodes.len(), 2);
        assert_eq!(wire_definitions(nodes.iter()).count(), 3);
    }
    #[test]
    fn paths_cover_elements_alternatives_and_stable_outcome_channels() {
        let sequence = WireNode::Sequence {
            element: vec![WireNode::Optional {
                element: vec![leaf("value")],
                metadata: None,
            }],
            metadata: None,
        };
        let mut nodes = vec![WireNode::Alternatives {
            metadata: None,
            alternatives: vec![vec![], vec![sequence, leaf("other")]],
        }];
        let entry = wire_definitions_with_paths(nodes.iter()).next().unwrap();
        assert_eq!(entry.path, WirePath(vec![0, 1, 0, 0, 0]));
        assert!(std::ptr::eq(
            entry.wire,
            wire_at_path(nodes.iter(), &entry.path).unwrap()
        ));
        nodes = vec![WireNode::Outcomes {
            success: nodes,
            source_error: None,
            binding_failure: Some(vec![leaf("message")]),
        }];
        let binding_failure_path = wire_definitions_with_paths(nodes.iter())
            .last()
            .unwrap()
            .path;
        let [WireNode::Outcomes { source_error, .. }] = nodes.as_mut_slice() else {
            panic!()
        };
        *source_error = Some(vec![leaf("value")]);
        assert_eq!(
            wire_at_path(nodes.iter(), &binding_failure_path)
                .unwrap()
                .name,
            "message"
        );
        let entries = wire_definitions_with_paths(nodes.iter()).collect::<Vec<_>>();
        assert_eq!(entries.len(), 4);
        assert_ne!(entries[0].path, entries[2].path);
        for entry in entries {
            assert!(std::ptr::eq(
                entry.wire,
                wire_at_path(nodes.iter(), &entry.path).unwrap()
            ));
        }
        assert!(wire_at_path(nodes.iter(), &WirePath(vec![99])).is_none());
        validate_wire_nodes(nodes.iter()).unwrap();
    }
    struct Kind(u32);
    impl WireKind for Kind {
        fn rust_type(&self) -> syn::Type {
            syn::parse_quote!(u32)
        }
    }
    struct Metadata {
        wire: Wire<Kind>,
    }
    fn non_clone_leaf(name: &str, value: u32) -> WireNode<Kind, Metadata> {
        WireNode::Wire(Wire {
            name: syn::parse_str(name).unwrap(),
            kind: Kind(value),
        })
    }
    #[test]
    fn filtered_and_chained_traversals_borrow_without_cloning_metadata() {
        let mut nodes = [
            non_clone_leaf("untouched", 1),
            WireNode::Group {
                metadata: Metadata {
                    wire: Wire {
                        name: syn::parse_quote!(metadata_value),
                        kind: Kind(99),
                    },
                },
                children: vec![WireNode::Sequence {
                    element: vec![non_clone_leaf("changed", 2)],
                    metadata: None,
                }],
            },
        ];
        for wire in wire_definitions_mut(nodes.iter_mut().skip(1)) {
            wire.kind.0 += 10;
        }
        let definitions = wire_definitions(nodes.iter()).collect::<Vec<_>>();
        assert_eq!(definitions[0].kind.0, 1);
        assert_eq!(definitions[1].kind.0, 12);
        let WireNode::Group { metadata, .. } = &nodes[1] else {
            panic!()
        };
        assert_eq!(metadata.wire.kind.0, 99);
        let selected = || nodes.iter().skip(1).chain(nodes.iter().take(1));
        assert_eq!(
            wire_definitions(selected())
                .map(|w| w.name.to_string())
                .collect::<Vec<_>>(),
            ["changed", "untouched"]
        );
        for entry in wire_definitions_with_paths(selected()) {
            assert!(std::ptr::eq(
                entry.wire,
                wire_at_path(selected(), &entry.path).unwrap()
            ));
        }
    }
    #[test]
    fn wire_traversal_is_fused_even_when_root_iterator_is_not() {
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
        let mut definitions = wire_definitions(roots);
        assert_eq!(definitions.next().unwrap().name, "first");
        assert!(definitions.next().is_none());
        assert!(definitions.next().is_none());
    }
    #[test]
    fn empty_optional_and_completion_payloads_keep_their_structure() {
        let mut nodes: Vec<WireNode<syn::Type>> = vec![WireNode::Outcomes {
            success: vec![WireNode::Optional {
                element: vec![],
                metadata: None,
            }],
            source_error: Some(vec![]),
            binding_failure: None,
        }];
        assert!(wire_definitions(nodes.iter()).next().is_none());
        let [WireNode::Outcomes {
            success,
            source_error,
            binding_failure,
        }] = nodes.as_mut_slice()
        else {
            panic!()
        };
        assert_eq!(success.len(), 1);
        assert!(matches!(&success[0], WireNode::Optional { .. }));
        assert!(source_error.as_ref().unwrap().is_empty());
        assert!(binding_failure.is_none());
        *success = vec![leaf("new_value")];
        assert!(source_error.is_some());
        assert_eq!(
            wire_definitions(nodes.iter()).next().unwrap().name,
            "new_value"
        );
        validate_wire_nodes(nodes.iter()).unwrap();
    }
    #[test]
    fn named_outcomes_traverse_and_mutate_all_declared_branches_without_cloning() {
        for has_source_error in [false, true] {
            for has_binding_failure in [false, true] {
                let mut nodes = [WireNode::Outcomes {
                    success: vec![non_clone_leaf("same", 1)],
                    source_error: has_source_error.then(|| vec![non_clone_leaf("same", 2)]),
                    binding_failure: has_binding_failure.then(|| vec![non_clone_leaf("same", 3)]),
                }];
                validate_wire_nodes(nodes.iter()).unwrap();
                let expected = std::iter::once(1)
                    .chain(has_source_error.then_some(2))
                    .chain(has_binding_failure.then_some(3))
                    .collect::<Vec<_>>();
                assert_eq!(
                    wire_definitions(nodes.iter())
                        .map(|w| w.kind.0)
                        .collect::<Vec<_>>(),
                    expected
                );
                for entry in wire_definitions_with_paths(nodes.iter()) {
                    // Absent channels keep their positions reserved in paths.
                    assert_eq!(
                        entry.path,
                        WirePath(vec![0, (entry.wire.kind.0 - 1) as usize, 0])
                    );
                    assert!(std::ptr::eq(
                        entry.wire,
                        wire_at_path(nodes.iter(), &entry.path).unwrap()
                    ));
                }
                assert_eq!(
                    wire_at_path(nodes.iter(), &WirePath(vec![0, 1, 0])).is_some(),
                    has_source_error
                );
                assert_eq!(
                    wire_at_path(nodes.iter(), &WirePath(vec![0, 2, 0])).is_some(),
                    has_binding_failure
                );
                assert!(wire_at_path(nodes.iter(), &WirePath(vec![0, 3, 0])).is_none());
                for wire in wire_definitions_mut(nodes.iter_mut()) {
                    wire.kind.0 += 10;
                }
                assert_eq!(
                    wire_definitions(nodes.iter())
                        .map(|w| w.kind.0)
                        .collect::<Vec<_>>(),
                    expected.into_iter().map(|v| v + 10).collect::<Vec<_>>()
                );
            }
        }
    }
    #[test]
    fn validation_checks_each_named_outcome_payload() {
        for branch in 0..3 {
            let mut success = vec![];
            let mut source_error = None;
            let mut binding_failure = None;
            let invalid: Vec<WireNode<syn::Type>> = vec![WireNode::Alternatives {
                metadata: None,
                alternatives: vec![],
            }];
            match branch {
                0 => success = invalid,
                1 => source_error = Some(invalid),
                2 => binding_failure = Some(invalid),
                _ => unreachable!(),
            }
            let nodes = [WireNode::Outcomes {
                success,
                source_error,
                binding_failure,
            }];
            assert!(validate_wire_nodes(nodes.iter())
                .unwrap_err()
                .contains("no alternatives"));
        }
    }
    #[test]
    fn validation_rejects_empty_choices_and_invalid_completion_roots() {
        let mut nodes: Vec<WireNode<syn::Type>> = vec![WireNode::Alternatives {
            metadata: None,
            alternatives: vec![],
        }];
        assert!(validate_wire_nodes(nodes.iter())
            .unwrap_err()
            .contains("no alternatives"));
        nodes = vec![WireNode::Outcomes {
            success: vec![leaf("value")],
            source_error: None,
            binding_failure: None,
        }];
        nodes.insert(0, leaf("extra"));
        assert!(validate_wire_nodes(nodes.iter())
            .unwrap_err()
            .contains("sole root"));
        validate_wire_nodes(nodes.iter().skip(1)).unwrap();
        let nested = [WireNode::Optional {
            element: nodes.into_iter().skip(1).collect(),
            metadata: None,
        }];
        assert!(validate_wire_nodes(nested.iter())
            .unwrap_err()
            .contains("sole root"));
    }
}
