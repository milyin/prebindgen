//! Resolved output-deconstruction plans.

/// Outer shape wrapping the [core decomposition](`UnfoldShape::Base`).
/// The output-side analog of [`crate::api::core::expand::FoldShape`], on the
/// unified [`Shape`](crate::api::core::shape::Shape) layer stack:
///   * `Base` — run the accessor's records on the value, producing all
///     [leaves](`UnfoldPlan::leaves`) and invoking the builder once;
///   * `Optional((), inner)` — `Option<T>`/`Option<&T>` return: `None` ⇒ a null
///     result (builder skipped), `Some` ⇒ decompose the inner;
///   * `Iterable(inner)` — `Vec<T>` return: deliver each element whole (via its
///     own output converter + projection — see [`UnfoldPlan::element`]) to a
///     caller-supplied fold `(acc, element) -> acc`; inner is `Base` (a
///     degenerate single whole-element step; per-element accessor decomposition
///     is future work).
///
/// The `()` payload is unused here — only the JNI adapter's
/// `Shape<NullableKind>` carries per-layer data.
pub use crate::api::core::shape::Shape as UnfoldShape;

use super::Delivery;

/// Identity of the deconstructor **declaration** a plan's records came from.
/// A `run`-signature artifact (e.g. a generated callback interface) is fully
/// determined by the declaration, so adapters key such artifacts on this —
/// functions selecting the same declaration share one artifact; differently
/// declared decompositions of the same type get distinct ones. The first
/// field is always the target type's canonical [`TypeKey`](crate::api::core::registry::TypeKey)
/// string.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum DeconId {
    /// The type's unnamed (top-level / `.default()`-applied) deconstructor.
    Canonical(String),
    /// A named declaration (`.deconstructor_name(name)`), selected per fn
    /// via the `_with(name)` selectors.
    Named(String, String),
    /// Per-fn inline records (`.fun_output(...)`) — unique to the function
    /// (second field = the fn ident).
    PerFn(String, String),
}

/// The declaration-canonical decomposition of one deconstructor: its leaf
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
    /// `Decompose`/`Optional` (accessor decomposition); **empty** for
    /// `Iterable`, which delivers each element whole (see [`Self::element`]).
    pub leaves: Vec<UnfoldLeaf>,
    /// For an `Iterable` plan: the owned/ref element type, delivered to the fold
    /// via its own output converter + projection (not decomposed). `None` for
    /// `Decompose`/`Optional`.
    pub element: Option<syn::Type>,
    /// Callback (`deconstruct_output`) vs return-value (`convert_output`)
    /// delivery.
    pub delivery: Delivery,
    /// For [`Delivery::Return`]: the single leaf's `out_ty` lifted through the
    /// shape (`Decompose` ⇒ `out_ty`, `Optional` ⇒ `Option<out_ty>`). The
    /// wrapper returns this value through its ordinary output converter (no
    /// callback). `None` for [`Delivery::Callback`].
    pub convert_out_ty: Option<syn::Type>,
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
}
