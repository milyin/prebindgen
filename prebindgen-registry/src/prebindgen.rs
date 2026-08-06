//! `Prebindgen` — what a generator still hands the emitter.
//!
//! One method per `#[prebindgen]` item kind (`on_function`, `on_struct`,
//! `on_enum`, `on_const`) returning the wrapper Rust tokens to emit, plus the
//! items they depend on (`prerequisites`), a cross-cutting rewrite
//! (`post_process_item`) and two invariant checks.
//!
//! **Conversion is not here.** A generator builds those itself, against the
//! demand `RegistryBuilder::crossings` hands it, and gives them back through
//! `RegistryBuilder::convert_with` — so there is no `on_input_type`, no deferral, and no
//! fixed-point loop retrying until it converges.
//!
//! [`ConverterImpl::function`] is the **complete** Rust function for a
//! converter — signature, body, attributes, lifetimes. The generator owns 100%
//! of the shape. Callers read the name from `function.sig.ident` and the wire
//! form from `destination`.

use proc_macro2::TokenStream;

use crate::{niches::Niches, registry::Registry};

/// A shared predicate over an item name, as used by
/// `Prebindgen`'s ignore hooks (bulk ignores keyed on a naming
/// family rather than an exact ident).
pub type NamePredicate = std::sync::Arc<dyn Fn(&str) -> bool + Send + Sync>;

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
    /// a callback's own converter (callback args are cross-direction — their
    /// required-ness flows through the registry's type-graph edges, not here). The
    /// resolver copies these into `TypeEntry::subs`, which `propagate_required`
    /// walks to mark reachable types required.
    ///
    /// **Identities, not spellings.** They are looked up and walked, never
    /// emitted — [`TypeEntry::subs`](crate::registry::TypeEntry::subs)
    /// was already `Vec<TypeKey>` and the resolver keyed these on arrival, so
    /// the spelling existed only to be converted. An adapter that composed the
    /// inner type keys it (`TypeKey::from_type`); one that read it off the model
    /// asks the reading (`TypeRef::key`), and names no escape to do it.
    pub subs: Vec<crate::registry::TypeKey>,
}

/// The single extension point of the pipeline: implement this trait once per
/// **destination language** (C/cbindgen, JNI/Kotlin, Swift, Python, …) to teach
/// the language-agnostic [`Registry`] how that language represents Rust types
/// on the wire and what wrapper code to emit.
///
/// The trait has no language-specific concepts of its own, and — since the
/// registry stopped asking it questions — one job left: **per-item emission**.
/// The file emitter calls `on_function` / `on_struct` / `on_enum` / `on_const`
/// to produce the per-item wrapper code, plus `prerequisites` and
/// `post_process_item` around them and the two `validate` hooks for
/// adapter invariants.
///
/// What used to be here and is not any more: which items to build, how
/// composites decompose, and the wire form of each type. A generator states the
/// first two into the builder (`RegistryBuilder::export`,
/// `RegistryBuilder::decompose`)
/// and answers the third by filling `RegistryBuilder::crossings` — so nothing in
/// core calls back to ask. Moving emission out too is what would delete this
/// trait entirely (prebindgen#251 phase E).
///
/// Anything language-specific the rest of the pipeline must carry — a JNI
/// adapter's Kotlin class names and exception info, a C adapter's header
/// names, etc. — rides in [`Self::Metadata`], an opaque type the adapter
/// chooses. It is set in each `ConverterImpl::metadata`, propagated by the
/// resolver into `TypeEntry::metadata`, and read back by the adapter's own
/// emitter. Adapters that need no extras leave it at the default `()`.
///
/// # The rule an adapter must obey
///
/// "Classify off [`kind`](crate::flat::TypeRef::kind), spell off the syntax"
/// tells an adapter where to get each fact. It is silent on the question
/// adapters actually face — what the **destination language** ends up seeing.
/// That one has its own answer:
///
/// > **Same `kind` ⇒ same destination-language type.** The *wire* is the
/// > generator's to choose, and may differ per spelling.
///
/// The weaker-sounding half is the important one. It is tempting to write "same
/// `kind` ⇒ same wire", and that is **false** — prebindgen's own adapters
/// violate it deliberately:
///
/// | Rust | `kind` | Kotlin type | wire |
/// |---|---|---|---|
/// | `&[Payload]` | `Ref(Slice)` | `List<Payload>` | `Long` — a handle to a Rust-side `Vec` |
/// | `Vec<Box<Payload>>` | `Vec(Boxed)` | `List<Payload>` | `List<Payload>` — a `JObject` |
///
/// Two wires, one surface. Choosing a wire is exactly the generator's job, and
/// the destination-language wrapper absorbs the difference; a caller cannot
/// tell. What a caller *can* tell — and what
/// [`unwrapped`](crate::flat::TypeRef::unwrapped) exists to prevent — is the
/// **type** changing because the source spelled a `Box`.
///
/// The rule scopes to **converted** positions: those where a converter stands
/// between the Rust value and the destination and is therefore free to bridge.
/// It cannot apply to a **layout mirror**, where the destination type is
/// reinterpreted from the source struct's bytes and is a *layout* fact rather
/// than a surface choice — there `Box<T>` (a pointer) genuinely is a different
/// destination type from `T` (inline), the spelling is load-bearing by
/// construction, and no erasure can apply. The C adapter's `repr_c_struct` is
/// the one such position in-tree, and its own documentation carries that half.
///
/// Reusing a mirror's spelling test in a converted position is how the rule
/// gets broken (prebindgen#230, #292).
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
    fn prerequisites(
        &self,
        _registry: &Registry<Self::Metadata>,
        _emit: &prebindgen_flat::Emit,
    ) -> Vec<syn::Item> {
        Vec::new()
    }

    // ── Declaration queries ────────────────────────────────────────

    /// Final post-processing pass applied to every emitted item right
    /// before write. Default: no-op.
    ///
    /// Use this for cross-cutting transforms that would otherwise have
    /// to be remembered at every individual emit site — e.g. qualifying
    /// bare type references against a source module so the emitted
    /// converter bodies compile in the binding crate's scope. Walks the
    /// entire AST, not just signatures, so type ascriptions and casts
    /// inside function bodies are covered.
    fn post_process_item(
        &self,
        _item: &mut syn::Item,
        _registry: &Registry<Self::Metadata>,
        _emit: &prebindgen_flat::Emit,
    ) {
    }

    /// Adapter-invariant checks that need registry **signatures** — the
    /// earliest they can run (decl objects are built before any source is
    /// read). Called by `RegistryBuilder::validate_with` right after the declaration
    /// scan (so a missing fn has already hard-errored; validate sees only
    /// indexed items) and before plan application. An `Err` aborts the
    /// resolve as `ScanError::AdapterInvariant` with the message verbatim
    /// — e.g. jnigen rejects a `.fun()` member whose target has no
    /// receiver parameter of the class type.
    ///
    /// Default: no checks.
    fn validate(
        &self,
        _binding: &crate::registry::Building<'_, Self::Metadata>,
    ) -> Result<(), String> {
        Ok(())
    }

    /// Post-**resolve** validation boundary — the counterpart of
    /// [`Self::validate`] that sees the fully resolved registry (converters,
    /// plans, metadata). Every artifact writer calls it before writing
    /// anything, so an invalid binding fails cleanly — with every problem
    /// reported at once — instead of panicking midway after a sibling
    /// artifact already reached disk. Deterministic over `(self, registry)`;
    /// it runs once per write call, which keeps artifact writes
    /// order-independent.
    ///
    /// Default: no checks.
    fn validate_resolved(&self, _registry: &Registry<Self::Metadata>) -> Result<(), String> {
        Ok(())
    }

    /// Absolute path under which the source crate's items are reachable
    /// from the generated file (e.g. `zenoh_flat`), for adapters that
    /// qualify emitted references against one. Drives the default
    /// [`Self::on_const`]: with a source module available, a named const
    /// re-emits as a path-alias to the source item instead of copying its
    /// initializer tokens. Default: `None`.
    fn source_module(&self) -> Option<&syn::Path> {
        None
    }

    // ── Item methods ───────────────────────────────────────────────
    //
    // Each takes the **element**, not the `syn` item it was parsed from.
    //
    // The element is the model's own node: its types are `TypeRef`s, already
    // classified. An adapter handed one therefore cannot ask what a type means
    // and be told "no reading" — the question a `&syn::ItemFn` forced it to ask
    // the registry, and which answered wrongly for a type that never entered
    // the pipeline (#275). What generated Rust must *spell* is still exactly
    // available, through `spell()`: classify off `kind`, spell with `spell()`.

    /// Wrap a `#[prebindgen]` fn into the destination-language wrapper
    /// (e.g. JNI `extern "C"` fn).
    fn on_function(
        &self,
        f: &prebindgen_flat::flat::Function,
        registry: &Registry<Self::Metadata>,
        emit: &prebindgen_flat::Emit,
    ) -> TokenStream;

    /// Per-struct emission. Typically empty for languages that get
    /// everything they need from auto-generated converters.
    fn on_struct(
        &self,
        s: &prebindgen_flat::flat::Struct,
        registry: &Registry<Self::Metadata>,
        emit: &prebindgen_flat::Emit,
    ) -> TokenStream;

    /// Per-sum emission — an `enum` whose alternatives carry payloads.
    ///
    /// Separate from [`Self::on_enum`] because the model separates them: the
    /// two are numbered differently and consumed as different constructs. An
    /// adapter with nothing to say about one shape returns an empty stream, as
    /// both in-tree adapters do for both.
    fn on_variant(
        &self,
        v: &prebindgen_flat::flat::Variant,
        registry: &Registry<Self::Metadata>,
        emit: &prebindgen_flat::Emit,
    ) -> TokenStream;

    /// Per-enum emission — the fieldless shape, a named set of integers.
    fn on_enum(
        &self,
        e: &prebindgen_flat::flat::Enum,
        registry: &Registry<Self::Metadata>,
        emit: &prebindgen_flat::Emit,
    ) -> TokenStream;

    /// Per-const emission. Default: a named const re-emits as a path-alias
    /// (see [`const_path_alias`]) when [`Self::source_module`] is available —
    /// initializer tokens are never copied, so a const whose initializer
    /// references source-crate internals stays valid in the generated file.
    /// An adapter without a source module passes the const through verbatim.
    ///
    /// A const reaching here is always named: prebindgen's own injected feature
    /// checks are [`Guard`](prebindgen_flat::flat::Guard)s, not consts, so this
    /// never has to recognise one.
    fn on_const(
        &self,
        c: &prebindgen_flat::flat::Constant,
        _registry: &Registry<Self::Metadata>,
        emit: &prebindgen_flat::Emit,
    ) -> TokenStream {
        match self.source_module() {
            Some(m) => emit.const_alias(c, m),
            None => emit.const_verbatim(c),
        }
    }
}
