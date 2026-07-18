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
    pub ident: syn::Ident,
    pub ty: syn::Type,
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
    /// `impl Fn(args)` callback: erased `Any` on the wire; the typed
    /// interface spec is derived at render time ([`callback_iface_spec`]).
    Callback { args: Vec<syn::Type> },
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
    /// `is_iterable_fold(shape)` — a bare `Iterable` OR one wrapped in a
    /// single `Optional` layer (`Option<Vec<T>>`). Selects the fold surface
    /// (`acc` + `fold`) over a scalar builder on every tier.
    pub iterable_fold: bool,
    /// Outer `Optional` layer present — the delivered result is nullable.
    pub optional: bool,
    /// Synthesized fixed-singleton delivery: no caller lambda, not generic.
    pub fixed_builder: bool,
    /// `plan.element.is_some()` — whole-element (M4) vs decomposed (M5) fold.
    pub whole_element: bool,
    /// Kotlin type variable of the wrapper: `None` for a fixed builder,
    /// `"A"` for a **bare** `Iterable` fold (an `Optional`-wrapped iterable
    /// takes the scalar-builder surface, hence `"R"`), `"R"` otherwise.
    pub generic: Option<&'static str>,
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

/// Plan construction failure. The Rust emitter maps each variant to its
/// historical panic; the Kotlin renderers map to `None` (skip).
pub(crate) enum PlanError {
    /// `registry.input_entry` has no converter for a source param type.
    Unresolved { ty: TypeKey },
    /// No converter for a constructor-expansion leaf type.
    UnresolvedLeaf { ty: TypeKey, param: syn::Ident },
    /// `registry.output_entry` has no converter for the output target type.
    UnresolvedOutput { ty: TypeKey },
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

impl JniFunctionPlan {
    /// Lower `f`'s inputs. Deterministic over `(ext, registry, f)` — until
    /// plans are built once and stored (a later stage), each emission site
    /// rebuilds the identical plan, exactly like [`build_struct_plan`].
    pub fn build(
        ext: &JniGen,
        registry: &Registry<KotlinMeta>,
        f: &syn::ItemFn,
    ) -> Result<Self, PlanError> {
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
        Ok(Self { params, output })
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

    // `impl Fn(args)` first: typed entirely from the interface spec — the
    // erased entry exists but its metadata carries no surface type.
    if let Some(args) = extract_fn_trait_args(ty) {
        return Ok(PlanLeaf {
            ident: ident.clone(),
            ty: ty.clone(),
            kt_public: None,
            kt_meta: registry
                .input_entry(ty)
                .and_then(|e| e.metadata.kotlin_name.clone()),
            optional,
            as_enum_value,
            kind: InputKind::Callback { args },
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
                         supported yet (param `{}`); add array codegen to lift this guard.",
                        kt_param_name(&ident.to_string())
                    );
                }
                let field =
                    value_projection_field_for_leaf(ext, &proj.leaf_key).unwrap_or_else(|| {
                        panic!(
                            "render_wrapper_fn: cannot determine inline-class field for value \
                     projection param `{}`",
                            kt_param_name(&ident.to_string())
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
        ident: ident.clone(),
        ty: ty.clone(),
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
        // is not generic; a **bare** `Iterable` folds with `<A>`; everything
        // else — including an `Optional`-wrapped iterable — builds with `<R>`.
        let generic = if fixed_builder {
            None
        } else if iterable_fold && !optional {
            Some("A")
        } else {
            Some("R")
        };
        return Ok(FnOutputPlan::Unfold(UnfoldOutputPlan {
            iterable_fold,
            optional,
            fixed_builder,
            whole_element: plan.element.is_some(),
            generic,
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
        // `value_rust_key`) declare no Kotlin return type.
        let inner_canon = outer_meta
            .as_ref()
            .and_then(|m| m.value_rust_key.clone())
            .unwrap_or_else(|| ty.to_token_stream().to_string());
        let canonical: syn::Type = syn::parse_str(&inner_canon).unwrap_or_else(|_| ty.clone());
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
