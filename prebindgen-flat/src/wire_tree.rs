//! Logical values exchanged by a conversion; transport belongs to its generator.
use crate::{Result, Wire, WireKind};

/// One payload/element schema. Its leaves may occur zero, one or many times.
#[derive(Clone, Debug)]
pub struct WireContainer<W: WireKind, G> {
    /// Definition of one present payload or sequence element.
    pub element: Vec<WireNode<W, G>>,
    /// Optional adapter-specific information, without common placement rules.
    pub metadata: Option<G>,
}
/// Mutually exclusive completion channels, independent of delivery mechanism.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum OutcomeKind {
    /// Normal return value, including a successful Result payload.
    Success = 0,
    /// The projected E of a source Result<T, E>.
    SourceError = 1,
    /// Failure of conversion or invocation, distinct from the source's Err(E).
    BindingFailure = 2,
}
/// The logical payload of one completion channel.
#[derive(Clone, Debug)]
pub struct WireOutcome<W: WireKind, G> {
    /// Which completion channel selects this payload.
    pub kind: OutcomeKind,
    /// Fields exchanged when this outcome is selected; may be empty.
    pub nodes: Vec<WireNode<W, G>>,
}
/// Logical structure. Common nodes impose no ABI positions or delivery rules;
/// adapters may retain explicit encoding decisions in their opaque metadata.
#[derive(Clone, Debug)]
pub enum WireNode<W: WireKind, G = ()> {
    /// Definition of one typed value, wherever the generator chooses to pass it.
    Wire(Wire<W>),
    /// Zero or one occurrence of the element schema.
    Optional(WireContainer<W, G>),
    /// Zero or more occurrences of the element schema.
    Sequence(WireContainer<W, G>),
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
    /// Exactly one completion channel is selected. Empty payloads retain their
    /// outcome identity; Result<(), E> is distinct from an ordinary unit return.
    Outcomes {
        /// Available channels and their payload schemas.
        alternatives: Vec<WireOutcome<W, G>>,
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
                WireNode::Optional(c) | WireNode::Sequence(c) => self.stack.push(c.element.iter()),
                WireNode::Group { children, .. } => self.stack.push(children.iter()),
                WireNode::Alternatives { alternatives, .. } => {
                    for arm in alternatives.iter().rev() {
                        self.stack.push(arm.iter());
                    }
                }
                WireNode::Outcomes { alternatives } => {
                    for outcome in alternatives.iter().rev() {
                        self.stack.push(outcome.nodes.iter());
                    }
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
                WireNode::Optional(c) | WireNode::Sequence(c) => {
                    self.stack.push(c.element.iter_mut())
                }
                WireNode::Group { children, .. } => self.stack.push(children.iter_mut()),
                WireNode::Alternatives { alternatives, .. } => {
                    for arm in alternatives.iter_mut().rev() {
                        self.stack.push(arm.iter_mut());
                    }
                }
                WireNode::Outcomes { alternatives } => {
                    for outcome in alternatives.iter_mut().rev() {
                        self.stack.push(outcome.nodes.iter_mut());
                    }
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
/// alternative boundaries; outcome components use stable channel identities.
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
            WireNode::Optional(c) | WireNode::Sequence(c) => children(&c.element, path, out),
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
            WireNode::Outcomes { alternatives } => {
                for outcome in alternatives {
                    path.push(outcome.kind as usize);
                    children(&outcome.nodes, path, out);
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
            WireNode::Optional(c) | WireNode::Sequence(c) => child(&c.element, rest),
            WireNode::Group { children, .. } => child(children, rest),
            WireNode::Alternatives { alternatives, .. } => {
                let (&arm, rest) = rest.split_first()?;
                child(alternatives.get(arm)?, rest)
            }
            WireNode::Outcomes { alternatives } => {
                let (&channel, rest) = rest.split_first()?;
                child(
                    &alternatives
                        .iter()
                        .find(|o| o.kind as usize == channel)?
                        .nodes,
                    rest,
                )
            }
        }
    }
    let (&index, rest) = path.0.split_first()?;
    get(nodes.into_iter().nth(index)?, rest)
}
/// Select an explicit completion payload. A plain node slice implicitly means
/// Success. Malformed mixed outcome roots have no implicit Success payload.
pub fn outcome_nodes<W: WireKind, G>(
    nodes: &[WireNode<W, G>],
    kind: OutcomeKind,
) -> Option<&[WireNode<W, G>]> {
    if let [WireNode::Outcomes { alternatives }] = nodes {
        alternatives
            .iter()
            .find(|o| o.kind == kind)
            .map(|o| o.nodes.as_slice())
    } else {
        (kind == OutcomeKind::Success
            && !nodes.iter().any(|n| matches!(n, WireNode::Outcomes { .. })))
        .then_some(nodes)
    }
}
/// Add or replace a completion channel. An ordinary node vector becomes the
/// Success payload; empty payloads retain their completion identity.
pub fn set_outcome<W: WireKind, G>(
    nodes: &mut Vec<WireNode<W, G>>,
    kind: OutcomeKind,
    payload: Vec<WireNode<W, G>>,
) {
    if let [WireNode::Outcomes { alternatives }] = nodes.as_mut_slice() {
        if let Some(outcome) = alternatives.iter_mut().find(|o| o.kind == kind) {
            outcome.nodes = payload;
        } else {
            alternatives.push(WireOutcome {
                kind,
                nodes: payload,
            });
            alternatives.sort_by_key(|o| o.kind);
        }
    } else {
        let success = std::mem::take(nodes);
        let mut alternatives = vec![WireOutcome {
            kind: OutcomeKind::Success,
            nodes: success,
        }];
        if kind == OutcomeKind::Success {
            alternatives[0].nodes = payload;
        } else {
            alternatives.push(WireOutcome {
                kind,
                nodes: payload,
            });
        }
        nodes.push(WireNode::Outcomes { alternatives });
    }
}
/// Replace the successful result nodes while retaining declared error channels.
pub fn replace_success<W: WireKind, G>(
    nodes: &mut Vec<WireNode<W, G>>,
    payload: Vec<WireNode<W, G>>,
) {
    if matches!(nodes.as_slice(), [WireNode::Outcomes { .. }]) {
        set_outcome(nodes, OutcomeKind::Success, payload)
    } else {
        *nodes = payload;
    }
}
/// Check logical branch structure. Completion channels must be the sole root,
/// include Success, and have unique kinds. Names may repeat across contexts.
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
            WireNode::Optional(c) | WireNode::Sequence(c) => nested(&c.element)?,
            WireNode::Group { children, .. } => nested(children)?,
            WireNode::Alternatives { alternatives, .. } => {
                if alternatives.is_empty() {
                    return Err("selection has no alternatives".into());
                }
                for arm in alternatives {
                    nested(arm)?;
                }
            }
            WireNode::Outcomes { alternatives } => {
                if !allow_outcomes {
                    return Err("outcomes must be the sole root node".into());
                }
                let mut kinds = std::collections::BTreeSet::new();
                for outcome in alternatives {
                    if !kinds.insert(outcome.kind) {
                        return Err("duplicate outcome kind".into());
                    }
                }
                if !kinds.contains(&OutcomeKind::Success) {
                    return Err("outcomes require a Success channel".into());
                }
                for outcome in alternatives {
                    nested(&outcome.nodes)?;
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
        let sequence = WireNode::Sequence(WireContainer {
            element: vec![WireNode::Optional(WireContainer {
                element: vec![leaf("value")],
                metadata: None,
            })],
            metadata: None,
        });
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
        set_outcome(
            &mut nodes,
            OutcomeKind::BindingFailure,
            vec![leaf("message")],
        );
        let binding_failure_path = wire_definitions_with_paths(nodes.iter())
            .last()
            .unwrap()
            .path;
        set_outcome(&mut nodes, OutcomeKind::SourceError, vec![leaf("value")]);
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
                children: vec![WireNode::Sequence(WireContainer {
                    element: vec![non_clone_leaf("changed", 2)],
                    metadata: None,
                })],
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
        let mut nodes: Vec<WireNode<syn::Type>> = vec![WireNode::Optional(WireContainer {
            element: vec![],
            metadata: None,
        })];
        assert_eq!(nodes.len(), 1);
        assert!(wire_definitions(nodes.iter()).next().is_none());
        set_outcome(&mut nodes, OutcomeKind::SourceError, vec![]);
        assert_eq!(
            outcome_nodes(&nodes, OutcomeKind::Success).unwrap().len(),
            1
        );
        assert!(outcome_nodes(&nodes, OutcomeKind::SourceError)
            .unwrap()
            .is_empty());
        assert!(outcome_nodes(&nodes, OutcomeKind::BindingFailure).is_none());
        replace_success(&mut nodes, vec![leaf("new_value")]);
        assert_eq!(
            wire_definitions(outcome_nodes(&nodes, OutcomeKind::Success).unwrap())
                .next()
                .unwrap()
                .name,
            "new_value"
        );
        assert!(outcome_nodes(&nodes, OutcomeKind::SourceError).is_some());
        validate_wire_nodes(nodes.iter()).unwrap();
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
            alternatives: vec![WireOutcome {
                kind: OutcomeKind::SourceError,
                nodes: vec![],
            }],
        }];
        assert!(validate_wire_nodes(nodes.iter())
            .unwrap_err()
            .contains("Success"));
        nodes = vec![WireNode::Outcomes {
            alternatives: vec![
                WireOutcome {
                    kind: OutcomeKind::Success,
                    nodes: vec![],
                },
                WireOutcome {
                    kind: OutcomeKind::Success,
                    nodes: vec![],
                },
            ],
        }];
        assert!(validate_wire_nodes(nodes.iter())
            .unwrap_err()
            .contains("duplicate outcome"));
        nodes = vec![WireNode::Outcomes {
            alternatives: vec![WireOutcome {
                kind: OutcomeKind::Success,
                nodes: vec![leaf("value")],
            }],
        }];
        nodes.insert(0, leaf("extra"));
        assert!(validate_wire_nodes(nodes.iter())
            .unwrap_err()
            .contains("sole root"));
        assert!(outcome_nodes(&nodes, OutcomeKind::Success).is_none());
        validate_wire_nodes(nodes.iter().skip(1)).unwrap();
        let nested = [WireNode::Optional(WireContainer {
            element: nodes.into_iter().skip(1).collect(),
            metadata: None,
        })];
        assert!(validate_wire_nodes(nested.iter())
            .unwrap_err()
            .contains("sole root"));
    }
}
