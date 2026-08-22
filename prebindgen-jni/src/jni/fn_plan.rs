//! Input side of the per-function lowered binding plan (issue #90).
//!
//! [`JniFunctionPlan`] classifies every input parameter of a bound function
//! ONCE, deterministically over `(ext, registry, f)`. The three coordinated
//! emission sites — the Rust `extern "C"` wrapper (`emit_input_param`), the
//! Kotlin wrapper classifier (`classify_params`), and the `JNINative`
//! `external fun` declaration (`render_extern_decl`) — all consume the same
//! [`InputKind`] decision instead of re-running their own copies of the
//! probe cascade, so the wire arity, types, and call forms agree by
//! construction. The pattern generalizes [`build_struct_plan`]'s field-level
//! plan to function granularity; the output side follows in a later stage.

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
    pub params: Vec<PlanParam>,
    pub output: FnOutputPlan,
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

/// How a source parameter crosses the boundary. The single leaf is boxed to
/// keep the variants near the same size (a [`PlanLeaf`] embeds whole
/// sub-plans; the `Expanded` payload is just a `Vec` header).
pub(crate) enum ParamForm {
    /// Ordinary parameter — one classified leaf.
    Single(Box<PlanLeaf>),
    /// Constructor-expansion ([`FoldPlan`] declared for this `(fn, param)`):
    /// the wire form is the plan's flattened leaves, classified individually;
    /// the Rust wrapper folds them back into the built value. Leaves use the
    /// same recursive data-class probe as ordinary parameters (vec-build
    /// remains a source-parameter-only collection optimization), so all three
    /// sites agree on the leaf wire.
    Expanded(Vec<PlanLeaf>),
}

/// One classified effective parameter (a source param, or one expansion leaf).
pub(crate) struct PlanLeaf {
    /// The leaf's **reading** — classification and spelling in one value, so
    /// the two cannot disagree and no consumer has to look the type up. Spell
    /// with `emit.spell(reading)` in an emission callback.
    pub reading: TypeRef,
    /// Kotlin parameter name (`kt_param_name(ident)`: camelCase +
    /// hard-keyword escaping) — shared by the wrapper signature and the
    /// `external fun` declaration.
    pub kt_name: String,
    /// Typed-wrapper surface type: the projection's Kotlin FQN for
    /// handle/value projections, else the resolved entry's Kotlin name.
    /// `None` when the metadata lacks a name (the Kotlin wrapper renderer
    /// skips the function — the escape-hatch path) and for [`InputKind::
    /// Callback`] (typed from the interface spec at render time).
    pub kt_public: Option<KtType>,
    /// The resolved entry's raw `metadata.kotlin_name` — the type the
    /// `JNINative` extern declares for pass-through leaves (for projections
    /// this is the erased wire name, not the typed surface).
    pub kt_meta: Option<KtType>,
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
    pub kind: InputKind,
}

/// The classified crossing form. Branches are mutually exclusive by
/// construction (each probe rejects the shapes the others accept), so the
/// probe order is canonical, not load-bearing.
pub(crate) enum InputKind {
    /// `impl Fn(args)` callback: erased `Any` on the wire. `iface` is the
    /// typed `fun interface` spec (memoized under [`SpecKey::Callback`] —
    /// the same allocation the trampoline and the declaration emitter read);
    /// `None` = underivable, the Kotlin wrapper renderer skips.
    Callback { iface: Option<Arc<IfaceSpec>> },
    /// `&[T]` / `Vec<T>` of a flattenable data_class: a single `jlong`
    /// Vec-handle on the wire, built by pushing element leaves.
    /// The element as a **reading**, and the CANONICAL one: the vec-helper plan
    /// and the element key are both taken from it, and generated Rust spells
    /// `emit.spell(elem)`. `elem_wrappers` is what the storage therefore does not
    /// carry, put back per element on consumption (#296) — empty for the
    /// ordinary case, and a list rather than a second `TypeRef` because every
    /// variant of this enum pays its size.
    VecBuild {
        elem: TypeRef,
        by_ref: bool,
        elem_wrappers: Vec<&'static str>,
    },
    /// Bare `Option<primitive>` / `Option<enum>`: a decoupled
    /// `(present: jboolean, value: <wire>)` pair.
    OptionScalar(OptionScalarInputPlan),
    /// Flattenable data_class: the field leaves cross as separate wire params.
    FlattenStruct(FlatInputPlan),
    /// Lockable opaque-handle projection (`jlong` wire). `direct` is
    /// [`KotlinMeta::is_direct_handle`] — `true` only for the bare
    /// `T`/`&T` shape, the by-value consume fast-path trigger.
    Handle { direct: bool },
    /// Rust `u64`: typed Kotlin `ULong`, raw JNI `Long`. The wrapper passes
    /// the bit-preserving `toLong()` representation and takes no lock.
    Unsigned64 { niche: Option<String> },
    /// Everything else: the resolved entry's converter/wire as-is.
    Plain,
}

/// How the return value crosses the boundary. Mirrors the unfold plan's
/// [`Delivery`](prebindgen_registry::unfold::Delivery), resolved per function:
/// `Unfold` = callback delivery (builder/fold lambda, erased `Any?` wire);
/// `Value` = everything else, including the `Return`-delivery convert.
pub(crate) enum FnOutputPlan {
    Unfold(UnfoldOutputPlan),
    Value(Box<ValueOutputPlan>),
}

/// Callback-delivery shape facts, read off the fn's `UnfoldPlan` once so the
/// Rust builder param, the erased extern params, and the typed Kotlin
/// builder/fold surface all branch on the same booleans.
pub(crate) struct UnfoldOutputPlan {
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
    /// `Return`-delivery convert (`convert_output`) — the wrapper returns the
    /// single deconstructed value through its ordinary output converter.
    pub is_convert: bool,
    /// The type whose output converter runs: `convert_out_ty` for a convert,
    /// the `Result` Ok type when an error plan peels, else the declared
    /// return. A **reading** — all three candidates are one, and the emitter
    /// asks it for the entry directly. Its entry is validated at plan build;
    /// the Rust emitter re-looks it up (`expect`) to keep the plan
    /// lifetime-free.
    pub target_ty: prebindgen_registry::flat::TypeRef,
    /// The resolved output entry's `destination` — the extern's wire return
    /// and the sentinel source.
    pub wire_ty: syn::Type,
    /// Kotlin surface classification over the **declared** return
    /// (`convert_out_ty` for a convert, else `f.sig.output` — not
    /// `target_ty`: the Kotlin error peel rides `value_rust_type`).
    pub surface: ReturnSurface,
    /// `enum_class` / `Option<enum>` probes over the canonical
    /// (`value_rust_type`-peeled) declared return. The extern decl uses them
    /// raw; the wrapper surface masks them with `!is_convert` (the historical
    /// `unfold.is_none()` gate).
    pub is_enum: bool,
    pub is_option_enum: bool,
}

/// The pure classification core of `classify_return` — no import
/// registration, no name shortening, no panics. The render adapter
/// (`render_return_surface`) maps it back to the historical
/// `(kt_return, projection)` pair, panicking on an unregistered projection
/// FQN exactly where `classify_return` always did (Kotlin render time).
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
                "JniGen::on_function: input type `{}` for `{}` is unresolved{at}",
                ty.key(),
                fn_ident,
            ),
            PlanError::UnresolvedLeaf { ty, param } => format!(
                "JniGen expand: leaf type `{}` (parameter `{}`) is unresolved{at}",
                ty.key(),
                param,
            ),
            PlanError::UnresolvedOutput { ty } => format!(
                "JniGen::on_function: return type `{}` of `{}` has no registered output \
                 converter — register one via `Declarations::output_wrapper(pat, |…| Some((ty, exc, body)))` \
                 (exc = `None` for non-throwing, `Some(parse_quote!(<full path>))` \
                  to bind a domain exception){at}",
                ty.key(),
                fn_ident,
            ),
            PlanError::UnknownOutputType { ty } => format!(
                "JniGen::on_function: return type `{}` of `{}` is not registered — the \
                 resolver never saw this type, so no converter can be selected for it. \
                 Declare the type (or the function that produces it) before binding `{}`",
                ty, fn_ident, fn_ident,
            ),
            PlanError::UnflattenableDataClass(error) => {
                format!("JniGen::on_function `{fn_ident}`: {}", error.message())
            }
            PlanError::JvmParameterLimit { slots } => format!(
                "JniGen::on_function `{fn_ident}`: flattened JNI signature uses {slots} JVM parameter slots (maximum 255, including the JNINative receiver); reduce the data-class shape or declare an intentional `data_class!(T).jobject_input()` boundary"
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
    // plan machinery (`Declarations::on_const`).
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
        let getter = const_expr_getter_fn(&decl.kotlin_name, &decl.ty, registry);
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

impl FnOutputPlan {
    /// The extern's wire return type: the erased builder result (`JObject`)
    /// for a callback delivery, the resolved entry's destination otherwise.
    /// Feeds `annotate_jobject_with_lifetime` + `sentinel_for_wire` on the
    /// Rust side.
    pub fn wire_ty(&self) -> syn::Type {
        match self {
            FnOutputPlan::Unfold(_) => syn::parse_quote!(jni::objects::JObject),
            FnOutputPlan::Value(v) => v.wire_ty.clone(),
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
    /// [`Declarations::iface_spec`]; drift is guarded externally by the byte-identity
    /// regen check (a plan change alters generated code).
    pub(crate) fn fn_plan(
        &self,
        registry: &Registry,
        f: &prebindgen_registry::flat::Function,
    ) -> Result<std::rc::Rc<JniFunctionPlan>, PlanError> {
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
        let onerror_iface = onerror_iface_spec(ext, registry, &f.name);
        // Output first: the Rust emitter historically resolved the output
        // before the inputs, so an unresolved-output failure takes precedence
        // over an unresolved-input one.
        let output = build_output(ext, registry, f)?;
        let mut params = Vec::new();
        // The element's parameters: each already a name and a `TypeRef`, so
        // there is no `FnArg`/`Pat` destructuring and no position that could
        // fail to yield a type.
        for param in &f.params {
            let ident = param.name.clone();
            let ty = param.ty.clone();

            let form = if let Some(plan) = registry
                .expansion_plans()
                .get(&(f.name.clone(), ident.clone()))
            {
                let mut leaves = Vec::new();
                for leaf in &plan.leaves {
                    // The lookup that stood here is gone: the fold leaf carries
                    // its own reading now, so there is nothing to fetch and
                    // nothing that can miss.
                    leaves.push(classify_leaf(
                        ext, registry, &leaf.name, &leaf.ty, /*expanded=*/ true, &ident,
                    )?);
                }
                ParamForm::Expanded(leaves)
            } else {
                ParamForm::Single(Box::new(classify_leaf(
                    ext, registry, &ident, &param.ty, /*expanded=*/ false, &ident,
                )?))
            };
            params.push(PlanParam { ident, ty, form });
        }
        let result = Self {
            jni_method,
            native_symbol,
            onerror_iface,
            params,
            output,
        };
        let slots = result.jvm_parameter_slots(ext, registry, f);
        if slots > 255 {
            return Err(PlanError::JvmParameterLimit { slots });
        }
        Ok(result)
    }

    /// The flattened effective-parameter view (expansion leaves inline) —
    /// the sequence the Kotlin wrapper and `external fun` declare, in order.
    pub fn leaves(&self) -> impl Iterator<Item = &PlanLeaf> {
        self.params.iter().flat_map(|p| match &p.form {
            ParamForm::Single(l) => std::slice::from_ref(&**l).iter(),
            ParamForm::Expanded(ls) => ls.iter(),
        })
    }

    fn jvm_parameter_slots(
        &self,
        ext: &Declarations,
        registry: &Registry,
        f: &prebindgen_registry::flat::Function,
    ) -> usize {
        // `JNINative` is a Kotlin object, so its external methods are instance
        // methods and the JVM counts the implicit receiver as one unit.
        let mut slots = 1usize;
        for leaf in self.leaves() {
            slots += match &leaf.kind {
                InputKind::FlattenStruct(plan) => plan
                    .leaves
                    .iter()
                    .map(|l| kotlin_jvm_slots(l.kt_wire_ty()))
                    .sum(),
                InputKind::OptionScalar(plan) => 1 + kotlin_jvm_slots(&plan.value_kt_type),
                InputKind::Handle { .. } | InputKind::VecBuild { .. } => 2,
                InputKind::Callback { .. } => 1,
                InputKind::Unsigned64 { .. } | InputKind::Plain => ext
                    .in_frag(&leaf.reading)
                    .and_then(|entry| JniPrim::from_wire(&entry.destination))
                    .map_or(1, |prim| match prim {
                        JniPrim::Long | JniPrim::Double => 2,
                        _ => 1,
                    }),
            };
        }
        slots += match &self.output {
            FnOutputPlan::Unfold(plan) if plan.iterable_fold => 2,
            FnOutputPlan::Unfold(_) => 1,
            FnOutputPlan::Value(_) => 0,
        };
        slots += 1; // binding-error sink
        if registry.error_plans().contains_key(&f.name) {
            slots += 1;
        }
        slots
    }
}

fn kotlin_jvm_slots(ty: &str) -> usize {
    if !ty.ends_with('?') && matches!(ty, "Long" | "Double") {
        2
    } else {
        1
    }
}

/// Classify one effective parameter. `expanded` disables only the Vec-build
/// collection helper; recursive data-class leaves are valid in constructor
/// expansions and reuse the same Rust/Kotlin lowering as ordinary parameters.
fn classify_leaf(
    ext: &Declarations,
    registry: &Registry,
    ident: &syn::Ident,
    reading: &TypeRef,
    expanded: bool,
    source_param: &syn::Ident,
) -> Result<PlanLeaf, PlanError> {
    use prebindgen_registry::recipe::{Compiler, Crossing, Direction, Role, Site};
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
        return Ok(PlanLeaf {
            reading: reading.clone(),
            kt_name: kt_param_name(&ident.to_string()),
            kt_public: None,
            kt_meta: ext
                .in_frag(reading)
                .and_then(|e| e.metadata.kotlin_name.clone()),
            optional: reading.optional_inner().is_some(),
            as_enum_value: ext.is_kotlin_enum_reading(reading),
            kind: InputKind::Callback { iface },
        });
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
    // The site names the parameter it is, and the position is the source
    // parameter's rather than a running count: a constructor expansion
    // contributes leaves the signature does not name, and they all belong to
    // the one parameter that expanded.
    let site = Site {
        owner: source_param.clone(),
        role: Role::Param { index: 0 },
    };
    let crossing = Crossing::new(reading.clone(), Direction::Construct);
    let planned = compiler.site(&mut adapter, site, crossing);
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

/// The values one function's decomposed return hands out, compiled through
/// `Compiler::site`.
///
/// Test support until the encode side takes it: this is the whole of what a
/// decomposed return site produces, and holding it to the expansion plan is
/// what says the two agree before anything depends on it.
#[cfg(test)]
pub(crate) fn decomposed_return_for_test(
    ext: &Declarations,
    registry: &Registry,
    func: &syn::Ident,
) -> Option<Vec<crate::jni::compile::OutWire>> {
    let f = registry.flat().function(func)?;
    let ret = f.ret.borrow_target().unwrap_or(&f.ret);
    let ret = ret.optional_inner().unwrap_or(ret);
    let ret = ret.sequence_elem().unwrap_or(ret);
    let ret = ret.borrow_target().unwrap_or(ret);
    return_site(ext, registry, func, ret, None).and_then(|p| p.decomposed())
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
) -> Result<FnOutputPlan, PlanError> {
    use prebindgen_registry::unfold::{Delivery, UnfoldShape};
    let ident = &f.name;
    let unfold_plan = registry.unfold_plans().get(ident);

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
        return Ok(FnOutputPlan::Unfold(UnfoldOutputPlan {
            iterable_fold,
            optional,
            fixed_builder,
            whole_element: plan.element.is_some(),
            generic,
            iface,
        }));
    }

    // Value return. The conversion target: the converted single value for a
    // `Return` delivery, the `Result` Ok type when an error plan peels, else
    // the function's own return.
    let is_convert = unfold_plan.is_some();
    // The element normalizes an elided return and a written `-> ()` to one
    // `Unit` reading, so there is no `ReturnType` match here.
    let error_plan = registry.error_plans().get(ident);
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
    // error peel rides the conversion's `value_rust_type`, so the full
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
    let ValueOutputPlan {
        wire_ty,
        surface,
        is_enum,
        is_option_enum,
        ..
    } = plan;
    Ok(FnOutputPlan::Value(Box::new(ValueOutputPlan {
        is_convert,
        target_ty: target_ty.clone(),
        wire_ty,
        surface,
        is_enum,
        is_option_enum,
    })))
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
    /// canonical (`value_rust_type`-peeled) type the enum probes run over —
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
        // Unit returns (incl. `ZResult<()>`, whose inner identity rides
        // `value_rust_type`) declare no Kotlin return type. The peeled type is
        // the one the converter's metadata stored — a canonical `syn::Type`,
        // so nothing is rebuilt here. Falling back to the declared return is
        // not a miss: `value_rust_type` is `None` exactly for plain values and
        // arity-0 converters, which have no inner identity to peel to.
        //
        // `value_rust_type` is a canonical `syn::Type` the ADAPTER composed,
        // which is why `is_unit` still asks it of a node; with no metadata the
        // question is the reading's own kind.
        let stored = outer_meta.as_ref().and_then(|m| m.value_rust_type.as_ref());
        let is_unit = match stored {
            Some(t) => crate::util::is_unit(t),
            None => matches!(ret.kind(), prebindgen_registry::flat::TypeKind::Unit),
        };
        // The two enum questions, answered where BOTH branches are in view.
        // They used to ride out on a `canonical: syn::Type` built by spelling
        // the reading — and then `option_inner_type` peeled that spelling to
        // ask about its inner, which is the layer-off-a-spelling mistake #273
        // is named for: the model erases wrappers the spelling still shows.
        // A stored `value_rust_type` is an adapter-composed node and keeps its
        // node-shaped answer; with no metadata the reading answers directly.
        let enum_probe = |t: &syn::Type| ext.is_kotlin_enum(t);
        let (is_enum, is_option_enum) = match stored {
            Some(t) => (
                enum_probe(t),
                prebindgen_registry::types_util::option_inner_type(t)
                    .map(|inner| enum_probe(&inner))
                    .unwrap_or(false),
            ),
            // `is_kotlin_enum_reading` is NOT the peer here: it probes THROUGH
            // the layers, so `Option<Level>` answers true for the first
            // question, where the node version keys on the whole type and
            // answers false. Both questions want the exact type's identity —
            // the second asks it of the optional's inner, which is what
            // `option_inner_type` did to the spelling.
            None => (
                ext.is_kotlin_enum_key(&ret.key()),
                ret.optional_inner()
                    .map(|inner| ext.is_kotlin_enum_key(&inner.key()))
                    .unwrap_or(false),
            ),
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
