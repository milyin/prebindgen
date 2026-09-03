use prebindgen_registry::{
    generation::SitePlan,
    recipe::{Direction, Role, Site},
    Building, Conversions, RegistryBuilder,
};

use super::*;

struct CPlanParts {
    sites: Vec<prebindgen_registry::generation::SitePlan<crate::compile::CRepresentation>>,
    artifacts: Vec<prebindgen_registry::generation::ArtifactPlan<crate::compile::CRepresentation>>,
}

/// What an emitter asks instead of the converter table.
///
/// [`CbindgenBuilder::compiled`] holds every fragment this binding compiled,
/// keyed by crossing. A fragment answers for a crossing that occupies several
/// wire values; the converter table's single `destination` cannot.
impl CbindgenBuilder {
    /// The fragment for `ty` crossing in the given direction, if one compiled.
    ///
    /// Shares the fragment rather than copying it: the store is read while
    /// compilation is still writing to it, so a borrow cannot be held across
    /// the next write.
    pub(crate) fn frag(
        &self,
        ty: &TypeRef,
        direction: Direction,
    ) -> Option<std::rc::Rc<crate::compile::CFrag>> {
        self.compiled.borrow().fragment(&ty.key(), direction)
    }

    /// The fragment that builds a Rust `ty` out of C parts.
    pub(crate) fn in_frag(&self, ty: &TypeRef) -> Option<std::rc::Rc<crate::compile::CFrag>> {
        self.frag(ty, Direction::Construct)
    }

    /// The fragment that takes a Rust `ty` apart into C parts.
    pub(crate) fn out_frag(&self, ty: &TypeRef) -> Option<std::rc::Rc<crate::compile::CFrag>> {
        self.frag(ty, Direction::Deconstruct)
    }
}

/// Ordinary whole-value input-terminal selection. This records semantic and
/// wire-side facts only; the source [`TypeRef`] remains opaque until final
/// rendering.
impl CbindgenBuilder {
    /// Select one ordinary whole-value input operation without spelling the
    /// source type. The retained [`TypeRef`] is materialized only by the final
    /// Rust renderer.
    pub(crate) fn in_terminal(
        &self,
        ty: &TypeRef,
        registry: &(impl Conversions + ?Sized),
    ) -> Option<crate::chain::InputTerminalPlan> {
        use crate::chain::InputTerminalOperation;

        let plan = |wire, operation| crate::chain::InputTerminalPlan {
            source: ty.clone(),
            wire,
            operation,
        };
        let key = ty.key();

        // Opaque handle, by-value consume: `*Box::from_raw(v)` — fallible
        // (null handle → message).
        if self.opaque.contains_key(&key) {
            let c_struct = self.c_type_ident(&key);
            return Some(plan(
                syn::parse_quote!(*mut #c_struct),
                InputTerminalOperation::OwnedHandle {
                    null_message: format!("null {} handle passed by value", type_short(&key)),
                },
            ));
        }

        // Inline-opaque, by-`*mut` consume. Ownership-specific write-back is a
        // Flat/adapter semantic decision and is safe to freeze as wire-side
        // statements before the source type is spelled.
        if let Some(opaque) = self.value_opaque_ty_of(&key).cloned() {
            let writeback = self.value_opaque_writeback_plan(registry, &key)?;
            return Some(plan(
                syn::parse_quote!(*mut #opaque),
                InputTerminalOperation::ValueOpaque {
                    opaque,
                    writeback,
                    null_message: format!("null {} value passed by value", type_short(&key)),
                },
            ));
        }

        // A C enum arrives through `MaybeUninit` so its discriminant can be
        // validated before any restricted Rust enum value is materialized.
        if self.enums.contains_key(&key) {
            let e = unit_enum(registry, &key)?;
            let c_name = self.c_type_ident(&key);
            let c_name_string = c_name.to_string();
            return Some(plan(
                syn::parse_quote!(::core::mem::MaybeUninit<#c_name>),
                InputTerminalOperation::Enum {
                    c_name,
                    variants: e.values.iter().map(|value| value.name.clone()).collect(),
                    invalid_message: format!("invalid discriminant {{}} for `{c_name_string}`"),
                    size_message: format!(
                        "`{c_name_string}`: a #[repr(C)] enum must have the size of a C `int`"
                    ),
                    align_message: format!(
                        "`{c_name_string}`: a #[repr(C)] enum must have the alignment of a C `int`"
                    ),
                },
            ));
        }

        if r_is_string(ty) {
            return Some(plan(
                syn::parse_quote!(*const ::core::ffi::c_char),
                InputTerminalOperation::String,
            ));
        }
        if r_is_str(ty) {
            return Some(plan(
                syn::parse_quote!(*const ::core::ffi::c_char),
                InputTerminalOperation::StrMarker,
            ));
        }
        if r_is_bool(ty) {
            return Some(plan(bool_wire(), InputTerminalOperation::Bool));
        }
        if r_is_scalar(ty) {
            return Some(plan(scalar_ty(ty)?, InputTerminalOperation::Scalar));
        }
        None
    }

    /// The mirror field idents to null in a by-value consume's gravestone write-back,
    /// for a `repr_c_struct` (`generate_mirror`) whose owned-pointer fields are all
    /// **nullable** (`Option<Box<T>>`). `Some(idents)` (possibly empty — a pure
    /// scalar/enum mirror needs no write-back) enables the cheap field-nulling path;
    /// `None` (not a generate_mirror, or a bare `Box<T>` field whose NULL would be an
    /// invalid `Box`) forces the full `gravestone()` write.
    fn nullable_owned_ptr_fields(
        &self,
        registry: &(impl Conversions + ?Sized),
        key: &TypeKey,
    ) -> Option<Vec<syn::Ident>> {
        let cfg = self.value_opaque.get(key)?;
        if !cfg.generate_mirror {
            return None;
        }
        let mut idents = Vec::new();
        for (fname, fty) in self.struct_fields(registry, key)? {
            // An owned-pointer field is one whose mirror wire is a raw pointer
            // (`Option<Box<T>>` / `Box<T>` → `*mut t_t`); scalars/enums are not.
            if matches!(self.mirror_field_wire(fty), Some(syn::Type::Ptr(_))) {
                // Bare `Box<T>`: cannot be nulled (an invalid `Box`).
                fty.optional_inner()?;
                idents.push(fname);
            }
        }
        Some(idents)
    }

    /// The semantic write-back policy shared by late input rendering and the
    /// emitted public `_take` helper.
    fn value_opaque_writeback_plan(
        &self,
        registry: &(impl Conversions + ?Sized),
        key: &TypeKey,
    ) -> Option<crate::chain::ValueOpaqueWriteback> {
        use crate::chain::ValueOpaqueWriteback;

        let cfg = self.value_opaque.get(key)?;
        if cfg.generate_mirror {
            match self.nullable_owned_ptr_fields(registry, key) {
                // No owned-pointer fields ⇒ plain data, nothing to clean up.
                Some(fields) if fields.is_empty() => Some(ValueOpaqueWriteback::None),
                // All owned-pointer fields nullable ⇒ null them in place (drop-safe).
                Some(fields) => Some(ValueOpaqueWriteback::NullFields(fields)),
                // Bare `Box<T>` field ⇒ a NULL would be an invalid `Box`; full gravestone.
                None => Some(ValueOpaqueWriteback::Gravestone),
            }
        } else {
            // Non-mirror opaque: the consumer chose the kind explicitly.
            match cfg.kind {
                OpaqueKind::Owned => Some(ValueOpaqueWriteback::Gravestone),
                OpaqueKind::Data => Some(ValueOpaqueWriteback::None),
            }
        }
    }

    /// Whether the auto-generated `Gravestone` impl is needed for a `repr_c_struct`
    /// mirror: only when its consume/`_take` write-back uses `gravestone()` — i.e. it
    /// has a bare `Box<T>` owned-pointer field (a null `Box` is invalid). Nullable
    /// (`Option<Box<T>>`) mirrors null in place and need no `Gravestone`/`Default`;
    /// non-mirror owned types get their `Gravestone` impl from the consumer.
    fn mirror_needs_gravestone_impl(&self, registry: &Registry, key: &TypeKey) -> bool {
        match self.value_opaque.get(key) {
            Some(cfg) if cfg.generate_mirror => {
                self.nullable_owned_ptr_fields(registry, key).is_none()
            }
            _ => false,
        }
    }

    /// Compile every site of every exported function.
    ///
    /// The second half of #450 reaching real positions. A **recipe** is what a
    /// type's conversion is, and both adapters have compiled those since
    /// #455/#457; a **site** is one position in the generated API, and this is
    /// where they start being asked about.
    ///
    /// What it buys is that the contracts a site owns are asked at all. Which
    /// answers count as failures is the target's, through
    /// [`Compile::tolerates`] — and Cbindgen answers `Borrowed` everywhere,
    /// because a `*const T` is C's own non-owning pointer and its zero-copy
    /// accessors return one. So the validity check passes here by policy
    /// rather than by luck, and the composition contracts — a part's type and
    /// how it is held — are what actually run.
    ///
    /// The returned plans become the immutable site store consumed by ordinary
    /// wrappers and callback artifacts alike. Every callback argument is a root
    /// deconstruction site, so its ABI and encoder are the same frozen payload
    /// an ordinary return consumes.
    /// The declared callback a function's parameter is, if it is one: its key
    /// and the argument types it delivers.
    fn callback_of(
        &self,
        registry: &Registry,
        owner: &syn::Ident,
        param: usize,
    ) -> Option<(CallbackKey, Vec<prebindgen_registry::flat::TypeRef>)> {
        let ty = &registry.flat().function(owner)?.params.get(param)?.ty;
        let args = ty.callback_args()?;
        let key: CallbackKey = args.iter().map(|arg| arg.key()).collect();
        self.callbacks
            .contains_key(&key)
            .then(|| (key, args.to_vec()))
    }

    /// The callback artifacts the compiled sites feed, and the sites
    /// themselves.
    ///
    /// The registry enumerates and compiles every site — which positions exist
    /// is the model's answer, and the same whatever the target. What is left
    /// here is C's: one callback artifact per distinct C signature, built from
    /// the argument sites that reach it, and the two ABI checks a C callback
    /// argument has to pass.
    fn callback_artifacts(
        &self,
        registry: &Registry,
        sites: Vec<(Site, SitePlan<crate::compile::CRepresentation>)>,
    ) -> Result<CPlanParts, String> {
        use prebindgen_registry::generation::{ArtifactId, ArtifactInput, ArtifactPlan};

        struct PendingCallback {
            callback: crate::chain::CallbackArtifact,
            inputs: Vec<ArtifactInput>,
        }

        let mut callbacks = std::collections::BTreeMap::<String, PendingCallback>::new();
        let mut arguments = std::collections::BTreeMap::<
            (syn::Ident, usize),
            Vec<crate::chain::CallbackArgument>,
        >::new();
        let mut plans = Vec::with_capacity(sites.len());
        for (site, plan) in sites {
            if let Role::CallbackArg { param, arg } = site.role {
                let (key, args) =
                    self.callback_of(registry, &site.owner, param)
                        .ok_or_else(|| {
                            format!(
                        "Cbindgen: callback parameter {param} of `{}` has no callback declaration",
                        site.owner
                    )
                        })?;
                let ty = &args[arg];
                let zero_copy_element = self.callback_slice_elem_wire_type_of(ty);
                let takeable = self.callbacks[&key].takeable.contains(&arg);
                if zero_copy_element.is_none() && !plan.abi().payload().has_abi() {
                    return Err(format!(
                        "Cbindgen: callback argument `{ty}` has no C ABI — deliver its parts as \
                         separate callback arguments instead"
                    ));
                }
                if takeable
                    && !matches!(plan.abi().payload(), crate::compile::CValue::Direct { .. })
                {
                    return Err(format!(
                        "Cbindgen: takeable callback argument `{ty}` must have one direct C wire"
                    ));
                }
                arguments
                    .entry((site.owner.clone(), param))
                    .or_default()
                    .push(crate::chain::CallbackArgument {
                        site: plan.id().clone(),
                        value: plan.abi().payload().clone(),
                        zero_copy_element,
                        takeable,
                    });
            }
            plans.push(plan);
        }
        // Every declared callback PARAMETER gets an artifact, not only the ones
        // that produced argument sites: `impl Fn()` delivers nothing and still
        // needs its closure struct.
        let mut owners: Vec<syn::Ident> = self.declared_functions().into_iter().collect();
        owners.sort_by_key(|name| name.to_string());
        for owner in owners {
            let Some(function) = registry.flat().function(&owner) else {
                continue;
            };
            for param in 0..function.params.len() {
                let Some((key, args)) = self.callback_of(registry, &owner, param) else {
                    continue;
                };
                let arguments = arguments
                    .remove(&(owner.clone(), param))
                    .unwrap_or_default();
                let ty = function.params[param].ty.clone();
                let inputs = arguments
                    .iter()
                    .map(|argument| ArtifactInput::Site {
                        site: argument.site.clone(),
                        slots: argument.value.slots(),
                    })
                    .collect();
                let callback_name = self.callback_c_name(&key);
                let callback = crate::chain::CallbackArtifact::new(
                    self.callback_c_ident(&key),
                    crate::compile::callback_operation(&ty),
                    ty,
                    args,
                    arguments,
                );
                match callbacks.entry(callback_name.clone()) {
                    std::collections::btree_map::Entry::Vacant(entry) => {
                        entry.insert(PendingCallback { callback, inputs });
                    }
                    std::collections::btree_map::Entry::Occupied(mut entry) => {
                        if entry.get().callback.signature() != callback.signature() {
                            return Err(format!(
                                "Cbindgen: callback name `{callback_name}` resolves to \
                                 incompatible C ABIs"
                            ));
                        }
                        entry.get_mut().inputs.extend(inputs);
                    }
                }
            }
        }

        let artifacts = callbacks
            .into_iter()
            .map(|(name, pending)| {
                let id = ArtifactId::new("c-callback", name).map_err(|e| e.to_string())?;
                Ok(ArtifactPlan::new(
                    id.clone(),
                    Vec::new(),
                    pending.inputs,
                    crate::assembly::CFinalArtifact::Callback(id, Box::new(pending.callback)),
                ))
            })
            .collect::<Result<Vec<_>, String>>()?;
        Ok(CPlanParts {
            sites: plans,
            artifacts,
        })
    }

    /// Which alternative of `key` these parts came from, by matching their
    /// field types against the model's.
    ///
    /// Only for a refusal message. `Compile::fields` is handed one arm's parts
    /// without being told which arm — the driver numbers parts per alternative
    /// and `choice` is where the `Alternative` arrives — so naming the arm the
    /// way the declaration writes it means finding it again. A refusal that
    /// says `Odd::Many` points at the line to change; one that says only the
    /// type does not.
    pub(crate) fn union_arm_name(
        &self,
        key: &TypeKey,
        registry: &(impl Conversions + ?Sized),
        parts: &[(
            prebindgen_registry::recipe::Part<'_>,
            &crate::compile::CFrag,
        )],
    ) -> Option<String> {
        let variant = match registry.flat().declared_type(&key.ident()?)? {
            prebindgen_registry::flat::Type::Variant(v) => v,
            _ => return None,
        };
        variant
            .alternatives
            .iter()
            .find(|a| {
                a.fields.len() == parts.len()
                    && a.fields
                        .iter()
                        .zip(parts)
                        .all(|(f, (p, _))| f.ty.key() == p.ty.key())
            })
            .map(|a| a.name.to_string())
    }

    /// Retain a tagged-union pointer payload without spelling its Rust type.
    ///
    /// The C wire owns one heap allocation. A source payload that already says
    /// `Box<T>` transfers that box directly; a bare declared handle `T` is
    /// boxed or unboxed at the boundary. Optionality is represented by NULL,
    /// while a NULL non-optional input is a binding error because it can be
    /// observed after a union arm has already been dropped.
    pub(crate) fn payload_plan(
        &self,
        fty: &TypeRef,
        direction: Direction,
    ) -> Option<crate::chain::PayloadPlan> {
        let optional = fty.optional_inner().is_some();
        let (wire, source_inner, boxed, short) =
            if let Some(inner) = self.declared_opaque_payload_inner(fty) {
                let c = self.c_type_ident(&inner);
                (
                    syn::parse_quote!(*mut #c),
                    fty.optional_inner().unwrap_or(fty).clone(),
                    false,
                    type_short(&inner),
                )
            } else {
                let inner = r_boxed_inner(fty)?;
                let c = self.c_type_ident(&inner.key());
                (
                    syn::parse_quote!(*mut #c),
                    inner.clone(),
                    true,
                    type_short(&inner.key()),
                )
            };
        Some(crate::chain::PayloadPlan {
            source: fty.clone(),
            source_inner,
            wire,
            direction,
            optional,
            boxed,
            null_message: format!(
                "null payload for `{short}` (a non-optional payload cannot be NULL — the \
                 union may already have been dropped)"
            ),
        })
    }

    /// `String` **input from a `data_struct`'s mirror**: a null `char *`
    /// decodes to an empty string rather than refusing.
    ///
    /// The second reading a `String` has, and the reason it needs a recipe of its
    /// own. A `String` **parameter** is a pointer the caller chose to pass, so
    /// a null one is a caller error and the ordinary input-terminal plan says so. A `String`
    /// **field** shares a struct with every other field, and refusing it would
    /// make the whole struct's decode fallible — so a function taking such a
    /// struct by value would need a `Result` return or `.panic()`, for a field
    /// it may not even read.
    ///
    /// Lossy on invalid UTF-8 for the same reason. This is the reading the
    /// hand-written field walk had; stating it as a recipe is what makes it
    /// visible rather than buried.
    pub(crate) fn in_string_field_plan(
        &self,
        ty: &TypeRef,
    ) -> Option<crate::chain::InputTerminalPlan> {
        if !r_is_string(ty) {
            return None;
        }
        Some(crate::chain::InputTerminalPlan {
            source: ty.clone(),
            wire: syn::parse_quote!(*const ::core::ffi::c_char),
            operation: crate::chain::InputTerminalOperation::StringField,
        })
    }

    /// `bool` **output into a `data_struct`'s mirror**: the mirror's field is
    /// [`bool_wire`], so the plain Rust `bool` is wrapped rather than passed
    /// through.
    ///
    /// The twin of [`Self::in_bool`], and the reason `bool` has a second recipe: a
    /// `bool` **return** is always already one of two values and crosses as
    /// itself, while a field shares one mirror with the decode that has to
    /// normalise it.
    pub(crate) fn out_bool_field_plan(
        &self,
        ty: &TypeRef,
    ) -> Option<crate::chain::OutputTerminalPlan> {
        if !r_is_bool(ty) {
            return None;
        }
        Some(crate::chain::OutputTerminalPlan {
            source: ty.clone(),
            wire: bool_wire(),
            operation: crate::chain::OutputTerminalOperation::BoolField,
        })
    }
}

/// Declaration queries the planned artifacts read. Each answers one concern
/// of the runtime support the file opens with; the artifacts that carry those
/// items are in [`crate::assembly`].
impl CbindgenBuilder {
    /// Converter fragments consumed by one type-level final artifact.
    fn artifact_fragment_inputs(
        &self,
        reading: &TypeRef,
    ) -> Vec<prebindgen_registry::generation::ArtifactInput> {
        [self.in_frag(reading), self.out_frag(reading)]
            .into_iter()
            .flatten()
            .map(|fragment| {
                prebindgen_registry::generation::ArtifactInput::Fragment(fragment.id.clone())
            })
            .collect()
    }

    /// Freeze source-dependent C declaration families as registry artifacts.
    ///
    /// Planning retains source TypeRefs and target-owned wire syntax. Only the
    /// final artifact renderer may ask the writer to spell a source type.
    fn type_artifact_plans(
        &self,
        registry: &Registry,
    ) -> Result<
        Vec<prebindgen_registry::generation::ArtifactPlan<crate::compile::CRepresentation>>,
        String,
    > {
        use prebindgen_registry::generation::{ArtifactId, ArtifactPlan};

        let mut artifacts = Vec::new();

        for (key, _cfg) in sorted_by_key(&self.opaque) {
            let Some(reading) = registry.reading(key) else {
                continue;
            };
            let dependencies = self.artifact_fragment_inputs(&reading);
            if dependencies.is_empty() {
                continue;
            }
            let id = ArtifactId::new("c-opaque-handle", key.as_str()).map_err(|e| e.to_string())?;
            artifacts.push(ArtifactPlan::new(
                id.clone(),
                Vec::new(),
                dependencies,
                crate::assembly::CFinalArtifact::OpaqueHandle(
                    id,
                    Box::new(crate::chain::OpaqueHandleArtifact {
                        source: reading,
                        c_struct: self.c_type_ident(key),
                        drop_ident: self.destructor_symbol(key),
                    }),
                ),
            ));
        }

        let takeable_keys = self.takeable_type_keys();
        let mut values: Vec<(&TypeKey, &ValueOpaqueCfg)> = self.value_opaque.iter().collect();
        values.sort_by(|a, b| a.0.as_str().cmp(b.0.as_str()));
        for (key, cfg) in values {
            let Some(reading) = registry.reading(key) else {
                continue;
            };
            let dependencies = self.artifact_fragment_inputs(&reading);
            if dependencies.is_empty() {
                continue;
            }
            let mirror = if cfg.generate_mirror {
                let fields = self.struct_fields(registry, key).unwrap_or_else(|| {
                    panic!(
                        "Cbindgen::repr_c_struct: `{}` is not a named struct",
                        type_short(key)
                    )
                });
                let restricted = self.restricted_validity_fields(registry, key);
                if !restricted.is_empty() && !cfg.assume_c_field_validity {
                    let listed: Vec<String> = restricted
                        .iter()
                        .map(|(fname, reason)| format!("  `{fname}`: {reason}"))
                        .collect();
                    panic!(
                        "Cbindgen::repr_c_struct: `{}` crosses C's memory by whole-struct \
                         reinterpret, but these fields have restricted-validity Rust types:\n\
                         {}\n\
                         A C caller can write a byte outside those domains, and the reinterpret \
                         materialises it with no hook to normalise or validate it first (#170, \
                         #158). Move the field to a `data_struct` (per-field wires), pass it as \
                         a separate parameter, or widen it to an integer. If this binding's C \
                         side is trusted to write only in-domain bytes — or never hands the \
                         mirror back at all — acknowledge it with `.assume_c_field_validity()`.",
                        type_short(key),
                        listed.join("\n"),
                    );
                }
                Some(crate::chain::ValueOpaqueMirror {
                    ident: self.c_type_ident(key),
                    fields: fields
                        .iter()
                        .map(|(name, ty)| {
                            let wire = self.mirror_field_wire(ty).unwrap_or_else(|| {
                                panic!(
                                    "Cbindgen::repr_c_struct: field `{}` of `{}` has unsupported \
                                     type `{}` (expected a scalar, a declared `enum_type`, or an \
                                     opaque pointer `Option<Box<T>>`/`Box<T>` with `T` an `opaque_ptr`)",
                                    name,
                                    type_short(key),
                                    ty
                                )
                            });
                            (name.clone(), wire)
                        })
                        .collect(),
                    gravestone: self.mirror_needs_gravestone_impl(registry, key),
                })
            } else {
                None
            };
            let take = takeable_keys
                .contains(key)
                .then(|| crate::chain::ValueOpaqueTake {
                    ident: self.take_symbol(key),
                    writeback: self
                        .value_opaque_writeback_plan(registry, key)
                        .expect("value-opaque declaration has a write-back policy"),
                });
            let id = ArtifactId::new("c-value-opaque", key.as_str()).map_err(|e| e.to_string())?;
            artifacts.push(ArtifactPlan::new(
                id.clone(),
                Vec::new(),
                dependencies,
                crate::assembly::CFinalArtifact::ValueOpaque(
                    id,
                    Box::new(crate::chain::ValueOpaqueArtifact {
                        source: reading,
                        opaque: cfg.opaque.clone(),
                        mirror,
                        drop_ident: self.destructor_symbol(key),
                        take,
                    }),
                ),
            ));
        }
        Ok(artifacts)
    }

    /// Freeze one owning payload's recursive release policy without spelling
    /// the Rust type used by a boxed-pointer cleanup.
    fn payload_cleanup_plan(
        &self,
        fty: &TypeRef,
        registry: &Registry,
    ) -> Result<crate::chain::PayloadCleanup, String> {
        use prebindgen_registry::generation::ArtifactId;

        if r_is_string(fty) {
            return Ok(crate::chain::PayloadCleanup::AllocatedString);
        }
        if self.tagged_union_has_drop(fty, registry) {
            return Ok(crate::chain::PayloadCleanup::NestedUnion {
                artifact: ArtifactId::new("c-tagged-union", fty.key().as_str())
                    .map_err(|e| e.to_string())?,
                drop_ident: self.destructor_symbol(&fty.key()),
            });
        }
        let owning = self.owning_data_struct_fields(fty, registry);
        if !owning.is_empty() {
            let mut fields = Vec::with_capacity(owning.len());
            for (name, ty) in owning {
                fields.push((name, self.payload_cleanup_plan(ty, registry)?));
            }
            return Ok(crate::chain::PayloadCleanup::Fields(fields));
        }
        Ok(crate::chain::PayloadCleanup::BoxedPointer {
            source: Box::new(r_boxed_inner(fty).unwrap_or(fty).clone()),
        })
    }

    /// Freeze every reached tagged-union declaration and typed destructor.
    fn tagged_union_artifact_plans(
        &self,
        registry: &Registry,
    ) -> Result<
        Vec<prebindgen_registry::generation::ArtifactPlan<crate::compile::CRepresentation>>,
        String,
    > {
        use prebindgen_registry::generation::{ArtifactId, ArtifactPlan};

        let mut artifacts = Vec::new();
        for (key, _cfg) in sorted_by_key(&self.tagged_unions) {
            let Some(reading) = registry.reading(key) else {
                continue;
            };
            let inputs = self.artifact_fragment_inputs(&reading);
            if inputs.is_empty() {
                continue;
            }
            let Some(sum) = payload_enum(registry, key) else {
                continue;
            };
            let mut prerequisites = Vec::new();
            let mut arms = Vec::with_capacity(sum.alternatives.len());
            for alternative in &sum.alternatives {
                let mut fields = Vec::with_capacity(alternative.fields.len());
                for field in &alternative.fields {
                    let wire = self.payload_wire_of(key, &alternative.name, field);
                    let cleanup = if self.payload_wire_owns(&field.ty, &wire, registry) {
                        let cleanup = self.payload_cleanup_plan(&field.ty, registry)?;
                        cleanup.prerequisites(&mut prerequisites);
                        Some(cleanup)
                    } else {
                        None
                    };
                    fields.push(crate::chain::TaggedUnionFieldArtifact { wire, cleanup });
                }
                arms.push(crate::chain::TaggedUnionArmArtifact {
                    alternative: alternative.clone(),
                    fields,
                });
            }
            prerequisites.sort();
            prerequisites.dedup();
            let drop_ident = arms
                .iter()
                .any(|arm| arm.fields.iter().any(|field| field.cleanup.is_some()))
                .then(|| self.destructor_symbol(key));
            let id = ArtifactId::new("c-tagged-union", key.as_str()).map_err(|e| e.to_string())?;
            artifacts.push(ArtifactPlan::new(
                id.clone(),
                prerequisites,
                inputs,
                crate::assembly::CFinalArtifact::TaggedUnion(
                    id,
                    Box::new(crate::chain::TaggedUnionArtifact {
                        c_name: self.c_type_ident(key),
                        arms,
                        drop_ident,
                    }),
                ),
            ));
        }
        Ok(artifacts)
    }

    /// The wire of one payload field, or a generation error naming the
    /// offending variant field and the supported set.
    fn payload_wire_of(&self, key: &TypeKey, variant: &syn::Ident, field: &Field) -> syn::Type {
        self.payload_field_wire(&field.ty).unwrap_or_else(|reason| {
            panic!(
                "Cbindgen::tagged_union: payload `{}::{}{}` of type `{}` cannot cross: {}",
                type_short(key),
                variant,
                match &field.name {
                    Some(n) => format!(".{n}"),
                    None => String::new(),
                },
                field.ty,
                reason,
            )
        })
    }
}

/// Where one artifact kind sits in the generated file.
///
/// The file reads the runtime helpers first, because anything may call them,
/// then types before the values that use them and mirrors before the converters
/// that fill them. Stated once here: the plan emits artifacts in the order they
/// were stated, so this table is the file's layout rather than a description of
/// it.
fn file_order(kind: &str) -> usize {
    match kind {
        // Both runtime helpers share this kind, and keep the order they were
        // stated in: the memory helpers, then the array builder that fills a
        // block through them.
        "c-runtime" => 0,
        "c-opaque-handle" => 1,
        "c-data-struct" => 2,
        "c-value-opaque" => 3,
        "c-enum" => 4,
        "c-tagged-union" => 5,
        "c-callback" => 6,
        "c-domain-constant" => 7,
        "c-converter" => 8,
        "c-wrapper" => 9,
        "c-const" => 10,
        other => unreachable!("unplaced C artifact kind `{other}`"),
    }
}

impl CbindgenBuilder {
    /// State this binding into `registry` — see `JniGenBuilder::declare_into`.
    ///
    /// Push, not pull: the build script calls this, and the registry never
    /// calls back. cbindgen declares no selective const surface — it plans a
    /// source-module alias for every captured const — and no decompositions.
    /// Binding-local fns declared by `convert!(..).local(..)`.
    fn collect_local_functions(&self) -> Vec<(syn::ItemFn, String)> {
        let mut result = Vec::new();
        let mut seen = HashMap::<syn::Ident, String>::new();
        for (ident, path, sig) in self.convert_decls.iter().flat_map(|decl| decl.locals()) {
            let origin = prebindgen_registry::decl::local_path_prefix(path);
            let mut sig = sig.clone();
            sig.ident = ident.clone();
            let signature = quote!(#origin #sig).to_string();
            match seen.get(ident) {
                Some(previous) if previous == &signature => continue,
                Some(_) => panic!(
                    "binding-local conversion fn `{ident}` is declared with two different signatures"
                ),
                None => {
                    seen.insert(ident.clone(), signature);
                }
            }
            let item: syn::ItemFn = syn::parse_quote!(#sig { unimplemented!() });
            result.push((item, origin));
        }
        result
    }

    /// State this binding into `registry`, then resolve it — see
    /// `JniGenBuilder::build`.
    /// Read the source, resolve every crossing, and hand back the binding —
    /// see `JniGenBuilder::build`.
    pub fn build(self) -> Result<Cbindgen, prebindgen_registry::WriteRustError> {
        let flat = self
            .sources
            .clone()
            .build()
            .map_err(prebindgen_registry::ScanError::from)?;
        let registry = prebindgen_registry::Registry::builder(flat)?;
        self.build_with(registry)
    }

    /// [`Self::build`] over a registry described elsewhere — the test seam.
    /// Every artifact of the generated file the plan does not already hold,
    /// stated as `ArtifactPlan`s so the plan orders them with the rest.
    ///
    /// The kinds here are the ones built from the declarations rather than from
    /// a compiled crossing: the `#[repr(C)]` mirrors, the reserved sum-type
    /// values, one wrapper per declared function, the constant aliases, and the
    /// runtime helpers. A private converter is here too, stated as **following**
    /// its own fragment — the file emits it because that fragment survived,
    /// which is a consequence of the fragment being reached rather than a reason
    /// to keep it.
    fn file_artifact_plans(
        &self,
        registry: &Registry,
        wrapper_sites: &[prebindgen_registry::generation::SitePlan<
            crate::compile::CRepresentation,
        >],
    ) -> Result<
        Vec<prebindgen_registry::generation::ArtifactPlan<crate::compile::CRepresentation>>,
        String,
    > {
        use prebindgen_registry::{
            generation::{ArtifactId, ArtifactPlan},
            write::{ArtifactKey, RustArtifact as _},
        };

        // Each artifact already names itself: `RustArtifact::key` is the
        // identity it answers to everywhere else, so the plan states it under
        // that and not under a second name built here.
        fn identity(artifact: &crate::assembly::CFinalArtifact) -> ArtifactId {
            match artifact.key() {
                ArtifactKey::Artifact(id) => id,
                ArtifactKey::Operation(operation) => {
                    ArtifactId::new("c-converter", operation.to_string())
                        .expect("an operation identity is a non-empty artifact name")
                }
            }
        }
        let plan_of = |artifact: crate::assembly::CFinalArtifact| {
            ArtifactPlan::new(identity(&artifact), Vec::new(), Vec::new(), artifact)
        };

        let mut out = Vec::new();
        out.extend(
            crate::assembly::CDataStruct::all(self, registry)
                .into_iter()
                .map(|m| plan_of(crate::assembly::CFinalArtifact::DataStruct(Box::new(m)))),
        );
        out.extend(
            crate::assembly::CEnum::all(self, registry)
                .into_iter()
                .map(|m| plan_of(crate::assembly::CFinalArtifact::Enum(Box::new(m)))),
        );
        out.extend(
            crate::assembly::CDomainConstant::all(self, registry)
                .into_iter()
                .map(|c| plan_of(crate::assembly::CFinalArtifact::DomainConstant(Box::new(c)))),
        );
        // A private converter follows its own fragment: the file emits it
        // because that fragment survived, which is a consequence of the fragment
        // being reached rather than a reason to keep it.
        //
        // Several fragments can render one operation — the same `bool` codec
        // reached through two crossings — so a converter is stated once and
        // follows every fragment that renders it. Emitting it twice is a
        // duplicate artifact; following only the first would prune it when that
        // one crossing is not reached.
        let mut converters: std::collections::BTreeMap<
            ArtifactId,
            (
                crate::assembly::CFinalArtifact,
                Vec<prebindgen_registry::FragmentId>,
            ),
        > = Default::default();
        let mut converter_order: Vec<ArtifactId> = Vec::new();
        for fragment in self.compiled.borrow().fragments() {
            let frozen = fragment.freeze();
            let Some(converter) = frozen.artifact() else {
                continue;
            };
            let artifact = crate::assembly::CFinalArtifact::Converter(Box::new(converter.clone()));
            let id = identity(&artifact);
            match converters.get_mut(&id) {
                Some((_, follows)) => follows.push(frozen.id().clone()),
                None => {
                    converter_order.push(id.clone());
                    converters.insert(id, (artifact, vec![frozen.id().clone()]));
                }
            }
        }
        // A converter that calls another is placed after it, stated rather than
        // inherited: the order used to be the plan's fragment order, which is a
        // dependency order for the fragments and only incidentally one for the
        // artifacts they render.
        let converter_ids: std::collections::HashSet<ArtifactId> =
            converters.keys().cloned().collect();
        for id in converter_order {
            let (artifact, follows) = converters.remove(&id).expect("stated just above");
            let mut prerequisites: Vec<ArtifactId> =
                prebindgen_registry::write::RustArtifact::calls(&artifact)
                    .into_iter()
                    .filter_map(|call| match call {
                        ArtifactKey::Operation(operation) => {
                            ArtifactId::new("c-converter", operation.to_string()).ok()
                        }
                        ArtifactKey::Artifact(_) => None,
                    })
                    .filter(|required| converter_ids.contains(required) && *required != id)
                    .collect();
            prerequisites.sort();
            prerequisites.dedup();
            out.push(ArtifactPlan::new(id, prerequisites, Vec::new(), artifact).follows(follows));
        }
        let declared = self.declared_functions();
        let mut wrapped: Vec<_> = registry
            .flat()
            .functions()
            .filter(|function| declared.contains(&function.name))
            .collect();
        wrapped.sort_by_key(|function| function.name.to_string());
        out.extend(wrapped.into_iter().map(|function| {
            plan_of(crate::assembly::CFinalArtifact::Wrapper(Box::new(
                crate::assembly::CWrapper::new(self, wrapper_sites, function),
            )))
        }));
        let mut sorted_constants: Vec<_> = registry.flat().constants().collect();
        sorted_constants.sort_by_key(|constant| constant.name.to_string());
        out.extend(sorted_constants.into_iter().map(|constant| {
            plan_of(crate::assembly::CFinalArtifact::Const(Box::new(
                crate::assembly::CConst::new(self, constant),
            )))
        }));
        // The runtime helpers, kept only while something requires them. They
        // exist because another artifact calls them, which every artifact above
        // states as a prerequisite, so the plan decides whether they are emitted
        // and nothing asks a second time.
        out.push(
            plan_of(crate::assembly::CFinalArtifact::Memory(Box::new(
                crate::assembly::CMemory::new(self),
            )))
            .only_if_required(),
        );
        out.push(plan_of(crate::assembly::CFinalArtifact::ArrayBuilder).only_if_required());

        Ok(out)
    }

    pub(crate) fn build_with(
        mut self,
        registry: prebindgen_registry::RegistryBuilder,
    ) -> Result<Cbindgen, prebindgen_registry::WriteRustError> {
        let mut declared = self.declare_into(registry)?.validate_with(&self)?;
        let invariant = |errors: Vec<prebindgen_registry::recipe::RecipeError>| {
            prebindgen_registry::ScanError::AdapterInvariant {
                message: errors
                    .iter()
                    .map(|e| e.to_string())
                    .collect::<Vec<_>>()
                    .join("; "),
            }
        };
        let recipes = self.recipes(declared.flat()).map_err(invariant)?;
        let bindings = self
            .bindings(declared.flat(), &recipes)
            .map_err(invariant)?;
        // The registry walks the crossings and drives the hooks; what comes
        // back is every fragment they produced. The site phase below resumes
        // from it, and the immutable generation plan is the only rendering
        // input.
        *self.compiled.borrow_mut() = declared.generate(
            &mut crate::compile::CCompile { gen: &self },
            &recipes,
            &bindings,
        )?;
        let registry = declared.build()?;
        // Freeze every ordinary and callback position plus the callback artifacts
        // that consume those sites.
        // The registry enumerates and compiles every site; what comes back to
        // this binding is one callback artifact per distinct C signature.
        let CPlanParts {
            sites,
            mut artifacts,
        } = {
            let carried = self.compiled.borrow().clone();
            let (sited, store) = registry.compile_sites(
                &mut crate::compile::CCompile { gen: &self },
                &recipes,
                &bindings,
                carried,
            );
            *self.compiled.borrow_mut() = store;
            if let Some((site, e)) = sited.refusals.into_iter().next() {
                return Err(prebindgen_registry::ScanError::AdapterInvariant {
                    message: format!("Cbindgen: {site} could not be planned: {e}"),
                }
                .into());
            }
            self.callback_artifacts(&registry, sited.plans)
                .map_err(|message| prebindgen_registry::ScanError::AdapterInvariant { message })?
        };
        artifacts.extend(
            self.type_artifact_plans(&registry)
                .map_err(|message| prebindgen_registry::ScanError::AdapterInvariant { message })?,
        );
        artifacts.extend(
            self.tagged_union_artifact_plans(&registry)
                .map_err(|message| prebindgen_registry::ScanError::AdapterInvariant { message })?,
        );
        let mut generation = prebindgen_registry::generation::GenerationPlanBuilder::new();
        for fragment in self.compiled.borrow().fragments() {
            generation.fragment(fragment.freeze());
        }
        // The sites a wrapper reads, kept before they are handed to the plan:
        // a wrapper is stated as an artifact of that plan, so it cannot be built
        // from it.
        let wrapper_sites = sites.clone();
        for site in sites {
            generation.site(site);
        }
        // The rest of the file's artifacts, stated to the plan in the order the
        // file emits them. `topo_artifacts` walks artifacts in the order they
        // were stated and emits each one's prerequisites first, so the plan's
        // order is this order — the placement is declared once, here, rather
        // than written out again beside the plan.
        //
        // A private converter is stated as following its own fragment, the way
        // `prebindgen-jni` states its operations: the file emits it because the
        // fragment that renders it survived, which is a consequence of that
        // fragment being reached rather than a reason to keep it.
        artifacts.extend(
            self.file_artifact_plans(&registry, &wrapper_sites)
                .map_err(|message| prebindgen_registry::ScanError::AdapterInvariant { message })?,
        );
        // One order, declared once. `topo_artifacts` walks artifacts in the
        // order they were stated and emits each one's prerequisites first, so
        // stating them in the order the file reads them is what puts them in
        // that order — rather than a second, hand-written placement beside the
        // plan.
        // What an artifact calls is what must precede it, and — for the runtime
        // helpers — what keeps it at all. Resolved through what each artifact
        // says it PROVIDES, because an identity is not always answered by the
        // artifact whose name it looks like: a callback renders the Invoke
        // helper of its own converter operation, so a wrapper calling that
        // operation depends on the callback rather than on a converter that does
        // not exist. Over every artifact, since the callbacks and the type
        // mirrors are stated by their own passes.
        let owner: std::collections::HashMap<
            prebindgen_registry::write::ArtifactKey,
            prebindgen_registry::generation::ArtifactId,
        > = artifacts
            .iter()
            .flat_map(|artifact| {
                let payload: &crate::assembly::CFinalArtifact = artifact.payload();
                let id = artifact.id().clone();
                prebindgen_registry::write::RustArtifact::provides(payload)
                    .into_iter()
                    .map(move |provided| (provided, id.clone()))
            })
            .collect();
        artifacts = artifacts
            .into_iter()
            .map(|artifact| {
                let payload: &crate::assembly::CFinalArtifact = artifact.payload();
                let mut prerequisites: Vec<_> =
                    prebindgen_registry::write::RustArtifact::calls(payload)
                        .into_iter()
                        .filter_map(|call| owner.get(&call).cloned())
                        .filter(|required| required != artifact.id())
                        .chain(artifact.prerequisites().iter().cloned())
                        .collect();
                prerequisites.sort();
                prerequisites.dedup();
                artifact.requires(prerequisites)
            })
            .collect();
        artifacts.sort_by_key(|artifact| file_order(artifact.id().kind()));
        for artifact in artifacts {
            generation.artifact(artifact);
        }
        let generation = std::rc::Rc::new(generation.build().map_err(|errors| {
            prebindgen_registry::ScanError::AdapterInvariant {
                message: errors.to_string(),
            }
        })?);
        // The file's artifacts: every private converter in the plan's own
        // dependency order, then one exported wrapper per declared function,
        // named in source order so the file's layout does not depend on how
        // the declarations were written.
        //
        // The four kinds the plan already holds are read back from it by kind:
        // the payload IS the final artifact now, so this is a filter rather
        // than a wrapper around a lookup.
        // The file IS the plan: every artifact it emits, in the plan's own
        // order, taken from the plan's payloads. Nothing is assembled beside the
        // plan and checked to agree with it.
        //
        // The runtime helpers are the exception, and are placed ahead of it. They
        // exist because another artifact CALLS them, which the plan expresses as
        // a prerequisite — and a prerequisite is a reason to place one artifact
        // before another, not a reason to keep something that would otherwise be
        // dropped. An artifact with no `follows` is kept unconditionally, so
        // stating the helpers would emit them into every binding whether or not
        // anything reaches them. Asking what the artifacts call is what makes the
        // answer exact: a `Vec` delivered to a callback needs the array builder
        // just as a `Vec` return does, which a gate over return types alone
        // missed (#437).
        // The memory helpers survive the plan's pruning only when a kept
        // artifact requires them, which is exactly when this binding hands C
        // memory it must free. Checked here, while the generator is still being
        // built: `write_rust` renders an assembly that is already valid, so a
        // missing declaration must not reach it.
        if self.free_fn.is_none()
            && generation
                .artifacts()
                .any(|artifact| artifact.id().kind() == "c-runtime")
        {
            return Err(prebindgen_registry::ScanError::AdapterInvariant {
                message: "Cbindgen: the generated layer hands C memory it must free — a \
                          `char*` block (a `String` return or a `String` data-struct field) \
                          or an array block (a `Vec` returned or delivered to a callback) — \
                          but no memory-freeing function is declared: add \
                          `.free_memory_function(\"z_free\")`"
                    .to_string(),
            }
            .into());
        }
        let mut assembly = prebindgen_registry::write::AssemblyBuilder::new();
        for artifact in generation.artifacts() {
            assembly.artifact(artifact.payload().clone());
        }
        self.assembly = Some(assembly.build(&registry, self.source_module.as_ref()));
        self.generation = Some(generation);
        self.validate_resolved(&registry)
            .map_err(|message| prebindgen_registry::ScanError::AdapterInvariant { message })?;
        Ok(Cbindgen {
            gen: self,
            registry,
        })
    }

    pub fn declare_into(
        &self,
        mut registry: RegistryBuilder,
    ) -> Result<RegistryBuilder, prebindgen_registry::ScanError> {
        for (item_fn, origin) in self.collect_local_functions() {
            registry = registry.local_function(item_fn, origin)?;
        }
        for ident in self.declared_functions() {
            registry = registry.export(&ident);
        }
        for ident in self.helper_functions() {
            registry = registry.reference(&ident);
        }
        for ty in self.declared_types().into_values() {
            registry = registry.export_type(ty);
        }
        Ok(registry)
    }
}

impl Prebindgen for CbindgenBuilder {
    /// Report what this binding left unclaimed. Here because it is the
    /// earliest generator-owned hook that sees the model, and it runs exactly
    /// where the registry used to print these itself. Moves into
    /// `CbindgenBuilder::generate` once that exists (prebindgen#251 phase E).
    ///
    /// `consts: None` — cbindgen has no selective const mechanism, so every
    /// captured const reaches its source-alias policy and none is ever a skip.
    fn validate(&self, binding: &Building<'_>) -> Result<(), String> {
        let mut functions = self.declared_functions();
        functions.extend(self.helper_functions());
        prebindgen_registry::warn_unclaimed(
            binding.flat(),
            &prebindgen_registry::Claimed {
                functions,
                // The report asks what was *claimed*, which is a set of
                // identities — the declarations' spellings are the scan's
                // business, not this one's.
                types: self.declared_types().into_keys().collect(),
                consts: None,
                ignored_functions: self.ignored_functions(),
                ignored_types: self.ignored_types(),
                ..Default::default()
            },
        );
        Ok(())
    }

    // ── Structural type resolution ──────────────────────────────────────
    // The adapter peels `ty` itself: a rank-0 terminal category, else a
    // wrapper shape (`Option<_>`, `&`/`&mut`/`&[_]`/`&str`). See `in_wrappers`
    // / `out_wrappers`.

    // ── Item emission ──────────────────────────────────────────────────
}

/// Output-direction terminal categories: the shapes that cross whole, reached
/// from the `atomic` hook.
impl CbindgenBuilder {
    pub(crate) fn out_terminal(
        &self,
        ty: &TypeRef,
        registry: &(impl Conversions + ?Sized),
    ) -> Option<crate::chain::OutputTerminalPlan> {
        let plan = |wire, operation| crate::chain::OutputTerminalPlan {
            source: ty.clone(),
            wire,
            operation,
        };
        // Unit return: trivial converter so `()` (and `Result<(), _>`) resolves.
        // Never actually called — void-returning wrappers ignore it, and
        // `emit_fallible_wrapper` special-cases `Result<(), E>` to drop the
        // out-param entirely (it exists only to satisfy the resolver).
        if matches!(ty.kind(), TypeKind::Unit) {
            return Some(plan(
                syn::parse_quote!(()),
                crate::chain::OutputTerminalOperation::Unit,
            ));
        }

        // `String` output: a `malloc`'d `char*` raw block freed via the
        // `free_memory_function`. A `String` explicitly declared `opaque_ptr`
        // (held by C as `string_t *`) opts out — the opaque-handle branch below
        // owns it then (mirroring the input side, where owned-handle selection wins).
        if r_is_string(ty) && !self.opaque.contains_key(&ty.key()) {
            return Some(plan(
                syn::parse_quote!(*mut ::core::ffi::c_char),
                crate::chain::OutputTerminalOperation::String,
            ));
        }

        // FFI-safe scalar (`bool`, integers, floats): identity pass-through.
        if r_is_scalar(ty) {
            let spelled = scalar_ty(ty)?;
            return Some(plan(spelled, crate::chain::OutputTerminalOperation::Scalar));
        }

        let key = ty.key();

        // Opaque handle output: `Box::into_raw` → the bare `*mut #c_struct` handle.
        if self.opaque.contains_key(&key) {
            let c_struct = self.c_type_ident(&ty.key());
            return Some(plan(
                syn::parse_quote!(*mut #c_struct),
                crate::chain::OutputTerminalOperation::OwnedHandle { c_struct },
            ));
        }

        // Opaque error output (e.g. `ZError`): not a by-value struct — marshal it
        // to a malloc'd `char*` message via the recorded accessor `fn(&E) ->
        // String`. The error out-param of a `Result<_, E>` wrapper is thus
        // `char **e`. Freed by the universal `free_memory_function`.
        if let Some(msg_fn) = self.opaque_errors.get(&key) {
            return Some(plan(
                syn::parse_quote!(*mut ::core::ffi::c_char),
                crate::chain::OutputTerminalOperation::OpaqueError {
                    message_path: self.src_fn(msg_fn),
                },
            ));
        }

        // Value-opaque output: move the Rust value's bytes into the opaque
        // counterpart, by value (no Box). Size/align equality is asserted at the
        // type's emission site (fail-closed).
        if let Some(opaque) = self.value_opaque_ty_of(&ty.key()) {
            let opaque = opaque.clone();
            return Some(plan(
                opaque,
                crate::chain::OutputTerminalOperation::ValueOpaque,
            ));
        }

        // Enum output: `match` the source enum to the C enum.
        if self.enums.contains_key(&key) {
            let e = unit_enum(registry, &ty.key())?;
            let cname = self.c_type_ident(&ty.key());
            return Some(plan(
                syn::parse_quote!(#cname),
                crate::chain::OutputTerminalOperation::Enum {
                    c_name: cname,
                    variants: e.values.iter().map(|value| value.name.clone()).collect(),
                },
            ));
        }

        None
    }
}

/// Structural wrapper-shape resolvers (the post-rank-machinery surface). Each
/// peels `ty`'s outermost layer and composes the inner's converter; `subs`
/// lists the immediate inner(s) it looked up.
impl CbindgenBuilder {
    /// `&[E]` slice **input** retained without a rendered converter body.
    /// The frozen site plan owns the two-parameter (`*const E_wire`, `usize`)
    /// ABI and zero-copy decode.
    pub(crate) fn in_slice_plan(&self, ty: &TypeRef) -> Option<crate::chain::SliceInputPlan> {
        let e = r_shared_slice_elem(ty)?;
        // #170, the slice instance. The two-param lowering builds the
        // `&[E]` zero-copy from C's own block, so there is nowhere to
        // normalise the bytes: `&[bool]` would materialise every
        // element's restricted domain at once. `MaybeUninit<bool>` is
        // not a fix here — the callee wants `&[bool]`, and rebuilding
        // the block would silently drop the zero-copy contract this
        // path exists for. Rejected until a raw-wire lowering exists.
        if r_is_bool(e) {
            panic!(
                "Cbindgen: `&[bool]` cannot cross IN from C. A `bool` slice is \
                 reinterpreted zero-copy from the caller's block, so a byte outside \
                 `{{0, 1}}` would become a Rust `bool` with no chance to normalise it \
                 (#170). Take the flags as an integer slice, or wrap them in a declared \
                 `opaque_ptr` handle."
            );
        }
        let scalar = scalar_ty(e);
        let wire: syn::Type = match &scalar {
            Some(e_ty) => syn::parse_quote!(*const #e_ty),
            None => {
                let counterpart = self.value_opaque_ty_of(&e.key())?.clone();
                syn::parse_quote!(*const #counterpart)
            }
        };
        Some(crate::chain::SliceInputPlan {
            element: e.clone(),
            wire,
            reinterpret: scalar.is_none(),
        })
    }

    /// Retain an input or output borrow without spelling its Rust referent.
    pub(crate) fn borrow_plan(
        &self,
        ty: &TypeRef,
        direction: Direction,
    ) -> Option<crate::chain::BorrowPlan> {
        use crate::chain::{BorrowOperation, BorrowPlan};

        // `mutable` off the `Ref` itself, NOT `is_exclusive_borrow`: that
        // reading deliberately answers `false` for `&mut MaybeUninit<_>` — an
        // out-param slot is not an exclusive borrow OF A VALUE — and these arms
        // ask the syntactic question, "did the source write `&mut`".
        let TypeKind::Ref {
            mutable: rf_mut,
            inner: rf_inner,
            ..
        } = ty.kind()
        else {
            return None;
        };
        // The borrow's target, as a reading — every use below is its identity
        // or its source path, both of which the model answers.
        let elem = rf_inner;

        let plan = |source_inner, wire, operation, null_message| BorrowPlan {
            source_inner,
            wire,
            operation,
            null_message,
        };

        if direction == Direction::Deconstruct {
            if *rf_mut {
                return None;
            }
            let key = elem.key();
            let wire_ty: syn::Type = if self.opaque.contains_key(&key) {
                let c_struct = self.c_type_ident(&key);
                syn::parse_quote!(#c_struct)
            } else {
                self.value_opaque_ty_of(&key)?.clone()
            };
            return Some(plan(
                elem.as_ref().clone(),
                syn::parse_quote!(*const #wire_ty),
                BorrowOperation::SharedOutput,
                String::new(),
            ));
        }

        // `&str`: borrow a UTF-8 C string directly from the caller.
        if !*rf_mut && r_is_str(rf_inner) {
            return Some(plan(
                elem.as_ref().clone(),
                syn::parse_quote!(*const ::core::ffi::c_char),
                BorrowOperation::StrInput,
                "null pointer passed for str argument".to_owned(),
            ));
        }
        // `&mut T` (mutable borrow). Three sub-cases, all wiring to a `*mut` of the
        // wire (the C memory IS the Rust value for a value-opaque mirror — asserted
        // layout-identical — so the cast is sound; `&mut` is a borrow, no gravestone).
        if *rf_mut {
            // `&mut MaybeUninit<X>` (X value-opaque): out-param into uninitialized
            // memory. Rust writes via the `MaybeUninit` (no drop of the garbage slot).
            // `TypeKind::Uninit` is the form `maybe_uninit_inner` matched by
            // reading a path's tail ident.
            if let prebindgen_registry::flat::TypeKind::Uninit(inner) = elem.kind() {
                let op = self.value_opaque_ty_of(&inner.key())?.clone();
                let short = type_short(&inner.key());
                return Some(plan(
                    inner.as_ref().clone(),
                    syn::parse_quote!(*mut #op),
                    BorrowOperation::MutableUninitInput,
                    format!("null {short} pointer"),
                ));
            }
            // `&mut` opaque handle, or `&mut` value-opaque: both reinterpret the C
            // pointer as a mutable Rust reference. The wire is the handle's C struct
            // or the value-opaque mirror.
            let wire_ty: syn::Type = if self.opaque.contains_key(&elem.key()) {
                let c_struct = self.c_type_ident(&elem.key());
                syn::parse_quote!(#c_struct)
            } else {
                self.value_opaque_ty_of(&elem.key())?.clone()
            };
            let short = type_short(&elem.key());
            return Some(plan(
                elem.as_ref().clone(),
                syn::parse_quote!(*mut #wire_ty),
                BorrowOperation::MutableInput,
                format!("null {short} pointer"),
            ));
        }
        // `&T` (shared borrow) of an opaque handle or value-opaque type.
        let key1 = elem.key();
        let wire_ty: syn::Type = if self.opaque.contains_key(&key1) {
            let c_struct = self.c_type_ident(&elem.key());
            syn::parse_quote!(#c_struct)
        } else {
            self.value_opaque_ty_of(&elem.key())?.clone()
        };
        let short = type_short(&elem.key());
        Some(plan(
            elem.as_ref().clone(),
            syn::parse_quote!(*const #wire_ty),
            BorrowOperation::SharedInput,
            format!("null {short} pointer"),
        ))
    }

    /// A typed Optional, Sequence, or Result output marker.
    ///
    /// Its frozen C value or function site owns the real multi-leaf ABI; the
    /// marker retains semantic identity and dependency edges for emission.
    pub(crate) fn out_marker_plan(
        &self,
        operation: crate::chain::MarkerOperation,
        subject: &TypeRef,
    ) -> Option<crate::chain::MarkerPlan> {
        let subs = match operation {
            crate::chain::MarkerOperation::ChoiceArm => return None,
            crate::chain::MarkerOperation::Optional | crate::chain::MarkerOperation::Sequence => {
                vec![subject.clone()]
            }
            crate::chain::MarkerOperation::Result => {
                let (ok, err) = subject.fallible_parts()?;
                vec![ok.clone(), err.clone()]
            }
        };
        Some(crate::chain::MarkerPlan { operation, subs })
    }
}

/// The declaration surface, stated once.
///
/// These were trait methods the registry called back into the adapter from
/// inside `resolve`. They are the adapter's own business now, gathered into the
/// one value the registry is constructed from.
impl CbindgenBuilder {
    pub(crate) fn declared_functions(&self) -> HashSet<syn::Ident> {
        self.functions.keys().cloned().collect()
    }
    pub(crate) fn ignored_functions(&self) -> HashSet<syn::Ident> {
        self.ignored_functions.clone()
    }
    pub(crate) fn helper_functions(&self) -> HashSet<syn::Ident> {
        self.convert_decls
            .iter()
            .flat_map(|decl| decl.input_spec().iter().chain(decl.output_spec().iter()))
            .filter_map(|spec| match spec {
                ConvertSpec::PrebindgenFn(ident) => Some(ident.clone()),
                ConvertSpec::Trait { .. } => None,
            })
            .filter(|ident| !self.functions.contains_key(ident))
            .collect()
    }
    /// Each with the spelling its declarator was written with — the scan needs
    /// real tokens to intern a type that is in no table yet (#291).
    pub(crate) fn declared_types(&self) -> HashMap<TypeKey, Origin<syn::Type>> {
        self.opaque
            .iter()
            .chain(self.data.iter())
            .map(|(k, c)| (k, &c.rust_type))
            .chain(self.value_opaque.iter().map(|(k, c)| (k, &c.cfg.rust_type)))
            .chain(
                self.enums
                    .iter()
                    .chain(self.tagged_unions.iter())
                    .map(|(k, c)| (k, &c.rust_type)),
            )
            .map(|(k, t)| (k.clone(), t.clone()))
            .collect()
    }
    pub(crate) fn ignored_types(&self) -> HashSet<TypeKey> {
        self.ignored_types.clone()
    }
}
