//! The plan that carries the registry's leaves under one of JniGen's two
//! deliveries.
//!
//! Not beside the leaves, because which delivery a decomposition takes follows
//! from what the target can receive: JniGen returns one non-nullable leaf and
//! hands everything else to a callback, since a nullable leaf crosses as a boxed
//! `Long` and a JVM `null`. A C binding could deliver several leaves as
//! out-parameters (#680 review).

use prebindgen_registry::leaf::{DeconId, Hoist, UnfoldLeaf, UnfoldShape};

/// How the decomposed value(s) are delivered to the foreign side. Derived
/// from the resolved leaf count (1 ⇒ `Return`, N ⇒ `Callback`); errors are
/// always `Callback`-shaped.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Delivery {
    /// Deliver the leaves to a foreign **callback** (builder / fold). Any
    /// leaf count.
    Callback,
    /// **Return**/deliver the single decomposed value (no builder). Requires
    /// exactly one leaf and a non-`Iterable` shape.
    Return,
}

/// A resolved output expansion for one function.
#[derive(Clone)]
pub struct UnfoldPlan {
    /// Owned core type the records decompose — the function's return after
    /// peeling `&` / `Option` / `Vec`.
    pub source: prebindgen_registry::flat::TypeRef,
    /// Which deconstructor declaration produced [`Self::leaves`] — the
    /// identity adapters key signature artifacts on. `None` only for the
    /// whole-element `Iterable` arm (no declaration involved).
    pub decon: Option<DeconId>,
    /// True when the return was `&T` / `Option<&T>`: the identity leaf clones
    /// the borrow; otherwise it moves the owned value.
    pub by_ref: bool,
    /// Outer shape over the core decomposition (`Decompose` for a plain
    /// `T`/`&T` return).
    pub shape: UnfoldShape,
    /// Flattened output leaves, in builder-argument order. Populated for
    /// `Decompose`/`Optional` (accessor decomposition) and for a **decomposed**
    /// `Iterable` fold (per-element leaves — explicit-accessor or a synthesized
    /// `data_class` [`Self::fixed_builder`]); **empty** only for a
    /// **whole-element** `Iterable`, which delivers each element via
    /// [`Self::element`].
    pub leaves: Vec<UnfoldLeaf>,
    /// For a **whole-element** `Iterable` plan: the owned/ref element type,
    /// delivered to the fold via its own output converter + projection (not
    /// decomposed). `None` for `Decompose`/`Optional` and for a **decomposed**
    /// `Iterable` fold (which uses [`Self::leaves`]).
    pub element: Option<prebindgen_registry::flat::TypeRef>,
    /// Callback (`deconstruct_output`) vs return-value (`convert_output`)
    /// delivery.
    pub delivery: Delivery,
    /// For [`Delivery::Return`]: the single leaf's `out_ty` lifted through the
    /// shape (`Decompose` ⇒ `out_ty`, `Optional` ⇒ `Option<out_ty>`). The
    /// wrapper returns this value through its ordinary output converter (no
    /// callback). `None` for [`Delivery::Callback`].
    pub convert_out_ty: Option<prebindgen_registry::flat::TypeRef>,
    /// `true` for a synthesized by-value `data_class` decomposition (see
    /// [`ValueDecon`](crate::unfold::ValueDecon)): the builder/folder
    /// is a **fixed, hoisted** foreign singleton that reconstructs the concrete
    /// class (the wrapper takes no caller `build`/`fold` param and is not
    /// generic over `R`/`A` — it returns the concrete type). `false` for the
    /// accessor-declared deconstructors, whose builder is caller-supplied.
    pub fixed_builder: bool,
    /// Value forms that must be evaluated **once** and bound to a local. Every
    /// leaf below one reaches off that local — otherwise each field would
    /// rebuild the whole struct, cloning all of it once per leaf.
    ///
    /// A list rather than a single accessor because value forms **compose**: a
    /// field may splice a child type whose own boundary is derived from *its*
    /// value form, and that child call is a second hoist nested under the
    /// first. Ordered outermost-first, so a hoist can be composed from the
    /// longest already-bound prefix of itself.
    pub hoists: Vec<Hoist>,
}

impl UnfoldPlan {
    /// Whether this plan is exactly one Optional layer over one scalar/product
    /// decomposition. Iterable and future composed inner shapes answer false.
    pub fn is_optional_base(&self) -> bool {
        matches!(
            &self.shape,
            UnfoldShape::Optional((), inner) if matches!(**inner, UnfoldShape::Base)
        )
    }
}
