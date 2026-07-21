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
    /// The onError sink interface — the typed `<Err>Handler` when an error
    /// plan exists, the global `JniErrorHandler` otherwise. Shared from the
    /// [`JniGen::iface_spec`] memo: one derivation feeds the Rust `__SINK_*`
    /// statics, the Kotlin `onError` wiring, and the interface declaration,
    /// so the FQN/descriptor pair of the cached `run` lookup cannot drift.
    /// `None` = underivable (the Rust emitter panics, the Kotlin renderer
    /// skips).
    pub onerror_iface: Option<Arc<IfaceSpec>>,
    pub params: Vec<PlanParam>,
    pub output: FnOutputPlan,
}

/// One source parameter: the ident/type as written plus its lowered form.
pub(crate) struct PlanParam {
    pub ident: syn::Ident,
    pub ty: syn::Type,
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
    /// restricted probe set of the fold path (no struct-flatten / vec-build
    /// nesting), so all three sites agree on the leaf wire.
    Expanded(Vec<PlanLeaf>),
}

/// One classified effective parameter (a source param, or one expansion leaf).
pub(crate) struct PlanLeaf {
    pub ty: syn::Type,
    /// Kotlin parameter name (`kt_param_name(ident)`: camelCase +
    /// hard-keyword escaping) — shared by the wrapper signature and the
    /// `external fun` declaration.
    pub kt_name: String,
    /// Typed-wrapper surface type: the projection's Kotlin FQN for
    /// handle/value projections, else the resolved entry's Kotlin name.
    /// `None` when the metadata lacks a name (the Kotlin wrapper renderer
    /// skips the function — the escape-hatch path) and for [`InputKind::
    /// Callback`] (typed from the interface spec at render time).
    pub kt_public: Option<kt::KtType>,
    /// The resolved entry's raw `metadata.kotlin_name` — the type the
    /// `JNINative` extern declares for pass-through leaves (for projections
    /// this is the erased wire name, not the typed surface).
    pub kt_meta: Option<kt::KtType>,
    /// Raw `is_option_type(ty)` — each site applies its own nullability rule
    /// (handles stay non-null `Long` on the extern but `T?` on the surface).
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
    VecBuild { elem: syn::Type, by_ref: bool },
    /// Bare `Option<primitive>` / `Option<enum>`: a decoupled
    /// `(present: jboolean, value: <wire>)` pair.
    OptionScalar(OptionScalarInputPlan),
    /// Flattenable data_class: the field leaves cross as separate wire params.
    FlattenStruct(FlatInputPlan),
    /// Lockable opaque-handle projection (`jlong` wire). `direct` is
    /// [`KotlinMeta::is_direct_handle`] — `true` only for the bare
    /// `T`/`&T` shape, the by-value consume fast-path trigger.
    Handle { direct: bool },
    /// Non-lockable value projection (`value_blob`): the call site passes the
    /// unwrapped inline-class `field`; the extern keeps the erased wire.
    ValueUnwrap { field: String },
    /// Everything else: the resolved entry's converter/wire as-is.
    Plain,
}

/// How the return value crosses the boundary. Mirrors the unfold plan's
/// [`Delivery`](crate::api::core::unfold::Delivery), resolved per function:
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
    /// otherwise. Shared from the [`JniGen::iface_spec`] memo: one
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
    /// return. Its entry is validated at plan build; the Rust emitter
    /// re-looks it up (`expect`) to keep the plan lifetime-free.
    pub target_ty: syn::Type,
    /// The resolved output entry's `destination` — the extern's wire return
    /// and the sentinel source.
    pub wire_ty: syn::Type,
    /// Kotlin surface classification over the **declared** return
    /// (`convert_out_ty` for a convert, else `f.sig.output` — not
    /// `target_ty`: the Kotlin error peel rides `value_rust_key`).
    pub surface: ReturnSurface,
    /// `enum_class` / `Option<enum>` probes over the canonical
    /// (`value_rust_key`-peeled) declared return. The extern decl uses them
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
    /// Projection return (opaque handle / value class). `leaf_fqn` is the
    /// resolved Kotlin FQN; `None` = unregistered (the adapter panics).
    Projected {
        projection: Projection,
        leaf_fqn: Option<String>,
    },
    /// Plain return typed by the entry's resolved Kotlin name (unshortened —
    /// the adapter registers/shortens at render time).
    Plain { kt: kt::KtType },
}

/// Plan construction failure. The validation boundary
/// ([`validate_bindings`]) reports every failure before any artifact is
/// written; the Rust emitter keeps the same messages as panic backstops and
/// the Kotlin renderers map to `None` (skip).
#[derive(Debug)]
pub(crate) enum PlanError {
    /// `registry.input_entry` has no converter for a source param type.
    Unresolved { ty: TypeKey },
    /// No converter for a constructor-expansion leaf type.
    UnresolvedLeaf { ty: TypeKey, param: syn::Ident },
    /// `registry.output_entry` has no converter for the output target type.
    UnresolvedOutput { ty: TypeKey },
}

impl PlanError {
    /// The historical emission-panic message for this failure, shared by the
    /// validation boundary and the Rust emitter's backstop panics so the
    /// wording cannot drift.
    pub fn message(&self, fn_ident: &syn::Ident) -> String {
        match self {
            PlanError::Unresolved { ty } => format!(
                "JniGen::on_function: input type `{}` for `{}` is unresolved",
                ty, fn_ident,
            ),
            PlanError::UnresolvedLeaf { ty, param } => format!(
                "JniGen expand: leaf type `{}` (parameter `{}`) is unresolved",
                ty, param,
            ),
            PlanError::UnresolvedOutput { ty } => format!(
                "JniGen::on_function: return type `{}` of `{}` has no registered output \
                 converter — register one via `JniGen::output_wrapper(pat, |…| Some((ty, exc, body)))` \
                 (exc = `None` for non-throwing, `Some(parse_quote!(<full path>))` \
                  to bind a domain exception)",
                ty, fn_ident,
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
/// [`Prebindgen::validate_resolved`]: crate::api::core::prebindgen::Prebindgen::validate_resolved
pub(crate) fn validate_bindings(
    ext: &JniGen,
    registry: &Registry<KotlinMeta>,
) -> Result<(), String> {
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
    let mut fn_idents: Vec<&syn::Ident> = registry.functions.keys().collect();
    fn_idents.sort();
    for ident in fn_idents {
        if !declared.contains(ident) {
            continue;
        }
        let (item_fn, _) = &registry.functions[ident];
        match ext.fn_plan(registry, item_fn) {
            Ok(plan) => record_symbol(&plan.native_symbol, ident.to_string(), &mut errors),
            Err(e) => errors.push(e.message(ident)),
        }
    }

    // Declared consts: their synthetic nullary getters run through the same
    // plan machinery (`JniGen::on_const`).
    if let Some(declared_consts) = ext.declared_consts() {
        let mut const_idents: Vec<&syn::Ident> = registry.consts.keys().collect();
        const_idents.sort();
        for ident in const_idents {
            if *ident == "_" || !declared_consts.contains(ident) {
                continue;
            }
            let (item_const, _) = &registry.consts[ident];
            let getter = const_getter_fn(item_const);
            match ext.fn_plan(registry, &getter) {
                Ok(plan) => record_symbol(&plan.native_symbol, ident.to_string(), &mut errors),
                Err(e) => errors.push(e.message(&getter.sig.ident)),
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
        let getter = const_expr_getter_fn(&decl.kotlin_name, &decl.ty);
        match ext.fn_plan(registry, &getter) {
            Ok(plan) => record_symbol(&plan.native_symbol, decl.kotlin_name.clone(), &mut errors),
            Err(e) => errors.push(e.message(&getter.sig.ident)),
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

impl JniGen {
    /// The memoized lowered plan for one bound function — the "build the plan
    /// once and store it" stage [`JniFunctionPlan::build`] anticipated (issue
    /// #90). Keyed by the function's ident (bound functions live in one flat
    /// namespace, and the synthetic const-getter idents `const_get_*` are
    /// distinct), so validation and every emitter share ONE derivation
    /// instead of rebuilding it ~8× per generation. `Ok` is cached; an `Err`
    /// (an unresolved converter) is passed through — it only occurs at the
    /// validation phase, which reports it and fails `resolve` before any
    /// emitter runs. Same interior-mutable contract as
    /// [`JniGen::iface_spec`]; drift is guarded externally by the byte-identity
    /// regen check (a plan change alters generated code).
    pub(crate) fn fn_plan(
        &self,
        registry: &Registry<KotlinMeta>,
        f: &syn::ItemFn,
    ) -> Result<std::rc::Rc<JniFunctionPlan>, PlanError> {
        if let Some(hit) = self.fn_plans.borrow().get(&f.sig.ident).cloned() {
            return Ok(hit);
        }
        let plan = std::rc::Rc::new(JniFunctionPlan::build(self, registry, f)?);
        self.fn_plans
            .borrow_mut()
            .insert(f.sig.ident.clone(), plan.clone());
        Ok(plan)
    }
}

impl JniFunctionPlan {
    /// Lower `f`'s inputs. Deterministic over `(ext, registry, f)`. Emission
    /// and validation go through the memo [`JniGen::fn_plan`], so the plan is
    /// built ONCE per function and shared; this is the underlying derivation.
    pub fn build(
        ext: &JniGen,
        registry: &Registry<KotlinMeta>,
        f: &syn::ItemFn,
    ) -> Result<Self, PlanError> {
        let jni_method = ext.mangle_jni_method(&kt_snake_to_camel(&f.sig.ident.to_string()));
        let native_symbol = ext.native_method_symbol(&jni_method);
        let onerror_iface = onerror_iface_spec(ext, registry, &f.sig.ident);
        // Output first: the Rust emitter historically resolved the output
        // before the inputs, so an unresolved-output failure takes precedence
        // over an unresolved-input one.
        let output = build_output(ext, registry, f)?;
        let mut params = Vec::new();
        for input in &f.sig.inputs {
            let syn::FnArg::Typed(pt) = input else {
                continue;
            };
            let syn::Pat::Ident(pid) = &*pt.pat else {
                continue;
            };
            let ident = pid.ident.clone();
            let ty = (*pt.ty).clone();

            let form = if let Some(plan) = registry
                .expansion_plans
                .get(&(f.sig.ident.clone(), ident.clone()))
            {
                let mut leaves = Vec::new();
                for leaf in &plan.leaves {
                    leaves.push(classify_leaf(
                        ext, registry, &leaf.name, &leaf.ty, /*expanded=*/ true, &ident,
                    )?);
                }
                ParamForm::Expanded(leaves)
            } else {
                ParamForm::Single(Box::new(classify_leaf(
                    ext, registry, &ident, &ty, /*expanded=*/ false, &ident,
                )?))
            };
            params.push(PlanParam { ident, ty, form });
        }
        Ok(Self {
            jni_method,
            native_symbol,
            onerror_iface,
            params,
            output,
        })
    }

    /// The flattened effective-parameter view (expansion leaves inline) —
    /// the sequence the Kotlin wrapper and `external fun` declare, in order.
    pub fn leaves(&self) -> impl Iterator<Item = &PlanLeaf> {
        self.params.iter().flat_map(|p| match &p.form {
            ParamForm::Single(l) => std::slice::from_ref(&**l).iter(),
            ParamForm::Expanded(ls) => ls.iter(),
        })
    }
}

/// Classify one effective parameter. `expanded` selects the restricted probe
/// set of the constructor-expansion fold path (its Rust decode handles
/// scalar-pair / consume / pass-through leaves only — struct-flatten and
/// vec-build never applied there, so probing them here would desynchronize
/// the wire).
fn classify_leaf(
    ext: &JniGen,
    registry: &Registry<KotlinMeta>,
    ident: &syn::Ident,
    ty: &syn::Type,
    expanded: bool,
    source_param: &syn::Ident,
) -> Result<PlanLeaf, PlanError> {
    let optional = is_option_type(ty);
    let as_enum_value = ext.is_kotlin_enum(&enum_probe_type(ty));
    let kt_name = kt_param_name(&ident.to_string());

    // `impl Fn(args)` first: typed entirely from the interface spec — the
    // erased entry exists but its metadata carries no surface type.
    if let Some(args) = extract_fn_trait_args(ty) {
        let iface = ext.iface_spec(registry, &SpecKey::callback(&args));
        return Ok(PlanLeaf {
            ty: ty.clone(),
            kt_name,
            kt_public: None,
            kt_meta: registry
                .input_entry(ty)
                .and_then(|e| e.metadata.kotlin_name.clone()),
            optional,
            as_enum_value,
            kind: InputKind::Callback { iface },
        });
    }

    // Every non-callback leaf requires a resolved input entry — the same
    // hard boundary the Rust emitter has always enforced.
    let Some(entry) = registry.input_entry(ty) else {
        let key = TypeKey::from_type(ty);
        return Err(if expanded {
            PlanError::UnresolvedLeaf {
                ty: key,
                param: source_param.clone(),
            }
        } else {
            PlanError::Unresolved { ty: key }
        });
    };

    let kind = if let Some((elem, by_ref)) = (!expanded)
        .then(|| vec_build_elem(ext, registry, ty))
        .flatten()
    {
        InputKind::VecBuild { elem, by_ref }
    } else if let Some(sp) = build_option_scalar_input_plan(ext, registry, ident, ty) {
        InputKind::OptionScalar(sp)
    } else if let Some(plan) = (!expanded)
        .then(|| build_flat_input_plan(ext, registry, ident, ty))
        .flatten()
    {
        InputKind::FlattenStruct(plan)
    } else {
        match entry.metadata.projection.as_ref().map(|p| p.kind.clone()) {
            Some(ProjectionKind::Handle) => InputKind::Handle {
                direct: entry.metadata.is_direct_handle(),
            },
            Some(ProjectionKind::ValueBlob) => {
                let proj = entry.metadata.projection.as_ref().expect("checked above");
                if matches!(proj.strategy, FoldStrategy::Iterable(_)) {
                    panic!(
                        "render_wrapper_fn: value-blob `Vec<_>` params aren't \
                         supported yet (param `{kt_name}`); add array codegen to lift this guard."
                    );
                }
                let field =
                    value_projection_field_for_leaf(ext, &proj.leaf_key).unwrap_or_else(|| {
                        panic!(
                            "render_wrapper_fn: cannot determine inline-class field for value \
                     projection param `{kt_name}`"
                        )
                    });
                InputKind::ValueUnwrap { field }
            }
            None => InputKind::Plain,
        }
    };

    // Typed surface: handle/value projections show their Kotlin class (from
    // the projection's leaf key); everything else the entry's resolved name.
    let kt_meta = entry.metadata.kotlin_name.clone();
    let kt_public = match entry.metadata.projection.as_ref() {
        Some(p) => ext.kotlin_fqn(&p.leaf_key).map(kt::KtType::cls),
        None => kt_meta.clone(),
    };

    Ok(PlanLeaf {
        ty: ty.clone(),
        kt_name,
        kt_public,
        kt_meta,
        optional,
        as_enum_value,
        kind,
    })
}

/// Lower the output side. Mirrors the historical derivations exactly:
/// the Rust facts (`is_convert`, target type, wire) from the former
/// `lower_output`/`output_target_type` (emit/wrapper.rs), the Kotlin
/// declared-surface facts from `classify_return`'s inputs
/// (render_extern_decl's `ret_decl` reconstruction).
fn build_output(
    ext: &JniGen,
    registry: &Registry<KotlinMeta>,
    f: &syn::ItemFn,
) -> Result<FnOutputPlan, PlanError> {
    use crate::api::core::{
        types_util::result_ok_type,
        unfold::{Delivery, UnfoldShape},
    };
    let ident = &f.sig.ident;
    let unfold_plan = registry.unfold_plans.get(ident);

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
    let return_ty: syn::Type = match &f.sig.output {
        syn::ReturnType::Default => syn::parse_quote!(()),
        syn::ReturnType::Type(_, ty) => (**ty).clone(),
    };
    let error_plan = registry.error_plans.get(ident);
    let ok_ty = error_plan.and_then(|_| result_ok_type(&return_ty));
    let target_ty = match unfold_plan {
        Some(p) => p
            .convert_out_ty
            .clone()
            .expect("Return delivery carries convert_out_ty"),
        None => ok_ty.unwrap_or(return_ty),
    };
    let Some(entry) = registry.output_entry(&target_ty) else {
        return Err(PlanError::UnresolvedOutput {
            ty: TypeKey::from_type(&target_ty),
        });
    };
    let wire_ty = entry.destination.clone();

    // The Kotlin surface classifies the DECLARED return — `convert_out_ty`
    // for a convert, else the signature's own output. (Not `target_ty`: the
    // Kotlin error peel rides the entry's `value_rust_key`, so the full
    // `Result<T, E>` type is looked up as written.)
    let ret_decl: syn::ReturnType = if is_convert {
        syn::parse_quote!(-> #target_ty)
    } else {
        f.sig.output.clone()
    };
    let (surface, canonical) = ReturnSurface::classify(ext, registry, &ret_decl);
    let is_enum = ext.is_kotlin_enum(&canonical);
    let is_option_enum = crate::api::core::types_util::option_inner_type(&canonical)
        .map(|inner| ext.is_kotlin_enum(&inner))
        .unwrap_or(false);

    Ok(FnOutputPlan::Value(Box::new(ValueOutputPlan {
        is_convert,
        target_ty,
        wire_ty,
        surface,
        is_enum,
        is_option_enum,
    })))
}

impl ReturnSurface {
    /// Classify a declared return type. Returns the surface plus the
    /// canonical (`value_rust_key`-peeled) type the enum probes run over —
    /// the single peel that subsumed both `classify_return`'s inline peel
    /// and the former `canonical_return_ty`.
    pub fn classify(
        ext: &JniGen,
        registry: &Registry<KotlinMeta>,
        output: &syn::ReturnType,
    ) -> (Self, syn::Type) {
        let ty = match output {
            syn::ReturnType::Default => return (Self::Unit, syn::parse_quote!(())),
            syn::ReturnType::Type(_, t) => &**t,
        };
        let outer_meta = registry.output_entry(ty).map(|e| e.metadata.clone());
        // Unit returns (incl. `ZResult<()>`, whose inner identity rides
        // `value_rust_key`) declare no Kotlin return type. The peeled type
        // comes straight off the stored key — no reparse, no silent
        // fallback.
        let canonical: syn::Type = outer_meta
            .as_ref()
            .and_then(|m| m.value_rust_key.as_ref())
            .map(TypeKey::to_type)
            .unwrap_or_else(|| ty.clone());
        if crate::api::lang::jnigen::util::is_unit(&canonical) {
            return (Self::Unit, canonical);
        }
        // Projection return (opaque handle or value class): read the folded
        // `Projection` the type-unfolding mechanism propagated onto this
        // return type's converter metadata — one source of truth, no
        // shape-specific peeling.
        if let Some(h) = outer_meta.as_ref().and_then(|m| m.projection.clone()) {
            let leaf_fqn = ext.kotlin_fqn(&h.leaf_key);
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
