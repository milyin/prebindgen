//! The into-Rust direction of the shared transformation tree (#442): how the
//! values that cross in are put back together into one Rust value.
//!
//! The tree is the plan. A constructor call is a [product](InProduct) over its
//! arguments, a selector dispatch is a [choice](InChoice) over its arms, the
//! `Option<…>` and `Vec<…>` a parameter is written with are the layers over
//! that ([`InPresence`], [`InSlot`]), and a wire value is a [leaf](InLeaf).
//! Recursion is the tree's, so a nested build is the same node kinds as the
//! top-level one rather than a second type spelling the same thing.
//!
//! Every wire slot is described exactly once, by the node that uses it — a
//! constructor argument, a dispatch's selector, a layer's presence flag, the
//! payload an `Option` layer unwraps. [`FoldPlan::leaves`] is
//! [collected](wire_leaves) from them, and [`FoldPlan::shape`] is read back off
//! the layer nodes.
//!
//! [`FoldPlan::leaves`]: super::FoldPlan::leaves
//! [`FoldPlan::shape`]: super::FoldPlan::shape

use super::{FoldLeaf, FoldShape};
use crate::transform::{
    Lowered, TransformChild, TransformDirection, TransformKind, TransformLowerer, TransformNode,
};

/// Direction marker: crossing values assembled into a Rust value.
pub enum IntoRust {}

impl TransformDirection for IntoRust {
    type Leaf = InLeaf;
    type Product = InProduct;
    type Choice = InChoice;
    type Optional = InPresence;
    /// The run's own wire slot: one value carrying the whole collection, which
    /// the layer iterates.
    type Sequence = InSlot;
    type Link = InLink;
}

/// One node of an into-Rust construction.
pub type InNode = TransformNode<IntoRust>;
/// One argument of an into-Rust product, or one arm of a choice.
pub type InChild = TransformChild<IntoRust>;

/// One wire slot: where it sits in the foreign signature, what it is called
/// there, and what it carries.
///
/// The single description of a slot. [`wire_leaves`] turns the slots a tree
/// names into [`FoldPlan::leaves`](super::FoldPlan::leaves), so a slot exists
/// exactly where the node that uses it says so.
#[derive(Clone)]
pub struct InSlot {
    /// Position in the foreign signature, and the index of the caller's
    /// decoded local.
    pub slot: usize,
    /// The slot's foreign-side parameter name.
    pub name: syn::Ident,
    /// What the slot carries. A **reading**: spell it with `emit.spell(&ty)`
    /// in an emission callback.
    pub ty: prebindgen_flat::flat::TypeRef,
}

/// Where one decoded value comes from.
// large_enum_variant: a plan has a handful of leaves, and boxing `InSlot` to
// even the arms out would only put an indirection between a node and the slot
// it names (same trade-off as `DeconRecord`).
#[allow(clippy::large_enum_variant)]
pub enum InLeaf {
    /// A wire slot of its own.
    Slot {
        slot: InSlot,
        /// The slot is `Option`-wrapped by **selector presence** (it belongs to
        /// a dispatched arm and only the taken arm's slots are filled), so a
        /// consumer unwraps it before use and treats a missing value as an
        /// error.
        ///
        /// `false` both outside a dispatch and for a *passthrough* argument —
        /// one the constructor itself declares `Option<…>`, which keeps its own
        /// type on the wire because `None` is a legitimate value for the taken
        /// arm and the wire cannot carry the double `Option` anyway.
        wrapped: bool,
    },
    /// The value the enclosing layer unwrapped and bound: a single-argument
    /// construction under an [`Option`](InPresence::Payload) layer or a run
    /// takes what that layer produced, and the slot belongs to the layer rather
    /// than to this argument.
    Bound,
}

/// How a product node's children combine into its value.
pub enum InProduct {
    /// Call this constructor on the children, in parameter order.
    Ctor {
        func: syn::Ident,
        /// The constructor returns `Result`; its `Err` is routed through the
        /// adapter's error channel.
        fallible: bool,
    },
    /// The value itself, decoded from the node's single child.
    Identity {
        /// A borrowed identity arm (`&T` parameter): the decoded value is a
        /// borrow, and the fold clones it (`T: Clone`) so the caller's handle
        /// is preserved rather than consumed.
        clone: bool,
    },
}

/// How a choice node picks the arm that runs. The selector is a slot of its
/// own — a wire value no source wrote, contributed by this node.
pub struct InChoice {
    /// The `i32` slot. Arm `i` is taken when it reads `i`; under an
    /// [`Optional`](InPresence) layer `-1` additionally means absent.
    pub selector: InSlot,
}

/// How an `Option<…>` layer decides whether its value is present. The three
/// forms differ in what crosses, which is why the layer names its own slot
/// rather than inheriting one.
pub enum InPresence {
    /// The dispatch under the layer also encodes absence: its selector reads
    /// `-1`. No slot of the layer's own.
    Selector,
    /// An explicit `bool` slot decides, and the construction's arguments cross
    /// plain. Used for a constructor of two or more arguments, where riding the
    /// arguments' own `Option`s would box a nullable primitive on the wire.
    Flag(InSlot),
    /// The layer decodes its own `Option<…>` slot and hands the payload to the
    /// single-argument construction under it — which reads it as
    /// [`InLeaf::Bound`], having no slot of its own.
    Payload(InSlot),
}

/// How one child hangs off its parent.
pub struct InLink {
    /// The consuming constructor parameter is `&T`, so the child's value is
    /// borrowed at the call site. Always `false` for a leaf child, whose
    /// decoded local already has the parameter's type, and for a choice arm,
    /// which is not consumed by anything.
    pub by_ref: bool,
}

impl InNode {
    /// The construction under any arity layers — what the `Option` / `Vec` a
    /// parameter is written with wraps.
    pub fn core(&self) -> &InNode {
        match &self.kind {
            TransformKind::Optional { inner, .. } | TransformKind::Sequence { inner, .. } => {
                inner.core()
            }
            _ => self,
        }
    }

    /// The arity layers this node wraps its construction in — the plan's
    /// [`shape`](super::FoldPlan::shape), read back off the nodes that carry it.
    pub fn shape(&self) -> FoldShape {
        match &self.kind {
            TransformKind::Optional { inner, .. } => FoldShape::optional((), inner.shape()),
            TransformKind::Sequence { inner, .. } => FoldShape::iterable(inner.shape()),
            _ => FoldShape::Base,
        }
    }

    /// The slot of the explicit presence flag, when an `Option` layer decides
    /// presence with one — see [`InPresence::Flag`].
    pub fn present(&self) -> Option<usize> {
        match &self.kind {
            TransformKind::Optional {
                op: InPresence::Flag(s),
                ..
            } => Some(s.slot),
            TransformKind::Optional { inner, .. } | TransformKind::Sequence { inner, .. } => {
                inner.present()
            }
            _ => None,
        }
    }

    /// The selector slot when the construction dispatches, `None` when it is a
    /// single unconditional one.
    pub fn selector(&self) -> Option<usize> {
        match &self.core().kind {
            TransformKind::Choice { op, .. } => Some(op.selector.slot),
            _ => None,
        }
    }

    /// The arms of a dispatch, or the single construction on its own — the
    /// reading a consumer wants when it asks what this parameter can be built
    /// from. In selector order: arm `i` is taken when the selector reads `i`.
    pub fn arms(&self) -> Vec<&InNode> {
        let core = self.core();
        match &core.kind {
            TransformKind::Choice { variants, .. } => variants.iter().map(|v| &v.node).collect(),
            _ => vec![core],
        }
    }

    /// The constructor an [arm](Self::arms) calls; `None` for the identity arm,
    /// which passes an existing value through rather than building one.
    pub fn ctor(&self) -> Option<&syn::Ident> {
        match &self.kind {
            TransformKind::Product {
                op: InProduct::Ctor { func, .. },
                ..
            } => Some(func),
            _ => None,
        }
    }

    /// Which slots of [`FoldPlan::leaves`](super::FoldPlan::leaves) an
    /// [arm](Self::arms)'s arguments decode, in parameter order.
    ///
    /// `None` when any argument is itself built from further slots, or reads a
    /// layer's payload instead of a slot: the arm then has no flat signature,
    /// which is what makes it unsplittable into a destination-language
    /// overload.
    pub fn leaf_args(&self) -> Option<Vec<usize>> {
        let TransformKind::Product { children, .. } = &self.kind else {
            return None;
        };
        children
            .iter()
            .map(|c| match &c.node.kind {
                TransformKind::Leaf(InLeaf::Slot { slot, .. }) => Some(slot.slot),
                _ => None,
            })
            .collect()
    }
}

/// Collect the wire slots a plan uses, in slot order — the flat signature
/// [`FoldPlan::leaves`](super::FoldPlan::leaves) exposes.
///
/// Every slot is named exactly once — a gap or a collision here means the
/// builder handed out a position twice.
pub fn wire_leaves(tree: &InNode) -> Vec<FoldLeaf> {
    let mut slots: Vec<(usize, FoldLeaf)> = Vec::new();
    tree.lower(&mut CollectSlots(&mut slots))
        .expect("collecting wire slots cannot fail");
    let mut out: Vec<Option<FoldLeaf>> = (0..slots.len()).map(|_| None).collect();
    for (slot, leaf) in slots {
        let cell = out
            .get_mut(slot)
            .expect("a wire slot outside the plan's own count");
        assert!(cell.is_none(), "two wire values claim slot {slot}");
        *cell = Some(leaf);
    }
    out.into_iter()
        .enumerate()
        .map(|(slot, leaf)| leaf.unwrap_or_else(|| panic!("wire slot {slot} is unclaimed")))
        .collect()
}

/// The lowerer behind [`wire_leaves`]: each node contributes the slots it uses
/// and nothing else.
struct CollectSlots<'a>(&'a mut Vec<(usize, FoldLeaf)>);

impl CollectSlots<'_> {
    fn take(&mut self, s: &InSlot) {
        self.0.push((
            s.slot,
            FoldLeaf {
                name: s.name.clone(),
                ty: s.ty.clone(),
            },
        ));
    }
}

impl TransformLowerer<IntoRust> for CollectSlots<'_> {
    type Value = ();
    type Error = std::convert::Infallible;

    fn leaf(&mut self, _node: &InNode, op: &InLeaf) -> Result<(), Self::Error> {
        // A bound argument reads what its layer unwrapped; that slot is the
        // layer's, contributed there.
        if let InLeaf::Slot { slot, .. } = op {
            self.take(slot);
        }
        Ok(())
    }

    fn product(
        &mut self,
        _node: &InNode,
        _op: &InProduct,
        _children: Lowered<'_, IntoRust, ()>,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    fn choice(
        &mut self,
        _node: &InNode,
        op: &InChoice,
        _variants: Lowered<'_, IntoRust, ()>,
    ) -> Result<(), Self::Error> {
        self.take(&op.selector);
        Ok(())
    }

    fn optional(
        &mut self,
        _node: &InNode,
        op: &InPresence,
        _inner: &InNode,
        _value: (),
    ) -> Result<(), Self::Error> {
        match op {
            InPresence::Selector => {}
            InPresence::Flag(s) | InPresence::Payload(s) => self.take(s),
        }
        Ok(())
    }

    fn sequence(
        &mut self,
        _node: &InNode,
        op: &InSlot,
        _inner: &InNode,
        _value: (),
    ) -> Result<(), Self::Error> {
        self.take(op);
        Ok(())
    }
}
