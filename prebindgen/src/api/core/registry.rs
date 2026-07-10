//! Single owner of everything parsed from the prebindgen source stream.
//!
//! [`Registry`] holds:
//! * Item maps (`functions`, `structs`, `enums`, `consts`) indexed by ident.
//!   Duplicate names across kinds OR within a kind are an error — prebindgen
//!   items live in one flat namespace.
//! * `passthrough` — items that aren't function/struct/enum/const (use, mod,
//!   type alias, macro_rules) emitted verbatim.
//! * `input_types` / `output_types` — direction-specific type tables. Each
//!   scanned type maps to either a resolved [`TypeEntry`] or an unresolved cell
//!   that the fixed-point resolver can retry.
//! * Expansion/deconstruction sidecars — adapter declarations are resolved into
//!   plans before type resolution, then consumed at wrapper-emission sites.

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

/// Canonical type-shape key — the `to_token_stream().to_string()` form of a
/// `syn::Type`. Whitespace-normalised (`"Vec<u8>"` and `"Vec < u8 >"` produce
/// the same key after parse-and-restringify).
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct TypeKey(String);

impl TypeKey {
    /// Build a key by parsing the input as a type and re-serialising. Panics
    /// if the input does not parse as a `syn::Type`.
    pub fn parse(s: &str) -> Self {
        let ty: syn::Type = syn::parse_str(s)
            .unwrap_or_else(|e| panic!("TypeKey::parse: invalid type `{}`: {}", s, e));
        Self::from_type(&ty)
    }

    /// Build a key directly from a `syn::Type`.
    pub fn from_type(ty: &syn::Type) -> Self {
        Self(ty.to_token_stream().to_string())
    }

    /// The canonical string form.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Parse the key back into a `syn::Type`. Always succeeds because the
    /// key was originally constructed from a parseable type.
    pub fn to_type(&self) -> syn::Type {
        syn::parse_str(&self.0).unwrap_or_else(|e| {
            panic!(
                "TypeKey::to_type: stored key `{}` no longer parses: {}",
                self.0, e
            )
        })
    }
}

impl fmt::Display for TypeKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
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
    /// Initially true for types that appear directly in a `#[prebindgen]` fn
    /// signature; false for sub-positions. Promoted true by the propagation
    /// pass for any type reachable via `subs` from another required type.
    pub required: bool,
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
    pub functions: HashMap<syn::Ident, (syn::ItemFn, SourceLocation)>,
    pub structs: HashMap<syn::Ident, (syn::ItemStruct, SourceLocation)>,
    pub enums: HashMap<syn::Ident, (syn::ItemEnum, SourceLocation)>,
    pub consts: HashMap<syn::Ident, (syn::ItemConst, SourceLocation)>,
    /// Anything else (use, mod, type alias, macro_rules) — passed through.
    pub passthrough: Vec<(syn::Item, SourceLocation)>,

    /// Type tables, one per direction. Each scanned type maps to its resolved
    /// [`TypeEntry`] (`Some`) or stays unresolved (`None`) until the structural
    /// resolver fills it.
    pub input_types: HashMap<TypeKey, Option<TypeEntry<M>>>,
    pub output_types: HashMap<TypeKey, Option<TypeEntry<M>>>,

    /// First-seen source location for each type key. Used in error messages
    /// to point the user at where a required-but-unresolved type came from.
    pub type_locations: HashMap<TypeKey, SourceLocation>,

    /// Sidecar tracking which keys were registered as top-level fn-signature
    /// types, separate from per-entry `required` (which the resolver flips
    /// into `TypeEntry::required` once an entry is filled).
    pub required_inputs_scan: HashSet<TypeKey>,
    pub required_outputs_scan: HashSet<TypeKey>,

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

impl<M> Default for Registry<M> {
    fn default() -> Self {
        Self {
            functions: HashMap::new(),
            structs: HashMap::new(),
            enums: HashMap::new(),
            consts: HashMap::new(),
            passthrough: Vec::new(),
            input_types: Default::default(),
            output_types: Default::default(),
            type_locations: HashMap::new(),
            required_inputs_scan: HashSet::new(),
            required_outputs_scan: HashSet::new(),
            expansion_plans: HashMap::new(),
            unfold_plans: HashMap::new(),
            error_plans: HashMap::new(),
            callback_arg_plans: HashMap::new(),
            decon_plans: HashMap::new(),
        }
    }
}

/// Errors surfaced by the scan phase.
#[derive(Debug)]
pub enum ScanError {
    DuplicateName {
        name: syn::Ident,
        first: SourceLocation,
        second: SourceLocation,
    },
    ConflictingFunctionIntent {
        name: syn::Ident,
    },
    ConflictingTypeIntent {
        key: TypeKey,
    },
    DisallowedImplTrait {
        ty: String,
        loc: SourceLocation,
    },
    UnsupportedReceiver {
        loc: SourceLocation,
    },
    UnsupportedParamPattern {
        loc: SourceLocation,
    },
}

impl fmt::Display for ScanError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ScanError::DuplicateName { name, first, second } => write!(
                f,
                "duplicate prebindgen name `{}`: first at {}, second at {}",
                name, first, second
            ),
            ScanError::ConflictingFunctionIntent { name } => write!(
                f,
                "function `{}` cannot be both declared and ignored",
                name
            ),
            ScanError::ConflictingTypeIntent { key } => write!(
                f,
                "type `{}` cannot be both declared and ignored",
                key
            ),
            ScanError::DisallowedImplTrait { ty, loc } => write!(
                f,
                "`impl Trait` is not allowed at {}: `{}` (only `impl Fn(...) + Send + Sync + 'static` is supported)",
                loc, ty
            ),
            ScanError::UnsupportedReceiver { loc } => {
                write!(f, "method receiver (`self`) parameters are not supported at {}", loc)
            }
            ScanError::UnsupportedParamPattern { loc } => {
                write!(f, "non-ident parameter pattern is not supported at {}", loc)
            }
        }
    }
}

impl std::error::Error for ScanError {}

/// Combined error surfaced by [`Registry::write_rust`].
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

/// Adapter declaration intent normalized once per pipeline run.
struct DeclaredItems {
    functions: HashSet<syn::Ident>,
    ignored_functions: HashSet<syn::Ident>,
    accessors: HashSet<syn::Ident>,
    method_receivers: HashMap<syn::Ident, TypeKey>,
    types: HashSet<TypeKey>,
    ignored_types: HashSet<TypeKey>,
    /// `None` = the adapter has no const declaration mechanism (all consts
    /// re-emitted verbatim, no scan, no warnings) — see
    /// [`Prebindgen::declared_consts`].
    consts: Option<HashSet<syn::Ident>>,
    ignored_consts: HashSet<syn::Ident>,
    /// Adapter-required extra output types (no `#[prebindgen]` item to
    /// scan — e.g. expression-constant value types); see
    /// [`Prebindgen::required_output_types`].
    required_output_types: Vec<syn::Type>,
}

impl DeclaredItems {
    fn from_adapter<E, M>(adapter: &E) -> Result<Self, ScanError>
    where
        E: Prebindgen<Metadata = M>,
    {
        let declared = Self {
            functions: adapter.declared_functions(),
            ignored_functions: adapter.ignored_functions(),
            accessors: adapter.accessor_functions(),
            method_receivers: adapter.method_receivers(),
            types: adapter.declared_types(),
            ignored_types: adapter.ignored_types(),
            consts: adapter.declared_consts(),
            ignored_consts: adapter.ignored_consts(),
            required_output_types: adapter.required_output_types(),
        };

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

impl<M> Registry<M> {
    /// Construct a `Registry` by indexing a stream of source items.
    ///
    /// Callers feed any `(syn::Item, SourceLocation)` iterator — typically
    /// `source.items_all()`, `source.items_except_groups(...)`, or a
    /// hand-rolled filter chain — so item-level selection happens upstream
    /// of the registry rather than inside it.
    ///
    /// This step only populates the item maps (`functions`, `structs`,
    /// `enums`, `consts`, `passthrough`). Signature/body scanning that
    /// drives type-resolution requirements happens later, in
    /// [`Self::scan_declared`], and is gated on what the language adapter
    /// has explicitly declared. Items that are never declared remain in
    /// the registry but never drive type resolution and never emit.
    pub fn from_items<I>(items: I) -> Result<Self, ScanError>
    where
        I: IntoIterator<Item = (syn::Item, SourceLocation)>,
    {
        let mut registry = Registry::default();
        for (item, loc) in items {
            registry.index_item(item, loc)?;
        }
        Ok(registry)
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
        let declared = DeclaredItems::from_adapter(ext)?;
        self.scan_declared_items(&declared)
    }

    fn scan_declared_items(&mut self, declared: &DeclaredItems) -> Result<(), ScanError> {
        // Scan declared functions.
        for ident in &declared.functions {
            if let Some((item_fn, loc)) = self.functions.get(ident).cloned() {
                self.scan_fn_signature(&item_fn, &loc)?;
            } else {
                println!(
                    "cargo:warning=prebindgen: declared function `{}` not found among #[prebindgen] items",
                    ident
                );
            }
        }

        for ident in &declared.ignored_functions {
            if !self.functions.contains_key(ident) {
                println!(
                    "cargo:warning=prebindgen: ignored function `{}` not found among #[prebindgen] items",
                    ident
                );
            }
        }

        // Scan declared consts (only when the adapter has a const
        // declaration mechanism): a const is a nullary source of its type,
        // so the type is required in the output direction only.
        if let Some(decl_consts) = &declared.consts {
            for ident in decl_consts {
                if let Some((item_const, loc)) = self.consts.get(ident).cloned() {
                    self.ensure_entry(Direction::Output, &item_const.ty, true, &loc);
                } else {
                    println!(
                        "cargo:warning=prebindgen: declared const `{}` not found among #[prebindgen] items",
                        ident
                    );
                }
            }
            for ident in &declared.ignored_consts {
                if !self.consts.contains_key(ident) {
                    println!(
                        "cargo:warning=prebindgen: ignored const `{}` not found among #[prebindgen] items",
                        ident
                    );
                }
            }
        }

        // Adapter-required extra output types — synthesized values with no
        // `#[prebindgen]` item behind them (e.g. expression constants).
        for ty in &declared.required_output_types {
            self.ensure_entry(Direction::Output, ty, true, &SourceLocation::default());
        }

        // Scan declared types.
        for key in &declared.types {
            let ty = key.to_type();
            let mut matched = false;
            if let Some(ident) = bare_path_ident(&ty) {
                if let Some((s, loc)) = self.structs.get(&ident).cloned() {
                    self.scan_struct(&s, &loc)?;
                    self.ensure_entry(Direction::Input, &ty, true, &loc);
                    self.ensure_entry(Direction::Output, &ty, true, &loc);
                    matched = true;
                } else if let Some((e, loc)) = self.enums.get(&ident).cloned() {
                    self.scan_enum(&e, &loc)?;
                    self.ensure_entry(Direction::Input, &ty, true, &loc);
                    self.ensure_entry(Direction::Output, &ty, true, &loc);
                    matched = true;
                }
            }
            if !matched {
                // Declared type without an indexed body (e.g.
                // `ptr_class(ZKeyExpr<'static>)` on a re-exported
                // foreign type). Still mark required so the resolver
                // tries to produce a converter for it.
                let loc = self.type_locations.get(key).cloned().unwrap_or_default();
                self.ensure_entry(Direction::Input, &ty, true, &loc);
                self.ensure_entry(Direction::Output, &ty, true, &loc);
            }
        }

        for key in &declared.ignored_types {
            let ty = key.to_type();
            let matched = bare_path_ident(&ty).is_some_and(|ident| {
                self.structs.contains_key(&ident) || self.enums.contains_key(&ident)
            });
            if !matched {
                println!(
                    "cargo:warning=prebindgen: ignored type `{}` not found among #[prebindgen] items",
                    key.as_str()
                );
            }
        }

        // Warn about indexed items that the adapter never claimed.
        let mut skipped_fns: Vec<String> = self
            .functions
            .keys()
            .filter(|k| {
                !declared.functions.contains(*k) && !declared.ignored_functions.contains(*k)
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
        for ident in self.structs.keys() {
            let key = TypeKey::parse(&ident.to_string());
            if !declared.types.contains(&key) && !declared.ignored_types.contains(&key) {
                skipped_types.push(ident.to_string());
            }
        }
        for ident in self.enums.keys() {
            let key = TypeKey::parse(&ident.to_string());
            if !declared.types.contains(&key) && !declared.ignored_types.contains(&key) {
                skipped_types.push(ident.to_string());
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
                .consts
                .keys()
                // Unnamed consts (`const _`, e.g. the injected feature
                // guard) are infrastructure: not declarable, always emitted
                // verbatim — never a skip.
                .filter(|k| {
                    *k != "_" && !decl_consts.contains(*k) && !declared.ignored_consts.contains(*k)
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

    /// True iff the key was scanned as a top-level fn-signature input type.
    pub fn is_required_input_at_scan(&self, key: &TypeKey) -> bool {
        self.required_inputs_scan.contains(key)
    }
    pub fn is_required_output_at_scan(&self, key: &TypeKey) -> bool {
        self.required_outputs_scan.contains(key)
    }

    /// Direction-indexed read access to the type-resolution tables.
    pub(crate) fn type_table(&self, dir: Direction) -> &HashMap<TypeKey, Option<TypeEntry<M>>> {
        match dir {
            Direction::Input => &self.input_types,
            Direction::Output => &self.output_types,
        }
    }

    /// Direction-indexed mutable access to the type-resolution tables.
    pub(crate) fn type_table_mut(
        &mut self,
        dir: Direction,
    ) -> &mut HashMap<TypeKey, Option<TypeEntry<M>>> {
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
        self.type_table(Direction::Input).get(&key)?.as_ref()
    }

    /// Look up the resolved output entry for `ty`. See [`Self::input_entry`].
    pub fn output_entry(&self, ty: &syn::Type) -> Option<&TypeEntry<M>> {
        let key = TypeKey::from_type(ty);
        self.type_table(Direction::Output).get(&key)?.as_ref()
    }

    /// Register `ty` (and its nested positions) as a required **input** so
    /// the resolver produces a converter for it. Used by
    /// [`crate::api::core::expand`] to pull in the leaf types a fold needs.
    pub(crate) fn require_input(&mut self, ty: &syn::Type, loc: &SourceLocation) {
        // Leaf/expansion types are concrete (no disallowed `impl Trait`), so
        // the recursive registration cannot fail here.
        let _ = self.register_type_recursive(Direction::Input, ty, true, loc);
    }

    /// Register `ty` (and its nested positions) as a required **output** so the
    /// resolver produces a converter for it. The output-side peer of
    /// [`Self::require_input`]; used by [`crate::api::core::unfold`] to pull in
    /// the leaf types a decomposition delivers.
    pub(crate) fn require_output(&mut self, ty: &syn::Type, loc: &SourceLocation) {
        let _ = self.register_type_recursive(Direction::Output, ty, true, loc);
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
        self.required_outputs_scan.remove(&TypeKey::from_type(ty));
    }

    fn index_item(&mut self, item: syn::Item, loc: SourceLocation) -> Result<(), ScanError> {
        match item {
            syn::Item::Fn(f) => {
                self.check_no_duplicate(&f.sig.ident, &loc)?;
                self.functions.insert(f.sig.ident.clone(), (f, loc));
                Ok(())
            }
            syn::Item::Struct(s) => {
                self.check_no_duplicate(&s.ident, &loc)?;
                self.structs.insert(s.ident.clone(), (s, loc));
                Ok(())
            }
            syn::Item::Enum(e) => {
                self.check_no_duplicate(&e.ident, &loc)?;
                self.enums.insert(e.ident.clone(), (e, loc));
                Ok(())
            }
            syn::Item::Const(c) => {
                self.check_no_duplicate(&c.ident, &loc)?;
                self.consts.insert(c.ident.clone(), (c, loc));
                Ok(())
            }
            other => {
                self.passthrough.push((other, loc));
                Ok(())
            }
        }
    }

    fn check_no_duplicate(&self, name: &syn::Ident, loc: &SourceLocation) -> Result<(), ScanError> {
        if let Some(first) = self.first_seen_loc(name) {
            return Err(ScanError::DuplicateName {
                name: name.clone(),
                first,
                second: loc.clone(),
            });
        }
        Ok(())
    }

    fn first_seen_loc(&self, name: &syn::Ident) -> Option<SourceLocation> {
        if let Some((_, loc)) = self.functions.get(name) {
            return Some(loc.clone());
        }
        if let Some((_, loc)) = self.structs.get(name) {
            return Some(loc.clone());
        }
        if let Some((_, loc)) = self.enums.get(name) {
            return Some(loc.clone());
        }
        if let Some((_, loc)) = self.consts.get(name) {
            return Some(loc.clone());
        }
        None
    }

    fn scan_fn_signature(
        &mut self,
        f: &syn::ItemFn,
        loc: &SourceLocation,
    ) -> Result<(), ScanError> {
        // Mechanical: register every fn-signature type as the user wrote it.
        // No semantic transformations (no &T→T strip, no ZResult<T>→T strip,
        // no skip for () / ZResult<()>). The adapter handles structural
        // wrappers; propagation through `subs` then marks transitive deps
        // (e.g. &Foo's `&_` converter returns subs=[Foo], so Foo becomes
        // required).
        for input in &f.sig.inputs {
            match input {
                syn::FnArg::Receiver(_) => {
                    return Err(ScanError::UnsupportedReceiver { loc: loc.clone() });
                }
                syn::FnArg::Typed(pt) => {
                    if !matches!(&*pt.pat, syn::Pat::Ident(_)) {
                        return Err(ScanError::UnsupportedParamPattern { loc: loc.clone() });
                    }
                    self.register_type_recursive(Direction::Input, &pt.ty, true, loc)?;
                }
            }
        }
        let ret_ty: syn::Type = match &f.sig.output {
            syn::ReturnType::Default => syn::parse_quote!(()),
            syn::ReturnType::Type(_, ty) => (**ty).clone(),
        };
        self.register_type_recursive(Direction::Output, &ret_ty, true, loc)?;
        Ok(())
    }

    fn scan_struct(&mut self, s: &syn::ItemStruct, loc: &SourceLocation) -> Result<(), ScanError> {
        // The struct itself can appear in either direction.
        let ty: syn::Type = syn::parse_str(&s.ident.to_string()).expect("ident is a valid type");
        self.ensure_entry(Direction::Input, &ty, false, loc);
        self.ensure_entry(Direction::Output, &ty, false, loc);

        if let syn::Fields::Named(named) = &s.fields {
            for field in &named.named {
                self.register_type_recursive(Direction::Input, &field.ty, false, loc)?;
                self.register_type_recursive(Direction::Output, &field.ty, false, loc)?;
            }
        }
        Ok(())
    }

    fn scan_enum(&mut self, e: &syn::ItemEnum, loc: &SourceLocation) -> Result<(), ScanError> {
        let ty: syn::Type = syn::parse_str(&e.ident.to_string()).expect("ident is a valid type");
        self.ensure_entry(Direction::Input, &ty, false, loc);
        self.ensure_entry(Direction::Output, &ty, false, loc);

        for variant in &e.variants {
            for field in &variant.fields {
                self.register_type_recursive(Direction::Input, &field.ty, false, loc)?;
                self.register_type_recursive(Direction::Output, &field.ty, false, loc)?;
            }
        }
        Ok(())
    }

    /// Register `ty` as an entry in the given direction, then recurse into
    /// every nested position. `top_required` applies only to `ty` itself;
    /// nested positions are always recorded as not-required.
    fn register_type_recursive(
        &mut self,
        dir: Direction,
        ty: &syn::Type,
        top_required: bool,
        loc: &SourceLocation,
    ) -> Result<(), ScanError> {
        let mut visited: HashSet<TypeKey> = HashSet::new();
        self.register_type_inner(dir, ty, top_required, loc, &mut visited)
    }

    fn register_type_inner(
        &mut self,
        dir: Direction,
        ty: &syn::Type,
        is_top: bool,
        loc: &SourceLocation,
        visited: &mut HashSet<TypeKey>,
    ) -> Result<(), ScanError> {
        // Reject `impl Trait` except `impl Fn(...) + Send + Sync + 'static`.
        if let syn::Type::ImplTrait(it) = ty {
            if extract_fn_trait_args(ty).is_none() {
                return Err(ScanError::DisallowedImplTrait {
                    ty: it.to_token_stream().to_string(),
                    loc: loc.clone(),
                });
            }
        }

        let key = TypeKey::from_type(ty);
        if !visited.insert(key.clone()) {
            return Ok(()); // cycle guard
        }

        self.ensure_entry(dir, ty, is_top, loc);

        for (child_dir, sub) in self.immediate_edges(dir, ty) {
            self.register_type_inner(child_dir, &sub, false, loc, visited)?;
        }
        Ok(())
    }

    fn ensure_entry(
        &mut self,
        dir: Direction,
        ty: &syn::Type,
        required: bool,
        loc: &SourceLocation,
    ) {
        let key = TypeKey::from_type(ty);
        let table = self.type_table_mut(dir);
        table.entry(key.clone()).or_insert(None);
        if required {
            match dir {
                Direction::Input => self.required_inputs_scan.insert(key.clone()),
                Direction::Output => self.required_outputs_scan.insert(key.clone()),
            };
        }
        self.type_locations
            .entry(key)
            .or_insert_with(|| loc.clone());
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
        if let Some(name) = bare_path_ident(ty) {
            if let Some((s, _)) = self.structs.get(&name) {
                if let syn::Fields::Named(named) = &s.fields {
                    for field in &named.named {
                        out.push((dir, field.ty.clone()));
                    }
                }
            }
            if let Some((e, _)) = self.enums.get(&name) {
                for variant in &e.variants {
                    for field in &variant.fields {
                        out.push((dir, field.ty.clone()));
                    }
                }
            }
        }
        out
    }

    /// One-shot: resolve every required type using an adapter, then write the
    /// generated Rust bindings file. The single public entry point for
    /// language-specific binding generation — language-agnostic because
    /// `ext` is any [`crate::api::core::prebindgen::Prebindgen`] impl
    /// whose `Metadata` matches this registry's `M` parameter.
    pub fn write_rust<E>(
        &mut self,
        ext: &E,
        out_path: impl AsRef<std::path::Path>,
    ) -> Result<std::path::PathBuf, WriteRustError>
    where
        E: Prebindgen<Metadata = M>,
        M: Clone + Default,
    {
        let declared = DeclaredItems::from_adapter(ext)?;
        self.scan_declared_items(&declared)?;
        self.apply_adapter_plans(ext, &declared)?;
        crate::api::core::resolve::resolve(self, ext)?;
        Ok(crate::api::core::write::write_rust(self, ext, out_path)?)
    }

    fn apply_adapter_plans<E>(
        &mut self,
        ext: &E,
        declared: &DeclaredItems,
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
                exp,
                &declared.functions,
                &declared.accessors,
                &declared.method_receivers,
            )?;
        }
        if let Some(dec) = ext.deconstructors() {
            crate::api::core::unfold::apply(self, dec, &declared.functions, &declared.accessors)?;
        }
        // Synthesized by-value `data_class` decompositions: build the leaves
        // (immutable borrow), then wire them into fixed-builder plans.
        let value_decons = ext.value_struct_decons(self);
        if !value_decons.is_empty() {
            crate::api::core::unfold::apply_value_structs(self, value_decons, &declared.functions)?;
        }
        // Single-leaf `Vec<T>`/`&[T]` whole-element folds — the dual of the
        // `data_class` folds above, for String / value-blob / handle elements
        // (so the list is built on the foreign side, not via a Rust ArrayList).
        let leaf_elements = ext.leaf_vec_fold_elements(self);
        if !leaf_elements.is_empty() {
            crate::api::core::unfold::apply_leaf_vec_folds(
                self,
                leaf_elements,
                &declared.functions,
            )?;
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

/// If `ty` is `impl Fn(T1, T2, ...) + Send + Sync + 'static`, return the
/// `Fn` argument types in declaration order. Otherwise None.
pub fn extract_fn_trait_args(ty: &syn::Type) -> Option<Vec<syn::Type>> {
    let syn::Type::ImplTrait(it) = ty else {
        return None;
    };
    let mut args: Option<Vec<syn::Type>> = None;
    let mut has_send = false;
    let mut has_sync = false;
    let mut has_static = false;
    for bound in &it.bounds {
        match bound {
            syn::TypeParamBound::Trait(tb) => {
                let last = tb.path.segments.last()?;
                let name = last.ident.to_string();
                match name.as_str() {
                    "Fn" => {
                        let syn::PathArguments::Parenthesized(p) = &last.arguments else {
                            return None;
                        };
                        args = Some(p.inputs.iter().cloned().collect());
                    }
                    "Send" => has_send = true,
                    "Sync" => has_sync = true,
                    _ => return None,
                }
            }
            syn::TypeParamBound::Lifetime(lt) if lt.ident == "static" => has_static = true,
            _ => return None,
        }
    }
    if has_send && has_sync && has_static {
        args
    } else {
        None
    }
}

#[cfg(test)]
mod tests;
