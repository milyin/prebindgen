//! Input side of the per-function lowered binding plan (issue #90).
//!
//! [`JniFunctionPlan`] lowers every input parameter of a bound function ONCE,
//! deterministically over `(ext, registry, f)`. Each leaf freezes three
//! independent answers: its exact ordered native ABI leaves, its Rust decode
//! operation, and its Kotlin call/locking operation. The Rust `extern "C"`
//! wrapper, Kotlin wrapper, `JNINative` declaration, and JVM-slot validator
//! consume those answers without reconstructing them from a source-shape tag.
//! The pattern generalizes [`build_struct_plan`]'s field-level plan to function
//! granularity. Ordinary decomposed outputs likewise retain their ordered
//! outgoing wires and converter pipelines before either writer runs.

use kotlin_codegen::KtType;
use prebindgen_registry::{flat::TypeRef, Conversions};

use super::*;

/// The lowered plan for one bound function: one [`PlanParam`] per source
/// `syn::Signature` parameter (non-`Typed`/non-`Ident` args — `self`,
/// patterns — are skipped, mirroring every prior walk), plus the classified
/// output side.
pub(crate) struct JniFunctionPlan {
    /// The mangled `JNINative` extern method name — the one name the Rust
    /// export symbol, the Kotlin `external fun` declaration, and the wrapper
    /// call target all key on. Computed ONCE as
    /// `ext.mangle_jni_method(&kt_snake_to_camel(rust_ident))`, so the three
    /// tiers agree by construction (previously the Rust symbol camelCased
    /// with a different helper — a silent mismatch for non-snake idents).
    pub jni_method: String,
    /// The spec-escaped JNI export symbol (`Java_<pkg>_<JNINative>_<method>`,
    /// see `symbol`, #86), derived from [`Self::jni_method`].
    pub native_symbol: String,
    /// The onError handler interfaces — the always-present binding
    /// `JniErrorHandler` plus, for a fallible function, its typed domain
    /// `<Err>Handler` (see [`ErrorIfaces`]). Shared from the
    /// [`Declarations::iface_spec`] memo: one derivation per channel feeds the Rust
    /// `__SINK_*` statics, the Kotlin sink wiring, and the interface
    /// declarations, so the FQN/descriptor pairs of the cached `run` lookups
    /// cannot drift. `None` = the domain channel is underivable (the Rust
    /// emitter panics, the Kotlin renderer skips).
    pub onerror_iface: Option<ErrorIfaces>,
    /// Registry-resolved output decomposition selected for this function.
    /// Rust delivery, Kotlin surface/KDoc, and the report all consume this
    /// owned plan after the registry phase is closed.
    pub unfold: Option<crate::unfold::UnfoldPlan>,
    /// Registry-resolved domain-error delivery selected for this function.
    /// The structural decomposition, exact outgoing JNI operations, and any
    /// composed converter are frozen together before either writer runs.
    pub error: Option<ErrorOutputPlan>,
    pub params: Vec<PlanParam>,
    pub output: FnOutputPlan,
}

/// Frozen delivery of a fallible function's `Err(E)` arm.
///
/// Kotlin and report generation read [`Self::unfold`]. Rust delivery consumes
/// [`Self::wires`] and [`Self::chain`], so it cannot reconstruct converters
/// from `TypeRef` while rendering the wrapper.
pub(crate) struct ErrorOutputPlan {
    pub unfold: crate::unfold::UnfoldPlan,
    pub wires: std::rc::Rc<Vec<crate::jni::compile::OutWire>>,
    pub chain: Option<crate::jni::compile::ComposedChain>,
    /// Origin qualification and sum shape for these leaves, frozen with them,
    /// so rendering the `Err` arm asks the registry nothing.
    pub delivery: crate::jni::emit::FrozenDelivery,
}

impl std::ops::Deref for ErrorOutputPlan {
    type Target = crate::unfold::UnfoldPlan;

    fn deref(&self) -> &Self::Target {
        &self.unfold
    }
}

/// One source parameter: the ident/type as written plus its lowered form.
pub(crate) struct PlanParam {
    pub ident: syn::Ident,
    /// The parameter's **reading**. For a `ParamForm::Single` it is the very
    /// reading the leaf carries — `emit/wrapper.rs` says as much where it uses
    /// the leaf's instead — so carrying the spelling was carrying a copy that
    /// could not disagree, in a form that could not be asked anything.
    pub ty: prebindgen_registry::flat::TypeRef,
    pub form: ParamForm,
}

impl PlanLeaf {
    /// The converter this leaf is decoded through, which is the one its own
    /// Rust operation invokes: a flattened data class and an optional pair are
    /// rebuilt by one composed converter of their own, and every other kind
    /// runs the frozen pipeline.
    pub(crate) fn calls(&self, out: &mut Vec<prebindgen_registry::write::ArtifactKey>) {
        match &self.rust {
            RustParamOp::Pipeline { .. } => self.pipeline.calls(out),
            RustParamOp::OptionalPair(plan) => out.push(
                prebindgen_registry::write::ArtifactKey::Operation(plan.chain.operation.clone()),
            ),
            RustParamOp::FlattenStruct(plan) => out.push(
                prebindgen_registry::write::ArtifactKey::Operation(plan.chain.operation.clone()),
            ),
        }
    }
}

/// How a source parameter crosses the boundary. The single leaf is boxed to
/// keep the variants near the same size (a [`PlanLeaf`] embeds whole
/// sub-plans; the `Expanded` payload is just a `Vec` header).
pub(crate) enum ParamForm {
    /// Ordinary parameter — one classified leaf.
    Single(std::rc::Rc<PlanLeaf>),
    /// Constructor-expansion ([`FoldPlan`] declared for this `(fn, param)`):
    /// the wire form is the plan's flattened leaves, classified individually;
    /// the Rust wrapper folds them back into the built value. Leaves use the
    /// same recursive data-class probe as ordinary parameters (vec-build
    /// remains a source-parameter-only collection optimization), so all three
    /// sites agree on the leaf wire.
    Expanded {
        /// The frozen constructor fold and qualified call paths. Keeping it
        /// beside its lowered leaves lets Rust reconstruction and Kotlin/report
        /// descriptions use the same decision after resolution.
        plan: Box<ExpandedParamPlan>,
        leaves: Vec<std::rc::Rc<PlanLeaf>>,
    },
}

/// Registry-built expansion fold plus every Rust constructor path it may call.
///
/// The core [`prebindgen_registry::expand::FoldPlan`] owns constructor
/// selection and recursive argument assembly. JNI freezes origin qualification
/// beside it while the registry is still available, so final wrapper emission
/// only supplies decoded leaf locals and never resumes a registry lookup.
pub(crate) struct ExpandedParamPlan {
    fold: prebindgen_registry::expand::FoldPlan,
    constructors: BTreeMap<String, syn::Path>,
}

impl ExpandedParamPlan {
    fn new(
        ext: &Declarations,
        registry: &Registry,
        fold: &prebindgen_registry::expand::FoldPlan,
    ) -> Self {
        let mut constructors = BTreeMap::new();
        Self::freeze_variants(ext, registry, &fold.variants, &mut constructors);
        Self {
            fold: fold.clone(),
            constructors,
        }
    }

    fn freeze_variants(
        ext: &Declarations,
        registry: &Registry,
        variants: &[prebindgen_registry::expand::FoldVariant],
        constructors: &mut BTreeMap<String, syn::Path>,
    ) {
        for variant in variants {
            if let Some(ctor) = &variant.ctor {
                let module = ext.fn_module(registry, ctor);
                let path = syn::parse_quote!(#module::#ctor);
                constructors.entry(ctor.to_string()).or_insert(path);
            }
            for input in &variant.inputs {
                if let prebindgen_registry::expand::FoldArg::Build(build) = input {
                    Self::freeze_variants(ext, registry, &build.variants, constructors);
                }
            }
        }
    }

    /// The core fold shared with Kotlin overload and report consumers.
    pub(crate) fn fold(&self) -> &prebindgen_registry::expand::FoldPlan {
        &self.fold
    }

    /// Assemble the fold expression from already-decoded leaf locals.
    pub(crate) fn emit(&self, leaf_locals: &[syn::Ident]) -> syn::Expr {
        prebindgen_registry::expand::emit_fold(&self.fold, leaf_locals, &|ident| {
            self.constructors
                .get(&ident.to_string())
                .unwrap_or_else(|| {
                    panic!("frozen JNI expansion plan has no constructor path for `{ident}`")
                })
                .clone()
        })
    }

    #[cfg(test)]
    pub(crate) fn constructor_path(&self, ident: &str) -> Option<&syn::Path> {
        self.constructors.get(ident)
    }
}

/// One classified effective parameter (a source param, or one expansion leaf).
pub(crate) struct PlanLeaf {
    /// The leaf's **reading** — classification and spelling in one value, so
    /// the two cannot disagree and no consumer has to look the type up. Spell
    /// with `emit.emit_source_type(reading)` in an emission callback.
    pub reading: TypeRef,
    /// Kotlin parameter name (`kt_param_name(ident)`: camelCase +
    /// hard-keyword escaping) — shared by the wrapper signature and the
    /// `external fun` declaration.
    pub kt_name: String,
    /// Typed-wrapper surface type: the projection's Kotlin FQN for
    /// handle/value projections, else the resolved entry's Kotlin name.
    /// `None` when the metadata lacks a name (the Kotlin wrapper renderer
    /// skips the function — the escape-hatch path) and for callback operations
    /// (typed from the interface spec at render time).
    pub kt_public: Option<KtType>,
    /// Whether the leaf crosses as optional, **per the model** — so a wrapped
    /// spelling (`Box<Option<T>>`) answers exactly as the bare one does. Each
    /// site applies its own nullability rule on top (handles stay non-null
    /// `Long` on the extern but `T?` on the surface).
    ///
    /// This used to be `is_option_type(ty)`, which asks the *spelling* whether
    /// its last path segment reads `Option` — and the model erases `Box` and
    /// `Cow`, so an optional behind one lost its `?` (#273).
    pub optional: bool,
    /// `true` when the (probed-through `&`/`Option`) type is an
    /// `enum_class` enum: surface keeps the typed enum, the extern declares
    /// `Int`/`Int?`, and the call site passes `.value` / `?.value`.
    pub as_enum_value: bool,
    /// Primitive sentinel carved by this Optional enum layer. When present,
    /// the JNI extern keeps `Int` non-null and Kotlin maps `null` to this value.
    pub enum_niche: Option<String>,
    /// Frozen Rust converter operation and its JNI wire. Special layouts may
    /// supply several site wires instead, but ordinary parameters and
    /// constructor-expansion leaves call this pipeline directly.
    pub pipeline: crate::jni::chain::JPipeline,
    /// Exact ordered parameters shared by the Rust extern, `JNINative`
    /// declaration, and JVM descriptor-slot validation.
    ///
    /// Shared, not copied: the site's `AbiLayout` holds this same list, so
    /// there is ONE ordered ABI rather than a carrier that has to be kept
    /// equal to it. That is what lets step 5 delete this side without
    /// re-deriving anything (#622 review).
    pub native: std::rc::Rc<Vec<NativeParam>>,
    /// How the Rust wrapper obtains this leaf's source value from `native`.
    pub rust: RustParamOp,
    /// How the typed Kotlin wrapper supplies `native`, including handle
    /// locking/consumption policy.
    pub kotlin: KotlinParamOp,
}

/// One exact JNI parameter in a leaf's ordered native ABI.
pub(crate) struct NativeParam {
    pub rust_ident: syn::Ident,
    /// Final Rust spelling, including the JNI object lifetime when required.
    pub rust_wire: TokenStream,
    pub kt_name: String,
    /// `None` preserves the unresolved-name escape hatch: Kotlin emission
    /// skips the containing function.
    pub kt_wire: Option<KtType>,
    pub jvm_slots: usize,
}

/// Frozen Rust-side operation for one effective parameter.
pub(crate) enum RustParamOp {
    /// Invoke the registry-planned pipeline with this single native leaf.
    Pipeline { wire_ident: syn::Ident },
    /// Rebuild an allocation-free optional from its two native leaves.
    OptionalPair(std::rc::Rc<crate::jni::compile::OptionalPairPlan>),
    /// Rebuild a data class from its recursively flattened native leaves.
    FlattenStruct(std::rc::Rc<FlatInputPlan>),
}

/// Frozen typed-Kotlin call and ownership operation for one effective
/// parameter. These variants are target operations, not a source-type
/// classification; native ABI and Rust decoding are stored independently.
pub(crate) enum KotlinParamOp {
    /// `impl Fn(args)` callback: erased `Any` on the wire. `iface` is the
    /// typed `fun interface` spec (memoized under [`SpecKey::Callback`] —
    /// the same allocation the trampoline and the declaration emitter read);
    /// `None` = underivable, the Kotlin wrapper renderer skips.
    Callback { iface: Option<Arc<IfaceSpec>> },
    /// `&[T]` / `Vec<T>` of a flattenable data_class: a single `jlong`
    /// Vec-handle on the wire, built by pushing element leaves.
    /// Rust reconstruction is frozen in [`PlanLeaf::pipeline`]; this kind keeps
    /// only the helper ABI that Kotlin and the synthetic externs share.
    VecBuild {
        helpers: std::rc::Rc<VecBuildHelpers>,
    },
    /// Bare `Option<primitive>` / `Option<enum>`: a decoupled
    /// `(present: jboolean, value: <wire>)` pair.
    OptionalPair(std::rc::Rc<crate::jni::compile::OptionalPairPlan>),
    /// Flattenable data_class: the field leaves cross as separate wire params.
    FlattenStruct(std::rc::Rc<FlatInputPlan>),
    /// Lockable opaque-handle projection (`jlong` wire). Ownership and
    /// nullability are Kotlin locking policy; Rust conversion is already
    /// frozen in the leaf pipeline.
    Handle { mode: HandleMode },
    /// Rust `u64`: typed Kotlin `ULong`, raw JNI `Long`. The wrapper passes
    /// the bit-preserving `toLong()` representation and takes no lock.
    Unsigned64 { niche: Option<String> },
    /// Everything else: the resolved entry's converter/wire as-is.
    Plain,
}

/// Frozen Kotlin ownership/locking mode for an opaque-handle leaf.
pub(crate) enum HandleMode {
    Borrow,
    Consume,
    BorrowNullable,
    ConsumeNullable,
}

/// How the return value crosses the boundary. Mirrors the unfold plan's
/// [`Delivery`](crate::unfold::Delivery), resolved per function:
/// `Unfold` = callback delivery (builder/fold lambda, erased `Any?` wire);
/// `Value` = everything else, including the `Return`-delivery convert.
pub(crate) enum FnOutputPlan {
    Unfold(Box<UnfoldOutputPlan>),
    Value(std::rc::Rc<ValueOutputPlan>),
}

/// Callback-delivery shape facts, read off the fn's `UnfoldPlan` once so the
/// Rust builder param, the erased extern params, and the typed Kotlin
/// builder/fold surface all branch on the same booleans.
pub(crate) struct UnfoldOutputPlan {
    /// Registry-composed values the return site hands to its builder/folder,
    /// with each target ABI and output pipeline frozen. Empty only for a
    /// whole-element iterable fold.
    /// Shared with the site plan that froze them, so the values a decomposed
    /// return hands out have one owner rather than a plan and a carrier that
    /// have to agree (#622 review).
    pub wires: std::rc::Rc<Vec<crate::jni::compile::OutWire>>,
    /// Exact composed converter for a fixed Product/Optional/Choice return.
    pub chain: Option<crate::jni::compile::ComposedChain>,
    /// Whole-element iterable conversion, when `wires` is empty because the
    /// fold receives each element through its ordinary one-value converter.
    pub element_pipeline: Option<crate::jni::chain::JPipeline>,
    /// `is_iterable_fold(shape)` — a bare `Iterable` OR one wrapped in an
    /// `Optional` layer (`Option<Vec<T>>`). Selects the fold surface
    /// (`acc` + `fold`) over a scalar builder on every tier.
    pub iterable_fold: bool,
    /// Outer `Optional` layer present — the delivered result is nullable
    /// (for a fold: `None` skips the fold and delivers null, so the wrapper
    /// returns `A?`).
    pub optional: bool,
    /// Synthesized fixed-singleton delivery: no caller lambda, not generic.
    pub fixed_builder: bool,
    /// `plan.element.is_some()` — whole-element (M4) vs decomposed (M5) fold.
    pub whole_element: bool,
    /// Declaration-normalized decomposition used by fixed Kotlin
    /// builder/folder singletons. `None` for a whole-element fold, which has
    /// no deconstructor declaration.
    pub decon: Option<std::rc::Rc<crate::unfold::DeconSpec>>,
    /// Origin qualification and sum shape for these leaves, frozen with them,
    /// so rendering the delivery asks the registry nothing.
    pub delivery: crate::jni::emit::FrozenDelivery,
    /// Kotlin type variable of the wrapper: `None` for a fixed builder,
    /// `"A"` for an `Iterable` fold (bare or `Optional`-wrapped), `"R"`
    /// otherwise.
    pub generic: Option<&'static str>,
    /// The builder/folder `fun interface` spec the delivery calls into —
    /// [`folder_iface_for_plan`] for an iterable fold (incl. the fixed
    /// whole-element form), the memoized [`SpecKey::Builder`] spec
    /// otherwise. Shared from the [`Declarations::iface_spec`] memo: one
    /// derivation feeds the Rust upcall statics, every Kotlin surface read,
    /// and the interface declaration, so the cached `run` FQN/descriptor
    /// pair cannot drift. `None` = underivable (the Rust emitter keeps its
    /// historical `expect`s, the Kotlin renderer skips).
    pub iface: Option<Arc<IfaceSpec>>,
}

/// Value-return facts: the resolved conversion target and wire on the Rust
/// side, the declared-surface classification on the Kotlin side.
pub(crate) struct ValueOutputPlan {
    /// Where this return crosses and which fragment answers it, carried from
    /// the site hook so the plan the emitters actually read — the one with the
    /// convert delivery attached — is the one frozen into the canonical site,
    /// rather than the intermediate the hook produced (#622 review).
    pub site: Option<(
        prebindgen_registry::recipe::Bound,
        prebindgen_registry::generation::FragmentId,
    )>,
    /// `Return`-delivery convert (`convert_output`) — the wrapper returns the
    /// single deconstructed value through its ordinary output converter.
    pub is_convert: bool,
    /// Frozen Rust converter operation and its JNI wire. The ordinary wrapper
    /// invokes this once instead of looking the crossing up and reconstructing
    /// its semantic pre-stage order during emission.
    pub pipeline: crate::jni::chain::JPipeline,
    /// Kotlin surface classification over the **declared** return
    /// (`convert_out_ty` for a convert, else `f.sig.output` — not
    /// `target_ty`: the Kotlin error peel rides `value_reading`).
    pub surface: ReturnSurface,
    /// `enum_class` / `Option<enum>` probes over the canonical
    /// (`value_reading`-peeled) declared return. The extern decl uses them
    /// raw; the wrapper surface masks them with `!is_convert` (the historical
    /// `unfold.is_none()` gate).
    pub is_enum: bool,
    pub is_option_enum: bool,
    /// Primitive sentinels consumed by nested Optional enum layers,
    /// outside-in. Every one collapses to Kotlin `null` on output.
    pub enum_niches: Vec<String>,
    /// Origin qualification for the accessor calls a `Return`-delivery
    /// convert reaches its single leaf through, frozen with the plan. `None`
    /// unless [`Self::is_convert`].
    pub convert_delivery: Option<crate::jni::emit::FrozenDelivery>,
}

/// The pure classification core of `classify_return` — no import
/// registration, no name shortening, no panics. The render adapter
/// (`render_return_surface`) maps it back to the historical
/// `(kt_return, projection)` pair, panicking on an unregistered projection
/// FQN exactly where `classify_return` always did (Kotlin render time).
#[derive(Clone)]
pub(crate) enum ReturnSurface {
    /// No Kotlin type resolvable (entry or `kotlin_name` missing): the
    /// Kotlin renderers skip the function; the Rust emitter ignores it.
    Skip,
    /// Unit return, including the canonical peel (`ZResult<()>`).
    Unit,
    /// Projection return (opaque handle / `ULong`). `leaf_fqn` is the
    /// resolved Kotlin FQN; `None` = unregistered (the adapter panics).
    Projected {
        projection: Projection,
        leaf_fqn: Option<String>,
    },
    /// Plain return typed by the entry's resolved Kotlin name (unshortened —
    /// the adapter registers/shortens at render time).
    Plain { kt: KtType },
}

/// Plan construction failure. The validation boundary
/// ([`validate_bindings`]) reports every failure before any artifact is
/// written; the Rust emitter keeps the same messages as panic backstops and
/// the Kotlin renderers map to `None` (skip).
#[derive(Debug)]
pub(crate) enum PlanError {
    /// `registry.input_entry` has no converter for a source param type.
    ///
    /// The **reading**, not a key: the registry knew the type well enough to
    /// classify it — what it lacks is a converter — so the reading is in hand,
    /// and it carries the source position [`Self::message`] points at.
    ///
    /// **Boxed**, and it has to be. A `TypeRef` holds a `syn::Type` inline and
    /// runs to ~264 bytes; a `Result` is sized by its largest variant, so an
    /// unboxed reading here would widen every `Result<_, PlanError>` on this
    /// path — the *success* return included, which is the one that always
    /// happens (`clippy::result_large_err`). An error is rare enough to afford
    /// the allocation; the plans it returns beside are not.
    Unresolved { ty: Box<TypeRef> },
    /// No converter for a constructor-expansion leaf type.
    UnresolvedLeaf { ty: Box<TypeRef>, param: syn::Ident },
    /// The output target type is known but `registry.output_entry` has no
    /// converter for it — the failure `output_wrapper` fixes.
    UnresolvedOutput { ty: Box<TypeRef> },
    /// The output target type is not in the registry **at all**, so there is no
    /// reading to hold and `output_wrapper` is not the answer.
    ///
    /// Split from [`Self::UnresolvedOutput`] because the two want opposite
    /// advice: one type needs a converter written for it, the other needs to
    /// reach the registry first. Collapsed together, the `output_wrapper`
    /// message sent the reader to write a converter for a type the resolver
    /// would still never ask about.
    UnknownOutputType { ty: TypeKey },
    /// An unmarked declared data class could not produce a complete recursive
    /// input plan. Silent `JObject` fallback is forbidden.
    UnflattenableDataClass(FlatInputError),
    /// The flattened native method would exceed the JVM's 255 parameter-unit
    /// descriptor limit (including the implicit receiver).
    JvmParameterLimit { slots: usize },
}

/// A plan failure, as the adapter's own error carries it.
pub(crate) fn plan_error(e: PlanError) -> crate::jni::compile::JErr {
    crate::jni::compile::JErr::Plan(Box::new(e))
}

impl PlanError {
    /// Where the offending type was written, when a file wrote it — the
    /// suffix [`Self::message`] appends.
    ///
    /// `has_position` gates it exactly as `resolve.rs` gates
    /// `UnresolvedEntry::location`: a composed type and a test's hand-built
    /// stream are lowered against `SourceLocation::default`, and printing
    /// `:0:0` for them would make a fabricated position look like a real one.
    fn location_suffix(&self) -> String {
        let reading = match self {
            PlanError::Unresolved { ty }
            | PlanError::UnresolvedLeaf { ty, .. }
            | PlanError::UnresolvedOutput { ty } => ty,
            PlanError::UnknownOutputType { .. }
            | PlanError::UnflattenableDataClass(_)
            | PlanError::JvmParameterLimit { .. } => return String::new(),
        };
        let loc = reading.location();
        if loc.has_position() {
            format!(" (declared at {loc})")
        } else {
            String::new()
        }
    }

    /// The historical emission-panic message for this failure, shared by the
    /// validation boundary and the Rust emitter's backstop panics so the
    /// wording cannot drift.
    ///
    /// The base wording is unchanged; a source position is appended when the
    /// reading has one, so a backstop that reaches the same failure without a
    /// reading still prints a prefix of this.
    pub fn message(&self, fn_ident: &syn::Ident) -> String {
        let at = self.location_suffix();
        match self {
            PlanError::Unresolved { ty } => format!(
                "JniGen extern: input type `{}` for `{}` is unresolved{at}",
                ty.key(),
                fn_ident,
            ),
            PlanError::UnresolvedLeaf { ty, param } => format!(
                "JniGen expand: leaf type `{}` (parameter `{}`) is unresolved{at}",
                ty.key(),
                param,
            ),
            PlanError::UnresolvedOutput { ty } => format!(
                "JniGen extern: return type `{}` of `{}` has no registered output \
                 converter — register one via `Declarations::output_wrapper(pat, |…| Some((ty, exc, body)))` \
                 (exc = `None` for non-throwing, `Some(parse_quote!(<full path>))` \
                  to bind a domain exception){at}",
                ty.key(),
                fn_ident,
            ),
            PlanError::UnknownOutputType { ty } => format!(
                "JniGen extern: return type `{}` of `{}` is not registered — the \
                 resolver never saw this type, so no converter can be selected for it. \
                 Declare the type (or the function that produces it) before binding `{}`",
                ty, fn_ident, fn_ident,
            ),
            PlanError::UnflattenableDataClass(error) => {
                format!("JniGen extern `{fn_ident}`: {}", error.message())
            }
            PlanError::JvmParameterLimit { slots } => format!(
                "JniGen extern `{fn_ident}`: flattened JNI signature uses {slots} JVM parameter slots (maximum 255, including the JNINative receiver); reduce the data-class shape or declare an intentional `data_class!(T).jobject_input()` boundary"
            ),
        }
    }
}

/// The post-resolve validation boundary (issue #90): build the lowered plan
/// for every bound function — declared functions, declared-const getters,
/// and expression-constant getters — and check the split declarations,
/// collecting every failure. Called by every artifact writer (via
/// [`Prebindgen::validate_resolved`]) before anything reaches disk, so an
/// invalid binding can no longer leave one artifact written and its sibling
/// missing.
///
/// [`Prebindgen::validate_resolved`]: prebindgen_registry::Prebindgen::validate_resolved
pub(crate) fn validate_bindings(ext: &Declarations, registry: &Registry) -> Result<(), String> {
    let mut errors: Vec<String> = Vec::new();

    if let Err(e) = ext.validate_split_declarations(registry) {
        errors.push(e);
    }

    // Native-symbol collision table (issue #89): every `#[no_mangle]` export
    // the emitters produce must be unique. Each successfully-built plan
    // carries its spec-escaped `native_symbol`; a duplicate (two functions
    // whose name hooks collapse to one JNINative method) is a hard error —
    // it would otherwise surface only as a duplicate `#[no_mangle]` Rust
    // symbol at link time. `origin` is the Rust ident that produced it.
    let mut native: std::collections::BTreeMap<NativeSymbol, String> = Default::default();
    let mut record_symbol = |sym: &str, origin: String, errors: &mut Vec<String>| {
        let key = NativeSymbol::new(sym);
        if let Some(prev) = native.insert(key, origin.clone()) {
            errors.push(format!(
                "duplicate native symbol `{sym}`: produced by both `{prev}` and `{origin}` \
                 — a name mangle hook or `.name()` collapsed two distinct methods onto one \
                 JNI export",
            ));
        }
    };

    // Declared functions (incl. binding-local synthetics and fn-backed
    // constants), in deterministic ident order.
    let declared = ext.declared_functions();
    // The elements themselves, not their names: looking a name back up would be
    // a second hash and an infallible-lookup-that-is-not. `Ident: Ord` is the
    // string order, so the sort is unchanged.
    let mut fns: Vec<&prebindgen_registry::flat::Function> = registry.flat().functions().collect();
    fns.sort_by(|a, b| a.name.cmp(&b.name));
    for f in fns {
        let ident = &f.name;
        if !declared.contains(ident) {
            continue;
        }
        match ext.fn_plan(registry, f) {
            Ok(plan) => record_symbol(&plan.native_symbol, ident.to_string(), &mut errors),
            Err(e) => errors.push(e.message(ident)),
        }
    }

    // Declared consts: their synthetic nullary getters run through the same
    // plan machinery, and reach the file as constant artifacts.
    if let Some(declared_consts) = ext.declared_consts() {
        let mut consts: Vec<&prebindgen_registry::flat::Constant> =
            registry.flat().constants().collect();
        consts.sort_by(|a, b| a.name.cmp(&b.name));
        for c in consts {
            let ident = &c.name;
            if !declared_consts.contains(ident) {
                continue;
            }
            let getter = const_getter_fn(c);
            match ext.fn_plan(registry, &getter) {
                Ok(plan) => record_symbol(&plan.native_symbol, ident.to_string(), &mut errors),
                Err(e) => errors.push(e.message(&getter.name)),
            }
        }
    }

    // Expression constants: same synthetic `const_get_*` getter shape,
    // seeded from the val name.
    let mut expr_decls: Vec<_> = ext
        .packages
        .values()
        .flat_map(|p| &p.constant_exprs)
        .collect();
    expr_decls.sort_by(|a, b| a.kotlin_name.cmp(&b.kotlin_name));
    for decl in expr_decls {
        let getter = {
            ext.freeze_reading_of(registry, &decl.ty);
            const_expr_getter_fn(&decl.kotlin_name, &decl.ty, ext)
        };
        match ext.fn_plan(registry, &getter) {
            Ok(plan) => record_symbol(&plan.native_symbol, decl.kotlin_name.clone(), &mut errors),
            Err(e) => errors.push(e.message(&getter.name)),
        }
    }

    // Kotlin identifier validity + per-package top-level-name collisions.
    errors.extend(validate_symbols(ext, registry));

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("\n"))
    }
}

impl JniFunctionPlan {
    /// Every converter the extern's body calls: one chain per parameter, the
    /// output's, and the error arm's when the function has one.
    pub(crate) fn calls(&self, out: &mut Vec<prebindgen_registry::write::ArtifactKey>) {
        for param in &self.params {
            match &param.form {
                ParamForm::Single(leaf) => leaf.calls(out),
                ParamForm::Expanded { leaves, .. } => {
                    for leaf in leaves {
                        leaf.calls(out);
                    }
                }
            }
        }
        match &self.output {
            FnOutputPlan::Value(value) => value.pipeline.calls(out),
            FnOutputPlan::Unfold(unfold) => {
                for wire in unfold.wires.iter() {
                    wire.calls(out);
                }
                if let Some(chain) = &unfold.chain {
                    out.push(prebindgen_registry::write::ArtifactKey::Operation(
                        chain.operation.clone(),
                    ));
                }
                if let Some(pipeline) = &unfold.element_pipeline {
                    pipeline.calls(out);
                }
            }
        }
        if let Some(error) = &self.error {
            for wire in error.wires.iter() {
                wire.calls(out);
            }
            if let Some(chain) = &error.chain {
                out.push(prebindgen_registry::write::ArtifactKey::Operation(
                    chain.operation.clone(),
                ));
            }
        }
    }
}

impl FnOutputPlan {
    /// The extern's wire return type: the erased builder result (`JObject`)
    /// for a callback delivery, the resolved entry's destination otherwise.
    /// Feeds `annotate_jobject_with_lifetime` + `sentinel_for_wire` on the
    /// Rust side.
    pub fn wire_ty(&self) -> syn::Type {
        match self {
            FnOutputPlan::Unfold(_) => syn::parse_quote!(jni::objects::JObject),
            FnOutputPlan::Value(v) => v.pipeline.wire().clone(),
        }
    }
}

impl Declarations {
    /// The memoized lowered plan for one bound function — the "build the plan
    /// once and store it" stage [`JniFunctionPlan::build`] anticipated (issue
    /// #90). Keyed by the function's ident (bound functions live in one flat
    /// namespace, and the synthetic const-getter idents `const_get_*` are
    /// distinct), so validation and every emitter share ONE derivation
    /// instead of rebuilding it ~8× per generation. `Ok` is cached; an `Err`
    /// (an unresolved converter) is passed through — it only occurs at the
    /// validation phase, which reports it and fails `resolve` before any
    /// emitter runs. Same interior-mutable contract as
    /// [`Declarations::iface_spec`]. Once resolution finishes both memos are
    /// drained into [`crate::jni::generation::JniGenerationPlan`], so emission
    /// cannot resume either derivation.
    /// This function's lowered plan, read from the frozen generation.
    ///
    /// For a **render-only** caller. `freeze` drains the lowering memos into
    /// `GenerationPlan`, which owns them afterwards, so a renderer reads rather
    /// than re-lowering — and needs no `Registry` to do it.
    ///
    /// Not for a caller that also runs during validation: no generation is
    /// installed then, this answers `None`, and a site that skips silently
    /// stops reporting what lowering would have diagnosed (#613 step 7).
    pub(crate) fn fn_plan_frozen(
        &self,
        f: &prebindgen_registry::flat::Function,
    ) -> Option<std::rc::Rc<JniFunctionPlan>> {
        self.generation.as_deref()?.function(&f.name)
    }

    pub(crate) fn fn_plan(
        &self,
        registry: &Registry,
        f: &prebindgen_registry::flat::Function,
    ) -> Result<std::rc::Rc<JniFunctionPlan>, PlanError> {
        if let Some(generation) = &self.generation {
            return Ok(generation.function(&f.name).unwrap_or_else(|| {
                panic!(
                    "frozen JNI generation plan has no function entry for `{}`",
                    f.name
                )
            }));
        }
        if let Some(hit) = self.fn_plans.borrow().get(&f.name).cloned() {
            return Ok(hit);
        }
        let plan = std::rc::Rc::new(JniFunctionPlan::build(self, registry, f)?);
        self.fn_plans
            .borrow_mut()
            .insert(f.name.clone(), plan.clone());
        Ok(plan)
    }
}

impl JniFunctionPlan {
    /// Lower `f`'s inputs. Deterministic over `(ext, registry, f)`. Emission
    /// and validation go through the memo [`Declarations::fn_plan`], so the plan is
    /// built ONCE per function and shared; this is the underlying derivation.
    pub fn build(
        ext: &Declarations,
        registry: &Registry,
        f: &prebindgen_registry::flat::Function,
    ) -> Result<Self, PlanError> {
        let jni_method = ext.mangle_jni_method(&kt_snake_to_camel(&f.name.to_string()));
        let native_symbol = ext.native_method_symbol(&jni_method);
        let unfold = ext.unfolded().unfold_plans.get(&f.name).cloned();
        let error = ext.unfolded().error_plans.get(&f.name).cloned();
        let onerror_iface = onerror_iface_spec(ext, registry, &f.name);
        // Output first: the Rust emitter historically resolved the output
        // before the inputs, so an unresolved-output failure takes precedence
        // over an unresolved-input one.
        let output = build_output(ext, registry, f, unfold.as_ref(), error.as_ref())?;
        let error = error
            .map(|plan| build_error_output(ext, registry, plan))
            .transpose()?;
        let mut params = Vec::new();
        // The element's parameters: each already a name and a `TypeRef`, so
        // there is no `FnArg`/`Pat` destructuring and no position that could
        // fail to yield a type.
        for (position, param) in f.params.iter().enumerate() {
            let ident = param.name.clone();
            let ty = param.ty.clone();

            let form = if let Some(plan) = registry
                .expansion_plans()
                .get(&(f.name.clone(), ident.clone()))
            {
                let mut leaves = Vec::new();
                for (leaf_index, leaf) in plan.leaves.iter().enumerate() {
                    // The lookup that stood here is gone: the fold leaf carries
                    // its own reading now, so there is nothing to fetch and
                    // nothing that can miss.
                    leaves.push(classify_leaf(
                        ext, registry, &leaf.name, &leaf.ty, /*expanded=*/ true, &ident,
                        &f.name, position, leaf_index,
                    )?);
                }
                ParamForm::Expanded {
                    plan: Box::new(ExpandedParamPlan::new(ext, registry, plan)),
                    leaves,
                }
            } else {
                ParamForm::Single(classify_leaf(
                    ext, registry, &ident, &param.ty, /*expanded=*/ false, &ident, &f.name,
                    position, 0,
                )?)
            };
            params.push(PlanParam { ident, ty, form });
        }
        let result = Self {
            jni_method,
            native_symbol,
            onerror_iface,
            unfold,
            error,
            params,
            output,
        };
        let slots = result.jvm_parameter_slots();
        if slots > 255 {
            return Err(PlanError::JvmParameterLimit { slots });
        }
        Ok(result)
    }

    /// The flattened effective-parameter view (expansion leaves inline) —
    /// the sequence the Kotlin wrapper and `external fun` declare, in order.
    pub fn leaves(&self) -> impl Iterator<Item = &PlanLeaf> {
        self.params
            .iter()
            .flat_map(|p| match &p.form {
                ParamForm::Single(l) => std::slice::from_ref(l).iter(),
                ParamForm::Expanded { leaves, .. } => leaves.iter(),
            })
            .map(std::rc::Rc::as_ref)
    }

    fn jvm_parameter_slots(&self) -> usize {
        // `JNINative` is a Kotlin object, so its external methods are instance
        // methods and the JVM counts the implicit receiver as one unit.
        let mut slots = 1usize;
        for leaf in self.leaves() {
            slots += leaf
                .native
                .iter()
                .map(|param| param.jvm_slots)
                .sum::<usize>();
        }
        slots += match &self.output {
            FnOutputPlan::Unfold(plan) if plan.iterable_fold => 2,
            FnOutputPlan::Unfold(_) => 1,
            FnOutputPlan::Value(_) => 0,
        };
        slots += 1; // binding-error sink
        if self.error.is_some() {
            slots += 1;
        }
        slots
    }
}

pub(crate) fn kotlin_jvm_slots(ty: &str) -> usize {
    if !ty.ends_with('?') && matches!(ty, "Long" | "Double") {
        2
    } else {
        1
    }
}

/// Classify one effective parameter. `expanded` disables only the Vec-build
/// collection helper; recursive data-class leaves are valid in constructor
/// expansions and reuse the same Rust/Kotlin lowering as ordinary parameters.
/// Which place in the exported function this leaf is.
///
/// One answer for every caller: the callback shortcut below also runs for an
/// expansion leaf, and hardcoding `Role::Param` there labelled an expanded
/// callback leaf as a parameter — undoing the rule the ordinary path states
/// (#622 review).
fn leaf_role(
    expanded: bool,
    position: usize,
    leaf_index: usize,
) -> prebindgen_registry::recipe::Role {
    use prebindgen_registry::recipe::Role;
    match expanded {
        false => Role::Param { index: position },
        true => Role::ExpansionLeaf {
            param: position,
            leaf: leaf_index,
        },
    }
}

#[allow(clippy::too_many_arguments)]
fn classify_leaf(
    ext: &Declarations,
    registry: &Registry,
    ident: &syn::Ident,
    reading: &TypeRef,
    expanded: bool,
    source_param: &syn::Ident,
    // The exported function this leaf belongs to, the SOURCE parameter
    // position it came from, and — when that parameter expanded — which of its
    // leaves this is.
    owner: &syn::Ident,
    position: usize,
    leaf_index: usize,
) -> Result<std::rc::Rc<PlanLeaf>, PlanError> {
    use prebindgen_registry::recipe::{Compiler, Crossing, Direction, Site};
    // `impl Fn(args)` never reaches the compiler, for the reason
    // `JniGen::compile_crossing` gives: a callback is answered whole, because a
    // JniGen callback ARGUMENT does not always have a conversion of its own —
    // a sealed class reaches the JVM as a selector plus the live arm's slots.
    // Driving the derived callback recipe here would ask for the one conversion
    // that does not exist. This carve-out goes when the arms are recipes a
    // callback can be composed from.
    if let Some(args) = reading.callback_args() {
        // `SpecKey` is a memo key and holds `TypeKey`s, so the args reach it as
        // each arg reading's own identity.
        let iface = ext.iface_spec(registry, &SpecKey::callback(args));
        let entry = ext.in_frag(reading).ok_or_else(|| PlanError::Unresolved {
            ty: Box::new(reading.clone()),
        })?;
        let pipeline = entry.pipeline(
            prebindgen_registry::recipe::Direction::Construct,
            prebindgen_registry::recipe::Mode::Owned,
        );
        let wire_ident = ident.clone();
        let native = std::rc::Rc::new(vec![NativeParam {
            rust_ident: wire_ident.clone(),
            rust_wire: annotate_jobject_with_lifetime(pipeline.wire(), "a").to_token_stream(),
            kt_name: kt_param_name(&ident.to_string()),
            kt_wire: entry.metadata.kotlin_name.clone(),
            jvm_slots: 1,
        }]);
        let leaf = std::rc::Rc::new(PlanLeaf {
            reading: reading.clone(),
            kt_name: kt_param_name(&ident.to_string()),
            kt_public: None,
            optional: reading.optional_inner().is_some(),
            as_enum_value: ext.is_kotlin_enum_reading(reading),
            enum_niche: crate::jni::compile::option_enum_niche(
                ext,
                reading,
                prebindgen_registry::recipe::Direction::Construct,
            ),
            native,
            pipeline,
            rust: RustParamOp::Pipeline { wire_ident },
            kotlin: KotlinParamOp::Callback { iface },
        });
        // This parameter never reaches `Compiler::site`, so nothing else
        // states it canonically — but every fact a site plan needs is here:
        // the place in the function, the crossing, and the fragment that
        // answers it, whose own id carries the recipe that was selected.
        // Stated here rather than left out, so the canonical site set covers
        // this path too (#622 review).
        {
            let fragment = entry.plan();
            ext.site_plans
                .borrow_mut()
                .push(std::rc::Rc::new(crate::jni::compile::site_plan(
                    &fragment,
                    &prebindgen_registry::recipe::Bound {
                        site: Site {
                            owner: owner.clone(),
                            role: leaf_role(expanded, position, leaf_index),
                        },
                        crossing: Crossing::new(reading.clone(), Direction::Construct),
                        recipe: fragment.id().recipe().clone(),
                        origin: prebindgen_registry::recipe::Origin::Function,
                    },
                    crate::jni::compile::JAbiLeaves::Params(leaf.clone()),
                )));
            // And one site per value the callback delivers. A delivered
            // argument is a place in THIS function, which is what
            // `Role::CallbackArg` names and why the registry keeps it apart
            // from the `Role::Part` the shared callback recipe is compiled at:
            // several functions taking one `impl Fn` signature share the
            // delivery, and each states its own site over it.
            //
            // The recipe is `site_bindings`' answer, not one composed here. A
            // fabricated `Bound` is what the first two attempts at this
            // produced, and a site that misreports which row it took is worse
            // than an absent one (#622 review).
            if let Some(plan) = entry.rust.invoke_plan() {
                let arguments = match fragment.converter().shape() {
                    prebindgen_registry::generation::ShapePlan::Invoke { arguments, .. } => {
                        arguments.as_slice()
                    }
                    _ => &[],
                };
                for (arg, ty) in args.iter().enumerate() {
                    let (Some(edge), Some(abi)) = (arguments.get(arg), plan.arg_abi(arg)) else {
                        continue;
                    };
                    let crossing = Crossing::new(ty.clone(), Direction::Deconstruct);
                    let site = Site {
                        owner: owner.clone(),
                        role: prebindgen_registry::recipe::Role::CallbackArg {
                            param: position,
                            arg,
                        },
                    };
                    let Some(bound) =
                        ext.site_bindings()
                            .resolve(&site, &crossing, ext.recipe_table())
                    else {
                        continue;
                    };
                    ext.site_plans.borrow_mut().push(std::rc::Rc::new(
                        crate::jni::compile::callback_arg_site(&bound, edge, abi),
                    ));
                }
            }
        }
        return Ok(leaf);
    }
    // The compiler, resumed over what the build already compiled. Every
    // fragment this site's recipe needs is in that store, so `site` finds them
    // rather than building them again — and the plan it wraps them in is the
    // one hook the registry calls per site rather than per crossing.
    let mut compiler = Compiler::resume(
        registry.flat(),
        ext.recipe_table(),
        ext.site_bindings(),
        ext.compiled.borrow().clone(),
    );
    let mut adapter = crate::jni::compile::JCompile {
        decls: ext,
        registry,
        declared_return: None,
        site: Some(crate::jni::compile::PlanSite::Param(
            crate::jni::compile::ParamSite {
                ident: ident.clone(),
                expanded,
            },
        )),
    };
    // A site is a place in an exported function, which is what the registry's
    // `Site` means: `owner` is the function and `role` names the position in
    // it. This named the PARAMETER as owner and always index 0, so two
    // functions with a parameter of the same name froze the same identity, and
    // every leaf of an expanded parameter collided with its siblings (#622
    // review).
    //
    // `Role::Param`'s index is the position in the SOURCE parameter list, so
    // an expansion's leaves cannot be numbered as parameters: doing that names
    // positions the function does not have and attaches one parameter's site to
    // another's crossing. They are `ExpansionLeaf`s of the parameter that
    // expanded, which is the same question `CallbackArg` already answers for a
    // callback's arguments (#622 review).
    let site = Site {
        owner: owner.clone(),
        role: leaf_role(expanded, position, leaf_index),
    };
    let crossing = Crossing::new(reading.clone(), Direction::Construct);
    let use_pair = crate::jni::compile::optional_pair_plan_candidate(ext, reading)
        && ext
            .recipe_table()
            .key_of(&crossing.key(), &crate::jni::recipes::pair())
            .is_some();
    let planned = if use_pair {
        compiler.site_recipe(&mut adapter, site, crossing, &crate::jni::recipes::pair())
    } else {
        compiler.site(&mut adapter, site, crossing)
    };
    *ext.compiled.borrow_mut() = compiler.finish();
    match planned {
        Ok(Some(plan)) => plan.param().ok_or_else(|| PlanError::Unresolved {
            ty: Box::new(reading.clone()),
        }),
        // A site the bindings omitted, which JniGen never declares.
        Ok(None) => Err(PlanError::Unresolved {
            ty: Box::new(reading.clone()),
        }),
        Err(prebindgen_registry::recipe::CompileError::Adapter(
            crate::jni::compile::JErr::Plan(e),
        )) => Err(*e),
        // A refusal or a table disagreement, which reach this path only for a
        // type with no conversion — the same failure the entry lookup reported.
        Err(_) => Err(if expanded {
            PlanError::UnresolvedLeaf {
                ty: Box::new(reading.clone()),
                param: source_param.clone(),
            }
        } else {
            PlanError::Unresolved {
                ty: Box::new(reading.clone()),
            }
        }),
    }
}

/// Compile the return of `func` as one site.
///
/// `declared` is what the signature says the function returns, when that
/// differs from the value that crosses — a `Return`-delivery convert crosses
/// what its decomposition produced. `None` when the two are the same.
fn return_site(
    ext: &Declarations,
    registry: &Registry,
    func: &syn::Ident,
    target: &TypeRef,
    declared: Option<TypeRef>,
) -> Option<crate::jni::compile::JPlan> {
    use prebindgen_registry::recipe::{Compiler, Crossing, Direction, Role, Site};
    let mut compiler = Compiler::resume(
        registry.flat(),
        ext.recipe_table(),
        ext.site_bindings(),
        ext.compiled.borrow().clone(),
    );
    let mut adapter = crate::jni::compile::JCompile {
        decls: ext,
        registry,
        declared_return: declared,
        site: Some(crate::jni::compile::PlanSite::Return),
    };
    let site = Site {
        owner: func.clone(),
        role: Role::Return,
    };
    let crossing = Crossing::new(target.clone(), Direction::Deconstruct);
    let planned = compiler.site(&mut adapter, site, crossing);
    *ext.compiled.borrow_mut() = compiler.finish();
    planned.ok().flatten()
}

/// Freeze the exact Rust-to-JNI delivery selected for one domain-error plan.
///
/// Error decompositions may be function-unique value-form walks, so their
/// leaves are always compiled directly through the registry. A model-derived
/// Product/Optional/Choice additionally retains its composed converter; the
/// renderer may use it only when its layout matches the declared error leaves.
fn build_error_output(
    ext: &Declarations,
    registry: &Registry,
    unfold: crate::unfold::UnfoldPlan,
) -> Result<ErrorOutputPlan, PlanError> {
    let wires = std::rc::Rc::new(
        crate::jni::compile::freeze_out_wires(ext, registry, &unfold.leaves).map_err(|_| {
            PlanError::UnresolvedOutput {
                ty: Box::new(unfold.source.clone()),
            }
        })?,
    );
    let delivered = if unfold.by_ref {
        unfold.source.borrowed()
    } else {
        unfold.source.clone()
    };
    let delivered = if unfold.is_optional_base() {
        delivered.optional()
    } else {
        delivered
    };
    let chain = if unfold.fixed_builder && unfold.hoists.is_empty() {
        crate::jni::compile::freeze_output_chain(ext, registry, &delivered).map_err(|_| {
            PlanError::UnresolvedOutput {
                ty: Box::new(delivered),
            }
        })?
    } else {
        None
    };
    let delivery =
        crate::jni::emit::FrozenDelivery::new(ext, registry, &unfold, wires.clone(), chain.clone());
    Ok(ErrorOutputPlan {
        unfold,
        wires,
        chain,
        delivery,
    })
}

/// Lower the output side. Mirrors the historical derivations exactly:
/// the Rust facts (`is_convert`, target type, wire) from the former
/// `lower_output`/`output_target_type` (emit/wrapper.rs), the Kotlin
/// declared-surface facts from `classify_return`'s inputs
/// (render_extern_decl's `ret_decl` reconstruction).
fn build_output(
    ext: &Declarations,
    registry: &Registry,
    f: &prebindgen_registry::flat::Function,
    unfold_plan: Option<&crate::unfold::UnfoldPlan>,
    error_plan: Option<&crate::unfold::UnfoldPlan>,
) -> Result<FnOutputPlan, PlanError> {
    use crate::unfold::{Delivery, UnfoldShape};
    let ident = &f.name;

    // Callback delivery: the return is decomposed to a foreign builder/fold
    // lambda; no output converter runs and the wire is the erased `JObject`.
    if let Some(plan) = unfold_plan.filter(|p| p.delivery == Delivery::Callback) {
        let iterable_fold = super::is_iterable_fold(&plan.shape);
        let optional = matches!(plan.shape, UnfoldShape::Optional(..));
        let fixed_builder = plan.fixed_builder;
        // The generic-surface rule (see `classify_output`): a fixed builder
        // is not generic; an `Iterable` fold — bare or `Optional`-wrapped —
        // folds with `<A>` (the wrapped form returns `A?`, null = `None`);
        // everything else builds with `<R>`.
        let generic = if fixed_builder {
            None
        } else if iterable_fold {
            Some("A")
        } else {
            Some("R")
        };
        let iface = if iterable_fold {
            folder_iface_for_plan(ext, registry, plan)
        } else {
            let decon = plan
                .decon
                .clone()
                .expect("record-built plan carries its DeconId");
            ext.iface_spec(registry, &SpecKey::Builder(decon))
        };
        let decon = plan.decon.as_ref().map(|id| {
            std::rc::Rc::new(
                ext.unfolded()
                    .decon_plans
                    .get(id)
                    .unwrap_or_else(|| panic!("unfold plan names unknown deconstructor `{id:?}`"))
                    .clone(),
            )
        });
        let (wires, chain, element_pipeline) = if let Some(element) = &plan.element {
            let pipeline = crate::jni::compile::freeze_output_pipeline(ext, registry, element)
                .map_err(|_| PlanError::UnresolvedOutput {
                    ty: Box::new(element.clone()),
                })?;
            (std::rc::Rc::new(Vec::new()), None, Some(pipeline))
        } else {
            let expected = crate::jni::compile::OutWire::from_leaves(&plan.leaves);
            let delivered = if plan.by_ref {
                plan.source.borrowed()
            } else {
                plan.source.clone()
            };
            // Only an Optional directly around the decomposed base belongs in
            // the converter chain. `optional` also covers
            // `Option<Vec<T>>`, whose presence gates the fold outside the
            // per-element `T` chain.
            let delivered = if plan.is_optional_base() {
                delivered.optional()
            } else {
                delivered
            };
            let composed = return_site(ext, registry, ident, &plan.source, None)
                .and_then(|site| site.decomposed())
                .filter(|wires| {
                    wires.len() == expected.len()
                        && wires
                            .iter()
                            .zip(&expected)
                            .all(|(left, right)| left.same_delivery(right))
                });
            let wires = match composed {
                Some(wires) => wires,
                None => std::rc::Rc::new(
                    crate::jni::compile::freeze_out_wires(ext, registry, &plan.leaves).map_err(
                        |_| PlanError::UnresolvedOutput {
                            ty: Box::new(plan.source.clone()),
                        },
                    )?,
                ),
            };
            // Chain availability depends on the exact value the delivery owns,
            // including borrow and outer Optional mode. Freeze that answer now;
            // the site binding itself intentionally names the core Product.
            let chain = if fixed_builder && plan.hoists.is_empty() {
                crate::jni::compile::freeze_output_chain(ext, registry, &delivered).map_err(
                    |_| PlanError::UnresolvedOutput {
                        ty: Box::new(delivered),
                    },
                )?
            } else {
                None
            };
            (wires, chain, None)
        };
        let delivery = crate::jni::emit::FrozenDelivery::new(
            ext,
            registry,
            plan,
            wires.clone(),
            chain.clone(),
        );
        return Ok(FnOutputPlan::Unfold(Box::new(UnfoldOutputPlan {
            wires,
            chain,
            delivery,
            element_pipeline,
            iterable_fold,
            optional,
            fixed_builder,
            whole_element: plan.element.is_some(),
            decon,
            generic,
            iface,
        })));
    }

    // Value return. The conversion target: the converted single value for a
    // `Return` delivery, the `Result` Ok type when an error plan peels, else
    // the function's own return.
    let is_convert = unfold_plan.is_some();
    // The element normalizes an elided return and a written `-> ()` to one
    // `Unit` reading, so there is no `ReturnType` match here.
    // The `Ok` side off `TypeKind::Fallible`, where `result_ok_type` found the
    // `Result` in a path first.
    let ok_ty = error_plan
        .and_then(|_| f.ret.fallible_parts())
        .map(|(ok, _)| ok);
    // All three candidates are readings: a plan's `convert_out_ty`, the `Ok`
    // side of the return, and the return itself. This met them as spellings
    // because one of them used to be a node.
    let target_ty: &prebindgen_registry::flat::TypeRef = match unfold_plan {
        Some(p) => p
            .convert_out_ty
            .as_ref()
            .expect("Return delivery carries convert_out_ty"),
        None => ok_ty.unwrap_or(&f.ret),
    };
    // Two failures, told apart. Holding a reading is not the same as being in
    // the type table — `target_ty` is reached through a plan or a peel, so it
    // may genuinely have no cell — and "not registered" wants different advice
    // from "registered, no converter". Collapsed into one `and_then`, both got
    // the `output_wrapper` message and the first one got it wrong.
    let Some(target) = registry.reading(&target_ty.key()) else {
        return Err(PlanError::UnknownOutputType {
            ty: target_ty.key(),
        });
    };
    // The site's own recipe, compiled through the hook. `Compiler::site` is what
    // makes this a site rather than a lookup: it picks the recipe, builds the
    // fragment, checks the value's validity against what a return tolerates —
    // the JVM keeps what it is given, so a borrowed one is refused here rather
    // than emitted — and hands the fragment to `Compile::plan`.
    //
    // The Kotlin surface classifies the DECLARED return: `convert_out_ty` for a
    // convert, else the signature's own output. Not `target_ty` — the Kotlin
    // error peel rides the conversion's `value_reading`, so the full
    // `Result<T, E>` is what the surface reads, and that is the one fact the
    // crossing cannot carry.
    let plan = return_site(
        ext,
        registry,
        ident,
        &target,
        is_convert.then(|| target_ty.clone()),
    )
    .and_then(|p| p.returned())
    .ok_or_else(|| PlanError::UnresolvedOutput {
        ty: Box::new(target.clone()),
    })?;
    // Read off the shared plan: it is the site's own answer, and the plan
    // built below adds the convert delivery to it rather than replacing it.
    let (pipeline, surface, is_enum, is_option_enum, enum_niches) = (
        plan.pipeline.clone(),
        plan.surface.clone(),
        plan.is_enum,
        plan.is_option_enum,
        plan.enum_niches.clone(),
    );
    // The convert shortcut reaches the plan's single leaf itself rather than
    // through the leaf encoder, and qualifies the accessor calls on that reach
    // — so those origins are frozen here, with the leaf they belong to.
    let convert_delivery = is_convert.then(|| {
        let unfold = unfold_plan.expect("a convert is a Return delivery, which carries its plan");
        crate::jni::emit::FrozenDelivery::new(
            ext,
            registry,
            unfold,
            std::rc::Rc::new(
                unfold
                    .leaves
                    .iter()
                    .map(crate::jni::compile::OutWire::from_leaf)
                    .collect(),
            ),
            None,
        )
    });
    // The plan the emitters read: the site's answer with the convert delivery
    // attached. This is the one frozen into the canonical site, because it is
    // the authoritative one — freezing the hook's intermediate carried a
    // `convert_delivery` of `None` while this object owned the real one.
    let final_plan = std::rc::Rc::new(ValueOutputPlan {
        site: plan.site.clone(),
        is_convert,
        pipeline,
        surface,
        is_enum,
        is_option_enum,
        enum_niches,
        convert_delivery,
    });
    if let Some((bound, fragment)) = &final_plan.site {
        ext.site_plans
            .borrow_mut()
            .push(std::rc::Rc::new(crate::jni::compile::whole_return_site(
                bound,
                fragment,
                final_plan.clone(),
            )));
    }
    Ok(FnOutputPlan::Value(final_plan))
}

/// Whether a return surfaces as a Kotlin enum class, and whether an optional
/// one does — the only two things the old `canonical` spelling was consulted
/// for, now answered at the one place that can see how it was obtained.
#[derive(Clone, Copy)]
pub(crate) struct EnumSurface {
    pub is_enum: bool,
    pub is_option_enum: bool,
}

impl ReturnSurface {
    /// Classify a declared return type. Returns the surface plus the
    /// canonical (`value_reading`-peeled) type the enum probes run over —
    /// the single peel that subsumed both `classify_return`'s inline peel
    /// and the former `canonical_return_ty`.
    pub fn classify(
        ext: &Declarations,
        ret: &prebindgen_registry::flat::TypeRef,
    ) -> (Self, EnumSurface) {
        // The RETURN, as the model classified it. Both callers used to spell a
        // reading into a `-> #ty` fragment for this to take apart again, and
        // the `ReturnType::Default` arm was a unit the element already states.
        let outer_meta = ext.out_frag(ret).map(|e| e.metadata.clone());
        // Unit returns (incl. `Result<()>`, whose inner identity rides
        // `value_reading`) declare no Kotlin return type. Falling back to the
        // declared return is not a miss: `None` means the crossing itself is
        // the surfaced value.
        let stored = outer_meta.as_ref().and_then(|m| m.value_reading.as_ref());
        let is_unit = match stored {
            Some(t) => matches!(t.kind(), prebindgen_registry::flat::TypeKind::Unit),
            None => matches!(ret.kind(), prebindgen_registry::flat::TypeKind::Unit),
        };
        // The two enum questions, answered where BOTH branches are in view.
        // They used to ride out on a `canonical: syn::Type` built by spelling
        // the reading — and then `option_inner_type` peeled that spelling to
        // ask about its inner, which is the layer-off-a-spelling mistake #273
        // is named for: the model erases wrappers the spelling still shows.
        // A stored `value_reading` is a classified reading. No Rust node is
        // reconstructed or peeled here.
        let (is_enum, is_option_enum) = match stored {
            Some(t) => {
                let mut inner = t;
                let mut optional = false;
                while let Some(next) = inner.optional_inner() {
                    optional = true;
                    inner = next;
                }
                (
                    ext.is_kotlin_enum_key(&t.key()),
                    optional && ext.is_kotlin_enum_key(&inner.key()),
                )
            }
            // `is_kotlin_enum_reading` is NOT the peer here: it probes THROUGH
            // the layers, so `Option<Level>` answers true for the first
            // question, where the node version keys on the whole type and
            // answers false. The first question wants the exact type's
            // identity; the second requires at least one Optional and asks the
            // terminal inner type after peeling every Optional layer.
            None => {
                let mut inner = ret;
                let mut optional = false;
                while let Some(next) = inner.optional_inner() {
                    optional = true;
                    inner = next;
                }
                (
                    ext.is_kotlin_enum_key(&ret.key()),
                    optional && ext.is_kotlin_enum_key(&inner.key()),
                )
            }
        };
        let canonical = EnumSurface {
            is_enum,
            is_option_enum,
        };
        if is_unit {
            return (Self::Unit, canonical);
        }
        // Projection return (opaque handle or `ULong`): read the folded
        // `Projection` the type-unfolding mechanism propagated onto this
        // return type's converter metadata — one source of truth, no
        // shape-specific peeling.
        if let Some(h) = outer_meta.as_ref().and_then(|m| m.projection.clone()) {
            let leaf_fqn = projection_leaf_kt(ext, &h).map(|t| t.to_string());
            return (
                Self::Projected {
                    projection: h,
                    leaf_fqn,
                },
                canonical,
            );
        }
        // Non-opaque: the resolved entry's Kotlin name — the rank-N handler
        // propagates `ZResult<T>` / `Option<T>` / `Vec<T>` derivations
        // alongside the wire, so no peel-and-fallback chain is needed here.
        match outer_meta.and_then(|m| m.kotlin_name) {
            Some(kt) => (Self::Plain { kt }, canonical),
            None => (Self::Skip, canonical),
        }
    }
}
