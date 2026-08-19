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
    type Sequence = InRun;
    type Link = InLink;
}

/// One node of an into-Rust construction.
pub type InNode = TransformNode<IntoRust>;
/// One argument of an into-Rust product, or one arm of a choice.
pub type InChild = TransformChild<IntoRust>;

/// One wire slot: where it sits in the foreign signature and what it is called
/// there.
///
/// The single description of a slot. [`wire_leaves`] turns the slots a tree
/// names into [`FoldPlan::leaves`](super::FoldPlan::leaves), so a slot exists
/// exactly where the node that uses it says so.
///
/// What a slot **carries** is not stored here: an argument slot carries its own
/// node's [`ty`](TransformNode::ty), a selector an `i32` and a presence flag a
/// `bool` by definition, and the two slots that carry something else — an
/// `Option` layer's payload and a run — say so on the layer. Storing it twice
/// would let a node and its slot disagree about one type.
#[derive(Clone)]
pub struct InSlot {
    /// Position in the foreign signature, and the index of the caller's
    /// decoded local.
    pub slot: usize,
    /// The slot's foreign-side parameter name.
    pub name: syn::Ident,
}

/// Where one decoded value comes from.
// large_enum_variant: a plan has a handful of leaves, and boxing `InSlot` to
// even the arms out would only put an indirection between a node and the slot
// it names (same trade-off as `DeconRecord`).
#[allow(clippy::large_enum_variant)]
#[derive(Clone)]
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
#[derive(Clone)]
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
#[derive(Clone)]
pub struct InChoice {
    /// The `i32` slot. Arm `i` is taken when it reads `i`; under an
    /// [`Optional`](InPresence) layer `-1` additionally means absent.
    pub selector: InSlot,
}

/// How an `Option<…>` layer decides whether its value is present. The three
/// forms differ in what crosses, which is why the layer names its own slot
/// rather than inheriting one.
// large_enum_variant: one presence per layer, and boxing the payload's reading
// to even the arms out would only put an indirection between a layer and the
// slot it names (same trade-off as `InLeaf`).
#[allow(clippy::large_enum_variant)]
#[derive(Clone)]
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
    Payload {
        slot: InSlot,
        /// The `Option<…>` the slot carries. Not the layer node's
        /// [`ty`](TransformNode::ty): the node produces `Option<Target>` while
        /// the slot carries the constructor's own argument, optionally.
        ty: prebindgen_flat::flat::TypeRef,
    },
}

/// A run's own wire slot: one value carrying the whole collection, which the
/// layer iterates.
#[derive(Clone)]
pub struct InRun {
    pub slot: InSlot,
    /// The collection the slot carries. Not the layer node's
    /// [`ty`](TransformNode::ty), for the reason
    /// [`InPresence::Payload`] gives.
    pub ty: prebindgen_flat::flat::TypeRef,
}

/// How one child hangs off its parent.
#[derive(Clone)]
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
    fn take(&mut self, s: &InSlot, ty: prebindgen_flat::flat::TypeRef) {
        self.0.push((
            s.slot,
            FoldLeaf {
                name: s.name.clone(),
                ty,
            },
        ));
    }
}

impl TransformLowerer<IntoRust> for CollectSlots<'_> {
    type Value = ();
    type Error = std::convert::Infallible;

    fn leaf(&mut self, node: &InNode, op: &InLeaf) -> Result<(), Self::Error> {
        // A bound argument reads what its layer unwrapped; that slot is the
        // layer's, contributed there.
        if let InLeaf::Slot { slot, .. } = op {
            self.take(slot, node.ty.clone());
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
        // The selector: composed, and placeless by construction.
        self.take(
            &op.selector,
            prebindgen_flat::flat::TypeRef::scalar(prebindgen_flat::flat::ScalarKind::I32),
        );
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
            // A presence flag no source wrote — placeless by construction.
            InPresence::Flag(s) => self.take(
                s,
                prebindgen_flat::flat::TypeRef::scalar(prebindgen_flat::flat::ScalarKind::Bool),
            ),
            InPresence::Payload { slot, ty } => self.take(slot, ty.clone()),
        }
        Ok(())
    }

    fn sequence(
        &mut self,
        _node: &InNode,
        op: &InRun,
        _inner: &InNode,
        _value: (),
    ) -> Result<(), Self::Error> {
        self.take(&op.slot, op.ty.clone());
        Ok(())
    }
}

/// What a construction needs converters for, split by where the need comes
/// from.
///
/// Derived by [`dependencies`], through the same traversal an adapter lowers
/// with — so a subtree claimed whole by a direct converter contributes nothing
/// here either.
#[derive(Default)]
pub struct Dependencies {
    /// Types the binding **demands** an input converter for: the values a
    /// constructor actually takes, and the payload an arity layer decodes.
    pub required: Vec<prebindgen_flat::flat::TypeRef>,
    /// Wire primitives a **layout** contributes rather than a source value: a
    /// dispatch's `i32` selector and an `Option` layer's `bool` presence flag.
    ///
    /// Named apart because which of them exist — and whether they exist at all
    /// — is the adapter's choice of physical representation, not something the
    /// transformation says. They are required today so the current layout keeps
    /// resolving; an adapter that picks its own selector representation should
    /// not inherit this one's (#444 §1).
    pub intrinsic: Vec<prebindgen_flat::flat::TypeRef>,
}

/// What a construction depends on, read off the tree rather than off the flat
/// signature.
///
/// Ask it of the tree the adapter actually lowers, for the reason
/// [`unfold::dependencies`](crate::unfold::dependencies) gives: a claimed
/// subtree is already a selected leaf by then.
pub fn dependencies(tree: &InNode) -> Dependencies {
    tree.lower(&mut CollectDeps)
        .expect("collecting dependencies of a built tree cannot fail")
}

/// The lowerer behind [`dependencies`]: each node states the crossings it
/// needs, and nothing states one twice.
struct CollectDeps;

impl CollectDeps {
    fn merge(parts: Lowered<'_, IntoRust, Dependencies>) -> Dependencies {
        let mut out = Dependencies::default();
        for (_, mut part) in parts {
            out.required.append(&mut part.required);
            out.intrinsic.append(&mut part.intrinsic);
        }
        out
    }

    fn scalar(kind: prebindgen_flat::flat::ScalarKind) -> prebindgen_flat::flat::TypeRef {
        prebindgen_flat::flat::TypeRef::scalar(kind)
    }
}

impl TransformLowerer<IntoRust> for CollectDeps {
    type Value = Dependencies;
    type Error = std::convert::Infallible;

    fn leaf(&mut self, node: &InNode, op: &InLeaf) -> Result<Dependencies, Self::Error> {
        Ok(match op {
            InLeaf::Slot { .. } => Dependencies {
                required: vec![node.ty.clone()],
                intrinsic: Vec::new(),
            },
            // A bound argument reads what its layer decoded; that crossing is
            // the layer's, stated there.
            InLeaf::Bound => Dependencies::default(),
        })
    }

    fn product(
        &mut self,
        _node: &InNode,
        _op: &InProduct,
        children: Lowered<'_, IntoRust, Dependencies>,
    ) -> Result<Dependencies, Self::Error> {
        Ok(Self::merge(children))
    }

    fn choice(
        &mut self,
        _node: &InNode,
        _op: &InChoice,
        variants: Lowered<'_, IntoRust, Dependencies>,
    ) -> Result<Dependencies, Self::Error> {
        let mut out = Self::merge(variants);
        out.intrinsic
            .push(Self::scalar(prebindgen_flat::flat::ScalarKind::I32));
        Ok(out)
    }

    fn optional(
        &mut self,
        _node: &InNode,
        op: &InPresence,
        _inner: &InNode,
        value: Dependencies,
    ) -> Result<Dependencies, Self::Error> {
        let mut out = value;
        match op {
            // Absence rides the dispatch's own selector.
            InPresence::Selector => {}
            InPresence::Flag(_) => out
                .intrinsic
                .push(Self::scalar(prebindgen_flat::flat::ScalarKind::Bool)),
            InPresence::Payload { ty, .. } => out.required.push(ty.clone()),
        }
        Ok(out)
    }

    fn sequence(
        &mut self,
        _node: &InNode,
        op: &InRun,
        _inner: &InNode,
        value: Dependencies,
    ) -> Result<Dependencies, Self::Error> {
        let mut out = value;
        out.required.push(op.ty.clone());
        Ok(out)
    }
}

/// Error returned by [`select`] when the adapter claims a subtree with no wire
/// slot to inherit.
///
/// A subtree made entirely of [`InLeaf::Bound`] values carries no position on
/// the foreign signature — those values are bound by a containing layer, not
/// passed as independent wire slots.  A converter must land on a wire slot, so
/// claiming such a subtree is always an error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoundOnlySubtreeClaimed {
    /// The construction that was claimed, so the report says which one — the
    /// same reason the planning errors carry a target and a node path.
    pub claimed: String,
}

impl std::fmt::Display for BoundOnlySubtreeClaimed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "input expansion: the claimed construction of `{}` has no wire slot — its subtree is \
             entirely layer-bound values, which a containing layer supplies rather than the \
             foreign signature, so there is no position for a converter to land on",
            self.claimed
        )
    }
}

impl std::error::Error for BoundOnlySubtreeClaimed {}

/// Apply one adapter's converter selection, producing the tree every later pass
/// reads: each subtree the adapter claims becomes one wire slot carrying the
/// reading of the converter it chose.
///
/// `claim` answers with that reading, or `None` to recurse, and sees the edge
/// the node was reached by.
///
/// Unlike the output direction, selecting here **changes the signature**: a
/// claimed subtree's arguments, selector and presence slots collapse into the
/// single value that crosses instead. The surviving slots are renumbered into a
/// dense run in their original order, and a claimed subtree takes the earliest
/// position it replaced — so an adapter's choice of converter is also its
/// choice of layout, which is what #444 §1 asks for.
///
/// # Errors
///
/// Returns [`BoundOnlySubtreeClaimed`] when the adapter claims a subtree whose
/// only leaves are [`InLeaf::Bound`] — values supplied by a containing layer
/// rather than independent wire slots.  Such a subtree has no position to
/// inherit on the foreign signature.
pub fn select(
    tree: &InNode,
    claim: &mut dyn FnMut(&InNode, Option<&InLink>) -> Option<prebindgen_flat::flat::TypeRef>,
) -> Result<InNode, BoundOnlySubtreeClaimed> {
    tree.lower(&mut Select { claim })
        .map(|selected| renumber(&selected))
}

/// The lowerer behind [`select`]: it rebuilds the tree, replacing a claimed
/// subtree with the slot that stands for its converter.
struct Select<'a> {
    claim: &'a mut dyn FnMut(&InNode, Option<&InLink>) -> Option<prebindgen_flat::flat::TypeRef>,
}

impl Select<'_> {
    /// The slot a claimed subtree inherits: the earliest one it replaced, whose
    /// name is what the foreign signature already called that position — and
    /// whether that position is `Option`-wrapped by selector presence.
    ///
    /// The wrapping belongs to the POSITION, not to the value in it. A subtree
    /// inside a live `Choice` arm is absent whenever another arm is selected,
    /// so whatever crosses there must still be able to say "not this one".
    /// Structural slots — a selector, a presence flag, a run length — are never
    /// selector-wrapped arguments, so they contribute `false`; that is also the
    /// answer when the claim swallows the whole choice, which then sits under
    /// no selector at all.
    fn first_slot(node: &InNode) -> Option<(InSlot, bool)> {
        match &node.kind {
            TransformKind::Leaf(InLeaf::Slot { slot, wrapped }) => Some((slot.clone(), *wrapped)),
            TransformKind::Leaf(InLeaf::Bound) => None,
            TransformKind::Product { children, .. } => children
                .iter()
                .filter_map(|c| Self::first_slot(&c.node))
                .min_by_key(|(s, _)| s.slot),
            TransformKind::Choice { op, variants } => variants
                .iter()
                .filter_map(|v| Self::first_slot(&v.node))
                .chain(std::iter::once((op.selector.clone(), false)))
                .min_by_key(|(s, _)| s.slot),
            TransformKind::Optional { op, inner } => {
                let own = match op {
                    InPresence::Selector => None,
                    InPresence::Flag(s) => Some((s.clone(), false)),
                    InPresence::Payload { slot, .. } => Some((slot.clone(), false)),
                };
                Self::first_slot(inner)
                    .into_iter()
                    .chain(own)
                    .min_by_key(|(s, _)| s.slot)
            }
            TransformKind::Sequence { op, inner } => Self::first_slot(inner)
                .into_iter()
                .chain(std::iter::once((op.slot.clone(), false)))
                .min_by_key(|(s, _)| s.slot),
        }
    }
}

impl TransformLowerer<IntoRust> for Select<'_> {
    type Value = InNode;
    type Error = BoundOnlySubtreeClaimed;

    fn descend(
        &mut self,
        node: &InNode,
        link: Option<&InLink>,
    ) -> Result<crate::transform::Descend<InNode>, Self::Error> {
        let Some(selected) = (self.claim)(node, link) else {
            return Ok(crate::transform::Descend::Recurse);
        };
        let Some((slot, wrapped)) = Self::first_slot(node) else {
            return Err(BoundOnlySubtreeClaimed {
                claimed: node.ty.key().to_string(),
            });
        };
        // The leaf's type and its `wrapped` flag have to say one thing: the
        // type is what the wire declares, the flag is whether the emitter
        // unwraps it. A structural node is offered WITHOUT the position's
        // `Option` — an arm's constructor product is `ZKeyExpr` while its
        // argument leaf is `Option<String>` — so the reading a claim returns
        // there needs the layer put back on.
        //
        // Idempotent, because an already-optional reading at a wrapped position
        // can only be the position type an existing leaf handed straight back:
        // a selector-wrapped position never carries an optional value, since a
        // parameter that is itself `Option` is built unwrapped (`wrapped =
        // dispatched && !popt`).
        let ty = if wrapped && selected.optional_inner().is_none() {
            selected.optional()
        } else {
            selected
        };
        let leaf = InNode {
            ty,
            kind: TransformKind::Leaf(InLeaf::Slot { slot, wrapped }),
        };
        // A leaf claim replaces a leaf: it is consumed as an ordinary
        // constructor argument, and the constructor above it already unwraps
        // selector presence and produces the `Result`.
        //
        // A STRUCTURAL claim replaces a node that produced a value — an arm's
        // constructor, an arity layer, a nested construction — and whatever
        // consumes it still expects one. `emit_fold` promises
        // `Result<target, String>`, so dropping to a bare leaf would hand a
        // choice arm an `Option<T>` where its sibling arms give `Result<T, _>`.
        // An identity product over the claimed leaf says exactly what a claim
        // means — this one value IS the target — and its lowering already
        // unwraps the presence (missing ⇒ `Err`) and lifts the value into `Ok`.
        Ok(crate::transform::Descend::Atomic(match &node.kind {
            TransformKind::Leaf(_) => leaf,
            _ => InNode {
                ty: node.ty.clone(),
                kind: TransformKind::Product {
                    // Not cloned: the claim says the decoded value is the
                    // target, so there is no borrow to preserve behind it.
                    op: InProduct::Identity { clone: false },
                    children: vec![InChild {
                        link: InLink { by_ref: false },
                        node: leaf,
                    }],
                },
            },
        }))
    }

    fn leaf(&mut self, node: &InNode, op: &InLeaf) -> Result<InNode, Self::Error> {
        Ok(InNode {
            ty: node.ty.clone(),
            kind: TransformKind::Leaf(match op {
                InLeaf::Slot { slot, wrapped } => InLeaf::Slot {
                    slot: slot.clone(),
                    wrapped: *wrapped,
                },
                InLeaf::Bound => InLeaf::Bound,
            }),
        })
    }

    fn product(
        &mut self,
        node: &InNode,
        op: &InProduct,
        children: Lowered<'_, IntoRust, InNode>,
    ) -> Result<InNode, Self::Error> {
        Ok(InNode {
            ty: node.ty.clone(),
            kind: TransformKind::Product {
                op: op.clone(),
                children: rebuilt(children),
            },
        })
    }

    fn choice(
        &mut self,
        node: &InNode,
        op: &InChoice,
        variants: Lowered<'_, IntoRust, InNode>,
    ) -> Result<InNode, Self::Error> {
        Ok(InNode {
            ty: node.ty.clone(),
            kind: TransformKind::Choice {
                op: op.clone(),
                variants: rebuilt(variants),
            },
        })
    }

    fn optional(
        &mut self,
        node: &InNode,
        op: &InPresence,
        _inner: &InNode,
        value: InNode,
    ) -> Result<InNode, Self::Error> {
        Ok(InNode {
            ty: node.ty.clone(),
            kind: TransformKind::Optional {
                op: op.clone(),
                inner: Box::new(value),
            },
        })
    }

    fn sequence(
        &mut self,
        node: &InNode,
        op: &InRun,
        _inner: &InNode,
        value: InNode,
    ) -> Result<InNode, Self::Error> {
        Ok(InNode {
            ty: node.ty.clone(),
            kind: TransformKind::Sequence {
                op: op.clone(),
                inner: Box::new(value),
            },
        })
    }
}

/// Children put back on the links they came in on.
fn rebuilt(children: Lowered<'_, IntoRust, InNode>) -> Vec<InChild> {
    children
        .into_iter()
        .map(|(child, node)| InChild {
            link: child.link.clone(),
            node,
        })
        .collect()
}

/// Close the gaps a selection leaves: the surviving slots keep their order and
/// take positions `0..n`.
///
/// Order rather than identity, because the foreign signature is a sequence —
/// what a caller passes second must still be what the wrapper reads second.
fn renumber(tree: &InNode) -> InNode {
    let mut old: Vec<usize> = Vec::new();
    collect_slots(tree, &mut old);
    old.sort_unstable();
    let dense: std::collections::HashMap<usize, usize> = old
        .into_iter()
        .enumerate()
        .map(|(new, o)| (o, new))
        .collect();
    rewrite_slots(tree, &dense)
}

fn collect_slots(node: &InNode, out: &mut Vec<usize>) {
    match &node.kind {
        TransformKind::Leaf(InLeaf::Slot { slot, .. }) => out.push(slot.slot),
        TransformKind::Leaf(InLeaf::Bound) => {}
        TransformKind::Product { children, .. } => {
            for c in children {
                collect_slots(&c.node, out);
            }
        }
        TransformKind::Choice { op, variants } => {
            out.push(op.selector.slot);
            for v in variants {
                collect_slots(&v.node, out);
            }
        }
        TransformKind::Optional { op, inner } => {
            match op {
                InPresence::Selector => {}
                InPresence::Flag(s) => out.push(s.slot),
                InPresence::Payload { slot, .. } => out.push(slot.slot),
            }
            collect_slots(inner, out);
        }
        TransformKind::Sequence { op, inner } => {
            out.push(op.slot.slot);
            collect_slots(inner, out);
        }
    }
}

fn rewrite_slots(node: &InNode, dense: &std::collections::HashMap<usize, usize>) -> InNode {
    let at = |s: &InSlot| InSlot {
        slot: dense[&s.slot],
        name: s.name.clone(),
    };
    InNode {
        ty: node.ty.clone(),
        kind: match &node.kind {
            TransformKind::Leaf(InLeaf::Slot { slot, wrapped }) => {
                TransformKind::Leaf(InLeaf::Slot {
                    slot: at(slot),
                    wrapped: *wrapped,
                })
            }
            TransformKind::Leaf(InLeaf::Bound) => TransformKind::Leaf(InLeaf::Bound),
            TransformKind::Product { op, children } => TransformKind::Product {
                op: op.clone(),
                children: children
                    .iter()
                    .map(|c| InChild {
                        link: c.link.clone(),
                        node: rewrite_slots(&c.node, dense),
                    })
                    .collect(),
            },
            TransformKind::Choice { op, variants } => TransformKind::Choice {
                op: InChoice {
                    selector: at(&op.selector),
                },
                variants: variants
                    .iter()
                    .map(|v| InChild {
                        link: v.link.clone(),
                        node: rewrite_slots(&v.node, dense),
                    })
                    .collect(),
            },
            TransformKind::Optional { op, inner } => TransformKind::Optional {
                op: match op {
                    InPresence::Selector => InPresence::Selector,
                    InPresence::Flag(s) => InPresence::Flag(at(s)),
                    InPresence::Payload { slot, ty } => InPresence::Payload {
                        slot: at(slot),
                        ty: ty.clone(),
                    },
                },
                inner: Box::new(rewrite_slots(inner, dense)),
            },
            TransformKind::Sequence { op, inner } => TransformKind::Sequence {
                op: InRun {
                    slot: at(&op.slot),
                    ty: op.ty.clone(),
                },
                inner: Box::new(rewrite_slots(inner, dense)),
            },
        },
    }
}
