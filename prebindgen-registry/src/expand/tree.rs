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
//! What is NOT here yet: the flat [`FoldPlan::leaves`] vector and the selector
//! and presence slots that ride in it are still built by the walk rather than
//! derived from the tree. Separating constructor semantics from those synthetic
//! wire slots is the next step of #442.
//!
//! [`FoldPlan::leaves`]: super::FoldPlan::leaves

use crate::transform::{TransformChild, TransformDirection, TransformKind, TransformNode};

/// Direction marker: crossing values assembled into a Rust value.
pub enum IntoRust {}

impl TransformDirection for IntoRust {
    type Leaf = InLeaf;
    type Product = InProduct;
    type Choice = InChoice;
    type Link = InLink;
}

/// One node of an into-Rust construction.
pub type InNode = TransformNode<IntoRust>;
/// One argument of an into-Rust product, or one arm of a choice.
pub type InChild = TransformChild<IntoRust>;

/// One decoded wire value, used as it stands.
pub struct InLeaf {
    /// Which slot of [`FoldPlan::leaves`](super::FoldPlan::leaves) this decodes
    /// — the wire signature's own order, and the index of the caller's decoded
    /// local.
    pub leaf: usize,
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

/// How a choice node picks the arm that runs.
pub struct InChoice {
    /// Which slot of [`FoldPlan::leaves`](super::FoldPlan::leaves) carries the
    /// `i32` selector. Arm `i` is taken when the selector reads `i`; under an
    /// [`Optional`](super::FoldShape::Optional) shape `-1` additionally means
    /// absent.
    pub selector: usize,
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
                TransformKind::Leaf(op) => Some(op.leaf),
                _ => None,
            })
            .collect()
    }
}
