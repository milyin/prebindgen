//! Which type conversions a binding needs, and whether it has them all.
//!
//! # The boundary
//!
//! [`Flat`](prebindgen_flat::flat::Flat) describes the source Rust code. A binding
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
//! The adapter's business. It keeps the conversion itself; what it hands back
//! is an [`Answer`], naming the crossings this one delegates to. That is what
//! reachability walks, and it is all the registry needs to know.
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
//! | the model | [`Flat`](prebindgen_flat::flat::Flat) — what the source offers |
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
//! **Describe it, hand over the answers, read it.** Two types, because those
//! are two different things: a [`RegistryBuilder`] is still being described,
//! and a [`Registry`] is finished and answerable.
//!
//! ```text
//!   describe    builder(flat) · export · cross · decompose · depends
//!      ↓
//!   the demand  crossings()      → every crossing needing a conversion,
//!      ↓                           sorted so each type's inners come first
//!   the answers convert_with(f)  → one call per crossing, in that order
//!      ↓        conversions(map) → or hand over a map you built yourself
//!      ↓
//!   close       build()          → fails naming any reachable crossing with
//!      ↓                           no conversion
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
//! let mut builder = Registry::builder(flat)?;
//! for name in &self.exported    { builder = builder.export(name); }
//! for ty in &self.foreign_types { builder = builder.cross(Direction::Deconstruct, ty); }
//!
//! let registry = builder
//!     .decompose(self.decompositions())
//!     // `built` already holds everything this crossing composes from: that is
//!     // what sorted means.
//!     .convert_with(|crossing, built| self.convert(crossing, built))?
//!     .build()?;
//!
//! self.emit(&registry, out)   // read-only from here
//! ```
//!
//! Prefer to drive the walk yourself? `crossings()` hands over the same list in
//! the same order, and `conversions(map)` takes the result — the two compose,
//! and neither is a second mechanism.
//!
//! **Nothing here calls back into the generator** — not by trait hook, and not
//! by a `next_request`/`supply` pull loop either, which is the same protocol
//! with the arrow flipped. `convert_with` is not that: the walk finishes before
//! it returns, the closure is the caller's, and the builder chooses nothing
//! about when it runs. It is `crossings()` plus a `for` loop, written once.
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
//! A `None` is not itself a failure. The scan over-approximates deliberately —
//! every nested position, every declared struct in both directions — so whether
//! a gap matters is reachability from the exports, which `build` decides.
//!
//! The structure covers almost every dependency, because an `Option<T>`
//! visibly contains a `T`. What it cannot show is one a *declaration* creates —
//! a `convert!` chaining through a helper's parameter type, or a callback
//! argument delivered as plan leaves. Those are stated with `depends`, and
//! getting one wrong is not silent: the conversion that needed the missing one
//! cannot be built, and `build` names it.
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

use prebindgen::SourceLocation;
use prebindgen_flat::{flat::Origin, types_util::bare_path_ident};

use crate::prebindgen::Prebindgen;

mod cell;
pub(crate) use self::cell::TypeCell;
mod declare;
mod error;
mod model;
mod order;
mod run;
mod scan;
mod view;

/// The canonical type identity, which the source model owns — re-exported
/// here because the registry's tables are keyed by it.
pub use prebindgen_flat::flat::{TypeKey, TypeKeyParseError};

pub use self::{
    cell::{Answer, Direction},
    declare::RegistryBuilder,
    error::{DuplicateNameError, NotExpressibleEntry, ScanError, WriteRustError},
    view::{Building, Conversions, Crossing},
};

/// Single owner of everything parsed from the prebindgen source stream.
///
/// Carries no adapter metadata. It used to be generic over one, so a converter's
/// language-specific extras could be stored beside it and read back; an adapter
/// keeps its own conversions now, so both the storage and the parameter are
/// gone.
pub struct Registry {
    /// The parsed model these maps project. Held rather than discarded, so a
    /// later stage can ask it what a name means through the registry it already
    /// has — see [`Self::flat`].
    flat: prebindgen_flat::flat::Flat,
    /// What the binding declared, pushed in through `RegistryBuilder`'s
    /// `export` / `export_type` / `cross` / `reference` before its `build`.
    ///
    /// Stored rather than asked for: the registry never calls the generator to
    /// find out what to build. It is also read after resolution — `write`'s
    /// emission gate is "did the binding declare this item" — so it outlives
    /// the scan that consumes it.
    declared: Declared,
    /// Type tables, one per direction. Each scanned type gets a `TypeCell`
    /// holding what the key names, whether the binding asks for it directly, and
    /// the conversion once the generator supplies one.
    ///
    /// **Crate-internal.** Outside, a table is reached through
    /// [`Conversions::conversion`] — which is what makes direction a parameter
    /// rather than half of a field name, and what stops anyone observing a cell
    /// before `RegistryBuilder::build` has graded it.
    pub(crate) input_types: HashMap<TypeKey, TypeCell>,
    pub(crate) output_types: HashMap<TypeKey, TypeCell>,

    /// Resolved constructor-expansion plans, keyed by `(function, parameter)`.
    /// Filled by [`crate::expand::apply`] before resolution; read
    /// by language adapters at the parameter-emission site. Empty unless the
    /// adapter declared expansions.
    pub(crate) expansion_plans: HashMap<(syn::Ident, syn::Ident), crate::expand::FoldPlan>,
}

// Opaque — exists so `Result<Registry, _>::expect_err` works in tests, the way
// `Generation`'s did before the generators took ownership of the built object.
impl std::fmt::Debug for Registry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Registry(..)")
    }
}

impl Registry {
    /// An empty registry: no model, no items, no types.
    ///
    /// **Not public.** A `Registry` is a projection of a [`Flat`](prebindgen_flat::Flat), and one built
    /// this way projects nothing — [`Self::flat`] would hand a later stage an
    /// empty model that claims to be this registry's source. Outside this crate
    /// the entry point is [`Self::builder`], which has a model behind it.
    /// `Self::empty` for the test suite of an out-of-crate adapter.
    ///
    /// Gated on the non-default `testing` feature, so the "not public" rule
    /// above is unchanged for every ordinary build; a fixture that wants a
    /// registry projecting nothing is the one caller it was never aimed at.
    #[cfg(any(test, feature = "testing"))]
    pub fn empty_for_test() -> Self {
        Self::empty()
    }

    /// Whether `key` is registered in `dir`, and if so whether the binding
    /// asked for it directly (the cell's root flag).
    ///
    /// Test support, `testing`-gated: it answers the one question a fixture
    /// asks of a cell without handing out `TypeCell`, which is the
    /// registry's storage and no part of the public surface.
    #[cfg(any(test, feature = "testing"))]
    pub fn is_root_for_test(&self, dir: Direction, key: &TypeKey) -> Option<bool> {
        self.type_table(dir).get(key).map(|cell| cell.root)
    }

    /// Whether `key`'s cell in `dir` has been answered.
    ///
    /// Test support, `testing`-gated: a fixture that built the type itself
    /// holds only its identity, and this is the one question about a cell that
    /// does not need a reading.
    #[cfg(any(test, feature = "testing"))]
    pub fn has_entry_for_test(&self, dir: Direction, key: &TypeKey) -> Option<bool> {
        self.type_table(dir)
            .get(key)
            .map(|cell| cell.entry.is_some())
    }

    pub(crate) fn empty() -> Self {
        Self {
            flat: prebindgen_flat::flat::Flat::default(),
            declared: Declared::default(),
            input_types: Default::default(),
            output_types: Default::default(),
            expansion_plans: HashMap::new(),
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
    /// Exported types, each **with the spelling its declaration was written
    /// with**.
    ///
    /// Keyed by identity, because that is what a declaration is looked up by —
    /// and carrying the `syn::Type` anyway, because the scan needs real tokens
    /// for these: to `intern` a type that is not yet in any table, and to say
    /// whether the build script path-qualified it. Recovering those *from the
    /// key* was the wrong direction — a build script wrote a `syn::Type`, and
    /// the declaration simply discarded it (#291).
    pub(crate) types: HashMap<TypeKey, Origin<syn::Type>>,
    /// Consts to scan and emit, or `None` when the adapter has no const
    /// declaration mechanism — then every captured const is re-emitted
    /// verbatim (see the const gate in [`crate::write`]).
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
/// One value, pushed once through `RegistryBuilder::decompose`, in place of the
/// separate hooks the registry used to call back for. Everything here reads the
/// model alone, which is what makes stating it up front possible.
///
/// The output side is stated as its **effects** rather than as its
/// declarations: an adapter applies its own deconstructor declarations with
/// [`unfold::Unfolding`](crate::unfold::Unfolding), keeps the plans it gets,
/// and hands over the two things a registry needs from them — the output
/// registrations to replay, and the leaf readings a callback argument's
/// delivery depends on. The parameter side is still stated as declarations,
/// because `expand::apply` still mutates the registry directly.
#[derive(Default)]
pub struct Decompositions {
    /// Parameter-side: values built on the Rust side from ingredients that
    /// cross separately.
    pub expansions: Option<crate::expand::Expansions>,
    /// Output-side registrations the adapter's decompositions asked for, in the
    /// order they were asked — see
    /// [`Unfolding::requirements`](crate::unfold::Unfolding::requirements).
    pub requirements: Vec<crate::unfold::Requirement>,
    /// Per callback-argument type, the readings its decomposition delivers —
    /// see
    /// [`Unfolding::callback_arg_leaves`](crate::unfold::Unfolding::callback_arg_leaves).
    ///
    /// The ordering fact no syntax shows: each leaf's own conversion has to
    /// exist before the callback's can be built, and a leaf is named by a plan
    /// rather than by the argument's type.
    pub callback_arg_leaves: HashMap<TypeKey, Vec<prebindgen_flat::flat::TypeRef>>,
    /// The whole-value crossings these decompositions make unnecessary.
    ///
    /// Stated **with** the decompositions rather than beside them: a type
    /// crosses only in pieces *because* something decomposes it, and once the
    /// plans are applied its own direct converter is genuinely not needed — for
    /// a type with no destination representation, not even resolvable.
    ///
    /// Carries each declaration's own spelling for the same reason the declared
    /// types do — these are build-script-authored types the scan diagnoses
    /// before anything has classified them.
    pub replaces: HashMap<TypeKey, Origin<syn::Type>>,
}

#[cfg(test)]
mod tests;
