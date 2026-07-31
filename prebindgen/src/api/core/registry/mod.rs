//! Which type conversions a binding needs, and whether it has them all.
//!
//! # The boundary
//!
//! [`Flat`](crate::core::flat::Flat) describes the source Rust code. A binding
//! puts a wrapper on each side of an FFI boundary — generated Rust that the
//! destination language can call, and destination-language code shaped to match
//! it (`#[repr(C)]` structs and a C header; JNI externs and Kotlin classes).
//!
//! ```text
//!    source flat API              generated wrapper            destination
//!    (idiomatic Rust)                                            language
//!    ────────────────             ─────────────────            ────────────
//!    fn ledger_filed(&Ledger) ──► #[no_mangle] extern fn  ◄──►  external fun
//!         -> Option<Report>         (jlong) -> jlong             fun filed(): Report?
//!                                       ▲
//!                                       └── the boundary: the WIRE
//!                                           jlong / jint / jobject  (JNI)
//!                                           *const T / size_t       (C)
//! ```
//!
//! The wrapper's **body** speaks source Rust; its **signature** speaks wire. The
//! translation between the two is a *conversion*, and collecting them is this
//! module's whole job.
//!
//! # What a conversion is
//!
//! A [`TypeEntry`]: a `destination` (the wire type), a wire-facing `function`,
//! and `pre_stages` — the Rust-side stages that compose with it. **A chain, not a
//! function**, which is how composition works: `Option<Handle>`'s chain embeds
//! `Handle`'s.
//!
//! A composite need not cross whole. `Option<T>` may cross as a `T` carrying a
//! niche value, as a `(bool, T)` pair, or as leaves delivered separately — which,
//! is the adapter's choice, and the registry records it so the emitter can call
//! it by name and the destination side can be written to match.
//!
//! Conversions are **directional**, which is why [`Direction`] is half of a
//! [`Crossing`] rather than a name prefix on two tables. `&str` inbound is a
//! `jstring` to decode, outbound a `jstring` to allocate, and one direction may
//! be convertible while the other is not. A callback flips it — `impl
//! Fn(Sample)` is an *input* whose argument crosses *outbound*.
//!
//! # What the registry does
//!
//! It **derives** the set, then **checks it is complete**.
//!
//! A binding names a surface: these functions, these types, these consts. Far
//! more types than that must convert — parameter and return types, type
//! arguments, struct fields, enum payloads, callback arguments in the flipped
//! direction, and the leaves a decomposed value arrives in. Computing that
//! closure is the work; completeness is meaningful precisely because the set is
//! derived here rather than handed over.
//!
//! **It never writes a conversion.** It cannot — only a language adapter knows
//! what a `jlong` handle or a `*const T` is. The registry decides *which* are
//! needed, asks the adapter for each, and fails naming any that could not be
//! supplied.
//!
//! # In, and out
//!
//! | in | |
//! |---|---|
//! | the model | [`Flat`](crate::core::flat::Flat) — what the source offers |
//! | the crossings | which `(direction, type)` pairs actually cross |
//! | the decompositions | how a composite crosses in pieces: which leaf crossings that adds, and which whole-value crossing it removes |
//! | a conversion builder | the [`Prebindgen`] adapter |
//!
//! Out: a conversion for every type in the closure — or a failure naming the
//! ones that must convert and cannot. The emitter then writes the file: the
//! conversions, and the per-item wrappers that call them.
//!
//! # Using a registry
//!
//! **Configure it, hand over the answers, read it.** In that order, once each:
//!
//! ```text
//!   configure   new(flat) · export(name) · cross(type) · decompose(d)
//!                                                    · depends(from, on)
//!      ↓
//!   the demand  crossings()  → every crossing needing a conversion,
//!      ↓                       sorted so each type's inners come first
//!   the answers supply(map)  → fails naming any reachable crossing with none
//!      ↓
//!   read        flat · exports · conversion(dir, ty) · decomposition(site) · …
//! ```
//!
//! Most types need no declaring: they are reached by walking a declared
//! element's signature, and deriving them per **usage** is what keeps an
//! output-only type from being demanded as an input too. Measured: dropping the
//! declaration-as-root for every type with a captured body leaves the generated
//! output byte-identical.
//!
//! But a type with **no captured item behind it** — `ptr_class!(zenoh::KeyExpr<'static>)`
//! on a re-exported foreign type — appears in no signature this model can walk,
//! so nothing derives it and the declaration is the only statement that it
//! crosses at all. That is what `cross` is for, and why the input cannot be
//! elements alone.
//!
//! ```ignore
//! let mut reg = Registry::new(flat)?;
//! for name in &self.exported     { reg.export(name)?; }
//! for ty in &self.foreign_types  { reg.cross(ty.clone())?; }
//! for d in self.decompositions() { reg.decompose(d)?; }
//!
//! // The generator's own loop, over a plain Vec — the registry is not in it.
//! let mut built = HashMap::new();
//! for c in reg.crossings() {
//!     // `c`'s inners are already in `built`: that is what sorted means.
//!     if let Some(conv) = self.convert(&c, &built) { built.insert(c, conv); }
//! }
//! reg.supply(built)?;
//!
//! self.emit(&reg, out)   // read-only from here
//! ```
//!
//! **Nothing here calls back into the generator** — not by trait hook, and not
//! by a `next_request`/`supply` pull loop either, which is the same protocol
//! with the arrow flipped. The registry answers one question and grades one
//! answer.
//!
//! What makes a single hand-off possible is the **sort**. The demand's edges
//! (`immediate_edges` — generic arguments, tuple/reference/slice targets,
//! declared struct fields, and `impl Fn` arguments with the direction flipped)
//! are structural, so they are known without asking anyone. Ordering
//! the closure by them means a generator building `Option<Handle>` already holds
//! `Handle`, which is why it can work from a flat list instead of being called
//! back per type. It also means each crossing is offered exactly once: a
//! generator's `None` says *cannot*, never *not yet*.
//!
//! A `None` is not itself a failure. The scan over-approximates deliberately
//! (see [`TypeCell::root`]); whether a gap matters is reachability from the
//! exports, which `supply` decides.
//!
//! The structure covers almost every dependency, because an `Option<T>`
//! visibly contains a `T`. What it cannot show is one a *declaration* creates —
//! a `convert!` chaining through a helper's parameter type, or a callback
//! argument delivered as plan leaves. Those are stated with `depends`, and
//! getting one wrong is not silent: the conversion that needed the missing one
//! cannot be built, and `supply` names it.
//!
//! **Cycles** are the one place the order cannot be honoured: a self-referential
//! type (`struct Node { next: Option<Box<Node>> }`) has none. `crossings` breaks
//! such a cycle at its entry, so exactly one member is offered before an inner
//! it contains. A generator that cannot build it omits it, and it is reported
//! like any other gap.
//!
//! Direction is a **parameter**, never part of a name: [`Direction`] already
//! carries it, and one `conversion(dir, ty)` cannot drift the way an
//! `input_`/`output_` pair can — as `required_output_types`, which never grew an
//! input peer, shows.

use std::collections::{HashMap, HashSet};

use crate::{
    api::core::{
        niches::Niches,
        prebindgen::{Prebindgen, Stage},
        types_util::bare_path_ident,
    },
    SourceLocation,
};

mod cell;
mod declare;
mod error;
mod generation;
mod key;
mod model;
mod order;
mod run;
mod scan;
mod view;
mod walk;

pub use self::{
    cell::{Direction, TypeCell, TypeEntry, TypeSubject},
    error::{DuplicateNameError, NotExpressibleEntry, ScanError, WriteRustError},
    generation::Generation,
    key::{TypeKey, TypeKeyParseError},
    view::{Building, Conversions, Crossing},
    walk::{extract_fn_trait_args, immediate_subtype_positions},
};

/// Single owner of everything parsed from the prebindgen source stream.
///
/// The metadata parameter `M` is the language adapter's per-converter
/// extra type, supplied via
/// [`crate::api::core::prebindgen::Prebindgen::Metadata`]. Each
/// [`TypeEntry`] carries one `M` copied in by the resolver from the
/// [`crate::api::core::prebindgen::ConverterImpl`] that produced it.
/// Adapters that don't carry extras leave `M = ()`.
pub struct Registry<M = ()> {
    /// The parsed model these maps project. Held rather than discarded, so a
    /// later stage can ask it what a name means through the registry it already
    /// has — see [`Self::flat`].
    flat: crate::api::core::flat::Flat,
    /// What the binding declared, pushed in through [`Self::export`],
    /// [`Self::export_type`], [`Self::cross`] and [`Self::reference`] before
    /// [`Self::resolve`].
    ///
    /// Stored rather than asked for: the registry never calls the generator to
    /// find out what to build. It is also read after resolution — `write`'s
    /// emission gate is "did the binding declare this item" — so it outlives
    /// the scan that consumes it.
    declared: Declared,
    /// Type tables, one per direction. Each scanned type gets a [`TypeCell`]
    /// holding what the key names, whether the binding asks for it directly, and
    /// the conversion once the generator supplies one.
    ///
    /// **Crate-internal.** Outside, a table is reached through
    /// [`Conversions::conversion`] and [`Self::crossings`] — which is what makes
    /// direction a parameter rather than half of a field name, and what stops
    /// anyone observing a cell before [`Self::supply`] has graded it.
    pub(crate) input_types: HashMap<TypeKey, TypeCell<M>>,
    pub(crate) output_types: HashMap<TypeKey, TypeCell<M>>,

    /// Resolved constructor-expansion plans, keyed by `(function, parameter)`.
    /// Filled by [`crate::api::core::expand::apply`] before resolution; read
    /// by language adapters at the parameter-emission site. Empty unless the
    /// adapter declared expansions.
    pub(crate) expansion_plans:
        HashMap<(syn::Ident, syn::Ident), crate::api::core::expand::FoldPlan>,

    /// Resolved output-expansion plans, keyed by function ident. Filled by
    /// [`crate::api::core::unfold::apply`] before resolution; read by language
    /// adapters at the return-emission site. Empty unless the adapter declared
    /// deconstructors.
    pub(crate) unfold_plans: HashMap<syn::Ident, crate::api::core::unfold::UnfoldPlan>,

    /// Resolved **error**-position expansion plans, keyed by function ident: the
    /// decomposition of a fallible fn's `Result<_, E>` domain error `E` (from
    /// `.convert_error` / `.deconstruct_error`). Separate from
    /// [`Self::unfold_plans`] — a fn may have both an output and an error plan.
    pub(crate) error_plans: HashMap<syn::Ident, crate::api::core::unfold::UnfoldPlan>,

    /// Default decomposition of a **callback argument** type — the `T` of a
    /// declared fn's `impl Fn(T, …)` parameter — keyed by the bare arg type
    /// (type-level, fn-independent). Filled by
    /// [`crate::api::core::unfold::apply`] from the type's default
    /// deconstructor (`by_ref = false`: the trampoline owns the value); read by
    /// language adapters when emitting the callback trampoline. A type without
    /// a default deconstructor has no entry and is delivered whole.
    pub(crate) callback_arg_plans: HashMap<TypeKey, crate::api::core::unfold::UnfoldPlan>,

    /// The declaration-default decomposition per deconstructor declaration
    /// ([`crate::api::core::unfold::DeconId`]) — resolved once with
    /// normalized inputs, independent of using functions and processing
    /// order. The single source language adapters derive declaration-keyed
    /// signature artifacts (e.g. generated callback interfaces) from, so
    /// every function selecting the same declaration sees one signature by
    /// construction.
    pub(crate) decon_plans:
        HashMap<crate::api::core::unfold::DeconId, crate::api::core::unfold::DeconSpec>,
}

impl<M> Registry<M> {
    /// An empty registry: no model, no items, no types.
    ///
    /// **Not public.** A `Registry` is a projection of a [`Flat`], and one built
    /// this way projects nothing — [`Self::flat`] would hand a later stage an
    /// empty model that claims to be this registry's source. Outside this crate
    /// the entry point is [`Self::new`], which has a model behind it.
    pub(crate) fn empty() -> Self {
        Self {
            flat: crate::api::core::flat::Flat::default(),
            declared: Declared::default(),
            input_types: Default::default(),
            output_types: Default::default(),
            expansion_plans: HashMap::new(),
            unfold_plans: HashMap::new(),
            error_plans: HashMap::new(),
            callback_arg_plans: HashMap::new(),
            decon_plans: HashMap::new(),
        }
    }
}

/// Everything the caller declares about what a binding emits.
///
/// **The registry's construction input.** It used to be assembled by calling
/// twenty-one getters back into the adapter from inside `resolve`, which put
/// "configuring" and "using" in the same call — and that is what let a converter
/// read a half-built registry, which is what made `None` ambiguous between
/// *defer* and *cannot*. The caller fills this first; `resolve` then passes or
/// fails.
#[derive(Default)]
pub(crate) struct Declared {
    pub(crate) functions: HashSet<syn::Ident>,
    /// Signature-scanned but not emitted — see [`Prebindgen::helper_functions`].
    pub(crate) helper_functions: HashSet<syn::Ident>,
    pub(crate) accessors: HashSet<syn::Ident>,
    pub(crate) method_receivers: HashMap<syn::Ident, TypeKey>,
    pub(crate) types: HashSet<TypeKey>,
    /// Consts to scan and emit, or `None` when the adapter has no const
    /// declaration mechanism — then every captured const is re-emitted
    /// verbatim (see the const gate in [`crate::api::core::write`]).
    ///
    /// The two are identical for the *crossing set* — neither scans anything —
    /// so this would be a plain `HashSet` if scanning were all it drove. It is
    /// emission that needs the distinction, which is why the sentinel outlives
    /// the skip warnings it also used to gate.
    pub(crate) consts: Option<HashSet<syn::Ident>>,
    /// Crossings with no `#[prebindgen]` element behind them, each in the one
    /// direction it actually crosses — see [`Registry::cross`].
    pub(crate) crossings: Vec<(Direction, syn::Type)>,
    /// How composites cross in pieces — see [`Registry::decompose`].
    pub(crate) decompositions: Decompositions,
    /// Ordering edges no syntax shows — see [`Registry::depends`].
    pub(crate) edges: Vec<(Crossing, Crossing)>,
}

/// How a binding's composites cross **in pieces** instead of whole.
///
/// One value, pushed once through [`Registry::decompose`], in place of the five
/// separate hooks the registry used to call back for (`expansions`,
/// `deconstructors`, `value_struct_decons`, `sum_decons`,
/// `leaf_vec_fold_elements`). All five are implemented by one adapter and none
/// of them ever needed more than the model, which is what makes stating them up
/// front possible.
///
/// The fields are still the five declaration families, because unifying the
/// plan IRs behind them is its own problem (see issue #223) and pretending
/// otherwise here would only move the seam. What this settles is *when* they
/// are stated and *by whom*.
#[derive(Default)]
pub struct Decompositions {
    /// Parameter-side: values built on the Rust side from ingredients that
    /// cross separately.
    pub expansions: Option<crate::api::core::expand::Expansions>,
    /// Return/error-side: values delivered as leaves the far side reassembles.
    pub deconstructors: Option<crate::api::core::unfold::Deconstructors>,
    /// By-value struct decompositions whose leaves the adapter computed.
    pub value_structs: Vec<crate::api::core::unfold::ValueDecon>,
    /// The selector-carrying sibling: a tag plus one leaf group per
    /// alternative.
    pub sums: Vec<crate::api::core::unfold::SumDecon>,
    /// Element types of a `Vec<T>`/`&[T]` delivered element-by-element.
    pub leaf_vec_elements: Vec<syn::Type>,
    /// The whole-value crossings these decompositions make unnecessary.
    ///
    /// Stated **with** the decompositions rather than beside them: a type
    /// crosses only in pieces *because* something decomposes it, and once the
    /// plans are applied its own direct converter is genuinely not needed — for
    /// a type with no destination representation, not even resolvable.
    pub replaces: HashSet<TypeKey>,
}

#[cfg(test)]
mod tests;
