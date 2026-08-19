//! The one recursive mechanism both boundary directions share.
//!
//! A **transformation tree** describes what happens at ONE boundary use: how a
//! Rust value is taken apart into the values that cross
//! ([`OutOfRust`](crate::unfold::OutOfRust)), or how the values that cross are
//! put back together into a Rust one
//! ([`IntoRust`](crate::expand::IntoRust)). It is not a second type model:
//! every node names its [`TypeRef`](prebindgen_flat::flat::TypeRef), which
//! stays the authoritative reading.
//!
//! The tree is generic over its **direction** so the recursion, the child
//! ordering and the traversal are written once. Only the payloads differ:
//! [`TransformDirection::Leaf`] says what one crossing value is, `Product` says
//! how a node's children combine, `Choice` says how one of them is selected,
//! and `Link` says how one child hangs off its parent. A language adapter
//! supplies node-level policy through [`TransformLowerer`] and never writes the
//! walk itself.
//!
//! A direction that has no use for a kind gives its payload an uninhabited
//! type, and the kind is then unconstructible for that direction: decomposition
//! has no `Choice`, so `OutOfRust::Choice` is
//! [`Infallible`](std::convert::Infallible).
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
    /// A node where **exactly one** child runs — a constructor dispatch.
    type Choice;
    /// An `Option<…>` layer over one inner node.
    type Optional;
    /// A run of one inner node.
    type Sequence;
    /// How one child hangs off its parent.
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
pub enum TransformKind<D: TransformDirection> {
    /// A value that crosses as it is.
    Leaf(D::Leaf),
    /// Every child contributes, in order.
    Product {
        op: D::Product,
        children: Vec<TransformChild<D>>,
    },
    /// Exactly one child runs — which one is `op`'s business. The children's
    /// [links](TransformChild::link) carry nothing: an alternative is named by
    /// its position, not by how it hangs off the choice.
    Choice {
        op: D::Choice,
        variants: Vec<TransformChild<D>>,
    },
    /// `Option<…>` over `inner`: absent, or `inner`.
    Optional {
        op: D::Optional,
        inner: Box<TransformNode<D>>,
    },
    /// A run of `inner`s.
    Sequence {
        op: D::Sequence,
        inner: Box<TransformNode<D>>,
    },
}

/// One child of a [`TransformKind::Product`], with the link that reaches it.
pub struct TransformChild<D: TransformDirection> {
    /// How this child hangs off its parent's value.
    pub link: D::Link,
    /// The child itself.
    pub node: TransformNode<D>,
}

/// One node's already lowered children, each paired with the
/// [`TransformChild`] it came from.
pub type Lowered<'a, D, V> = Vec<(&'a TransformChild<D>, V)>;

/// What a lowerer decides about a node **before** its children are visited.
///
/// The point of deciding first is that an adapter with a direct converter for a
/// whole subtree — a `Vec<u8>` that crosses as one array rather than as a run
/// of elements — must be able to stop there. Answering [`Atomic`](Self::Atomic)
/// means the subtree is never lowered, so it contributes no slots, no
/// converter dependencies, no conversion and no cleanup: one decision, not the
/// same decision repeated by every pass that walks the tree.
pub enum Descend<V> {
    /// This node's whole subtree is handled by `V`. Nothing below it is
    /// visited.
    Atomic(V),
    /// Visit the children and combine them as usual.
    Recurse,
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

    /// Decide what to do with `node` before its children are visited.
    ///
    /// The default recurses, which is what a lowerer that has no direct
    /// converters wants. Answer [`Descend::Atomic`] to claim the whole subtree
    /// — see [`Descend`] for what that settles.
    ///
    /// `link` is the edge the node was reached by, `None` at the root and under
    /// an arity layer, which reaches its inner node directly. A converter is
    /// chosen for a value *in a position*, not for a type — the same
    /// `ZKeyExpr` reached by an owning accessor and by a borrowing one converts
    /// and cleans up differently — so the decision needs the edge, not only the
    /// node.
    ///
    /// **What this still does not say** (#444 §5): the edge is the only context
    /// offered, and a layer has none, so a lowerer cannot tell the element of a
    /// run from a value at the root by looking at its arguments. Synthesising
    /// an empty link for a layer would not help — a layer contributes no access
    /// step and no name, so `Some(empty)` and `None` would say the same thing.
    /// What an adapter actually wants there is the *position* — root,
    /// product child, choice arm, run element — because an element must lower
    /// to a single wire value where a root need not. That is a different
    /// parameter from the link, and it should land with the adapter that reads
    /// it rather than be guessed at now.
    fn descend(
        &mut self,
        node: &TransformNode<D>,
        link: Option<&D::Link>,
    ) -> Result<Descend<Self::Value>, Self::Error> {
        let _ = (node, link);
        Ok(Descend::Recurse)
    }

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
        children: Lowered<'_, D, Self::Value>,
    ) -> Result<Self::Value, Self::Error>;

    /// Select between the already lowered alternatives of a choice, in
    /// declaration order — which is also selector order.
    ///
    /// A direction whose `Choice` payload is uninhabited discharges this with
    /// `match *op {}`: the kind cannot occur, so there is nothing to write.
    fn choice(
        &mut self,
        node: &TransformNode<D>,
        op: &D::Choice,
        variants: Lowered<'_, D, Self::Value>,
    ) -> Result<Self::Value, Self::Error>;

    /// Lift the already lowered `inner` over an `Option<…>` layer. `inner` is
    /// handed over as well as its value, for a rule that turns on what kind of
    /// node sits under the layer.
    fn optional(
        &mut self,
        node: &TransformNode<D>,
        op: &D::Optional,
        inner: &TransformNode<D>,
        value: Self::Value,
    ) -> Result<Self::Value, Self::Error>;

    /// Lift the already lowered `inner` over a run of it.
    fn sequence(
        &mut self,
        node: &TransformNode<D>,
        op: &D::Sequence,
        inner: &TransformNode<D>,
        value: Self::Value,
    ) -> Result<Self::Value, Self::Error>;
}

impl<D: TransformDirection> TransformNode<D> {
    /// Run `lowerer` over this node and everything below it, children before
    /// parents and in declaration order — unless the lowerer answers
    /// [`Descend::Atomic`] for a node, which ends that subtree there.
    pub fn lower<L: TransformLowerer<D>>(&self, lowerer: &mut L) -> Result<L::Value, L::Error> {
        self.lower_at(None, lowerer)
    }

    /// [`lower`](Self::lower), told which edge reached this node so the
    /// pre-descent decision can see it.
    fn lower_at<L: TransformLowerer<D>>(
        &self,
        link: Option<&D::Link>,
        lowerer: &mut L,
    ) -> Result<L::Value, L::Error> {
        // Asked before anything below is touched: an atomic answer is what
        // makes a subtree contribute nothing at all.
        if let Descend::Atomic(value) = lowerer.descend(self, link)? {
            return Ok(value);
        }
        match &self.kind {
            TransformKind::Leaf(op) => lowerer.leaf(self, op),
            TransformKind::Product { op, children } => {
                let lowered = lower_all(children, lowerer)?;
                lowerer.product(self, op, lowered)
            }
            TransformKind::Choice { op, variants } => {
                let lowered = lower_all(variants, lowerer)?;
                lowerer.choice(self, op, lowered)
            }
            TransformKind::Optional { op, inner } => {
                // A layer reaches its inner node directly — there is no edge.
                let value = inner.lower_at(None, lowerer)?;
                lowerer.optional(self, op, inner, value)
            }
            TransformKind::Sequence { op, inner } => {
                let value = inner.lower_at(None, lowerer)?;
                lowerer.sequence(self, op, inner, value)
            }
        }
    }
}

/// Lower each child in order, keeping it paired with the child it came from.
fn lower_all<'a, D: TransformDirection, L: TransformLowerer<D>>(
    children: &'a [TransformChild<D>],
    lowerer: &mut L,
) -> Result<Lowered<'a, D, L::Value>, L::Error> {
    let mut lowered = Vec::with_capacity(children.len());
    for child in children {
        let value = child.node.lower_at(Some(&child.link), lowerer)?;
        lowered.push((child, value));
    }
    Ok(lowered)
}

/// A tree is cloned where one decomposition serves several boundary uses — the
/// same sum returned by a dozen functions, each wrapping it in its own arity
/// layers. Written out rather than derived: `derive(Clone)` would ask for
/// `D: Clone`, and a direction marker is uninhabited.
impl<D: TransformDirection> Clone for TransformNode<D>
where
    D::Leaf: Clone,
    D::Product: Clone,
    D::Choice: Clone,
    D::Optional: Clone,
    D::Sequence: Clone,
    D::Link: Clone,
{
    fn clone(&self) -> Self {
        Self {
            ty: self.ty.clone(),
            kind: self.kind.clone(),
        }
    }
}

impl<D: TransformDirection> Clone for TransformKind<D>
where
    D::Leaf: Clone,
    D::Product: Clone,
    D::Choice: Clone,
    D::Optional: Clone,
    D::Sequence: Clone,
    D::Link: Clone,
{
    fn clone(&self) -> Self {
        match self {
            Self::Leaf(op) => Self::Leaf(op.clone()),
            Self::Product { op, children } => Self::Product {
                op: op.clone(),
                children: children.clone(),
            },
            Self::Choice { op, variants } => Self::Choice {
                op: op.clone(),
                variants: variants.clone(),
            },
            Self::Optional { op, inner } => Self::Optional {
                op: op.clone(),
                inner: inner.clone(),
            },
            Self::Sequence { op, inner } => Self::Sequence {
                op: op.clone(),
                inner: inner.clone(),
            },
        }
    }
}

impl<D: TransformDirection> Clone for TransformChild<D>
where
    D::Leaf: Clone,
    D::Product: Clone,
    D::Choice: Clone,
    D::Optional: Clone,
    D::Sequence: Clone,
    D::Link: Clone,
{
    fn clone(&self) -> Self {
        Self {
            link: self.link.clone(),
            node: self.node.clone(),
        }
    }
}
