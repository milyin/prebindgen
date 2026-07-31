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
//! Conversions are **directional**: [`Registry::input_types`] and
//! [`Registry::output_types`] are separate. `&str` inbound is a `jstring` to decode,
//! outbound a `jstring` to allocate, and one direction may be convertible while
//! the other is not. A callback flips it — `impl Fn(Sample)` is an *input* whose
//! argument crosses *outbound*.
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

use std::{
    collections::{HashMap, HashSet},
    fmt,
};

use quote::ToTokens;

use crate::{
    api::core::{
        niches::Niches,
        prebindgen::{Prebindgen, Stage},
        types_util::bare_path_ident,
    },
    SourceLocation,
};

/// Canonical type-shape key: identity is the token string of the
/// **normalized** type ([`crate::api::core::types_util::normalize_type`] —
/// group/paren unwrap, `crate::`/`self::` and std-prelude path reduction;
/// the complete equivalence rule set is documented there). The normalized
/// parsed form is kept alongside the string, so [`Self::to_type`] is an
/// infallible clone — no core invariant depends on serialize-then-reparse
/// round trips (issue #95).
#[derive(Clone)]
pub struct TypeKey {
    /// Canonical token string — the identity `Eq`/`Hash` compare.
    canon: std::rc::Rc<str>,
    /// The normalized parsed form the string was rendered from.
    ty: std::rc::Rc<syn::Type>,
}

impl PartialEq for TypeKey {
    fn eq(&self, other: &Self) -> bool {
        self.canon == other.canon
    }
}
impl Eq for TypeKey {}
impl std::hash::Hash for TypeKey {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.canon.hash(state)
    }
}
impl PartialOrd for TypeKey {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for TypeKey {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.canon.cmp(&other.canon)
    }
}
// Keep the historical single-field tuple rendering (`TypeKey("Vec < u8 >")`)
// — error text and test expectations format keys through it.
impl fmt::Debug for TypeKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("TypeKey").field(&&*self.canon).finish()
    }
}

/// Structured failure of [`TypeKey::parse`]: the offending input plus the
/// underlying `syn` parse error.
#[derive(Debug)]
pub struct TypeKeyParseError {
    pub input: String,
    pub error: syn::Error,
}

impl fmt::Display for TypeKeyParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid type `{}`: {}", self.input, self.error)
    }
}

impl std::error::Error for TypeKeyParseError {}

impl TypeKey {
    /// Build a key by parsing the input as a type and normalizing.
    pub fn parse(s: &str) -> Result<Self, TypeKeyParseError> {
        let ty: syn::Type = syn::parse_str(s).map_err(|error| TypeKeyParseError {
            input: s.to_string(),
            error,
        })?;
        Ok(Self::from_type(&ty))
    }

    /// Build a key directly from a `syn::Type` (normalizing a clone; the
    /// input is not modified).
    pub fn from_type(ty: &syn::Type) -> Self {
        // Off the shared reduction, so this key and the model's type index
        // cannot drift apart about what a type is called.
        let t = crate::api::core::types_util::canonical_type(ty);
        Self {
            canon: t.to_token_stream().to_string().into(),
            ty: std::rc::Rc::new(t),
        }
    }

    /// Build a key for a bare item ident — infallible by construction (an
    /// ident IS a single-segment path type; nothing to parse or normalize).
    pub fn from_ident(ident: &syn::Ident) -> Self {
        Self::from_type(&crate::api::core::types_util::type_from_ident(ident))
    }

    /// The canonical string form.
    pub fn as_str(&self) -> &str {
        &self.canon
    }

    /// The normalized parsed form. Infallible — a clone of the stored type,
    /// never a reparse.
    pub fn to_type(&self) -> syn::Type {
        (*self.ty).clone()
    }
}

impl fmt::Display for TypeKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.canon)
    }
}

/// What a type-table key names.
///
/// Two populations, and saying which is which is what keeps one origin per cell:
/// a type the flat API contains **is** a [`TypeRef`](crate::api::core::flat::TypeRef), reused whole, so its
/// classification and its source location are already there.
#[derive(Clone, Debug)]
pub enum TypeSubject {
    /// A type the flat API contains — the frontend's own reading, unmodified.
    Source(crate::api::core::flat::TypeRef),
    /// A type only the binding authored: a declared wire type with no
    /// `#[prebindgen]` item behind it, an [`unfold`](crate::api::core::unfold)
    /// leaf. It has no reading and no source location — a fact about it, rather
    /// than information that went missing.
    Adapter(syn::Type),
}

impl TypeSubject {
    /// Where the source wrote this type, or `None` when no source did.
    pub fn location(&self) -> Option<&SourceLocation> {
        match self {
            // Having a reading and having a reportable position are different
            // facts: a binding-local fn's types are lowered — so they have
            // readings — against no file at all. Reporting `:0:0` would invent a
            // position; `None` says what is true.
            TypeSubject::Source(t) => Some(&*t.origin.location).filter(|l| l.has_position()),
            TypeSubject::Adapter(_) => None,
        }
    }

    /// The frontend's classification, or `None` for an adapter-authored type.
    pub fn kind(&self) -> Option<&crate::api::core::flat::TypeKind> {
        match self {
            TypeSubject::Source(t) => Some(&t.kind),
            TypeSubject::Adapter(_) => None,
        }
    }

    /// The type as Rust must spell it, either way.
    pub fn syntax(&self) -> &syn::Type {
        match self {
            TypeSubject::Source(t) => &t.origin.syntax,
            TypeSubject::Adapter(ty) => ty,
        }
    }
}

/// One type-table cell: what the key names, and the adapter's answer for it.
pub struct TypeCell<M = ()> {
    /// The type itself, as the frontend reads it when it can.
    pub subject: TypeSubject,
    /// The binding asks for this cell **directly** — a declared fn's signature, a
    /// declared type, an `unfold` leaf — as opposed to reaching it through some
    /// converter's [`TypeEntry::subs`].
    ///
    /// A scan fact. Whether a converter is *needed* here is reachability from
    /// these roots, which [`crate::api::core::resolve`] derives rather than
    /// stores: the scan deliberately over-approximates the table (every nested
    /// position, every struct in both directions), so the roots are what say
    /// which of it has to work.
    pub root: bool,
    /// The adapter's converter, once resolved.
    pub entry: Option<TypeEntry<M>>,
}

/// Per-cell registry entry.
#[derive(Clone)]
pub struct TypeEntry<M = ()> {
    /// Wire/destination type — the form the value takes on the wire as
    /// chosen by the adapter (e.g. an `i64` handle for a JNI adapter, or
    /// a `*const T` raw pointer for a C adapter). Other converters that
    /// ask "what's the wire form of this rust type?" read this.
    pub destination: syn::Type,
    /// Complete generated function for the **wire-facing** stage of the
    /// converter (signature, body, attributes, lifetimes). The adapter
    /// owns the shape. Callers compute this stage's name via
    /// `function.sig.ident`.
    pub function: syn::ItemFn,
    /// **Rust-side** stages that compose with [`Self::function`] to form
    /// the full chain — copied verbatim from the resolving
    /// [`crate::api::core::prebindgen::ConverterImpl::pre_stages`]. See
    /// that field's docs for the chain-order semantics.
    pub pre_stages: Vec<Stage<M>>,
    /// Inner types whose function delegates to their converters. Empty for
    /// terminal converters; populated by wrapper converters. Used by the
    /// post-resolution propagation pass.
    pub subs: Vec<TypeKey>,
    /// Wire bit-patterns this converter never produces / always rejects.
    /// Wrappers (`Option<_>`, sum-typed enums) carve from this set for
    /// their own discriminants. See [`Niches`] for the cascade model.
    pub niches: Niches,
    /// Adapter-specific extras carried in by the
    /// [`crate::api::core::prebindgen::ConverterImpl`] that filled this
    /// slot. Emitter code reads this directly — the registry is the
    /// single source of truth for cross-language facts (C header names,
    /// JVM class names, etc.). Defaults to `()` for adapters that don't
    /// need any.
    pub metadata: M,
}

impl<M> TypeEntry<M> {
    /// Identifier of the wire-facing converter function.
    pub fn converter_ident(&self) -> &syn::Ident {
        &self.function.sig.ident
    }

    /// Wire/destination type carried by this converter on success.
    pub fn wire_type(&self) -> &syn::Type {
        &self.destination
    }

    /// Rust-side stages in input execution order, after the wire-facing
    /// converter has decoded the wire value.
    pub fn input_stage_order(&self) -> impl Iterator<Item = (usize, &Stage<M>)> {
        self.pre_stages.iter().enumerate().rev()
    }

    /// Rust-side stages in output execution order, before the wire-facing
    /// converter encodes the final wire value.
    pub fn output_stage_order(&self) -> impl Iterator<Item = (usize, &Stage<M>)> {
        self.pre_stages.iter().enumerate()
    }

    /// Immediate converter dependencies recorded by the adapter when this entry
    /// resolved.
    pub fn dependency_keys(&self) -> &[TypeKey] {
        &self.subs
    }
}

/// Direction of a converter pair.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum Direction {
    /// Wire → Rust.
    Input,
    /// Rust → Wire.
    Output,
}

impl Direction {
    pub fn flip(self) -> Self {
        match self {
            Direction::Input => Direction::Output,
            Direction::Output => Direction::Input,
        }
    }
}

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
    /// the resolved [`TypeEntry`] once the structural resolver fills it.
    pub input_types: HashMap<TypeKey, TypeCell<M>>,
    pub output_types: HashMap<TypeKey, TypeCell<M>>,

    /// Resolved constructor-expansion plans, keyed by `(function, parameter)`.
    /// Filled by [`crate::api::core::expand::apply`] before resolution; read
    /// by language adapters at the parameter-emission site. Empty unless the
    /// adapter declared expansions.
    pub expansion_plans: HashMap<(syn::Ident, syn::Ident), crate::api::core::expand::FoldPlan>,

    /// Resolved output-expansion plans, keyed by function ident. Filled by
    /// [`crate::api::core::unfold::apply`] before resolution; read by language
    /// adapters at the return-emission site. Empty unless the adapter declared
    /// deconstructors.
    pub unfold_plans: HashMap<syn::Ident, crate::api::core::unfold::UnfoldPlan>,

    /// Resolved **error**-position expansion plans, keyed by function ident: the
    /// decomposition of a fallible fn's `Result<_, E>` domain error `E` (from
    /// `.convert_error` / `.deconstruct_error`). Separate from
    /// [`Self::unfold_plans`] — a fn may have both an output and an error plan.
    pub error_plans: HashMap<syn::Ident, crate::api::core::unfold::UnfoldPlan>,

    /// Default decomposition of a **callback argument** type — the `T` of a
    /// declared fn's `impl Fn(T, …)` parameter — keyed by the bare arg type
    /// (type-level, fn-independent). Filled by
    /// [`crate::api::core::unfold::apply`] from the type's default
    /// deconstructor (`by_ref = false`: the trampoline owns the value); read by
    /// language adapters when emitting the callback trampoline. A type without
    /// a default deconstructor has no entry and is delivered whole.
    pub callback_arg_plans: HashMap<TypeKey, crate::api::core::unfold::UnfoldPlan>,

    /// The declaration-default decomposition per deconstructor declaration
    /// ([`crate::api::core::unfold::DeconId`]) — resolved once with
    /// normalized inputs, independent of using functions and processing
    /// order. The single source language adapters derive declaration-keyed
    /// signature artifacts (e.g. generated callback interfaces) from, so
    /// every function selecting the same declaration sees one signature by
    /// construction.
    pub decon_plans:
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

impl From<crate::api::core::flat::ParseError> for ScanError {
    fn from(e: crate::api::core::flat::ParseError) -> Self {
        match e {
            crate::api::core::flat::ParseError::DuplicateName(d) => {
                ScanError::DuplicateName(Box::new(DuplicateNameError {
                    name: d.name,
                    first: d.first,
                    second: d.second,
                    first_crate: d.first_crate,
                    second_crate: d.second_crate,
                }))
            }
        }
    }
}

/// One item of a [`ScanError::NotExpressible`] report.
#[derive(Debug)]
pub struct NotExpressibleEntry {
    /// The item's name, or `None` for an item kind that has none.
    pub name: Option<syn::Ident>,
    /// Rendered [`ItemError`](crate::core::flat::ItemError) — the frontend's own
    /// message, so one authority produces it.
    pub reason: String,
    pub location: SourceLocation,
}

/// Payload of [`ScanError::DuplicateName`], boxed to keep the error enum
/// small (`clippy::result_large_err`).
#[derive(Debug)]
pub struct DuplicateNameError {
    pub name: syn::Ident,
    pub first: SourceLocation,
    pub second: SourceLocation,
    /// Origin crates of the colliding items, when known (multi-source
    /// ingestion via several `Flat::builder().source(..)` feeders) — the `SourceLocation`
    /// file paths are crate-relative, so with several sources they alone
    /// may not identify the colliding crates.
    pub first_crate: Option<String>,
    pub second_crate: Option<String>,
}

/// Errors surfaced by the scan phase.
#[derive(Debug)]
pub enum ScanError {
    DuplicateName(Box<DuplicateNameError>),
    /// Items the flat language cannot express, all of them at once.
    ///
    /// The message for each comes from
    /// [`ItemError`](crate::core::flat::ItemError), so one authority produces it.
    /// This replaces the per-item guards the registry used to duplicate — a `self`
    /// receiver, a non-ident parameter pattern, a disallowed `impl Trait` — which
    /// the frontend now catches with a richer diagnosis (it names the parameter).
    NotExpressible {
        entries: Vec<NotExpressibleEntry>,
    },
    /// An adapter-invariant check failed — see [`Prebindgen::validate`].
    /// The message is adapter-authored and printed verbatim.
    AdapterInvariant {
        message: String,
    },
    /// Explicitly declared items (functions, helper functions, constants)
    /// that match no indexed `#[prebindgen]` item. A declaration is a
    /// statement of intent — its target being absent is always a bug (a
    /// typo in build.rs, or the item was renamed/removed in the source
    /// crate), so this is a hard error, unlike the soft warnings for stale
    /// *ignore* entries. All missing names are collected before failing.
    DeclaredNotFound {
        entries: Vec<(&'static str, String)>,
    },
    /// Declared type keys that qualify a source item with its crate path
    /// (`ptr_class!(myflat::Foo)` where `myflat` is a chained source crate).
    /// Source items live in one flat namespace and are keyed by their bare
    /// name — the qualified spelling can never match a captured signature,
    /// so it is a hard error with a fix-it instead of a silent miss (issue
    /// #95). All offenders are collected before failing.
    QualifiedDeclaredTypes {
        /// `(qualified spelling, bare fix-it name)` pairs.
        entries: Vec<(String, String)>,
    },
}

impl fmt::Display for ScanError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ScanError::DuplicateName(e) => {
                let in_crate = |c: &Option<String>| match c {
                    Some(c) => format!(" in crate `{c}`"),
                    None => String::new(),
                };
                write!(
                    f,
                    "duplicate prebindgen name `{}`: first{} at {}, second{} at {} — prebindgen \
                     items live in one flat namespace across all sources; rename one of them",
                    e.name,
                    in_crate(&e.first_crate),
                    e.first,
                    in_crate(&e.second_crate),
                    e.second
                )
            }
            ScanError::NotExpressible { entries } => {
                write!(
                    f,
                    "{} `#[prebindgen]` item(s) the flat language cannot express:",
                    entries.len()
                )?;
                for e in entries {
                    // The crate, because a captured path is crate-relative: with
                    // several sources, two offenders both read `src/lib.rs:..`
                    // and the location alone says nothing about which one to fix.
                    // Same reason the duplicate-name diagnostic carries it.
                    let in_crate = match &e.location.crate_name {
                        Some(c) => format!(" in crate `{c}`"),
                        None => String::new(),
                    };
                    match &e.name {
                        Some(name) => {
                            write!(f, "\n  {}{in_crate}: {name} {}", e.location, e.reason)?
                        }
                        None => write!(f, "\n  {}{in_crate}: {}", e.location, e.reason)?,
                    }
                }
                Ok(())
            }
            ScanError::AdapterInvariant { message } => write!(f, "{}", message),
            ScanError::DeclaredNotFound { entries } => {
                writeln!(
                    f,
                    "{} declared item(s) not found among #[prebindgen] items:",
                    entries.len()
                )?;
                for (kind, name) in entries {
                    writeln!(f, "  - {kind} `{name}`")?;
                }
                write!(
                    f,
                    "a declaration names an item that does not exist — typo in build.rs, \
                     or renamed/removed in the source crate?"
                )
            }
            ScanError::QualifiedDeclaredTypes { entries } => {
                writeln!(
                    f,
                    "{} declared type(s) qualify a source item with its crate path:",
                    entries.len()
                )?;
                for (spelled, bare) in entries {
                    writeln!(f, "  - `{spelled}` — declare it as `{bare}`")?;
                }
                write!(
                    f,
                    "source items live in one flat namespace keyed by their bare name; \
                     a crate-qualified spelling never matches captured signatures"
                )
            }
        }
    }
}

impl std::error::Error for ScanError {}

/// Combined error surfaced by [`Registry::resolve`] / [`Generation::write_rust`].
#[derive(Debug)]
pub enum WriteRustError {
    Scan(ScanError),
    Expand(crate::api::core::expand::ExpandError),
    Unfold(crate::api::core::unfold::UnfoldError),
    Resolve(crate::api::core::resolve::ResolveError),
    Write(crate::api::core::write::WriteError),
}

impl fmt::Display for WriteRustError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WriteRustError::Scan(e) => write!(f, "{}", e),
            WriteRustError::Expand(e) => write!(f, "{}", e),
            WriteRustError::Unfold(e) => write!(f, "{}", e),
            WriteRustError::Resolve(e) => write!(f, "{}", e),
            WriteRustError::Write(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for WriteRustError {}

impl From<ScanError> for WriteRustError {
    fn from(e: ScanError) -> Self {
        WriteRustError::Scan(e)
    }
}

impl From<crate::api::core::expand::ExpandError> for WriteRustError {
    fn from(e: crate::api::core::expand::ExpandError) -> Self {
        WriteRustError::Expand(e)
    }
}

impl From<crate::api::core::unfold::UnfoldError> for WriteRustError {
    fn from(e: crate::api::core::unfold::UnfoldError) -> Self {
        WriteRustError::Unfold(e)
    }
}

impl From<crate::api::core::resolve::ResolveError> for WriteRustError {
    fn from(e: crate::api::core::resolve::ResolveError) -> Self {
        WriteRustError::Resolve(e)
    }
}

impl From<crate::api::core::write::WriteError> for WriteRustError {
    fn from(e: crate::api::core::write::WriteError) -> Self {
        WriteRustError::Write(e)
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

impl<M> Registry<M> {
    /// A registry over this model.
    ///
    /// A `Flat` is what a registry projects, and reading captured prebindgen
    /// output into one is [`FlatBuilder`](crate::core::flat::FlatBuilder)'s job
    /// — so a build script says where items come from at the layer that owns
    /// the question, and there is one such layer rather than two:
    ///
    /// ```
    /// # prebindgen::Source::init_doctest_simulate();
    /// use prebindgen::core::{Flat, Registry};
    ///
    /// let flat = Flat::builder().source("source_ffi").build()?;
    /// // Annotated only because nothing here resolves: in a build script `M` is
    /// // fixed by the adapter passed to `resolve`, so no call site names it.
    /// let registry: Registry<()> = Registry::new(flat)?;
    /// assert!(registry.flat().function("test_function").is_some());
    /// # Ok::<_, Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// Several sources compose there too, including one this crate renames:
    ///
    /// ```ignore
    /// let flat = Flat::builder()
    ///     .source(flat_crate::PREBINDGEN_OUT_DIR)
    ///     .source_named(helpers::PREBINDGEN_OUT_DIR, "helpers")
    ///     .build()?;
    /// ```
    ///
    /// **Fails on anything the language cannot express** — a `self` receiver, an
    /// `async fn`, a generic binder, a type form outside the grammar, or a
    /// reference to a type the flat API does not declare. All of them at once, so
    /// a source crate that needs migrating sees one list instead of one rebuild
    /// per item. This is independent of what any binding declares: an
    /// inexpressible item is a hard error whether or not it is ever named.
    pub fn new(flat: crate::api::core::flat::Flat) -> Result<Self, ScanError> {
        let entries: Vec<NotExpressibleEntry> = flat
            .unsupported()
            .map(|u| NotExpressibleEntry {
                name: u.name.clone(),
                reason: u.error.to_string(),
                location: (*u.origin.location).clone(),
            })
            .collect();
        if !entries.is_empty() {
            return Err(ScanError::NotExpressible { entries });
        }

        let mut registry = Registry::empty();
        registry.flat = flat;
        Ok(registry)
    }

    // ── configure: what this binding builds ───────────────────────────
    //
    // Pushed in by the generator before `resolve`. The registry never asks —
    // it records, then derives the crossing set from what it was given.

    /// An element this binding **exports**.
    ///
    /// The model says how to derive its crossings, so the caller does not: a
    /// function's signature gives its parameters (in) and its return (out); a
    /// const gives its value type (out). A name matching no element is an
    /// error, reported with every other missing name at once by `resolve`
    /// rather than here — a build script with three typos should learn all
    /// three in one build.
    pub fn export(&mut self, name: &syn::Ident) {
        self.declared.functions.insert(name.clone());
    }

    /// A const this binding exports.
    ///
    /// Separate from [`Self::export`] only because *having a const mechanism at
    /// all* is itself a fact: a binding that never calls this re-emits every
    /// captured const verbatim, while one that calls it emits exactly what it
    /// names. See [`Self::declares_consts`].
    pub fn export_const(&mut self, name: &syn::Ident) {
        self.declared
            .consts
            .get_or_insert_with(HashSet::new)
            .insert(name.clone());
    }

    /// Declare that this binding has a const mechanism, even if it exports no
    /// consts. Without it every captured const is re-emitted verbatim.
    pub fn declares_consts(&mut self) {
        self.declared.consts.get_or_insert_with(HashSet::new);
    }

    /// A type this binding **exports**: it crosses in both directions, and its
    /// body — a struct's fields, an enum's payloads — is scanned too.
    pub fn export_type(&mut self, key: TypeKey) {
        self.declared.types.insert(key);
    }

    /// A type that **crosses** in one direction without being exported.
    ///
    /// The escape hatch for a crossing no signature can yield: a re-exported
    /// foreign type named by a class declaration, or the value type of a
    /// constant the binding synthesizes. Direction is explicit because these
    /// are genuinely one-sided — which is what stops an output-only crossing
    /// from silently lacking its input twin, the asymmetry the old
    /// `required_output_types` had.
    pub fn cross(&mut self, dir: Direction, ty: &syn::Type) {
        self.declared.crossings.push((dir, ty.clone()));
    }

    /// A function this binding **references but never emits** — a helper whose
    /// name appears in a declaration. Its absence is an error; its presence
    /// emits nothing.
    pub fn reference(&mut self, name: &syn::Ident) {
        self.declared.helper_functions.insert(name.clone());
    }

    /// A function the **binding crate itself** defines, with the module path
    /// generated calls should qualify it by.
    ///
    /// There is no `#[prebindgen]` item behind it, so this is the one input
    /// that adds to the model rather than selecting from it: only the
    /// signature is read, never the body. A name colliding with a captured
    /// item is an error — the generated call would resolve the wrong function.
    pub fn local_function(
        &mut self,
        item_fn: syn::ItemFn,
        origin: String,
    ) -> Result<(), ScanError> {
        let ident = item_fn.sig.ident.clone();
        // Written by hand in a build script, so the grammar is checked here or
        // nowhere: a dropped `self` receiver would surface as an arity mismatch
        // out of rustc on generated code, which is the wrong end of the pipeline
        // to learn about a build.rs typo.
        let lowered =
            self.flat
                .lower_signature(&item_fn)
                .map_err(|error| ScanError::AdapterInvariant {
                    message: format!("binding-local fn `{ident}`: {error}"),
                })?;
        if self.flat.element(&ident).is_some() {
            return Err(ScanError::AdapterInvariant {
                message: format!(
                    "binding-local fn `{ident}` collides with a `#[prebindgen]` item — \
                     the generated call would resolve the wrong fn; rename the \
                     binding-local fn"
                ),
            });
        }
        self.flat.add_local_function(lowered, origin);
        Ok(())
    }

    /// A function a decomposition reaches through rather than emits — excluded
    /// from constructor composition, and the only functions a decomposer record
    /// may name.
    ///
    /// Rides here until decompositions carry their own shape (step 2 of #251);
    /// it is a property of the decomposition, not of the binding.
    pub fn accessor(&mut self, name: &syn::Ident) {
        self.declared.accessors.insert(name.clone());
    }

    /// The receiver type of a function emitted as a method. Same temporary
    /// home as [`Self::accessor`].
    pub fn method_receiver(&mut self, name: &syn::Ident, receiver: TypeKey) {
        self.declared
            .method_receivers
            .insert(name.clone(), receiver);
    }

    /// How this binding's composites cross **in pieces** instead of whole.
    ///
    /// Stated once, before [`Self::resolve`]. Replaces five separate callbacks
    /// the registry used to make into the generator; see [`Decompositions`].
    pub fn decompose(&mut self, d: Decompositions) {
        self.declared.decompositions = d;
    }

    /// What the binding declared — read by the emitter's gate.
    pub(crate) fn declared(&self) -> &Declared {
        &self.declared
    }

    /// The parsed model this registry projects.
    pub fn flat(&self) -> &crate::api::core::flat::Flat {
        &self.flat
    }

    /// Every **named** item the model holds — functions, structs, either enum
    /// shape, consts — regardless of whether the stream carried an origin stamp.
    ///
    /// Lives here so an adapter that needs "anything the source crate defines"
    /// does not enumerate element kinds itself: a new kind is taught here once
    /// instead of drifting in each adapter. An **alias is deliberately absent**
    /// — see the arm below — and callers are expected to pair this with
    /// `origin_module(..).unwrap_or_else(default_module)`.
    pub fn named_item_idents(&self) -> impl Iterator<Item = &syn::Ident> {
        use crate::api::core::flat::{Element, Type};
        self.flat.elements().filter_map(|e| match e {
            // An `Extern` names a type without declaring a body, and is
            // deliberately absent: its caller decides which names to qualify in
            // generated Rust, and qualifying an alias would move that output.
            Element::Type(Type::Extern(_)) => None,
            Element::Function(_) | Element::Type(_) | Element::Constant(_) => e.name(),
            Element::Guard(_) | Element::Unsupported(_) => None,
        })
    }

    /// Whether the source declares a type under this name — **including an
    /// alias**.
    ///
    /// An alias counts because `#[prebindgen] pub type Handle = ..` *is* a
    /// declaration of that name: it can be declared bare by an adapter (landing
    /// in the no-indexed-body branch below, which is what
    /// `ptr_class(ZKeyExpr<'static>)` relies on), so a diagnostic that says
    /// "no such captured item" would be false.
    fn declares_type(&self, ident: &syn::Ident) -> bool {
        self.flat.declared_type(ident).is_some()
    }

    /// The origin crate's **module path** for an item, read off the element's
    /// own [`SourceLocation`] stamp, or `None` when unknown — callers then fall
    /// back to [`Self::default_module`].
    pub fn origin_module(&self, ident: &syn::Ident) -> Option<syn::Path> {
        // Off the element's own location, which covers both populations: a
        // captured item stamped at capture time, and a binding-local fn stamped
        // by `add_local_function`.
        let crate_name = self.flat.element(&ident)?.location().crate_name.as_ref()?;
        let module = crate_name.replace('-', "_");
        syn::parse_str(&module).ok()
    }

    /// The default module for references with no recorded origin: the
    /// first-seen item origin. `None` for an origin-less item-level
    /// registry (adapters then fall back to `crate`). To change a module
    /// name, override it at the source — a stream's origin stamps
    /// (`Source::builder(dir).crate_name("myflat")`) — never here: a
    /// registry-level override could only fix ONE module, which is
    /// incomplete with chained multi-source streams.
    pub fn default_module(&self) -> Option<syn::Path> {
        self.flat
            .source_modules()
            .first()
            .and_then(|m| syn::parse_str(m).ok())
    }

    /// Module paths of every ingested source, ingestion order — e.g. for a
    /// glob import that must see all sources' items.
    pub fn all_source_modules(&self) -> Vec<syn::Path> {
        self.flat
            .source_modules()
            .iter()
            .filter_map(|m| syn::parse_str(m).ok())
            .collect()
    }

    /// Scan the signature/body of every item declared by the adapter.
    ///
    /// * For each ident in `adapter.declared_functions()` ∩ indexed functions,
    ///   call `scan_fn_signature` so parameter and return types
    ///   are registered as required.
    /// * For each `TypeKey` in `adapter.declared_types()`, mark the key as
    ///   required in both directions; if the key resolves to an indexed
    ///   struct/enum, also scan its body so field types are registered
    ///   (still `required: false` — propagation later promotes them
    ///   through `subs`).
    /// * Idents / types returned by `adapter.ignored_functions()` /
    ///   `adapter.ignored_types()` are treated as intentional skips: they are
    ///   neither scanned nor emitted, but they do suppress the "skipping
    ///   undeclared" warnings.
    ///
    /// Declared items that don't match any indexed body get a build
    /// warning (likely a typo in the build script). Indexed items that
    /// were neither declared nor ignored also get a `cargo:warning=` skip
    /// line so the user sees the remaining unexpected skips per build.
    /// Scan everything pushed in through the configure methods.
    ///
    /// Takes no adapter: what to build was stated by the caller, not asked for.
    pub fn scan_declared(&mut self) -> Result<(), ScanError> {
        let declared = std::mem::take(&mut self.declared);
        let out = self.scan_declared_items(&declared);
        self.declared = declared;
        out
    }

    fn scan_declared_items(&mut self, declared: &Declared) -> Result<(), ScanError> {
        // Source-qualified declared types are a hard error (issue #95). The
        // key's own normalization already reduced `crate::`/`self::` and std
        // prelude spellings, so a remaining multi-segment declared path
        // either qualifies a SOURCE item with its crate name (can never
        // match — the flat namespace keys are bare) or names a genuinely
        // foreign type (supported verbatim; warned about below only when it
        // shadows a captured item's name — the likely-mistake heuristic).
        let mut qualified: Vec<(String, String)> = Vec::new();
        let mut probed: HashSet<&TypeKey> = HashSet::new();
        for key in declared
            .types
            .iter()
            .chain(declared.decompositions.replaces.iter())
        {
            if !probed.insert(key) {
                continue;
            }
            let ty = key.to_type();
            // Peel one reference level; the qualified head only appears on
            // path types.
            let inner = match &ty {
                syn::Type::Reference(r) => &*r.elem,
                other => other,
            };
            let syn::Type::Path(tp) = inner else { continue };
            if tp.qself.is_some() || tp.path.segments.len() < 2 {
                continue;
            }
            let head = tp
                .path
                .segments
                .first()
                .expect("len checked")
                .ident
                .to_string();
            let last = tp.path.segments.last().expect("len checked");
            if self.flat.source_modules().contains(&head) {
                qualified.push((key.to_string(), last.to_token_stream().to_string()));
            } else if self.declares_type(&last.ident) {
                println!(
                    "cargo:warning=prebindgen: declared type `{}` is path-qualified, but a \
                     captured #[prebindgen] item `{}` exists — if you meant the source item, \
                     declare it by its bare name",
                    key, last.ident
                );
            }
        }
        if !qualified.is_empty() {
            qualified.sort();
            return Err(ScanError::QualifiedDeclaredTypes { entries: qualified });
        }

        // Declared-but-missing items are collected across all three loops and
        // reported together as one hard error (see
        // [`ScanError::DeclaredNotFound`]).
        let mut missing: Vec<(&'static str, String)> = Vec::new();

        // Scan declared functions.
        for ident in &declared.functions {
            if let Some(item_fn) = self.flat.function(&ident).map(|f| f.origin.syntax.clone()) {
                self.scan_fn_signature(&item_fn)?;
            } else {
                missing.push(("function", ident.to_string()));
            }
        }

        // Helper functions: never emitted, no blanket signature scan (the
        // adapter registers the specific requirements via
        // `extra_required_types`) — but they are referenced by name from
        // adapter declarations, so a missing one is a hard error.
        for ident in &declared.helper_functions {
            if self.flat.function(&ident).is_none() {
                missing.push(("helper function", ident.to_string()));
            }
        }

        // Scan declared consts: a const is a nullary source of its type, so
        // the type is required in the output direction only.
        for ident in declared.consts.iter().flatten() {
            if let Some(item_const) = self.flat.constant(&ident).map(|c| c.origin.syntax.clone()) {
                self.ensure_entry(Direction::Output, &item_const.ty, true);
            } else {
                missing.push(("constant", ident.to_string()));
            }
        }

        if !missing.is_empty() {
            missing.sort();
            return Err(ScanError::DeclaredNotFound { entries: missing });
        }

        // Declared crossings with no element behind them (a foreign class type,
        // a synthesized constant's value type), each in its own direction.
        for (dir, ty) in &declared.crossings {
            self.ensure_entry(*dir, ty, true);
        }

        // Scan declared types.
        for key in &declared.types {
            let ty = key.to_type();
            let mut matched = false;
            if let Some(ident) = bare_path_ident(&ty) {
                if let Some(s) = self
                    .flat
                    .struct_type(&ident)
                    .map(|s| s.origin.syntax.clone())
                {
                    self.scan_struct(&s)?;
                    self.ensure_entry(Direction::Input, &ty, true);
                    self.ensure_entry(Direction::Output, &ty, true);
                    matched = true;
                } else if let Some(e) = self.flat.enum_item(&ident).cloned() {
                    self.scan_enum(&e)?;
                    self.ensure_entry(Direction::Input, &ty, true);
                    self.ensure_entry(Direction::Output, &ty, true);
                    matched = true;
                }
            }
            if !matched {
                // Declared type without an indexed body (e.g.
                // `ptr_class(ZKeyExpr<'static>)` on a re-exported
                // foreign type). Still mark required so the resolver
                // tries to produce a converter for it.
                self.ensure_entry(Direction::Input, &ty, true);
                self.ensure_entry(Direction::Output, &ty, true);
            }
        }

        Ok(())
    }

    /// Direction-indexed read access to the type-resolution tables.
    pub(crate) fn type_table(&self, dir: Direction) -> &HashMap<TypeKey, TypeCell<M>> {
        match dir {
            Direction::Input => &self.input_types,
            Direction::Output => &self.output_types,
        }
    }

    /// Direction-indexed mutable access to the type-resolution tables.
    pub(crate) fn type_table_mut(&mut self, dir: Direction) -> &mut HashMap<TypeKey, TypeCell<M>> {
        match dir {
            Direction::Input => &mut self.input_types,
            Direction::Output => &mut self.output_types,
        }
    }

    /// Look up the resolved input entry for `ty`, returning `None` if it
    /// was never registered or is still unresolved. The returned entry's
    /// `function.sig.ident` is the converter's call name; `destination` is
    /// its wire form.
    pub fn input_entry(&self, ty: &syn::Type) -> Option<&TypeEntry<M>> {
        let key = TypeKey::from_type(ty);
        self.type_table(Direction::Input).get(&key)?.entry.as_ref()
    }

    /// Look up the resolved output entry for `ty`. See [`Self::input_entry`].
    pub fn output_entry(&self, ty: &syn::Type) -> Option<&TypeEntry<M>> {
        let key = TypeKey::from_type(ty);
        self.type_table(Direction::Output).get(&key)?.entry.as_ref()
    }

    /// Register `ty` (and its nested positions) as a required **input** so
    /// the resolver produces a converter for it. Used by
    /// [`crate::api::core::expand`] to pull in the leaf types a fold needs.
    pub(crate) fn require_input(&mut self, ty: &syn::Type) {
        // Leaf/expansion types are concrete (no disallowed `impl Trait`), so
        // the recursive registration cannot fail here.
        let _ = self.register_type_recursive(Direction::Input, ty, true);
    }

    /// Register `ty` (and its nested positions) as a required **output** so the
    /// resolver produces a converter for it. The output-side peer of
    /// [`Self::require_input`]; used by [`crate::api::core::unfold`] to pull in
    /// the leaf types a decomposition delivers.
    pub(crate) fn require_output(&mut self, ty: &syn::Type) {
        let _ = self.register_type_recursive(Direction::Output, ty, true);
    }

    /// Drop `ty` from the required-output scan set. The type's table entry is
    /// left intact (so [`crate::api::core::resolve`]'s PASS A still resolves it
    /// if it can, and emits it when resolved), but a `None` resolution no longer
    /// counts as an unresolved-required error. Used by
    /// [`crate::api::core::unfold::apply_leaf_vec_folds`]: when a `Vec<T>` /
    /// `Option<Vec<T>>` return is delivered element-by-element through a fold,
    /// the whole-collection converter is genuinely not needed — and for a
    /// `Vec<opaque-handle>` it cannot resolve at all (a `jlong` wire is not
    /// JObject-shaped), so requiring it would wrongly fail resolution.
    pub(crate) fn unrequire_output(&mut self, ty: &syn::Type) {
        self.clear_root(Direction::Output, ty);
    }

    /// Drop `ty` from the required-input scan set — the input-side peer of
    /// [`Self::unrequire_output`]. Used by [`Self::apply_adapter_plans`] for
    /// the adapter's boundary-only types: a fold plan replaces every direct
    /// crossing of the type with its ingredients, so the type's own input
    /// converter is genuinely not needed (and for an undeclared type cannot
    /// resolve at all).
    pub(crate) fn unrequire_input(&mut self, ty: &syn::Type) {
        self.clear_root(Direction::Input, ty);
    }

    /// Stop treating `ty` as a root. The cell stays, so the resolver still fills
    /// it if it can — only the demand that it *must* resolve is dropped.
    fn clear_root(&mut self, dir: Direction, ty: &syn::Type) {
        let key = TypeKey::from_type(ty);
        if let Some(cell) = self.type_table_mut(dir).get_mut(&key) {
            cell.root = false;
        }
    }

    fn scan_fn_signature(&mut self, f: &syn::ItemFn) -> Result<(), ScanError> {
        // Mechanical: register every fn-signature type as the user wrote it.
        // No semantic transformations (no &T→T strip, no ZResult<T>→T strip,
        // no skip for () / ZResult<()>). The adapter handles structural
        // wrappers; propagation through `subs` then marks transitive deps
        // (e.g. &Foo's `&_` converter returns subs=[Foo], so Foo becomes
        // required).
        // No receiver or non-ident pattern can reach here: a captured item was
        // refused by the frontend and `from_flat` failed before indexing it, and
        // a binding-local fn was checked against the same grammar
        // (`Flat::lower_signature`) when `resolve` synthesized it.
        for input in &f.sig.inputs {
            match input {
                syn::FnArg::Receiver(_) => continue,
                syn::FnArg::Typed(pt) => {
                    self.register_type_recursive(Direction::Input, &pt.ty, true)?;
                }
            }
        }
        let ret_ty: syn::Type = match &f.sig.output {
            syn::ReturnType::Default => syn::parse_quote!(()),
            syn::ReturnType::Type(_, ty) => (**ty).clone(),
        };
        self.register_type_recursive(Direction::Output, &ret_ty, true)?;
        Ok(())
    }

    fn scan_struct(&mut self, s: &syn::ItemStruct) -> Result<(), ScanError> {
        // The struct itself can appear in either direction.
        let ty: syn::Type = crate::api::core::types_util::type_from_ident(&s.ident);
        self.ensure_entry(Direction::Input, &ty, false);
        self.ensure_entry(Direction::Output, &ty, false);

        if let syn::Fields::Named(named) = &s.fields {
            for field in &named.named {
                self.register_type_recursive(Direction::Input, &field.ty, false)?;
                self.register_type_recursive(Direction::Output, &field.ty, false)?;
            }
        }
        Ok(())
    }

    fn scan_enum(&mut self, e: &syn::ItemEnum) -> Result<(), ScanError> {
        let ty: syn::Type = crate::api::core::types_util::type_from_ident(&e.ident);
        self.ensure_entry(Direction::Input, &ty, false);
        self.ensure_entry(Direction::Output, &ty, false);

        for variant in &e.variants {
            for field in &variant.fields {
                self.register_type_recursive(Direction::Input, &field.ty, false)?;
                self.register_type_recursive(Direction::Output, &field.ty, false)?;
            }
        }
        Ok(())
    }

    /// Register `ty` as a cell in the given direction, then recurse into every
    /// nested position. `root` applies only to `ty` itself — a nested position is
    /// never something the binding asked for directly.
    fn register_type_recursive(
        &mut self,
        dir: Direction,
        ty: &syn::Type,
        root: bool,
    ) -> Result<(), ScanError> {
        let mut visited: HashSet<TypeKey> = HashSet::new();
        self.register_type_inner(dir, ty, root, &mut visited)
    }

    fn register_type_inner(
        &mut self,
        dir: Direction,
        ty: &syn::Type,
        is_top: bool,
        visited: &mut HashSet<TypeKey>,
    ) -> Result<(), ScanError> {
        // A disallowed `impl Trait` cannot reach here: every fn whose signature
        // reaches this point passed the frontend's grammar — captured items at
        // ingestion, binding-local ones at synthesis — and it names the
        // parameter the bad type sits on.

        let key = TypeKey::from_type(ty);
        if !visited.insert(key.clone()) {
            return Ok(()); // cycle guard
        }

        self.ensure_entry(dir, ty, is_top);

        for (child_dir, sub) in self.immediate_edges(dir, ty) {
            self.register_type_inner(child_dir, &sub, false, visited)?;
        }
        Ok(())
    }

    /// Create the cell for `ty` in `dir` if it has none, and mark it a root when
    /// the binding asked for it directly.
    ///
    /// The one place a cell is born, which is what lets the subject be decided
    /// once: the model's reading if the flat API mentions this type, an
    /// adapter-authored type otherwise.
    fn ensure_entry(&mut self, dir: Direction, ty: &syn::Type, root: bool) {
        let key = TypeKey::from_type(ty);
        let subject = match self.flat.type_ref(ty) {
            Some(t) => TypeSubject::Source(t.clone()),
            None => TypeSubject::Adapter(key.to_type()),
        };
        let cell = self
            .type_table_mut(dir)
            .entry(key)
            .or_insert_with(|| TypeCell {
                subject,
                root: false,
                entry: None,
            });
        cell.root |= root;
    }

    /// Enumerate the immediate type-graph edges out of `(dir, ty)`:
    /// generic args / Fn args / tuple elements / ref/array/slice/ptr targets,
    /// plus — if `ty` is the bare ident of an indexed struct or enum — the
    /// field types of that struct/enum.
    ///
    /// `impl Fn(args)` arg types flow with `dir.flip()`; everything else
    /// inherits `dir`. Used by both `register_type_inner` (during scan) and
    /// the unresolved-descendants BFS in `resolve` (for diagnostics).
    pub(crate) fn immediate_edges(
        &self,
        dir: Direction,
        ty: &syn::Type,
    ) -> Vec<(Direction, syn::Type)> {
        let mut out: Vec<(Direction, syn::Type)> = Vec::new();
        let (positions, child_dir) = if let Some(args) = extract_fn_trait_args(ty) {
            (args, dir.flip())
        } else {
            (immediate_subtype_positions(ty), dir)
        };
        for sub in positions {
            out.push((child_dir, sub));
        }
        // A declared type's own fields, read off the element rather than off its
        // `syn::Fields`: a positional field is an ordinary `Field` there, so the
        // named-only asymmetry the syntax walk had does not arise. An `Enum` has
        // no fields and an `Extern` declares none, which is what makes both
        // contribute nothing here.
        if let Some(name) = bare_path_ident(ty) {
            use crate::api::core::flat::{Field, Type};
            let fields: Vec<&Field> = match self.flat.declared_type(&name) {
                Some(Type::Struct(s)) => s.fields.iter().collect(),
                Some(Type::Variant(v)) => v
                    .alternatives
                    .iter()
                    .flat_map(|a| a.fields.iter())
                    .collect(),
                Some(Type::Enum(_) | Type::Extern(_)) | None => Vec::new(),
            };
            for field in fields {
                out.push((dir, field.ty.origin.syntax.clone()));
            }
        }
        out
    }

    /// Resolve the binding: scan the adapter's declarations, apply its
    /// plans, and run type resolution — consuming both the registry and the
    /// adapter into a [`Generation`], whose `write_*` methods are pure,
    /// order-free emissions. This is the single public entry point for
    /// language-specific binding generation; language-agnostic because
    /// `adapter` is any [`crate::api::core::prebindgen::Prebindgen`] impl
    /// whose `Metadata` matches this registry's `M` parameter.
    ///
    /// ```ignore
    /// let gen = Registry::new(Flat::builder().items(source.items_all()).build()?)?
    ///     .resolve(jni)?;
    /// gen.write_rust(&rust_dest)?;
    /// gen.write_kotlin(&kotlin_root)?;   // JNI adapter's second artifact
    /// ```
    pub fn resolve<E>(mut self, adapter: E) -> Result<Generation<E>, WriteRustError>
    where
        E: Prebindgen<Metadata = M>,
        M: Clone + Default,
    {
        let mut declared = std::mem::take(&mut self.declared);
        self.scan_declared_items(&declared)?;
        adapter
            .validate(&self)
            .map_err(|message| ScanError::AdapterInvariant { message })?;
        self.apply_adapter_plans(&mut declared)?;
        self.declared = declared;
        crate::api::core::resolve::resolve(&mut self, &adapter)?;
        // Post-resolve validation runs ONCE here, so a `Generation` is valid
        // by construction and the `write_*` emitters are genuinely pure
        // (previously each writer re-ran this, validating twice per build).
        // Sibling of the pre-resolve `validate` above — same adapter-invariant
        // channel. An invalid binding fails `resolve`; no `Generation` is
        // produced, so nothing can be written.
        adapter
            .validate_resolved(&self)
            .map_err(|message| ScanError::AdapterInvariant { message })?;
        Ok(Generation {
            registry: self,
            adapter,
        })
    }

    fn apply_adapter_plans(&mut self, declared: &mut Declared) -> Result<(), WriteRustError> {
        // The set of declared fns drives `.default()` auto-apply: a defaulted
        // constructor/deconstructor is synthesized for every matching declared
        // fn. `accessors` is the `.fun_accessor` subset: excluded from
        // constructor composition and the only fns a decomposer record may
        // reference.
        let d = &mut declared.decompositions;
        if let Some(exp) = &d.expansions {
            crate::api::core::expand::apply(
                self,
                exp,
                &declared.functions,
                &declared.accessors,
                &declared.method_receivers,
            )?;
        }
        if let Some(dec) = &d.deconstructors {
            crate::api::core::unfold::apply(self, dec, &declared.functions, &declared.accessors)?;
        }
        // Synthesized by-value `data_class` decompositions: the adapter already
        // built the leaves; this wires them into fixed-builder plans.
        if !d.value_structs.is_empty() {
            crate::api::core::unfold::apply_value_structs(
                self,
                std::mem::take(&mut d.value_structs),
                &declared.functions,
            )?;
        }
        // The same wiring for a value whose alternatives are chosen at runtime
        // (tag + one leaf group per variant) rather than being a fixed product.
        if !d.sums.is_empty() {
            crate::api::core::unfold::apply_sum_returns(
                self,
                std::mem::take(&mut d.sums),
                &declared.functions,
            )?;
        }
        // Single-leaf `Vec<T>`/`&[T]` whole-element folds — the dual of the
        // `data_class` folds above, for String / scalar / handle elements
        // (so the list is built on the foreign side, not via a Rust ArrayList).
        if !d.leaf_vec_elements.is_empty() {
            crate::api::core::unfold::apply_leaf_vec_folds(
                self,
                std::mem::take(&mut d.leaf_vec_elements),
                &declared.functions,
            )?;
        }
        // Every crossing these types make is now covered by a plan, so the
        // scan-time direct converter requirement is stale — and typically
        // unresolvable, since such a type has no destination representation.
        // Drop it both ways; the cell stays, so a converter is still produced
        // if one happens to resolve.
        for key in &declared.decompositions.replaces {
            let ty = key.to_type();
            self.unrequire_input(&ty);
            self.unrequire_output(&ty);
        }
        Ok(())
    }
}

// ──────────────────────────────────────────────────────────────────────
// Helpers
// ──────────────────────────────────────────────────────────────────────

/// Immediate child type positions of `ty` (one level deep).
pub fn immediate_subtype_positions(ty: &syn::Type) -> Vec<syn::Type> {
    match ty {
        syn::Type::Path(p) => {
            if let Some(last) = p.path.segments.last() {
                if let syn::PathArguments::AngleBracketed(ab) = &last.arguments {
                    return ab
                        .args
                        .iter()
                        .filter_map(|a| {
                            if let syn::GenericArgument::Type(t) = a {
                                Some(t.clone())
                            } else {
                                None
                            }
                        })
                        .collect();
                }
            }
            vec![]
        }
        syn::Type::Reference(r) => vec![(*r.elem).clone()],
        syn::Type::Tuple(t) => t.elems.iter().cloned().collect(),
        syn::Type::Array(a) => vec![(*a.elem).clone()],
        syn::Type::Slice(s) => vec![(*s.elem).clone()],
        syn::Type::Ptr(p) => vec![(*p.elem).clone()],
        syn::Type::Group(g) => immediate_subtype_positions(&g.elem),
        syn::Type::Paren(p) => immediate_subtype_positions(&p.elem),
        syn::Type::ImplTrait(_) => extract_fn_trait_args(ty).unwrap_or_default(),
        _ => vec![],
    }
}

/// The callback grammar, which the source language owns — re-exported here for
/// the existing call sites until they consume elements (stages L2–L4 of #229).
pub use crate::api::core::flat::extract_fn_trait_args;

/// A **resolved** binding generation: the [`Registry`] after
/// [`Registry::resolve`] ran the adapter's scan, plans, and type
/// resolution, bound together with the adapter that produced it. Both
/// halves of a generation run are methods here — [`Self::write_rust`] and
/// any adapter-specific artifact (e.g. `write_kotlin` for the JNI
/// adapter) — so the resolve-before-write ordering is enforced by
/// construction, and the writes themselves are pure reads that may run in
/// any order.
pub struct Generation<E: Prebindgen> {
    registry: Registry<E::Metadata>,
    adapter: E,
}

// Opaque — exists so `Result<Generation, _>::expect_err` works in tests.
impl<E: Prebindgen> fmt::Debug for Generation<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Generation(..)")
    }
}

impl<E: Prebindgen> Generation<E> {
    /// Write the generated Rust bindings file. `out_path` may be relative
    /// (resolved against `OUT_DIR`) or absolute; returns the path actually
    /// written. Pure emission — the registry was fully resolved by
    /// [`Registry::resolve`].
    pub fn write_rust(
        &self,
        out_path: impl AsRef<std::path::Path>,
    ) -> Result<std::path::PathBuf, WriteRustError> {
        Ok(crate::api::core::write::write_rust(
            &self.registry,
            &self.adapter,
            out_path,
        )?)
    }

    /// The resolved registry (converter tables, plans, item maps).
    pub fn registry(&self) -> &Registry<E::Metadata> {
        &self.registry
    }

    /// The adapter this generation was resolved with.
    pub fn adapter(&self) -> &E {
        &self.adapter
    }
}

#[cfg(test)]
mod tests;
