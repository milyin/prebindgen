//! A slot's authoritative tree of physical wires and the values they represent.
use crate::{Result, Wire, WireKind};

/// Explicit boundary representation, independent of optional/sequence cardinality.
/// Each physical wire is stored exactly once in the tree.
#[derive(Clone, Debug)]
pub enum WireRepresentation<W: WireKind> {
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

/// Common structure for an optional value or a sequence.
#[derive(Clone, Debug)]
pub struct WireContainer<W: WireKind, G> {
    /// Explicit boundary representation and wire scope.
    pub representation: WireRepresentation<W>,
    /// One present payload/sequence element. Control wires are kept separately.
    pub element: WireLayout<W, G>,
    /// Adapter-specific information; cardinality and scope are defined above it.
    pub metadata: Option<G>,
}
impl<W: WireKind, G> WireContainer<W, G> {
    /// Physical wires occupied in the enclosing scope.
    pub fn wire_count(&self) -> usize {
        match &self.representation {
            WireRepresentation::Inline { controls } => controls.len() + self.element.len(),
            WireRepresentation::Carrier(_) => 1,
        }
    }
    /// Carrier when element fields have their own scope.
    pub fn carrier(&self) -> Option<&Wire<W>> {
        match &self.representation {
            WireRepresentation::Carrier(wire) => Some(wire),
            WireRepresentation::Inline { .. } => None,
        }
    }
}

/// One FFI value, a container, or a relationship between child values.
#[derive(Clone, Debug)]
pub enum WireNode<W: WireKind, G> {
    /// One physical FFI value. Its metadata belongs in `W`.
    Wire(Wire<W>),
    /// Zero or one occurrence of the element layout.
    Optional(WireContainer<W, G>),
    /// Zero or more occurrences of the element layout.
    Sequence(WireContainer<W, G>),
    /// Adapter-described relationships such as records and named fields.
    /// Group metadata adds no physical wires by itself.
    Group {
        /// Information about the relationship between child values.
        metadata: G,
        /// Child values in ABI order.
        children: WireLayout<W, G>,
    },
}
impl<W: WireKind, G> WireNode<W, G> {
    /// Physical wires occupied in the enclosing scope.
    pub fn wire_count(&self) -> usize {
        match self {
            Self::Wire(_) => 1,
            Self::Optional(c) | Self::Sequence(c) => c.wire_count(),
            Self::Group { children, .. } => children.len(),
        }
    }
}

/// Authoritative wire order and structure for one conversion slot.
///
/// Wires are visited depth first, left to right. Inline containers expose their
/// control wires then their element wires. Carried containers expose only the
/// carrier; their element layouts have independent positions. This order also
/// defines `ConversionExt::wires()` and position-based code requests, including
/// inactive alternatives. Empty groups and inline elements are allowed.
///
/// Build/edit the tree before emission; positions must not survive structural
/// edits. Emission borrows it immutably so every request sees the same order.
#[derive(Clone, Debug)]
pub struct WireLayout<W: WireKind, G = ()> {
    nodes: Vec<WireNode<W, G>>,
    // Derived counts, not a second source of wires or metadata.
    ends: Vec<usize>,
}
impl<W: WireKind, G> Default for WireLayout<W, G> {
    fn default() -> Self {
        Self {
            nodes: vec![],
            ends: vec![],
        }
    }
}
impl<W: WireKind, G> From<Vec<Wire<W>>> for WireLayout<W, G> {
    fn from(wires: Vec<Wire<W>>) -> Self {
        wires.into_iter().collect()
    }
}
impl<W: WireKind, G> FromIterator<Wire<W>> for WireLayout<W, G> {
    fn from_iter<T: IntoIterator<Item = Wire<W>>>(iter: T) -> Self {
        Self::from_nodes(iter.into_iter().map(WireNode::Wire).collect())
    }
}
impl<W: WireKind, G> WireLayout<W, G> {
    /// Build a tree, deriving positions from its explicit representations.
    pub fn from_nodes(nodes: Vec<WireNode<W, G>>) -> Self {
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
    pub fn into_nodes(self) -> Vec<WireNode<W, G>> {
        self.nodes
    }
    /// Immediate child nodes, including container structure and metadata.
    pub fn nodes(&self) -> &[WireNode<W, G>] {
        &self.nodes
    }
    /// Append one physical wire during preparation.
    pub fn push(&mut self, wire: Wire<W>) {
        self.ends.push(self.len() + 1);
        self.nodes.push(WireNode::Wire(wire));
    }
    /// Put one physical wire before the current children.
    pub fn prepend(&mut self, wire: Wire<W>) {
        for end in &mut self.ends {
            *end += 1;
        }
        self.ends.insert(0, 1);
        self.nodes.insert(0, WireNode::Wire(wire));
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
        *self = Self::from_nodes(vec![WireNode::Group { metadata, children }]);
    }
    /// Make the current tree an optional payload with an explicit representation.
    pub fn enclose_optional(&mut self, representation: WireRepresentation<W>, metadata: Option<G>) {
        let element = std::mem::take(self);
        *self = Self::from_nodes(vec![WireNode::Optional(WireContainer {
            representation,
            element,
            metadata,
        })]);
    }
    /// Make the current tree a sequence element with an explicit representation.
    pub fn enclose_sequence(&mut self, representation: WireRepresentation<W>, metadata: Option<G>) {
        let element = std::mem::take(self);
        *self = Self::from_nodes(vec![WireNode::Sequence(WireContainer {
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
    pub fn iter(&self) -> Wires<'_, W, G> {
        Wires {
            stack: vec![Scope::Nodes(self.nodes.iter())],
            remaining: self.len(),
        }
    }
    /// Edit physical wires without changing their order or structure.
    pub fn iter_mut(&mut self) -> WiresMut<'_, W, G> {
        WiresMut {
            stack: vec![ScopeMut::Nodes(self.nodes.iter_mut())],
        }
    }
    /// Find one physical wire at its derived position in this scope.
    pub fn get(&self, pos: usize) -> Option<&Wire<W>> {
        let index = self.ends.partition_point(|end| *end <= pos);
        let local = pos - if index == 0 { 0 } else { self.ends[index - 1] };
        match self.nodes.get(index)? {
            WireNode::Wire(wire) => Some(wire),
            WireNode::Group { children, .. } => children.get(local),
            WireNode::Optional(c) | WireNode::Sequence(c) => match &c.representation {
                WireRepresentation::Carrier(wire) => Some(wire),
                WireRepresentation::Inline { controls } => {
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
            layout: &WireLayout<W, G>,
            names: &mut std::collections::BTreeSet<String>,
        ) -> Result<()> {
            for node in layout.nodes() {
                match node {
                    WireNode::Wire(wire) => insert(wire, names)?,
                    WireNode::Group { children, .. } => walk(children, names)?,
                    WireNode::Optional(c) | WireNode::Sequence(c) => {
                        match &c.representation {
                            WireRepresentation::Inline { controls } => {
                                for wire in controls {
                                    insert(wire, names)?;
                                }
                            }
                            WireRepresentation::Carrier(wire) => insert(wire, names)?,
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
impl<W: WireKind, G> std::ops::Index<usize> for WireLayout<W, G> {
    type Output = Wire<W>;
    fn index(&self, pos: usize) -> &Self::Output {
        self.get(pos).expect("wire position out of range")
    }
}
impl<W: WireKind, G> std::ops::IndexMut<usize> for WireLayout<W, G> {
    fn index_mut(&mut self, pos: usize) -> &mut Self::Output {
        self.iter_mut()
            .nth(pos)
            .expect("wire position out of range")
    }
}
enum Scope<'a, W: WireKind, G> {
    Nodes(std::slice::Iter<'a, WireNode<W, G>>),
    Wires(std::slice::Iter<'a, Wire<W>>),
}
/// Borrowed physical wires in the enclosing scope's ABI order.
pub struct Wires<'a, W: WireKind, G> {
    stack: Vec<Scope<'a, W, G>>,
    remaining: usize,
}
impl<'a, W: WireKind, G> Iterator for Wires<'a, W, G> {
    type Item = &'a Wire<W>;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.stack.last_mut()? {
                Scope::Wires(wires) => {
                    if let Some(wire) = wires.next() {
                        self.remaining -= 1;
                        return Some(wire);
                    }
                }
                Scope::Nodes(nodes) => match nodes.next() {
                    Some(WireNode::Wire(wire)) => {
                        self.remaining -= 1;
                        return Some(wire);
                    }
                    Some(WireNode::Group { children, .. }) => {
                        self.stack.push(Scope::Nodes(children.nodes.iter()));
                        continue;
                    }
                    Some(WireNode::Optional(c) | WireNode::Sequence(c)) => {
                        match &c.representation {
                            WireRepresentation::Carrier(wire) => {
                                self.remaining -= 1;
                                return Some(wire);
                            }
                            WireRepresentation::Inline { controls } => {
                                self.stack.push(Scope::Nodes(c.element.nodes.iter()));
                                self.stack.push(Scope::Wires(controls.iter()));
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
impl<W: WireKind, G> ExactSizeIterator for Wires<'_, W, G> {}
impl<W: WireKind, G> std::iter::FusedIterator for Wires<'_, W, G> {}
enum ScopeMut<'a, W: WireKind, G> {
    Nodes(std::slice::IterMut<'a, WireNode<W, G>>),
    Wires(std::slice::IterMut<'a, Wire<W>>),
}
/// Mutable physical wires in the enclosing scope; independent element scopes are excluded.
pub struct WiresMut<'a, W: WireKind, G> {
    stack: Vec<ScopeMut<'a, W, G>>,
}
impl<'a, W: WireKind, G> Iterator for WiresMut<'a, W, G> {
    type Item = &'a mut Wire<W>;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.stack.last_mut()? {
                ScopeMut::Wires(wires) => {
                    if let Some(wire) = wires.next() {
                        return Some(wire);
                    }
                }
                ScopeMut::Nodes(nodes) => match nodes.next() {
                    Some(WireNode::Wire(wire)) => return Some(wire),
                    Some(WireNode::Group { children, .. }) => {
                        self.stack.push(ScopeMut::Nodes(children.nodes.iter_mut()));
                        continue;
                    }
                    Some(WireNode::Optional(c) | WireNode::Sequence(c)) => {
                        match &mut c.representation {
                            WireRepresentation::Carrier(wire) => return Some(wire),
                            WireRepresentation::Inline { controls } => {
                                self.stack.push(ScopeMut::Nodes(c.element.nodes.iter_mut()));
                                self.stack.push(ScopeMut::Wires(controls.iter_mut()));
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
impl<'a, W: WireKind, G> IntoIterator for &'a WireLayout<W, G> {
    type Item = &'a Wire<W>;
    type IntoIter = Wires<'a, W, G>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
impl<'a, W: WireKind, G> IntoIterator for &'a mut WireLayout<W, G> {
    type Item = &'a mut Wire<W>;
    type IntoIter = WiresMut<'a, W, G>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
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
                let mut payload: WireLayout<_, Meaning> = vec![leaf("x"), leaf("y")].into();
                payload.enclose(Meaning::Record);
                let representation = if inline {
                    WireRepresentation::Inline {
                        controls: vec![leaf("present")],
                    }
                } else {
                    WireRepresentation::Carrier(leaf("array"))
                };
                if optional {
                    payload.enclose_optional(representation, Some(Meaning::Choice));
                } else {
                    payload.enclose_sequence(representation, Some(Meaning::Choice));
                }
                let mut tree: WireLayout<_, Meaning> = vec![leaf("before")].into();
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
                let (WireNode::Optional(c) | WireNode::Sequence(c)) = &tree.nodes()[1] else {
                    panic!()
                };
                assert_eq!(c.metadata, Some(Meaning::Choice));
                assert_eq!(c.element.len(), 2);
                // Mutating outer wires never descends into an independent scope.
                assert_eq!(
                    crate::type_key(&c.element[0].kind),
                    if inline { "u64" } else { "i32" }
                );
            }
        }
    }
    #[test]
    fn nested_optional_sequences_keep_flags_outside_element_schemas() {
        let mut element: WireLayout<_, ()> = vec![leaf("value")].into();
        element.enclose_optional(
            WireRepresentation::Inline {
                controls: vec![leaf("item_present")],
            },
            None,
        );
        element.enclose_sequence(WireRepresentation::Carrier(leaf("rows")), None);
        element.enclose_optional(
            WireRepresentation::Inline {
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
        let WireNode::Optional(outer) = &element.nodes()[0] else {
            panic!()
        };
        let WireNode::Sequence(sequence) = &outer.element.nodes()[0] else {
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
        let WireNode::Optional(item) = &sequence.element.nodes()[0] else {
            panic!()
        };
        assert_eq!(item.element.len(), 1);
        assert_eq!(item.element[0].name, "value");
        let duplicate: WireLayout<_, ()> =
            WireLayout::from_nodes(vec![WireNode::Optional(WireContainer {
                representation: WireRepresentation::Inline {
                    controls: vec![leaf("value")],
                },
                element: vec![leaf("value")].into(),
                metadata: None,
            })]);
        assert!(duplicate.validate().unwrap_err().contains("duplicate wire"));
        let mut empty: WireLayout<syn::Type, ()> = WireLayout::default();
        empty.enclose_optional(WireRepresentation::Inline { controls: vec![] }, None);
        assert_eq!(empty.len(), 0);
        assert!(empty.iter_mut().next().is_none());
        empty.enclose_optional(
            WireRepresentation::Inline {
                controls: vec![leaf("flag")],
            },
            None,
        );
        assert_eq!(empty.len(), 1);
        assert_eq!(empty[0].name, "flag");
    }
    #[test]
    fn sequence_is_one_outer_position_with_an_independent_element_schema() {
        let mut rows: WireLayout<_, Meaning> = vec![leaf("x"), leaf("y")].into();
        rows.enclose(Meaning::Record);
        let mut layout = WireLayout::from_nodes(vec![
            WireNode::Wire(leaf("before")),
            WireNode::Sequence(WireContainer {
                representation: WireRepresentation::Carrier(leaf("rows")),
                element: rows,
                metadata: None,
            }),
            WireNode::Wire(leaf("after")),
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
        let WireNode::Sequence(container) = &layout.nodes()[1] else {
            panic!()
        };
        let carrier = container.carrier().unwrap();
        let element = &container.element;
        assert_eq!(crate::type_key(&carrier.kind), "* const u8");
        assert_eq!(element.len(), 2);
        let WireNode::Group { metadata, .. } = &element.nodes()[0] else {
            panic!()
        };
        assert_eq!(metadata, &Meaning::Record);
        let duplicate = WireLayout::from_nodes(vec![WireNode::Sequence(WireContainer {
            representation: WireRepresentation::Carrier(leaf("x")),
            element: element.clone(),
            metadata: None,
        })]);
        assert!(duplicate.validate().unwrap_err().contains("duplicate wire"));
    }
    #[test]
    fn variants_empty_groups_and_records_have_one_physical_leaf_order() {
        let mut empty = WireLayout::default();
        empty.enclose(Meaning::Variant("Empty"));
        let mut point: WireLayout<_, _> = vec![leaf("x"), leaf("y")].into();
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
        let WireNode::Group { metadata, children } = &variants.nodes()[1] else {
            panic!()
        };
        assert_eq!(metadata, &Meaning::Choice);
        let WireNode::Group { metadata, children } = &children.nodes()[0] else {
            panic!()
        };
        assert_eq!(metadata, &Meaning::Variant("Empty"));
        assert!(children.is_empty());
        variants.iter_mut().nth(1).unwrap().kind = syn::parse_quote!(u64);
        assert_eq!(crate::type_key(&variants.get(1).unwrap().kind), "u64");
    }
    #[test]
    fn indexing_skips_empty_nodes_before_between_and_after_leaves() {
        let empty = || WireNode::Group {
            metadata: (),
            children: WireLayout::default(),
        };
        let nested = WireLayout::from_nodes(vec![
            empty(),
            WireNode::Wire(leaf("a")),
            empty(),
            WireNode::Wire(leaf("b")),
            empty(),
        ]);
        let mut tree = WireLayout::from_nodes(vec![
            empty(),
            WireNode::Group {
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
        let mut a: WireLayout<_, ()> = vec![leaf("value")].into();
        a.enclose(());
        let mut b: WireLayout<_, ()> = vec![leaf("value")].into();
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
}
