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
    /// A **sum**: exactly one alternative of the decomposed value is live, and
    /// which one is what the choice's synthesized selector says.
    type Choice = OutChoice;
    /// The `Option<T>` / `Option<&T>` return layer carries nothing of its own:
    /// absent delivers a null result and the builder is skipped.
    type Optional = ();
    /// A run says whether its elements are reached **through a borrow** — a
    /// shared slice is read by copying out of it, an owned collection is
    /// consumed. How its elements are *delivered* is what sits under it (see
    /// [`element_of`]).
    type Sequence = OutRun;
    type Link = OutLink;
}

/// One node of an out-of-Rust decomposition.
pub type OutNode = TransformNode<OutOfRust>;
/// One child of an out-of-Rust product, with the link reaching it.
pub type OutChild = TransformChild<OutOfRust>;

/// A value that crosses out as one leaf. Its type is the node's
/// [`ty`](TransformNode::ty) and its access path is the links above it — this
/// carries only what neither of those says.
#[derive(Clone)]
pub struct OutLeaf {
    /// `true` when the leaf is null **of itself**, independently of the path
    /// reaching it: a conditional handle delivery (`Option<&T>`), or a
    /// pre-built leaf whose own field was optional. Absence coming from a
    /// nesting link is derived, not stored.
    pub nullable: bool,
    /// The move/clone-the-value handle leaf — see [`UnfoldLeaf::identity`].
    pub identity: bool,
    /// How the leaf is reached.
    pub reach: OutReach,
}

/// How a leaf is reached from the value the links above it lead to.
///
/// The [`LeafSource`] a derived leaf carries is composed from this and from
/// where the leaf sits: a [`VariantMember`](Self::VariantMember) only becomes a
/// [`LeafSource::VariantField`] once the [choice arm](OutProduct::Variant) above
/// it says which variant, so the arm is named once rather than on every leaf
/// under it.
#[derive(Clone)]
pub enum OutReach {
    /// The links above are an accessor chain — see [`LeafSource::Accessor`].
    Accessor,
    /// The links above are a struct-field chain — see [`LeafSource::Field`].
    Field,
    /// A payload member of the enclosing arm's variant pattern — see
    /// [`LeafSource::VariantField`].
    VariantMember(syn::Member),
}

/// What a [`Choice`](TransformKind::Choice) node selects between: the
/// alternatives of a decomposed sum, exactly one of which is live per value.
///
/// The selector is a wire value no source wrote, contributed by this node. It
/// carries **the sum** rather than an `i32`, which is how an emitter finds the
/// enum to `match` — the node's own [`ty`](TransformNode::ty) says which, so
/// nothing is stored twice.
#[derive(Clone)]
pub struct OutChoice {
    /// The selector leaf's name segment.
    pub name: String,
}

/// How a product node's children are obtained from its value.
#[derive(Clone)]
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
    /// One alternative of a [choice](OutChoice): its fields are live only when
    /// the selector reads this arm's tag.
    Variant {
        /// The variant's ident as declared in the source enum.
        name: syn::Ident,
        /// The selector value that makes this arm live — the
        /// [`group`](UnfoldLeaf::group) every leaf under it derives.
        tag: i32,
    },
}

/// How one child is reached from its parent's value.
#[derive(Clone)]
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
pub fn flat_view(root: &OutNode) -> Result<(Vec<UnfoldLeaf>, Vec<Hoist>), super::UnfoldError> {
    // The one parent position the lowerer cannot check for itself.
    if is_variant_arm(root) {
        return Err(arm_outside_choice(root));
    }
    let derived = root.lower(&mut FlatView)?;
    let leaves = derived
        .leaves
        .into_iter()
        .map(|(segs, member, mut leaf)| {
            // A member binding still pending here had no variant arm above it
            // to say which variant it binds in, so it would project as though
            // the value were walked to rather than matched out. The mirror of
            // the arm's own check.
            if let Some(member) = member {
                return Err(super::UnfoldError::VariantMemberOutsideArm {
                    member: match member {
                        syn::Member::Named(i) => i.to_string(),
                        syn::Member::Unnamed(i) => i.index.to_string(),
                    },
                });
            }
            // A leaf that names nothing at any level is the root identity —
            // the only one, since every nesting level contributes a segment.
            leaf.name = if segs.is_empty() {
                "handle".to_string()
            } else {
                segs.join("__")
            };
            Ok(leaf)
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok((leaves, derived.hoists))
}

/// Everything below one node, with paths and names still **relative** to it —
/// each level prefixes its own link on the way up.
#[derive(Default)]
struct Partial {
    /// Leaves with their name segments carried alongside: the segments are
    /// joined only at the root, so a level that names nothing (a value form)
    /// prefixes nothing rather than an empty string.
    ///
    /// A leaf reached as a variant payload also carries the member it binds,
    /// until the [arm](OutProduct::Variant) above it says which variant that
    /// member belongs to.
    leaves: Vec<PartialLeaf>,
    hoists: Vec<Hoist>,
}

/// One leaf on its way up: its name segments so far, the member it binds if it
/// is a variant payload, and the leaf itself.
type PartialLeaf = (Vec<String>, Option<syn::Member>, UnfoldLeaf);

/// The error a variant arm found outside a choice reports.
fn arm_outside_choice(node: &OutNode) -> super::UnfoldError {
    let variant = match &node.kind {
        TransformKind::Product {
            op: OutProduct::Variant { name, .. },
            ..
        } => name.to_string(),
        _ => unreachable!("only asked of a variant arm"),
    };
    super::UnfoldError::VariantArmOutsideChoice { variant }
}

/// Whether a node is a variant arm — the only thing a choice's alternative may
/// be, and the only place a variant arm may sit.
fn is_variant_arm(node: &OutNode) -> bool {
    matches!(
        node.kind,
        TransformKind::Product {
            op: OutProduct::Variant { .. },
            ..
        }
    )
}

/// What a node is, for a diagnostic about where it may not sit.
fn node_kind_name(node: &OutNode) -> &'static str {
    match &node.kind {
        TransformKind::Leaf(_) => "a leaf",
        TransformKind::Product {
            op: OutProduct::Variant { .. },
            ..
        } => "a variant arm",
        TransformKind::Product { .. } => "a product",
        TransformKind::Choice { .. } => "a choice",
        TransformKind::Optional { .. } => "an option",
        TransformKind::Sequence { .. } => "a run",
    }
}

/// The lowerer behind [`flat_view`].
struct FlatView;

impl TransformLowerer<OutOfRust> for FlatView {
    type Value = Partial;
    /// Unsupported structure is a planning error, not an abort: a projection is
    /// public, and an adapter handing over a shape it cannot have should be told
    /// which sum and which arm rather than stopping generation.
    type Error = super::UnfoldError;

    fn leaf(&mut self, node: &OutNode, op: &OutLeaf) -> Result<Partial, Self::Error> {
        // A variant payload's `LeafSource` is only complete once its arm names
        // the variant, so the member waits beside the leaf until then.
        let (source, member) = match &op.reach {
            OutReach::Accessor => (LeafSource::Accessor, None),
            OutReach::Field => (LeafSource::Field, None),
            OutReach::VariantMember(m) => (LeafSource::Accessor, Some(m.clone())),
        };
        Ok(Partial {
            leaves: vec![(
                Vec::new(),
                member,
                UnfoldLeaf {
                    name: String::new(),
                    path: Vec::new(),
                    out_ty: node.ty.clone(),
                    identity: op.identity,
                    nullable: op.nullable,
                    source,
                    group: None,
                },
            )],
            hoists: Vec::new(),
        })
    }

    fn product(
        &mut self,
        node: &OutNode,
        op: &OutProduct,
        children: Lowered<'_, OutOfRust, Partial>,
    ) -> Result<Partial, Self::Error> {
        // The mirror: an arm anywhere but under a choice carries a tag no
        // selector chooses between. Checked at every parent a node can have —
        // here, at the two layers, and at the root — so an arm is reachable
        // only from `choice`.
        if !matches!(op, OutProduct::Variant { .. }) {
            for (child, _) in &children {
                if is_variant_arm(&child.node) {
                    return Err(arm_outside_choice(&child.node));
                }
            }
        }
        // A variant arm's payloads must each be a leaf that BINDS A MEMBER.
        // Being a leaf is not enough: the binding an arm upgrades to
        // `LeafSource::VariantField` comes from the leaf's own
        // `OutReach::VariantMember`, so a leaf reached any other way is grouped
        // under the arm while keeping reach semantics that say it was found by
        // walking the value — which is not how a payload is reached at all.
        //
        // Asked of the children themselves rather than inferred from what came
        // back: a nested sum happens to show up as leaves that already carry a
        // group, but nothing else does.
        if let OutProduct::Variant { name, .. } = op {
            for (child, _) in &children {
                let found = match &child.node.kind {
                    TransformKind::Leaf(OutLeaf {
                        reach: OutReach::VariantMember(_),
                        ..
                    }) => continue,
                    TransformKind::Leaf(OutLeaf {
                        reach: OutReach::Field,
                        ..
                    }) => "a leaf reached by field access, binding no member",
                    TransformKind::Leaf(OutLeaf {
                        reach: OutReach::Accessor,
                        ..
                    }) => "a leaf reached by an accessor, binding no member",
                    TransformKind::Product { .. } => "a product",
                    TransformKind::Choice { .. } => "a choice",
                    TransformKind::Optional { .. } => "an option",
                    TransformKind::Sequence { .. } => "a run",
                };
                return Err(super::UnfoldError::UnsupportedVariantPayload {
                    target: node.ty.key().to_string(),
                    variant: name.to_string(),
                    found,
                });
            }
        }
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
            for (segs, _, leaf) in &mut part.leaves {
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
        // An arm names the variant its payload members bind in, and marks
        // everything under it live only for its own tag.
        if let OutProduct::Variant { name, tag } = op {
            for (_, member, leaf) in &mut out.leaves {
                leaf.group = Some(*tag);
                if let Some(member) = member.take() {
                    leaf.source = LeafSource::VariantField {
                        variant: name.clone(),
                        member,
                    };
                }
            }
        }
        Ok(out)
    }

    fn choice(
        &mut self,
        node: &OutNode,
        op: &OutChoice,
        variants: Lowered<'_, OutOfRust, Partial>,
    ) -> Result<Partial, Self::Error> {
        // Only a variant arm says WHICH alternative it is. Anything else here
        // flattens into a leaf belonging to no alternative, while the selector
        // ahead of it claims to choose between some.
        for (child, _) in &variants {
            if !is_variant_arm(&child.node) {
                return Err(super::UnfoldError::ChoiceAlternativeNotAnArm {
                    target: node.ty.key().to_string(),
                    found: node_kind_name(&child.node),
                });
            }
        }
        // The selector rides ahead of the arms it chooses between, and carries
        // **which sum** it selects over — the node's own type — rather than the
        // `i32` it is on the wire. That is how an emitter finds the enum to
        // `match`; nothing resolves a converter for it.
        let mut out = Partial {
            leaves: vec![(
                vec![op.name.clone()],
                None,
                UnfoldLeaf {
                    name: String::new(),
                    path: Vec::new(),
                    out_ty: node.ty.clone(),
                    identity: false,
                    nullable: false,
                    source: LeafSource::SumTag,
                    group: None,
                },
            )],
            hoists: Vec::new(),
        };
        for (child, mut part) in variants {
            for (segs, _, leaf) in &mut part.leaves {
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
            }
            out.leaves.append(&mut part.leaves);
            out.hoists.append(&mut part.hoists);
        }
        Ok(out)
    }

    fn optional(
        &mut self,
        _node: &OutNode,
        _op: &(),
        inner: &OutNode,
        value: Partial,
    ) -> Result<Partial, Self::Error> {
        if is_variant_arm(inner) {
            return Err(arm_outside_choice(inner));
        }
        // Whole-value presence, decided once for the delivery — not a step on
        // any leaf's path, and not what makes a leaf nullable.
        Ok(value)
    }

    fn sequence(
        &mut self,
        _node: &OutNode,
        _op: &OutRun,
        inner: &OutNode,
        value: Partial,
    ) -> Result<Partial, Self::Error> {
        if is_variant_arm(inner) {
            return Err(arm_outside_choice(inner));
        }
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
                    // The layer's own type says it: `&[T]` is reached through a
                    // borrow, `Vec<T>` is not.
                    op: OutRun {
                        borrowed: tys[0].borrow_target().is_some(),
                    },
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

/// The converters a decomposition needs, and the types it only names.
///
/// Derived by [`dependencies`], which is what makes it agree with the plan an
/// adapter lowers: a subtree claimed whole by a direct converter contributes
/// nothing here either.
#[derive(Default)]
pub struct Dependencies {
    /// Types the binding **demands** an output converter for — every value that
    /// actually crosses.
    pub required: Vec<prebindgen_flat::flat::TypeRef>,
    /// Types the decomposition names without converting: a sum, which the
    /// selector chooses between. Registering one says it entered the pipeline;
    /// requiring it would demand a whole-value converter that cannot exist.
    pub referenced: Vec<prebindgen_flat::flat::TypeRef>,
}

/// What a decomposition depends on, read off the tree rather than off a leaf
/// list — so the question is answered by the same structure every other pass
/// walks.
///
/// Ask it of the tree the adapter actually lowers: a subtree claimed by a
/// direct converter is a [`select`]ed leaf by then, so it contributes that
/// converter and nothing from below it. Registration and lowering cannot
/// disagree, because there is one tree rather than one policy consulted twice.
pub fn dependencies(root: &OutNode) -> Dependencies {
    root.lower(&mut CollectDeps)
        .expect("collecting dependencies of a built tree cannot fail")
}

/// The lowerer behind [`dependencies`]: a leaf needs its converter, a choice
/// names its sum, and everything else contributes only what is under it.
struct CollectDeps;

impl CollectDeps {
    fn merge(parts: Lowered<'_, OutOfRust, Dependencies>) -> Dependencies {
        let mut out = Dependencies::default();
        for (_, mut part) in parts {
            out.required.append(&mut part.required);
            out.referenced.append(&mut part.referenced);
        }
        out
    }
}

impl TransformLowerer<OutOfRust> for CollectDeps {
    type Value = Dependencies;
    type Error = std::convert::Infallible;

    fn leaf(&mut self, node: &OutNode, _op: &OutLeaf) -> Result<Dependencies, Self::Error> {
        Ok(Dependencies {
            required: vec![node.ty.clone()],
            referenced: Vec::new(),
        })
    }

    fn product(
        &mut self,
        _node: &OutNode,
        _op: &OutProduct,
        children: Lowered<'_, OutOfRust, Dependencies>,
    ) -> Result<Dependencies, Self::Error> {
        Ok(Self::merge(children))
    }

    fn choice(
        &mut self,
        node: &OutNode,
        _op: &OutChoice,
        variants: Lowered<'_, OutOfRust, Dependencies>,
    ) -> Result<Dependencies, Self::Error> {
        let mut out = Self::merge(variants);
        // The selector names the sum it chooses between and converts nothing:
        // the emitter assigns the tag per arm.
        out.referenced.push(node.ty.clone());
        Ok(out)
    }

    fn optional(
        &mut self,
        _node: &OutNode,
        _op: &(),
        _inner: &OutNode,
        value: Dependencies,
    ) -> Result<Dependencies, Self::Error> {
        Ok(value)
    }

    fn sequence(
        &mut self,
        _node: &OutNode,
        _op: &OutRun,
        _inner: &OutNode,
        value: Dependencies,
    ) -> Result<Dependencies, Self::Error> {
        // A whole element is exactly the value crossing through its own
        // converter, so it is a dependency like any other. It is not a named
        // wire slot — which is why the derived leaf list drops it — but the two
        // questions are different, and answering the second here left the
        // required set of a `Vec<T>` empty.
        Ok(value)
    }
}

/// What a run says about itself.
///
/// A property of the Rust type, not of any one boundary: `&[T]` is read through
/// a borrow and `Vec<T>` is not. Carried here so a lowering reads a plan fact
/// rather than classifying the type a second time.
#[derive(Clone)]
pub struct OutRun {
    pub borrowed: bool,
}

/// One arity layer a boundary reads off a type.
pub enum OrdinaryLayer {
    /// The value may be absent.
    Optional,
    /// The value is a run of its inner type, read through a borrow or not.
    Sequence { borrowed: bool },
}

/// The semantic plan of an **ordinary** boundary use: one with no declared
/// decomposition, where the value crosses whole under whatever arity layers the
/// boundary reads off it.
///
/// This is what lets an adapter consume the mechanism without inventing
/// declarations (#444 §2). Cbindgen declares no decompositions at all, so
/// without it there is no tree to lower for the great majority of crossings —
/// and an adapter that has to special-case "no plan" is back to walking
/// `TypeRef`, which is the walk the tree exists to remove.
///
/// Uses [`TypeRef::layer_stack`](prebindgen_flat::flat::TypeRef::layer_stack)'s
/// reading of the layers, which is deliberately bounded — at most one `Option`,
/// then at most one run. A boundary that accepts more says so with
/// [`ordinary_with`] rather than by walking the type itself.
pub fn ordinary(ty: &prebindgen_flat::flat::TypeRef) -> OutNode {
    ordinary_with(ty, &mut |t| {
        let (shape, core) = t.layer_stack();
        match shape {
            UnfoldShape::Base => None,
            UnfoldShape::Optional(..) => t
                .optional_inner()
                .map(|inner| (OrdinaryLayer::Optional, inner.clone())),
            UnfoldShape::Iterable(_) => {
                let _ = core;
                t.sequence_elem().map(|elem| {
                    (
                        OrdinaryLayer::Sequence {
                            borrowed: t.borrow_target().is_some(),
                        },
                        elem.clone(),
                    )
                })
            }
        }
    })
}

/// [`ordinary`] under one boundary's reading of the arity layers.
///
/// `peel` answers what the outermost layer of a type is and what it wraps, or
/// `None` when the value crosses whole. The recursion is here rather than in
/// the adapter for the same reason every other walk is: which layers a boundary
/// accepts is node-level policy, and rebuilding the descent around it is not.
///
/// C accepts shapes the decomposition boundary does not — nested `Option`s,
/// each spending a niche, and a shared-slice borrow as a run — and says so here
/// instead of classifying the type itself.
pub fn ordinary_with(
    ty: &prebindgen_flat::flat::TypeRef,
    peel: &mut dyn FnMut(
        &prebindgen_flat::flat::TypeRef,
    ) -> Option<(OrdinaryLayer, prebindgen_flat::flat::TypeRef)>,
) -> OutNode {
    match peel(ty) {
        Some((layer, inner)) => {
            let inner = Box::new(ordinary_with(&inner, peel));
            OutNode {
                ty: ty.clone(),
                kind: match layer {
                    OrdinaryLayer::Optional => TransformKind::Optional { op: (), inner },
                    OrdinaryLayer::Sequence { borrowed } => TransformKind::Sequence {
                        op: OutRun { borrowed },
                        inner,
                    },
                },
            }
        }
        None => OutNode {
            ty: ty.clone(),
            kind: TransformKind::Leaf(OutLeaf {
                nullable: false,
                identity: false,
                reach: OutReach::Accessor,
            }),
        },
    }
}

/// Apply one adapter's converter selection, producing the tree every later pass
/// reads: each subtree the adapter claims is replaced by a leaf carrying the
/// reading of the converter it chose.
///
/// `claim` answers with that reading, or `None` to recurse. It is asked before
/// anything below a node is visited, and it sees the edge the node was reached
/// by — a converter is chosen for a value *in a position*, not for a type.
///
/// The point of returning a **tree** rather than taking the policy at each pass
/// is that there is then only one decision to disagree with: dependencies,
/// layout, conversion and cleanup all read this value. A policy consulted twice
/// can answer differently the second time; a tree cannot.
pub fn select(
    root: &OutNode,
    claim: &mut dyn FnMut(&OutNode, Option<&OutLink>) -> Option<prebindgen_flat::flat::TypeRef>,
) -> OutNode {
    root.lower(&mut Select { claim })
        .expect("selecting over a built tree cannot fail")
}

/// The lowerer behind [`select`]: it rebuilds the tree it walks, replacing a
/// claimed subtree with the leaf that stands for its converter.
struct Select<'a> {
    claim: &'a mut dyn FnMut(&OutNode, Option<&OutLink>) -> Option<prebindgen_flat::flat::TypeRef>,
}

impl TransformLowerer<OutOfRust> for Select<'_> {
    type Value = OutNode;
    type Error = std::convert::Infallible;

    fn descend(
        &mut self,
        node: &OutNode,
        link: Option<&OutLink>,
    ) -> Result<crate::transform::Descend<OutNode>, Self::Error> {
        Ok(match (self.claim)(node, link) {
            Some(selected) => crate::transform::Descend::Atomic(OutNode {
                ty: selected,
                kind: TransformKind::Leaf(OutLeaf {
                    nullable: false,
                    identity: false,
                    // How the claimed value is REACHED is unchanged by claiming
                    // it: the links above still say, and the last step says
                    // which kind of chain they are.
                    reach: match link.and_then(|l| l.steps.last()) {
                        Some(step) if step.is_field() => OutReach::Field,
                        _ => OutReach::Accessor,
                    },
                }),
            }),
            None => crate::transform::Descend::Recurse,
        })
    }

    fn leaf(&mut self, node: &OutNode, op: &OutLeaf) -> Result<OutNode, Self::Error> {
        Ok(OutNode {
            ty: node.ty.clone(),
            kind: TransformKind::Leaf(OutLeaf {
                nullable: op.nullable,
                identity: op.identity,
                reach: op.reach.clone(),
            }),
        })
    }

    fn product(
        &mut self,
        node: &OutNode,
        op: &OutProduct,
        children: Lowered<'_, OutOfRust, OutNode>,
    ) -> Result<OutNode, Self::Error> {
        Ok(OutNode {
            ty: node.ty.clone(),
            kind: TransformKind::Product {
                op: op.clone(),
                children: rebuilt(children),
            },
        })
    }

    fn choice(
        &mut self,
        node: &OutNode,
        op: &OutChoice,
        variants: Lowered<'_, OutOfRust, OutNode>,
    ) -> Result<OutNode, Self::Error> {
        Ok(OutNode {
            ty: node.ty.clone(),
            kind: TransformKind::Choice {
                op: op.clone(),
                variants: rebuilt(variants),
            },
        })
    }

    fn optional(
        &mut self,
        node: &OutNode,
        _op: &(),
        _inner: &OutNode,
        value: OutNode,
    ) -> Result<OutNode, Self::Error> {
        Ok(OutNode {
            ty: node.ty.clone(),
            kind: TransformKind::Optional {
                op: (),
                inner: Box::new(value),
            },
        })
    }

    fn sequence(
        &mut self,
        node: &OutNode,
        op: &OutRun,
        _inner: &OutNode,
        value: OutNode,
    ) -> Result<OutNode, Self::Error> {
        Ok(OutNode {
            ty: node.ty.clone(),
            kind: TransformKind::Sequence {
                op: op.clone(),
                inner: Box::new(value),
            },
        })
    }
}

/// Children put back on the links they came in on.
fn rebuilt(children: Lowered<'_, OutOfRust, OutNode>) -> Vec<OutChild> {
    children
        .into_iter()
        .map(|(child, node)| OutChild {
            link: child.link.clone(),
            node,
        })
        .collect()
}
