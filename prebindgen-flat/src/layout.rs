//! Logical values exchanged by a conversion; transport belongs to its generator.
use crate::{Result, Wire, WireKind};

/// One payload/element schema. Its leaves may occur zero, one or many times.
#[derive(Clone, Debug)]
pub struct WireContainer<W: WireKind, G> {
    /// Definition of one present payload or sequence element.
    pub element: WireLayout<W, G>,
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
    pub layout: WireLayout<W, G>,
}
/// Logical structure. Common nodes impose no ABI positions or delivery rules;
/// adapters may retain explicit encoding decisions in their opaque metadata.
#[derive(Clone, Debug)]
pub enum WireNode<W: WireKind, G> {
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
        children: WireLayout<W, G>,
    },
    /// Exactly one alternative is selected; selector tags belong to the generator.
    /// Empty alternatives represent choices carrying no values.
    Alternatives {
        /// Adapter-specific selection information, without common placement rules.
        metadata: Option<G>,
        /// Mutually exclusive payload schemas in declaration order.
        alternatives: Vec<WireLayout<W, G>>,
    },
    /// Exactly one completion channel is selected. Empty payloads retain their
    /// outcome identity; Result<(), E> is distinct from an ordinary unit return.
    Outcomes {
        /// Available channels and their payload schemas.
        alternatives: Vec<WireOutcome<W, G>>,
    },
}
/// Logical input/output specification. A generator chooses physical signatures and
/// how values are packed, repeated, selected or delivered.
///
/// Iteration visits each leaf definition once, including every sequence element
/// schema and every alternative. It does not count runtime values or establish
/// native argument order. Use paths when branch/element context matters.
#[derive(Clone, Debug)]
pub struct WireLayout<W: WireKind, G = ()> {
    nodes: Vec<WireNode<W, G>>,
}
impl<W: WireKind, G> Default for WireLayout<W, G> {
    fn default() -> Self {
        Self { nodes: vec![] }
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
    /// Build an ordered logical schema.
    pub fn from_nodes(nodes: Vec<WireNode<W, G>>) -> Self {
        Self { nodes }
    }
    /// Immediate logical nodes, including branch and element structure.
    pub fn nodes(&self) -> &[WireNode<W, G>] {
        &self.nodes
    }
    /// Edit nodes and their adapter metadata in place. Structural edits invalidate paths.
    pub fn nodes_mut(&mut self) -> &mut [WireNode<W, G>] {
        &mut self.nodes
    }
    /// Consume the schema for structural editing.
    pub fn into_nodes(self) -> Vec<WireNode<W, G>> {
        self.nodes
    }
    /// Append a logical value definition.
    pub fn push(&mut self, wire: Wire<W>) {
        self.nodes.push(WireNode::Wire(wire));
    }
    /// Prepend a logical value definition; this chooses no ABI placement.
    pub fn prepend(&mut self, wire: Wire<W>) {
        self.nodes.insert(0, WireNode::Wire(wire));
    }
    /// Append definitions, preserving their structure.
    pub fn extend(&mut self, other: Self) {
        self.nodes.extend(other.nodes);
    }
    /// Enclose definitions in an adapter-described relationship.
    pub fn enclose(&mut self, metadata: G) {
        let children = std::mem::take(self);
        self.nodes.push(WireNode::Group { metadata, children });
    }
    /// Make these definitions the payload of an optional value.
    pub fn enclose_optional(&mut self, metadata: Option<G>) {
        let element = std::mem::take(self);
        self.nodes
            .push(WireNode::Optional(WireContainer { element, metadata }));
    }
    /// Make these definitions the schema of one sequence element.
    pub fn enclose_sequence(&mut self, metadata: Option<G>) {
        let element = std::mem::take(self);
        self.nodes
            .push(WireNode::Sequence(WireContainer { element, metadata }));
    }
    /// Add or replace a completion channel. A plain layout becomes Success.
    pub fn set_outcome(&mut self, kind: OutcomeKind, layout: Self) {
        if let [WireNode::Outcomes { alternatives }] = self.nodes.as_mut_slice() {
            if let Some(existing) = alternatives.iter_mut().find(|o| o.kind == kind) {
                existing.layout = layout;
            } else {
                alternatives.push(WireOutcome { kind, layout });
                alternatives.sort_by_key(|o| o.kind);
            }
        } else {
            let success = std::mem::take(self);
            let mut alternatives = vec![WireOutcome {
                kind: OutcomeKind::Success,
                layout: success,
            }];
            if kind == OutcomeKind::Success {
                alternatives[0].layout = layout;
            } else {
                alternatives.push(WireOutcome { kind, layout });
            }
            self.nodes.push(WireNode::Outcomes { alternatives });
        }
    }
    /// Replace the success payload, retaining any declared error channels.
    pub fn replace_success(&mut self, layout: Self) {
        if matches!(self.nodes.as_slice(), [WireNode::Outcomes { .. }]) {
            self.set_outcome(OutcomeKind::Success, layout);
        } else {
            *self = layout;
        }
    }
    /// Payload of a declared outcome; an ordinary layout implicitly means Success.
    pub fn outcome(&self, kind: OutcomeKind) -> Option<&Self> {
        if let [WireNode::Outcomes { alternatives }] = self.nodes.as_slice() {
            alternatives
                .iter()
                .find(|o| o.kind == kind)
                .map(|o| &o.layout)
        } else {
            (kind == OutcomeKind::Success
                && !self
                    .nodes
                    .iter()
                    .any(|node| matches!(node, WireNode::Outcomes { .. })))
            .then_some(self)
        }
    }
    /// Number of leaf definitions, visiting each element/alternative once.
    /// This is neither an ABI arity nor a runtime value count.
    pub fn len(&self) -> usize {
        self.iter().count()
    }
    /// Whether the schema has no leaf definitions; structural nodes may remain.
    pub fn is_empty(&self) -> bool {
        self.iter().next().is_none()
    }
    /// Visit all leaf definitions in structural order, without choosing transport.
    pub fn iter(&self) -> Wires<'_, W, G> {
        Wires {
            stack: vec![self.nodes.iter()],
        }
    }
    /// Edit leaf descriptions without changing structure.
    pub fn iter_mut(&mut self) -> WiresMut<'_, W, G> {
        WiresMut {
            stack: vec![self.nodes.iter_mut()],
        }
    }
    /// A leaf by structural traversal index, not by native argument position.
    pub fn get(&self, index: usize) -> Option<&Wire<W>> {
        self.iter().nth(index)
    }
    /// Definitions with their full structural addresses. A path includes child
    /// node indices and an alternative index before its children. Outcomes use
    /// stable channel codes: Success=0, SourceError=1, BindingFailure=2.
    pub fn entries(&self) -> impl Iterator<Item = WireEntry<'_, W>> {
        fn visit<'a, W: WireKind, G>(
            layout: &'a WireLayout<W, G>,
            prefix: &mut Vec<usize>,
            out: &mut Vec<WireEntry<'a, W>>,
        ) {
            for (index, node) in layout.nodes.iter().enumerate() {
                prefix.push(index);
                match node {
                    WireNode::Wire(wire) => out.push(WireEntry {
                        path: WirePath(prefix.clone()),
                        wire,
                    }),
                    WireNode::Optional(c) | WireNode::Sequence(c) => visit(&c.element, prefix, out),
                    WireNode::Group { children, .. } => visit(children, prefix, out),
                    WireNode::Alternatives { alternatives, .. } => {
                        for (arm, layout) in alternatives.iter().enumerate() {
                            prefix.push(arm);
                            visit(layout, prefix, out);
                            prefix.pop();
                        }
                    }
                    WireNode::Outcomes { alternatives } => {
                        for o in alternatives {
                            prefix.push(o.kind as usize);
                            visit(&o.layout, prefix, out);
                            prefix.pop();
                        }
                    }
                }
                prefix.pop();
            }
        }
        let mut out = vec![];
        visit(self, &mut vec![], &mut out);
        out.into_iter()
    }
    /// Resolve a structural leaf path, independently of its delivery mechanism.
    pub fn get_path(&self, path: &WirePath) -> Option<&Wire<W>> {
        fn get<'a, W: WireKind, G>(
            layout: &'a WireLayout<W, G>,
            path: &[usize],
        ) -> Option<&'a Wire<W>> {
            let (&index, rest) = path.split_first()?;
            match layout.nodes.get(index)? {
                WireNode::Wire(wire) => rest.is_empty().then_some(wire),
                WireNode::Optional(c) | WireNode::Sequence(c) => get(&c.element, rest),
                WireNode::Group { children, .. } => get(children, rest),
                WireNode::Alternatives { alternatives, .. } => {
                    let (&arm, rest) = rest.split_first()?;
                    get(alternatives.get(arm)?, rest)
                }
                WireNode::Outcomes { alternatives } => {
                    let (&channel, rest) = rest.split_first()?;
                    get(
                        &alternatives
                            .iter()
                            .find(|o| o.kind as usize == channel)?
                            .layout,
                        rest,
                    )
                }
            }
        }
        get(self, &path.0)
    }
    /// Check branch structure. Completion channels must form the sole root node,
    /// include Success, and have unique kinds. Names may repeat across contexts.
    pub fn validate(&self) -> Result<()> {
        fn walk<W: WireKind, G>(layout: &WireLayout<W, G>, root: bool) -> Result<()> {
            for node in layout.nodes() {
                match node {
                    WireNode::Wire(_) => {}
                    WireNode::Optional(c) | WireNode::Sequence(c) => walk(&c.element, false)?,
                    WireNode::Group { children, .. } => walk(children, false)?,
                    WireNode::Alternatives { alternatives, .. } => {
                        if alternatives.is_empty() {
                            return Err("selection has no alternatives".into());
                        }
                        for arm in alternatives {
                            walk(arm, false)?;
                        }
                    }
                    WireNode::Outcomes { alternatives } => {
                        if !root || layout.nodes.len() != 1 {
                            return Err("outcomes must be the sole root node".into());
                        }
                        let mut kinds = std::collections::BTreeSet::new();
                        for o in alternatives {
                            if !kinds.insert(o.kind) {
                                return Err("duplicate outcome kind".into());
                            }
                        }
                        if !kinds.contains(&OutcomeKind::Success) {
                            return Err("outcomes require a Success channel".into());
                        }
                        for o in alternatives {
                            walk(&o.layout, false)?;
                        }
                    }
                }
            }
            Ok(())
        }
        walk(self, true)
    }
}
/// Address of a definition, not a runtime instance or an ABI position. Rebuild
/// paths after structural edits; a generator supplies any element/branch context.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct WirePath(pub Vec<usize>);
/// A logical value definition paired with its structural context.
pub struct WireEntry<'a, W: WireKind> {
    /// Structural address, including element and outcome boundaries.
    pub path: WirePath,
    /// The leaf definition at that address.
    pub wire: &'a Wire<W>,
}
impl<W: WireKind, G> std::ops::Index<usize> for WireLayout<W, G> {
    type Output = Wire<W>;
    fn index(&self, index: usize) -> &Self::Output {
        self.get(index).expect("logical wire index out of range")
    }
}
impl<W: WireKind, G> std::ops::IndexMut<usize> for WireLayout<W, G> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.iter_mut()
            .nth(index)
            .expect("logical wire index out of range")
    }
}
/// Structural leaf traversal; sequence schemas and alternatives are each visited once.
pub struct Wires<'a, W: WireKind, G> {
    stack: Vec<std::slice::Iter<'a, WireNode<W, G>>>,
}
impl<'a, W: WireKind, G> Iterator for Wires<'a, W, G> {
    type Item = &'a Wire<W>;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.stack.last_mut()?.next() {
                Some(WireNode::Wire(wire)) => return Some(wire),
                Some(WireNode::Optional(c) | WireNode::Sequence(c)) => {
                    self.stack.push(c.element.nodes.iter())
                }
                Some(WireNode::Group { children, .. }) => self.stack.push(children.nodes.iter()),
                Some(WireNode::Alternatives { alternatives, .. }) => {
                    for arm in alternatives.iter().rev() {
                        self.stack.push(arm.nodes.iter());
                    }
                }
                Some(WireNode::Outcomes { alternatives }) => {
                    for o in alternatives.iter().rev() {
                        self.stack.push(o.layout.nodes.iter());
                    }
                }
                None => {
                    self.stack.pop();
                }
            }
        }
    }
}
impl<W: WireKind, G> std::iter::FusedIterator for Wires<'_, W, G> {}
/// Mutable structural leaf traversal, including element and outcome definitions.
pub struct WiresMut<'a, W: WireKind, G> {
    stack: Vec<std::slice::IterMut<'a, WireNode<W, G>>>,
}
impl<'a, W: WireKind, G> Iterator for WiresMut<'a, W, G> {
    type Item = &'a mut Wire<W>;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.stack.last_mut()?.next() {
                Some(WireNode::Wire(wire)) => return Some(wire),
                Some(WireNode::Optional(c) | WireNode::Sequence(c)) => {
                    self.stack.push(c.element.nodes.iter_mut())
                }
                Some(WireNode::Group { children, .. }) => {
                    self.stack.push(children.nodes.iter_mut())
                }
                Some(WireNode::Alternatives { alternatives, .. }) => {
                    for arm in alternatives.iter_mut().rev() {
                        self.stack.push(arm.nodes.iter_mut());
                    }
                }
                Some(WireNode::Outcomes { alternatives }) => {
                    for o in alternatives.iter_mut().rev() {
                        self.stack.push(o.layout.nodes.iter_mut());
                    }
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
    fn leaf(name: &str) -> Wire<syn::Type> {
        Wire {
            name: syn::parse_str(name).unwrap(),
            kind: syn::parse_quote!(i64),
        }
    }
    #[test]
    fn repetitions_and_outcomes_are_definitions_with_distinct_paths() {
        let mut payload: WireLayout<_, ()> = vec![leaf("value"), leaf("other")].into();
        payload.enclose_optional(None);
        payload.enclose_sequence(None);
        payload.enclose_optional(None);
        assert_eq!(payload.len(), 2);
        payload.set_outcome(OutcomeKind::SourceError, vec![leaf("value")].into());
        payload.set_outcome(OutcomeKind::BindingFailure, vec![leaf("message")].into());
        payload.validate().unwrap();
        assert_eq!(payload.len(), 4);
        let entries: Vec<_> = payload.entries().collect();
        assert_eq!(entries[0].wire.name, "value");
        assert_eq!(entries[2].wire.name, "value");
        assert_ne!(entries[0].path, entries[2].path);
        for entry in &entries {
            assert!(std::ptr::eq(
                entry.wire,
                payload.get_path(&entry.path).unwrap()
            ));
        }
        let paths: Vec<_> = entries.iter().map(|e| e.path.clone()).collect();
        for wire in payload.iter_mut() {
            wire.kind = syn::parse_quote!(u64);
        }
        for path in &paths {
            assert_eq!(
                crate::type_key(&payload.get_path(path).unwrap().kind),
                "u64"
            );
        }
        assert!(payload.get_path(&WirePath(vec![99])).is_none());
        assert_eq!(payload.outcome(OutcomeKind::Success).unwrap().len(), 2);
        assert_eq!(payload.outcome(OutcomeKind::SourceError).unwrap().len(), 1);
    }
    #[test]
    fn empty_outcomes_and_optional_unit_keep_their_structure() {
        let mut unit: WireLayout<syn::Type, ()> = WireLayout::default();
        unit.enclose_optional(None);
        assert!(unit.is_empty());
        assert!(matches!(unit.nodes(), [WireNode::Optional(_)]));
        unit.set_outcome(OutcomeKind::SourceError, WireLayout::default());
        assert!(unit.outcome(OutcomeKind::Success).is_some());
        assert!(unit.outcome(OutcomeKind::SourceError).is_some());
        assert!(unit.outcome(OutcomeKind::BindingFailure).is_none());
        let duplicate = WireLayout::from_nodes(vec![WireNode::Outcomes {
            alternatives: vec![
                WireOutcome {
                    kind: OutcomeKind::Success,
                    layout: unit.clone(),
                },
                WireOutcome {
                    kind: OutcomeKind::Success,
                    layout: unit,
                },
            ],
        }]);
        assert!(duplicate
            .validate()
            .unwrap_err()
            .contains("duplicate outcome"));
        let mut iter = duplicate.iter();
        assert!(iter.next().is_none());
        assert!(iter.next().is_none());
    }
    #[test]
    fn alternatives_preserve_empty_arms_and_outcome_paths_use_channel_identity() {
        let choice = WireLayout::from_nodes(vec![WireNode::Alternatives {
            metadata: None,
            alternatives: vec![WireLayout::default(), vec![leaf("value")].into()],
        }]);
        let mut output: WireLayout<_, ()> = choice;
        output.validate().unwrap();
        let entry = output.entries().next().unwrap();
        assert_eq!(entry.path, WirePath(vec![0, 1, 0]));
        assert!(std::ptr::eq(
            entry.wire,
            output.get_path(&entry.path).unwrap()
        ));
        output.set_outcome(OutcomeKind::BindingFailure, vec![leaf("message")].into());
        let path = output.entries().last().unwrap().path;
        output.set_outcome(OutcomeKind::SourceError, vec![leaf("error")].into());
        assert_eq!(output.get_path(&path).unwrap().name, "message");
        output.validate().unwrap();
        for wire in output.iter_mut() {
            wire.kind = syn::parse_quote!(u64);
        }
        assert_eq!(output.iter().count(), 3);
        assert_eq!(
            crate::type_key(&output.get_path(&path).unwrap().kind),
            "u64"
        );

        let mut nested = output.clone();
        nested.enclose_optional(None);
        assert!(nested.validate().unwrap_err().contains("sole root"));
        let mut mixed = output;
        mixed.push(leaf("extra"));
        assert!(mixed.validate().unwrap_err().contains("sole root"));
        assert!(mixed.outcome(OutcomeKind::Success).is_none());
        let missing_success: WireLayout<_, ()> = WireLayout::from_nodes(vec![WireNode::Outcomes {
            alternatives: vec![WireOutcome {
                kind: OutcomeKind::SourceError,
                layout: vec![leaf("error")].into(),
            }],
        }]);
        assert!(missing_success.validate().unwrap_err().contains("Success"));
    }
}
