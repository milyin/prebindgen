//! Resolved constructor-expansion plans.

/// Outer shape wrapping the core construct. The value-side analog of how the
/// `Option<_>` / `Vec<_>` wrapper converters compose at the wire.
///
/// The unified [`Shape`](prebindgen_flat::shape::Shape) layer stack: `Base`
/// builds the target directly from the decoded leaves (a single constructor of
/// any arity or a combined-selector dispatch); `Optional((), inner)` lifts that
/// over `Option<T>`/`Option<&T>` (`Some` ⇒ run `inner` on the unwrapped value
/// and re-wrap, `None` ⇒ `None`; inner is always `Base` today);
/// `Iterable(inner)` maps `inner` over each element of a `Vec<T>` (emit-ready
/// but not yet produced by `apply`). The `()` payload is unused here — only the
/// JNI adapter's `Shape<NullableKind>` carries per-layer data.
pub use prebindgen_flat::shape::Shape as FoldShape;

/// A resolved expansion for one `(function, parameter)`.
///
/// [`leaves`](Self::leaves) is readable but not settable from outside the
/// crate, and neither is a plan constructible there: it is collected from
/// [`Self::tree`] where the plan is built, and a caller that could set it could
/// make a tree-reading adapter and a signature-reading one see different
/// expansions.
pub struct FoldPlan {
    /// Owned type the core construct produces — what the underlying call needs
    /// (before any [`Self::shape`] wrapping). A **reading**: `emit.spell(&target)` in an emission callback
    /// for generated Rust, `target.key()` for a lookup.
    pub target: prebindgen_flat::flat::TypeRef,
    /// True when the original parameter was `&T` / `Option<&T>`: the call
    /// receives `&folded` (or `folded.as_ref()` when also optional). A
    /// call-site concern (the resolver's `&_` handler shares the inner
    /// converter the same way), not part of the fold.
    pub by_ref: bool,
    /// Flattened wire leaves, in foreign-signature order.
    ///
    /// A derived view of [`Self::tree`]: every slot is described by the node
    /// that uses it, and [`wire_leaves`](crate::expand::wire_leaves) collects
    /// them.
    pub(crate) leaves: Vec<FoldLeaf>,
    /// The plan itself: the `Option` / `Vec` layers the parameter is written
    /// with, and the construction under them (#442). Everything recursive about
    /// an expansion lives here — a constructor argument that is itself built
    /// has the same node kinds as the top level.
    pub(crate) tree: crate::expand::InNode,
    /// Where each value sits on the foreign signature — the compatibility
    /// projection the tree stops carrying (#447 §1).
    pub(crate) layout: SlotLayout,
    /// The **root** construction's dispatch position, when it has one.
    ///
    /// Recorded rather than searched for: a nested build has a selector of its
    /// own, so "the first selector in the layout" is a different question and
    /// answers the inner one.
    pub(crate) selector: Option<usize>,
    /// The root layer's presence flag, when presence is carried by one.
    pub(crate) present: Option<usize>,
}

impl FoldPlan {
    /// The construction itself — see the field's own note.
    ///
    /// Handed out by reference and never by value, for the reason
    /// [`UnfoldPlan::tree`](crate::unfold::UnfoldPlan::tree) gives: the
    /// signature beside it is collected from this tree when the plan is built.
    /// Where each value sits on the foreign signature.
    ///
    /// Slot numbers and names are one flat layout's facts, not the semantic
    /// act of folding, so they live beside the tree rather than in it. An
    /// adapter wanting a different physical shape builds its own from
    /// [`Self::tree`].
    pub fn layout(&self) -> &SlotLayout {
        &self.layout
    }

    pub fn tree(&self) -> &crate::expand::InNode {
        &self.tree
    }

    /// Flattened wire leaves, in foreign-signature order — see the field's own
    /// note.
    pub fn leaves(&self) -> &[FoldLeaf] {
        &self.leaves
    }

    /// Outer shape over the core construct (`Construct` for a plain `T`/`&T`
    /// param; `Optional(Construct)` for `Option<T>`/`Option<&T>`).
    ///
    /// A derived view of [`Self::tree`]: each layer is a node wrapping the
    /// construction, and [`InNode::shape`](crate::expand::InNode::shape) reads
    /// the stack back off them.
    pub fn shape(&self) -> FoldShape {
        self.tree.shape()
    }

    /// True when the fold produces an `Option<_>` (outermost shape layer is
    /// `Optional`) — drives the by-ref call-site form (`folded.as_ref()`).
    pub fn produces_option(&self) -> bool {
        matches!(self.shape(), FoldShape::Optional((), _))
    }

    /// Index into [`Self::leaves`] of the explicit presence-flag (`bool`) leaf
    /// for a **multi-argument** `Optional` shape (`Option<T>` built from a
    /// constructor taking ≥2 args): the flag decides `Some`/`None`, the arg
    /// leaves are plain (non-`Option`). `None` for a non-optional fold or a
    /// single-argument `Optional` (where presence rides the layer's own
    /// `Option` slot). A separate flag avoids boxing a nullable primitive arg
    /// (e.g. `Option<i32>` → `Integer?`) on the wire.
    pub fn present(&self) -> Option<usize> {
        self.present
    }

    /// Index into [`Self::leaves`] of the selector leaf; `None` for a single
    /// constructor (the sole variant is applied unconditionally). Under an
    /// [`Optional`](FoldShape::Optional) layer the selector also encodes
    /// **absence**: `-1` = `None`, `0..n-1` = the taken arm.
    ///
    /// Read off [`Self::tree`] rather than stored beside it: the dispatch is
    /// the node, and a second copy of which slot selects it could disagree
    /// with the node the emitter actually walks.
    /// The positions each dispatch arm's arguments occupy.
    ///
    /// `None` for an arm that builds one of its arguments from further
    /// positions: such an arm has no flat signature, which is what makes it
    /// unsplittable into a destination-language overload.
    ///
    /// Derived, because positions are the layout's and not the tree's: they are
    /// claimed in walk order, so the selector comes first and each arm's
    /// arguments follow in turn (#447 §1).
    pub fn arm_arg_slots(&self) -> Vec<Option<Vec<usize>>> {
        let mut next = self.selector.map_or(0, |s| s + 1);
        self.tree
            .arms()
            .iter()
            .map(|arm| {
                let claimed = crate::expand::derive_layout(arm).len();
                let slots = arm
                    .leaf_args()
                    .map(|flat| (next..next + flat.len()).collect::<Vec<_>>());
                next += claimed;
                slots
            })
            .collect()
    }

    pub fn selector(&self) -> Option<usize> {
        self.selector
    }
}

/// One flattened wire leaf of an expanded parameter.
#[derive(Clone)]
pub struct FoldLeaf {
    /// Foreign-side parameter name.
    pub name: syn::Ident,
    /// The **reading** of the type whose resolved input converter decodes this
    /// leaf. For a single constructor these are the raw constructor parameter
    /// types; for a combined one the selector (`i32`) and `Option`-wrapped
    /// variant inputs. Spell it with `emit.spell(&ty)` in an emission callback.
    ///
    /// A reading rather than a spelling for the reason `UnfoldLeaf::out_ty`
    /// gives: a consumer asking what this leaf's type MEANS had to hand the
    /// spelling back to the registry (#275). The leaves no source wrote — the
    /// presence flag, the selector — are built by
    /// [`TypeRef::scalar`](prebindgen_flat::flat::TypeRef::scalar), which
    /// pairs the kind with its own spelling and is placeless by construction.
    pub ty: prebindgen_flat::flat::TypeRef,
}

/// Where every value sits on the foreign signature: one entry per position, in
/// the order the wire carries them.
///
/// The **compatibility projection** of #447 §1. Slot numbers and names are
/// properties of one flat wire layout rather than of the semantic act of
/// folding values into Rust, so they live here and not on the tree's nodes. A
/// future adapter that wants an object parameter, a tagged union, overloads or
/// another presence representation builds its own from the same tree instead of
/// inheriting this one.
#[derive(Debug, Default, Clone)]
pub struct SlotLayout {
    positions: Vec<(syn::Ident, SlotKind)>,
}

/// What one foreign-signature position carries.
///
/// The encoding of a dispatch and of presence — an `i32` tag, a `bool` flag, an
/// `Option`-wrapped argument — is this layout's choice and not the tree's. A
/// different adapter picks differently from the same semantic nodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlotKind {
    /// A value: a constructor argument, an identity arm's own value, or the
    /// collection a run is built from.
    Value,
    /// Which arm of a dispatch runs, and `-1` when a `Selector` layer above it
    /// means absent.
    Selector,
    /// An explicit presence flag, for a construction whose arguments cannot
    /// carry absence themselves.
    PresenceFlag,
    /// Presence riding the layer's own argument: absent is that argument's
    /// `None`.
    PresencePayload,
}

impl SlotLayout {
    /// Claim the next position, returning its index.
    pub(crate) fn claim(&mut self, name: syn::Ident, kind: SlotKind) -> usize {
        self.positions.push((name, kind));
        self.positions.len() - 1
    }

    /// What the position at `slot` is called on the foreign signature.
    pub fn name(&self, slot: usize) -> &syn::Ident {
        &self.positions[slot].0
    }

    /// What the position at `slot` carries.
    pub fn kind(&self, slot: usize) -> SlotKind {
        self.positions[slot].1
    }

    /// The first position of `kind`, which for a dispatch or a presence flag is
    /// the outermost one — positions are claimed outside in.
    pub fn first_of(&self, kind: SlotKind) -> Option<usize> {
        self.positions.iter().position(|(_, k)| *k == kind)
    }

    /// How many positions the signature has.
    pub fn len(&self) -> usize {
        self.positions.len()
    }

    /// Whether the signature has no positions at all.
    pub fn is_empty(&self) -> bool {
        self.positions.is_empty()
    }
}
