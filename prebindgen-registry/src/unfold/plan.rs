//! Resolved output-deconstruction plans.

use super::Delivery;
/// Outer shape wrapping the [core decomposition](`UnfoldShape::Base`).
/// The output-side analog of [`crate::expand::FoldShape`], on the
/// unified [`Shape`](crate::shape::Shape) layer stack:
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
pub use crate::shape::Shape as UnfoldShape;

/// Identity of the deconstructor **declaration** a plan's records came from.
/// A `run`-signature artifact (e.g. a generated callback interface) is fully
/// determined by the declaration, so adapters key such artifacts on this —
/// functions selecting the same declaration share one artifact; differently
/// declared decompositions of the same type get distinct ones. The first
/// field is always the target type's canonical [`TypeKey`](crate::TypeKey)
/// string.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum DeconId {
    /// The type's default (`expand_return!`-declared) deconstructor.
    Default(String),
    /// Per-fn inline records (`.expand_return`) — unique to the
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
    /// The decomposed type as first encountered. A **reading**, so "compare via
    /// [`TypeKey`](crate::TypeKey), not syntactically" is
    /// the type rather than an instruction: `source.key()` is the identity and
    /// `emit.emit_source_type(&source)` is what an emission callback writes.
    pub source: crate::flat::TypeRef,
    /// Flattened leaves in declared record order — names, types, paths,
    /// nullability all declaration-fixed.
    pub leaves: Vec<UnfoldLeaf>,
}

/// One step of a leaf's [`UnfoldLeaf::path`] — how to get from the value
/// reached so far to the next one.
///
/// A step is typed rather than a bare ident because a single path may **mix**
/// the two: an `expand_return!(T).fields(fields!(t_to_struct))` leaf calls the
/// value-form accessor, reads a struct field, and may then call that field
/// type's own accessor — `Call(t_to_struct)`, `Field(key_expr)`,
/// `Call(keyexpr_as_str)`. [`LeafSource`] still says what *kind* of leaf sits
/// at the end of the path; the steps say how it is reached.
///
/// Each step also records whether it is **optional** — its accessor returns
/// `Option<…>`, or its field is typed `Option<…>`. A `true` on a step *before*
/// the last makes it a nullable nesting step: the emitter matches on it and the
/// `None` arm short-circuits the whole leaf to null. The flag is carried rather
/// than re-derived so both kinds answer the question the same way and the
/// emitter needs no type walk (an accessor's `Option` was already peeled where
/// the step was built).
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum PathStep {
    /// Call a `#[prebindgen]` accessor on the value reached so far:
    /// `source_module::f(&value)`.
    Call {
        ident: syn::Ident,
        optional: bool,
        /// Whether the value this call yields (its return with any `Option`
        /// peeled) is **owned** rather than a borrow — `f(..) -> T` / `-> Option<T>`
        /// against `-> &T` / `-> Option<&T>`.
        ///
        /// Only an OPTIONAL step's payload can reach an emitter as a bare
        /// binding, so this is only ever consulted there: what a `Some` arm
        /// binds is the accessor's own value, and everything downstream of it —
        /// the next step's receiver, the value form's argument — needs to know
        /// whether it may be moved or has to be borrowed. Recorded here, at the
        /// one place the signature is in hand, because deriving it later from
        /// the path alone is not possible.
        owned: bool,
    },
    /// Read a struct field of the value reached so far: `value.f`.
    Field { ident: syn::Ident, optional: bool },
}

impl PathStep {
    /// An accessor call step. `owned` says whether its (`Option`-peeled) return
    /// is an owned value rather than a borrow — see [`Self::Call::owned`].
    pub fn call(ident: syn::Ident, optional: bool, owned: bool) -> Self {
        Self::Call {
            ident,
            optional,
            owned,
        }
    }

    /// Whether the value this step yields is owned. A field read composes as
    /// `&(e).f`, so it is a borrow by construction.
    pub fn yields_owned(&self) -> bool {
        matches!(self, Self::Call { owned: true, .. })
    }

    /// A struct-field read step.
    pub fn field(ident: syn::Ident, optional: bool) -> Self {
        Self::Field { ident, optional }
    }

    /// The step's ident, whichever kind it is.
    pub fn ident(&self) -> &syn::Ident {
        match self {
            Self::Call { ident, .. } | Self::Field { ident, .. } => ident,
        }
    }

    /// Whether the step yields an `Option` — a nullable nesting step when it is
    /// not the last on the path.
    pub fn is_optional(&self) -> bool {
        match self {
            Self::Call { optional, .. } | Self::Field { optional, .. } => *optional,
        }
    }

    /// Whether the step is a plain (non-optional) field read — a path made only
    /// of these renders as `value.a.b`, needing no nesting `match`.
    pub fn is_plain_field(&self) -> bool {
        matches!(
            self,
            Self::Field {
                optional: false,
                ..
            }
        )
    }

    /// Whether the step is a field read, `Option` or not.
    pub fn is_field(&self) -> bool {
        matches!(self, Self::Field { .. })
    }
}

/// Whether a run of steps can be **moved** out of the value it hangs off:
/// field reads only, with an `Option` allowed on the last one — a `None` arm
/// still hands over the whole `Option` by value, while an `Option` in the
/// middle would have to be unwrapped and so can only be borrowed through.
///
/// This is the one place the rule is written: the resolver uses it to decide
/// whether a leaf OWNS what it reaches (its `out_ty` then being the owned type
/// rather than a borrow), and the emitters use it to project that place. Two
/// readings of it would drift, and the disagreement would be a borrow handed to
/// an owning converter.
pub fn steps_are_movable(steps: &[PathStep]) -> bool {
    steps
        .iter()
        .enumerate()
        .all(|(i, s)| s.is_field() && (!s.is_optional() || i + 1 == steps.len()))
}

/// How a leaf's [`UnfoldLeaf::path`] is reached from the decomposed value.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub enum LeafSource {
    /// The value is **reached off** the decomposed value: an accessor chain, a
    /// field chain, or the two mixed, which is what
    /// [`UnfoldLeaf::path`] spells.
    ///
    /// One variant rather than two. `Accessor` and `Field` were separate while
    /// an adapter branched on which it was; none does — the difference is
    /// whether the path ENDS in a field read, which the path itself says. Two
    /// spellings of one fact are two things to keep in step, and this is the
    /// one that had no reader left.
    #[default]
    Reach,
    /// The **synthesized selector** of a decomposed sum: an `i32` naming which
    /// alternative is live. It is not read off the value at all — the emitter
    /// assigns it per `match` arm — so it has no path. Emitted once, ahead of
    /// the groups it selects between (see
    /// [`SumDecon`](crate::unfold::SumDecon)).
    ///
    /// Its [`out_ty`](UnfoldLeaf::out_ty) is **the sum**, not the `i32` — it
    /// carries *which* sum it chooses between, which is how the emitter finds
    /// the enum to `match`. That type is **registered and not required** (#282):
    /// it gets a table cell like every other leaf's, but no root, because a sum
    /// has no whole-value output converter and demanding one would fail
    /// resolution over a type that never crosses whole. The reading comes from
    /// the declaration — [`Variant::type_ref`](crate::flat::Variant::type_ref)
    /// — never from an adapter composing one out of a name.
    SumTag,
    /// A payload field of ONE alternative of a decomposed sum, reached through
    /// a **variant pattern** rather than a path: the emitter binds `member`
    /// inside `variant`'s `match` arm. The leaf is live only when
    /// [`UnfoldLeaf::groups`] ends in the value's tag; in every other arm its
    /// slot carries the wire default.
    ///
    /// This is the selector [`Reach`](Self::Reach) deliberately lacks — a
    /// reached value is a deterministic product, every one of them
    /// contributing unconditionally.
    VariantField {
        /// The variant's ident as declared in the source enum.
        variant: syn::Ident,
        /// How the payload field is addressed in the arm's pattern.
        member: syn::Member,
    },
    /// The **synthesized presence** of an optional value the decomposition
    /// looks through: a boolean saying whether the leaves that follow carry
    /// anything.
    ///
    /// The selector [`SumTag`](Self::SumTag) is for a value that is one of
    /// several alternatives; this is for a value that is either there or not,
    /// and the difference is not a two-alternative sum: absence has no
    /// alternative of its own to name, and the group it gates is the value's
    /// own leaves rather than one arm's payload. Like a tag it is not read off
    /// the value — the emitter assigns it — so [`UnfoldLeaf::path`] reaches
    /// the OPTIONAL value it tests, not a place holding a boolean.
    Presence,
}

/// A resolved output expansion for one function.
#[derive(Clone)]
pub struct UnfoldPlan {
    /// Owned core type the records decompose — the function's return after
    /// peeling `&` / `Option` / `Vec`.
    pub source: crate::flat::TypeRef,
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
    pub element: Option<crate::flat::TypeRef>,
    /// Callback (`deconstruct_output`) vs return-value (`convert_output`)
    /// delivery.
    pub delivery: Delivery,
    /// For [`Delivery::Return`]: the single leaf's `out_ty` lifted through the
    /// shape (`Decompose` ⇒ `out_ty`, `Optional` ⇒ `Option<out_ty>`). The
    /// wrapper returns this value through its ordinary output converter (no
    /// callback). `None` for [`Delivery::Callback`].
    pub convert_out_ty: Option<crate::flat::TypeRef>,
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

/// One hoisted value form: where it sits, and whether it **consumes** the value
/// it decomposes.
#[derive(Clone)]
pub struct Hoist {
    /// The path prefix to bind, ending in the value form's
    /// [`PathStep::Call`] (`DeconRecord::Fields`).
    pub prefix: Vec<PathStep>,
    /// `true` when the accessor takes its receiver **by value**
    /// (`f(v: T) -> TStruct`), so the value is moved in and each field can be
    /// moved *out* into its leaf instead of cloned — the whole point of a
    /// consuming value form.
    ///
    /// Carried on the hoist rather than on [`PathStep::Call`] because only a
    /// value-form root can consume: the ordinary accessor-chain steps are
    /// always borrows.
    pub consuming: bool,
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
    /// Reach chain from the root value (`[]` = the identity/root itself;
    /// `[Call(f)]` = `f(&root)`; longer = nested records, M3). Steps of both
    /// kinds may mix — see [`PathStep`].
    pub path: Vec<PathStep>,
    /// The **reading** of the type whose resolved output converter encodes this
    /// leaf — a reference type for accessors (`&str`, `&F`), `&Source` for the
    /// identity leaf (so the borrowed-opaque clone converter / projection is
    /// reused). Spell it with `emit.emit_source_type(&out_ty)` in an emission callback.
    ///
    /// A reading rather than a spelling because a consumer asking what this
    /// leaf's type *means* had to hand the spelling back to the registry and
    /// hope for a cell — the round trip #263 removed from `api/core`, surviving
    /// in the plans, and answering "no layer" for a type it had never seen
    /// (#275). The composed ones (`&Source`) are built by
    /// [`TypeRef::borrowed`](crate::flat::TypeRef::borrowed), which
    /// pairs the kind with its own spelling.
    pub out_ty: crate::flat::TypeRef,
    /// `true` for the move/clone-the-value handle leaf, emitted **last** (after
    /// every reference leaf's JVM conversion has ended its borrow).
    pub identity: bool,
    /// `true` when a nesting accessor on [`Self::path`] returns `Option` (M3):
    /// the reached value may be absent, so the leaf is nullable on the
    /// destination side (e.g. a Kotlin `?` type); emit wraps the encode in a
    /// `match Some/None`.
    pub nullable: bool,
    /// How [`Self::path`] is reached from the value — an accessor-fn chain
    /// (default), a struct-field chain (synthesized `data_class`), a variant
    /// pattern binding (decomposed sum), or one of the two synthesized
    /// selectors, which are assigned rather than reached.
    pub source: LeafSource,
    /// **Group membership**, as the path of arms this leaf sits inside —
    /// outermost first. Empty for a leaf that is always live. A non-empty path
    /// marks the leaf as belonging to the group a **selector** chooses: live
    /// only when that selector says so, wire-defaulted otherwise.
    ///
    /// Each element is one arm, and what an arm number means is the selector's
    /// own answer:
    ///
    /// * a [`SumTag`](LeafSource::SumTag) chooses among alternatives, and its
    ///   arm is the alternative's tag;
    /// * a [`Presence`](LeafSource::Presence) chooses between "the value is
    ///   there" and "it is not", and its arm is `0` — the one group a presence
    ///   flag gates, carried by the leaves of the value it speaks for.
    ///
    /// Grouping is what turns a leaf list into a `match`: leaves sharing a
    /// group are emitted together in one arm instead of as independent
    /// per-leaf expressions.
    ///
    /// **A selector carries the path it is nested in, not its own arms.** An
    /// unconditional selector's path is empty; one inside a group carries that
    /// group's path, the same as any other member — which is what lets a
    /// selector own another. Its own members extend that path by one, so
    /// "member of the outer group" and "selector of an inner one" are two
    /// different lengths of the same path rather than two meanings of one
    /// number, and [`segments`](crate::unfold::segments) reads one nesting
    /// level at a time (#602).
    pub groups: Vec<i32>,
}

impl UnfoldLeaf {
    /// Whether this leaf's [`out_ty`](Self::out_ty) needs a resolved **output
    /// converter**. False for a synthesized **selector** — a
    /// [`SumTag`](LeafSource::SumTag) or a [`Presence`](LeafSource::Presence):
    /// each is assigned by the emitter rather than converted, so requiring
    /// a converter for one would make every sum depend on an unrelated `i32`
    /// crossing existing in the binding.
    ///
    /// **This is the root question, not the registration question.** Every
    /// leaf's `out_ty` gets a table cell; this decides which of them the
    /// binding additionally *demands* a converter for. A cell says the type
    /// entered the pipeline, a root says the binding asked for it directly, and
    /// an entry says one resolved — three separate claims, and a `SumTag` leaf
    /// makes only the first (#282).
    pub fn has_converter(&self) -> bool {
        // A presence flag is synthesized like a tag: it says whether the
        // leaves after it carry anything, and the value it tests crosses
        // through those leaves rather than through this one.
        !matches!(self.source, LeafSource::SumTag | LeafSource::Presence)
    }
}
