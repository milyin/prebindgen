use prebindgen_registry::{recipe::Direction, Building, Conversions, Crossing, RegistryBuilder};

use super::{builder::callback_fn_type, *};

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
    /// the next write, and `shape_is_lowerable` walks a type's layers asking
    /// this at every one.
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

/// Per-category **input** terminal converter builders. Each returns
/// `Some(ConverterImpl)` only for the type category it claims (and `None`
/// otherwise); [`Prebindgen::on_input_type`] chains them in priority order
/// before the wrapper shapes. The categories are mutually exclusive, so the
/// chain's fall-through is equivalent to a sequential `if … return` block.
impl CbindgenBuilder {
    /// Opaque handle, by-value consume: `*Box::from_raw(v)` — fallible (null
    /// handle → message). The wire is the bare handle pointer `*mut #c_struct`.
    pub(crate) fn in_opaque_handle(&self, ty: &TypeRef) -> Option<ConverterImpl> {
        let key = ty.key();
        if !self.opaque.contains_key(&key) {
            return None;
        }
        let name = Self::in_name_of(&ty.key());
        let c_struct = self.c_type_ident(&ty.key());
        let src = self.src_ty_of(&ty.key());
        let short = type_short(&ty.key());
        let null_msg = format!("null {short} handle passed by value");
        let function: syn::ItemFn = syn::parse_quote!(
            #[allow(non_snake_case, unused_variables, dead_code)]
            pub(crate) unsafe fn #name(
                v: *mut #c_struct,
            ) -> ::core::result::Result<#src, ::std::string::String> {
                if v.is_null() {
                    return ::core::result::Result::Err(
                        ::std::string::String::from(#null_msg),
                    );
                }
                ::core::result::Result::Ok(*::std::boxed::Box::from_raw(v as *mut #src))
            }
        );
        Some(ConverterImpl {
            subs: vec![],
            destination: syn::parse_quote!(*mut #c_struct),
            function,
            pre_stages: vec![],
            niches: Niches::empty(),
            metadata: (),
        })
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
        let cfg = self.value_opaque.get(key)?;
        let opaque = &cfg.opaque;
        if cfg.generate_mirror {
            match self.nullable_owned_ptr_fields(registry, key) {
                // No owned-pointer fields ⇒ plain data, nothing to clean up.
                Some(fields) if fields.is_empty() => None,
                // All owned-pointer fields nullable ⇒ null them in place (drop-safe).
                Some(fields) => Some(quote!(#( (*#slot).#fields = ::core::ptr::null_mut(); )*)),
                // Bare `Box<T>` field ⇒ a NULL would be an invalid `Box`; full gravestone.
                None => Some(
                    quote!(::core::ptr::write(#slot, <#opaque as ::prebindgen_c_runtime::Gravestone>::gravestone());),
                ),
            }
        } else {
            // Non-mirror opaque: the consumer chose the kind explicitly.
            match cfg.kind {
                OpaqueKind::Owned => Some(
                    quote!(::core::ptr::write(#slot, <#opaque as ::prebindgen_c_runtime::Gravestone>::gravestone());),
                ),
                OpaqueKind::Data => None,
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

    /// Inline-opaque, by-`*mut` consume: read the live Rust value out by
    /// transmute (move). For an `opaque_owned_struct` type, write a gravestone back so a
    /// later `_drop` is a no-op (safe drop-after-move); an `opaque_data_struct` type
    /// owns no external resource, so the moved-from bitwise duplicate is
    /// harmlessly droppable and no write-back is needed. Only the C pointer is
    /// null-checked — NULL ⇒ Err, and the `Option<_>` wrapper maps a NULL pointer
    /// wire → None. (We do NOT reject gravestone values: for types whose
    /// gravestone coincides with a legitimate value — e.g. an *empty* `ZBytes` —
    /// that would wrongly reject valid inputs; the move + write-back is safe.)
    ///
    /// **Write-back optimization for a `repr_c_struct` mirror:** the generator knows
    /// the mirror's fields, so when all its owned-pointer fields are nullable
    /// (`Option<Box<T>>`) it nulls just those fields (`(*v).label = null`) instead of
    /// rebuilding+writing the whole `Default` gravestone — drop-safe (scalars are
    /// `Copy`; nulling the owned pointers prevents the double-free) and far cheaper.
    /// Non-mirror types (`opaque_owned_struct` blobs) and mirrors with a bare `Box<T>`
    /// field (NULL would be an invalid `Box`) keep the full `gravestone()` write.
    pub(crate) fn in_value_opaque(
        &self,
        ty: &TypeRef,
        registry: &impl Conversions,
    ) -> Option<ConverterImpl> {
        let opaque = self.value_opaque_ty_of(&ty.key())?.clone();
        let name = Self::in_name_of(&ty.key());
        let src = self.src_ty_of(&ty.key());
        let short = type_short(&ty.key());
        let null_msg = format!("null {short} value passed by value");
        // Owned-ness (whether to clean up the moved-from slot) is inferred from the
        // mirror's fields for a `repr_c_struct`, or the explicit kind for a non-mirror.
        let writeback = self.value_opaque_writeback(registry, &ty.key(), &format_ident!("v"));
        let function: syn::ItemFn = syn::parse_quote!(
            #[allow(non_snake_case, unused_variables, dead_code)]
            pub(crate) unsafe fn #name(
                v: *mut #opaque,
            ) -> ::core::result::Result<#src, ::std::string::String> {
                if v.is_null() {
                    return ::core::result::Result::Err(
                        ::std::string::String::from(#null_msg),
                    );
                }
                let __live = <#opaque as ::prebindgen_c_runtime::Transmute>::into_rust(
                    ::core::ptr::read(v),
                );
                #writeback
                ::core::result::Result::Ok(__live)
            }
        );
        Some(ConverterImpl {
            subs: vec![],
            destination: syn::parse_quote!(*mut #opaque),
            function,
            pre_stages: vec![],
            niches: Niches::empty(),
            metadata: (),
        })
    }

    /// Enum input: read the C-supplied discriminant as a plain integer,
    /// **validate** it, then build the source enum — fallible.
    ///
    /// A C `enum` is an `int` at the ABI, so nothing stops a caller passing a
    /// value no variant has. Taking the mirror `#[repr(C)]` enum by value would
    /// **materialise** that invalid discriminant at the boundary — undefined
    /// behaviour *before* any `match` in this converter could inspect it, which
    /// is why validating an already-materialised enum is not a fix (#158).
    ///
    /// So the wire is `::core::mem::MaybeUninit<mirror>`, which is
    /// `#[repr(transparent)]` over the mirror (identical ABI, identical C
    /// spelling — cbindgen renders `MaybeUninit<T>` as `T`) and, unlike the
    /// mirror itself, may legally hold **any** bit pattern. The discriminant is
    /// then read out as `c_int` — the representation a `#[repr(C)]` fieldless
    /// enum has by definition, asserted below — and compared against the
    /// mirror's own variants, so a `const`- or `cfg`-driven discriminant needs
    /// no generator-side evaluation. An unmatched value is a binding error
    /// through the wrapper's error channel; no Rust enum is ever constructed
    /// from it.
    pub(crate) fn in_enum(&self, ty: &TypeRef, r: &impl Conversions) -> Option<ConverterImpl> {
        let key = ty.key();
        if !self.enums.contains_key(&key) {
            return None;
        }
        let e = unit_enum(r, &ty.key())?;
        let name = Self::in_name_of(&ty.key());
        let cname = self.c_type_ident(&ty.key());
        let src = self.src_ty_of(&ty.key());
        let cname_str = cname.to_string();
        let arms = e.values.iter().map(|v| {
            let id = &v.name;
            quote!(
                if __raw == #cname::#id as ::core::ffi::c_int {
                    return ::core::result::Result::Ok(#src::#id);
                }
            )
        });
        let bad_msg = format!("invalid discriminant {{}} for `{cname_str}`");
        let size_msg = format!("`{cname_str}`: a #[repr(C)] enum must have the size of a C `int`");
        let align_msg =
            format!("`{cname_str}`: a #[repr(C)] enum must have the alignment of a C `int`");
        let function: syn::ItemFn = syn::parse_quote!(
            #[allow(non_snake_case, unused_variables, dead_code)]
            pub(crate) unsafe fn #name(
                v: ::core::mem::MaybeUninit<#cname>,
            ) -> ::core::result::Result<#src, ::std::string::String> {
                const _: () = {
                    assert!(
                        ::core::mem::size_of::<#cname>()
                            == ::core::mem::size_of::<::core::ffi::c_int>(),
                        #size_msg
                    );
                    assert!(
                        ::core::mem::align_of::<#cname>()
                            == ::core::mem::align_of::<::core::ffi::c_int>(),
                        #align_msg
                    );
                };
                let __raw: ::core::ffi::c_int =
                    ::core::ptr::read(v.as_ptr() as *const ::core::ffi::c_int);
                #(#arms)*
                ::core::result::Result::Err(::std::format!(#bad_msg, __raw))
            }
        );
        Some(ConverterImpl {
            subs: vec![],
            destination: syn::parse_quote!(::core::mem::MaybeUninit<#cname>),
            function,
            pre_stages: vec![],
            niches: Niches::empty(),
            metadata: (),
        })
    }

    /// `String` input: `*const c_char` → owned `String` — fallible.
    pub(crate) fn in_string(&self, ty: &TypeRef) -> Option<ConverterImpl> {
        if !r_is_string(ty) {
            return None;
        }
        let name = Self::in_name_of(&ty.key());
        let function: syn::ItemFn = syn::parse_quote!(
            #[allow(non_snake_case, unused_variables, dead_code)]
            pub(crate) unsafe fn #name(
                v: *const ::core::ffi::c_char,
            ) -> ::core::result::Result<::std::string::String, ::std::string::String> {
                if v.is_null() {
                    return ::core::result::Result::Err(
                        ::std::string::String::from("null pointer passed for String argument"),
                    );
                }
                match ::std::ffi::CStr::from_ptr(v).to_str() {
                    ::core::result::Result::Ok(s) => {
                        ::core::result::Result::Ok(s.to_owned())
                    }
                    ::core::result::Result::Err(_) => {
                        ::core::result::Result::Err(
                            ::std::string::String::from("invalid UTF-8 in String argument"),
                        )
                    }
                }
            }
        );
        Some(ConverterImpl {
            subs: vec![],
            destination: syn::parse_quote!(*const ::core::ffi::c_char),
            function,
            pre_stages: vec![],
            niches: Niches::empty(),
            metadata: (),
        })
    }

    /// Bare `str` never crosses the C ABI directly, but resolving `&str`
    /// inputs requires its inner node to have a filled rank-0 cell.
    pub(crate) fn in_str(&self, ty: &TypeRef) -> Option<ConverterImpl> {
        if !r_is_str(ty) {
            return None;
        }
        let name = Self::in_name_of(&ty.key());
        let function: syn::ItemFn = syn::parse_quote!(
            #[allow(non_snake_case, dead_code, unused_variables)]
            pub(crate) fn #name() {}
        );
        Some(ConverterImpl {
            subs: vec![],
            destination: syn::parse_quote!(*const ::core::ffi::c_char),
            function,
            pre_stages: vec![],
            niches: Niches::empty(),
            metadata: (),
        })
    }

    /// `bool` input: the one scalar that is **not** a pass-through (#170).
    ///
    /// A `bool` parameter is the broadest place C hands over a byte that no
    /// Rust `bool` may hold, so it crosses as [`bool_wire`] and is normalised
    /// by [`bool_in_expr`] before a `bool` exists. The C prototype is
    /// unchanged — cbindgen simplifies `MaybeUninit<T>` to `T`.
    pub(crate) fn in_bool(&self, ty: &TypeRef) -> Option<ConverterImpl> {
        if !r_is_bool(ty) {
            return None;
        }
        let name = Self::in_name_of(&ty.key());
        let wire = bool_wire();
        let read = bool_in_expr(quote!(v));
        let function: syn::ItemFn = syn::parse_quote!(
            #[allow(non_snake_case, unused_variables, dead_code)]
            pub(crate) unsafe fn #name(v: #wire) -> bool {
                #read
            }
        );
        Some(ConverterImpl {
            subs: vec![],
            destination: wire,
            function,
            pre_stages: vec![],
            niches: Niches::empty(),
            metadata: (),
        })
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
    /// wrapper emission. Callback delivery is frozen separately in the next stage.
    fn compile_sites<'v>(
        &'v self,
        compiler: &mut prebindgen_registry::recipe::Compiler<
            '_,
            crate::compile::CCompile<'v, Registry>,
        >,
        registry: &'v Registry,
    ) -> Result<
        Vec<prebindgen_registry::generation::SitePlan<crate::compile::CRepresentation>>,
        String,
    > {
        use prebindgen_registry::recipe::{Crossing, Direction, Role, Site};

        let mut adapter = crate::compile::CCompile {
            gen: self,
            registry,
        };
        let mut plans = Vec::new();
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
        Ok(plans)
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

    /// A `Box`-over-handle **payload of a tagged union**, C to Rust.
    ///
    /// The payload rides as a bare `*mut t_t` the C caller gave up ownership
    /// of, so the pointer is reclaimed. A NULL one is reachable rather than
    /// hypothetical — the typed drop nulls the arm it frees, so a union passed
    /// back after being dropped arrives here NULL — and is reported, never
    /// materialised. An `Option<Box<T>>` reads NULL as `None` instead, which is
    /// the representation it has room for.
    pub(crate) fn in_boxed_payload(&self, fty: &TypeRef) -> Option<ConverterImpl> {
        let name = format_ident!("{}_payload", Self::in_name_of(&fty.key()));
        let optional = fty.optional_inner().is_some();
        let (wire, src_inner, owned, short) =
            if let Some(inner) = self.declared_opaque_payload_inner(fty) {
                let c = self.c_type_ident(&inner);
                let src_inner = self.src_ty_of(&inner);
                let short = type_short(&inner);
                // The value is moved out of the box the C side owned.
                (
                    quote!(*mut #c),
                    src_inner.clone(),
                    quote!(*::std::boxed::Box::from_raw(v as *mut #src_inner)),
                    short,
                )
            } else {
                let inner = r_boxed_inner(fty)?;
                let c = self.c_type_ident(&inner.key());
                let src_inner = self.src_ty_of(&inner.key());
                let short = type_short(&inner.key());
                (
                    quote!(*mut #c),
                    src_inner.clone(),
                    quote!(::std::boxed::Box::from_raw(v as *mut #src_inner)),
                    short,
                )
            };
        let _ = src_inner;
        let produced = self.src_ty_of(&fty.key());
        let function: syn::ItemFn = if optional {
            syn::parse_quote!(
                #[allow(non_snake_case, unused_variables, dead_code)]
                pub(crate) unsafe fn #name(v: #wire) -> #produced {
                    if v.is_null() {
                        ::core::option::Option::None
                    } else {
                        ::core::option::Option::Some(#owned)
                    }
                }
            )
        } else {
            let null_msg = format!(
                "null payload for `{short}` (a non-optional payload cannot be NULL — the \
                 union may already have been dropped)"
            );
            syn::parse_quote!(
                #[allow(non_snake_case, unused_variables, dead_code)]
                pub(crate) unsafe fn #name(
                    v: #wire,
                ) -> ::core::result::Result<#produced, ::std::string::String> {
                    if v.is_null() {
                        return ::core::result::Result::Err(
                            ::std::string::String::from(#null_msg),
                        );
                    }
                    ::core::result::Result::Ok(#owned)
                }
            )
        };
        Some(ConverterImpl {
            subs: vec![],
            destination: syn::parse_quote!(#wire),
            function,
            pre_stages: vec![],
            niches: Niches::empty(),
            metadata: (),
        })
    }

    /// The peer of [`Self::in_boxed_payload`]: an owned value the C side must
    /// later release, boxed here rather than having arrived boxed.
    pub(crate) fn out_boxed_payload(&self, fty: &TypeRef) -> Option<ConverterImpl> {
        let name = format_ident!("{}_payload", Self::out_name_of(&fty.key()));
        let optional = fty.optional_inner().is_some();
        let src = self.src_ty_of(&fty.key());
        let (wire, some_expr, bare_expr) =
            if let Some(inner) = self.declared_opaque_payload_inner(fty) {
                let c = self.c_type_ident(&inner);
                (
                    quote!(*mut #c),
                    quote!(::std::boxed::Box::into_raw(::std::boxed::Box::new(__v)) as *mut #c),
                    quote!(::std::boxed::Box::into_raw(::std::boxed::Box::new(v)) as *mut #c),
                )
            } else {
                let inner = r_boxed_inner(fty)?;
                let c = self.c_type_ident(&inner.key());
                (
                    quote!(*mut #c),
                    quote!(::std::boxed::Box::into_raw(__v) as *mut #c),
                    quote!(::std::boxed::Box::into_raw(v) as *mut #c),
                )
            };
        let body: TokenStream = if optional {
            quote!(match v {
                ::core::option::Option::Some(__v) => #some_expr,
                ::core::option::Option::None => ::core::ptr::null_mut(),
            })
        } else {
            bare_expr
        };
        let function: syn::ItemFn = syn::parse_quote!(
            #[allow(non_snake_case, unused_variables, dead_code)]
            pub(crate) fn #name(v: #src) -> #wire {
                #body
            }
        );
        Some(ConverterImpl {
            subs: vec![],
            destination: syn::parse_quote!(#wire),
            function,
            pre_stages: vec![],
            niches: Niches::empty(),
            metadata: (),
        })
    }

    /// `String` **input from a `data_struct`'s mirror**: a null `char *`
    /// decodes to an empty string rather than refusing.
    ///
    /// The second reading a `String` has, and the reason it needs a recipe of its
    /// own. A `String` **parameter** is a pointer the caller chose to pass, so
    /// a null one is a caller error and [`Self::in_string`] says so. A `String`
    /// **field** shares a struct with every other field, and refusing it would
    /// make the whole struct's decode fallible — so a function taking such a
    /// struct by value would need a `Result` return or `.panic()`, for a field
    /// it may not even read.
    ///
    /// Lossy on invalid UTF-8 for the same reason. This is the reading the
    /// hand-written field walk had; stating it as a recipe is what makes it
    /// visible rather than buried.
    pub(crate) fn in_string_field(&self, ty: &TypeRef) -> Option<ConverterImpl> {
        if !r_is_string(ty) {
            return None;
        }
        let name = format_ident!("{}_field", Self::in_name_of(&ty.key()));
        let function: syn::ItemFn = syn::parse_quote!(
            #[allow(non_snake_case, unused_variables, dead_code)]
            pub(crate) unsafe fn #name(
                v: *const ::core::ffi::c_char,
            ) -> ::std::string::String {
                if v.is_null() {
                    ::std::string::String::new()
                } else {
                    ::std::ffi::CStr::from_ptr(v).to_string_lossy().into_owned()
                }
            }
        );
        Some(ConverterImpl {
            subs: vec![],
            destination: syn::parse_quote!(*const ::core::ffi::c_char),
            function,
            pre_stages: vec![],
            niches: Niches::empty(),
            metadata: (),
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
    pub(crate) fn out_bool_field(&self, ty: &TypeRef) -> Option<ConverterImpl> {
        if !r_is_bool(ty) {
            return None;
        }
        let name = format_ident!("{}_field", Self::out_name_of(&ty.key()));
        let wire = bool_wire();
        let wrap = bool_out_expr(quote!(v));
        let function: syn::ItemFn = syn::parse_quote!(
            #[allow(non_snake_case, unused_variables, dead_code)]
            pub(crate) fn #name(v: bool) -> #wire {
                #wrap
            }
        );
        Some(ConverterImpl {
            subs: vec![],
            destination: wire,
            function,
            pre_stages: vec![],
            niches: Niches::empty(),
            metadata: (),
        })
    }

    /// FFI-safe scalar (integers, floats): identity pass-through. `bool` is
    /// claimed earlier by [`Self::in_bool`] and never reaches here.
    pub(crate) fn in_scalar(&self, ty: &TypeRef) -> Option<ConverterImpl> {
        if !r_is_scalar(ty) || r_is_bool(ty) {
            return None;
        }
        let name = Self::in_name_of(&ty.key());
        // A scalar's spelling is its name, so this needs no captured syntax.
        let spelled = scalar_ty(ty)?;
        let function: syn::ItemFn = syn::parse_quote!(
            #[allow(non_snake_case, unused_variables, dead_code)]
            pub(crate) fn #name(v: #spelled) -> #spelled {
                v
            }
        );
        Some(ConverterImpl {
            subs: vec![],
            destination: spelled.clone(),
            function,
            pre_stages: vec![],
            niches: Niches::empty(),
            metadata: (),
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

    /// Callback closure structs: one `#[repr(C)]` `{ context, call, drop }`
    /// per declared signature actually used (its `impl Fn(...)` input
    /// resolved). `call` takes each arg's output wire (the owned handle the
    /// C callback must drop) plus the `void *context`; `drop` releases the
    /// context. Deterministic order by emitted name.
    fn prereq_callback_structs(&self, registry: &Registry) -> Vec<syn::Item> {
        let mut items: Vec<syn::Item> = Vec::new();
        // The declaration's own argument types. `CallbackKey` is a list of
        // identities — what the map is keyed by — and the arguments it was
        // declared with are beside it, so neither is rebuilt from the other
        // (#291).
        let mut cb_keys: Vec<(&CallbackKey, &CbCfg)> = self.callbacks.iter().collect();
        cb_keys.sort_by_key(|(k, _)| self.callback_c_name(k));
        for (key, cfg) in cb_keys {
            let args: Vec<syn::Type> = cfg.args.clone();
            // Emit only if the callback is required (its input resolved); skip a
            // declared-but-unused signature.
            if registry
                .reading_of(&callback_fn_type(&args))
                .and_then(|tr| self.in_frag(&tr))
                .is_none()
            {
                continue;
            }
            let takeable = &self.callbacks.get(key).expect("callback cfg").takeable;
            let mut arg_wires: Vec<syn::Type> = Vec::new();
            for (i, a) in args.iter().enumerate() {
                // `&[E]` slice arg → TWO C `call` params: `const E_wire *` + `size_t`
                // (the slice delivered by reference, zero-copy).
                if let Some((_src, elem_wire)) = self.callback_slice_elem_wire(a) {
                    arg_wires.push(syn::parse_quote!(*const #elem_wire));
                    arg_wires.push(syn::parse_quote!(usize));
                    continue;
                }
                let reading = registry.reading_of(a).unwrap_or_else(|| {
                    panic!(
                        "Cbindgen: callback arg `{}` was never classified",
                        a.to_token_stream()
                    )
                });
                let wire = self
                    .out_frag(&reading)
                    .unwrap_or_else(|| {
                        panic!(
                            "Cbindgen: callback arg `{}` has no output converter (declare it \
                             as a opaque_ptr/data_struct/enum_type)",
                            a.to_token_stream()
                        )
                    })
                    .destination
                    .clone();
                // Takeable params are delivered as an owned pointer.
                if takeable.contains(&i) {
                    arg_wires.push(syn::parse_quote!(*mut #wire));
                    continue;
                }
                // A composite has no wire of its own, so its C params are the
                // fields its shape lowers to — exactly as `dispatch_fn_input`
                // fills them (#428). Each is `MaybeUninit`: an absent value
                // leaves its slot unwritten, and the wrapper must not build a
                // Rust value to fill it with. `#[repr(transparent)]` keeps both
                // the C ABI and the header spelling.
                if marker_destination(&wire) && self.is_lowered_composite(&reading) {
                    for field in self.lower_shape(&reading, registry).fields {
                        let w = field.wire;
                        arg_wires.push(syn::parse_quote!(::core::mem::MaybeUninit<#w>));
                    }
                    continue;
                }
                arg_wires.push(wire);
            }
            let c_struct = self.callback_c_ident(key);
            items.push(syn::parse_quote!(
                #[repr(C)]
                #[allow(non_camel_case_types)]
                pub struct #c_struct {
                    pub context: *mut ::core::ffi::c_void,
                    pub call: ::core::option::Option<
                        unsafe extern "C" fn(#(#arg_wires,)* *mut ::core::ffi::c_void),
                    >,
                    pub drop: ::core::option::Option<
                        unsafe extern "C" fn(*mut ::core::ffi::c_void),
                    >,
                }
            ));
        }
        items
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
        // The driver's state lives on `self` rather than here, because the
        // adapter reads it **while** it compiles: `dispatch_fn_input` builds a
        // callback's closure struct out of `lower_shape` and `encode_value`,
        // both of which ask what a type crosses as. Handing the compiler the
        // store by `mem::take` would empty it for exactly the span of that
        // call, so it is cloned — a map of `Rc`s.
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
        // Freeze every ordinary function position and the fragment graph it reaches.
        let sites = {
            let mut compiler = prebindgen_registry::recipe::Compiler::resume(
                &model,
                &recipes,
                &bindings,
                self.compiled.borrow().clone(),
            );
            let sites = self
                .compile_sites(&mut compiler, &registry)
                .map_err(|message| prebindgen_registry::ScanError::AdapterInvariant { message })?;
            *self.compiled.borrow_mut() = compiler.finish();
            sites
        };
        let mut generation = prebindgen_registry::generation::GenerationPlanBuilder::new();
        for fragment in self.compiled.borrow().fragments() {
            generation.fragment(fragment.freeze());
        }
        for site in sites {
            generation.site(site);
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

impl CbindgenBuilder {
    pub(crate) fn dispatch_fn_input(
        &self,
        source: &TypeRef,
        args: &[TypeRef],
        fragments: Option<&[&crate::compile::CFrag]>,
        registry: &impl Conversions,
    ) -> Option<(syn::Type, crate::chain::CFunction)> {
        let key: CallbackKey = args.iter().map(|a| a.key()).collect();
        if !self.callbacks.contains_key(&key) {
            // Undeclared callback signature: leave unresolved so the registry
            // reports it (the consumer must `.callback(...)`-declare it).
            return None;
        }
        let c_struct = self.callback_c_ident(&key);

        // Per-arg: closure parameter (`__aN: <src>`) + encode statement
        // (`let __wN = <output_conv>(__aN);`, panicking if the converter is
        // fallible — a firing callback has no error channel). A non-takeable arg
        // is passed to the C `call` by value (the C side owns + drops it); a
        // **takeable** arg is passed as `&mut __wN` (`*mut z_x_t`) and dropped here
        // after the call (no-op if the C side took it, leaving a gravestone).
        let takeable = &self.callbacks.get(&key).expect("callback cfg").takeable;
        let mut parts: Vec<crate::chain::InvokePart> = Vec::new();
        for (i, arg) in args.iter().enumerate() {
            let mut encode_stmts = TokenStream::new();
            let mut call_args: Vec<TokenStream> = Vec::new();
            let mut post_drops = TokenStream::new();
            // `&[E]` slice arg: deliver the slice to the C `call` **by reference** —
            // `(*const E_wire, size_t)`, zero-copy (the closure borrows the slice for
            // the call). The element wire is layout-identical to `E`, so the pointer
            // cast is sound; no per-element encode and no post-call drop.
            if let Some(elem_wire) = self.callback_slice_elem_wire_type_of(arg) {
                let ai = crate::chain::invoke_argument_name(i);
                call_args.push(quote!(#ai.as_ptr() as *const #elem_wire));
                call_args.push(quote!(#ai.len()));
                parts.push(crate::chain::InvokePart {
                    source: ai,
                    prepare: encode_stmts,
                    arguments: call_args,
                    cleanup: post_drops,
                });
                continue;
            }
            let supplied = fragments.and_then(|fragments| fragments.get(i).copied());
            let fallback = supplied.is_none().then(|| self.out_frag(arg)).flatten();
            let entry = supplied.or(fallback.as_deref())?;
            let conv = entry.function.call().ident().clone();
            let opaque = entry.destination.clone();
            let fallible = entry.function.call().fallible();
            let ai = crate::chain::invoke_argument_name(i);
            let wi = format_ident!("__w{}", i);
            let is_takeable = takeable.contains(&i);
            // A COMPOSITE argument — `Option<T>`, `Vec<T>`, `Cow<'_, [T]>` — has
            // no converter of its own: `out_wrappers` gives it a marker with a
            // `()` destination, which exists to resolve the entry and make the
            // inner required while the real ABI is structural. The return path
            // lowers those in `lower_shape` / `encode_value`; this one used to
            // call the marker as if it were a converter, which takes no
            // arguments (#428). Same lowering, so the two directions cannot
            // disagree about which shapes they know.
            //
            // A takeable argument is a whole-value policy over an opaque handle
            // and never a composite, so it keeps the by-reference path below.
            //
            // Which shapes those are is the MODEL's answer, not the marker's: a
            // `()` destination says the type has no wire of its own, and a
            // `Result` has one of those too while no arm lowers it. Field COUNT
            // cannot say it either — `Option<&T>` carves the pointer's niche and
            // lowers to a single `*const`, one field and still nothing a
            // converter call can produce.
            if !is_takeable
                && marker_destination(&entry.destination)
                && !self.is_lowered_composite(arg)
            {
                panic!(
                    "Cbindgen: callback argument `{}` has no C ABI — it resolves to a marker \
                     converter and is not one of the shapes lowered structurally (`Option<T>`, \
                     `Vec<T>`, `Cow<'_, [T]>`). Deliver its parts as separate callback \
                     arguments instead.",
                    arg,
                );
            }
            // Both halves of the marker test, and both are load-bearing. The
            // MODEL says which shapes `lower_shape` decomposes; the marker says
            // this type has no wire of its own — and a `convert!`-declared
            // `Option<T>` has one, because `out_custom` is tried before
            // `out_wrappers`. Decomposing that from its shape alone would pass
            // several arguments to a `call` the struct declared with one
            // (#428 review).
            let composite = !is_takeable
                && marker_destination(&entry.destination)
                && self.is_lowered_composite(arg);
            if composite {
                let shape = self.lower_shape(arg, registry);
                let mut targets = Vec::new();
                for (f, field) in shape.fields.iter().enumerate() {
                    let fi = if shape.fields.len() == 1 {
                        wi.clone()
                    } else {
                        format_ident!("__w{}_{}", i, f)
                    };
                    let wire = &field.wire;
                    // A `MaybeUninit`, zeroed. Two things it must not be.
                    //
                    // Not a `wire` value: a shape with a `present` flag writes
                    // only the flag when the value is absent, and materialising
                    // something to fill the slot is undefined for a wire whose
                    // all-zero pattern is not a legal value of its type — a
                    // declared `enum_type`'s discriminants are the source's own,
                    // so zero need not name a variant at all.
                    //
                    // And not left indeterminate: the slot is passed BY VALUE to
                    // foreign code, so whatever the stack or register held is
                    // handed to a C callback that reads it despite the flag.
                    // Zeroing costs a store and discloses nothing, while
                    // `MaybeUninit` keeps it from ever being a `wire` (#428
                    // review).
                    //
                    // Neither assumes anything about WHICH fields the encode
                    // writes, which is the encoder's business and not this
                    // caller's.
                    encode_stmts
                        .extend(quote!(let mut #fi = ::core::mem::MaybeUninit::<#wire>::zeroed();));
                    targets.push(quote!(*#fi.as_mut_ptr()));
                    call_args.push(quote!(#fi));
                }
                // A firing callback has no error channel, so a fallible
                // converter aborts — the same answer the single-value path
                // below gives, spelled by the route the emitters share.
                encode_stmts.extend(self.encode_value(
                    arg,
                    quote!(#ai),
                    &targets,
                    registry,
                    &ErrRoute::Panic,
                ));
                parts.push(crate::chain::InvokePart {
                    source: ai.clone(),
                    prepare: encode_stmts,
                    arguments: call_args,
                    cleanup: post_drops,
                });
                continue;
            }
            let mut_kw = if is_takeable { quote!(mut) } else { quote!() };
            if fallible {
                encode_stmts.extend(quote!(
                    let #mut_kw #wi = match #conv(#ai) {
                        ::core::result::Result::Ok(__v) => __v,
                        ::core::result::Result::Err(__e) => {
                            ::core::panic!("cbindgen: callback argument conversion failed: {}", __e)
                        }
                    };
                ));
            } else {
                encode_stmts.extend(quote!(let #mut_kw #wi = #conv(#ai);));
            }
            if is_takeable {
                call_args.push(quote!(&mut #wi as *mut #opaque));
                // Always drop after the call (leak-safe): live value if untaken,
                // gravestone (no-op) if the C side took it via `z_x_take`.
                post_drops.extend(
                    quote!(let _ = <#opaque as ::prebindgen_c_runtime::Transmute>::into_rust(#wi);),
                );
            } else {
                call_args.push(quote!(#wi));
            }
            parts.push(crate::chain::InvokePart {
                source: ai.clone(),
                prepare: encode_stmts,
                arguments: call_args,
                cleanup: post_drops,
            });
        }

        let name = format_ident!("__cbg_in_{}", self.callback_c_name(&key));
        let wire: syn::Type = syn::parse_quote!(#c_struct);
        let function = crate::chain::CFunction::invoke(crate::chain::InvokePlan {
            ident: name,
            source: source.clone(),
            source_module: self.source_module.clone(),
            wire: wire.clone(),
            arguments: args.to_vec(),
            parts,
        });
        Some((wire, function))
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
        items.extend(self.prereq_callback_structs(registry));
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
        _r: &impl Conversions,
        _emit: &prebindgen_registry::Emit,
    ) -> Option<ConverterImpl> {
        // Unit return: trivial converter so `()` (and `Result<(), _>`) resolves.
        // Never actually called — void-returning wrappers ignore it, and
        // `emit_fallible_wrapper` special-cases `Result<(), E>` to drop the
        // out-param entirely (it exists only to satisfy the resolver).
        if matches!(ty.kind(), TypeKind::Unit) {
            let function: syn::ItemFn = syn::parse_quote!(
                #[allow(non_snake_case, dead_code, unused_variables)]
                pub(crate) fn __cbg_out_unit(v: ()) {}
            );
            return Some(ConverterImpl {
                subs: vec![],
                destination: syn::parse_quote!(()),
                function,
                pre_stages: vec![],
                niches: Niches::empty(),
                metadata: (),
            });
        }

        // `String` output: a `malloc`'d `char*` raw block freed via the
        // `free_memory_function`. A `String` explicitly declared `opaque_ptr`
        // (held by C as `string_t *`) opts out — the opaque-handle branch below
        // owns it then (mirroring the input side, where `in_opaque_handle` wins).
        if r_is_string(ty) && !self.opaque.contains_key(&ty.key()) {
            let name = Self::out_name_of(&ty.key());
            let function: syn::ItemFn = syn::parse_quote!(
                #[allow(non_snake_case, unused_variables, dead_code)]
                pub(crate) fn #name(v: ::std::string::String) -> *mut ::core::ffi::c_char {
                    __cbg_alloc_cstr(v)
                }
            );
            return Some(ConverterImpl {
                subs: vec![],
                destination: syn::parse_quote!(*mut ::core::ffi::c_char),
                function,
                pre_stages: vec![],
                niches: Niches::empty(),
                metadata: (),
            });
        }

        // FFI-safe scalar (`bool`, integers, floats): identity pass-through.
        if r_is_scalar(ty) {
            let name = Self::out_name_of(&ty.key());
            let spelled = scalar_ty(ty)?;
            let function: syn::ItemFn = syn::parse_quote!(
                #[allow(non_snake_case, unused_variables, dead_code)]
                pub(crate) fn #name(v: #spelled) -> #spelled {
                    v
                }
            );
            return Some(ConverterImpl {
                subs: vec![],
                destination: spelled.clone(),
                function,
                pre_stages: vec![],
                niches: Niches::empty(),
                metadata: (),
            });
        }

        let key = ty.key();

        // Opaque handle output: `Box::into_raw` → the bare `*mut #c_struct` handle.
        if self.opaque.contains_key(&key) {
            let name = Self::out_name_of(&ty.key());
            let c_struct = self.c_type_ident(&ty.key());
            let src = self.src_ty_of(&ty.key());
            let function: syn::ItemFn = syn::parse_quote!(
                #[allow(non_snake_case, unused_variables, dead_code)]
                pub(crate) fn #name(v: #src) -> *mut #c_struct {
                    ::std::boxed::Box::into_raw(::std::boxed::Box::new(v)) as *mut #c_struct
                }
            );
            return Some(ConverterImpl {
                subs: vec![],
                destination: syn::parse_quote!(*mut #c_struct),
                function,
                pre_stages: vec![],
                niches: Niches::empty(),
                metadata: (),
            });
        }

        // Opaque error output (e.g. `ZError`): not a by-value struct — marshal it
        // to a malloc'd `char*` message via the recorded accessor `fn(&E) ->
        // String`. The error out-param of a `Result<_, E>` wrapper is thus
        // `char **e`. Freed by the universal `free_memory_function`.
        if let Some(msg_fn) = self.opaque_errors.get(&key) {
            let name = Self::out_name_of(&ty.key());
            let src = self.src_ty_of(&ty.key());
            let msg_path = self.src_fn(msg_fn);
            let function: syn::ItemFn = syn::parse_quote!(
                #[allow(non_snake_case, unused_variables, dead_code)]
                pub(crate) fn #name(v: #src) -> *mut ::core::ffi::c_char {
                    __cbg_alloc_cstr(#msg_path(&v))
                }
            );
            return Some(ConverterImpl {
                subs: vec![],
                destination: syn::parse_quote!(*mut ::core::ffi::c_char),
                function,
                pre_stages: vec![],
                niches: Niches::empty(),
                metadata: (),
            });
        }

        // Value-opaque output: move the Rust value's bytes into the opaque
        // counterpart, by value (no Box). Size/align equality is asserted at the
        // type's emission site (fail-closed).
        if let Some(opaque) = self.value_opaque_ty_of(&ty.key()) {
            let opaque = opaque.clone();
            let name = Self::out_name_of(&ty.key());
            let src = self.src_ty_of(&ty.key());
            let function: syn::ItemFn = syn::parse_quote!(
                #[allow(non_snake_case, unused_variables, dead_code)]
                pub(crate) fn #name(v: #src) -> #opaque {
                    <#opaque as ::prebindgen_c_runtime::Transmute>::from_rust(v)
                }
            );
            return Some(ConverterImpl {
                subs: vec![],
                destination: opaque,
                function,
                pre_stages: vec![],
                niches: Niches::empty(),
                metadata: (),
            });
        }

        // Enum output: `match` the source enum to the C enum.
        if self.enums.contains_key(&key) {
            let e = unit_enum(_r, &ty.key())?;
            let name = Self::out_name_of(&ty.key());
            let cname = self.c_type_ident(&ty.key());
            let src = self.src_ty_of(&ty.key());
            let arms = e.values.iter().map(|v| {
                let id = &v.name;
                quote!(#src::#id => #cname::#id,)
            });
            let function: syn::ItemFn = syn::parse_quote!(
                #[allow(non_snake_case, unused_variables, dead_code)]
                pub(crate) fn #name(v: #src) -> #cname {
                    match v { #(#arms)* }
                }
            );
            return Some(ConverterImpl {
                subs: vec![],
                destination: syn::parse_quote!(#cname),
                function,
                pre_stages: vec![],
                niches: Niches::empty(),
                metadata: (),
            });
        }

        None
    }
}

/// Structural wrapper-shape resolvers (the post-rank-machinery surface). Each
/// peels `ty`'s outermost layer and composes the inner's converter; `subs`
/// lists the immediate inner(s) it looked up.
impl CbindgenBuilder {
    /// `&[E]` slice **input**: marker only for converter-artifact compatibility.
    /// The frozen site plan owns the two-parameter (`*const E_wire`, `usize`)
    /// ABI and zero-copy decode.
    pub(crate) fn in_slice(&self, ty: &TypeRef) -> Option<ConverterImpl> {
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
        let wire: syn::Type = if let Some(e_ty) = scalar_ty(e) {
            syn::parse_quote!(*const #e_ty)
        } else {
            let counterpart = self.value_opaque_ty_of(&e.key())?.clone();
            syn::parse_quote!(*const #counterpart)
        };
        let name = format_ident!("__cbg_inmark_slice_{}", sanitize(&e.key()));
        let function: syn::ItemFn = syn::parse_quote!(
            #[allow(non_snake_case, dead_code, unused)]
            pub(crate) fn #name() {}
        );
        Some(ConverterImpl {
            subs: vec![e.key()],
            destination: wire,
            function,
            pre_stages: vec![],
            niches: Niches::empty(),
            metadata: (),
        })
    }

    /// `&str`, `&mut T` and `&T` **input** shapes: a borrow reached through the
    /// pointer the C caller supplied.
    pub(crate) fn in_borrow(&self, ty: &TypeRef) -> Option<ConverterImpl> {
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

        // `&str`: borrow a UTF-8 C string directly from the caller.
        if !*rf_mut && r_is_str(rf_inner) {
            let name = Self::in_name_of(&ty.key());
            let function: syn::ItemFn = syn::parse_quote!(
                #[allow(non_snake_case, unused_variables, dead_code)]
                pub(crate) unsafe fn #name<'a>(
                    v: *const ::core::ffi::c_char,
                ) -> ::core::result::Result<&'a str, ::std::string::String> {
                    if v.is_null() {
                        return ::core::result::Result::Err(
                            ::std::string::String::from("null pointer passed for str argument"),
                        );
                    }
                    match ::std::ffi::CStr::from_ptr(v).to_str() {
                        ::core::result::Result::Ok(s) => ::core::result::Result::Ok(s),
                        ::core::result::Result::Err(_) => ::core::result::Result::Err(
                            ::std::string::String::from("invalid UTF-8 in str argument"),
                        ),
                    }
                }
            );
            return Some(ConverterImpl {
                subs: vec![elem.key()],
                destination: syn::parse_quote!(*const ::core::ffi::c_char),
                function,
                pre_stages: vec![],
                niches: Niches::empty(),
                metadata: (),
            });
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
                let name = Self::in_name_of(&ty.key());
                let src = self.src_ty_of(&inner.key());
                let short = type_short(&inner.key());
                let null_ptr_msg = format!("null {short} pointer");
                let function: syn::ItemFn = syn::parse_quote!(
                    #[allow(non_snake_case, unused_variables, dead_code)]
                    pub(crate) unsafe fn #name<'a>(
                        v: *mut #op,
                    ) -> ::core::result::Result<&'a mut ::core::mem::MaybeUninit<#src>, ::std::string::String> {
                        if v.is_null() {
                            return ::core::result::Result::Err(
                                ::std::string::String::from(#null_ptr_msg),
                            );
                        }
                        ::core::result::Result::Ok(&mut *(v as *mut ::core::mem::MaybeUninit<#src>))
                    }
                );
                return Some(ConverterImpl {
                    subs: vec![inner.key()],
                    destination: syn::parse_quote!(*mut #op),
                    function,
                    pre_stages: vec![],
                    niches: Niches::empty(),
                    metadata: (),
                });
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
            let name = Self::in_name_of(&ty.key());
            let src = self.src_ty_of(&elem.key());
            let short = type_short(&elem.key());
            let null_ptr_msg = format!("null {short} pointer");
            let function: syn::ItemFn = syn::parse_quote!(
                #[allow(non_snake_case, unused_variables, dead_code)]
                pub(crate) unsafe fn #name<'a>(
                    v: *mut #wire_ty,
                ) -> ::core::result::Result<&'a mut #src, ::std::string::String> {
                    if v.is_null() {
                        return ::core::result::Result::Err(
                            ::std::string::String::from(#null_ptr_msg),
                        );
                    }
                    ::core::result::Result::Ok(&mut *(v as *mut #src))
                }
            );
            return Some(ConverterImpl {
                subs: vec![elem.key()],
                destination: syn::parse_quote!(*mut #wire_ty),
                function,
                pre_stages: vec![],
                niches: Niches::empty(),
                metadata: (),
            });
        }
        // `&T` (shared borrow) of an opaque handle or value-opaque type.
        let key1 = elem.key();
        let wire_ty: syn::Type = if self.opaque.contains_key(&key1) {
            let c_struct = self.c_type_ident(&elem.key());
            syn::parse_quote!(#c_struct)
        } else {
            self.value_opaque_ty_of(&elem.key())?.clone()
        };
        let name = Self::in_name_of(&ty.key());
        let src = self.src_ty_of(&elem.key());
        let short = type_short(&elem.key());
        let null_ptr_msg = format!("null {short} pointer");
        let function: syn::ItemFn = syn::parse_quote!(
            #[allow(non_snake_case, unused_variables, dead_code)]
            pub(crate) unsafe fn #name<'a>(
                v: *const #wire_ty,
            ) -> ::core::result::Result<&'a #src, ::std::string::String> {
                if v.is_null() {
                    return ::core::result::Result::Err(::std::string::String::from(#null_ptr_msg));
                }
                ::core::result::Result::Ok(&*(v as *const #src))
            }
        );
        Some(ConverterImpl {
            subs: vec![elem.key()],
            destination: syn::parse_quote!(*const #wire_ty),
            function,
            pre_stages: vec![],
            niches: Niches::empty(),
            metadata: (),
        })
    }

    /// The `Option<X>` / `Vec<X>` / `Cow<'_, [X]>` **output** marker.
    ///
    /// Carries a `()` destination: the real lowering is structural in
    /// `emit_function_wrapper`, and this exists so the shape resolves and its
    /// inner is marked reachable.
    pub(crate) fn out_arity_marker(&self, kind: &str, inner: &TypeRef) -> ConverterImpl {
        let name = format_ident!("__cbg_outmark_{}_{}", kind, sanitize(&inner.key()));
        let function: syn::ItemFn = syn::parse_quote!(
            #[allow(non_snake_case, dead_code, unused)]
            pub(crate) fn #name() {}
        );
        ConverterImpl {
            subs: vec![inner.key()],
            destination: syn::parse_quote!(()),
            function,
            pre_stages: vec![],
            niches: Niches::empty(),
            metadata: (),
        }
    }

    /// The `&[E]` shared-slice **output** marker — a callback argument.
    ///
    /// The real two-component `(*const E_wire, size_t)` lowering of the closure
    /// `call` parameter is structural in `prereq_callback_structs` /
    /// `dispatch_fn_input`; `subs: [E]` forces E's output so the closure wire
    /// element type exists.
    pub(crate) fn out_slice_marker(&self, ty: &TypeRef) -> Option<ConverterImpl> {
        let elem = self
            .r_value_opaque_slice_elem(ty)
            .or_else(|| r_scalar_slice_elem(ty))?;
        let name = format_ident!("__cbg_outmark_slice_{}", sanitize(&elem.key()));
        let function: syn::ItemFn = syn::parse_quote!(
            #[allow(non_snake_case, dead_code, unused)]
            pub(crate) fn #name() {}
        );
        Some(ConverterImpl {
            subs: vec![elem.key()],
            destination: syn::parse_quote!(()),
            function,
            pre_stages: vec![],
            niches: Niches::empty(),
            metadata: (),
        })
    }

    /// The `&T` shared borrow and the `Result<T, E>` marker — the two **output**
    /// shapes that are neither terminal nor a run.
    pub(crate) fn out_borrow_or_result(&self, ty: &TypeRef) -> Option<ConverterImpl> {
        // `&T` shared borrow of an opaque/value-opaque type → non-owning `*const`.
        if let TypeKind::Ref { mutable, inner, .. } = ty.kind() {
            if !*mutable {
                let key = inner.key();
                let wire_ty: syn::Type = if self.opaque.contains_key(&key) {
                    let c_struct = self.c_type_ident(&key);
                    syn::parse_quote!(#c_struct)
                } else {
                    self.value_opaque_ty_of(&key)?.clone()
                };
                let src = self.src_ty_of(&key);
                let name = format_ident!("__cbg_out_ref_{}", sanitize(&key));
                let function: syn::ItemFn = syn::parse_quote!(
                    #[allow(non_snake_case, dead_code, unused)]
                    pub(crate) unsafe fn #name(v: &#src) -> *const #wire_ty {
                        v as *const #src as *const #wire_ty
                    }
                );
                return Some(ConverterImpl {
                    subs: vec![key],
                    destination: syn::parse_quote!(*const #wire_ty),
                    function,
                    pre_stages: vec![],
                    niches: Niches::empty(),
                    metadata: (),
                });
            }
            return None;
        }
        // `Result<T, E>` marker — real lowering (bool + out-param + error-param)
        // is in `on_function`.
        let (ok, err) = ty.fallible_parts()?;
        let name = format_ident!("__cbg_result_{}", sanitize(&ty.key()));
        let function: syn::ItemFn = syn::parse_quote!(
            #[allow(non_snake_case, dead_code, unused)]
            pub(crate) fn #name() {}
        );
        Some(ConverterImpl {
            subs: vec![ok.key(), err.key()],
            destination: syn::parse_quote!(()),
            function,
            pre_stages: vec![],
            niches: Niches::empty(),
            metadata: (),
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
