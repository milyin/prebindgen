//! Optional native layout used by the staged C/JNI protocol.
//! This is a reader-selected transport, not the common logical specification.
use prebindgen_flat::{Result, Wire, WireKind};

/// Explicit boundary representation, independent of optional/sequence cardinality.
/// Each physical wire is stored exactly once in the tree.
#[derive(Clone, Debug)]
pub enum TransportRepresentation<W: WireKind> {
    /// Control wires followed by the element's wires in the enclosing scope.
    Inline {
        /// Physical controls, such as a presence flag; empty when a niche is used.
        controls: Vec<Wire<W>>,
    },
    /// One complete FFI value in the enclosing scope. Element fields have
    /// independent positions, such as array row fields or callback arguments.
    /// The carrier transports the elements or reports delivery/count/presence.
    Carrier(Wire<W>),
}

/// Native layout of an optional value or a sequence for the staged reader.
#[derive(Clone, Debug)]
pub struct TransportContainer<W: WireKind, G> {
    /// Explicit boundary representation and wire scope.
    pub representation: TransportRepresentation<W>,
    /// One present payload/sequence element. Control wires are kept separately.
    pub element: TransportLayout<W, G>,
    /// Adapter-specific information; cardinality and scope are defined above it.
    pub metadata: Option<G>,
}
impl<W: WireKind, G> TransportContainer<W, G> {
    /// Physical wires occupied in the enclosing scope.
    pub fn wire_count(&self) -> usize {
        match &self.representation {
            TransportRepresentation::Inline { controls } => controls.len() + self.element.len(),
            TransportRepresentation::Carrier(_) => 1,
        }
    }
    /// Carrier when element fields have their own scope.
    pub fn carrier(&self) -> Option<&Wire<W>> {
        match &self.representation {
            TransportRepresentation::Carrier(wire) => Some(wire),
            TransportRepresentation::Inline { .. } => None,
        }
    }
}

/// One FFI value, a container, or a relationship between child values.
#[derive(Clone, Debug)]
pub enum TransportNode<W: WireKind, G> {
    /// One physical FFI value. Its metadata belongs in `W`.
    Wire(Wire<W>),
    /// Zero or one occurrence of the element layout.
    Optional(TransportContainer<W, G>),
    /// Zero or more occurrences of the element layout.
    Sequence(TransportContainer<W, G>),
    /// Selector followed by the physical slots of every alternative. Only one
    /// arm is active, but all slots remain in the native signature.
    Alternatives {
        /// Native selector control, excluded from the logical payload.
        selector: Wire<W>,
        /// Arm layouts in selector order.
        alternatives: Vec<TransportLayout<W, G>>,
    },
    /// Adapter-described relationships such as records and named fields.
    /// Group metadata adds no physical wires by itself.
    Group {
        /// Information about the relationship between child values.
        metadata: G,
        /// Child values in ABI order.
        children: TransportLayout<W, G>,
    },
}
impl<W: WireKind, G> TransportNode<W, G> {
    /// Physical wires occupied in the enclosing scope.
    pub fn wire_count(&self) -> usize {
        match self {
            Self::Wire(_) => 1,
            Self::Alternatives { alternatives, .. } => {
                1 + alternatives.iter().map(TransportLayout::len).sum::<usize>()
            }
            Self::Optional(c) | Self::Sequence(c) => c.wire_count(),
            Self::Group { children, .. } => children.len(),
        }
    }
}

/// Authoritative wire order and structure for one conversion slot.
///
/// TransportWires are visited depth first, left to right. Inline containers expose their
/// control wires then their element wires. Carried containers expose only the
/// carrier; their element layouts have independent positions. This order also
/// defines `crate::NativeConversionExt::native_wires()` and its native requests, including
/// inactive alternatives. Empty groups and inline elements are allowed.
///
/// Build/edit the tree before emission; positions must not survive structural
/// edits. Emission borrows it immutably so every request sees the same order.
#[derive(Clone, Debug)]
pub struct TransportLayout<W: WireKind, G = ()> {
    nodes: Vec<TransportNode<W, G>>,
    // Derived counts, not a second source of wires or metadata.
    ends: Vec<usize>,
}
impl<W: WireKind, G> Default for TransportLayout<W, G> {
    fn default() -> Self {
        Self {
            nodes: vec![],
            ends: vec![],
        }
    }
}
impl<W: WireKind, G> From<Vec<Wire<W>>> for TransportLayout<W, G> {
    fn from(wires: Vec<Wire<W>>) -> Self {
        wires.into_iter().collect()
    }
}
impl<W: WireKind, G> FromIterator<Wire<W>> for TransportLayout<W, G> {
    fn from_iter<T: IntoIterator<Item = Wire<W>>>(iter: T) -> Self {
        Self::from_nodes(iter.into_iter().map(TransportNode::Wire).collect())
    }
}
impl<W: WireKind, G> TransportLayout<W, G> {
    /// Build a tree, deriving positions from its explicit representations.
    pub fn from_nodes(nodes: Vec<TransportNode<W, G>>) -> Self {
        let mut count = 0;
        let ends = nodes
            .iter()
            .map(|node| {
                count += node.wire_count();
                count
            })
            .collect();
        Self { nodes, ends }
    }
    /// Consume the tree for structural editing; rebuild with `from_nodes`.
    pub fn into_nodes(self) -> Vec<TransportNode<W, G>> {
        self.nodes
    }
    /// Immediate child nodes, including container structure and metadata.
    pub fn nodes(&self) -> &[TransportNode<W, G>] {
        &self.nodes
    }
    /// Append one physical wire during preparation.
    pub fn push(&mut self, wire: Wire<W>) {
        self.ends.push(self.len() + 1);
        self.nodes.push(TransportNode::Wire(wire));
    }
    /// Put one physical wire before the current children.
    pub fn prepend(&mut self, wire: Wire<W>) {
        for end in &mut self.ends {
            *end += 1;
        }
        self.ends.insert(0, 1);
        self.nodes.insert(0, TransportNode::Wire(wire));
    }
    /// Append another tree without discarding structure or metadata.
    pub fn extend(&mut self, other: Self) {
        let offset = self.len();
        self.ends
            .extend(other.ends.into_iter().map(|end| end + offset));
        self.nodes.extend(other.nodes);
    }
    /// Enclose the current tree in an adapter-described group.
    pub fn enclose(&mut self, metadata: G) {
        let children = std::mem::take(self);
        *self = Self::from_nodes(vec![TransportNode::Group { metadata, children }]);
    }
    /// Make the current tree an optional payload with an explicit representation.
    pub fn enclose_optional(
        &mut self,
        representation: TransportRepresentation<W>,
        metadata: Option<G>,
    ) {
        let element = std::mem::take(self);
        *self = Self::from_nodes(vec![TransportNode::Optional(TransportContainer {
            representation,
            element,
            metadata,
        })]);
    }
    /// Make the current tree a sequence element with an explicit representation.
    pub fn enclose_sequence(
        &mut self,
        representation: TransportRepresentation<W>,
        metadata: Option<G>,
    ) {
        let element = std::mem::take(self);
        *self = Self::from_nodes(vec![TransportNode::Sequence(TransportContainer {
            representation,
            element,
            metadata,
        })]);
    }
    /// Physical wire count in this scope, including inactive alternatives.
    pub fn len(&self) -> usize {
        self.ends.last().copied().unwrap_or(0)
    }
    /// Whether this scope has no physical wires; structure may still be present.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    /// Physical wires in ABI order. Individual metadata is available in `kind`.
    pub fn iter(&self) -> TransportWires<'_, W, G> {
        TransportWires {
            stack: vec![Scope::Nodes(self.nodes.iter())],
            remaining: self.len(),
        }
    }
    /// Edit physical wires without changing their order or structure.
    pub fn iter_mut(&mut self) -> TransportWiresMut<'_, W, G> {
        TransportWiresMut {
            stack: vec![ScopeMut::Nodes(self.nodes.iter_mut())],
        }
    }
    /// Find one physical wire at its derived position in this scope.
    pub fn get(&self, pos: usize) -> Option<&Wire<W>> {
        let index = self.ends.partition_point(|end| *end <= pos);
        let local = pos - if index == 0 { 0 } else { self.ends[index - 1] };
        match self.nodes.get(index)? {
            TransportNode::Wire(wire) => Some(wire),
            TransportNode::Group { children, .. } => children.get(local),
            TransportNode::Alternatives {
                selector,
                alternatives,
            } => {
                if local == 0 {
                    Some(selector)
                } else {
                    let mut offset = local - 1;
                    for arm in alternatives {
                        if offset < arm.len() {
                            return arm.get(offset);
                        }
                        offset -= arm.len();
                    }
                    None
                }
            }
            TransportNode::Optional(c) | TransportNode::Sequence(c) => match &c.representation {
                TransportRepresentation::Carrier(wire) => Some(wire),
                TransportRepresentation::Inline { controls } => {
                    if local < controls.len() {
                        controls.get(local)
                    } else {
                        c.element.get(local - controls.len())
                    }
                }
            },
        }
    }
    /// Check unique physical-wire names across all scopes of this slot.
    /// Adapter-specific metadata rules are checked by the adapter.
    pub fn validate(&self) -> Result<()> {
        fn insert<W: WireKind>(
            wire: &Wire<W>,
            names: &mut std::collections::BTreeSet<String>,
        ) -> Result<()> {
            if !names.insert(wire.name.to_string()) {
                return Err(format!("duplicate wire identifier `{}`", wire.name));
            }
            Ok(())
        }
        fn walk<W: WireKind, G>(
            layout: &TransportLayout<W, G>,
            names: &mut std::collections::BTreeSet<String>,
        ) -> Result<()> {
            for node in layout.nodes() {
                match node {
                    TransportNode::Wire(wire) => insert(wire, names)?,
                    TransportNode::Group { children, .. } => walk(children, names)?,
                    TransportNode::Alternatives {
                        selector,
                        alternatives,
                    } => {
                        if alternatives.is_empty() {
                            return Err("selector has no alternatives".into());
                        }
                        insert(selector, names)?;
                        for arm in alternatives {
                            walk(arm, names)?;
                        }
                    }
                    TransportNode::Optional(c) | TransportNode::Sequence(c) => {
                        match &c.representation {
                            TransportRepresentation::Inline { controls } => {
                                for wire in controls {
                                    insert(wire, names)?;
                                }
                            }
                            TransportRepresentation::Carrier(wire) => insert(wire, names)?,
                        }
                        walk(&c.element, names)?;
                    }
                }
            }
            Ok(())
        }
        walk(self, &mut std::collections::BTreeSet::new())
    }
}
impl<W: WireKind, G> std::ops::Index<usize> for TransportLayout<W, G> {
    type Output = Wire<W>;
    fn index(&self, pos: usize) -> &Self::Output {
        self.get(pos).expect("wire position out of range")
    }
}
impl<W: WireKind, G> std::ops::IndexMut<usize> for TransportLayout<W, G> {
    fn index_mut(&mut self, pos: usize) -> &mut Self::Output {
        self.iter_mut()
            .nth(pos)
            .expect("wire position out of range")
    }
}
enum Scope<'a, W: WireKind, G> {
    Nodes(std::slice::Iter<'a, TransportNode<W, G>>),
    TransportWires(std::slice::Iter<'a, Wire<W>>),
}
/// Borrowed physical wires in the enclosing scope's ABI order.
pub struct TransportWires<'a, W: WireKind, G> {
    stack: Vec<Scope<'a, W, G>>,
    remaining: usize,
}
impl<'a, W: WireKind, G> Iterator for TransportWires<'a, W, G> {
    type Item = &'a Wire<W>;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.stack.last_mut()? {
                Scope::TransportWires(wires) => {
                    if let Some(wire) = wires.next() {
                        self.remaining -= 1;
                        return Some(wire);
                    }
                }
                Scope::Nodes(nodes) => match nodes.next() {
                    Some(TransportNode::Wire(wire)) => {
                        self.remaining -= 1;
                        return Some(wire);
                    }
                    Some(TransportNode::Alternatives {
                        selector,
                        alternatives,
                    }) => {
                        for arm in alternatives.iter().rev() {
                            self.stack.push(Scope::Nodes(arm.nodes.iter()));
                        }
                        self.remaining -= 1;
                        return Some(selector);
                    }
                    Some(TransportNode::Group { children, .. }) => {
                        self.stack.push(Scope::Nodes(children.nodes.iter()));
                        continue;
                    }
                    Some(TransportNode::Optional(c) | TransportNode::Sequence(c)) => {
                        match &c.representation {
                            TransportRepresentation::Carrier(wire) => {
                                self.remaining -= 1;
                                return Some(wire);
                            }
                            TransportRepresentation::Inline { controls } => {
                                self.stack.push(Scope::Nodes(c.element.nodes.iter()));
                                self.stack.push(Scope::TransportWires(controls.iter()));
                            }
                        }
                        continue;
                    }
                    None => {}
                },
            }
            self.stack.pop();
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining, Some(self.remaining))
    }
}
impl<W: WireKind, G> ExactSizeIterator for TransportWires<'_, W, G> {}
impl<W: WireKind, G> std::iter::FusedIterator for TransportWires<'_, W, G> {}
enum ScopeMut<'a, W: WireKind, G> {
    Nodes(std::slice::IterMut<'a, TransportNode<W, G>>),
    TransportWires(std::slice::IterMut<'a, Wire<W>>),
}
/// Mutable physical wires in the enclosing scope; independent element scopes are excluded.
pub struct TransportWiresMut<'a, W: WireKind, G> {
    stack: Vec<ScopeMut<'a, W, G>>,
}
impl<'a, W: WireKind, G> Iterator for TransportWiresMut<'a, W, G> {
    type Item = &'a mut Wire<W>;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.stack.last_mut()? {
                ScopeMut::TransportWires(wires) => {
                    if let Some(wire) = wires.next() {
                        return Some(wire);
                    }
                }
                ScopeMut::Nodes(nodes) => match nodes.next() {
                    Some(TransportNode::Wire(wire)) => return Some(wire),
                    Some(TransportNode::Alternatives {
                        selector,
                        alternatives,
                    }) => {
                        for arm in alternatives.iter_mut().rev() {
                            self.stack.push(ScopeMut::Nodes(arm.nodes.iter_mut()));
                        }
                        return Some(selector);
                    }
                    Some(TransportNode::Group { children, .. }) => {
                        self.stack.push(ScopeMut::Nodes(children.nodes.iter_mut()));
                        continue;
                    }
                    Some(TransportNode::Optional(c) | TransportNode::Sequence(c)) => {
                        match &mut c.representation {
                            TransportRepresentation::Carrier(wire) => return Some(wire),
                            TransportRepresentation::Inline { controls } => {
                                self.stack.push(ScopeMut::Nodes(c.element.nodes.iter_mut()));
                                self.stack
                                    .push(ScopeMut::TransportWires(controls.iter_mut()));
                            }
                        }
                        continue;
                    }
                    None => {}
                },
            }
            self.stack.pop();
        }
    }
}
impl<'a, W: WireKind, G> IntoIterator for &'a TransportLayout<W, G> {
    type Item = &'a Wire<W>;
    type IntoIter = TransportWires<'a, W, G>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
impl<'a, W: WireKind, G> IntoIterator for &'a mut TransportLayout<W, G> {
    type Item = &'a mut Wire<W>;
    type IntoIter = TransportWiresMut<'a, W, G>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<W: WireKind + Clone, G: Clone> TransportLayout<W, G> {
    /// Describe logical payloads, discarding this transport's carriers and controls.
    /// Container metadata in this native plan describes presence/transport details
    /// and is omitted. Field/record metadata is retained. A later reader may
    /// choose another transport for these same logical definitions.
    pub fn logical(&self) -> prebindgen_flat::WireLayout<W, G> {
        use prebindgen_flat::{WireContainer, WireLayout, WireNode};
        WireLayout::from_nodes(
            self.nodes()
                .iter()
                .map(|node| match node {
                    TransportNode::Wire(w) => WireNode::Wire(w.clone()),
                    TransportNode::Alternatives { alternatives, .. } => WireNode::Alternatives {
                        alternatives: alternatives.iter().map(TransportLayout::logical).collect(),
                    },
                    TransportNode::Group { metadata, children } => WireNode::Group {
                        metadata: metadata.clone(),
                        children: children.logical(),
                    },
                    TransportNode::Optional(c) => WireNode::Optional(WireContainer {
                        element: c.element.logical(),
                        metadata: None,
                    }),
                    TransportNode::Sequence(c) => WireNode::Sequence(WireContainer {
                        element: c.element.logical(),
                        metadata: None,
                    }),
                })
                .collect(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[derive(Clone, Debug, PartialEq)]
    enum Meaning {
        Record,
        Choice,
        Variant(&'static str),
    }
    fn leaf(name: &str) -> Wire<syn::Type> {
        Wire {
            name: syn::parse_str(name).unwrap(),
            kind: syn::parse_quote!(i32),
        }
    }
    #[test]
    fn cardinality_does_not_choose_wire_scope() {
        for optional in [true, false] {
            for inline in [true, false] {
                let mut payload: TransportLayout<_, Meaning> = vec![leaf("x"), leaf("y")].into();
                payload.enclose(Meaning::Record);
                let representation = if inline {
                    TransportRepresentation::Inline {
                        controls: vec![leaf("present")],
                    }
                } else {
                    TransportRepresentation::Carrier(leaf("array"))
                };
                if optional {
                    payload.enclose_optional(representation, Some(Meaning::Choice));
                } else {
                    payload.enclose_sequence(representation, Some(Meaning::Choice));
                }
                let mut tree: TransportLayout<_, Meaning> = vec![leaf("before")].into();
                tree.extend(payload);
                tree.push(leaf("after"));
                tree.validate().unwrap();
                let expected = if inline {
                    vec!["before", "present", "x", "y", "after"]
                } else {
                    vec!["before", "array", "after"]
                };
                assert_eq!(
                    tree.iter().map(|w| w.name.to_string()).collect::<Vec<_>>(),
                    expected
                );
                assert_eq!(tree.len(), expected.len());
                for (i, wire) in tree.iter().enumerate() {
                    assert!(std::ptr::eq(wire, tree.get(i).unwrap()));
                }
                assert!(tree.get(expected.len()).is_none());
                assert!(tree.get(usize::MAX).is_none());
                let mut iter = tree.iter();
                for remaining in (1..=expected.len()).rev() {
                    assert_eq!(iter.len(), remaining);
                    iter.next().unwrap();
                }
                assert_eq!(iter.len(), 0);
                assert!(iter.next().is_none());
                assert!(iter.next().is_none());
                for wire in tree.iter_mut() {
                    wire.kind = syn::parse_quote!(u64);
                }
                let (TransportNode::Optional(c) | TransportNode::Sequence(c)) = &tree.nodes()[1]
                else {
                    panic!()
                };
                assert_eq!(c.metadata, Some(Meaning::Choice));
                assert_eq!(c.element.len(), 2);
                // Mutating outer wires never descends into an independent scope.
                assert_eq!(
                    prebindgen_flat::type_key(&c.element[0].kind),
                    if inline { "u64" } else { "i32" }
                );
            }
        }
    }
    #[test]
    fn nested_optional_sequences_keep_flags_outside_element_schemas() {
        let mut element: TransportLayout<_, ()> = vec![leaf("value")].into();
        element.enclose_optional(
            TransportRepresentation::Inline {
                controls: vec![leaf("item_present")],
            },
            None,
        );
        element.enclose_sequence(TransportRepresentation::Carrier(leaf("rows")), None);
        element.enclose_optional(
            TransportRepresentation::Inline {
                controls: vec![leaf("list_present")],
            },
            None,
        );
        element.validate().unwrap();
        assert_eq!(
            element
                .iter()
                .map(|w| w.name.to_string())
                .collect::<Vec<_>>(),
            ["list_present", "rows"]
        );
        let TransportNode::Optional(outer) = &element.nodes()[0] else {
            panic!()
        };
        let TransportNode::Sequence(sequence) = &outer.element.nodes()[0] else {
            panic!()
        };
        assert_eq!(
            sequence
                .element
                .iter()
                .map(|w| w.name.to_string())
                .collect::<Vec<_>>(),
            ["item_present", "value"]
        );
        let TransportNode::Optional(item) = &sequence.element.nodes()[0] else {
            panic!()
        };
        assert_eq!(item.element.len(), 1);
        assert_eq!(item.element[0].name, "value");
        let duplicate: TransportLayout<_, ()> =
            TransportLayout::from_nodes(vec![TransportNode::Optional(TransportContainer {
                representation: TransportRepresentation::Inline {
                    controls: vec![leaf("value")],
                },
                element: vec![leaf("value")].into(),
                metadata: None,
            })]);
        assert!(duplicate.validate().unwrap_err().contains("duplicate wire"));
        let mut empty: TransportLayout<syn::Type, ()> = TransportLayout::default();
        empty.enclose_optional(TransportRepresentation::Inline { controls: vec![] }, None);
        assert_eq!(empty.len(), 0);
        assert!(empty.iter_mut().next().is_none());
        empty.enclose_optional(
            TransportRepresentation::Inline {
                controls: vec![leaf("flag")],
            },
            None,
        );
        assert_eq!(empty.len(), 1);
        assert_eq!(empty[0].name, "flag");
    }
    #[test]
    fn sequence_is_one_outer_position_with_an_independent_element_schema() {
        let mut rows: TransportLayout<_, Meaning> = vec![leaf("x"), leaf("y")].into();
        rows.enclose(Meaning::Record);
        let mut layout = TransportLayout::from_nodes(vec![
            TransportNode::Wire(leaf("before")),
            TransportNode::Sequence(TransportContainer {
                representation: TransportRepresentation::Carrier(leaf("rows")),
                element: rows,
                metadata: None,
            }),
            TransportNode::Wire(leaf("after")),
        ]);
        layout.validate().unwrap();
        assert_eq!(layout.len(), 3);
        assert_eq!(
            layout
                .iter()
                .map(|w| w.name.to_string())
                .collect::<Vec<_>>(),
            ["before", "rows", "after"]
        );
        for (i, wire) in layout.iter().enumerate() {
            assert!(std::ptr::eq(wire, &layout[i]));
        }
        layout.iter_mut().nth(1).unwrap().kind = syn::parse_quote!(*const u8);
        let TransportNode::Sequence(container) = &layout.nodes()[1] else {
            panic!()
        };
        let carrier = container.carrier().unwrap();
        let element = &container.element;
        assert_eq!(prebindgen_flat::type_key(&carrier.kind), "* const u8");
        assert_eq!(element.len(), 2);
        let TransportNode::Group { metadata, .. } = &element.nodes()[0] else {
            panic!()
        };
        assert_eq!(metadata, &Meaning::Record);
        let duplicate =
            TransportLayout::from_nodes(vec![TransportNode::Sequence(TransportContainer {
                representation: TransportRepresentation::Carrier(leaf("x")),
                element: element.clone(),
                metadata: None,
            })]);
        assert!(duplicate.validate().unwrap_err().contains("duplicate wire"));
    }
    #[test]
    fn variants_empty_groups_and_records_have_one_physical_leaf_order() {
        let mut empty = TransportLayout::default();
        empty.enclose(Meaning::Variant("Empty"));
        let mut point: TransportLayout<_, _> = vec![leaf("x"), leaf("y")].into();
        point.enclose(Meaning::Record);
        point.enclose(Meaning::Variant("Point"));
        let mut variants = empty;
        variants.extend(point);
        variants.enclose(Meaning::Choice);
        variants.prepend(leaf("tag"));
        variants.validate().unwrap();
        let mut iter = variants.iter();
        assert_eq!(iter.len(), 3);
        assert_eq!(iter.next().unwrap().name, "tag");
        assert_eq!(iter.len(), 2);
        assert_eq!(
            iter.map(|w| w.name.to_string()).collect::<Vec<_>>(),
            ["x", "y"]
        );
        for (i, w) in variants.iter().enumerate() {
            assert!(std::ptr::eq(w, variants.get(i).unwrap()));
        }
        assert!(variants.get(3).is_none());
        // Metadata is borrowed from the same authoritative tree as the wire view.
        let TransportNode::Group { metadata, children } = &variants.nodes()[1] else {
            panic!()
        };
        assert_eq!(metadata, &Meaning::Choice);
        let TransportNode::Group { metadata, children } = &children.nodes()[0] else {
            panic!()
        };
        assert_eq!(metadata, &Meaning::Variant("Empty"));
        assert!(children.is_empty());
        variants.iter_mut().nth(1).unwrap().kind = syn::parse_quote!(u64);
        assert_eq!(
            prebindgen_flat::type_key(&variants.get(1).unwrap().kind),
            "u64"
        );
    }
    #[test]
    fn indexing_skips_empty_nodes_before_between_and_after_leaves() {
        let empty = || TransportNode::Group {
            metadata: (),
            children: TransportLayout::default(),
        };
        let nested = TransportLayout::from_nodes(vec![
            empty(),
            TransportNode::Wire(leaf("a")),
            empty(),
            TransportNode::Wire(leaf("b")),
            empty(),
        ]);
        let mut tree = TransportLayout::from_nodes(vec![
            empty(),
            TransportNode::Group {
                metadata: (),
                children: nested,
            },
            empty(),
        ]);
        tree.push(leaf("c"));
        tree.prepend(leaf("flag"));
        tree.enclose(());
        assert_eq!(tree.len(), 4);
        for (i, wire) in tree.iter().enumerate() {
            assert!(std::ptr::eq(wire, tree.get(i).unwrap()));
        }
        assert_eq!(tree[0].name, "flag");
        assert_eq!(tree[3].name, "c");
        assert!(tree.get(4).is_none());
        assert!(tree.get(usize::MAX).is_none());
    }
    #[test]
    fn duplicates_are_checked_across_groups_and_order_follows_edits() {
        let mut a: TransportLayout<_, ()> = vec![leaf("value")].into();
        a.enclose(());
        let mut b: TransportLayout<_, ()> = vec![leaf("value")].into();
        b.enclose(());
        a.extend(b);
        assert!(a.validate().unwrap_err().contains("duplicate wire"));
        a.iter_mut().nth(1).unwrap().name = syn::parse_quote!(second);
        a.prepend(leaf("flag"));
        a.validate().unwrap();
        assert_eq!(
            a.iter().map(|w| w.name.to_string()).collect::<Vec<_>>(),
            ["flag", "value", "second"]
        );
    }
    #[test]
    fn selector_transport_retains_all_slots_but_logical_arms_exclude_the_tag() {
        let mut tree: TransportLayout<_, ()> =
            TransportLayout::from_nodes(vec![TransportNode::Alternatives {
                selector: leaf("tag"),
                alternatives: vec![
                    TransportLayout::default(),
                    vec![leaf("x"), leaf("y")].into(),
                ],
            }]);
        tree.validate().unwrap();
        assert_eq!(tree.len(), 3);
        assert_eq!(
            tree.iter().map(|w| w.name.to_string()).collect::<Vec<_>>(),
            ["tag", "x", "y"]
        );
        for (i, wire) in tree.iter().enumerate() {
            assert!(std::ptr::eq(wire, tree.get(i).unwrap()));
        }
        for wire in tree.iter_mut() {
            wire.kind = syn::parse_quote!(u64);
        }
        let logical = tree.logical();
        logical.validate().unwrap();
        let [prebindgen_flat::WireNode::Alternatives { alternatives }] = logical.nodes() else {
            panic!()
        };
        assert!(alternatives[0].is_empty());
        assert_eq!(alternatives[1].len(), 2);
        assert_eq!(
            logical
                .iter()
                .map(|w| w.name.to_string())
                .collect::<Vec<_>>(),
            ["x", "y"]
        );
        assert!(logical
            .iter()
            .all(|w| prebindgen_flat::type_key(&w.kind) == "u64"));
    }
}
