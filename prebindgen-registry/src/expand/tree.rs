//! The into-Rust direction of the shared transformation tree (#442): how the
//! values that cross in are put back together into one Rust value.
//!
//! The tree replaces the former `FoldVariant` / `FoldArg` / `FoldBuild` triple.
//! A constructor call is a [product](InProduct) over its arguments, a
//! selector dispatch is a [choice](InChoice) over its arms, and a wire value is
//! a [leaf](InLeaf) naming its slot in [`FoldPlan::leaves`]. Recursion is the
//! tree's, so a nested build is the same node kind as the top-level one rather
//! than a second type spelling the same thing.
//!
//! Every wire slot is described exactly once, by the node that uses it: its
//! type is that node's [`ty`](TransformNode::ty) and its foreign-side name is
//! on the node's payload. [`FoldPlan::leaves`] is [collected](wire_leaves) from
//! them — a derived view of the tree, in slot order, not a second list the
//! builder keeps in step by hand.
//!
//! [`FoldPlan::leaves`]: super::FoldPlan::leaves

use crate::transform::{
    Lowered, TransformChild, TransformDirection, TransformKind, TransformLowerer, TransformNode,
};

use super::FoldLeaf;

/// Direction marker: crossing values assembled into a Rust value.
pub enum IntoRust {}

impl TransformDirection for IntoRust {
    type Leaf = InLeaf;
    type Product = InProduct;
    type Choice = InChoice;
    /// The `Option<T>` parameter layer and the `Vec<T>` one still live on
    /// [`FoldPlan::shape`](super::FoldPlan::shape): moving them in means the
    /// emitter reading a *bound* value the layer unwrapped, which the
    /// children-first traversal has no hook for yet (#442). Uninhabited until
    /// then, so neither node kind can be built in this direction.
    type Optional = std::convert::Infallible;
    type Sequence = std::convert::Infallible;
    type Link = InLink;
}

/// One node of an into-Rust construction.
pub type InNode = TransformNode<IntoRust>;
/// One argument of an into-Rust product, or one arm of a choice.
pub type InChild = TransformChild<IntoRust>;

/// One decoded wire value, used as it stands. The slot's type is the node's
/// [`ty`](TransformNode::ty).
pub struct InLeaf {
    /// Which slot of [`FoldPlan::leaves`](super::FoldPlan::leaves) this decodes
    /// — the wire signature's own order, and the index of the caller's decoded
    /// local.
    pub slot: usize,
    /// The slot's foreign-side parameter name.
    pub name: syn::Ident,
    /// The slot is `Option`-wrapped by **selector presence** (it belongs to a
    /// dispatched arm and only the taken arm's slots are filled), so a consumer
    /// unwraps it before use and treats a missing value as an error.
    ///
    /// `false` both outside a dispatch and for a *passthrough* argument — one
    /// the constructor itself declares `Option<…>`, which keeps its own type on
    /// the wire because `None` is a legitimate value for the taken arm and the
    /// wire cannot carry the double `Option` anyway.
    pub wrapped: bool,
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
    /// Which slot of [`FoldPlan::leaves`](super::FoldPlan::leaves) carries the
    /// `i32` selector. Arm `i` is taken when the selector reads `i`; under an
    /// [`Optional`](super::FoldShape::Optional) shape `-1` additionally means
    /// absent.
    pub selector: usize,
    /// The selector slot's foreign-side parameter name.
    pub name: syn::Ident,
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
    /// The selector slot when this node dispatches, `None` when it is a single
    /// unconditional construction.
    pub fn selector(&self) -> Option<usize> {
        match &self.kind {
            crate::transform::TransformKind::Choice { op, .. } => Some(op.selector),
            _ => None,
        }
    }

    /// The arms of a dispatch, or the single construction on its own — the
    /// reading a consumer wants when it asks what this parameter can be built
    /// from. In selector order: arm `i` is taken when the selector reads `i`.
    pub fn arms(&self) -> Vec<&InNode> {
        match &self.kind {
            TransformKind::Choice { variants, .. } => variants.iter().map(|v| &v.node).collect(),
            _ => vec![self],
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
    /// `None` when any argument is itself built from further slots: the arm
    /// then has no flat signature, which is what makes it unsplittable into a
    /// destination-language overload.
    pub fn leaf_args(&self) -> Option<Vec<usize>> {
        let TransformKind::Product { children, .. } = &self.kind else {
            return None;
        };
        children
            .iter()
            .map(|c| match &c.node.kind {
                TransformKind::Leaf(op) => Some(op.slot),
                _ => None,
            })
            .collect()
    }
}

/// Collect the wire slots a construction uses, in slot order — the flat
/// signature [`FoldPlan::leaves`](super::FoldPlan::leaves) exposes.
///
/// `extra` carries slots that belong to the parameter rather than to the
/// construction: today only the whole-parameter presence flag of a multi-argument
/// `Option<T>`, which the [`Optional`](super::FoldShape::Optional) shape reads
/// and the construction never sees. Every slot is named exactly once — a gap or
/// a collision here means the builder handed out an id twice.
pub fn wire_leaves(core: &InNode, extra: Vec<(usize, FoldLeaf)>) -> Vec<FoldLeaf> {
    let mut slots = extra;
    core.lower(&mut CollectSlots(&mut slots))
        .expect("collecting wire slots cannot fail");
    let mut out: Vec<Option<FoldLeaf>> = (0..slots.len()).map(|_| None).collect();
    for (slot, leaf) in slots {
        let cell = out
            .get_mut(slot)
            .expect("a wire slot outside the construction's own count");
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

impl TransformLowerer<IntoRust> for CollectSlots<'_> {
    type Value = ();
    type Error = std::convert::Infallible;

    fn leaf(&mut self, node: &InNode, op: &InLeaf) -> Result<(), Self::Error> {
        self.0.push((
            op.slot,
            FoldLeaf {
                name: op.name.clone(),
                ty: node.ty.clone(),
            },
        ));
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
        self.0.push((
            op.selector,
            FoldLeaf {
                name: op.name.clone(),
                // The selector: composed, and placeless by construction.
                ty: prebindgen_flat::flat::TypeRef::scalar(prebindgen_flat::flat::ScalarKind::I32),
            },
        ));
        Ok(())
    }

    fn optional(
        &mut self,
        _node: &InNode,
        op: &std::convert::Infallible,
        _inner: &InNode,
        _value: (),
    ) -> Result<(), Self::Error> {
        match *op {}
    }

    fn sequence(
        &mut self,
        _node: &InNode,
        op: &std::convert::Infallible,
        _inner: &InNode,
        _value: (),
    ) -> Result<(), Self::Error> {
        match *op {}
    }
}
