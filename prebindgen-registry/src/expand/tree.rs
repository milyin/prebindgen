//! The into-Rust direction of the shared transformation tree (#442): how the
//! values that cross in are put back together into one Rust value.
//!
//! The tree is the plan. A constructor call is a [product](InProduct) over its
//! arguments, a selector dispatch is a [choice](InChoice) over its arms, the
//! `Option<…>` and `Vec<…>` a parameter is written with are the layers over
//! that ([`InPresence`], [`InRun`]), and a crossing value is a [leaf](InLeaf).
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

/// Where one decoded value comes from.
#[derive(Clone)]
pub enum InLeaf {
    /// A wire slot of its own.
    Slot {
        /// The position is `Option`-wrapped by **selector presence** (it belongs to
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
        /// How the child's decoded value becomes this node's value.
        ///
        /// A borrowed identity arm (`&T` parameter) is
        /// [`CloneDeref`](Lift::CloneDeref): the decoded value reaches the
        /// caller's handle, and the fold copies out of it rather than consuming
        /// it. A selection states its own — see [`Claim`].
        lift: Lift,
    },
}

/// How a choice node picks the arm that runs. The selector is a slot of its
/// own — a wire value no source wrote, contributed by this node.
#[derive(Clone)]
pub struct InChoice {
    /// A dispatch happens here. WHICH position signals it, and that the signal
    /// is an `i32` at all, is the layout's choice rather than this node's
    /// (#447 §1) — as is the convention that `-1` means absent under an
    /// [`Optional`](InPresence) layer.
    pub dispatch: (),
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
    /// An explicit flag decides, and the construction's arguments cross plain.
    /// Used for a constructor of two or more arguments, where riding the
    /// arguments' own `Option`s would box a nullable primitive on the wire.
    Flag,
    /// The layer decodes its own `Option<…>` slot and hands the payload to the
    /// single-argument construction under it — which reads it as
    /// [`InLeaf::Bound`], having no slot of its own.
    Payload {
        /// The `Option<…>` the position carries. Not the layer node's
        /// [`ty`](TransformNode::ty): the node produces `Option<Target>` while
        /// the slot carries the constructor's own argument, optionally.
        ty: prebindgen_flat::flat::TypeRef,
    },
}

/// A run's own wire slot: one value carrying the whole collection, which the
/// layer iterates.
#[derive(Clone)]
pub struct InRun {
    /// The collection the position carries. Not the layer node's
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
    pub fn leaf_args(&self) -> Option<Vec<()>> {
        let TransformKind::Product { children, .. } = &self.kind else {
            return None;
        };
        children
            .iter()
            .map(|c| match &c.node.kind {
                TransformKind::Leaf(InLeaf::Slot { .. }) => Some(()),
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
/// Derive a flat layout for a tree that has none — the tree a
/// [`select`] produced.
///
/// Selection replaces subtrees, so the signature it leaves is not the one the
/// plan was built with. Rather than renumber a signature the tree no longer
/// carries, an adapter claims positions over the selected tree: same walk,
/// same order, and the surviving values close ranks because nothing was
/// skipped. Names are positional here; an adapter that wants its own naming
/// builds its own layout the same way (#447 §1).
pub fn derive_layout(tree: &InNode) -> super::SlotLayout {
    struct Derive(super::SlotLayout);
    impl TransformLowerer<IntoRust> for Derive {
        type Value = ();
        type Error = std::convert::Infallible;

        fn descend(
            &mut self,
            node: &InNode,
            _link: Option<&InLink>,
        ) -> Result<crate::transform::Descend<()>, Self::Error> {
            let kind = match &node.kind {
                TransformKind::Leaf(InLeaf::Slot { .. }) => Some(super::SlotKind::Value),
                TransformKind::Choice { .. } => Some(super::SlotKind::Selector),
                TransformKind::Sequence { .. } => Some(super::SlotKind::Value),
                TransformKind::Optional { op, .. } => match op {
                    InPresence::Selector => None,
                    InPresence::Flag => Some(super::SlotKind::PresenceFlag),
                    InPresence::Payload { .. } => Some(super::SlotKind::PresencePayload),
                },
                TransformKind::Leaf(InLeaf::Bound) | TransformKind::Product { .. } => None,
            };
            if let Some(kind) = kind {
                let name = prebindgen_flat::types_util::ident(&format!("p{}", self.0.len()));
                self.0.claim(name, kind);
            }
            Ok(crate::transform::Descend::Recurse)
        }

        fn leaf(&mut self, _n: &InNode, _o: &InLeaf) -> Result<(), Self::Error> {
            Ok(())
        }
        fn product(
            &mut self,
            _n: &InNode,
            _o: &InProduct,
            _c: Lowered<'_, IntoRust, ()>,
        ) -> Result<(), Self::Error> {
            Ok(())
        }
        fn choice(
            &mut self,
            _n: &InNode,
            _o: &InChoice,
            _v: Lowered<'_, IntoRust, ()>,
        ) -> Result<(), Self::Error> {
            Ok(())
        }
        fn optional(
            &mut self,
            _n: &InNode,
            _o: &InPresence,
            _i: &InNode,
            _v: (),
        ) -> Result<(), Self::Error> {
            Ok(())
        }
        fn sequence(
            &mut self,
            _n: &InNode,
            _o: &InRun,
            _i: &InNode,
            _v: (),
        ) -> Result<(), Self::Error> {
            Ok(())
        }
    }
    let mut derive = Derive(super::SlotLayout::default());
    tree.lower(&mut derive)
        .expect("deriving a layout cannot fail");
    derive.0
}

pub fn wire_leaves(tree: &InNode, layout: &super::SlotLayout) -> Vec<FoldLeaf> {
    let mut types: Vec<prebindgen_flat::flat::TypeRef> = Vec::new();
    tree.lower(&mut CollectSlots(&mut types))
        .expect("collecting wire values cannot fail");
    assert_eq!(
        types.len(),
        layout.len(),
        "the tree carries one value per position the layout claimed"
    );
    types
        .into_iter()
        .enumerate()
        .map(|(slot, ty)| FoldLeaf {
            name: layout.name(slot).clone(),
            ty,
        })
        .collect()
}

/// The lowerer behind [`wire_leaves`]: each node contributes the slots it uses
/// and nothing else.
struct CollectSlots<'a>(&'a mut Vec<prebindgen_flat::flat::TypeRef>);

impl CollectSlots<'_> {
    /// Contribute the value at the next position. Pushed on the way DOWN, so
    /// the order is the one positions were claimed in — a dispatch's selector
    /// precedes the arms it selects between, which a bottom-up hook would
    /// reverse.
    fn take(&mut self, ty: prebindgen_flat::flat::TypeRef) {
        self.0.push(ty);
    }
}

impl TransformLowerer<IntoRust> for CollectSlots<'_> {
    type Value = ();
    type Error = std::convert::Infallible;

    /// Contribute on the way DOWN, so values land in the order their positions
    /// were claimed: a layer before its construction, a dispatch's selector
    /// before its arms.
    fn descend(
        &mut self,
        node: &InNode,
        _link: Option<&InLink>,
    ) -> Result<crate::transform::Descend<()>, Self::Error> {
        match &node.kind {
            // A bound argument reads what its layer unwrapped; that position is
            // the layer's, contributed there.
            TransformKind::Leaf(InLeaf::Slot { wrapped }) => {
                self.take(slot_wire_ty(&node.ty, *wrapped))
            }
            TransformKind::Leaf(InLeaf::Bound) | TransformKind::Product { .. } => {}
            // The selector: composed, and placeless by construction.
            TransformKind::Choice { .. } => self.take(prebindgen_flat::flat::TypeRef::scalar(
                prebindgen_flat::flat::ScalarKind::I32,
            )),
            TransformKind::Sequence { op, .. } => self.take(op.ty.clone()),
            TransformKind::Optional { op, .. } => match op {
                InPresence::Selector => {}
                // A presence flag no source wrote — placeless by construction.
                InPresence::Flag => self.take(prebindgen_flat::flat::TypeRef::scalar(
                    prebindgen_flat::flat::ScalarKind::Bool,
                )),
                InPresence::Payload { ty, .. } => self.take(ty.clone()),
            },
        }
        Ok(crate::transform::Descend::Recurse)
    }

    fn leaf(&mut self, _node: &InNode, _op: &InLeaf) -> Result<(), Self::Error> {
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
        _op: &InChoice,
        _variants: Lowered<'_, IntoRust, ()>,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    fn optional(
        &mut self,
        _node: &InNode,
        _op: &InPresence,
        _inner: &InNode,
        _value: (),
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    fn sequence(
        &mut self,
        _node: &InNode,
        _op: &InRun,
        _inner: &InNode,
        _value: (),
    ) -> Result<(), Self::Error> {
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
            // The crossing a converter must exist for is what the slot
            // carries, which is the payload plus any presence over it.
            InLeaf::Slot { wrapped, .. } => Dependencies {
                required: vec![slot_wire_ty(&node.ty, *wrapped)],
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
            InPresence::Flag => out
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
pub enum SelectError {
    /// The claimed subtree carries no wire slot to inherit — see the type's
    /// own note.
    BoundOnlySubtree {
        /// The construction that was claimed, so the report says which one —
        /// the same reason the planning errors carry a target and a node path.
        claimed: String,
    },
    /// An arity layer was claimed with a reading that is not of that layer's
    /// shape, so the value the layer binds cannot be derived from it.
    ///
    /// A layer iterates or unwraps the slot it is given: a run binds one
    /// element of the collection, an optional binds the payload of the
    /// `Option`. A reading with neither shape leaves nothing to bind, and the
    /// node under the layer would have to advertise a type its expression
    /// never produces.
    /// A claim stated [`Lift::Direct`] — the reading IS the value — for a
    /// position where the two differ.
    ///
    /// Exact rather than a guess: `Ok(value)` offers no coercion, because the
    /// `Result`'s type parameter is inferred rather than a coercion site. So a
    /// direct claim between two different types cannot be honoured by any
    /// spelling of the same expression.
    DirectLiftMismatch {
        /// The position that was claimed.
        claimed: String,
        /// What the claim binds, once selector presence or an arity layer has
        /// taken its share.
        bound: String,
        /// What that position owes.
        target: String,
    },
    /// A leaf was claimed with a lift whose result cannot be the value that
    /// position holds.
    ///
    /// The two deref lifts produce an owned value. A leaf whose declared type
    /// is a borrow needs the value borrowed *through* the reading instead,
    /// which no [`Lift`] states — so the claim is refused rather than lowered
    /// into a value of the wrong ownership.
    LeafLiftTarget {
        /// The position that was claimed.
        claimed: String,
        /// The lift the adapter asked for.
        lift: Lift,
    },
    LayerReadingShape {
        /// The layer that was claimed, as `"a run"` or `"an optional"`.
        layer: &'static str,
        /// The node the layer stands over.
        claimed: String,
        /// The reading the adapter answered with.
        reading: String,
    },
}

impl std::fmt::Display for SelectError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BoundOnlySubtree { claimed } => write!(
                f,
                "input expansion: the claimed construction of `{claimed}` has no wire slot — its \
                 subtree is entirely layer-bound values, which a containing layer supplies rather \
                 than the foreign signature, so there is no position for a converter to land on",
            ),
            Self::DirectLiftMismatch {
                claimed,
                bound,
                target,
            } => write!(
                f,
                "input expansion: the claim at `{claimed}` says its reading is the value \
                 (`Lift::Direct`), but it binds `{bound}` where that position owes `{target}` — \
                 state the operation between them, or claim a reading that is already the value",
            ),
            Self::LeafLiftTarget { claimed, lift } => write!(
                f,
                "input expansion: the leaf at `{claimed}` was claimed with `{lift:?}`, which \
                 produces an owned value, but that position holds a borrow — borrowing through \
                 the reading is not a lift the tree can state, so claim the value it borrows \
                 from instead",
            ),
            Self::LayerReadingShape {
                layer,
                claimed,
                reading,
            } => write!(
                f,
                "input expansion: `{layer}` over `{claimed}` was claimed with the reading \
                 `{reading}`, which is not of that layer's shape — the layer binds one value out \
                 of the slot it is given, and this reading has none to bind. Claim the \
                 construction under the layer instead, or answer with the layer's own shape",
            ),
        }
    }
}

impl std::error::Error for SelectError {}

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
/// Returns [`SelectError`] when the adapter claims a subtree whose
/// only leaves are [`InLeaf::Bound`] — values supplied by a containing layer
/// rather than independent wire slots.  Such a subtree has no position to
/// inherit on the foreign signature.
pub fn select(
    tree: &InNode,
    claim: &mut dyn FnMut(&InNode, Option<&InLink>) -> Option<Claim>,
) -> Result<InNode, SelectError> {
    tree.lower(&mut Select { claim })
}

/// The lowerer behind [`select`]: it rebuilds the tree, replacing a claimed
/// subtree with the slot that stands for its converter.
struct Select<'a> {
    claim: &'a mut dyn FnMut(&InNode, Option<&InLink>) -> Option<Claim>,
}

/// How a claimed reading becomes the value the claimed node declares.
///
/// The registry cannot infer this from the reading's spelling, and it stopped
/// trying: `Box<T>` owns its target and can be moved out of, jnigen's
/// `OwnedObject<T>` dereferences to storage Java still owns and can only be
/// cloned from, and the two can present the same target and the same deref
/// shape. The adapter chose the converter, so the adapter states the operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lift {
    /// The reading already IS the value — move it.
    Direct,
    /// Dereference and clone: the reading reaches storage it does not own, so
    /// the value has to be copied out of it. `&T` and non-owning adapter
    /// handles take this.
    CloneDeref,
    /// Dereference and move: the reading owns its target and hands it over.
    /// `Box<T>` takes this, and unlike [`CloneDeref`](Self::CloneDeref) it asks
    /// nothing of the target type.
    MoveDeref,
}

/// What an adapter answers when it claims a subtree: the reading of the
/// converter it chose, and how that reading yields the claimed node's value.
///
/// Two facts rather than one because they are independent — the same `&T` may
/// be cloned from or, behind a different wrapper, moved out of — and only the
/// adapter knows the second.
#[derive(Debug, Clone)]
pub struct Claim {
    /// The reading whose input converter decodes the claimed wire slot.
    pub reading: prebindgen_flat::flat::TypeRef,
    /// How that reading becomes the value the node declares.
    pub lift: Lift,
}

impl Claim {
    /// A claim whose reading is already the value.
    pub fn direct(reading: prebindgen_flat::flat::TypeRef) -> Self {
        Self {
            reading,
            lift: Lift::Direct,
        }
    }

    /// A claim whose reading must be dereferenced and cloned — the usual answer
    /// for a borrow or a non-owning handle.
    pub fn clone_deref(reading: prebindgen_flat::flat::TypeRef) -> Self {
        Self {
            reading,
            lift: Lift::CloneDeref,
        }
    }

    /// A claim whose reading owns its target and is dereferenced to move it.
    pub fn move_deref(reading: prebindgen_flat::flat::TypeRef) -> Self {
        Self {
            reading,
            lift: Lift::MoveDeref,
        }
    }
}

/// What a slot carries on the wire: its payload, plus the `Option` selector
/// presence adds when a live choice arm gates it.
///
/// Presence and the crossing type are **one fact** derived here, not two fields
/// that can disagree. A node's `ty` is always the value the position holds, and
/// the wire type follows from whether the position is gated — so the state this
/// used to be able to represent, a plain `T` beside `wrapped = true`, no longer
/// exists to be constructed (#447 §1).
pub fn slot_wire_ty(
    payload: &prebindgen_flat::flat::TypeRef,
    wrapped: bool,
) -> prebindgen_flat::flat::TypeRef {
    if wrapped {
        payload.optional()
    } else {
        payload.clone()
    }
}

/// Which arity layer a claim landed on — the two differ only in what they bind
/// and how they name themselves in a refusal.
#[derive(Clone, Copy)]
enum LayerKind {
    Optional,
    Sequence,
}

impl LayerKind {
    fn name(self) -> &'static str {
        match self {
            Self::Optional => "an optional",
            Self::Sequence => "a run",
        }
    }
}

/// The value a claimed arity layer binds out of its slot — what the emitter's
/// own operation on that slot yields, not what the model calls its element.
///
/// The two part company wherever a wrapper survives into the generated code.
/// The layer accessors read *through* `Box` and `Cow` because a destination
/// language sees an optional or a run either way; the emitter runs
/// `slot.into_iter()` and `match slot { Some(..) }` on the type as written, and
/// those answer to the wrappers. A node typed from the semantic answer alone
/// advertises a value its own expression never produces — invisible to the
/// construct emitter, which reads only the `clone` bit, and wrong for the
/// adapter lowerers this tree exists for.
///
/// A run binds a **borrowed** element unless the reading is an owned `Vec`:
///
/// * `&[T]`, `&Vec<T>` — iterating a borrow cannot move out;
/// * `Cow<'_, [T]>` — iterates as its borrowed slice whatever it holds, so the
///   item is `&T` even though the model's element is `T`;
/// * `[T]` — a slice is never owned by value;
/// * `Vec<T>`, `Box<Vec<T>>` — these move their elements out (a `Box` derefs
///   into the `Vec`, which is consumed).
///
/// An optional binds the payload of a `match`, so its reading has to *be* an
/// `Option` and not merely denote one: `Box<Option<T>>` reads as optional to
/// the model, but `match` against `Some(..)` does not see through the `Box`.
/// Nothing plans such a slot — a payload layer is built as `pty.optional()` —
/// so only a claim can introduce one, and it is refused.
fn layer_item(
    reading: &prebindgen_flat::flat::TypeRef,
    layer: LayerKind,
    node: &InNode,
) -> Result<prebindgen_flat::flat::TypeRef, SelectError> {
    let item = match layer {
        LayerKind::Optional => reading
            .optional_inner()
            // The `match` is on the slot as written, so a wrapper between it
            // and the `Option` is not looked through.
            .filter(|_| matches!(reading.kind(), prebindgen_flat::flat::TypeKind::Optional(_)))
            .cloned(),
        LayerKind::Sequence => {
            // A `Cow` cannot be moved out of, so `into_iter()` on one reaches
            // its borrowed side or does not compile at all: `Cow<'_, [T]>`
            // iterates the slice and yields `&T`, while `Cow<'_, Vec<T>>` is
            // rejected outright ("cannot move out of dereference"). A slice is
            // likewise never owned by value. Everything else that survives to
            // here is a `Vec` the layer consumes — `Box<Vec<T>>` included,
            // since a `Box` does hand its contents over.
            let cow = reading.erased_wrappers().contains(&"Cow");
            match reading.unwrapped().kind() {
                prebindgen_flat::flat::TypeKind::Vec(_) if cow => None,
                _ => {
                    let by_ref = cow
                        || reading.borrow_target().is_some()
                        || matches!(
                            reading.unwrapped().kind(),
                            prebindgen_flat::flat::TypeKind::Slice(_)
                        );
                    let elem = match reading.borrow_target() {
                        Some(collection) => collection.sequence_elem(),
                        None => reading.sequence_elem(),
                    };
                    elem.map(|e| if by_ref { e.borrowed() } else { e.clone() })
                }
            }
        }
    };
    item.ok_or_else(|| SelectError::LayerReadingShape {
        layer: layer.name(),
        claimed: node.ty.key().to_string(),
        reading: reading.key().to_string(),
    })
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
    fn first_slot(node: &InNode) -> Option<bool> {
        match &node.kind {
            TransformKind::Leaf(InLeaf::Slot { wrapped }) => Some(*wrapped),
            TransformKind::Leaf(InLeaf::Bound) => None,
            // The FIRST position the subtree occupies, in claim order — which
            // is this walk's order, so the earliest is simply the first found.
            TransformKind::Product { children, .. } => {
                children.iter().find_map(|c| Self::first_slot(&c.node))
            }
            // A dispatch claims its selector before its arms, and a selector is
            // never a selector-wrapped argument.
            TransformKind::Choice { .. } => Some(false),
            TransformKind::Optional { op, inner } => match op {
                // The layer claims nothing; the dispatch under it does.
                InPresence::Selector => Self::first_slot(inner),
                // A flag and a payload are structural, so neither is wrapped.
                InPresence::Flag | InPresence::Payload { .. } => Some(false),
            },
            TransformKind::Sequence { .. } => Some(false),
        }
    }
}

impl TransformLowerer<IntoRust> for Select<'_> {
    type Value = InNode;
    type Error = SelectError;

    fn descend(
        &mut self,
        node: &InNode,
        link: Option<&InLink>,
    ) -> Result<crate::transform::Descend<InNode>, Self::Error> {
        let Some(Claim {
            reading: selected,
            lift,
        }) = (self.claim)(node, link)
        else {
            return Ok(crate::transform::Descend::Recurse);
        };
        let Some(wrapped) = Self::first_slot(node) else {
            return Err(SelectError::BoundOnlySubtree {
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
        // ── The claim site, normalized ──────────────────────────────────
        //
        // Four facts, so that validating and rebuilding a claim never has to
        // reason about a branch's particular spelling:
        //
        // * `position_ty` — the wire slot, selector presence included;
        // * `bound_ty`    — what is left once presence or an arity layer has
        //                   bound one value, which is the lift's input;
        // * `target_ty`   — what the node owes with that same positional
        //                   wrapper removed, which is the lift's output;
        // * `lift`        — the operation between them, stated by the adapter.
        //
        // The emitter unwraps before it lifts, so a selector-wrapped position's
        // `Option` belongs to neither end of the operation. Reading `node.ty`
        // directly is what let an owned-producing lift through onto a borrowed
        // argument hidden under presence.
        //
        // A claim answers the PAYLOAD — the value the position holds — and the
        // wire follows from whether that position is gated. One derivation, so
        // the two cannot be stated inconsistently; before the invariant this
        // needed an idempotence rule to decide whether an `Option` in the
        // reading was the position's or the value's own (#447 §1).
        let (bound_ty, target_ty) = match &node.kind {
            // Both ends are payloads already.
            TransformKind::Leaf(_) => (selected.clone(), node.ty.clone()),
            TransformKind::Optional { .. } => (
                layer_item(&selected, LayerKind::Optional, node)?,
                node.core().ty.clone(),
            ),
            TransformKind::Sequence { .. } => (
                layer_item(&selected, LayerKind::Sequence, node)?,
                node.core().ty.clone(),
            ),
            // A structural node is offered without the position's `Option`, so
            // the claim's reading is already the lift's input.
            _ => (selected.clone(), node.ty.clone()),
        };

        // ── …and validated once, in those terms ─────────────────────────
        match lift {
            // `Direct` says the bound value IS the target, and nothing coerces
            // through `Ok(..)` — the `Result`'s parameter is inferred, not a
            // coercion site. So this is a contradiction to detect, not a
            // spelling to guess about.
            Lift::Direct if bound_ty.key() != target_ty.key() => {
                return Err(SelectError::DirectLiftMismatch {
                    claimed: node.ty.key().to_string(),
                    bound: bound_ty.key().to_string(),
                    target: target_ty.key().to_string(),
                })
            }
            // Both deref lifts produce an owned value. A target that is a
            // borrow needs the value borrowed *through* the reading, which no
            // `Lift` states.
            Lift::CloneDeref | Lift::MoveDeref if target_ty.borrow_target().is_some() => {
                return Err(SelectError::LeafLiftTarget {
                    claimed: node.ty.key().to_string(),
                    lift,
                })
            }
            _ => {}
        }

        let leaf = InNode {
            // The payload, like every other slot leaf: the position's `Option`
            // is `wrapped`'s to add, and stating it here as well is the
            // disagreement the invariant removes.
            ty: selected.clone(),
            kind: TransformKind::Leaf(InLeaf::Slot { wrapped }),
        };
        // An identity over one value the enclosing node binds, typed with what
        // the node owes rather than what the claim reads: under a layer it
        // stands where the construction stood and `InNode::core` descends to
        // it, and under selector presence the `Option` is the position's, not
        // the value's.
        let identity_over = |bound: InNode| InNode {
            ty: target_ty.clone(),
            kind: TransformKind::Product {
                op: InProduct::Identity { lift },
                children: vec![InChild {
                    link: InLink { by_ref: false },
                    node: bound,
                }],
            },
        };
        let bound_leaf = || InNode {
            ty: bound_ty.clone(),
            kind: TransformKind::Leaf(InLeaf::Bound),
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
            // A leaf claimed `Direct` is the value already, and stays one slot
            // the enclosing construction reads. A leaf claimed with a deref
            // lift is NOT: the operation has to happen somewhere, and a bare
            // leaf has nowhere to put it — the constructor would receive the
            // reading itself. So the leaf gains the same identity node a
            // structural claim gets, which is the node that performs a lift.
            TransformKind::Leaf(_) if lift == Lift::Direct => leaf,
            TransformKind::Leaf(_) => identity_over(leaf),
            // An ARITY LAYER maps its inner value over a shape, and the claim
            // replaced the whole layer — its presence or length slots included.
            // Keeping the layer over an identity core is what preserves that
            // mapping: the layer unwraps the claimed slot and binds one inner
            // value, and the identity lifts that value to what the node
            // declares. Collapsing to a bare identity instead would hand
            // `Clone::clone(&*v)` an `Option<&T>`, which does not deref, and
            // would owe `Option<T>` while producing `Option<&T>`.
            //
            // The layer's slot carries the claimed reading, so its own
            // `Option` / collection IS the presence or the run — there is no
            // second one to reconcile. A selector-wrapped position cannot hold
            // an arity layer at all: `build_arg` refuses a recursive input
            // under a dispatched constructor variant, so nothing nests one
            // there.
            TransformKind::Optional { .. } => InNode {
                ty: node.ty.clone(),
                kind: TransformKind::Optional {
                    op: InPresence::Payload {
                        ty: selected.clone(),
                    },
                    inner: Box::new(identity_over(bound_leaf())),
                },
            },
            TransformKind::Sequence { .. } => InNode {
                ty: node.ty.clone(),
                kind: TransformKind::Sequence {
                    op: InRun {
                        ty: selected.clone(),
                    },
                    inner: Box::new(identity_over(bound_leaf())),
                },
            },
            // A base product or choice produces one value directly.
            _ => identity_over(leaf),
        }))
    }

    fn leaf(&mut self, node: &InNode, op: &InLeaf) -> Result<InNode, Self::Error> {
        Ok(InNode {
            ty: node.ty.clone(),
            kind: TransformKind::Leaf(match op {
                InLeaf::Slot { wrapped } => InLeaf::Slot { wrapped: *wrapped },
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
