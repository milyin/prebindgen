//! Single owner of everything parsed from the prebindgen source stream.
//!
//! [`Registry`] holds:
//! * Item maps (`functions`, `structs`, `enums`, `consts`) indexed by ident.
//!   Duplicate names across kinds OR within a kind are an error — prebindgen
//!   items live in one flat namespace.
//! * `guards` — anonymous consts, emitted verbatim. Not API: having no name,
//!   they cannot be declared, so they are neither gated nor addressable.
//! * `input_types` / `output_types` — direction-specific type tables. Each
//!   scanned type maps to either a resolved [`TypeEntry`] or an unresolved cell
//!   that the fixed-point resolver can retry.
//! * Expansion/deconstruction sidecars — adapter declarations are resolved into
//!   plans before type resolution, then consumed at wrapper-emission sites.

use std::{
    collections::{HashMap, HashSet},
    fmt,
    marker::PhantomData,
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
    /// the entry points are [`Self::from_items`], [`Self::from_flat`] and
    /// [`Self::builder`], each of which has a model behind it.
    pub(crate) fn empty() -> Self {
        Self {
            flat: crate::api::core::flat::Flat::default(),
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
    /// ingestion via [`Registry::from_items`]) — the `SourceLocation`
    /// file paths are crate-relative, so with several sources they alone
    /// may not identify the colliding crates.
    pub first_crate: Option<String>,
    pub second_crate: Option<String>,
}

/// Errors surfaced by the scan phase.
#[derive(Debug)]
pub enum ScanError {
    DuplicateName(Box<DuplicateNameError>),
    ConflictingFunctionIntent {
        name: syn::Ident,
    },
    ConflictingTypeIntent {
        key: TypeKey,
    },
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
            ScanError::ConflictingFunctionIntent { name } => {
                write!(f, "function `{}` cannot be both declared and ignored", name)
            }
            ScanError::ConflictingTypeIntent { key } => {
                write!(f, "type `{}` cannot be both declared and ignored", key)
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
pub struct Declarations {
    pub(crate) functions: HashSet<syn::Ident>,
    pub(crate) ignored_functions: HashSet<syn::Ident>,
    /// Bulk-ignore predicates over item names — every matching *undeclared*
    /// item (fn, struct/enum, const) is an acknowledged skip (no warning).
    /// Kind-agnostic: prebindgen names live in one flat namespace. A
    /// declared item matching a predicate is unaffected: declaration wins.
    /// See [`Prebindgen::ignored_name_predicates`].
    pub(crate) ignored_name_predicates: Vec<crate::api::core::prebindgen::NamePredicate>,
    /// Signature-scanned but not emitted — see [`Prebindgen::helper_functions`].
    pub(crate) helper_functions: HashSet<syn::Ident>,
    pub(crate) accessors: HashSet<syn::Ident>,
    pub(crate) method_receivers: HashMap<syn::Ident, TypeKey>,
    pub(crate) types: HashSet<TypeKey>,
    pub(crate) ignored_types: HashSet<TypeKey>,
    /// Types converted exclusively through the adapter's plans (built from
    /// ingredients / decomposed into fields); acknowledged for warning
    /// purposes and un-required after plans — see
    /// [`Prebindgen::boundary_only_types`].
    pub(crate) boundary_only_types: HashSet<TypeKey>,
    /// `None` = the adapter has no const declaration mechanism (all consts
    /// re-emitted verbatim, no scan, no warnings) — see
    /// [`Prebindgen::declared_consts`].
    pub(crate) consts: Option<HashSet<syn::Ident>>,
    pub(crate) ignored_consts: HashSet<syn::Ident>,
    /// Adapter-required extra output types (no `#[prebindgen]` item to
    /// scan — e.g. expression-constant value types); see
    /// [`Prebindgen::required_output_types`].
    pub(crate) required_output_types: Vec<syn::Type>,
}

impl Declarations {
    /// A binding that declares nothing.
    pub fn new() -> Self {
        Self::default()
    }

    /// Functions to emit.
    pub fn functions(mut self, v: HashSet<syn::Ident>) -> Self {
        self.functions = v;
        self
    }
    /// Functions deliberately not emitted — acknowledged, so no warning.
    pub fn ignored_functions(mut self, v: HashSet<syn::Ident>) -> Self {
        self.ignored_functions = v;
        self
    }
    /// Bulk-ignore predicates over item names, kind-agnostic. A declared item
    /// matching one is unaffected: declaration wins.
    pub fn ignored_name_predicates(
        mut self,
        v: Vec<crate::api::core::prebindgen::NamePredicate>,
    ) -> Self {
        self.ignored_name_predicates = v;
        self
    }
    /// Signature-scanned but never emitted.
    pub fn helper_functions(mut self, v: HashSet<syn::Ident>) -> Self {
        self.helper_functions = v;
        self
    }
    /// Functions used as accessors by a decomposition.
    pub fn accessors(mut self, v: HashSet<syn::Ident>) -> Self {
        self.accessors = v;
        self
    }
    /// The receiver type of each function emitted as a method.
    pub fn method_receivers(mut self, v: HashMap<syn::Ident, TypeKey>) -> Self {
        self.method_receivers = v;
        self
    }
    /// Types to emit.
    pub fn types(mut self, v: HashSet<TypeKey>) -> Self {
        self.types = v;
        self
    }
    /// Types deliberately not emitted.
    pub fn ignored_types(mut self, v: HashSet<TypeKey>) -> Self {
        self.ignored_types = v;
        self
    }
    /// Types that cross only through a plan, never whole. Acknowledged for
    /// warnings and un-required once plans are applied.
    pub fn boundary_only_types(mut self, v: HashSet<TypeKey>) -> Self {
        self.boundary_only_types = v;
        self
    }
    /// Consts to emit. `None` = this binding has no const mechanism, so every
    /// const is re-emitted verbatim with no scan and no warnings.
    pub fn consts(mut self, v: Option<HashSet<syn::Ident>>) -> Self {
        self.consts = v;
        self
    }
    /// Consts deliberately not emitted.
    pub fn ignored_consts(mut self, v: HashSet<syn::Ident>) -> Self {
        self.ignored_consts = v;
        self
    }
    /// Output types with no `#[prebindgen]` item behind them — an expression
    /// constant's value type, say — that still need a converter.
    pub fn required_output_types(mut self, v: Vec<syn::Type>) -> Self {
        self.required_output_types = v;
        self
    }

    /// Reject a declaration that says two things at once.
    fn check(self) -> Result<Self, ScanError> {
        let declared = self;

        if let Some(name) = declared
            .functions
            .intersection(&declared.ignored_functions)
            .cloned()
            .min_by_key(|ident| ident.to_string())
        {
            return Err(ScanError::ConflictingFunctionIntent { name });
        }
        if let Some(key) = declared
            .types
            .intersection(&declared.ignored_types)
            .cloned()
            .min_by_key(|key| key.as_str().to_owned())
        {
            return Err(ScanError::ConflictingTypeIntent { key });
        }

        Ok(declared)
    }
}

/// Collects what a [`Registry`] will index, then hands over the registry.
///
/// The same shape [`Flat`](crate::core::flat) reads prebindgen data with — name a
/// directory, or feed a stream, then `build()` — so there is one way to say where
/// captured items come from, whoever consumes them.
///
/// Feeders accumulate, so mix them freely.
pub struct RegistryBuilder<M> {
    items: Vec<(syn::Item, SourceLocation)>,
    /// `M` is fixed by the adapter a caller eventually `resolve`s with, and is
    /// carried through so `Registry::builder()` needs no turbofish: it threads
    /// from here to [`Self::build`] and is inferred at the `resolve` call.
    _metadata: PhantomData<M>,
}

impl<M> RegistryBuilder<M> {
    /// Every `#[prebindgen]` item captured in `dir` — pass
    /// `<source_crate>::PREBINDGEN_OUT_DIR`.
    ///
    /// Panics the way [`Source::new`](crate::Source::new) does if `dir` is not
    /// readable prebindgen output: a build script has nothing to recover with.
    pub fn source<P: AsRef<std::path::Path>>(self, dir: P) -> Self {
        let source = crate::Source::new(dir);
        self.items(source.items_all())
    }

    /// The same, for a dependency this crate **renames** in `Cargo.toml`.
    ///
    /// The origin recorded at capture time is the dependency's real package name,
    /// which will not resolve from a crate that refers to it by another name.
    /// `crate_name` is the name *this* crate uses.
    ///
    /// Per directory, deliberately: a registry-level override could only fix one
    /// module, and a registry may layer several sources.
    pub fn source_named<P: AsRef<std::path::Path>>(
        self,
        dir: P,
        crate_name: impl Into<String>,
    ) -> Self {
        let source = crate::Source::builder(dir).crate_name(crate_name).build();
        self.items(source.items_all())
    }

    /// Add a captured item stream — a group selection, an otherwise-configured
    /// [`Source`](crate::Source), or synthetic items in a test.
    pub fn items<I>(mut self, items: I) -> Self
    where
        I: IntoIterator<Item = (syn::Item, SourceLocation)>,
    {
        self.items.extend(items);
        self
    }

    /// Index everything collected so far.
    ///
    /// Sugar over [`Registry::from_items`], which stays the primitive: one place
    /// decides what a stream of items means.
    pub fn build(self) -> Result<Registry<M>, ScanError> {
        Registry::from_items(self.items)
    }
}

impl<M> Registry<M> {
    /// Start collecting what to index.
    ///
    /// The way a build script reads prebindgen output: name the directory and get
    /// a registry, with no [`Source`](crate::Source) in between.
    ///
    /// ```
    /// # prebindgen::Source::init_doctest_simulate();
    /// use prebindgen::core::Registry;
    ///
    /// // Annotated only because nothing here resolves: in a build script `M` is
    /// // fixed by the adapter passed to `resolve`, so no call site names it.
    /// let registry: Registry<()> = Registry::builder().source("source_ffi").build()?;
    /// assert!(registry.flat().function("test_function").is_some());
    /// # Ok::<_, prebindgen::core::ScanError>(())
    /// ```
    ///
    /// Several directories compose, including one this crate renames:
    ///
    /// ```ignore
    /// let registry = Registry::builder()
    ///     .source(flat_crate::PREBINDGEN_OUT_DIR)
    ///     .source_named(helpers::PREBINDGEN_OUT_DIR, "helpers")
    ///     .build()?;
    /// ```
    pub fn builder() -> RegistryBuilder<M> {
        RegistryBuilder {
            items: Vec::new(),
            _metadata: PhantomData,
        }
    }

    /// Construct a `Registry` by indexing a stream of source items.
    ///
    /// The primitive: [`Self::builder`] is sugar over this, and is what a build
    /// script reading a directory should reach for. Callers feed any
    /// `(syn::Item, SourceLocation)` iterator — typically `source.items_all()`,
    /// `source.items_except_groups(...)`, or a hand-rolled filter chain — so
    /// item-level selection happens upstream of the registry rather than inside
    /// it. Streams from several sources combine with plain iterator composition:
    ///
    /// ```ignore
    /// let registry = Registry::from_items(
    ///     flat.items_all().chain(helpers.items_all()),
    /// )?;
    /// ```
    ///
    /// Each item's **origin crate** rides its [`SourceLocation`] (stamped
    /// by [`Source`](crate::Source) when parsing records): named items get
    /// their origin recorded for qualified references in generated code
    /// (`flat_crate::…` vs `helper_crate::…`), and the first origin seen
    /// becomes the default module ([`Self::default_module`]). When a
    /// dependency is renamed in Cargo.toml, override the stamp at the
    /// source: `Source::builder(dir).crate_name("myflat")` — being
    /// per-source, it composes across chained streams (a registry-level
    /// override could only fix one module).
    ///
    /// This step only populates the item maps (`functions`, `structs`,
    /// `enums`, `consts`, `guards`). Signature/body scanning that
    /// drives type-resolution requirements happens later, in
    /// [`Self::scan_declared`], and is gated on what the language adapter
    /// has explicitly declared. An **API** item that is never declared remains
    /// in the registry but never drives type resolution and never emits.
    ///
    /// `guards` is the exception, and it is not one of the API maps: an
    /// anonymous const has no name to declare, so it is outside the gate
    /// entirely and always emits.
    pub fn from_items<I>(items: I) -> Result<Self, ScanError>
    where
        I: IntoIterator<Item = (syn::Item, SourceLocation)>,
    {
        let flat = crate::api::core::flat::Flat::builder()
            .items(items)
            .build()?;
        Self::from_flat(flat)
    }

    /// Index a parsed [`Flat`](crate::api::core::flat::Flat) model.
    ///
    /// The registry is a **projection** of the model, not a second reading of the
    /// source: `Flat` decided what every item means, and this arranges those
    /// decisions into the maps adapters read. The model itself is kept
    /// ([`Self::flat`]) so later stages can ask it questions rather than
    /// re-deriving them.
    ///
    /// **Fails on anything the language cannot express** — a `self` receiver, an
    /// `async fn`, a generic binder, a type form outside the grammar, or a
    /// reference to a type the flat API does not declare. All of them at once, so
    /// a source crate that needs migrating sees one list instead of one rebuild
    /// per item.
    pub fn from_flat(flat: crate::api::core::flat::Flat) -> Result<Self, ScanError> {
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

    /// Every **struct or enum** name — either enum shape, never an alias.
    ///
    /// Named for its population rather than as the iterator form of
    /// [`Self::declares_type`], which it is **not**: that predicate counts every
    /// declared type, aliases included. This one feeds *"skipping undeclared
    /// `#[prebindgen]` struct/enum"*, which names a kind an alias is not — so the
    /// two answer differently on purpose, and the names now say so.
    fn struct_enum_idents(&self) -> impl Iterator<Item = &syn::Ident> {
        use crate::api::core::flat::Type;
        self.flat.types().filter_map(|t| match t {
            Type::Struct(_) | Type::Variant(_) | Type::Enum(_) => Some(t.name()),
            Type::Extern(_) => None,
        })
    }

    /// Whether the source declares a type under this name — **including an
    /// alias**.
    ///
    /// The question both type-diagnostic sites ask, shared so they cannot drift.
    /// An alias counts because `#[prebindgen] pub type Handle = ..` *is* a
    /// declaration of that name: it can be declared bare by an adapter (landing
    /// in the no-indexed-body branch above, which is what
    /// `ptr_class(ZKeyExpr<'static>)` relies on), so a diagnostic that says
    /// "no such captured item" would be false.
    ///
    /// Distinct from [`Self::struct_enum_idents`], which excludes aliases
    /// because it feeds a *"skipping undeclared struct/enum"* warning — a
    /// different question, about what an adapter left unclaimed.
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
    pub fn scan_declared<E>(&mut self, ext: &E) -> Result<(), ScanError>
    where
        E: Prebindgen<Metadata = M>,
    {
        let declared = ext.declarations().check()?;
        self.scan_declared_items(&declared)
    }

    fn scan_declared_items(&mut self, declared: &Declarations) -> Result<(), ScanError> {
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
            .chain(declared.ignored_types.iter())
            .chain(declared.boundary_only_types.iter())
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
        // [`ScanError::DeclaredNotFound`]); stale *ignore* entries below only
        // warn.
        let mut missing: Vec<(&'static str, String)> = Vec::new();

        // Scan declared functions.
        for ident in &declared.functions {
            if let Some(item_fn) = self.flat.function(&ident).map(|f| f.origin.syntax.clone()) {
                self.scan_fn_signature(&item_fn)?;
            } else {
                missing.push(("function", ident.to_string()));
            }
        }

        for ident in &declared.ignored_functions {
            if self.flat.function(&ident).is_none() {
                println!(
                    "cargo:warning=prebindgen: ignored function `{}` not found among #[prebindgen] items",
                    ident
                );
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

        // Scan declared consts (only when the adapter has a const
        // declaration mechanism): a const is a nullary source of its type,
        // so the type is required in the output direction only.
        if let Some(decl_consts) = &declared.consts {
            for ident in decl_consts {
                if let Some(item_const) =
                    self.flat.constant(&ident).map(|c| c.origin.syntax.clone())
                {
                    self.ensure_entry(Direction::Output, &item_const.ty, true);
                } else {
                    missing.push(("constant", ident.to_string()));
                }
            }
            for ident in &declared.ignored_consts {
                if self.flat.constant(&ident).is_none() {
                    println!(
                        "cargo:warning=prebindgen: ignored const `{}` not found among #[prebindgen] items",
                        ident
                    );
                }
            }
        }

        if !missing.is_empty() {
            missing.sort();
            return Err(ScanError::DeclaredNotFound { entries: missing });
        }

        // Adapter-required extra output types — synthesized values with no
        // `#[prebindgen]` item behind them (e.g. expression constants).
        for ty in &declared.required_output_types {
            self.ensure_entry(Direction::Output, ty, true);
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

        for key in &declared.ignored_types {
            let ty = key.to_type();
            let matched = bare_path_ident(&ty).is_some_and(|ident| self.declares_type(&ident));
            if !matched {
                println!(
                    "cargo:warning=prebindgen: ignored type `{}` not found among #[prebindgen] items",
                    key.as_str()
                );
            }
        }

        // Warn about indexed items that the adapter never claimed. An
        // ignore *predicate* acknowledges every matching item in bulk —
        // kind-agnostic, since prebindgen names live in one flat namespace;
        // a predicate matching nothing is silent by design (it is a filter,
        // not a claim — match counts vary across feature configurations).
        let pred_ignored = |name: &str| {
            !declared.ignored_name_predicates.is_empty()
                && declared.ignored_name_predicates.iter().any(|p| p(name))
        };
        let mut skipped_fns: Vec<String> = self
            .flat
            .functions()
            .map(|f| &f.name)
            .filter(|k| {
                !declared.functions.contains(*k)
                    && !declared.ignored_functions.contains(*k)
                    && !declared.helper_functions.contains(*k)
                    && !pred_ignored(&k.to_string())
            })
            .map(|k| k.to_string())
            .collect();
        skipped_fns.sort();
        for name in &skipped_fns {
            println!(
                "cargo:warning=prebindgen: skipping undeclared #[prebindgen] fn `{}`",
                name
            );
        }

        let mut skipped_types: Vec<String> = Vec::new();
        let type_acknowledged = |key: &TypeKey| {
            declared.types.contains(key)
                || declared.ignored_types.contains(key)
                || declared.boundary_only_types.contains(key)
        };
        for ident in self.struct_enum_idents() {
            let name = ident.to_string();
            let key = TypeKey::from_ident(ident);
            if !type_acknowledged(&key) && !pred_ignored(&name) {
                skipped_types.push(name);
            }
        }
        skipped_types.sort();
        for name in &skipped_types {
            println!(
                "cargo:warning=prebindgen: skipping undeclared #[prebindgen] struct/enum `{}`",
                name
            );
        }

        if let Some(decl_consts) = &declared.consts {
            let mut skipped_consts: Vec<String> = self
                .flat
                .constants()
                .map(|c| &c.name)
                .filter(|k| {
                    !decl_consts.contains(*k)
                        && !declared.ignored_consts.contains(*k)
                        && !pred_ignored(&k.to_string())
                })
                .map(|k| k.to_string())
                .collect();
            skipped_consts.sort();
            for name in &skipped_consts {
                println!(
                    "cargo:warning=prebindgen: skipping undeclared #[prebindgen] const `{}`",
                    name
                );
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
    /// let gen = Registry::from_items(source.items_all())?.resolve(jni)?;
    /// gen.write_rust(&rust_dest)?;
    /// gen.write_kotlin(&kotlin_root)?;   // JNI adapter's second artifact
    /// ```
    pub fn resolve<E>(mut self, adapter: E) -> Result<Generation<E>, WriteRustError>
    where
        E: Prebindgen<Metadata = M>,
        M: Clone + Default,
    {
        // Synthesis pre-pass: adapter-declared BINDING-LOCAL fns become
        // ordinary registry entries (signature read from the synthesized
        // item, calls qualified by the recorded origin), so every downstream
        // stage treats them exactly like `#[prebindgen]` fns.
        for (item_fn, origin) in adapter.local_functions() {
            let ident = item_fn.sig.ident.clone();
            // The one input that does not come through `Flat`: a `sig!(..)` is
            // written by hand in a build script, so the grammar has to be checked
            // here or nowhere. Silently dropping a `self` receiver or a pattern
            // parameter would surface as an arity mismatch out of rustc on
            // generated code, which is the wrong end of the pipeline to learn
            // about a build.rs typo.
            let lowered = match self.flat.lower_signature(&item_fn) {
                Ok(f) => f,
                Err(error) => {
                    return Err(ScanError::AdapterInvariant {
                        message: format!("binding-local fn `{ident}`: {error}"),
                    }
                    .into())
                }
            };
            if self.flat.element(&ident).is_some() {
                return Err(ScanError::AdapterInvariant {
                    message: format!(
                        "binding-local fn `{ident}` collides with a `#[prebindgen]` item — \
                         the generated call would resolve the wrong fn; rename the \
                         binding-local fn"
                    ),
                }
                .into());
            }
            // Into the model, so every downstream stage finds it exactly where it
            // finds a captured fn — there is one index, and this is it.
            self.flat.add_local_function(lowered, origin);
        }
        let declared = adapter.declarations().check()?;
        self.scan_declared_items(&declared)?;
        adapter
            .validate(&self)
            .map_err(|message| ScanError::AdapterInvariant { message })?;
        self.apply_adapter_plans(&adapter, &declared)?;
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

    fn apply_adapter_plans<E>(
        &mut self,
        ext: &E,
        declared: &Declarations,
    ) -> Result<(), WriteRustError>
    where
        E: Prebindgen<Metadata = M>,
    {
        // The set of declared fns drives `.default()` auto-apply: a defaulted
        // constructor/deconstructor is synthesized for every matching declared
        // fn. `accessors` is the `.fun_accessor` subset: excluded from
        // constructor composition and the only fns a decomposer record may
        // reference.
        if let Some(exp) = ext.expansions() {
            crate::api::core::expand::apply(
                self,
                &exp,
                &declared.functions,
                &declared.accessors,
                &declared.method_receivers,
            )?;
        }
        if let Some(dec) = ext.deconstructors(self) {
            crate::api::core::unfold::apply(self, &dec, &declared.functions, &declared.accessors)?;
        }
        // Synthesized by-value `data_class` decompositions: build the leaves
        // (immutable borrow), then wire them into fixed-builder plans.
        let value_decons = ext.value_struct_decons(self);
        if !value_decons.is_empty() {
            crate::api::core::unfold::apply_value_structs(self, value_decons, &declared.functions)?;
        }
        // Synthesized sum decompositions: the same fixed-builder wiring for a
        // value whose alternatives are chosen at runtime (tag + one leaf group
        // per variant) rather than being a fixed product.
        let sum_decons = ext.sum_decons(self);
        if !sum_decons.is_empty() {
            crate::api::core::unfold::apply_sum_returns(self, sum_decons, &declared.functions)?;
        }
        // Single-leaf `Vec<T>`/`&[T]` whole-element folds — the dual of the
        // `data_class` folds above, for String / scalar / handle elements
        // (so the list is built on the foreign side, not via a Rust ArrayList).
        let leaf_elements = ext.leaf_vec_fold_elements(self);
        if !leaf_elements.is_empty() {
            crate::api::core::unfold::apply_leaf_vec_folds(
                self,
                leaf_elements,
                &declared.functions,
            )?;
        }
        // Adapter-derived extra requirements (registry-aware — e.g. the
        // other-side types of `convert!` conversion fns, per direction).
        for (dir, ty) in ext.extra_required_types(self) {
            match dir {
                Direction::Input => self.require_input(&ty),
                Direction::Output => self.require_output(&ty),
            }
        }
        // Boundary-only types: every crossing is now covered by a plan (fold
        // in, unfold out / error channel), so the scan-time direct converter
        // requirement is stale — and typically unresolvable, since the type
        // has no destination-language representation. Drop it both ways; the
        // entry stays in the table, so a converter is still produced if one
        // happens to resolve.
        for key in &declared.boundary_only_types {
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
