//! The out-of-Rust direction of the shared transformation tree (#442): how a
//! returned Rust value is taken apart into the values that cross.
//!
//! The tree is what the plan IS — the `Option` / `Vec` layers the value
//! crosses in ([`shaped`]) and the decomposition under them. [`UnfoldPlan::shape`],
//! [`UnfoldPlan::element`], [`UnfoldPlan::leaves`] and [`UnfoldPlan::hoists`]
//! are **derived views** of it, produced through the same [`TransformLowerer`]
//! a language adapter implements. Nothing is stored twice: a leaf's crossing
//! type is its node's [`ty`](TransformNode::ty), its access path is the links
//! above it, and its nullability is which of those links pass through an
//! `Option`.
//!
//! [`UnfoldPlan::shape`]: super::UnfoldPlan::shape
//! [`UnfoldPlan::element`]: super::UnfoldPlan::element
//! [`UnfoldPlan::leaves`]: super::UnfoldPlan::leaves
//! [`UnfoldPlan::hoists`]: super::UnfoldPlan::hoists

use super::{Hoist, LeafSource, PathStep, UnfoldLeaf, UnfoldShape};
use crate::transform::{
    Lowered, TransformChild, TransformDirection, TransformKind, TransformLowerer, TransformNode,
};

/// Direction marker: a Rust value taken apart into crossing values.
pub enum OutOfRust {}

impl TransformDirection for OutOfRust {
    type Leaf = OutLeaf;
    type Product = OutProduct;
    /// Decomposition has no alternatives: every record of a deconstructor
    /// contributes. Uninhabited, so a [`TransformKind::Choice`] node cannot be
    /// built in this direction at all.
    type Choice = std::convert::Infallible;
    /// The `Option<T>` / `Option<&T>` return layer carries nothing of its own:
    /// absent delivers a null result and the builder is skipped.
    type Optional = ();
    /// The `Vec<T>` / `&[T]` return layer carries nothing of its own — how its
    /// elements are delivered is what sits under it (see [`element_of`]).
    type Sequence = ();
    type Link = OutLink;
}

/// One node of an out-of-Rust decomposition.
pub type OutNode = TransformNode<OutOfRust>;
/// One child of an out-of-Rust product, with the link reaching it.
pub type OutChild = TransformChild<OutOfRust>;

/// A value that crosses out as one leaf. Its type is the node's
/// [`ty`](TransformNode::ty) and its access path is the links above it — this
/// carries only what neither of those says.
pub struct OutLeaf {
    /// `true` when the leaf is null **of itself**, independently of the path
    /// reaching it: a conditional handle delivery (`Option<&T>`), or a
    /// pre-built leaf whose own field was optional. Absence coming from a
    /// nesting link is derived, not stored.
    pub nullable: bool,
    /// The move/clone-the-value handle leaf — see [`UnfoldLeaf::identity`].
    pub identity: bool,
    /// How the leaf is reached — see [`LeafSource`].
    pub source: LeafSource,
    /// Sum-alternative membership — see [`UnfoldLeaf::group`].
    pub group: Option<i32>,
}

/// How a product node's children are obtained from its value.
pub enum OutProduct {
    /// Read the records off the value directly: the root of a decomposition,
    /// or a spliced child deconstructor reached through its link.
    Records,
    /// Call a **value form** once and bind the result, then read fields off
    /// that binding — the [`Hoist`] the derived view emits for this node.
    ValueForm {
        /// The accessor takes its receiver by value, so its fields move out
        /// instead of being cloned — see [`Hoist::consuming`].
        consuming: bool,
    },
}

/// How one child is reached from its parent's value.
pub struct OutLink {
    /// Access steps from the parent's value to the child's.
    pub steps: Vec<PathStep>,
    /// Leaf-name segments this level contributes, joined with `"__"` into the
    /// final name. Empty where a level names nothing (a value form; the
    /// identity leaf, which takes the chain it sits at).
    pub name: Vec<String>,
}

/// The flat views the plans expose: leaves in decomposition order, and the
/// value forms to bind once. Derived — never stored beside the tree.
pub fn flat_view(root: &OutNode) -> (Vec<UnfoldLeaf>, Vec<Hoist>) {
    let derived = root
        .lower(&mut FlatView)
        .expect("deriving a flat view of a built tree cannot fail");
    let leaves = derived
        .leaves
        .into_iter()
        .map(|(segs, mut leaf)| {
            // A leaf that names nothing at any level is the root identity —
            // the only one, since every nesting level contributes a segment.
            leaf.name = if segs.is_empty() {
                "handle".to_string()
            } else {
                segs.join("__")
            };
            leaf
        })
        .collect();
    (leaves, derived.hoists)
}

/// Everything below one node, with paths and names still **relative** to it —
/// each level prefixes its own link on the way up.
#[derive(Default)]
struct Partial {
    /// Leaves with their name segments carried alongside: the segments are
    /// joined only at the root, so a level that names nothing (a value form)
    /// prefixes nothing rather than an empty string.
    leaves: Vec<(Vec<String>, UnfoldLeaf)>,
    hoists: Vec<Hoist>,
}

/// The lowerer behind [`flat_view`].
struct FlatView;

impl TransformLowerer<OutOfRust> for FlatView {
    type Value = Partial;
    type Error = std::convert::Infallible;

    fn leaf(&mut self, node: &OutNode, op: &OutLeaf) -> Result<Partial, Self::Error> {
        Ok(Partial {
            leaves: vec![(
                Vec::new(),
                UnfoldLeaf {
                    name: String::new(),
                    path: Vec::new(),
                    out_ty: node.ty.clone(),
                    identity: op.identity,
                    nullable: op.nullable,
                    source: op.source.clone(),
                    group: op.group,
                },
            )],
            hoists: Vec::new(),
        })
    }

    fn product(
        &mut self,
        _node: &OutNode,
        op: &OutProduct,
        children: Lowered<'_, OutOfRust, Partial>,
    ) -> Result<Partial, Self::Error> {
        let mut out = Partial::default();
        // Outermost-first: this node's own binding precedes everything reached
        // through it.
        if let OutProduct::ValueForm { consuming } = op {
            out.hoists.push(Hoist {
                prefix: Vec::new(),
                consuming: *consuming,
            });
        }
        for (child, mut part) in children {
            // An `Option` on the way to a child is a nullable **nesting** step
            // only when something is decomposed below it. A leaf's own final
            // `Option` is what its converter takes, so it rides the leaf
            // instead — the leaf says so itself where that is not true.
            let nullable = !matches!(child.node.kind, TransformKind::Leaf(_))
                && child.link.steps.iter().any(PathStep::is_optional);
            for (segs, leaf) in &mut part.leaves {
                *segs = child
                    .link
                    .name
                    .iter()
                    .cloned()
                    .chain(segs.drain(..))
                    .collect();
                leaf.path = child
                    .link
                    .steps
                    .iter()
                    .cloned()
                    .chain(leaf.path.drain(..))
                    .collect();
                leaf.nullable |= nullable;
            }
            for hoist in &mut part.hoists {
                hoist.prefix = child
                    .link
                    .steps
                    .iter()
                    .cloned()
                    .chain(hoist.prefix.drain(..))
                    .collect();
            }
            out.leaves.append(&mut part.leaves);
            out.hoists.append(&mut part.hoists);
        }
        Ok(out)
    }

    fn choice(
        &mut self,
        _node: &OutNode,
        op: &std::convert::Infallible,
        _variants: Lowered<'_, OutOfRust, Partial>,
    ) -> Result<Partial, Self::Error> {
        match *op {}
    }

    fn optional(
        &mut self,
        _node: &OutNode,
        _op: &(),
        _inner: &OutNode,
        value: Partial,
    ) -> Result<Partial, Self::Error> {
        // Whole-value presence, decided once for the delivery — not a step on
        // any leaf's path, and not what makes a leaf nullable.
        Ok(value)
    }

    fn sequence(
        &mut self,
        _node: &OutNode,
        _op: &(),
        inner: &OutNode,
        value: Partial,
    ) -> Result<Partial, Self::Error> {
        // A leaf directly under the run is the WHOLE element: it crosses
        // through its own output converter as the fold's element, not as a
        // named slot of the call. Decomposed elements — a product under the
        // run — contribute their leaves as usual.
        if matches!(inner.kind, TransformKind::Leaf(_)) {
            return Ok(Partial::default());
        }
        Ok(value)
    }
}

/// The arity layers of `ty`, as nodes wrapping `core`.
///
/// `shape` is the layer stack already read off `ty`, and `layer_tys` are the
/// types those layers wrap, outermost first — the caller holds both, and
/// passing them beats re-reading a stack that was decided where the boundary
/// was classified.
pub fn shaped(
    shape: &UnfoldShape,
    layer_tys: &[prebindgen_flat::flat::TypeRef],
    core: OutNode,
) -> OutNode {
    fn wrap(shape: &UnfoldShape, tys: &[prebindgen_flat::flat::TypeRef], core: OutNode) -> OutNode {
        match shape {
            UnfoldShape::Base => core,
            UnfoldShape::Optional((), rest) => OutNode {
                ty: tys[0].clone(),
                kind: TransformKind::Optional {
                    op: (),
                    inner: Box::new(wrap(rest, &tys[1..], core)),
                },
            },
            UnfoldShape::Iterable(rest) => OutNode {
                ty: tys[0].clone(),
                kind: TransformKind::Sequence {
                    op: (),
                    inner: Box::new(wrap(rest, &tys[1..], core)),
                },
            },
        }
    }
    assert_eq!(
        layer_count(shape),
        layer_tys.len(),
        "each arity layer names the type it wraps"
    );
    wrap(shape, layer_tys, core)
}

/// How many `Option` / `Vec` layers a shape stacks.
fn layer_count(shape: &UnfoldShape) -> usize {
    match shape {
        UnfoldShape::Base => 0,
        UnfoldShape::Optional((), rest) | UnfoldShape::Iterable(rest) => 1 + layer_count(rest),
    }
}

/// The arity layers a tree wraps its decomposition in — the plan's
/// [`shape`](super::UnfoldPlan::shape), read back off the nodes that carry it.
pub fn shape_of(node: &OutNode) -> UnfoldShape {
    match &node.kind {
        TransformKind::Optional { inner, .. } => UnfoldShape::optional((), shape_of(inner)),
        TransformKind::Sequence { inner, .. } => UnfoldShape::iterable(shape_of(inner)),
        _ => UnfoldShape::Base,
    }
}

/// The element type of a **whole-element** run: a leaf directly under a
/// [`Sequence`](TransformKind::Sequence) crosses through its own output
/// converter instead of being taken apart. `None` for every other tree,
/// including a run whose elements ARE taken apart.
pub fn element_of(node: &OutNode) -> Option<&prebindgen_flat::flat::TypeRef> {
    match &node.kind {
        TransformKind::Optional { inner, .. } => element_of(inner),
        TransformKind::Sequence { inner, .. } => {
            matches!(inner.kind, TransformKind::Leaf(_)).then(|| &inner.ty)
        }
        _ => None,
    }
}

/// The tree of a decomposition whose leaves an **adapter** computed (a
/// by-value `data_class`, a sum): one product over the list it handed over,
/// each leaf reached by its own whole path.
///
/// Shallow rather than wrong — [`flat_view`] gives the list back unchanged.
/// Those declaration families describe a leaf list and not yet a tree, which is
/// the next thing #442 moves.
pub fn flat_tree(source: &prebindgen_flat::flat::TypeRef, leaves: &[UnfoldLeaf]) -> OutNode {
    OutNode {
        ty: source.clone(),
        kind: TransformKind::Product {
            op: OutProduct::Records,
            children: leaves
                .iter()
                .map(|leaf| OutChild {
                    link: OutLink {
                        steps: leaf.path.clone(),
                        name: vec![leaf.name.clone()],
                    },
                    node: OutNode {
                        ty: leaf.out_ty.clone(),
                        kind: TransformKind::Leaf(OutLeaf {
                            nullable: leaf.nullable,
                            identity: leaf.identity,
                            source: leaf.source.clone(),
                            group: leaf.group,
                        }),
                    },
                })
                .collect(),
        },
    }
}
