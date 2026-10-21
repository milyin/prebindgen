//! A slot's wire tree: leaves own FFI values, parents describe how values fit together.
use crate::{Result, Wire, WireKind};

/// The authoritative wire order and grouping for one conversion slot.
///
/// Leaves are visited depth first, from left to right. This is the order used by
/// `ConversionExt::wires()` and position-based code requests, including wires for
/// inactive alternatives. Group metadata never adds an ABI parameter by itself.
/// Empty groups are allowed, for example for an enum variant with no payload.
///
/// Build or edit the tree before emitting code. Positions refer to the current
/// tree and must not be retained across structural edits. Emission borrows the
/// slot immutably, so all requests see the same order.
#[derive(Clone, Debug)]
pub struct WireLayout<W: WireKind, G = ()> {
    nodes: Vec<WireNode<W, G>>,
    // Derived cumulative leaf counts; no separately stored wires or metadata.
    ends: Vec<usize>,
}
/// Either one complete FFI value or a description of several child values.
#[derive(Clone, Debug)]
pub enum WireNode<W: WireKind, G> {
    /// One physical FFI value. All metadata about this wire belongs in `W`.
    Wire(Wire<W>),
    /// A record, option, variant or other value described by the slot's group type.
    Group {
        /// Information about the relationship between the child values.
        metadata: G,
        /// Children in ABI order. A physical wire is owned by exactly one leaf.
        children: WireLayout<W, G>,
    },
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
        let nodes: Vec<_> = iter.into_iter().map(WireNode::Wire).collect();
        let ends = (1..=nodes.len()).collect();
        Self { nodes, ends }
    }
}
impl<W: WireKind, G> WireLayout<W, G> {
    /// Build a tree from ordered leaf and group nodes.
    /// Leaf positions are computed from the children; metadata does not add wires.
    pub fn from_nodes(nodes: Vec<WireNode<W, G>>) -> Self {
        let mut count = 0;
        let ends = nodes
            .iter()
            .map(|node| {
                count += match node {
                    WireNode::Wire(_) => 1,
                    WireNode::Group { children, .. } => children.len(),
                };
                count
            })
            .collect();
        Self { nodes, ends }
    }
    /// Immediate children of this tree, including group metadata.
    pub fn nodes(&self) -> &[WireNode<W, G>] {
        &self.nodes
    }
    /// Append one physical wire during preparation.
    pub fn push(&mut self, wire: Wire<W>) {
        self.ends.push(self.len() + 1);
        self.nodes.push(WireNode::Wire(wire));
    }
    /// Put a physical wire before the current children, such as an option flag.
    pub fn prepend(&mut self, wire: Wire<W>) {
        for end in &mut self.ends {
            *end += 1;
        }
        self.ends.insert(0, 1);
        self.nodes.insert(0, WireNode::Wire(wire));
    }
    /// Append another tree without discarding its groups.
    pub fn extend(&mut self, other: Self) {
        let offset = self.len();
        self.ends
            .extend(other.ends.into_iter().map(|end| end + offset));
        self.nodes.extend(other.nodes);
    }
    /// Place the current tree inside a parent with the supplied metadata.
    pub fn enclose(&mut self, metadata: G) {
        let children = std::mem::take(self);
        self.ends.push(children.len());
        self.nodes.push(WireNode::Group { metadata, children });
    }
    /// Number of physical wires, including those belonging to inactive variants.
    pub fn len(&self) -> usize {
        self.ends.last().copied().unwrap_or(0)
    }
    /// Whether this tree contains no physical wires. It may still contain groups.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    /// Borrow all physical wires in ABI order; metadata remains accessible in `kind`.
    pub fn iter(&self) -> Wires<'_, W, G> {
        Wires {
            stack: vec![self.nodes.iter()],
            remaining: self.len(),
        }
    }
    /// Edit wire descriptions during preparation, without changing their order.
    pub fn iter_mut(&mut self) -> WiresMut<'_, W, G> {
        WiresMut {
            stack: vec![self.nodes.iter_mut()],
        }
    }
    /// Find the physical wire at a derived ABI position.
    pub fn get(&self, pos: usize) -> Option<&Wire<W>> {
        let index = self.ends.partition_point(|end| *end <= pos);
        match self.nodes.get(index)? {
            WireNode::Wire(wire) => Some(wire),
            WireNode::Group { children, .. } => {
                children.get(pos - if index == 0 { 0 } else { self.ends[index - 1] })
            }
        }
    }
    /// Check that each physical wire has a unique name within this slot.
    /// Group-specific rules are checked by the code interpreting `G`.
    pub fn validate(&self) -> Result<()> {
        let mut names = std::collections::BTreeSet::new();
        for wire in self.iter() {
            if !names.insert(wire.name.to_string()) {
                return Err(format!("duplicate wire identifier `{}`", wire.name));
            }
        }
        Ok(())
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
/// Borrowed physical wires in the tree's ABI order.
pub struct Wires<'a, W: WireKind, G> {
    stack: Vec<std::slice::Iter<'a, WireNode<W, G>>>,
    remaining: usize,
}
impl<'a, W: WireKind, G> Iterator for Wires<'a, W, G> {
    type Item = &'a Wire<W>;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.stack.last_mut()?.next() {
                Some(WireNode::Wire(w)) => {
                    self.remaining -= 1;
                    return Some(w);
                }
                Some(WireNode::Group { children, .. }) => self.stack.push(children.nodes.iter()),
                None => {
                    self.stack.pop();
                }
            }
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining, Some(self.remaining))
    }
}
impl<W: WireKind, G> ExactSizeIterator for Wires<'_, W, G> {}
impl<W: WireKind, G> std::iter::FusedIterator for Wires<'_, W, G> {}
/// Mutable access to leaf descriptions during preparation.
pub struct WiresMut<'a, W: WireKind, G> {
    stack: Vec<std::slice::IterMut<'a, WireNode<W, G>>>,
}
impl<'a, W: WireKind, G> Iterator for WiresMut<'a, W, G> {
    type Item = &'a mut Wire<W>;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.stack.last_mut()?.next() {
                Some(WireNode::Wire(w)) => return Some(w),
                Some(WireNode::Group { children, .. }) => {
                    self.stack.push(children.nodes.iter_mut())
                }
                None => {
                    self.stack.pop();
                }
            }
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
    #[derive(Debug, PartialEq)]
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
