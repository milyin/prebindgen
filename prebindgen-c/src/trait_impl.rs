use prebindgen_registry::{recipe::Direction, Building, Conversions, Crossing, RegistryBuilder};

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
        registry: &impl Conversions,
    ) -> Option<crate::chain::InputTerminalPlan> {
        use crate::chain::InputTerminalOperation;

        let plan = |ident, wire, operation| crate::chain::InputTerminalPlan {
            ident,
            source: ty.clone(),
            source_module: self.source_module.clone(),
            wire,
            operation,
        };
        let key = ty.key();

        // Opaque handle, by-value consume: `*Box::from_raw(v)` — fallible
        // (null handle → message).
        if self.opaque.contains_key(&key) {
            let c_struct = self.c_type_ident(&key);
            return Some(plan(
                Self::in_name_of(&key),
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
                Self::in_name_of(&key),
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
                Self::in_name_of(&key),
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
                Self::in_name_of(&key),
                syn::parse_quote!(*const ::core::ffi::c_char),
                InputTerminalOperation::String,
            ));
        }
        if r_is_str(ty) {
            return Some(plan(
                Self::in_name_of(&key),
                syn::parse_quote!(*const ::core::ffi::c_char),
                InputTerminalOperation::StrMarker,
            ));
        }
        if r_is_bool(ty) {
            return Some(plan(
                Self::in_name_of(&key),
                bool_wire(),
                InputTerminalOperation::Bool,
            ));
        }
        if r_is_scalar(ty) {
            return Some(plan(
                Self::in_name_of(&key),
                scalar_ty(ty)?,
                InputTerminalOperation::Scalar,
            ));
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
        registry: &impl Conversions,
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

    /// The gravestone write-back statements for a by-value **consume** / `_take` of a
    /// value-opaque type, writing into the slot pointed to by `slot` (a `*mut #opaque`).
    /// `None` ⇒ no write-back needed (plain data — the moved-from bitwise copy drops
    /// harmlessly). Owned-ness is **inferred** for a `repr_c_struct` mirror (the
    /// generator knows the fields): nullable owned-pointer fields are nulled in place
    /// (cheap, no `Default`); a bare `Box<T>` field falls back to the full `gravestone()`
    /// write. A non-mirror (`opaque_data_struct`/`opaque_owned_struct`) uses its explicit
    /// declared `kind` (its fields are an opaque blob the generator can't introspect).
    fn value_opaque_writeback(
        &self,
        registry: &impl Conversions,
        key: &TypeKey,
        slot: &syn::Ident,
    ) -> Option<TokenStream> {
        let opaque = &self.value_opaque.get(key)?.opaque;
        let plan = self.value_opaque_writeback_plan(registry, key)?;
        match plan {
            crate::chain::ValueOpaqueWriteback::None => None,
            _ => Some(plan.render(slot, opaque)),
        }
    }

    /// The semantic write-back policy shared by late input rendering and the
    /// emitted public `_take` helper.
    fn value_opaque_writeback_plan(
        &self,
        registry: &impl Conversions,
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
    fn compile_sites<'v>(
        &'v self,
        compiler: &mut prebindgen_registry::recipe::Compiler<
            '_,
            crate::compile::CCompile<'v, Registry>,
        >,
        registry: &'v Registry,
    ) -> Result<CPlanParts, String> {
        use prebindgen_registry::{
            generation::{ArtifactId, ArtifactInput, ArtifactPlan},
            recipe::{Crossing, Direction, Role, Site},
        };

        struct PendingCallback {
            callback: crate::chain::CallbackArtifact,
            inputs: Vec<ArtifactInput>,
        }

        let mut adapter = crate::compile::CCompile {
            gen: self,
            registry,
        };
        let mut plans = Vec::new();
        let mut callbacks = std::collections::BTreeMap::<String, PendingCallback>::new();
        let declared = self.declared_functions();
        let mut names: Vec<&syn::Ident> = declared.iter().collect();
        names.sort_by_key(|i| i.to_string());
        for name in names {
            let Some(f) = registry.flat().function(name) else {
                continue;
            };
            for (index, param) in f.params.iter().enumerate() {
                let site = Site {
                    owner: name.clone(),
                    role: Role::Param { index },
                };
                let crossing = Crossing::new(param.ty.clone(), Direction::Construct);
                if let Some(plan) = compiler
                    .site(&mut adapter, site, crossing)
                    .map_err(|e| e.to_string())?
                {
                    plans.push(plan);
                }

                let Some(args) = param.ty.callback_args() else {
                    continue;
                };
                let key: CallbackKey = args.iter().map(|arg| arg.key()).collect();
                let cfg = self.callbacks.get(&key).ok_or_else(|| {
                    format!(
                        "Cbindgen: callback parameter {} of `{name}` has no callback declaration",
                        index
                    )
                })?;
                let mut arguments = Vec::new();
                for (arg_index, arg) in args.iter().enumerate() {
                    let site = Site {
                        owner: name.clone(),
                        role: Role::CallbackArg {
                            param: index,
                            arg: arg_index,
                        },
                    };
                    let crossing = Crossing::new(arg.clone(), Direction::Deconstruct);
                    let plan = compiler
                        .site(&mut adapter, site, crossing)
                        .map_err(|e| e.to_string())?
                        .ok_or_else(|| {
                            format!(
                                "Cbindgen: callback argument {arg_index} of parameter {index} in \
                                 `{name}` was omitted"
                            )
                        })?;
                    let zero_copy_element = self.callback_slice_elem_wire_type_of(arg);
                    let takeable = cfg.takeable.contains(&arg_index);
                    if zero_copy_element.is_none() && !plan.abi().payload().has_abi() {
                        return Err(format!(
                            "Cbindgen: callback argument `{arg}` has no C ABI — deliver its parts \
                             as separate callback arguments instead"
                        ));
                    }
                    if takeable
                        && !matches!(plan.abi().payload(), crate::compile::CValue::Direct { .. })
                    {
                        return Err(format!(
                            "Cbindgen: takeable callback argument `{arg}` must have one direct C wire"
                        ));
                    }
                    arguments.push(crate::chain::CallbackArgument {
                        site: plan.id().clone(),
                        value: plan.abi().payload().clone(),
                        zero_copy_element,
                        takeable,
                    });
                    plans.push(plan);
                }

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
                    format_ident!("__cbg_in_{callback_name}"),
                    param.ty.clone(),
                    self.source_module.clone(),
                    args.to_vec(),
                    arguments,
                );
                match callbacks.entry(callback_name.clone()) {
                    std::collections::btree_map::Entry::Vacant(entry) => {
                        entry.insert(PendingCallback { callback, inputs });
                    }
                    std::collections::btree_map::Entry::Occupied(mut entry) => {
                        if entry.get().callback.signature() != callback.signature() {
                            return Err(format!(
                                "Cbindgen: callback name `{callback_name}` resolves to incompatible C ABIs"
                            ));
                        }
                        entry.get_mut().inputs.extend(inputs);
                    }
                }
            }
            // A `Result`'s arms are their own sites; the whole `Result` is not
            // a value C ever holds, so it is not one.
            let returns: Vec<(Role, &prebindgen_registry::flat::TypeRef)> =
                match f.ret.fallible_parts() {
                    Some((ok, err)) => vec![(Role::Return, ok), (Role::Error, err)],
                    None => vec![(Role::Return, &f.ret)],
                };
            for (role, ty) in returns {
                if matches!(ty.kind(), prebindgen_registry::flat::TypeKind::Unit) {
                    continue;
                }
                let site = Site {
                    owner: name.clone(),
                    role,
                };
                let crossing = Crossing::new(ty.clone(), Direction::Deconstruct);
                if let Some(plan) = compiler
                    .site(&mut adapter, site, crossing)
                    .map_err(|e| e.to_string())?
                {
                    plans.push(plan);
                }
            }
        }
        let artifacts = callbacks
            .into_iter()
            .map(|(name, pending)| {
                let id = ArtifactId::new("c-callback", name).map_err(|e| e.to_string())?;
                Ok(ArtifactPlan::new(
                    id,
                    Vec::new(),
                    pending.inputs,
                    crate::chain::CArtifact::Callback(pending.callback),
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
        registry: &impl Conversions,
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
        let prefix = match direction {
            Direction::Construct => Self::in_name_of(&fty.key()),
            Direction::Deconstruct => Self::out_name_of(&fty.key()),
        };
        Some(crate::chain::PayloadPlan {
            ident: format_ident!("{prefix}_payload"),
            source: fty.clone(),
            source_inner,
            source_module: self.source_module.clone(),
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
            ident: format_ident!("{}_field", Self::in_name_of(&ty.key())),
            source: ty.clone(),
            source_module: self.source_module.clone(),
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
            ident: format_ident!("{}_field", Self::out_name_of(&ty.key())),
            source: ty.clone(),
            source_module: self.source_module.clone(),
            wire: bool_wire(),
            operation: crate::chain::OutputTerminalOperation::BoolField,
        })
    }
}

/// Per-section [`CbindgenBuilder::prerequisites`] emitters. Each returns the runtime-
/// support items for one concern; the trait method concatenates them in order,
/// so the emitted preamble is identical to the former single function.
impl CbindgenBuilder {
    /// C allocator extern + raw C-string allocator + the universal memory freer.
    /// Emitted when the layer hands `char*`/array memory to C. Panics if such
    /// memory is produced but no `.free_memory_function` is declared.
    fn prereq_alloc_free(&self, registry: &Registry, produces_array: bool) -> Vec<syn::Item> {
        let mut items: Vec<syn::Item> = Vec::new();
        if !(self.needs_free(registry) || produces_array) {
            return items;
        }
        let free_ident = match &self.free_fn {
            Some(name) => format_ident!("{}", name),
            None => panic!(
                "Cbindgen: the generated layer hands `char*` string memory to C \
                 (a `String` return or a `String` data-struct field) but no \
                 memory-freeing function is declared — add \
                 `.free_memory_function(\"z_free\")`"
            ),
        };
        // C allocator (linked from the C runtime; no crate dependency).
        items.push(syn::parse_quote!(
            extern "C" {
                fn malloc(size: usize) -> *mut ::core::ffi::c_void;
                fn free(ptr: *mut ::core::ffi::c_void);
            }
        ));
        // Raw, destructor-free C-string block. `CString::new` drops interior
        // NULs so the terminator marks the true end for C consumers.
        items.push(syn::parse_quote!(
            #[allow(non_snake_case, dead_code)]
            pub(crate) fn __cbg_alloc_cstr(s: ::std::string::String) -> *mut ::core::ffi::c_char {
                let c = ::std::ffi::CString::new(s).unwrap_or_default();
                let bytes = c.as_bytes_with_nul();
                unsafe {
                    let p = malloc(bytes.len()) as *mut u8;
                    if p.is_null() {
                        return ::core::ptr::null_mut();
                    }
                    ::core::ptr::copy_nonoverlapping(bytes.as_ptr(), p, bytes.len());
                    p as *mut ::core::ffi::c_char
                }
            }
        ));
        // Universal raw memory freer: type-agnostic C `free`, no length, no
        // destructor (NULL-safe via C `free`).
        items.push(syn::parse_quote!(
            #[no_mangle]
            #[allow(non_snake_case, unused_variables)]
            pub unsafe extern "C" fn #free_ident(p: *mut ::core::ffi::c_void) {
                free(p);
            }
        ));
        items
    }

    /// Array builder: copy a `Vec<W>` into a C-`malloc`'d block of `W` and
    /// return `(ptr, len)` (empty ⇒ `(NULL, 0)`). The block is freed C-side
    /// via the `z_free_array` macro (per-element drop + the universal freer).
    fn prereq_array_builder(&self, produces_array: bool) -> Vec<syn::Item> {
        let mut items: Vec<syn::Item> = Vec::new();
        if !produces_array {
            return items;
        }
        items.push(syn::parse_quote!(
            #[allow(non_snake_case, dead_code)]
            pub(crate) unsafe fn __cbg_alloc_array<W>(v: ::std::vec::Vec<W>) -> (*mut W, usize) {
                let n = v.len();
                if n == 0 {
                    return (::core::ptr::null_mut(), 0);
                }
                let p = malloc(n.wrapping_mul(::core::mem::size_of::<W>())) as *mut W;
                if p.is_null() {
                    return (::core::ptr::null_mut(), 0);
                }
                for (i, e) in v.into_iter().enumerate() {
                    ::core::ptr::write(p.add(i), e);
                }
                (p, n)
            }
        ));
        items
    }

    /// Opaque handles: bare-pointer C type (`z_*_t*` = `Box::into_raw`) + typed
    /// `_drop`. The C type is an opaque/incomplete struct.
    fn prereq_opaque_handles(&self, registry: &Registry) -> Vec<syn::Item> {
        let mut items: Vec<syn::Item> = Vec::new();
        for (key, _cfg) in sorted_by_key(&self.opaque) {
            // Keyed directly: this used to spell the key into tokens purely so
            // `reading_of` could re-key them, twice (#291).
            let Some(reading) = registry.reading(key) else {
                continue;
            };
            if self.in_frag(&reading).is_none() && self.out_frag(&reading).is_none() {
                continue;
            }
            let c_struct = self.c_type_ident(&reading.key());
            // Opaque/incomplete C type: the handle is `#c_struct *`, which IS the
            // `Box::into_raw` pointer to the source value.
            items.push(syn::parse_quote!(
                #[repr(C)]
                #[allow(non_camel_case_types)]
                pub struct #c_struct {
                    _private: [u8; 0],
                }
            ));
            let src = self.src_ty_of(&reading.key());
            let drop_ident = self.destructor_symbol(&reading.key());
            items.push(syn::parse_quote!(
                #[no_mangle]
                #[allow(non_snake_case, unused_variables)]
                pub unsafe extern "C" fn #drop_ident(this_: *mut #c_struct) {
                    if !this_.is_null() {
                        drop(::std::boxed::Box::from_raw(this_ as *mut #src));
                    }
                }
            ));
        }
        items
    }

    /// Data structs: `#[repr(C)]` mirror only. Heap (`String`) fields are
    /// `char*` raw blocks the C user releases individually via the
    /// `free_memory_function` — no per-struct destructor.
    fn prereq_data_structs(&self, registry: &Registry) -> Vec<syn::Item> {
        let mut items: Vec<syn::Item> = Vec::new();
        for (key, _cfg) in sorted_by_key(&self.data) {
            let Some(reading) = registry.reading(key) else {
                continue;
            };
            if self.in_frag(&reading).is_none() && self.out_frag(&reading).is_none() {
                continue;
            }
            let Some(fields) = self.struct_fields(registry, &reading.key()) else {
                continue;
            };
            let c_struct = self.c_type_ident(&reading.key());
            let mut field_defs: Vec<TokenStream> = Vec::new();
            for (fname, fty) in &fields {
                let wire = self.data_field_wire(fty).unwrap_or_else(|| {
                    panic!(
                        "Cbindgen: field `{}` of data struct `{}` has unsupported type `{}`",
                        fname,
                        type_short(&reading.key()),
                        fty
                    )
                });
                field_defs.push(quote!(pub #fname: #wire));
            }
            items.push(syn::parse_quote!(
                #[repr(C)]
                #[allow(non_camel_case_types)]
                pub struct #c_struct {
                    #(#field_defs,)*
                }
            ));
        }
        items
    }

    /// Value-opaque types: the opaque `#[repr(C, align(_))]` counterpart is
    /// defined elsewhere (e.g. a size/align probe generator). Here we emit only
    /// the fail-closed size+align equality asserts and the typed `_drop` (drops
    /// the live Rust value in place; NULL/gravestone ⇒ no-op), plus a `_take`
    /// for types delivered as takeable callback params.
    fn prereq_value_opaque(&self, registry: &Registry) -> Vec<syn::Item> {
        let mut items: Vec<syn::Item> = Vec::new();
        let takeable_keys = self.takeable_type_keys();
        let mut vo: Vec<(&TypeKey, &ValueOpaqueCfg)> = self.value_opaque.iter().collect();
        vo.sort_by(|a, b| a.0.as_str().cmp(b.0.as_str()));
        for (key, cfg) in vo {
            let Some(reading) = registry.reading(key) else {
                continue;
            };
            if self.in_frag(&reading).is_none() && self.out_frag(&reading).is_none() {
                continue;
            }
            let src = self.src_ty_of(&reading.key());
            let opaque = &cfg.opaque;
            // `repr_c_struct`: the opaque counterpart is an auto-generated
            // **visible-field** `#[repr(C)]` mirror (so C reads the fields directly),
            // not an externally-provided blob. Each field is lowered by
            // `mirror_field_wire` (scalar / enum / opaque pointer). The size/align
            // assert below then proves the whole-struct reinterpret sound.
            if cfg.generate_mirror {
                let mirror_ident = self.c_type_ident(&reading.key());
                let fields = self
                    .struct_fields(registry, &reading.key())
                    .unwrap_or_else(|| {
                        panic!(
                            "Cbindgen::repr_c_struct: `{}` is not a named struct",
                            type_short(&reading.key())
                        )
                    });
                // Restricted-validity audit (#170 instance 3, #158 instance 3):
                // a mirror is reinterpreted whole, so a field whose Rust type
                // rejects some bit patterns is UB the moment C writes one and
                // hands the struct back.
                //
                // Not narrowed to inbound mirrors, though only those are
                // reachable: converter reachability is not derived from use
                // today (a declared type resolves BOTH directions whether or
                // not either is called — the accounting #194/#196 replace), so
                // "does it cross in" has no truthful answer here. Over-
                // reporting is the safe direction, and the acknowledgement
                // below is the escape for a genuinely write-only mirror.
                let restricted = self.restricted_validity_fields(registry, &reading.key());
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
                        type_short(&reading.key()),
                        listed.join("\n"),
                    );
                }
                let field_defs: Vec<TokenStream> = fields
                    .iter()
                    .map(|(fname, fty)| {
                        let wire = self.mirror_field_wire(fty).unwrap_or_else(|| {
                            panic!(
                                "Cbindgen::repr_c_struct: field `{}` of `{}` has unsupported \
                                 type `{}` (expected a scalar, a declared `enum_type`, or an \
                                 opaque pointer `Option<Box<T>>`/`Box<T>` with `T` an `opaque_ptr`)",
                                fname,
                                type_short(&reading.key()),
                                fty
                            )
                        });
                        quote!(pub #fname: #wire)
                    })
                    .collect();
                items.push(syn::parse_quote!(
                    #[repr(C)]
                    #[allow(non_camel_case_types)]
                    pub struct #mirror_ident {
                        #(#field_defs,)*
                    }
                ));
                // A mirror that needs `gravestone()` (only the bare-`Box<T>` fallback —
                // nullable owned-pointer fields are nulled in place) gets an
                // auto-generated `Gravestone` from the source type's `Default`. Nullable
                // mirrors emit nothing here, so they impose no `Default` requirement.
                if self.mirror_needs_gravestone_impl(registry, &reading.key()) {
                    items.push(syn::parse_quote!(
                        impl ::prebindgen_c_runtime::Gravestone for #mirror_ident {
                            #[inline]
                            fn rust_gravestone() -> #src {
                                <#src as ::core::default::Default>::default()
                            }
                        }
                    ));
                }
            }
            // Fail-closed size/align equality guard (proves the transmute sound).
            items.push(syn::parse_quote!(
                const _: () = {
                    assert!(
                        ::core::mem::size_of::<#src>() == ::core::mem::size_of::<#opaque>(),
                        "value_opaque: Rust type and opaque counterpart differ in size"
                    );
                    assert!(
                        ::core::mem::align_of::<#src>() == ::core::mem::align_of::<#opaque>(),
                        "value_opaque: Rust type and opaque counterpart differ in alignment"
                    );
                };
            ));
            // Autogenerated transmute glue: the single place that owns the
            // unsafe rust<->opaque reinterpretation. `Gravestone` (user logic)
            // and the converters below are all expressed via these methods.
            items.push(syn::parse_quote!(
                impl ::prebindgen_c_runtime::Transmute for #opaque {
                    type Rust = #src;
                    #[inline]
                    fn from_rust(value: Self::Rust) -> Self {
                        let __v = ::core::mem::ManuallyDrop::new(value);
                        unsafe {
                            ::core::ptr::read(&*__v as *const Self::Rust as *const Self)
                        }
                    }
                    #[inline]
                    fn into_rust(self) -> Self::Rust {
                        let __v = ::core::mem::ManuallyDrop::new(self);
                        unsafe {
                            ::core::ptr::read(&*__v as *const Self as *const Self::Rust)
                        }
                    }
                    #[inline]
                    fn as_rust(&self) -> &Self::Rust {
                        unsafe { &*(self as *const Self as *const Self::Rust) }
                    }
                    #[inline]
                    fn as_rust_mut(&mut self) -> &mut Self::Rust {
                        unsafe { &mut *(self as *mut Self as *mut Self::Rust) }
                    }
                }
            ));
            let drop_ident = self.destructor_symbol(&reading.key());
            // Unconditional drop: safe because a moved-from slot holds a
            // gravestone (a valid, safely-droppable empty value), so dropping
            // it is a harmless no-op; a live slot drops normally.
            items.push(syn::parse_quote!(
                #[no_mangle]
                #[allow(non_snake_case, unused_variables)]
                pub unsafe extern "C" fn #drop_ident(this_: *mut #opaque) {
                    if !this_.is_null() {
                        ::core::ptr::drop_in_place(
                            <#opaque as ::prebindgen_c_runtime::Transmute>::as_rust_mut(&mut *this_),
                        );
                    }
                }
            ));
            // For a type delivered as a takeable callback param, also emit a
            // public `<base>_take(dst, src)`: move `src`'s value into `dst`. For
            // an `opaque_owned_struct` type, leave `src` a gravestone (so the
            // trampoline's post-call drop is a no-op); an `opaque_data_struct` type owns
            // nothing, so the leftover bitwise copy in `src` drops harmlessly and
            // no write-back is needed. This is the C user's "take" operation.
            if takeable_keys.contains(key) {
                let take_ident = self.take_symbol(&reading.key());
                // Same inferred write-back as a consume (field-null for a nullable
                // mirror, `gravestone()` for a bare-`Box` mirror / non-mirror owned).
                let writeback =
                    self.value_opaque_writeback(registry, &reading.key(), &format_ident!("src"));
                items.push(syn::parse_quote!(
                    #[no_mangle]
                    #[allow(non_snake_case, unused_variables)]
                    pub unsafe extern "C" fn #take_ident(
                        dst: *mut #opaque,
                        src: *mut #opaque,
                    ) {
                        if dst.is_null() || src.is_null() {
                            return;
                        }
                        ::core::ptr::write(dst, ::core::ptr::read(src));
                        #writeback
                    }
                ));
            }
        }
        items
    }

    /// Enums: `#[repr(C)]` mirror — variant idents with each discriminant
    /// **re-emitted verbatim**, exactly as the source wrote it.
    ///
    /// Deliberately NOT routed through the shared
    /// [`enum_discriminant_values`](prebindgen_registry::types_util::enum_discriminant_values).
    /// That helper resolves each variant to a concrete `i64`, which is what an
    /// adapter needs when it must *know the number* — JniGenBuilder's `jint` decode
    /// and the Kotlin `value(N)` constants. This mirror needs no number: it is
    /// Rust source that cbindgen re-reads, so passing the expression through
    /// keeps every discriminant C already accepted — a `const` or `cfg`-driven
    /// expression, and any value the source's own `repr` admits, including
    /// ones outside `i64`. Resolving here would narrow that domain to what
    /// `i64` and a literal can express, for no gain.
    ///
    /// The two adapters therefore agree on the *rule* (Rust's own assignment
    /// order, which the shared helper encodes) while differing on what they
    /// need from it — a number versus a spelling.
    fn prereq_enums(
        &self,
        registry: &Registry,
        emit: &prebindgen_registry::Emit,
    ) -> Vec<syn::Item> {
        let mut items: Vec<syn::Item> = Vec::new();
        for (key, _cfg) in sorted_by_key(&self.enums) {
            let Some(reading) = registry.reading(key) else {
                continue;
            };
            if self.in_frag(&reading).is_none() && self.out_frag(&reading).is_none() {
                continue;
            }
            let Some(e) = unit_enum(registry, &reading.key()) else {
                continue;
            };
            let cname = self.c_type_ident(&reading.key());
            // The C mirror re-states the discriminant **as written** — `= 0x07`
            // stays `0x07` — which is the one consumer `EnumValue`'s retained
            // syntax exists for, and the model's own docs name it.
            let variants = e.values.iter().map(|v| {
                let id = &v.name;
                match emit.discriminant(v) {
                    Some(expr) => quote!(#id = #expr),
                    None => quote!(#id),
                }
            });
            items.push(syn::parse_quote!(
                #[repr(C)]
                #[derive(Copy, Clone, Debug, Eq, PartialEq)]
                #[allow(non_camel_case_types)]
                pub enum #cname {
                    #(#variants),*
                }
            ));
        }
        items
    }

    /// Tagged unions: the `#[repr(C)]` mirror with payload variants, which
    /// cbindgen renders as a tag enum plus a `union` of the variant bodies —
    /// the idiomatic C tagged union, with no hand-written header fragment.
    /// Variant shape is mirrored faithfully (named stays named, tuple stays
    /// tuple, unit stays unit); each payload field takes the wire chosen by
    /// [`CbindgenBuilder::payload_field_wire`].
    ///
    /// A union whose payload wires own memory also gets a typed
    /// `<base>_drop(t_t *)` that frees the **active arm** and nulls the freed
    /// slots, so a second drop is a no-op. A union of plain data owns nothing
    /// and gets no drop.
    fn prereq_tagged_unions(
        &self,
        registry: &Registry,
        emit: &prebindgen_registry::Emit,
    ) -> Vec<syn::Item> {
        let mut items: Vec<syn::Item> = Vec::new();
        for (key, _cfg) in sorted_by_key(&self.tagged_unions) {
            let Some(reading) = registry.reading(key) else {
                continue;
            };
            if self.in_frag(&reading).is_none() && self.out_frag(&reading).is_none() {
                continue;
            }
            let Some(e) = payload_enum(registry, &reading.key()) else {
                continue;
            };
            let cname = self.c_type_ident(&reading.key());

            let mut variant_defs: Vec<TokenStream> = Vec::new();
            // Per-variant drop arm, collected only for variants that own
            // something; the rest fall to a single wildcard arm.
            let mut drop_arms: Vec<TokenStream> = Vec::new();
            for a in &e.alternatives {
                let vident = &a.name;
                let wires: Vec<syn::Type> = a
                    .fields
                    .iter()
                    .map(|f| self.payload_wire_of(&reading.key(), vident, f))
                    .collect();
                // `Alternative::spell` writes the delimiters the source wrote,
                // which is what the three-armed `syn::Fields` match was doing —
                // and `Field::bind` decides `name: wire` or `wire` per field.
                let defs: Vec<TokenStream> = a
                    .fields
                    .iter()
                    .zip(&wires)
                    .map(|(f, w)| f.bind(w))
                    .collect();
                variant_defs.push(emit.shape_alternative(a, quote!(#vident), &defs));

                // Drop arm: bind every field, free the owning ones.
                let owning: Vec<(usize, &Field, &syn::Type)> = a
                    .fields
                    .iter()
                    .zip(&wires)
                    .enumerate()
                    .filter(|(_, (f, w))| self.payload_wire_owns(&f.ty, w, registry))
                    .map(|(i, (f, w))| (i, f, w))
                    .collect();
                if owning.is_empty() {
                    continue;
                }
                let binds: Vec<syn::Ident> = (0..a.fields.len())
                    .map(|i| format_ident!("__f{}", i))
                    .collect();
                let parts: Vec<TokenStream> = a
                    .fields
                    .iter()
                    .zip(&binds)
                    .map(|(f, b)| f.bind(b))
                    .collect();
                let pattern = emit.shape_alternative(a, quote!(#cname::#vident), &parts);
                let frees = owning.iter().map(|(i, f, _)| {
                    let b = &binds[*i];
                    self.payload_free_stmt(&f.ty, b, registry)
                });
                drop_arms.push(quote!(#pattern => { #(#frees)* }));
            }

            items.push(syn::parse_quote!(
                #[repr(C)]
                #[allow(non_camel_case_types)]
                pub enum #cname {
                    #(#variant_defs),*
                }
            ));

            // The same predicate a CONTAINING struct uses to decide whether to
            // call this drop, so a nested union can never be freed through a
            // symbol that was not emitted.
            if self.tagged_union_has_drop(&reading, registry) {
                debug_assert!(!drop_arms.is_empty(), "has_drop implies an owning arm");
                let drop_ident = self.destructor_symbol(&reading.key());
                // The drop is a second C entry point into the same bytes, so it
                // owes the same tag check as the input converter — `&mut *this_`
                // on an out-of-range tag would be the very UB that check exists
                // to prevent. It emits that check from the same place, and,
                // having nowhere to report to, ignores the value (there is no
                // live arm to release), which keeps `_drop` the always-safe
                // no-op it is everywhere else.
                let tag_guard = self.tag_guard(
                    &cname,
                    e.alternatives.len(),
                    quote!((*this_)),
                    quote!(return;),
                );
                items.push(syn::parse_quote!(
                    #[no_mangle]
                    #[allow(non_snake_case, unused_variables)]
                    pub unsafe extern "C" fn #drop_ident(
                        this_: *mut ::core::mem::MaybeUninit<#cname>,
                    ) {
                        if this_.is_null() {
                            return;
                        }
                        #tag_guard
                        match (*this_).assume_init_mut() {
                            #(#drop_arms)*
                            _ => {}
                        }
                    }
                ));
            }
        }
        items
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

    /// Release one owning payload slot held behind `binding` (a `&mut` to the
    /// wire, from a `match &mut *this_` arm) and null it, so a second drop of
    /// the same union is a no-op. A `char *` block goes back to the C
    /// allocator; an opaque pointer is re-boxed and dropped, running the Rust
    /// destructor.
    fn payload_free_stmt(
        &self,
        fty: &TypeRef,
        binding: &syn::Ident,
        registry: &Registry,
    ) -> TokenStream {
        if r_is_string(fty) {
            return quote!(
                free(*#binding as *mut ::core::ffi::c_void);
                *#binding = ::core::ptr::null_mut();
            );
        }
        // A nested `data_struct` payload crosses BY VALUE, so the arm binds the
        // mirror itself and what has to be released is each of its OWNING
        // fields — reached through the binding and nulled in place, exactly as
        // a directly-owning payload is. This is the shape zenoh-flat#30 needs
        // (`ReplyResult`'s alternatives are structs whose fields are handles),
        // and without it those fields would leak silently.
        let owning = self.owning_data_struct_fields(fty, registry);
        if !owning.is_empty() {
            let frees = owning.iter().map(|(fname, fty)| {
                if r_is_string(fty) {
                    quote!(
                        free((*#binding).#fname as *mut ::core::ffi::c_void);
                        (*#binding).#fname = ::core::ptr::null_mut();
                    )
                } else if self.tagged_union_has_drop(fty, registry) {
                    // The field is ANOTHER union, crossing by value. Its own
                    // typed drop releases whichever arm is live and nulls the
                    // slot, so this stays idempotent like every other arm here
                    // — and the owning pointer is reached even though it is two
                    // levels down. Nothing else can reach it: a union arm is not
                    // a top-level struct field the C caller releases by hand.
                    let drop_ident = self.destructor_symbol(&fty.key());
                    quote!(#drop_ident(&mut (*#binding).#fname);)
                } else {
                    // `owning_data_struct_fields` yields exactly the two shapes
                    // above (`data_field_owns`), so this is unreachable — and a
                    // silent fall-through here would be a leak, which is the
                    // defect this whole path exists to prevent.
                    panic!(
                        "Cbindgen: data-struct field `{}` of type `{}` is owning but has no \
                         release form (expected a `String` or a declared `tagged_union`)",
                        fname, fty,
                    )
                }
            });
            return quote!(#(#frees)*);
        }
        let src_inner = self.src_ty_of(&r_boxed_inner(fty).unwrap_or(fty).key());
        quote!(
            if !(*#binding).is_null() {
                drop(::std::boxed::Box::from_raw(*#binding as *mut #src_inner));
                *#binding = ::core::ptr::null_mut();
            }
        )
    }

    /// The statements that make a C-supplied `MaybeUninit<mirror>` safe to
    /// `assume_init`: read the leading discriminant as a plain `c_int` and
    /// reject anything outside `0..variants`.
    ///
    /// `slot` is an expression for the `MaybeUninit` in scope and `on_bad` is
    /// what to do with an out-of-range tag — the **only** thing the two C entry
    /// points into these bytes differ in (the input converter returns `Err`,
    /// the typed drop returns `()` and so just bails). Passing that difference
    /// in, rather than letting the drop repeat the check inline, is what keeps
    /// the two from drifting apart.
    pub(crate) fn tag_guard(
        &self,
        cname: &syn::Ident,
        variants: usize,
        slot: TokenStream,
        on_bad: TokenStream,
    ) -> TokenStream {
        let n = variants as i64;
        let bounds_msg = format!(
            "`{cname}`: a #[repr(C)] enum with payload variants must be at least as large as \
             its C `int` discriminant"
        );
        quote!(
            const _: () = {
                assert!(
                    ::core::mem::size_of::<#cname>()
                        >= ::core::mem::size_of::<::core::ffi::c_int>(),
                    #bounds_msg
                );
            };
            let __tag: ::core::ffi::c_int =
                ::core::ptr::read(#slot.as_ptr() as *const ::core::ffi::c_int);
            if !((__tag as i64) >= 0 && (__tag as i64) < #n) {
                #on_bad
            }
        )
    }
}

impl CbindgenBuilder {
    /// State this binding into `registry` — see `JniGenBuilder::declare_into`.
    ///
    /// Push, not pull: the build script calls this, and the registry never
    /// calls back. cbindgen declares no consts (it has no const mechanism, so
    /// every captured const re-emits verbatim) and no decompositions.
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
    pub(crate) fn build_with(
        mut self,
        registry: prebindgen_registry::RegistryBuilder,
    ) -> Result<Cbindgen, prebindgen_registry::WriteRustError> {
        let declared = self.declare_into(registry)?.validate_with(&self)?;
        // A second holding of the model: `convert_with` consumes the builder,
        // and the table outlives that call.
        let model = declared.flat().clone();
        let recipes = self.recipes(&model).map_err(|errors| {
            prebindgen_registry::ScanError::AdapterInvariant {
                message: errors
                    .iter()
                    .map(|e| e.to_string())
                    .collect::<Vec<_>>()
                    .join("; "),
            }
        })?;
        let bindings = self.bindings(&model, &recipes).map_err(|errors| {
            prebindgen_registry::ScanError::AdapterInvariant {
                message: errors
                    .iter()
                    .map(|e| e.to_string())
                    .collect::<Vec<_>>()
                    .join("; "),
            }
        })?;
        // The compiler resumes from the fragments produced while registry
        // conversion resolution ran. It is cloned here as a map of `Rc`s`; the
        // immutable generation plan below becomes the only rendering input.
        let registry = declared
            .convert_with(|crossing, built, _emit| {
                let mut compiler = prebindgen_registry::recipe::Compiler::resume(
                    &model,
                    &recipes,
                    &bindings,
                    self.compiled.borrow().clone(),
                );
                let conv = self.compile_crossing(&mut compiler, crossing, built);
                *self.compiled.borrow_mut() = compiler.finish();
                // The conversion stays here; what the registry gets back is
                // which other crossings this one delegates to, which is what
                // its reachability walk needs.
                conv.map(|c| prebindgen_registry::Answer::over(c.subs))
            })?
            .build()?;
        // Freeze every ordinary and callback position plus the callback artifacts
        // that consume those sites.
        let CPlanParts { sites, artifacts } = {
            let mut compiler = prebindgen_registry::recipe::Compiler::resume(
                &model,
                &recipes,
                &bindings,
                self.compiled.borrow().clone(),
            );
            let parts = self
                .compile_sites(&mut compiler, &registry)
                .map_err(|message| prebindgen_registry::ScanError::AdapterInvariant { message })?;
            *self.compiled.borrow_mut() = compiler.finish();
            parts
        };
        let mut generation = prebindgen_registry::generation::GenerationPlanBuilder::new();
        for fragment in self.compiled.borrow().fragments() {
            generation.fragment(fragment.freeze());
        }
        for site in sites {
            generation.site(site);
        }
        for artifact in artifacts {
            generation.artifact(artifact);
        }
        self.generation = Some(generation.build().map_err(|errors| {
            prebindgen_registry::ScanError::AdapterInvariant {
                message: errors.to_string(),
            }
        })?);
        // What the compilation produced, kept for emission and for lookup.
        self.compiled_fns = self
            .compiled
            .borrow()
            .fragments()
            .into_iter()
            .map(|f| f.function.clone())
            .filter(|function| !function.is_deferred_invoke())
            .collect();
        self.validate_resolved(&registry)
            .map_err(|message| prebindgen_registry::ScanError::AdapterInvariant { message })?;
        Ok(Cbindgen {
            gen: self,
            registry,
        })
    }

    /// Build the conversion for one crossing by asking the table which recipe it
    /// takes and the driver to compile that recipe.
    ///
    /// `None` records a gap, exactly as the chain of guesses this replaced did:
    /// whether the gap matters is the registry's call, and its report names the
    /// crossing.
    fn compile_crossing<'v, R: Conversions>(
        &'v self,
        compiler: &mut prebindgen_registry::recipe::Compiler<'_, crate::compile::CCompile<'v, R>>,
        crossing: &Crossing,
        built: &'v R,
    ) -> Option<crate::compile::CFrag> {
        let (dir, key) = crossing;
        // The reading the scan already took for this crossing, fetched by the
        // key the crossing IS.
        let ty = built.reading(key)?;
        let direction = *dir;
        let mut adapter = crate::compile::CCompile {
            gen: self,
            registry: built,
        };
        let crossing = prebindgen_registry::recipe::Crossing::new(ty, direction);
        let fragment = compiler.crossing(&mut adapter, &crossing).ok()?;
        Some((*fragment).clone())
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
    /// `consts: None` — cbindgen has no const declaration mechanism, so every
    /// captured const is re-emitted verbatim and none is ever a skip.
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

    // Consts have no declaration mechanism here (`declared_consts` stays
    // `None`), so every indexed const re-emits through the default
    // `on_const` — a path-alias against this source module, keeping consts
    // with non-portable initializers valid in the generated file. (cbindgen
    // cannot evaluate a path initializer, so aliased consts don't surface
    // as `#define`s in the C header.)
    fn source_module(&self) -> Option<&syn::Path> {
        self.source_module.as_ref()
    }

    // ── Structural type resolution ──────────────────────────────────────
    // The adapter peels `ty` itself: a rank-0 terminal category, else a
    // wrapper shape (`Option<_>`, `&`/`&mut`/`&[_]`/`&str`). See `in_wrappers`
    // / `out_wrappers`.

    fn prerequisites(
        &self,
        registry: &Registry,
        emit: &prebindgen_registry::Emit,
    ) -> Vec<syn::Item> {
        // C-string data memory (string returns + `String` fields of data structs)
        // is malloc'd raw and freed by the single universal `free_memory_function`.
        // Array returns (`Vec<T>`) also hand out a malloc'd block freed via the
        // same function (per element through the `z_free_array` macro), so the
        // allocator/freer prelude is needed for them too. Each section's emitter
        // lives in the `impl CbindgenBuilder` block above; order is significant.
        let produces_array = self.produces_array(registry);
        let mut items: Vec<syn::Item> = Vec::new();
        items.extend(self.prereq_alloc_free(registry, produces_array));
        items.extend(self.prereq_array_builder(produces_array));
        items.extend(self.prereq_opaque_handles(registry));
        items.extend(self.prereq_data_structs(registry));
        items.extend(self.prereq_value_opaque(registry));
        items.extend(self.prereq_enums(registry, emit));
        items.extend(self.prereq_tagged_unions(registry, emit));
        items.extend(crate::chain::render_callback_artifacts(
            self.generation
                .as_ref()
                .expect("C generation plan was not frozen"),
            emit,
        ));
        items.extend(self.prereq_domain_constants(registry));
        items
    }

    // ── Item emission ──────────────────────────────────────────────────

    fn on_function(
        &self,
        f: &prebindgen_registry::flat::Function,
        _registry: &Registry,
        emit: &prebindgen_registry::Emit,
    ) -> TokenStream {
        self.emit_function_wrapper(f, emit)
    }

    fn on_struct(
        &self,
        _s: &prebindgen_registry::flat::Struct,
        _registry: &Registry,
        _emit: &prebindgen_registry::Emit,
    ) -> TokenStream {
        // The `#[repr(C)]` mirror + converters come from prerequisites /
        // on_output_type; the original (non-FFI-safe) struct is dropped.
        TokenStream::new()
    }

    fn on_variant(
        &self,
        _v: &prebindgen_registry::flat::Variant,
        _registry: &Registry,
        _emit: &prebindgen_registry::Emit,
    ) -> TokenStream {
        TokenStream::new()
    }

    fn on_enum(
        &self,
        _e: &prebindgen_registry::flat::Enum,
        _registry: &Registry,
        _emit: &prebindgen_registry::Emit,
    ) -> TokenStream {
        TokenStream::new()
    }
}

/// Output-direction terminal categories: the shapes that cross whole, reached
/// from the `atomic` hook.
impl CbindgenBuilder {
    pub(crate) fn out_terminal(
        &self,
        ty: &TypeRef,
        registry: &impl Conversions,
    ) -> Option<crate::chain::OutputTerminalPlan> {
        let plan = |ident, wire, operation| crate::chain::OutputTerminalPlan {
            ident,
            source: ty.clone(),
            source_module: self.source_module.clone(),
            wire,
            operation,
        };
        // Unit return: trivial converter so `()` (and `Result<(), _>`) resolves.
        // Never actually called — void-returning wrappers ignore it, and
        // `emit_fallible_wrapper` special-cases `Result<(), E>` to drop the
        // out-param entirely (it exists only to satisfy the resolver).
        if matches!(ty.kind(), TypeKind::Unit) {
            return Some(plan(
                format_ident!("__cbg_out_unit"),
                syn::parse_quote!(()),
                crate::chain::OutputTerminalOperation::Unit,
            ));
        }

        // `String` output: a `malloc`'d `char*` raw block freed via the
        // `free_memory_function`. A `String` explicitly declared `opaque_ptr`
        // (held by C as `string_t *`) opts out — the opaque-handle branch below
        // owns it then (mirroring the input side, where owned-handle selection wins).
        if r_is_string(ty) && !self.opaque.contains_key(&ty.key()) {
            let name = Self::out_name_of(&ty.key());
            return Some(plan(
                name,
                syn::parse_quote!(*mut ::core::ffi::c_char),
                crate::chain::OutputTerminalOperation::String,
            ));
        }

        // FFI-safe scalar (`bool`, integers, floats): identity pass-through.
        if r_is_scalar(ty) {
            let name = Self::out_name_of(&ty.key());
            let spelled = scalar_ty(ty)?;
            return Some(plan(
                name,
                spelled,
                crate::chain::OutputTerminalOperation::Scalar,
            ));
        }

        let key = ty.key();

        // Opaque handle output: `Box::into_raw` → the bare `*mut #c_struct` handle.
        if self.opaque.contains_key(&key) {
            let name = Self::out_name_of(&ty.key());
            let c_struct = self.c_type_ident(&ty.key());
            return Some(plan(
                name,
                syn::parse_quote!(*mut #c_struct),
                crate::chain::OutputTerminalOperation::OwnedHandle { c_struct },
            ));
        }

        // Opaque error output (e.g. `ZError`): not a by-value struct — marshal it
        // to a malloc'd `char*` message via the recorded accessor `fn(&E) ->
        // String`. The error out-param of a `Result<_, E>` wrapper is thus
        // `char **e`. Freed by the universal `free_memory_function`.
        if let Some(msg_fn) = self.opaque_errors.get(&key) {
            let name = Self::out_name_of(&ty.key());
            return Some(plan(
                name,
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
            let name = Self::out_name_of(&ty.key());
            return Some(plan(
                name,
                opaque,
                crate::chain::OutputTerminalOperation::ValueOpaque,
            ));
        }

        // Enum output: `match` the source enum to the C enum.
        if self.enums.contains_key(&key) {
            let e = unit_enum(registry, &ty.key())?;
            let name = Self::out_name_of(&ty.key());
            let cname = self.c_type_ident(&ty.key());
            return Some(plan(
                name,
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
            ident: format_ident!("__cbg_inmark_slice_{}", sanitize(&e.key())),
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

        let plan = |ident, source_inner, wire, operation, null_message| BorrowPlan {
            ident,
            source_inner,
            source_module: self.source_module.clone(),
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
                format_ident!("__cbg_out_ref_{}", sanitize(&key)),
                elem.as_ref().clone(),
                syn::parse_quote!(*const #wire_ty),
                BorrowOperation::SharedOutput,
                String::new(),
            ));
        }

        // `&str`: borrow a UTF-8 C string directly from the caller.
        if !*rf_mut && r_is_str(rf_inner) {
            return Some(plan(
                Self::in_name_of(&ty.key()),
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
                    Self::in_name_of(&ty.key()),
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
                Self::in_name_of(&ty.key()),
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
            Self::in_name_of(&ty.key()),
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
        let (ident, subs) = match operation {
            crate::chain::MarkerOperation::Optional => (
                format_ident!("__cbg_outmark_option_{}", sanitize(&subject.key())),
                vec![subject.clone()],
            ),
            crate::chain::MarkerOperation::Sequence => (
                format_ident!("__cbg_outmark_vec_{}", sanitize(&subject.key())),
                vec![subject.clone()],
            ),
            crate::chain::MarkerOperation::Result => {
                let (ok, err) = subject.fallible_parts()?;
                (
                    format_ident!("__cbg_result_{}", sanitize(&subject.key())),
                    vec![ok.clone(), err.clone()],
                )
            }
        };
        Some(crate::chain::MarkerPlan {
            ident,
            operation,
            subs,
        })
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
