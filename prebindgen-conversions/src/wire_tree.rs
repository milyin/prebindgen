//! Structural descriptions of conversion values; adapters interpret node metadata.
use crate::{Wire, WireKind};

/// A value's wire definitions arranged as terminal bundles and parent nodes.
///
/// Each node shape has independent adapter metadata: `Annotation`, `Chain`,
/// `Branch` and `Container`. An annotation preserves its
/// child's boundary. A chain adds controls to one child's boundary; a branch adds
/// controls to its ordered children's boundaries. A container exposes only its
/// own wires, and interprets its child internally. A leaf exposes its wire bundle.
///
/// Metadata describes how the adapter interprets children; it must not retain
/// wire descriptors or duplicate subtrees. Naming belongs to annotations, options
/// can use chains, fields/alternatives can use branches, and lists/packed frames
/// can use containers. All-definition traversal includes container internals;
/// boundary traversal stops at each container.
///
/// Annotations cannot declare control wires:
/// ```compile_fail
/// use prebindgen_conversions::WireNode;
/// let node: WireNode<syn::Type> = WireNode::Annotation {
///     controls: vec![], meta: (), child: Box::default(),
/// };
/// ```
///
/// Chain and branch metadata cannot be interchanged:
/// ```compile_fail,E0308
/// use prebindgen_conversions::WireNode;
/// struct ChainInfo;
/// struct BranchInfo;
/// let node: WireNode<syn::Type, ChainInfo, BranchInfo> = WireNode::Chain {
///     controls: vec![], child: Box::default(), meta: BranchInfo,
/// };
/// ```
/// Completion channels surround value trees and cannot be nested inside them:
/// ```compile_fail,E0308
///
/// use prebindgen_conversions::{Outcomes, WireNode};
/// let node: WireNode<syn::Type> = WireNode::Chain {
///     controls: vec![], child: Box::new(Outcomes::<WireNode<syn::Type>>::default()), meta: (),
/// };
/// ```
#[derive(Clone, Debug)]
pub enum WireNode<W: WireKind, Chain = (), Branch = (), Annotation = (), Container = ()> {
    /// A terminal bundle of values; an empty bundle carries no wires.
    Leaf(Vec<Wire<W>>),
    /// Describe one subtree without adding wires or changing its boundary.
    Annotation {
        /// Adapter description such as a Rust field or Kotlin parameter name.
        meta: Annotation,
        /// The annotated representation, unchanged at this boundary.
        child: Box<WireNode<W, Chain, Branch, Annotation, Container>>,
    },
    /// Expose only this node's wires at its enclosing boundary.
    /// The child describes an internal representation interpreted by the container,
    /// such as one repeated list element or the contents of a packed frame.
    Container {
        /// Adapter policy for packing, repeating or delivering the child.
        meta: Container,
        /// Internal representation; its wires do not appear at the enclosing boundary.
        child: Box<WireNode<W, Chain, Branch, Annotation, Container>>,
        /// The complete representation exposed at the enclosing boundary.
        wires: Vec<Wire<W>>,
    },
    /// Add supporting controls while interpreting one child at the same boundary.
    Chain {
        /// Adapter policy or annotations for this parent.
        meta: Chain,
        /// Exactly one child description; runtime occurrence count is adapter-defined.
        child: Box<WireNode<W, Chain, Branch, Annotation, Container>>,
        /// Supporting wire definitions outside the child payload.
        controls: Vec<Wire<W>>,
    },
    /// Ordered child descriptions, interpreted according to adapter metadata.
    Branch {
        /// Adapter policy or annotations for this parent.
        meta: Branch,
        /// Ordered child descriptions. Metadata determines whether an empty list is valid.
        children: Vec<WireNode<W, Chain, Branch, Annotation, Container>>,
        /// Supporting wire definitions outside the child payloads.
        controls: Vec<Wire<W>>,
    },
}
impl<W: WireKind, C, B, A, T> Default for WireNode<W, C, B, A, T> {
    fn default() -> Self {
        Self::Leaf(vec![])
    }
}

/// Completion channels surrounding output value trees; never children of a value.
#[derive(Clone, Debug)]
pub struct Outcomes<N> {
    /// Successful completion, including unit.
    pub success: N,
    /// Projected `Err(E)`; an empty value tree can represent a payload-free error.
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

/// Visit all wire definitions, controls before children, without interpreting metadata.
/// Each child description is visited once, regardless of runtime repetition or selection.
pub fn wire_definitions<'a, W: WireKind + 'a, C: 'a, B: 'a, A: 'a, T: 'a>(
    nodes: impl IntoIterator<Item = &'a WireNode<W, C, B, A, T>>,
) -> impl std::iter::FusedIterator<Item = &'a Wire<W>> {
    definitions(nodes, true)
}
/// Visit only wires exposed at this boundary, stopping at container wires.
/// Annotations are transparent; chain/branch controls precede their child wires.
pub fn boundary_wires<'a, W: WireKind + 'a, C: 'a, B: 'a, A: 'a, T: 'a>(
    nodes: impl IntoIterator<Item = &'a WireNode<W, C, B, A, T>>,
) -> impl std::iter::FusedIterator<Item = &'a Wire<W>> {
    definitions(nodes, false)
}
fn definitions<'a, W: WireKind + 'a, C: 'a, B: 'a, A: 'a, T: 'a>(
    nodes: impl IntoIterator<Item = &'a WireNode<W, C, B, A, T>>,
    descend_containers: bool,
) -> impl std::iter::FusedIterator<Item = &'a Wire<W>> {
    enum Frame<'a, W: WireKind, C, B, A, T> {
        Nodes(std::slice::Iter<'a, WireNode<W, C, B, A, T>>),
        Wires(std::slice::Iter<'a, Wire<W>>),
    }
    let mut roots = nodes.into_iter().fuse();
    let mut stack = vec![];
    std::iter::from_fn(move || loop {
        let node = match stack.last_mut() {
            Some(Frame::Wires(wires)) => {
                if let Some(w) = wires.next() {
                    return Some(w);
                }
                stack.pop();
                continue;
            }
            Some(Frame::Nodes(nodes)) => {
                if let Some(n) = nodes.next() {
                    n
                } else {
                    stack.pop();
                    continue;
                }
            }
            None => roots.next()?,
        };
        match node {
            WireNode::Leaf(w) => stack.push(Frame::Wires(w.iter())),
            WireNode::Annotation { child, .. } => {
                stack.push(Frame::Nodes(std::slice::from_ref(child.as_ref()).iter()))
            }
            WireNode::Container { wires, child, .. } => {
                if descend_containers {
                    stack.push(Frame::Nodes(std::slice::from_ref(child.as_ref()).iter()));
                }
                stack.push(Frame::Wires(wires.iter()));
            }
            WireNode::Chain {
                controls, child, ..
            } => {
                stack.push(Frame::Nodes(std::slice::from_ref(child.as_ref()).iter()));
                stack.push(Frame::Wires(controls.iter()));
            }
            WireNode::Branch {
                controls, children, ..
            } => {
                stack.push(Frame::Nodes(children.iter()));
                stack.push(Frame::Wires(controls.iter()));
            }
        }
    })
    .fuse()
}
/// Mutably visit every definition without requiring cloneable wires or metadata.
/// Traversal order matches [`wire_definitions`].
pub fn wire_definitions_mut<'a, W: WireKind + 'a, C: 'a, B: 'a, A: 'a, T: 'a>(
    nodes: impl IntoIterator<Item = &'a mut WireNode<W, C, B, A, T>>,
) -> impl std::iter::FusedIterator<Item = &'a mut Wire<W>> {
    definitions_mut(nodes, true)
}
/// Visit only wires exposed at this boundary, stopping at container wires.
/// Annotations are transparent; chain/branch controls precede their child wires.
pub fn boundary_wires_mut<'a, W: WireKind + 'a, C: 'a, B: 'a, A: 'a, T: 'a>(
    nodes: impl IntoIterator<Item = &'a mut WireNode<W, C, B, A, T>>,
) -> impl std::iter::FusedIterator<Item = &'a mut Wire<W>> {
    definitions_mut(nodes, false)
}
fn definitions_mut<'a, W: WireKind + 'a, C: 'a, B: 'a, A: 'a, T: 'a>(
    nodes: impl IntoIterator<Item = &'a mut WireNode<W, C, B, A, T>>,
    descend_containers: bool,
) -> impl std::iter::FusedIterator<Item = &'a mut Wire<W>> {
    enum Frame<'a, W: WireKind, C, B, A, T> {
        Nodes(std::slice::IterMut<'a, WireNode<W, C, B, A, T>>),
        Wires(std::slice::IterMut<'a, Wire<W>>),
    }
    let mut roots = nodes.into_iter().fuse();
    let mut stack = vec![];
    std::iter::from_fn(move || loop {
        let node = match stack.last_mut() {
            Some(Frame::Wires(wires)) => {
                if let Some(w) = wires.next() {
                    return Some(w);
                }
                stack.pop();
                continue;
            }
            Some(Frame::Nodes(nodes)) => {
                if let Some(n) = nodes.next() {
                    n
                } else {
                    stack.pop();
                    continue;
                }
            }
            None => roots.next()?,
        };
        match node {
            WireNode::Leaf(w) => stack.push(Frame::Wires(w.iter_mut())),
            WireNode::Annotation { child, .. } => stack.push(Frame::Nodes(
                std::slice::from_mut(child.as_mut()).iter_mut(),
            )),
            WireNode::Container { wires, child, .. } => {
                if descend_containers {
                    stack.push(Frame::Nodes(
                        std::slice::from_mut(child.as_mut()).iter_mut(),
                    ));
                }
                stack.push(Frame::Wires(wires.iter_mut()));
            }
            WireNode::Chain {
                controls, child, ..
            } => {
                stack.push(Frame::Nodes(
                    std::slice::from_mut(child.as_mut()).iter_mut(),
                ));
                stack.push(Frame::Wires(controls.iter_mut()));
            }
            WireNode::Branch {
                controls, children, ..
            } => {
                stack.push(Frame::Nodes(children.iter_mut()));
                stack.push(Frame::Wires(controls.iter_mut()));
            }
        }
    })
    .fuse()
}
/// Address relative to selected roots. The first index selects a root. At any
/// parent, `[0, i]` selects control or container wire `i`, and `[1, i]` selects child `i`; a chain
/// or container has only child zero. An annotation has only the `[1, 0]` route.
/// At a leaf, one final index selects a wire in its bundle.
/// Select the output channel before traversal. Rebuild paths after structural
/// edits or changing the selected roots.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct WirePath(pub Vec<usize>);
/// A wire definition borrowed together with its structural address.
pub struct WireEntry<'a, W: WireKind> {
    /// Address within the selected roots.
    pub path: WirePath,
    /// Definition in a leaf, chain/branch controls, or container wires.
    pub wire: &'a Wire<W>,
}
/// Visit all wire definitions with paths, in [`wire_definitions`] order.
pub fn wire_definitions_with_paths<'a, W: WireKind + 'a, C: 'a, B: 'a, A: 'a, T: 'a>(
    nodes: impl IntoIterator<Item = &'a WireNode<W, C, B, A, T>>,
) -> impl std::iter::FusedIterator<Item = WireEntry<'a, W>> {
    fn visit<'a, W: WireKind, C, B, A, T>(
        node: &'a WireNode<W, C, B, A, T>,
        path: &mut Vec<usize>,
        out: &mut Vec<WireEntry<'a, W>>,
    ) {
        let (controls, children) = match node {
            WireNode::Leaf(wires) => {
                for (i, wire) in wires.iter().enumerate() {
                    path.push(i);
                    out.push(WireEntry {
                        path: WirePath(path.clone()),
                        wire,
                    });
                    path.pop();
                }
                return;
            }
            WireNode::Annotation { child, .. } => (&[][..], std::slice::from_ref(child.as_ref())),
            WireNode::Chain {
                controls, child, ..
            }
            | WireNode::Container {
                wires: controls,
                child,
                ..
            } => (controls.as_slice(), std::slice::from_ref(child.as_ref())),
            WireNode::Branch {
                controls, children, ..
            } => (controls.as_slice(), children.as_slice()),
        };
        for (i, wire) in controls.iter().enumerate() {
            path.extend([0, i]);
            out.push(WireEntry {
                path: WirePath(path.clone()),
                wire,
            });
            path.truncate(path.len() - 2);
        }
        for (i, child) in children.iter().enumerate() {
            path.extend([1, i]);
            visit(child, path, out);
            path.truncate(path.len() - 2);
        }
    }
    let mut out = vec![];
    for (i, node) in nodes.into_iter().enumerate() {
        visit(node, &mut vec![i], &mut out);
    }
    out.into_iter()
}
/// Resolve a path for the same roots and output channel used when producing it.
pub fn wire_at_path<'a, W: WireKind + 'a, C: 'a, B: 'a, A: 'a, T: 'a>(
    nodes: impl IntoIterator<Item = &'a WireNode<W, C, B, A, T>>,
    path: &WirePath,
) -> Option<&'a Wire<W>> {
    fn get<'a, W: WireKind, C, B, A, T>(
        node: &'a WireNode<W, C, B, A, T>,
        path: &[usize],
    ) -> Option<&'a Wire<W>> {
        let (&index, rest) = path.split_first()?;
        let (controls, children) = match node {
            WireNode::Leaf(wires) => return rest.is_empty().then(|| wires.get(index)).flatten(),
            WireNode::Annotation { child, .. } => (&[][..], std::slice::from_ref(child.as_ref())),
            WireNode::Chain {
                controls, child, ..
            }
            | WireNode::Container {
                wires: controls,
                child,
                ..
            } => (controls.as_slice(), std::slice::from_ref(child.as_ref())),
            WireNode::Branch {
                controls, children, ..
            } => (controls.as_slice(), children.as_slice()),
        };
        match (index, rest) {
            (0, [i]) => controls.get(*i),
            (1, [i, rest @ ..]) => get(children.get(*i)?, rest),
            _ => None,
        }
    }
    let (&root, rest) = path.0.split_first()?;
    get(nodes.into_iter().nth(root)?, rest)
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
        WireNode::Leaf(vec![wire(name)])
    }
    #[test]
    fn terminal_bundles_preserve_zero_one_and_multiple_values() {
        let nodes = [
            WireNode::Leaf(vec![]),
            leaf("one"),
            WireNode::Leaf(vec![wire("two"), wire("three")]),
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
    fn paths_cover_branch_and_chain_controls_and_children() {
        let nodes = [WireNode::Branch {
            controls: vec![wire("header")],
            meta: (),
            children: vec![WireNode::Branch {
                controls: vec![wire("tag")],
                meta: (),
                children: vec![
                    WireNode::default(),
                    WireNode::Chain {
                        controls: vec![wire("rows")],
                        meta: (),
                        child: Box::new(WireNode::Chain {
                            controls: vec![wire("present")],
                            meta: (),
                            child: Box::new(leaf("value")),
                        }),
                    },
                ],
            }],
        }];
        let entries = wire_definitions_with_paths(nodes.iter()).collect::<Vec<_>>();
        assert_eq!(
            entries
                .iter()
                .map(|e| e.wire.name.to_string())
                .collect::<Vec<_>>(),
            ["header", "tag", "rows", "present", "value"]
        );
        assert_eq!(
            entries.iter().map(|e| e.path.clone()).collect::<Vec<_>>(),
            [
                WirePath(vec![0, 0, 0]),
                WirePath(vec![0, 1, 0, 0, 0]),
                WirePath(vec![0, 1, 0, 1, 1, 0, 0]),
                WirePath(vec![0, 1, 0, 1, 1, 1, 0, 0, 0]),
                WirePath(vec![0, 1, 0, 1, 1, 1, 0, 1, 0, 0]),
            ]
        );
        for entry in entries {
            assert!(std::ptr::eq(
                entry.wire,
                wire_at_path(nodes.iter(), &entry.path).unwrap()
            ));
        }
        for invalid in [
            WirePath(vec![0, 0, 99]),
            WirePath(vec![0, 0, 0, 9]),
            WirePath(vec![0, 0, 1, 99]),
            // Chains expose child zero only; parent tags other than 0/1 are invalid.
            WirePath(vec![0, 1, 0, 1, 1, 1, 1, 0]),
            WirePath(vec![0, 2, 0]),
            // A leaf address must include its final wire index.
            WirePath(vec![0, 1, 0, 1, 1, 1, 0, 1, 0]),
        ] {
            assert!(wire_at_path(nodes.iter(), &invalid).is_none());
        }
    }
    struct Kind(u32);
    impl WireKind for Kind {
        fn rust_type(&self) -> syn::Type {
            syn::parse_quote!(u32)
        }
    }
    struct ChainInfo {
        annotation: u32,
    }
    struct BranchInfo;
    fn typed_leaf(name: &str, value: u32) -> WireNode<Kind, ChainInfo, BranchInfo> {
        WireNode::Leaf(vec![Wire {
            name: syn::parse_str(name).unwrap(),
            kind: Kind(value),
        }])
    }
    #[test]
    fn typed_metadata_and_mutable_traversal_need_no_clone() {
        let mut nodes = [
            typed_leaf("untouched", 1),
            WireNode::Chain {
                controls: vec![Wire {
                    name: syn::parse_quote!(flag),
                    kind: Kind(5),
                }],
                meta: ChainInfo { annotation: 99 },
                child: Box::new(WireNode::Chain {
                    controls: vec![Wire {
                        name: syn::parse_quote!(rows),
                        kind: Kind(6),
                    }],
                    meta: ChainInfo { annotation: 7 },
                    child: Box::new(typed_leaf("changed", 2)),
                }),
            },
            WireNode::Branch {
                meta: BranchInfo,
                controls: vec![Wire {
                    name: syn::parse_quote!(header),
                    kind: Kind(3),
                }],
                children: vec![typed_leaf("branch_value", 4)],
            },
        ];
        for wire in wire_definitions_mut(nodes.iter_mut().skip(1)) {
            wire.kind.0 += 10;
        }
        assert_eq!(
            wire_definitions(nodes.iter())
                .map(|w| w.kind.0)
                .collect::<Vec<_>>(),
            [1, 15, 16, 12, 13, 14]
        );
        let WireNode::Chain { meta: metadata, .. } = &nodes[1] else {
            panic!()
        };
        assert_eq!(metadata.annotation, 99);
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
    fn empty_payloads_preserve_chain_and_branch_structure() {
        let nodes: [WireNode<syn::Type>; 2] = [
            WireNode::Chain {
                controls: vec![],
                meta: (),
                child: Box::default(),
            },
            WireNode::Branch {
                controls: vec![],
                meta: (),
                children: vec![],
            },
        ];
        assert!(wire_definitions(nodes.iter()).next().is_none());
        assert!(matches!(nodes[0], WireNode::Chain { .. }));
        assert!(matches!(nodes[1], WireNode::Branch { .. }));
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
                    prebindgen_flat::type_key(
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
    fn empty_branch_can_retain_controls_without_children() {
        let node: WireNode<syn::Type> = WireNode::Branch {
            controls: vec![wire("header")],
            children: vec![],
            meta: (),
        };
        let entries = wire_definitions_with_paths(std::iter::once(&node)).collect::<Vec<_>>();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].path, WirePath(vec![0, 0, 0]));
        assert!(std::ptr::eq(
            entries[0].wire,
            wire_at_path(std::iter::once(&node), &entries[0].path).unwrap()
        ));
        assert!(wire_at_path(std::iter::once(&node), &WirePath(vec![0, 1, 0, 0])).is_none());
    }
    #[test]
    fn annotations_preserve_boundaries_and_containers_hide_internal_wires() {
        // Shape-specific metadata has unrelated types; no wrapper enum is needed.
        type Node = WireNode<syn::Type, (), (), String, usize>;
        let mut nodes: [Node; 1] = [WireNode::Annotation {
            meta: "packets".into(),
            child: Box::new(WireNode::Container {
                meta: 1,
                wires: vec![wire("rows")],
                child: Box::new(WireNode::Branch {
                    meta: (),
                    controls: vec![wire("tag")],
                    children: vec![
                        leaf_for_shapes("x"),
                        WireNode::Container {
                            meta: 2,
                            wires: vec![wire("nested_rows")],
                            child: Box::new(leaf_for_shapes("nested_value")),
                        },
                    ],
                }),
            }),
        }];
        assert_eq!(
            boundary_wires(nodes.iter())
                .map(|w| w.name.to_string())
                .collect::<Vec<_>>(),
            ["rows"]
        );
        assert_eq!(
            wire_definitions(nodes.iter())
                .map(|w| w.name.to_string())
                .collect::<Vec<_>>(),
            ["rows", "tag", "x", "nested_rows", "nested_value"]
        );
        for entry in wire_definitions_with_paths(nodes.iter()) {
            assert!(std::ptr::eq(
                entry.wire,
                wire_at_path(nodes.iter(), &entry.path).unwrap()
            ));
        }
        // An annotation owns no controls or wires. Its only child is at [1, 0].
        assert!(wire_at_path(nodes.iter(), &WirePath(vec![0, 0, 0])).is_none());
        assert_eq!(
            wire_at_path(nodes.iter(), &WirePath(vec![0, 1, 0, 0, 0]))
                .unwrap()
                .name,
            "rows"
        );
        for wire in boundary_wires_mut(nodes.iter_mut()) {
            wire.kind = syn::parse_quote!(u8);
        }
        assert_eq!(
            wire_definitions(nodes.iter())
                .map(|w| prebindgen_flat::type_key(&w.kind))
                .collect::<Vec<_>>(),
            ["u8", "i64", "i64", "i64", "i64"]
        );
        let WireNode::Annotation { child, .. } = &nodes[0] else {
            panic!()
        };
        let WireNode::Container { child, .. } = child.as_ref() else {
            panic!()
        };
        assert_eq!(
            boundary_wires(std::iter::once(child.as_ref()))
                .map(|w| w.name.to_string())
                .collect::<Vec<_>>(),
            ["tag", "x", "nested_rows"]
        );
        fn leaf_for_shapes(name: &str) -> Node {
            WireNode::Leaf(vec![wire(name)])
        }
    }
    #[test]
    fn empty_container_boundary_still_hides_nonempty_internal_representation() {
        let mut node: WireNode<syn::Type> = WireNode::Container {
            meta: (),
            wires: vec![],
            child: Box::new(leaf("internal")),
        };
        assert!(boundary_wires(std::iter::once(&node)).next().is_none());
        assert!(boundary_wires_mut(std::iter::once(&mut node))
            .next()
            .is_none());
        assert_eq!(
            wire_definitions(std::iter::once(&node))
                .next()
                .unwrap()
                .name,
            "internal"
        );
    }
}
