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

/// The lowered input-side plan for one bound function: one [`PlanParam`] per
/// source `syn::Signature` parameter (non-`Typed`/non-`Ident` args — `self`,
/// patterns — are skipped, mirroring every prior walk).
pub(crate) struct JniFunctionPlan {
    pub params: Vec<PlanParam>,
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

/// Input-plan construction failure. The Rust emitter maps each variant to
/// its historical panic; the Kotlin renderers map to `None` (skip).
pub(crate) enum PlanError {
    /// `registry.input_entry` has no converter for a source param type.
    Unresolved { ty: TypeKey },
    /// No converter for a constructor-expansion leaf type.
    UnresolvedLeaf { ty: TypeKey, param: syn::Ident },
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
        Ok(Self { params })
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
