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
    pub tree: crate::expand::InNode,
}

impl FoldPlan {
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
        self.tree.present()
    }

    /// Index into [`Self::leaves`] of the selector leaf; `None` for a single
    /// constructor (the sole variant is applied unconditionally). Under an
    /// [`Optional`](FoldShape::Optional) layer the selector also encodes
    /// **absence**: `-1` = `None`, `0..n-1` = the taken arm.
    ///
    /// Read off [`Self::tree`] rather than stored beside it: the dispatch is
    /// the node, and a second copy of which slot selects it could disagree
    /// with the node the emitter actually walks.
    pub fn selector(&self) -> Option<usize> {
        self.tree.selector()
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
