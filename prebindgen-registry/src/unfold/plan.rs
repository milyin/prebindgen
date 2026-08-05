//! Resolved output-deconstruction plans.

/// Outer shape wrapping the [core decomposition](`UnfoldShape::Base`).
/// The output-side analog of [`crate::expand::FoldShape`], on the
/// unified [`Shape`](prebindgen::core::shape::Shape) layer stack:
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
pub use prebindgen::core::shape::Shape as UnfoldShape;

use super::Delivery;

/// Identity of the deconstructor **declaration** a plan's records came from.
/// A `run`-signature artifact (e.g. a generated callback interface) is fully
/// determined by the declaration, so adapters key such artifacts on this —
/// functions selecting the same declaration share one artifact; differently
/// declared decompositions of the same type get distinct ones. The first
/// field is always the target type's canonical [`TypeKey`](crate::registry::TypeKey)
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
    /// [`TypeKey`](crate::registry::TypeKey), not syntactically" is
    /// the type rather than an instruction: `source.key()` is the identity and
    /// `source.spell()` is what generated Rust says.
    pub source: prebindgen::core::flat::TypeRef,
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
    /// The path is a chain of `#[prebindgen]` **accessor functions**:
    /// `source_module::f(&value)`, composing nested accessors. Nesting steps
    /// that return `Option` make the leaf nullable. This is the form produced
    /// by `.deconstructor_record*` / `.fun_accessor` declarations.
    #[default]
    Accessor,
    /// The path is a chain of **struct field idents** reached by field access
    /// and cloned: `value.a.b.clone()`. Produced by the synthesized
    /// decomposition of a by-value `data_class` (see
    /// [`crate::unfold::apply_value_structs`]); the value's own
    /// fields cross as decoupled leaves and the foreign side reassembles the
    /// object (so no Java object is built on the Rust side).
    Field,
    /// The **synthesized selector** of a decomposed sum: an `i32` naming which
    /// alternative is live. It is not read off the value at all — the emitter
    /// assigns it per `match` arm — so it has no path. Emitted once, ahead of
    /// the groups it selects between (see
    /// [`crate::unfold::apply_sum_returns`]).
    ///
    /// Its [`out_ty`](UnfoldLeaf::out_ty) is **the sum**, not the `i32` — it
    /// carries *which* sum it chooses between, which is how the emitter finds
    /// the enum to `match`. That type is **registered and not required** (#282):
    /// it gets a table cell like every other leaf's, but no root, because a sum
    /// has no whole-value output converter and demanding one would fail
    /// resolution over a type that never crosses whole. The reading comes from
    /// the declaration — [`Variant::type_ref`](prebindgen::core::flat::Variant::type_ref)
    /// — never from an adapter composing one out of a name.
    SumTag,
    /// A payload field of ONE alternative of a decomposed sum, reached through
    /// a **variant pattern** rather than an accessor chain or a field chain:
    /// the emitter binds `member` inside `variant`'s `match` arm. The leaf is
    /// live only when [`UnfoldLeaf::group`] equals the value's tag; in every
    /// other arm its slot carries the wire default.
    ///
    /// This is the selector [`Accessor`](Self::Accessor) and
    /// [`Field`](Self::Field) deliberately lack — both are deterministic
    /// products, every record contributing unconditionally.
    VariantField {
        /// The variant's ident as declared in the source enum.
        variant: syn::Ident,
        /// How the payload field is addressed in the arm's pattern.
        member: syn::Member,
    },
}

/// A resolved output expansion for one function.
#[derive(Clone)]
pub struct UnfoldPlan {
    /// Owned core type the records decompose — the function's return after
    /// peeling `&` / `Option` / `Vec`.
    pub source: prebindgen::core::flat::TypeRef,
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
    pub element: Option<prebindgen::core::flat::TypeRef>,
    /// Callback (`deconstruct_output`) vs return-value (`convert_output`)
    /// delivery.
    pub delivery: Delivery,
    /// For [`Delivery::Return`]: the single leaf's `out_ty` lifted through the
    /// shape (`Decompose` ⇒ `out_ty`, `Optional` ⇒ `Option<out_ty>`). The
    /// wrapper returns this value through its ordinary output converter (no
    /// callback). `None` for [`Delivery::Callback`].
    pub convert_out_ty: Option<prebindgen::core::flat::TypeRef>,
    /// `true` for a synthesized by-value `data_class` decomposition (see
    /// [`crate::unfold::apply_value_structs`]): the builder/folder
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
    /// reused). Spell it with `out_ty.spell()`.
    ///
    /// A reading rather than a spelling because a consumer asking what this
    /// leaf's type *means* had to hand the spelling back to the registry and
    /// hope for a cell — the round trip #263 removed from `api/core`, surviving
    /// in the plans, and answering "no layer" for a type it had never seen
    /// (#275). The composed ones (`&Source`) are built by
    /// [`TypeRef::borrowed`](prebindgen::core::flat::TypeRef::borrowed), which
    /// pairs the kind with its own spelling.
    pub out_ty: prebindgen::core::flat::TypeRef,
    /// `true` for the move/clone-the-value handle leaf, emitted **last** (after
    /// every reference leaf's JVM conversion has ended its borrow).
    pub identity: bool,
    /// `true` when a nesting accessor on [`Self::path`] returns `Option` (M3):
    /// the reached value may be absent, so the leaf is nullable on the
    /// destination side (e.g. a Kotlin `?` type); emit wraps the encode in a
    /// `match Some/None`.
    pub nullable: bool,
    /// How [`Self::path`] is reached from the value — an accessor-fn chain
    /// (default), a struct-field chain (synthesized `data_class`), or a
    /// variant pattern binding (decomposed sum).
    pub source: LeafSource,
    /// **Group membership**: `Some(tag)` marks the leaf as belonging to the
    /// leaf group of the sum alternative with that tag — live only when the
    /// value's [`LeafSource::SumTag`] leaf equals `tag`, wire-defaulted
    /// otherwise. `None` for an unconditional (product) leaf, including the
    /// tag leaf itself, which selects between groups rather than joining one.
    ///
    /// Grouping is what turns a leaf list into a `match`: leaves sharing a
    /// group are emitted together in one arm instead of as independent
    /// per-leaf expressions.
    pub group: Option<i32>,
}

impl UnfoldLeaf {
    /// Whether this leaf's [`out_ty`](Self::out_ty) needs a resolved **output
    /// converter**. False only for the synthesized [`LeafSource::SumTag`]
    /// selector: it is assigned per `match` arm, never converted, so requiring
    /// a converter for it would make every sum depend on an unrelated `i32`
    /// crossing existing in the binding.
    ///
    /// **This is the root question, not the registration question.** Every
    /// leaf's `out_ty` gets a table cell; this decides which of them the
    /// binding additionally *demands* a converter for. A cell says the type
    /// entered the pipeline, a root says the binding asked for it directly, and
    /// an entry says one resolved — three separate claims, and a `SumTag` leaf
    /// makes only the first (#282). See
    /// [`Registry::reference_output`](crate::registry::Registry::reference_output).
    pub fn has_converter(&self) -> bool {
        self.source != LeafSource::SumTag
    }
}
