//! `Prebindgen` — the single extension point for the new pipeline.
//!
//! One method per `#[prebindgen]` item kind (`on_function`, `on_struct`,
//! `on_enum`, `on_const`) returning the wrapper Rust tokens to emit, plus a
//! pair of structural converter methods split by direction:
//!
//! * Input  (wire → rust): `on_input_type`
//! * Output (rust → wire): `on_output_type`
//!
//! Each converter method returns `Some(ConverterImpl)` if the adapter handles
//! the type, or `None` to defer. Deferred types are retried by the fixed-point
//! resolver and ultimately reported as "unresolved required type" errors if no
//! converter can fill the cell.
//!
//! `ConverterImpl::function` is the **complete** Rust function for the
//! converter — signature, body, attributes, lifetimes. The adapter owns
//! 100% of the shape. Other code that wants to call this converter reads
//! the name from `function.sig.ident`; the wire form from `destination`.

use std::collections::HashSet;

use proc_macro2::TokenStream;

use crate::api::core::{
    niches::Niches,
    registry::{Registry, TypeKey},
};

/// One link in a converter's [stage chain](`ConverterImpl::pre_stages`) —
/// a value-inspecting step that sits between the rust value the
/// `#[prebindgen]` fn yields/receives and the wire-facing
/// [`ConverterImpl::function`].
///
/// Each stage is a fallible `In → Result<Out, Err>` function. The core
/// pipeline only ever emits and de-duplicates [`Self::function`]; how a
/// stage's `Err` arm is surfaced to the foreign side — throw an exception,
/// return an error code, set `errno`, … — is entirely up to the
/// destination-language adapter and is described by [`Self::metadata`].
#[derive(Clone)]
pub struct Stage<M = ()> {
    /// Complete function definition for this stage. Same shape as
    /// [`ConverterImpl::function`] but typed for this stage's own `In →
    /// Out` and own error type.
    pub function: syn::ItemFn,
    /// Adapter-specific extras for this stage — same [`Metadata`] type as
    /// the owning converter ([`ConverterImpl::metadata`]). The core never
    /// inspects this; the adapter's emitter reads it to decide how the
    /// stage's `Err` arm is surfaced (e.g. a JNI adapter stores the JVM
    /// exception class and `throw_*` fn to call here; a C adapter might
    /// store the error-code sentinel). Defaults to `()`.
    ///
    /// [`Metadata`]: Prebindgen::Metadata
    pub metadata: M,
}

/// Result of resolving one converter — the wire (destination) type the rest
/// of the registry sees, plus the complete generated function.
///
/// Invariant: `function.sig.ident` MUST be a deterministic function of the
/// `(rust_type, destination)` pair so that callers of this converter — both
/// other generated converters from the same adapter and any hand-written code
/// that knows the convention — can compute or look up the name.
#[derive(Clone)]
pub struct ConverterImpl<M = ()> {
    /// Wire/destination type. Other converters that ask "what's the wire
    /// form of this rust type?" read this. The actual function may return
    /// a wrapped form (e.g. an adapter's own `Result`-like envelope) — that
    /// is the adapter's internal calling convention; `destination` is the
    /// value the wire carries on success.
    pub destination: syn::Type,
    /// Complete function definition for the **wire-facing** stage. The
    /// adapter owns the parameter list, return type, `unsafe`/`pub`
    /// modifiers, lifetime parameters, and any attribute annotations.
    /// For input direction this is the FIRST stage in execution order
    /// (it takes the wire); for output direction this is the LAST stage
    /// (it produces the wire).
    pub function: syn::ItemFn,
    /// **Rust-side** stages that compose with [`Self::function`] to form
    /// the full conversion chain. Default empty — a 1-stage converter
    /// is just `function`.
    ///
    /// Order is rust-side-first → function-side-last. Concretely:
    /// * **Input** (wire → rust): chain runs `wire → function →
    ///   pre_stages[0] → pre_stages[1] → … → pre_stages[N-1] → rust`.
    /// * **Output** (rust → wire): chain runs `rust → pre_stages[N-1] →
    ///   … → pre_stages[1] → pre_stages[0] → function → wire`.
    ///
    /// Each stage is fallible; how its `Err` arm is surfaced is adapter
    /// specific and carried in [`Stage::metadata`].
    pub pre_stages: Vec<Stage<M>>,
    /// Bit-patterns the wire type can represent but this converter never
    /// produces (output) and rejects (input). Wrapper handlers like
    /// `Option<_>` consume one slot for their own discriminant and
    /// re-export the rest — see [`Niches`] for the cascade model.
    /// Default is empty (no niche optimisation).
    pub niches: Niches,
    /// Adapter-specific extras carried alongside the converter. Filled by
    /// the same handler that produces `destination` / `function` /
    /// `niches`, copied through into `TypeEntry::metadata` by the resolver,
    /// and read by the adapter's language-side emitters. Set this where you
    /// build the converter, not in a side channel.
    pub metadata: M,
    /// Inner types this converter composed from — the types whose
    /// `input_entry`/`output_entry` the adapter looked up to build a wrapper
    /// (`Option<X>` → `[X]`, `Result<T,E>` → `[T, E]`, `&T` → `[&T]`). Empty
    /// for a terminal converter (scalar, opaque handle, string) and for
    /// `dispatch_fn_input` (callback args are cross-direction — their
    /// required-ness flows through `Registry::immediate_edges`, not here). The
    /// resolver copies these into `TypeEntry::subs`, which `propagate_required`
    /// walks to mark reachable types required.
    pub subs: Vec<syn::Type>,
}

/// The single extension point of the pipeline: implement this trait once per
/// **destination language** (C/cbindgen, JNI/Kotlin, Swift, Python, …) to teach
/// the language-agnostic [`Registry`] how that language represents Rust types
/// on the wire and what wrapper code to emit.
///
/// The trait has no language-specific concepts of its own. Two jobs:
/// * **Type resolution.** The resolver asks `on_input_type` / `on_output_type`
///   for the wire form of each required type and gets back a [`ConverterImpl`]
///   (a generated converter fn + its wire type); these fill
///   `Registry::input_types` / `output_types`.
/// * **Per-item emission.** The file emitter calls `on_function` / `on_struct`
///   / `on_enum` / `on_const` to produce the per-item wrapper code for the
///   destination language.
///
/// Anything language-specific the rest of the pipeline must carry — a JNI
/// adapter's Kotlin class names and exception info, a C adapter's header
/// names, etc. — rides in [`Self::Metadata`], an opaque type the adapter
/// chooses. It is set in each `ConverterImpl::metadata`, propagated by the
/// resolver into `TypeEntry::metadata`, and read back by the adapter's own
/// emitter. Adapters that need no extras leave it at the default `()`.
pub trait Prebindgen {
    /// Adapter-specific extras every resolved converter carries. The
    /// resolver copies this from each `ConverterImpl` it accepts into
    /// the matching `TypeEntry`, so emitter code reads metadata off
    /// the registry rather than through a parallel side channel.
    type Metadata: Clone + Default;

    /// Rust items the adapter's emitted converters depend on (helper
    /// structs, type aliases, runtime-support code). Emitted at the top
    /// of the destination file, before all auto-generated converters.
    ///
    /// Default: none. Wrapper adapters that compose a base adapter should
    /// forward to or extend the base's `prerequisites()`. The resolved
    /// `registry` is supplied so prerequisites can be gated on what the
    /// (feature-aware) scan actually contains — e.g. emitting a
    /// per-opaque-handle item only for handles a scanned `#[prebindgen]`
    /// fn references.
    fn prerequisites(&self, _registry: &Registry<Self::Metadata>) -> Vec<syn::Item> {
        Vec::new()
    }

    /// Constructor-expansion declarations for this adapter, or `None` if it
    /// doesn't support expansion. Consulted by `write_rust` after scanning and
    /// before resolution: each `.expand` is resolved into a
    /// [`crate::api::core::expand::FoldPlan`] on the registry and its leaf
    /// types are registered as required inputs.
    ///
    /// Default: `None`.
    fn expansions(&self) -> Option<&crate::api::core::expand::Expansions> {
        None
    }

    /// Output-expansion (deconstructor / converter) declarations for this
    /// adapter, or `None` if it doesn't support output expansion. Consulted by
    /// `write_rust` after `expansions` and before resolution: each
    /// `.deconstruct_output` / `.convert_output` is resolved into a
    /// [`crate::api::core::unfold::UnfoldPlan`] on the registry and its leaf
    /// types are registered as required outputs.
    ///
    /// Default: `None`.
    fn deconstructors(&self) -> Option<&crate::api::core::unfold::Deconstructors> {
        None
    }

    /// Synthesized by-value `data_class` decompositions for this adapter. Each
    /// names a value struct and its field-access leaves (the adapter knows the
    /// per-field encoding — projections, enums, nested classes — so it builds
    /// the leaves; the registry is available so field converters resolve).
    /// Consulted by `write_rust` right after [`Self::deconstructors`]: each is
    /// wired by [`crate::api::core::unfold::apply_value_structs`] into a
    /// fixed-builder [`crate::api::core::unfold::UnfoldPlan`] for every function
    /// that returns / callbacks the struct, so it crosses the boundary as
    /// decoupled leaves (reassembled on the foreign side) instead of a Java
    /// object built on the Rust side.
    ///
    /// Default: empty.
    fn value_struct_decons(
        &self,
        _registry: &Registry<Self::Metadata>,
    ) -> Vec<crate::api::core::unfold::ValueDecon> {
        Vec::new()
    }

    // ── Declaration queries ────────────────────────────────────────

    /// Idents of `#[prebindgen]` functions the adapter claims for emission.
    /// Anything not in this set is left in the registry's `functions`
    /// map but never scanned for type requirements and never emitted —
    /// the build prints a `cargo:warning=` line per skip.
    ///
    /// Default: empty (strict allowlist; an adapter with no declarations
    /// emits nothing for functions).
    fn declared_functions(&self) -> HashSet<syn::Ident> {
        HashSet::new()
    }

    /// Subset of [`Self::declared_functions`] declared as **read accessors**:
    /// the parameter composer (constructor expansion) is never applied to them,
    /// and a decomposer record may only reference one. Adapters without the
    /// concept return empty (then no fn is treated as an accessor).
    ///
    /// Default: empty.
    fn accessor_functions(&self) -> HashSet<syn::Ident> {
        HashSet::new()
    }

    /// `#[prebindgen]` functions declared as **methods** of a class, mapping the
    /// fn ident to its class's canonical [`TypeKey`]. A method's first parameter
    /// of that class type is the receiver and is excluded from input-flattening
    /// (it is bound to `this`); the remaining parameters flatten normally.
    /// Adapters without the concept return empty.
    ///
    /// Default: empty.
    fn method_receivers(&self) -> std::collections::HashMap<syn::Ident, TypeKey> {
        std::collections::HashMap::new()
    }

    /// Idents of `#[prebindgen]` functions the adapter explicitly knows about but
    /// intentionally does not emit. These suppress the registry's
    /// "skipping undeclared" warning while still leaving the items out of the
    /// scan and write pipelines.
    ///
    /// Default: empty.
    fn ignored_functions(&self) -> HashSet<syn::Ident> {
        HashSet::new()
    }

    /// Canonical keys of types (structs / enums) the adapter claims for
    /// emission. Matched against `Registry::structs` and `Registry::enums`
    /// by bare-ident lookup. Anything not in this set is left in the
    /// registry but never scanned for body type requirements and never
    /// emitted — the build prints a `cargo:warning=` line per skip.
    ///
    /// Default: empty (strict allowlist).
    fn declared_types(&self) -> HashSet<TypeKey> {
        HashSet::new()
    }

    /// Canonical keys of types the adapter explicitly knows about but
    /// intentionally does not emit. These suppress the registry's
    /// "skipping undeclared" warning while still leaving the items out of the
    /// scan and write pipelines.
    ///
    /// Default: empty.
    fn ignored_types(&self) -> HashSet<TypeKey> {
        HashSet::new()
    }

    /// Final post-processing pass applied to every emitted item right
    /// before write. Default: no-op.
    ///
    /// Use this for cross-cutting transforms that would otherwise have
    /// to be remembered at every individual emit site — e.g. qualifying
    /// bare type references against a source module so the emitted
    /// converter bodies compile in the binding crate's scope. Walks the
    /// entire AST, not just signatures, so type ascriptions and casts
    /// inside function bodies are covered.
    fn post_process_item(&self, _item: &mut syn::Item) {}

    // ── Item methods ───────────────────────────────────────────────

    /// Wrap a `#[prebindgen]` fn into the destination-language wrapper
    /// (e.g. JNI `extern "C"` fn).
    fn on_function(&self, f: &syn::ItemFn, registry: &Registry<Self::Metadata>) -> TokenStream;

    /// Per-struct emission. Typically empty for languages that get
    /// everything they need from auto-generated converters.
    fn on_struct(&self, s: &syn::ItemStruct, registry: &Registry<Self::Metadata>) -> TokenStream;

    /// Per-enum emission.
    fn on_enum(&self, e: &syn::ItemEnum, registry: &Registry<Self::Metadata>) -> TokenStream;

    /// Per-const emission. Default: pass-through.
    fn on_const(&self, c: &syn::ItemConst, _registry: &Registry<Self::Metadata>) -> TokenStream {
        use quote::ToTokens;
        c.to_token_stream()
    }

    // ── Structural type resolution (the converter-resolution surface) ──

    /// Resolve the **input** (wire → rust) converter for `ty`. The adapter
    /// inspects `ty`'s outermost structure itself (peeling with
    /// `core::types_util` helpers) and returns either a *terminal* converter
    /// (`ConverterImpl::subs` empty) or a *wrapper* that looked up inner
    /// converters via [`Registry::input_entry`] (listing those inners in
    /// `subs`). Return `None` to **defer** — when an inner isn't resolved yet
    /// the resolver retries on a later fixed-point iteration.
    fn on_input_type(
        &self,
        ty: &syn::Type,
        registry: &Registry<Self::Metadata>,
    ) -> Option<ConverterImpl<Self::Metadata>>;

    /// Resolve the **output** (rust → wire) converter for `ty`. The dual of
    /// [`Self::on_input_type`]; same terminal-vs-wrapper / `subs` / defer
    /// contract, looking up inners via [`Registry::output_entry`].
    fn on_output_type(
        &self,
        ty: &syn::Type,
        registry: &Registry<Self::Metadata>,
    ) -> Option<ConverterImpl<Self::Metadata>>;

    /// Build the wrapper converter for an
    /// `impl Fn(args...) + Send + Sync + 'static` parameter, given the
    /// already-extracted arg types in declaration order. The resolver calls
    /// this only after [`Self::on_input_type`] returns `None`, so wrappers that
    /// need custom callback dispatch can intercept earlier and skip this path.
    ///
    /// `args` are the rust-side argument types as they appear in the source
    /// signature. Note that callback args flow inverse to the callback
    /// parameter itself: the callback parameter is *input*, but its args are
    /// produced by the rust side and consumed by the foreign side, so they are
    /// *output* direction for converter resolution. The framework handles this
    /// direction-flip at registration time (`register_type_inner` in
    /// `core::registry`), so implementations of this method should look up
    /// already-registered *output* converters for each arg type. The returned
    /// `ConverterImpl::subs` should be empty — the callback-arg required-ness
    /// flows through that direction-flipped `immediate_edges`, not `subs`.
    ///
    /// Default: `None`. Adapters that support `impl Fn` callbacks override this.
    fn dispatch_fn_input(
        &self,
        args: &[syn::Type],
        registry: &Registry<Self::Metadata>,
    ) -> Option<ConverterImpl<Self::Metadata>> {
        let _ = (args, registry);
        None
    }
}
