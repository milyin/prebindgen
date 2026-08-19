//! The one recursive mechanism both boundary directions share.
//!
//! A **transformation tree** describes what happens at ONE boundary use: how a
//! Rust value is taken apart into the values that cross ([`OutOfRust`](crate::unfold::OutOfRust)), or how
//! the values that cross are put back together into a Rust one (`IntoRust`, not
//! migrated yet — see #442). It is not a second type model: every node names
//! its [`TypeRef`](prebindgen_flat::flat::TypeRef), which stays the authoritative reading.
//!
//! The tree is generic over its **direction** so the recursion, the child
//! ordering and the traversal are written once. Only the payloads differ:
//! [`TransformDirection::Leaf`] says what one crossing value is, `Product` says
//! how a node's children are obtained, and `Link` says how one child hangs off
//! its parent. A language adapter supplies node-level policy through
//! [`TransformLowerer`] and never writes the walk itself.
//!
//! The flat views the plans still expose — a leaf vector, a hoist list — are
//! **derived** by lowering the tree ([`crate::unfold::UnfoldPlan::leaves`] is
//! built this way), not carried beside it.

/// The direction a transformation runs in, and the payloads its nodes carry.
///
/// Implemented by an uninhabited marker per direction, so a tree cannot mix the
/// two: every node of a [`TransformNode<D>`] is that one direction's.
pub trait TransformDirection {
    /// A terminal: one value as it crosses the boundary.
    type Leaf;
    /// A node whose children **all** contribute — a deterministic product
    /// (a deconstructor's records, a constructor's arguments).
    type Product;
    /// How one child is reached from its parent's value.
    type Link;
}

/// One node of a transformation tree.
pub struct TransformNode<D: TransformDirection> {
    /// The Rust type this node transforms — a leaf's crossing type, a
    /// product's decomposed/constructed type. The single source-type model:
    /// nothing below re-spells it.
    pub ty: prebindgen_flat::flat::TypeRef,
    /// What happens at this node.
    pub kind: TransformKind<D>,
}

/// What a [`TransformNode`] does.
///
/// `Optional` / `Sequence` / `Choice` nodes are the remaining structural kinds
/// (#442): today the outer `Option` / `Vec` layers live on the plans' own
/// [`Shape`](prebindgen_flat::shape::Shape) and the choice payloads belong to
/// the not-yet-migrated input direction, so adding those variants here would
/// add arms nothing produces.
pub enum TransformKind<D: TransformDirection> {
    /// A value that crosses as it is.
    Leaf(D::Leaf),
    /// Every child contributes, in order.
    Product {
        op: D::Product,
        children: Vec<TransformChild<D>>,
    },
}

/// One child of a [`TransformKind::Product`], with the link that reaches it.
pub struct TransformChild<D: TransformDirection> {
    /// How this child hangs off its parent's value.
    pub link: D::Link,
    /// The child itself.
    pub node: TransformNode<D>,
}

/// Node-level policy for one traversal of a tree: the caller says what a leaf
/// and a product *mean*, the tree supplies the recursion and the order.
///
/// Bottom-up by construction — a product is handed its children's already
/// lowered values, each with the link it came through, so a lowerer that needs
/// to prefix something (an access path, a name chain) does it once per level
/// rather than threading state down a walk it wrote itself.
pub trait TransformLowerer<D: TransformDirection> {
    /// What lowering one node yields.
    type Value;
    /// Why lowering a node can fail.
    type Error;

    /// Lower a terminal. `node.ty` is the crossing type.
    fn leaf(&mut self, node: &TransformNode<D>, op: &D::Leaf) -> Result<Self::Value, Self::Error>;

    /// Combine the already lowered children of a product, in child order.
    ///
    /// Each child arrives with the [`TransformChild`] it came from, so a
    /// lowerer can read the link it hangs on and what kind of node it is — the
    /// two questions a per-level rule asks — without descending itself.
    fn product(
        &mut self,
        node: &TransformNode<D>,
        op: &D::Product,
        children: Vec<(&TransformChild<D>, Self::Value)>,
    ) -> Result<Self::Value, Self::Error>;
}

impl<D: TransformDirection> TransformNode<D> {
    /// Run `lowerer` over this node and everything below it, children before
    /// parents and in declaration order.
    pub fn lower<L: TransformLowerer<D>>(&self, lowerer: &mut L) -> Result<L::Value, L::Error> {
        match &self.kind {
            TransformKind::Leaf(op) => lowerer.leaf(self, op),
            TransformKind::Product { op, children } => {
                let mut lowered = Vec::with_capacity(children.len());
                for child in children {
                    let value = child.node.lower(lowerer)?;
                    lowered.push((child, value));
                }
                lowerer.product(self, op, lowered)
            }
        }
    }
}
