//! Resolved output-deconstruction plans.

use super::Delivery;
/// Outer shape wrapping the [core decomposition](`UnfoldShape::Base`).
/// The output-side analog of [`crate::api::core::expand::FoldShape`], on the
/// unified [`Shape`](crate::api::core::shape::Shape) layer stack:
///   * `Base` — run the accessor's records on the value, producing all
///     [leaves](`UnfoldPlan::leaves`) and invoking the builder once;
///   * `Optional((), inner)` — `Option<T>`/`Option<&T>` return: `None` ⇒ a null
///     result (builder skipped), `Some` ⇒ decompose the inner;
///   * `Iterable(inner)` — `Vec<T>` return: fold the elements through an
///     accumulator `(acc, …) -> acc`. Each element is delivered either WHOLE (via
///     its own output converter + projection — see [`UnfoldPlan::element`]) or
///     DECOMPOSED into per-element leaves (explicit accessors, or a synthesized
///     `data_class` — see [`UnfoldPlan::fixed_builder`]); inner is `Base`.
///
/// The `()` payload is unused here — only the JNI adapter's
/// `Shape<NullableKind>` carries per-layer data.
pub use crate::api::core::shape::Shape as UnfoldShape;

/// Identity of the deconstructor **declaration** a plan's records came from.
/// A `run`-signature artifact (e.g. a generated callback interface) is fully
/// determined by the declaration, so adapters key such artifacts on this —
/// functions selecting the same declaration share one artifact; differently
/// declared decompositions of the same type get distinct ones. The first
/// field is always the target type's canonical [`TypeKey`](crate::api::core::registry::TypeKey)
/// string.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum DeconId {
    /// The type's default (`.default_return_expand*`-declared) deconstructor.
    Default(String),
    /// Per-fn inline records (`.return_expand*`) — unique to the
    /// function (second field = the fn ident).
    PerFn(String, String),
}

/// The declaration-default decomposition of one deconstructor: its leaf
/// list resolved ONCE from the declaration's records with **normalized**
/// inputs (borrowed identity form, no outer shape), so the content is
/// independent of which functions use the declaration and in what order.
/// Stored in `Registry::decon_plans`; the single source language adapters
/// derive declaration-keyed signature artifacts (e.g. generated callback
/// interfaces) from. Per-function aspects (`by_ref`, shape, delivery) live on
/// each function's [`UnfoldPlan`], which points here via [`UnfoldPlan::decon`].
///
/// Normalization detail: the identity leaf's `out_ty` is always the borrowed
/// `&Source` form (an owned-return function's own plan carries owned `Source`
/// instead) — both resolve to the same projection/class, and adapters reading
/// the spec must tolerate whichever form their type tables resolved.
#[derive(Clone)]
pub struct DeconSpec {
    /// The decomposed type as first encountered (path qualification may vary
    /// by call site; compare via [`TypeKey`](crate::api::core::registry::TypeKey),
    /// not syntactically).
    pub source: syn::Type,
    /// Flattened leaves in declared record order — names, types, paths,
    /// nullability all declaration-fixed.
    pub leaves: Vec<UnfoldLeaf>,
}

/// How a leaf's [`UnfoldLeaf::path`] is reached from the decomposed value.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum LeafSource {
    /// The path is a chain of `#[prebindgen]` **accessor functions**:
    /// `source_module::f(&value)`, composing nested accessors. Nesting steps
    /// that return `Option` make the leaf nullable. This is the form produced
    /// by `.deconstructor_record*` / `.fun_accessor` declarations.
    #[default]
    Accessor,
    /// The path is a chain of **struct field idents** reached by field access
    /// and cloned: `value.a.b.clone()`. Produced by the synthesized
    /// decomposition of a by-value `data_class` (see
    /// [`crate::api::core::unfold::apply_value_structs`]); the value's own
    /// fields cross as decoupled leaves and the foreign side reassembles the
    /// object (so no Java object is built on the Rust side).
    Field,
}

/// A resolved output expansion for one function.
#[derive(Clone)]
pub struct UnfoldPlan {
    /// Owned core type the records decompose — the function's return after
    /// peeling `&` / `Option` / `Vec`.
    pub source: syn::Type,
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
    pub element: Option<syn::Type>,
    /// Callback (`deconstruct_output`) vs return-value (`convert_output`)
    /// delivery.
    pub delivery: Delivery,
    /// For [`Delivery::Return`]: the single leaf's `out_ty` lifted through the
    /// shape (`Decompose` ⇒ `out_ty`, `Optional` ⇒ `Option<out_ty>`). The
    /// wrapper returns this value through its ordinary output converter (no
    /// callback). `None` for [`Delivery::Callback`].
    pub convert_out_ty: Option<syn::Type>,
    /// `true` for a synthesized by-value `data_class` decomposition (see
    /// [`crate::api::core::unfold::apply_value_structs`]): the builder/folder
    /// is a **fixed, hoisted** foreign singleton that reconstructs the concrete
    /// class (the wrapper takes no caller `build`/`fold` param and is not
    /// generic over `R`/`A` — it returns the concrete type). `false` for the
    /// accessor-declared deconstructors, whose builder is caller-supplied.
    pub fixed_builder: bool,
}

/// One flattened output leaf of a decomposed return value.
#[derive(Clone)]
pub struct UnfoldLeaf {
    /// The author-supplied leaf name, used **literally** (no casing / stripping /
    /// keyword escaping). Nested records prefix the child's name with their own
    /// name, joined by the reserved `"__"` separator (`"sample"` splicing
    /// `"keyExpr"` → `"sample__keyExpr"`); a root identity leaf is `"handle"`.
    /// Names are unique within a deconstructor (a duplicate is a hard error).
    pub name: String,
    /// Accessor-call chain from the root value (`[]` = the identity/root
    /// itself; `[f]` = `f(&root)`; longer = nested records, M3).
    pub path: Vec<syn::Ident>,
    /// Type whose resolved **output** converter encodes this leaf — a
    /// reference type for accessors (`&str`, `&F`), `&Source` for the identity
    /// leaf (so the borrowed-opaque clone converter / projection is reused).
    pub out_ty: syn::Type,
    /// `true` for the move/clone-the-value handle leaf, emitted **last** (after
    /// every reference leaf's JVM conversion has ended its borrow).
    pub identity: bool,
    /// `true` when a nesting accessor on [`Self::path`] returns `Option` (M3):
    /// the reached value may be absent, so the leaf is nullable on the
    /// destination side (e.g. a Kotlin `?` type); emit wraps the encode in a
    /// `match Some/None`.
    pub nullable: bool,
    /// How [`Self::path`] is reached from the value — an accessor-fn chain
    /// (default) or a struct-field chain (synthesized `data_class`).
    pub source: LeafSource,
}
