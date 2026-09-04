//! The leaves a decomposed value crosses as, and the walk over them.
//!
//! A **leaf** is one value that crosses once a returned struct has been taken
//! apart as far as it goes. Here are the leaf model — where a leaf is reached
//! from, which arms it is live inside, what it converts through — and the walk
//! grouping a leaf list into the arms an emitter renders as a `match`. Which
//! **delivery** those leaves take, a returned value against a foreign callback,
//! is deliberately not here: that follows from what the target language can
//! receive, so it is the adapter's (#680 review).

/// Outer shape wrapping the [core decomposition](`UnfoldShape::Base`).
/// The output-side analog of [`crate::expand::FoldShape`], on the
/// unified [`Shape`](crate::shape::Shape) layer stack:
///   * `Base` — run the accessor's records on the value, producing all
///     leaves and invoking the builder once;
///   * `Optional((), inner)` — `Option<T>`/`Option<&T>` return: `None` ⇒ a null
///     result (builder skipped), `Some` ⇒ decompose the inner;
///   * `Iterable(inner)` — `Vec<T>` return: fold the elements through an
///     accumulator `(acc, …) -> acc`. Each element is delivered either WHOLE (via
///     its own output converter + projection — see an adapter's plan `element`) or
///     DECOMPOSED into per-element leaves (explicit accessors, or a synthesized
///     `data_class` — see an adapter's plan `fixed_builder`); inner is `Base`.
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
/// each function's an adapter's decomposition plan, which points here via an adapter's plan `decon`.
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
    /// the adapter's sum deconstruction).
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
    /// number, and [`segments`] reads one nesting
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
    /// **This is the root question, not the registration question.** A cell
    /// says the type entered the pipeline, a root says the binding asked for it
    /// directly, and an entry says one resolved — three separate claims, and
    /// this answers only the second: a leaf that answers `true` is handed over
    /// as a delivered output and demands a converter. A `SumTag` leaf makes no
    /// root, and its `out_ty` gets its cell from the binding's own declaration
    /// of the sum rather than from the plan (#282).
    pub fn has_converter(&self) -> bool {
        // A presence flag is synthesized like a tag: it says whether the
        // leaves after it carry anything, and the value it tests crosses
        // through those leaves rather than through this one.
        !matches!(self.source, LeafSource::SumTag | LeafSource::Presence)
    }
}

use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::flat::TypeRef;

/// What the walk needs to know about one delivered leaf.
///
/// The adapter's own leaf carries far more — its wire type, its JNI or C
/// encoding, its projection — and none of that reaches here.
pub trait DecomposedLeaf {
    /// Steps from the decomposed value to this leaf.
    fn reach(&self) -> &[PathStep];

    /// The leaf's own name, used in diagnostics and in local names.
    fn name(&self) -> &str;

    /// Whether this leaf *is* the decomposed value rather than a part of it.
    fn identity(&self) -> bool;

    /// The reading the leaf delivers.
    fn source(&self) -> &TypeRef;

    /// Whether this leaf is a synthesized **selector** — a value naming which
    /// group of the leaves after it is live. It chooses between groups rather
    /// than joining one, so its own [`Self::groups`] is the path it sits in,
    /// not one of the arms it chooses.
    ///
    /// Two kinds select: a sum's tag, which names the live alternative, and an
    /// optional value's presence, which says whether its one group carries
    /// anything.
    fn selects(&self) -> bool;

    /// The arms this leaf sits inside, outermost first — empty for a leaf that
    /// is always live. A selector carries the path it is nested in, and its own
    /// members extend that path by one.
    ///
    /// See [`UnfoldLeaf::groups`](UnfoldLeaf::groups) for what
    /// an arm number means and why the path is what lets one selector own
    /// another.
    fn groups(&self) -> &[i32];

    /// Whether the last step reaching it is a field read rather than a call.
    ///
    /// Derived from [`Self::reach`], so no implementation can disagree with
    /// its own path — and the answer decides whether the reached place is
    /// cloned, which is an ownership decision rather than a spelling.
    fn is_field_read(&self) -> bool {
        matches!(self.reach().last(), Some(PathStep::Field { .. }))
    }
}

/// One delivered value's slot, as the walk needs to see it: a slot exists for
/// every leaf whether or not the value behind it does, so a gate that skips an
/// encode still has to declare the slot and fill it.
pub struct Slot {
    /// The slot's type.
    pub ty: TokenStream,
    /// What an unfilled slot carries.
    pub default: TokenStream,
}

/// A leaf's reach, rendered **around** an encoding body: the walk hands the
/// body the reached Rust expression and gets back the encoded value, so an
/// absent value short-circuits to the adapter's own absence rather than to one
/// the walk chose. `dyn` on both halves because each side is written where it
/// is known and neither can name the other's closure.
pub type Reach<'a> = dyn Fn(&dyn Fn(TokenStream) -> TokenStream) -> TokenStream + 'a;

/// The adapter's half of a delivery.
///
/// The walk in this module owns how a decomposed value is reached — which
/// hoist a leaf sits under, what is owned, what is borrowed. What a leaf
/// becomes in the target language, and how the delivered values are handed to
/// the call, is the adapter's, and this is where it says so. Stated over
/// [`DecomposedLeaf`] rather than over an adapter's own leaf type, so the walk
/// can call it.
pub trait DeliveryBridge {
    /// The adapter's leaf.
    type Leaf: DecomposedLeaf;

    /// Where the source item declaring `ident` is qualified from — the one
    /// fact about a captured path this crate cannot answer for itself.
    fn qualify(&self, ident: &syn::Ident) -> syn::Path;

    /// The sum a selector leaf names: the qualified source path of the type,
    /// and its Flat shape.
    fn sum(&self, leaf: &Self::Leaf) -> (syn::Path, &crate::flat::Variant);

    /// Encode one leaf into the value the delivery call receives, and bind
    /// that value to `slot`.
    ///
    /// The adapter emits the binding rather than the walk, because the slot's
    /// type is the adapter's: one leaf may cross as a scalar and the next as a
    /// reference. `index` is the leaf's position in the delivery, which is
    /// what any temporary the encode needs is named from. An encoding that can
    /// fail routes its failure through `fail`.
    fn encode(
        &self,
        leaf: &Self::Leaf,
        index: usize,
        slot: &syn::Ident,
        reach: &Reach<'_>,
        fail: &dyn Fn(TokenStream) -> TokenStream,
        emit: &crate::RustWriter,
    ) -> TokenStream;

    /// The expression that passes `slot` as one argument of the delivery call.
    fn argument(&self, leaf: &Self::Leaf, slot: &syn::Ident) -> TokenStream;

    /// What one leaf's slot holds when the walk fills it without running the
    /// leaf's encode — the `None` arm of a gate, where a slot exists but the
    /// value behind it does not.
    fn slot(&self, leaf: &Self::Leaf) -> Slot;

    /// What a leaf delivers when an optional step on the way to it found
    /// nothing. [`reach_leaf`] gates on this rather than on a value of its
    /// own: absence is a target-language representation, and this crate has
    /// none.
    fn absent(&self) -> TokenStream;
}

/// Compose one [`PathStep`] onto the reference expression reached so far.
/// A `Call` applies its accessor (origin-qualified); a `Field` reads the field
/// and re-borrows, so the result is a reference either way and steps chain
/// uniformly.
pub fn compose_step(
    qualify: &dyn Fn(&syn::Ident) -> syn::Path,
    step: &PathStep,
    e: TokenStream,
) -> TokenStream {
    match step {
        PathStep::Call { ident, .. } => {
            let m = qualify(ident);
            quote!(#m::#ident(#e))
        }
        PathStep::Field { ident, .. } => quote!(&(#e).#ident),
    }
}
/// Fold a run of steps onto `e`, borrowing wherever ownership demands it.
///
/// [`compose_step`] hands a `Call` its receiver as written, and an accessor
/// takes that receiver **by reference** — so an owned value in hand has to be
/// borrowed before the next call composes onto it. A value is in hand whenever
/// the previous step returned one (`f(..) -> T` rather than `-> &T`), which is
/// what [`PathStep::yields_owned`] records; `owned` says whether `e` itself
/// started that way.
///
/// A `Field` step needs no borrow either way — it composes as `&(e).f`, which
/// reads through a value and a reference alike.
///
/// This is the ONE place the rule lives, so every fold — a leaf's reach, a
/// conditional hoist's prefix, a sum's matched value — answers it the same way.
/// Splitting it produced exactly the bug it exists to prevent: ownership was
/// handled at the optional binding and at the value form, and lost at every
/// ordinary call in between.
pub fn fold_steps(
    qualify: &dyn Fn(&syn::Ident) -> syn::Path,
    steps: &[PathStep],
    mut e: TokenStream,
    mut owned: bool,
) -> TokenStream {
    for step in steps {
        if owned && matches!(step, PathStep::Call { .. }) {
            e = quote!(&#e);
        }
        e = compose_step(qualify, step, e);
        owned = step.yields_owned();
    }
    e
}
/// Compose a value form's OWN call — the one step in the whole system whose
/// receiver may be by value.
///
/// Four cases, from what the fold ended up holding crossed with what the
/// accessor takes. A CONSUMING form takes its receiver by value: hand it the
/// value when that is ours, clone when it is not — the same cost the borrowing
/// form of the accessor would have paid, which keeps one declaration usable by
/// both owned and `&T` roots. A borrowing form takes a reference, so an owned
/// value is borrowed for it.
///
/// The decision is made from the fold's RESULT, never from where the fold
/// began: that is what lets a consuming form sit behind ordinary accessors,
/// where the chain in front borrows and the form itself still moves.
///
/// One function because both hoist paths — the conditional binding and the
/// ordinary one — need exactly this rule, and stating it twice is what turned
/// each new shape into a new defect.
fn compose_value_form_call(
    qualify: &dyn Fn(&syn::Ident) -> syn::Path,
    call: &PathStep,
    e: TokenStream,
    e_owned: bool,
    consuming: bool,
) -> TokenStream {
    match (consuming, e_owned) {
        (true, owned) => {
            let (m, f) = (qualify(call.ident()), call.ident());
            // Parenthesized: the clone applies to whatever the fold holds, and
            // `&x.clone()` would parse as `&(x.clone())`.
            let arg = if owned { e } else { quote!((#e).clone()) };
            quote!(#m::#f(#arg))
        }
        (false, true) => compose_step(qualify, call, quote!(&#e)),
        (false, false) => compose_step(qualify, call, e),
    }
}
/// Start a reach from `base`, projecting the **leading run of plain field
/// steps** directly (`&base.a.b`) instead of through a borrow of the base
/// (`&(&base).a.b`). Returns the expression and how many steps it consumed.
///
/// The two forms name the same value, but the second borrows the base **as a
/// whole**, which the borrow checker rejects once a sibling leaf has moved a
/// different field out of it. Projecting directly makes each leaf's borrow
/// disjoint, so a consuming value form's field moves are order-independent
/// rather than compiling only while the borrowing leaves happen to be declared
/// first.
pub fn project_leading_fields(
    base: &TokenStream,
    base_is_ref: bool,
    path: &[PathStep],
) -> (TokenStream, usize) {
    if base_is_ref {
        return (base.clone(), 0);
    }
    let n = path.iter().take_while(|s| s.is_plain_field()).count();
    if n == 0 {
        return (quote!(&#base), 0);
    }
    let segs: Vec<&syn::Ident> = path[..n].iter().map(PathStep::ident).collect();
    (quote!(&#base #(.#segs)*), n)
}

/// Where one leaf's reach starts, once the hoists are bound.
///
/// Produced by `Hoisted::place`, which is the one place that decides it: the
/// innermost value form the leaf sits under, the name a conditional form's
/// `Some` arm binds, or the delivered value itself.
pub struct LeafPlace {
    /// What to reach from.
    pub base: TokenStream,
    /// Whether `base` is already a reference.
    pub base_is_ref: bool,
    /// The steps left from `base` — the leaf's own path with the prefix that
    /// bound the hoist already consumed.
    pub path: Vec<PathStep>,
    /// Whether the form that produced `base` gave its value away.
    pub consuming: bool,
    /// The conditional hoist whose `Some` arm this leaf belongs in. Its
    /// statements go there rather than beside the others, because the arm is
    /// where the binding they reach off exists.
    pub conditional: Option<usize>,
}

impl LeafPlace {
    /// The place this leaf may MOVE out of, or `None` when what it reaches is
    /// not the delivery's to give away, or is not a place a move can name.
    ///
    /// The same question [`reach_leaf`] answers on its way to the terminal
    /// treatment, asked separately by a leaf whose encode needs the place
    /// itself rather than an expression reaching it — a handle that is boxed
    /// rather than converted.
    pub fn owned<L: DecomposedLeaf>(&self, leaf: &L) -> Option<TokenStream> {
        (reached_is_ours(leaf, self.consuming) && steps_are_movable(&self.path)).then(|| {
            let base = &self.base;
            let segs: Vec<&syn::Ident> = self.path.iter().map(PathStep::ident).collect();
            quote!(#base #(.#segs)*)
        })
    }
}

/// Whether what a leaf reaches is OURS, and so is moved rather than borrowed
/// or cloned. The two leaf kinds say it differently:
///
/// * an IDENTITY leaf carries the answer in its own reading — the plan
///   resolved that to the owned type exactly when the value is the plan's to
///   give away (an owned root, or a field of a CONSUMING value form), and that
///   is also what selected the owning converter, which boxes the move rather
///   than cloning a borrow;
/// * every other leaf reads a field, whose reading is the field type as
///   written and owned either way, so ownership is the enclosing form's: only
///   a consuming one gives its fields away.
pub fn reached_is_ours<L: DecomposedLeaf>(leaf: &L, consuming: bool) -> bool {
    if leaf.identity() {
        !matches!(leaf.source().kind(), crate::flat::TypeKind::Ref { .. })
    } else {
        consuming
    }
}

/// One leaf, and the place a reach of it starts from.
///
/// A caller that has already bound a hoist hands over the local it bound and
/// the steps that are left, which is why `path` is a suffix of the leaf's own
/// reach rather than always equal to it.
pub struct LeafAt<'a, L> {
    /// The leaf being reached.
    pub leaf: &'a L,
    /// The steps still to walk from `base`.
    pub path: &'a [PathStep],
    /// What the walk starts from — the delivered value, or a local a hoist or
    /// a gate bound.
    pub base: TokenStream,
    /// Whether `base` is already a reference.
    pub base_is_ref: bool,
    /// Whether the form that produced `base` gave its value away, which is
    /// what lets a field of it be moved rather than cloned.
    pub consuming: bool,
    /// Whether a FINAL optional step is gated too. An identity leaf delivers
    /// the reached value itself, so it is; every other leaf hands the final
    /// step's full type to its own converter, `Option` and all.
    pub unwrap_last: bool,
}

/// Reach one leaf from `base` and hand what it reached to `body`.
///
/// Three things happen on the way, and all three are facts about Rust values
/// rather than about any target language:
///
/// * **Gating.** An optional step before the leaf becomes a `match` whose
///   `None` arm yields `absent()` — the adapter's own absence, since this
///   crate has none to offer. `unwrap_last` says whether a FINAL optional step
///   is gated too: an identity leaf delivers the reached value itself, so it
///   is, while every other leaf hands the final step's full type to its own
///   converter, `Option` and all.
/// * **Refusal.** `absent` is `None` at a site that cannot express absence at
///   all — a single delivered return value, which has no arm to put one in.
///   An optional step before the end is then refused rather than composed into
///   code the consumer's crate cannot type-check.
/// * **Ownership**: the reached place is moved when it is ours, cloned when
///   it is a field read of a place that is not, and borrowed otherwise.
///
/// One derivation, used by every delivery an adapter renders and by the
/// single-leaf shortcut a decomposed return takes. They drifted once while
/// they were two — the shortcut was missing the field clone and handed `&F` to
/// an `F` converter — and a second pair, ungated here and gated in JniGen,
/// drifted the same way until #607 folded them together.
pub fn reach_leaf<L: DecomposedLeaf>(
    qualify: &dyn Fn(&syn::Ident) -> syn::Path,
    at: LeafAt<'_, L>,
    absent: Option<&dyn Fn() -> TokenStream>,
    body: &dyn Fn(TokenStream) -> TokenStream,
) -> TokenStream {
    reach_leaf_at(qualify, at, absent, 0, body)
}

fn reach_leaf_at<L: DecomposedLeaf>(
    qualify: &dyn Fn(&syn::Ident) -> syn::Path,
    at: LeafAt<'_, L>,
    absent: Option<&dyn Fn() -> TokenStream>,
    depth: usize,
    body: &dyn Fn(TokenStream) -> TokenStream,
) -> TokenStream {
    let LeafAt {
        path,
        ref base,
        base_is_ref,
        unwrap_last,
        ..
    } = at;
    // Every optional step on the way to the leaf becomes a `match` whose
    // `None` arm is the adapter's absent value. A site with no absent value to
    // give — a single delivered return, which has no arm to put one in —
    // passes `None`, and an optional step before the end is then refused
    // rather than composed into code the consumer's crate cannot type-check.
    if let Some(absent) = absent {
        let limit = if unwrap_last {
            path.len()
        } else {
            // A non-identity leaf's converter takes the final step's FULL type,
            // `Option` included, so only the steps before it are nesting.
            path.len().saturating_sub(1)
        };
        let (projected, lead) = project_leading_fields(base, base_is_ref, path);
        if let Some(k) = (lead..limit).find(|&i| path[i].is_optional()) {
            // Through the optional step INCLUSIVE: the same fold, so the borrow
            // in front of it is the ordinary rule rather than a second
            // statement of it.
            let opt_e = fold_steps(qualify, &path[lead..=k], projected, false);
            let nested = format_ident!("__n{}", depth);
            // What the arm binds is the step's OWN value: an owned payload is
            // a bare `T`, so composing the next step onto it directly would
            // hand `T` to an accessor typed for `&T`. Say it is not a
            // reference and let `project_leading_fields` borrow it; a borrowed
            // payload is already one and passes through. With no steps left
            // the binding goes to `body` untouched, which is what lets an
            // owned payload be moved rather than borrowed straight back.
            //
            // The same rule [`reach_optional`] states for a conditional
            // hoist's prefix. This used to be a hardcoded `true` in the gated
            // reach JniGen owned, where an `Option<T>` accessor followed by
            // another accessor emitted `next(__n0)` against a `&T` receiver.
            let rest = &path[k + 1..];
            let inner = reach_leaf_at(
                qualify,
                LeafAt {
                    path: rest,
                    base: quote!(#nested),
                    base_is_ref: rest.is_empty() || !path[k].yields_owned(),
                    ..at
                },
                Some(absent),
                depth + 1,
                body,
            );
            let gone = absent();
            // A FIELD read composes to a borrow (`&(e).f`), so it goes through
            // a coercion site and the destructuring stops caring which
            // representation the source spelled the optional as (#268).
            //
            // A CALL composes to the accessor's own returned value, which is
            // owned and whose payload downstream may move. Borrowing it to
            // coerce would change that ownership, so it keeps its direct match.
            if path[k].is_field() {
                let opt_bind = format_ident!("__o{}", depth);
                return quote! {
                    {
                        let #opt_bind: &::core::option::Option<_> = #opt_e;
                        match #opt_bind {
                            ::core::option::Option::Some(#nested) => { #inner }
                            ::core::option::Option::None => #gone,
                        }
                    }
                };
            }
            return quote! {
                match #opt_e {
                    ::core::option::Option::Some(#nested) => { #inner }
                    ::core::option::Option::None => #gone,
                }
            };
        }
        return body(reached_place(qualify, &at));
    }
    // An optional step BEFORE the last one needs a `match` whose `None` arm has
    // somewhere to go. This derivation has none — it yields a plain Rust value,
    // not a representation that can carry absence — so the shape is refused
    // here rather than composed into code that cannot type-check in the
    // consumer's crate.
    //
    // Asked of the leaf's OWN path, not of `path`. The caller may hand a
    // suffix: `wrapper.rs` rebases onto a hoisted local, and `Hoisted::innermost`
    // strips the prefix that bound it — including any optional step inside it.
    // Checking the parameter would therefore pass exactly when the hoist is the
    // conditional one, which is the case that cannot compose (an `Option<T>`
    // local with a field read hung off it). The full path is what the shape
    // question is about.
    let own_path = at.leaf.reach();
    assert!(
        !own_path.iter().rev().skip(1).any(PathStep::is_optional),
        "unfold: leaf `{}` reaches through an optional step but is \
         delivered as a single return value, which has no `None` arm — this \
         shape needs callback delivery",
        at.leaf.name(),
    );
    // Whether what this leaf reaches is OURS, and so is moved rather than
    // borrowed or cloned. The two leaf kinds say it differently:
    //
    // * an IDENTITY leaf carries the answer in its `out_ty` — the plan resolved
    //   it to the owned type exactly when the value is the plan's to give away
    //   (`place_is_owned`: an owned root, or a field of a CONSUMING value form),
    //   and that is also what selected the owning converter, which boxes the
    //   move rather than cloning a borrow;
    // * a FIELD leaf's `out_ty` is the field type as written, owned either way,
    //   so ownership is the enclosing form's: only a consuming one gives its
    //   fields away.
    //
    // How to project that place is `steps_are_movable`'s question, and it is
    // asked there rather than restated here. This used to spell it
    // `all(is_plain_field)`, defending the restatement on the grounds that a
    // trailing `Option` cannot reach return delivery anyway — true, and enforced
    // in `single_return` (`core/unfold.rs`), which is precisely why a local
    // restatement could disagree with the rule for as long as the invariant held
    // somewhere else. `plan.rs` says two readings would drift and the
    // disagreement would be a borrow handed to an owning converter; this is the
    // second reading, removed.
    body(reached_place(qualify, &at))
}

/// The place a leaf reaches, with the terminal treatment its ownership calls
/// for: moved out when it is ours and the path projects a place, cloned out
/// when it is a field read of a place that is not, and borrowed otherwise.
fn reached_place<L: DecomposedLeaf>(
    qualify: &dyn Fn(&syn::Ident) -> syn::Path,
    at: &LeafAt<'_, L>,
) -> TokenStream {
    let LeafAt {
        leaf,
        path,
        base,
        base_is_ref,
        consuming,
        ..
    } = at;
    let (base_is_ref, consuming) = (*base_is_ref, *consuming);
    // How to project a movable place is `steps_are_movable`'s question, asked
    // there rather than restated here. This used to spell it
    // `all(is_plain_field)`, defending the restatement on the grounds that a
    // trailing `Option` cannot reach return delivery anyway — true, and
    // enforced in `single_return`, which is precisely why a local restatement
    // could disagree with the rule for as long as the invariant held somewhere
    // else. `plan.rs` says two readings would drift and the disagreement would
    // be a borrow handed to an owning converter; this is the second reading,
    // removed.
    if reached_is_ours(*leaf, consuming) && steps_are_movable(path) {
        let segs: Vec<&syn::Ident> = path.iter().map(PathStep::ident).collect();
        return quote!(#base #(.#segs)*);
    }
    let (e, lead) = project_leading_fields(base, base_is_ref, path);
    let e = fold_steps(qualify, &path[lead..], e, false);
    // Cloned out of the BORROW the reach yields, not out of the place behind
    // it. The two agree for a field of an owned type and disagree for a field
    // that is itself a reference: `place.clone()` there resolves through the
    // reference and deep-clones the pointee, where the converter takes the
    // field type as written. JniGen's own reach spelled it the short way and
    // no declared field exercised the difference — the drift this step exists
    // to close.
    if leaf.is_field_read() {
        quote!((#e).clone())
    } else {
        e
    }
}
/// The **segments** of a decomposition: each selector leaf, together with the
/// group leaves that follow it.
///
/// A segment's leaves are not independent — one alternative of a sum is live
/// per value, and a gated group is live only when its value is there — so a
/// segment is emitted as one `match` rather than as a leaf at a time. The plan
/// already says which leaves those are: a selector says so of itself, and each
/// of the leaves after it carries the group it belongs to until one does not.
///
/// **One nesting level at a time.** A group's leaves may themselves include a
/// selector — an optional nested class whose own fields choose, an `Option` of
/// a sum — and the ranges returned here would overlap if such an inner selector
/// opened a segment of its own. So this answers for one level: at `depth`, the
/// selectors whose arm path is exactly that deep, each with everything nested
/// under it. A renderer that has entered an arm asks again one level down, and
/// the inner selectors become the segments of that answer.
///
/// A decomposition may carry several segments, or none: a sum that IS the
/// delivered value is one segment covering everything, and a value form
/// contributes one per sum-typed field.
///
/// Not recognising a segment is silent rather than a compile error — the
/// leaves are all there, and a delivery that treats them as independent reads
/// a dead alternative's fields — which is why this is the registry's answer
/// and not an adapter's.
pub fn segments<L: DecomposedLeaf>(leaves: &[L]) -> Vec<std::ops::Range<usize>> {
    segments_at(leaves, 0)
}

/// [`segments`] one nesting level down: the selectors whose arm path is exactly
/// `depth` long, with the leaves nested under each.
///
/// `depth` is how many arms the caller has already entered, so a leaf deeper
/// than that belongs to the segment it follows rather than opening one.
pub fn segments_at<L: DecomposedLeaf>(leaves: &[L], depth: usize) -> Vec<std::ops::Range<usize>> {
    let n = leaves.len();
    let nested = |i: usize| leaves[i].groups().len() > depth;
    // Exactly this deep, not "no deeper": a selector shallower than `depth`
    // encloses the level being asked about rather than sitting on it, and
    // answering with it would return the enclosing segment a second time.
    (0..n)
        .filter(|&i| leaves[i].selects() && leaves[i].groups().len() == depth)
        .map(|start| {
            let end = (start + 1..n)
                .take_while(|&i| nested(i))
                .last()
                .map_or(start + 1, |i| i + 1);
            start..end
        })
        .collect()
}

/// Render one segment off `place`, gating the whole of it when the selector
/// reaches the value it selects on through an optional step.
///
/// A segment's **selected value** is what its selector chooses over: the sum a
/// tag names an alternative of, or the optional value a presence flag says is
/// there. `group` renders the segment's own encode from the expression naming
/// it — that half is the adapter's, since what a group of slots is filled with
/// is a target-language question.
///
/// **An optional value gates the segment, not each slot.** A segment's leaves
/// are not independent, so absence cannot be the per-leaf absent value
/// [`reach_leaf`] gives an ordinary optional field: it is one tuple bind whose
/// `None` arm carries every slot's default. That is the same shape a
/// conditional value form's hoist emits, applied to an optional step inside
/// the segment's own path (#220).
///
/// `index` names the segment's locals, so two segments of one delivery cannot
/// collide.
pub fn segment<B: DeliveryBridge>(
    bridge: &B,
    qualify: &dyn Fn(&syn::Ident) -> syn::Path,
    place: &LeafPlace,
    index: usize,
    leaves: &[B::Leaf],
    slots: &[syn::Ident],
    group: &dyn Fn(TokenStream) -> TokenStream,
) -> TokenStream {
    let path = &place.path;
    // A plain field chain is borrowed DIRECTLY (`&base.a.b`) rather than
    // through the base (`&(&base).a.b`). The two are the same value, but the
    // second borrows the base as a whole, which the borrow checker rejects
    // once a sibling leaf has moved another field out of it — and borrowing
    // this field while sibling fields move is exactly what a consuming value
    // form does.
    let (projected, lead) = project_leading_fields(&place.base, place.base_is_ref, path);
    // The selector's own path reaches the selected value (empty when that
    // value IS the delivered one), and a step on it MAY be optional — which is
    // always so for a presence flag, whose whole subject is that step.
    let Some(k) = (lead..path.len()).find(|&i| path[i].is_optional()) else {
        return group(fold_steps(qualify, &path[lead..], projected, false));
    };
    // Through the optional step INCLUSIVE, then the rest off the binding — the
    // same split [`reach_leaf`] makes, so the borrow in front of it stays the
    // ordinary rule rather than a second statement of it.
    let opt_e = fold_steps(qualify, &path[lead..=k], projected, false);
    let bind = format_ident!("__sg{}", index);
    // ONE optional step is what this gate handles. A second one in the tail
    // would compose `match &Option<..>` against the patterns the group's own
    // encode writes — an E0308 in the consumer's crate — so the named
    // diagnostic is raised here instead.
    //
    // `assert!`, not `debug_assert!`: a build script inherits the consumer's
    // profile, so a debug-only check is absent from exactly the release build
    // where a mis-emission costs the most to diagnose. Same rule, same
    // phrasing, as the single-return optional-step assert in [`reach_leaf`].
    //
    // The condition is what actually breaks, not the stronger fact that
    // happens to hold: a selector's path stops AT the value it selects on, so
    // the tail is empty today, but a NON-optional tail composes correctly
    // through [`fold_steps`] — refusing it would refuse a shape that works.
    assert!(
        !path[k + 1..].iter().any(PathStep::is_optional),
        "unfold: leaf `{}` reaches the value it selects on through TWO optional \
         steps — the segment gate has one `None` arm, so the second would be \
         matched as if it were that value itself",
        leaves
            .first()
            .expect("a segment has at least its selector")
            .name(),
    );
    // What the `match` binds, asked of the step rather than assumed: the FIELD
    // branch scrutinizes `&Option<_>`, so ergonomics binds a borrow of the
    // selected value. Only an owned-yielding CALL binds an owned one.
    let inner = group(fold_steps(
        qualify,
        &path[k + 1..],
        quote!(#bind),
        path[k].yields_owned(),
    ));
    let filled: Vec<Slot> = leaves.iter().map(|leaf| bridge.slot(leaf)).collect();
    let tys = filled.iter().map(|slot| &slot.ty);
    let defaults = filled.iter().map(|slot| &slot.default);
    // A FIELD step composes to a borrow, so it goes through a coercion site
    // and the destructure stops caring which representation the source spelled
    // the optional as (#268). A CALL yields its own owned value, whose payload
    // downstream may move, so it keeps the direct match — the same division
    // [`reach_leaf`] makes.
    let (prelude, scrutinee) = if path[k].is_field() {
        let opt_bind = format_ident!("__so{}", index);
        (bind_as_option(&opt_e, &opt_bind), quote!(#opt_bind))
    } else {
        (TokenStream::new(), opt_e)
    };
    quote! {
        let (#(#slots,)*): (#(#tys,)*) = {
            #prelude
            match #scrutinee {
                ::core::option::Option::Some(#bind) => {
                    #inner
                    (#(#slots,)*)
                }
                ::core::option::Option::None => (#(#defaults,)*),
            }
        };
    }
}

/// The `match` a **conditional** value form's leaves share.
///
/// A hoist under an optional step ran only where the value was present, so its
/// leaves cannot be independent statements: their slots exist either way, and
/// only one arm computes them. The `Some` arm runs `body` — the statements
/// those leaves contributed — and yields their slots as a tuple; the `None`
/// arm yields each slot's default, the same shape [`segment`] gives an absent
/// sum. Binding the tuple outside the `match` keeps the slots in scope for the
/// call's argument list, which is indifferent to how a slot was filled.
///
/// Matched BY VALUE: the local is this arm's alone, every leaf under the hoist
/// being in it, so a consuming value form's fields move out here exactly as
/// they do at an unconditional one.
pub fn conditional_arm<B: DeliveryBridge>(
    bridge: &B,
    hoisted: &Hoisted,
    index: usize,
    leaves: &[B::Leaf],
    slots: &[syn::Ident],
    body: TokenStream,
) -> TokenStream {
    let local = hoisted.local(index);
    let bind = format_ident!("__u{}", index);
    let under: Vec<usize> = (0..leaves.len())
        .filter(|&k| {
            hoisted
                .conditional(leaves[k].reach())
                .is_some_and(|(j, ..)| j == index)
        })
        .collect();
    let ids: Vec<&syn::Ident> = under.iter().map(|&k| &slots[k]).collect();
    let filled: Vec<Slot> = under.iter().map(|&k| bridge.slot(&leaves[k])).collect();
    let tys = filled.iter().map(|slot| &slot.ty);
    let defaults = filled.iter().map(|slot| &slot.default);
    quote! {
        let (#(#ids,)*): (#(#tys,)*) = match #local {
            ::core::option::Option::Some(#bind) => { #body (#(#ids,)*) }
            ::core::option::Option::None => (#(#defaults,)*),
        };
    }
}

/// Bind `e` so it can be destructured as an `Option` **whatever Rust
/// representation the source used for it**.
///
/// The model says a position is optional; it deliberately does not say whether
/// Rust spells that `Option<T>`, `Box<Option<T>>`, or something else — the
/// side interpreting the classification is the side that must accept any
/// representation. Matching the reached place directly assumed one, which is
/// "classify off the model, spell from the model" broken in the direction
/// nothing was watching: `Box<Option<T>>` then produced
/// `match &place { Some(..) => .. }` and an E0308 (#268).
///
/// A type-ascribed `let` is a coercion site, and deref coercion is transitive
/// **and** a no-op when the types already match — so this one shape serves
/// every representation, and the plain `Option<T>` case is unchanged.
///
/// `e` is expected to be a **reference** already — [`compose_step`] composes a
/// field read as `&(e).f` — so nothing is borrowed here. Borrowing only: an
/// owned position cannot be made representation-agnostic this way, because
/// deref coercion applies to references and moving out of a wrapper is
/// something only some of them permit (`Box` does, `Rc` cannot). A site that
/// must MOVE the payload keeps its direct match.
pub fn bind_as_option(e: &TokenStream, bind: &syn::Ident) -> TokenStream {
    quote! { let #bind: &::core::option::Option<_> = #e; }
}

/// Every value form on a plan, evaluated **once** and bound to a local
/// (`__vf0`, `__vf1`, …), so a struct is built once per delivery rather than
/// once per field. The bound prefixes come back with the statements, since
/// reaching a leaf means starting from the innermost local it sits under.
///
/// Shared by both delivery paths — the multi-leaf encoder below and the
/// single-leaf `Delivery::Return` shortcut in `emit/wrapper.rs`. The shortcut
/// used to compose its reach straight off the raw value, which for a consuming
/// value form emitted `f(&v)` against a by-value receiver: ill-typed Rust in
/// the consumer's crate. One binder, so the two cannot disagree about what a
/// hoist is or who owns it.
pub struct Hoisted {
    /// The `let __vfN = …;` bindings, outermost-first.
    pub stmts: TokenStream,
    /// Each hoist's path prefix and the local it was bound to.
    bound: Vec<(Vec<PathStep>, syn::Ident)>,
    /// Whether each bound hoist consumed the value it decomposed.
    consuming: Vec<bool>,
    /// Whether each bound local is `Option<TStruct>` rather than `TStruct` —
    /// the hoist sits under an optional step, so the value form ran only where
    /// the value was present. Its leaves cannot be emitted as independent
    /// statements: they share ONE `match` on the local (see
    /// [`encode_plan_leaves`]), taken by value, so a consuming form's fields
    /// still move out inside the arm.
    optional: Vec<bool>,
}
impl Hoisted {
    /// Index of the innermost bound hoist `path` sits under, with that prefix
    /// already consumed. `None` for a leaf under no value form at all — a
    /// sibling `.field()` / `.field_self()`, which still reaches from the value
    /// itself.
    fn innermost(&self, path: &[PathStep]) -> Option<(usize, Vec<PathStep>)> {
        self.bound
            .iter()
            .enumerate()
            .filter(|(_, (p, _))| p.len() < path.len() && path.starts_with(p))
            .max_by_key(|(_, (p, _))| p.len())
            .map(|(i, (p, _))| (i, path[p.len()..].to_vec()))
    }

    /// The innermost bound local `path` sits under, with that prefix already
    /// consumed, and whether that hoist gave its value away.
    pub fn rebase(&self, path: &[PathStep]) -> Option<(syn::Ident, Vec<PathStep>, bool)> {
        self.innermost(path)
            .map(|(i, rest)| (self.bound[i].1.clone(), rest, self.consuming[i]))
    }

    /// The innermost **conditional** hoist `path` sits under: its index, the
    /// local holding the `Option`, the name its `Some` arm binds, and the steps
    /// left to reach the leaf from there. `None` when the leaf's innermost
    /// hoist is unconditional (or there is none) — then [`Self::rebase`]
    /// applies and the leaf is an ordinary standalone statement.
    pub fn conditional(
        &self,
        path: &[PathStep],
    ) -> Option<(usize, syn::Ident, syn::Ident, Vec<PathStep>)> {
        let (i, rest) = self.innermost(path)?;
        self.optional[i].then(|| (i, self.bound[i].1.clone(), format_ident!("__u{}", i), rest))
    }

    /// Where a leaf's reach starts, once these hoists are bound.
    ///
    /// Three cases, and the leaf's own path decides which.
    ///
    /// A leaf under a **conditional** value form reaches off the name that
    /// form's `Some` arm binds. That arm matches the hoist's `Option` by
    /// value — every leaf under the hoist is inside it, so the local is the
    /// arm's alone — which makes the binding an owned payload: a consuming
    /// form's fields move out of it exactly as they do at an unconditional
    /// one, and [`LeafPlace::owned`] says so. The leaf's statements belong in
    /// that arm, because that is where the binding exists.
    ///
    /// A leaf under an **ordinary** value form reaches off the local that form
    /// was bound to, with the prefix already consumed and the form's own
    /// ownership carried along.
    ///
    /// A leaf under **no** value form — a sibling accessor of the delivered
    /// value — reaches from that value.
    pub fn place<L: DecomposedLeaf>(
        &self,
        leaf: &L,
        value: &TokenStream,
        by_ref: bool,
    ) -> LeafPlace {
        let path = leaf.reach();
        if let Some((i, _, bind, rest)) = self.conditional(path) {
            return LeafPlace {
                base: quote!(#bind),
                base_is_ref: false,
                path: rest,
                consuming: self.consumed(i),
                conditional: Some(i),
            };
        }
        match self.rebase(path) {
            Some((local, rest, consuming)) => LeafPlace {
                base: quote!(#local),
                base_is_ref: false,
                path: rest,
                consuming,
                conditional: None,
            },
            None => LeafPlace {
                base: value.clone(),
                base_is_ref: by_ref,
                path: path.to_vec(),
                consuming: false,
                conditional: None,
            },
        }
    }

    /// The local a hoist was bound to.
    pub fn local(&self, i: usize) -> syn::Ident {
        self.bound[i].1.clone()
    }

    /// Whether a hoist consumed the value it decomposed.
    pub fn consumed(&self, i: usize) -> bool {
        self.consuming[i]
    }
}
/// Fold `path` over `base` the way an adapter's own gated reach does, but
/// yielding an `Option<…>` rather than the adapter's absent value: the optional
/// steps become a `map`/`and_then` chain, so an absent value short-circuits to
/// `None` instead of to whatever that adapter uses for absence. `body` renders
/// the innermost reached expression as a BARE value — the chain's last link
/// wraps it.
///
/// The gated reach this mirrors is [`reach_leaf`], which takes the absent
/// value from the adapter rather than choosing one. This one yields an
/// `Option` instead, because what it binds is a hoist rather than a delivered
/// leaf: the value form runs where the value is present, and the `None` the
/// chain produces is the local's own.
///
/// This is how a CONDITIONAL value form is bound — the accessor runs only where
/// the value it decomposes is actually present.
fn reach_optional(
    qualify: &dyn Fn(&syn::Ident) -> syn::Path,
    path: &[PathStep],
    base: TokenStream,
    base_is_ref: bool,
    depth: usize,
    body: &dyn Fn(TokenStream) -> TokenStream,
) -> TokenStream {
    let (e, lead) = project_leading_fields(&base, base_is_ref, path);
    match (lead..path.len()).find(|&i| path[i].is_optional()) {
        None => body(fold_steps(qualify, &path[lead..], e, false)),
        Some(k) => {
            // Through the optional step INCLUSIVE: the same fold, so the
            // borrow in front of it is the ordinary rule rather than a second
            // statement of it.
            let opt_e = fold_steps(qualify, &path[lead..=k], e, false);
            let bind = format_ident!("__hb{}", depth);
            // What the arm binds is the step's own value: an OWNED payload is a
            // bare `T`, so composing the next step onto it directly would hand
            // `T` to an accessor typed for `&T`. Say it is not a reference and
            // let `project_leading_fields` borrow it; a borrowed payload is
            // already one and passes through.
            //
            // With NO steps left the binding goes to `body` untouched — that is
            // what lets a consuming value form MOVE an owned payload rather than
            // borrow it straight back, so the terminal case stays "already a
            // reference" whatever the payload is.
            let rest = &path[k + 1..];
            let inner = reach_optional(
                qualify,
                rest,
                quote!(#bind),
                rest.is_empty() || !path[k].yields_owned(),
                depth + 1,
                body,
            );
            // `map` when this is the LAST optional step (the body yields a bare
            // value) and `and_then` when another follows (the recursion yields
            // an `Option` that must not nest). The equivalent `match` reads the
            // same but generated code runs through the consumer's lints, where
            // `clippy::manual_map` is a denial.
            let combinator = if rest.iter().any(PathStep::is_optional) {
                format_ident!("and_then")
            } else {
                format_ident!("map")
            };
            quote! {
                #opt_e.#combinator(|#bind| #inner)
            }
        }
    }
}
pub fn bind_hoists(
    qualify: &dyn Fn(&syn::Ident) -> syn::Path,
    hoists: &[Hoist],
    value: &TokenStream,
    by_ref: bool,
) -> Hoisted {
    let mut out = Hoisted {
        stmts: TokenStream::new(),
        bound: Vec::new(),
        consuming: Vec::new(),
        optional: Vec::new(),
    };
    // Value forms COMPOSE, so each hoist is built from the longest hoist that
    // is already a proper prefix of it (they arrive outermost-first), and from
    // `value` otherwise.
    for (i, h) in hoists.iter().enumerate() {
        let local = format_ident!("__vf{}", i);
        // A hoist under an optional step binds `Option<TStruct>`: the value
        // form runs in the `Some` arm only. Core refuses to nest these, so the
        // enclosing value is always the plan's own — no rebase to consider.
        if h.prefix.iter().any(PathStep::is_optional) {
            let (last, lead) = h
                .prefix
                .split_last()
                .expect("a hoist prefix ends in its value-form call");
            let consuming = h.consuming;
            // The value form is handed the payload only when the optional step
            // is the LAST thing before it; any step in between composes as a
            // borrow, so what arrives is a reference either way.
            let owned = lead.last().is_some_and(PathStep::yields_owned);
            let expr = reach_optional(qualify, lead, value.clone(), by_ref, 0, &|reached| {
                compose_value_form_call(qualify, last, reached, owned, consuming)
            });
            out.stmts.extend(quote! { let #local = #expr; });
            out.bound.push((h.prefix.clone(), local));
            out.consuming.push(h.consuming);
            out.optional.push(true);
            continue;
        }
        // Where the fold starts, and whether what it starts from is OWNED. The
        // value form's own boundary is decided below, from what the fold ends
        // up holding — never from where it began.
        let (from, start, start_owned) = match out.rebase(&h.prefix) {
            // A NESTED consuming form is handed the parent's field by MOVE: a
            // hoisted value form is an owned struct and its fields are
            // disjoint, so moving one out leaves every sibling leaf readable.
            // `compose_step` borrows (`&(e).f`), so a plain field run to that
            // field is projected here instead of going through it.
            Some((outer, rest, _))
                if h.consuming && rest[..rest.len() - 1].iter().all(PathStep::is_plain_field) =>
            {
                let lead = &rest[..rest.len() - 1];
                let segs: Vec<&syn::Ident> = lead.iter().map(PathStep::ident).collect();
                (h.prefix.len() - 1, quote!(#outer #(.#segs)*), true)
            }
            // Any other rebased hoist: project its own leading field run
            // DIRECTLY off the parent local rather than reaching it through a
            // borrow of the parent. A sibling hoist may already have moved a
            // different field out — that is what a consuming value form does —
            // and `&(&__vf0).wrapper` borrows the partially moved parent as a
            // whole where `&__vf0.wrapper` is a disjoint borrow that survives.
            // Same invariant `project_leading_fields` states for leaf reaches,
            // and the same reason.
            Some((outer, rest, _)) => {
                let (e, lead) = project_leading_fields(&quote!(#outer), false, &rest);
                (h.prefix.len() - rest.len() + lead, e, false)
            }
            None if by_ref => (0, value.clone(), false),
            None => (0, value.clone(), true),
        };
        // Everything before the value form is an ordinary accessor chain.
        let last = h.prefix.len() - 1;
        let head = &h.prefix[from..last];
        let e = fold_steps(qualify, head, start, start_owned);
        let e_owned = head.last().map_or(start_owned, PathStep::yields_owned);
        // The value form itself. A CONSUMING one takes its receiver BY VALUE —
        // that is the move the whole declaration exists for — so it is handed
        // what the fold holds when that is ours, and a clone when it is not:
        // the same cost the borrowing form of the accessor would have paid,
        // which keeps one declaration usable by both owned and `&T` returns.
        // A borrowing one takes a reference, so an owned value is borrowed.
        //
        // Deciding this from the fold's RESULT rather than from its start is
        // what lets a consuming form sit behind ordinary accessors: the chain
        // in front borrows, the form itself still moves.
        let expr = compose_value_form_call(qualify, &h.prefix[last], e, e_owned, h.consuming);
        out.stmts.extend(quote! { let #local = #expr; });
        out.bound.push((h.prefix.clone(), local));
        out.consuming.push(h.consuming);
        out.optional.push(false);
    }
    out
}
