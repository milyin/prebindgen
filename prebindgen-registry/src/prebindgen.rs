//! `Prebindgen` — what a generator still hands the emitter.
//!
//! Two invariant checks, and nothing else. Everything an adapter emits is
//! planned into its frozen assembly, and what the writer needs of the registry
//! is frozen with it — so a generator hands the emitter no items, no
//! prerequisites, and no questions to ask back.
//!
//! **Conversion is not here.** A generator builds those in its
//! [`Compile`](crate::recipe::Compile) hooks, one per crossing, which
//! `RegistryBuilder::generate` drives — so there is no `on_input_type`, no
//! deferral, and no fixed-point loop retrying until it converges.
//!
//! [`ConverterImpl::converter`] is the stable identity of the wire-facing
//! converter. Its executable artifact is retained separately by the adapter's
//! frozen generation plan; this shared registry carrier never stores rendered
//! Rust syntax.

use crate::{generation::OperationId, niches::Niches, registry::Registry};

/// A shared predicate over an item name, as used by
/// `Prebindgen`'s ignore hooks (bulk ignores keyed on a naming
/// family rather than an exact ident).
pub type NamePredicate = std::sync::Arc<dyn Fn(&str) -> bool + Send + Sync>;

/// Result of resolving one converter — the wire (destination) type the rest
/// of the registry sees, plus the identity of its separately retained
/// executable artifact.
///
/// `converter` is registry-owned semantic identity. Composed parents retain
/// that identity directly; they never recompute it from a Rust type or wire
/// type. The writer turns it into a private Rust symbol only through
/// [`crate::RustWriter`] during final file assembly.
#[derive(Clone)]
pub struct ConverterImpl<M = ()> {
    /// Wire/destination type. Other converters that ask "what's the wire
    /// form of this rust type?" read this. The actual function may return
    /// a wrapped form (e.g. an adapter's own `Result`-like envelope) — that
    /// is the adapter's internal calling convention; `destination` is the
    /// value the wire carries on success.
    pub destination: syn::Type,
    /// Stable identity of the **wire-facing** stage. The adapter's frozen
    /// generation plan owns its parameter list, return type, body, and other
    /// Rust emission policy.
    /// For input direction this is the FIRST stage in execution order
    /// (it takes the wire); for output direction this is the LAST stage
    /// (it produces the wire).
    pub converter: OperationId,
    /// Bit-patterns the wire type can represent but this converter never
    /// produces (output) and rejects (input). Wrapper handlers like
    /// `Option<_>` consume one slot for their own discriminant and
    /// re-export the rest — see [`Niches`] for the cascade model.
    /// Default is empty (no niche optimisation).
    pub niches: Niches,
    /// Adapter-specific extras carried alongside the converter. Filled by
    /// the same handler that produces `destination` / `function` / `niches`,
    /// and read by the adapter's language-side emitters off the fragment this
    /// conversion belongs to. Set this where you build the converter, not in a
    /// side channel.
    pub metadata: M,
    /// Inner crossings this converter composed from — the ones the adapter
    /// looked up to build a wrapper (`Option<X>` → `[X]`, `Result<T,E>` →
    /// `[T, E]`, `&T` → `[&T]`). Empty for a terminal converter (scalar, opaque
    /// handle, string) and for a callback's own converter (callback args are
    /// cross-direction — their required-ness flows through the registry's
    /// type-graph edges, not here).
    ///
    /// This is what a fragment answers with through
    /// [`Carrier::delegates_to`](crate::recipe::Carrier::delegates_to), and what
    /// `propagate_required` walks to mark reachable crossings required.
    ///
    /// **Identities, not spellings.** They are looked up and walked, never
    /// emitted. An adapter that composed the inner type keys it
    /// (`TypeKey::from_type`); one that read it off the model asks the reading
    /// (`TypeRef::key`), and names no escape to do it.
    pub subs: Vec<crate::registry::TypeKey>,
}

/// What an emitter asks of a conversion it holds.
impl<M> ConverterImpl<M> {
    /// Semantic identity of the wire-facing converter function.
    pub fn converter_id(&self) -> &OperationId {
        &self.converter
    }

    /// Wire type this conversion carries on success.
    pub fn wire_type(&self) -> &syn::Type {
        &self.destination
    }
}

/// The single extension point of the pipeline: implement this trait once per
/// **destination language** (C/cbindgen, JNI/Kotlin, Swift, Python, …) to teach
/// the language-agnostic [`Registry`] how that language represents Rust types
/// on the wire and what wrapper code to emit.
///
/// The trait has no language-specific concepts of its own, and — since the
/// registry stopped asking it questions and every emitted item became an
/// artifact of the adapter's frozen assembly — what is left is the two
/// `validate` hooks for adapter invariants.
///
/// What used to be here and is not any more: which items to build, how
/// composites decompose, and the wire form of each type. A generator states the
/// first two into the builder (`RegistryBuilder::export`,
/// `RegistryBuilder::decompose`) and answers the third in its `Compile` hooks —
/// so nothing in core has to ask for it item by item. Moving emission out too is
/// what would delete this trait entirely (prebindgen#251 phase E).
///
/// Anything language-specific the rest of the pipeline must carry — a JNI
/// adapter's Kotlin class names and exception info, a C adapter's header names,
/// etc. — rides in [`ConverterImpl::metadata`], and is read back by the
/// adapter's own emitter off the fragment that conversion belongs to. The
/// registry neither stores it nor names its type.
///
/// # The rule an adapter must obey
///
/// "Analyze the Flat model; generate Rust only" tells an adapter where every
/// decision comes from. Final source-type output is an inert fragment produced
/// by [`RustWriter`](crate::RustWriter), never a typed view of retained syntax. It is silent
/// on the question adapters actually face — what the **destination language**
/// ends up seeing.
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
/// | `Vec<Box<Payload>>` | `Vec(Boxed)` | `List<Payload>` | `JObject` — a Java `List<Payload>` |
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
    // ── Declaration queries ────────────────────────────────────────

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
    fn validate(&self, _binding: &crate::registry::Building<'_>) -> Result<(), String> {
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
    fn validate_resolved(&self, _registry: &Registry) -> Result<(), String> {
        Ok(())
    }
}
