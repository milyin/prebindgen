use super::{builder::callback_fn_type, *};
use crate::api::core::registry::{Building, Conversions, Crossing, RegistryBuilder};

/// Per-category **input** terminal converter builders. Each returns
/// `Some(ConverterImpl)` only for the type category it claims (and `None`
/// otherwise); [`Prebindgen::on_input_type`] chains them in priority order
/// before the wrapper shapes. The categories are mutually exclusive, so the
/// chain's fall-through is equivalent to a sequential `if … return` block.
impl CbindgenBuilder {
    /// Opaque handle, by-value consume: `*Box::from_raw(v)` — fallible (null
    /// handle → message). The wire is the bare handle pointer `*mut #c_struct`.
    pub(crate) fn in_opaque_handle(&self, ty: &syn::Type) -> Option<ConverterImpl<()>> {
        let key = TypeKey::from_type(ty);
        if !self.opaque.contains_key(&key) {
            return None;
        }
        let name = Self::in_name(ty);
        let c_struct = self.c_type_ident(ty);
        let src = self.src_ty(ty);
        let short = type_short(ty);
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

    /// Data struct: decode each field from its C wire — infallible.
    pub(crate) fn in_data_struct(
        &self,
        ty: &syn::Type,
        r: &impl Conversions<()>,
    ) -> Option<ConverterImpl<()>> {
        let key = TypeKey::from_type(ty);
        if !self.data.contains_key(&key) {
            return None;
        }
        let fields = self.struct_fields(r, ty)?;
        let name = Self::in_name(ty);
        let c_struct = self.c_type_ident(ty);
        let src = self.src_ty(ty);
        let mut inits: Vec<TokenStream> = Vec::new();
        let mut subs: Vec<syn::Type> = Vec::new();
        let mut fallible = false;
        for (fname, fty) in &fields {
            if is_string(fty) {
                inits.push(quote!(#fname: if v.#fname.is_null() {
                    ::std::string::String::new()
                } else {
                    ::std::ffi::CStr::from_ptr(v.#fname).to_string_lossy().into_owned()
                }));
            } else if self.tagged_unions.contains_key(&TypeKey::from_type(fty)) {
                // A sum field crosses by value as its mirror; its own converter
                // validates the tag and rebuilds the live arm, which is what
                // makes this whole decode fallible.
                let conv = Self::in_name(fty);
                subs.push(fty.clone());
                fallible = true;
                inits.push(quote!(#fname: #conv(v.#fname)?));
            } else if is_bool(fty) {
                // #170 instance 2: the field's wire is `MaybeUninit<bool>`, so
                // the byte C wrote is normalised here — a Rust `bool` never
                // holds it unchecked.
                let read = bool_in_expr(quote!(v.#fname));
                inits.push(quote!(#fname: #read));
            } else {
                inits.push(quote!(#fname: v.#fname));
            }
        }
        // Only a union field can fail; a struct of strings and scalars keeps
        // its infallible signature (and its callers keep theirs).
        let function: syn::ItemFn = if fallible {
            syn::parse_quote!(
                #[allow(non_snake_case, unused_variables, dead_code)]
                pub(crate) unsafe fn #name(
                    v: #c_struct,
                ) -> ::core::result::Result<#src, ::std::string::String> {
                    ::core::result::Result::Ok(#src { #(#inits),* })
                }
            )
        } else {
            syn::parse_quote!(
                #[allow(non_snake_case, unused_variables, dead_code)]
                pub(crate) unsafe fn #name(v: #c_struct) -> #src {
                    #src { #(#inits),* }
                }
            )
        };
        Some(ConverterImpl {
            subs,
            destination: syn::parse_quote!(#c_struct),
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
        registry: &impl Conversions<()>,
        ty: &syn::Type,
    ) -> Option<Vec<syn::Ident>> {
        let cfg = self.value_opaque.get(&TypeKey::from_type(ty))?;
        if !cfg.generate_mirror {
            return None;
        }
        let mut idents = Vec::new();
        for (fname, fty) in self.struct_fields(registry, ty)? {
            // An owned-pointer field is one whose mirror wire is a raw pointer
            // (`Option<Box<T>>` / `Box<T>` → `*mut t_t`); scalars/enums are not.
            if matches!(self.mirror_field_wire(&fty), Some(syn::Type::Ptr(_))) {
                if !is_option(&fty) {
                    return None; // bare `Box<T>`: cannot be nulled (invalid `Box`)
                }
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
        registry: &impl Conversions<()>,
        ty: &syn::Type,
        slot: &syn::Ident,
    ) -> Option<TokenStream> {
        let cfg = self.value_opaque.get(&TypeKey::from_type(ty))?;
        let opaque = &cfg.opaque;
        if cfg.generate_mirror {
            match self.nullable_owned_ptr_fields(registry, ty) {
                // No owned-pointer fields ⇒ plain data, nothing to clean up.
                Some(fields) if fields.is_empty() => None,
                // All owned-pointer fields nullable ⇒ null them in place (drop-safe).
                Some(fields) => Some(quote!(#( (*#slot).#fields = ::core::ptr::null_mut(); )*)),
                // Bare `Box<T>` field ⇒ a NULL would be an invalid `Box`; full gravestone.
                None => Some(
                    quote!(::core::ptr::write(#slot, <#opaque as ::prebindgen::Gravestone>::gravestone());),
                ),
            }
        } else {
            // Non-mirror opaque: the consumer chose the kind explicitly.
            match cfg.kind {
                OpaqueKind::Owned => Some(
                    quote!(::core::ptr::write(#slot, <#opaque as ::prebindgen::Gravestone>::gravestone());),
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
    fn mirror_needs_gravestone_impl(&self, registry: &Registry<()>, ty: &syn::Type) -> bool {
        match self.value_opaque.get(&TypeKey::from_type(ty)) {
            Some(cfg) if cfg.generate_mirror => {
                self.nullable_owned_ptr_fields(registry, ty).is_none()
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
        ty: &syn::Type,
        registry: &impl Conversions<()>,
    ) -> Option<ConverterImpl<()>> {
        let opaque = self.value_opaque_ty(ty)?.clone();
        let name = Self::in_name(ty);
        let src = self.src_ty(ty);
        let short = type_short(ty);
        let null_msg = format!("null {short} value passed by value");
        // Owned-ness (whether to clean up the moved-from slot) is inferred from the
        // mirror's fields for a `repr_c_struct`, or the explicit kind for a non-mirror.
        let writeback = self.value_opaque_writeback(registry, ty, &format_ident!("v"));
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
                let __live = <#opaque as ::prebindgen::Transmute>::into_rust(
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
    /// So the wire is [`::core::mem::MaybeUninit<mirror>`], which is
    /// `#[repr(transparent)]` over the mirror (identical ABI, identical C
    /// spelling — cbindgen renders `MaybeUninit<T>` as `T`) and, unlike the
    /// mirror itself, may legally hold **any** bit pattern. The discriminant is
    /// then read out as `c_int` — the representation a `#[repr(C)]` fieldless
    /// enum has by definition, asserted below — and compared against the
    /// mirror's own variants, so a `const`- or `cfg`-driven discriminant needs
    /// no generator-side evaluation. An unmatched value is a binding error
    /// through the wrapper's error channel; no Rust enum is ever constructed
    /// from it.
    pub(crate) fn in_enum(
        &self,
        ty: &syn::Type,
        r: &impl Conversions<()>,
    ) -> Option<ConverterImpl<()>> {
        let key = TypeKey::from_type(ty);
        if !self.enums.contains_key(&key) {
            return None;
        }
        let e = enum_item(r, ty)?;
        assert_unit_enum(e);
        let name = Self::in_name(ty);
        let cname = self.c_type_ident(ty);
        let src = self.src_ty(ty);
        let cname_str = cname.to_string();
        let arms = e.variants.iter().map(|v| {
            let id = &v.ident;
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
    pub(crate) fn in_string(&self, ty: &syn::Type) -> Option<ConverterImpl<()>> {
        if !is_string(ty) {
            return None;
        }
        let name = Self::in_name(ty);
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
    pub(crate) fn in_str(&self, ty: &syn::Type) -> Option<ConverterImpl<()>> {
        if !is_str(ty) {
            return None;
        }
        let name = Self::in_name(ty);
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
    pub(crate) fn in_bool(&self, ty: &syn::Type) -> Option<ConverterImpl<()>> {
        if !is_bool(ty) {
            return None;
        }
        let name = Self::in_name(ty);
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

    /// FFI-safe scalar (integers, floats): identity pass-through. `bool` is
    /// claimed earlier by [`Self::in_bool`] and never reaches here.
    pub(crate) fn in_scalar(&self, ty: &syn::Type) -> Option<ConverterImpl<()>> {
        if !is_scalar(ty) || is_bool(ty) {
            return None;
        }
        let name = Self::in_name(ty);
        let function: syn::ItemFn = syn::parse_quote!(
            #[allow(non_snake_case, unused_variables, dead_code)]
            pub(crate) fn #name(v: #ty) -> #ty {
                v
            }
        );
        Some(ConverterImpl {
            subs: vec![],
            destination: ty.clone(),
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
    fn prereq_alloc_free(&self, registry: &Registry<()>, produces_array: bool) -> Vec<syn::Item> {
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
    fn prereq_opaque_handles(&self, registry: &Registry<()>) -> Vec<syn::Item> {
        let mut items: Vec<syn::Item> = Vec::new();
        for (key, _cfg) in sorted_by_key(&self.opaque) {
            let ty = key.to_type();
            if registry.input_entry(&ty).is_none() && registry.output_entry(&ty).is_none() {
                continue;
            }
            let c_struct = self.c_type_ident(&ty);
            // Opaque/incomplete C type: the handle is `#c_struct *`, which IS the
            // `Box::into_raw` pointer to the source value.
            items.push(syn::parse_quote!(
                #[repr(C)]
                #[allow(non_camel_case_types)]
                pub struct #c_struct {
                    _private: [u8; 0],
                }
            ));
            let src = self.src_ty(&ty);
            let drop_ident = self.destructor_symbol(&ty);
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
    fn prereq_data_structs(&self, registry: &Registry<()>) -> Vec<syn::Item> {
        let mut items: Vec<syn::Item> = Vec::new();
        for (key, _cfg) in sorted_by_key(&self.data) {
            let ty = key.to_type();
            if registry.input_entry(&ty).is_none() && registry.output_entry(&ty).is_none() {
                continue;
            }
            let Some(fields) = self.struct_fields(registry, &ty) else {
                continue;
            };
            let c_struct = self.c_type_ident(&ty);
            let mut field_defs: Vec<TokenStream> = Vec::new();
            for (fname, fty) in &fields {
                let wire = self.data_field_wire(fty).unwrap_or_else(|| {
                    panic!(
                        "Cbindgen: field `{}` of data struct `{}` has unsupported type `{}`",
                        fname,
                        type_short(&ty),
                        fty.to_token_stream()
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
    fn prereq_value_opaque(&self, registry: &Registry<()>) -> Vec<syn::Item> {
        let mut items: Vec<syn::Item> = Vec::new();
        let takeable_keys = self.takeable_type_keys();
        let mut vo: Vec<(&TypeKey, &ValueOpaqueCfg)> = self.value_opaque.iter().collect();
        vo.sort_by(|a, b| a.0.as_str().cmp(b.0.as_str()));
        for (key, cfg) in vo {
            let ty = key.to_type();
            if registry.input_entry(&ty).is_none() && registry.output_entry(&ty).is_none() {
                continue;
            }
            let src = self.src_ty(&ty);
            let opaque = &cfg.opaque;
            // `repr_c_struct`: the opaque counterpart is an auto-generated
            // **visible-field** `#[repr(C)]` mirror (so C reads the fields directly),
            // not an externally-provided blob. Each field is lowered by
            // `mirror_field_wire` (scalar / enum / opaque pointer). The size/align
            // assert below then proves the whole-struct reinterpret sound.
            if cfg.generate_mirror {
                let mirror_ident = self.c_type_ident(&ty);
                let fields = self.struct_fields(registry, &ty).unwrap_or_else(|| {
                    panic!(
                        "Cbindgen::repr_c_struct: `{}` is not a named struct",
                        type_short(&ty)
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
                let restricted = self.restricted_validity_fields(registry, &ty);
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
                        type_short(&ty),
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
                                type_short(&ty),
                                fty.to_token_stream()
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
                if self.mirror_needs_gravestone_impl(registry, &ty) {
                    items.push(syn::parse_quote!(
                        impl ::prebindgen::Gravestone for #mirror_ident {
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
                impl ::prebindgen::Transmute for #opaque {
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
            let drop_ident = self.destructor_symbol(&ty);
            // Unconditional drop: safe because a moved-from slot holds a
            // gravestone (a valid, safely-droppable empty value), so dropping
            // it is a harmless no-op; a live slot drops normally.
            items.push(syn::parse_quote!(
                #[no_mangle]
                #[allow(non_snake_case, unused_variables)]
                pub unsafe extern "C" fn #drop_ident(this_: *mut #opaque) {
                    if !this_.is_null() {
                        ::core::ptr::drop_in_place(
                            <#opaque as ::prebindgen::Transmute>::as_rust_mut(&mut *this_),
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
                let take_ident = self.take_symbol(&ty);
                // Same inferred write-back as a consume (field-null for a nullable
                // mirror, `gravestone()` for a bare-`Box` mirror / non-mirror owned).
                let writeback = self.value_opaque_writeback(registry, &ty, &format_ident!("src"));
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
    /// [`enum_discriminant_values`](crate::api::core::types_util::enum_discriminant_values).
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
    fn prereq_enums(&self, registry: &Registry<()>) -> Vec<syn::Item> {
        let mut items: Vec<syn::Item> = Vec::new();
        for (key, _cfg) in sorted_by_key(&self.enums) {
            let ty = key.to_type();
            if registry.input_entry(&ty).is_none() && registry.output_entry(&ty).is_none() {
                continue;
            }
            let Some(e) = enum_item(registry, &ty) else {
                continue;
            };
            assert_unit_enum(e);
            let cname = self.c_type_ident(&ty);
            let variants = e.variants.iter().map(|v| {
                let id = &v.ident;
                match &v.discriminant {
                    Some((_, expr)) => quote!(#id = #expr),
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
    fn prereq_tagged_unions(&self, registry: &Registry<()>) -> Vec<syn::Item> {
        let mut items: Vec<syn::Item> = Vec::new();
        for (key, _cfg) in sorted_by_key(&self.tagged_unions) {
            let ty = key.to_type();
            if registry.input_entry(&ty).is_none() && registry.output_entry(&ty).is_none() {
                continue;
            }
            let Some(e) = enum_item(registry, &ty) else {
                continue;
            };
            assert_payload_enum(e);
            let cname = self.c_type_ident(&ty);

            let mut variant_defs: Vec<TokenStream> = Vec::new();
            // Per-variant drop arm, collected only for variants that own
            // something; the rest fall to a single wildcard arm.
            let mut drop_arms: Vec<TokenStream> = Vec::new();
            for v in &e.variants {
                let vident = &v.ident;
                let wires: Vec<syn::Type> = v
                    .fields
                    .iter()
                    .map(|f| self.payload_wire_of(&ty, vident, f, registry))
                    .collect();
                match &v.fields {
                    syn::Fields::Unit => variant_defs.push(quote!(#vident)),
                    syn::Fields::Named(named) => {
                        let defs = named.named.iter().zip(&wires).map(|(f, w)| {
                            let n = f.ident.as_ref().expect("named field");
                            quote!(#n: #w)
                        });
                        variant_defs.push(quote!(#vident { #(#defs),* }));
                    }
                    syn::Fields::Unnamed(_) => {
                        variant_defs.push(quote!(#vident(#(#wires),*)));
                    }
                }

                // Drop arm: bind every field, free the owning ones.
                let owning: Vec<(usize, &syn::Field, &syn::Type)> = v
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
                let binds: Vec<syn::Ident> = (0..v.fields.len())
                    .map(|i| format_ident!("__f{}", i))
                    .collect();
                let pattern = variant_pattern(&cname, vident, &v.fields, &binds);
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
            if self.tagged_union_has_drop(&ty, registry) {
                debug_assert!(!drop_arms.is_empty(), "has_drop implies an owning arm");
                let drop_ident = self.destructor_symbol(&ty);
                // The drop is a second C entry point into the same bytes, so it
                // owes the same tag check as the input converter — `&mut *this_`
                // on an out-of-range tag would be the very UB that check exists
                // to prevent. It emits that check from the same place, and,
                // having nowhere to report to, ignores the value (there is no
                // live arm to release), which keeps `_drop` the always-safe
                // no-op it is everywhere else.
                let tag_guard =
                    self.tag_guard(&cname, e.variants.len(), quote!((*this_)), quote!(return;));
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
    fn payload_wire_of(
        &self,
        ty: &syn::Type,
        variant: &syn::Ident,
        field: &syn::Field,
        registry: &Registry<()>,
    ) -> syn::Type {
        self.payload_field_wire(&field.ty, registry)
            .unwrap_or_else(|reason| {
                panic!(
                    "Cbindgen::tagged_union: payload `{}::{}{}` of type `{}` cannot cross: {}",
                    type_short(ty),
                    variant,
                    match &field.ident {
                        Some(n) => format!(".{n}"),
                        None => String::new(),
                    },
                    field.ty.to_token_stream(),
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
        fty: &syn::Type,
        binding: &syn::Ident,
        registry: &Registry<()>,
    ) -> TokenStream {
        if is_string(fty) {
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
                if is_string(fty) {
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
                    let drop_ident = self.destructor_symbol(fty);
                    quote!(#drop_ident(&mut (*#binding).#fname);)
                } else {
                    // `owning_data_struct_fields` yields exactly the two shapes
                    // above (`data_field_owns`), so this is unreachable — and a
                    // silent fall-through here would be a leak, which is the
                    // defect this whole path exists to prevent.
                    panic!(
                        "Cbindgen: data-struct field `{}` of type `{}` is owning but has no \
                         release form (expected a `String` or a declared `tagged_union`)",
                        fname,
                        fty.to_token_stream(),
                    )
                }
            });
            return quote!(#(#frees)*);
        }
        let inner = opaque_ptr_payload_inner(fty).unwrap_or_else(|| fty.clone());
        let src_inner = self.src_ty(&inner);
        quote!(
            if !(*#binding).is_null() {
                drop(::std::boxed::Box::from_raw(*#binding as *mut #src_inner));
                *#binding = ::core::ptr::null_mut();
            }
        )
    }

    /// Tagged-union **input**: **validate the tag**, then `match` the C union
    /// back to the source enum, converting each arm's payload through the
    /// per-field policy. The generalization of [`Self::in_enum`] from "match
    /// idents" to "match idents and convert each arm's fields" — fallible for
    /// the same reason, and by the same rule (#158): a Rust `enum` must never
    /// be *materialised* from C-supplied bytes without checking first, because
    /// an undeclared discriminant is UB at the boundary, before any `match`.
    ///
    /// So the wire is [`::core::mem::MaybeUninit`] over the mirror. A
    /// `#[repr(C)]` enum with payload variants is laid out as a leading
    /// discriminant of a C `int` followed by the variant union, so the tag is
    /// read from the front as a plain `c_int` and range-checked against the
    /// variants (the mirror carries no explicit discriminants, so its tags are
    /// declaration order `0..N`). Only then is the value `assume_init`ed —
    /// which is sound because [`CbindgenBuilder::payload_field_wire`] makes every
    /// payload wire bit-pattern-agnostic, leaving the tag as the sole
    /// obligation.
    pub(crate) fn in_tagged_union(
        &self,
        ty: &syn::Type,
        r: &impl Conversions<()>,
    ) -> Option<ConverterImpl<()>> {
        let key = TypeKey::from_type(ty);
        if !self.tagged_unions.contains_key(&key) {
            return None;
        }
        let e = enum_item(r, ty)?;
        assert_payload_enum(e);
        let name = Self::in_name(ty);
        let cname = self.c_type_ident(ty);
        let src = self.src_ty(ty);
        // A payload that crosses through its own converter needs that converter
        // to exist before this one can call it. `subs` only drives the
        // post-resolution propagation pass, so it cannot order the build —
        // returning `None` here is the resolver's DEFERRAL protocol, and it
        // retries at the next fixed point. Without this the payload silently
        // degrades to a passthrough and the generated code does not compile.
        for v in &e.variants {
            for f in &v.fields {
                if self.payload_needs_converter(&f.ty) && r.input_entry(&f.ty).is_none() {
                    return None;
                }
            }
        }
        let mut subs: Vec<syn::Type> = Vec::new();
        let arms: Vec<TokenStream> = e
            .variants
            .iter()
            .map(|v| {
                let vident = &v.ident;
                let binds: Vec<syn::Ident> = (0..v.fields.len())
                    .map(|i| format_ident!("__f{}", i))
                    .collect();
                let from = variant_pattern(&cname, vident, &v.fields, &binds);
                let exprs: Vec<TokenStream> = v
                    .fields
                    .iter()
                    .zip(&binds)
                    .map(|(f, b)| {
                        // Every payload that crosses through a converter of its
                        // own — a declared `enum_type`, a nested `data_struct`,
                        // an opaque handle, a converted leaf — is a resolver
                        // dependency, so its converter exists before this one is
                        // emitted. Without it the payload silently falls back to
                        // a passthrough and the generated code does not compile.
                        if self.payload_needs_converter(&f.ty) {
                            subs.push(f.ty.clone());
                        }
                        self.payload_in_expr(&f.ty, b, r)
                    })
                    .collect();
                let to = variant_ctor(&src, vident, &v.fields, &exprs);
                quote!(#from => #to,)
            })
            .collect();
        let bad_msg = format!(
            "invalid tag {{}} for `{cname}` (expected 0..{})",
            e.variants.len()
        );
        let tag_guard = self.tag_guard(
            &cname,
            e.variants.len(),
            quote!(v),
            quote!(return ::core::result::Result::Err(::std::format!(#bad_msg, __tag));),
        );
        let function: syn::ItemFn = syn::parse_quote!(
            #[allow(non_snake_case, unused_variables, dead_code)]
            pub(crate) unsafe fn #name(
                v: ::core::mem::MaybeUninit<#cname>,
            ) -> ::core::result::Result<#src, ::std::string::String> {
                #tag_guard
                let v = v.assume_init();
                ::core::result::Result::Ok(match v { #(#arms)* })
            }
        );
        Some(ConverterImpl {
            subs,
            destination: syn::parse_quote!(::core::mem::MaybeUninit<#cname>),
            function,
            pre_stages: vec![],
            niches: Niches::empty(),
            metadata: (),
        })
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
    fn tag_guard(
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

    /// Tagged-union **output**: `match` the source enum to the C union,
    /// converting each arm's payload. The counterpart of
    /// [`Self::in_tagged_union`]; a `String` payload is allocated here and
    /// released by the union's typed drop.
    pub(crate) fn out_tagged_union(
        &self,
        ty: &syn::Type,
        r: &impl Conversions<()>,
    ) -> Option<ConverterImpl<()>> {
        let key = TypeKey::from_type(ty);
        if !self.tagged_unions.contains_key(&key) {
            return None;
        }
        let e = enum_item(r, ty)?;
        assert_payload_enum(e);
        let name = Self::out_name(ty);
        let cname = self.c_type_ident(ty);
        let src = self.src_ty(ty);
        // Deferral, as in `in_tagged_union` — the output counterpart.
        for v in &e.variants {
            for f in &v.fields {
                if self.payload_needs_converter(&f.ty) && r.output_entry(&f.ty).is_none() {
                    return None;
                }
            }
        }
        let mut subs: Vec<syn::Type> = Vec::new();
        let arms: Vec<TokenStream> = e
            .variants
            .iter()
            .map(|v| {
                let vident = &v.ident;
                let binds: Vec<syn::Ident> = (0..v.fields.len())
                    .map(|i| format_ident!("__f{}", i))
                    .collect();
                let from = variant_pattern(&src, vident, &v.fields, &binds);
                let exprs: Vec<TokenStream> = v
                    .fields
                    .iter()
                    .zip(&binds)
                    .map(|(f, b)| {
                        if self.payload_needs_converter(&f.ty) {
                            subs.push(f.ty.clone());
                        }
                        self.payload_out_expr(&f.ty, b, r)
                    })
                    .collect();
                let to = variant_ctor(&cname_ty(&cname), vident, &v.fields, &exprs);
                quote!(#from => #to,)
            })
            .collect();
        // Same wire as the input direction — one mirror type serves both, and a
        // union carried through a `data_struct` field has only one field type
        // to be. Rust always writes a live arm, so nothing is validated here.
        let function: syn::ItemFn = syn::parse_quote!(
            #[allow(non_snake_case, unused_variables, dead_code)]
            pub(crate) fn #name(v: #src) -> ::core::mem::MaybeUninit<#cname> {
                ::core::mem::MaybeUninit::new(match v { #(#arms)* })
            }
        );
        Some(ConverterImpl {
            subs,
            destination: syn::parse_quote!(::core::mem::MaybeUninit<#cname>),
            function,
            pre_stages: vec![],
            niches: Niches::empty(),
            metadata: (),
        })
    }

    /// One payload field, C wire → Rust value. Mirrors the `data_struct`
    /// input policy, plus the opaque-pointer and declared-enum cases the
    /// mirror wire allows.
    fn payload_in_expr(
        &self,
        fty: &syn::Type,
        b: &syn::Ident,
        registry: &impl Conversions<()>,
    ) -> TokenStream {
        if is_string(fty) {
            return quote!(if #b.is_null() {
                ::std::string::String::new()
            } else {
                ::std::ffi::CStr::from_ptr(#b).to_string_lossy().into_owned()
            });
        }
        if self.enums.contains_key(&TypeKey::from_type(fty)) {
            // The payload rides as `MaybeUninit<enum mirror>` and goes through
            // the same validating decode a top-level enum parameter does; an
            // out-of-range one propagates out of the union's own converter.
            let conv = Self::in_name(fty);
            return quote!(#conv(#b)?);
        }
        if let Some(inner) = opaque_ptr_payload_inner(fty) {
            let src_inner = self.src_ty(&inner);
            let boxed = quote!(::std::boxed::Box::from_raw(#b as *mut #src_inner));
            return if is_option(fty) {
                quote!(if #b.is_null() {
                    ::core::option::Option::None
                } else {
                    ::core::option::Option::Some(#boxed)
                })
            } else {
                // A bare `Box<T>` has no null representation, so a NULL slot
                // cannot be decoded — and it is reachable, not hypothetical:
                // the typed drop nulls the arm it frees, so a union passed back
                // in after being dropped arrives here NULL. Same rule as the
                // tag: report it, never materialise it.
                let null_msg = format!(
                    "null payload for `{}` (a non-optional `Box` payload cannot be NULL — the \
                     union may already have been dropped)",
                    type_short(&inner)
                );
                quote!({
                    if #b.is_null() {
                        return ::core::result::Result::Err(
                            ::std::string::String::from(#null_msg),
                        );
                    }
                    #boxed
                })
            };
        }
        // A `bool` payload rides as `MaybeUninit<bool>` (see `bool_wire`), so
        // the byte C wrote is normalised rather than materialised.
        if is_bool(fty) {
            return bool_in_expr(quote!(#b));
        }
        // A scalar is its own wire and needs no call.
        if is_scalar(fty) {
            return quote!(#b);
        }
        // Everything else rides its own resolved input converter — the wire
        // came from that converter's destination, so the two cannot disagree.
        // A fallible one propagates with `?`, which the union's own `Result`
        // already provides.
        match registry.input_entry(fty) {
            Some(entry) => {
                let conv = &entry.function.sig.ident;
                if returns_result(&entry.function.sig.output) {
                    quote!(#conv(#b)?)
                } else {
                    quote!(#conv(#b))
                }
            }
            None => quote!(#b),
        }
    }

    /// One payload field, Rust value → C wire. The `String` arm allocates the
    /// `char *` block the union's typed drop later frees.
    fn payload_out_expr(
        &self,
        fty: &syn::Type,
        b: &syn::Ident,
        registry: &impl Conversions<()>,
    ) -> TokenStream {
        if is_string(fty) {
            return quote!(__cbg_alloc_cstr(#b));
        }
        if self.enums.contains_key(&TypeKey::from_type(fty)) {
            let conv = Self::out_name(fty);
            return quote!(::core::mem::MaybeUninit::new(#conv(#b)));
        }
        if let Some(inner) = opaque_ptr_payload_inner(fty) {
            let c = self.c_type_ident(&inner);
            return if is_option(fty) {
                quote!(match #b {
                    ::core::option::Option::Some(__b) => {
                        ::std::boxed::Box::into_raw(__b) as *mut #c
                    }
                    ::core::option::Option::None => ::core::ptr::null_mut(),
                })
            } else {
                quote!(::std::boxed::Box::into_raw(#b) as *mut #c)
            };
        }
        // The counterpart of the normalising read above: Rust always writes a
        // valid `0`/`1`, so this only wraps.
        if is_bool(fty) {
            return bool_out_expr(quote!(#b));
        }
        if is_scalar(fty) {
            return quote!(#b);
        }
        // The output counterpart of the input dispatch above. Acceptance —
        // including the refusal of a FALLIBLE output converter, which a union
        // cannot report through — is decided once in `payload_field_wire`, so
        // this site only emits the call.
        match registry.output_entry(fty) {
            Some(entry) => {
                let conv = entry.function.sig.ident.clone();
                quote!(#conv(#b))
            }
            None => quote!(#b),
        }
    }

    /// Callback closure structs: one `#[repr(C)]` `{ context, call, drop }`
    /// per declared signature actually used (its `impl Fn(...)` input
    /// resolved). `call` takes each arg's output wire (the owned handle the
    /// C callback must drop) plus the `void *context`; `drop` releases the
    /// context. Deterministic order by emitted name.
    fn prereq_callback_structs(&self, registry: &Registry<()>) -> Vec<syn::Item> {
        let mut items: Vec<syn::Item> = Vec::new();
        let mut cb_keys: Vec<&CallbackKey> = self.callbacks.keys().collect();
        cb_keys.sort_by_key(|k| {
            let args: Vec<syn::Type> = k.iter().map(|t| t.to_type()).collect();
            self.callback_c_name(&args)
        });
        for key in cb_keys {
            let args: Vec<syn::Type> = key.iter().map(|t| t.to_type()).collect();
            // Emit only if the callback is required (its input resolved); skip a
            // declared-but-unused signature.
            if registry.input_entry(&callback_fn_type(&args)).is_none() {
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
                let wire = registry
                    .output_entry(a)
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
                } else {
                    arg_wires.push(wire);
                }
            }
            let c_struct = self.callback_c_ident(&args);
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
        for (ident, path, sig) in self.convert_decls.iter().flat_map(|decl| &decl.locals) {
            let origin = crate::api::lang::jnigen::jni::local_path_prefix(path);
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
    pub fn build(self) -> Result<Cbindgen, crate::core::WriteRustError> {
        let flat = self
            .sources
            .clone()
            .build()
            .map_err(crate::core::ScanError::from)?;
        let registry = crate::core::Registry::builder(flat)?;
        self.build_with(registry)
    }

    /// [`Self::build`] over a registry described elsewhere — the test seam.
    pub(crate) fn build_with(
        self,
        registry: crate::api::core::registry::RegistryBuilder<()>,
    ) -> Result<Cbindgen, crate::core::WriteRustError> {
        let registry = self
            .declare_into(registry)?
            .validate_with(&self)?
            .convert_with(|crossing, built| self.convert_crossing(crossing, built))?
            .build()?;
        self.validate_resolved(&registry)
            .map_err(|message| crate::core::ScanError::AdapterInvariant { message })?;
        Ok(Cbindgen {
            gen: self,
            registry,
        })
    }

    /// Build the conversion for one crossing — see `JniGenBuilder::convert_crossing`.
    fn convert_crossing(
        &self,
        crossing: &Crossing,
        built: &Building<'_, ()>,
    ) -> Option<ConverterImpl<()>> {
        let (dir, key) = crossing;
        let ty = key.to_type();
        match dir {
            Direction::Input => self.select_input_type(&ty, built).or_else(|| {
                let args = crate::api::core::flat::extract_fn_trait_args(&ty)?;
                self.dispatch_fn_input(&args, built)
            }),
            Direction::Output => self.select_output_type(&ty, built),
        }
    }

    pub fn declare_into(
        &self,
        mut registry: RegistryBuilder<()>,
    ) -> Result<RegistryBuilder<()>, crate::core::ScanError> {
        for (item_fn, origin) in self.collect_local_functions() {
            registry = registry.local_function(item_fn, origin)?;
        }
        for ident in self.declared_functions() {
            registry = registry.export(&ident);
        }
        for ident in self.helper_functions() {
            registry = registry.reference(&ident);
        }
        for key in self.declared_types() {
            registry = registry.export_type(key);
        }
        Ok(registry)
    }
}

impl CbindgenBuilder {
    fn dispatch_fn_input(
        &self,
        args: &[syn::Type],
        registry: &impl Conversions<()>,
    ) -> Option<ConverterImpl<()>> {
        let key: CallbackKey = args.iter().map(TypeKey::from_type).collect();
        if !self.callbacks.contains_key(&key) {
            // Undeclared callback signature: leave unresolved so the registry
            // reports it (the consumer must `.callback(...)`-declare it).
            return None;
        }
        let c_struct = self.callback_c_ident(args);

        // Per-arg: closure parameter (`__aN: <src>`) + encode statement
        // (`let __wN = <output_conv>(__aN);`, panicking if the converter is
        // fallible — a firing callback has no error channel). A non-takeable arg
        // is passed to the C `call` by value (the C side owns + drops it); a
        // **takeable** arg is passed as `&mut __wN` (`*mut z_x_t`) and dropped here
        // after the call (no-op if the C side took it, leaving a gravestone).
        let takeable = &self.callbacks.get(&key).expect("callback cfg").takeable;
        let mut closure_params: Vec<TokenStream> = Vec::new();
        let mut encode_stmts: Vec<TokenStream> = Vec::new();
        let mut call_args: Vec<TokenStream> = Vec::new();
        let mut post_drops: Vec<TokenStream> = Vec::new();
        for (i, arg) in args.iter().enumerate() {
            // `&[E]` slice arg: deliver the slice to the C `call` **by reference** —
            // `(*const E_wire, size_t)`, zero-copy (the closure borrows the slice for
            // the call). The element wire is layout-identical to `E`, so the pointer
            // cast is sound; no per-element encode and no post-call drop.
            if let Some((src_elem, elem_wire)) = self.callback_slice_elem_wire(arg) {
                let ai = format_ident!("__a{}", i);
                closure_params.push(quote!(#ai: &[#src_elem]));
                call_args.push(quote!(#ai.as_ptr() as *const #elem_wire));
                call_args.push(quote!(#ai.len()));
                continue;
            }
            let entry = registry.output_entry(arg)?;
            let conv = entry.function.sig.ident.clone();
            let opaque = entry.destination.clone();
            let fallible = matches!(
                &entry.function.sig.output,
                syn::ReturnType::Type(_, ty) if is_result(ty)
            );
            let src = self.src_ty_deep(arg);
            let ai = format_ident!("__a{}", i);
            let wi = format_ident!("__w{}", i);
            closure_params.push(quote!(#ai: #src));
            let is_takeable = takeable.contains(&i);
            let mut_kw = if is_takeable { quote!(mut) } else { quote!() };
            if fallible {
                encode_stmts.push(quote!(
                    let #mut_kw #wi = match #conv(#ai) {
                        ::core::result::Result::Ok(__v) => __v,
                        ::core::result::Result::Err(__e) => {
                            ::core::panic!("cbindgen: callback argument conversion failed: {}", __e)
                        }
                    };
                ));
            } else {
                encode_stmts.push(quote!(let #mut_kw #wi = #conv(#ai);));
            }
            if is_takeable {
                call_args.push(quote!(&mut #wi as *mut #opaque));
                // Always drop after the call (leak-safe): live value if untaken,
                // gravestone (no-op) if the C side took it via `z_x_take`.
                post_drops
                    .push(quote!(let _ = <#opaque as ::prebindgen::Transmute>::into_rust(#wi);));
            } else {
                call_args.push(quote!(#wi));
            }
        }

        let fn_ty = callback_fn_type(&args.iter().map(|a| self.src_ty_deep(a)).collect::<Vec<_>>());
        let name = format_ident!("__cbg_in_{}", self.callback_c_name(args));
        let function: syn::ItemFn = syn::parse_quote!(
            #[allow(non_snake_case, unused_variables, dead_code)]
            pub(crate) unsafe fn #name(c: #c_struct) -> #fn_ty {
                struct __Ctx {
                    context: *mut ::core::ffi::c_void,
                    drop: ::core::option::Option<unsafe extern "C" fn(*mut ::core::ffi::c_void)>,
                }
                unsafe impl ::core::marker::Send for __Ctx {}
                unsafe impl ::core::marker::Sync for __Ctx {}
                impl ::core::ops::Drop for __Ctx {
                    fn drop(&mut self) {
                        if let ::core::option::Option::Some(__d) = self.drop {
                            unsafe { __d(self.context) }
                        }
                    }
                }
                let __call = c.call;
                let __ctx = ::std::sync::Arc::new(__Ctx { context: c.context, drop: c.drop });
                move |#(#closure_params),*| {
                    #(#encode_stmts)*
                    if let ::core::option::Option::Some(__f) = __call {
                        unsafe { __f(#(#call_args,)* __ctx.context) }
                    }
                    #(#post_drops)*
                }
            }
        );
        Some(ConverterImpl {
            subs: vec![],
            destination: syn::parse_quote!(#c_struct),
            function,
            pre_stages: vec![],
            niches: Niches::empty(),
            metadata: (),
        })
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
    fn validate(&self, binding: &Building<'_, Self::Metadata>) -> Result<(), String> {
        let mut functions = self.declared_functions();
        functions.extend(self.helper_functions());
        crate::core::warn_unclaimed(
            binding.flat(),
            &crate::core::Claimed {
                functions,
                types: self.declared_types(),
                consts: None,
                ignored_functions: self.ignored_functions(),
                ignored_types: self.ignored_types(),
                ..Default::default()
            },
        );
        Ok(())
    }

    type Metadata = ();

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

    fn prerequisites(&self, registry: &Registry<()>) -> Vec<syn::Item> {
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
        items.extend(self.prereq_enums(registry));
        items.extend(self.prereq_tagged_unions(registry));
        items.extend(self.prereq_callback_structs(registry));
        items.extend(self.prereq_domain_constants(registry));
        items
    }

    // ── Item emission ──────────────────────────────────────────────────

    fn on_function(
        &self,
        f: &crate::api::core::flat::Function,
        registry: &Registry<()>,
    ) -> TokenStream {
        self.emit_function_wrapper(&f.origin.syntax, registry)
    }

    fn on_struct(
        &self,
        _s: &crate::api::core::flat::Struct,
        _registry: &Registry<()>,
    ) -> TokenStream {
        // The `#[repr(C)]` mirror + converters come from prerequisites /
        // on_output_type; the original (non-FFI-safe) struct is dropped.
        TokenStream::new()
    }

    fn on_variant(
        &self,
        _v: &crate::api::core::flat::Variant,
        _registry: &Registry<()>,
    ) -> TokenStream {
        TokenStream::new()
    }

    fn on_enum(&self, _e: &crate::api::core::flat::Enum, _registry: &Registry<()>) -> TokenStream {
        TokenStream::new()
    }
}

/// Output-direction terminal categories — the rank-0 chain, now an inherent
/// helper called by [`CbindgenBuilder::select_output_type`].
impl CbindgenBuilder {
    pub(crate) fn out_terminal(
        &self,
        ty: &syn::Type,
        _r: &impl Conversions<()>,
    ) -> Option<ConverterImpl<()>> {
        // Unit return: trivial converter so `()` (and `Result<(), _>`) resolves.
        // Never actually called — void-returning wrappers ignore it, and
        // `emit_fallible_wrapper` special-cases `Result<(), E>` to drop the
        // out-param entirely (it exists only to satisfy the resolver).
        if is_unit(ty) {
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
        if is_string(ty) && !self.opaque.contains_key(&TypeKey::from_type(ty)) {
            let name = Self::out_name(ty);
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
        if is_scalar(ty) {
            let name = Self::out_name(ty);
            let function: syn::ItemFn = syn::parse_quote!(
                #[allow(non_snake_case, unused_variables, dead_code)]
                pub(crate) fn #name(v: #ty) -> #ty {
                    v
                }
            );
            return Some(ConverterImpl {
                subs: vec![],
                destination: ty.clone(),
                function,
                pre_stages: vec![],
                niches: Niches::empty(),
                metadata: (),
            });
        }

        let key = TypeKey::from_type(ty);

        // Opaque handle output: `Box::into_raw` → the bare `*mut #c_struct` handle.
        if self.opaque.contains_key(&key) {
            let name = Self::out_name(ty);
            let c_struct = self.c_type_ident(ty);
            let src = self.src_ty(ty);
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
            let name = Self::out_name(ty);
            let src = self.src_ty(ty);
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

        // Data struct output: encode each field into its C wire (`String` →
        // malloc'd `char*` raw block, freed by the `free_memory_function`).
        if self.data.contains_key(&key) {
            let fields = self.struct_fields(_r, ty)?;
            let name = Self::out_name(ty);
            let c_struct = self.c_type_ident(ty);
            let src = self.src_ty(ty);
            let mut inits: Vec<TokenStream> = Vec::new();
            let mut subs: Vec<syn::Type> = Vec::new();
            for (fname, fty) in &fields {
                if is_string(fty) {
                    inits.push(quote!(#fname: __cbg_alloc_cstr(v.#fname)));
                } else if self.tagged_unions.contains_key(&TypeKey::from_type(fty)) {
                    let conv = Self::out_name(fty);
                    subs.push(fty.clone());
                    inits.push(quote!(#fname: #conv(v.#fname)));
                } else if is_bool(fty) {
                    let wrap = bool_out_expr(quote!(v.#fname));
                    inits.push(quote!(#fname: #wrap));
                } else {
                    inits.push(quote!(#fname: v.#fname));
                }
            }
            let function: syn::ItemFn = syn::parse_quote!(
                #[allow(non_snake_case, unused_variables, dead_code)]
                pub(crate) fn #name(v: #src) -> #c_struct {
                    #c_struct { #(#inits),* }
                }
            );
            return Some(ConverterImpl {
                subs,
                destination: syn::parse_quote!(#c_struct),
                function,
                pre_stages: vec![],
                niches: Niches::empty(),
                metadata: (),
            });
        }

        // Value-opaque output: move the Rust value's bytes into the opaque
        // counterpart, by value (no Box). Size/align equality is asserted at the
        // type's emission site (fail-closed).
        if let Some(opaque) = self.value_opaque_ty(ty) {
            let opaque = opaque.clone();
            let name = Self::out_name(ty);
            let src = self.src_ty(ty);
            let function: syn::ItemFn = syn::parse_quote!(
                #[allow(non_snake_case, unused_variables, dead_code)]
                pub(crate) fn #name(v: #src) -> #opaque {
                    <#opaque as ::prebindgen::Transmute>::from_rust(v)
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
            let e = enum_item(_r, ty)?;
            assert_unit_enum(e);
            let name = Self::out_name(ty);
            let cname = self.c_type_ident(ty);
            let src = self.src_ty(ty);
            let arms = e.variants.iter().map(|v| {
                let id = &v.ident;
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

        // Tagged-union output: `match` the source enum to the C union,
        // converting each arm's payload.
        if let Some(c) = self.out_tagged_union(ty, _r) {
            return Some(c);
        }

        None
    }
}

/// Structural wrapper-shape resolvers (the post-rank-machinery surface). Each
/// peels `ty`'s outermost layer and composes the inner's converter; `subs`
/// lists the immediate inner(s) it looked up.
impl CbindgenBuilder {
    /// `Option<X>` and reference (`&`/`&mut`/`&[E]`/`&str`) **input** shapes.
    pub(crate) fn in_wrappers(
        &self,
        ty: &syn::Type,
        r: &impl Conversions<()>,
    ) -> Option<ConverterImpl<()>> {
        // `Option<X>` input: a single nullable C param, NULL = `None`. The inner
        // `X` is reused wholesale (its own converter — e.g. an `&T` borrow — does
        // the non-null decode), so `Option<&ZConfig>` binds the *reference*
        // converter, never the owned one.
        if is_option(ty) {
            let inner = first_type_arg(ty)?;
            let entry = r.input_entry(&inner)?;
            let inner_wire = entry.destination.clone();
            let inner_conv = entry.function.sig.ident.clone();
            let (inner_ok, fallible): (syn::Type, bool) = match &entry.function.sig.output {
                syn::ReturnType::Type(_, t) if is_result(t) => {
                    let (ok, _e) = result_parts(t).expect("is_result ⇒ result_parts");
                    (ok, true)
                }
                syn::ReturnType::Type(_, t) => ((**t).clone(), false),
                syn::ReturnType::Default => (syn::parse_quote!(()), false),
            };
            if let Some((slot, rest)) = entry.niches.clone().carve() {
                let pred = &slot.matches;
                let name =
                    format_ident!("__cbg_in_option_{}", sanitize(&TypeKey::from_type(&inner)));
                let function: syn::ItemFn = if fallible {
                    syn::parse_quote!(
                        #[allow(non_snake_case, unused_variables, dead_code)]
                        pub(crate) unsafe fn #name(
                            v: #inner_wire,
                        ) -> ::core::result::Result<
                            ::core::option::Option<#inner_ok>,
                            ::std::string::String
                        > {
                            if #pred {
                                ::core::result::Result::Ok(::core::option::Option::None)
                            } else {
                                #inner_conv(v).map(::core::option::Option::Some)
                            }
                        }
                    )
                } else {
                    syn::parse_quote!(
                        #[allow(non_snake_case, unused_variables, dead_code)]
                        pub(crate) unsafe fn #name(
                            v: #inner_wire,
                        ) -> ::core::option::Option<#inner_ok> {
                            if #pred {
                                ::core::option::Option::None
                            } else {
                                ::core::option::Option::Some(#inner_conv(v))
                            }
                        }
                    )
                };
                return Some(ConverterImpl {
                    subs: vec![inner],
                    destination: inner_wire,
                    function,
                    pre_stages: vec![],
                    niches: rest,
                    metadata: (),
                });
            }
            let is_ptr = matches!(inner_wire, syn::Type::Ptr(_));
            let wire: syn::Type = if is_ptr {
                inner_wire.clone()
            } else {
                syn::parse_quote!(*const #inner_wire)
            };
            let read = if is_ptr { quote!(v) } else { quote!(*v) };
            let name = format_ident!("__cbg_in_option_{}", sanitize(&TypeKey::from_type(&inner)));
            let lt: TokenStream = if matches!(inner, syn::Type::Reference(_)) {
                quote!(<'a>)
            } else {
                quote!()
            };
            let function: syn::ItemFn = if fallible {
                syn::parse_quote!(
                    #[allow(non_snake_case, unused_variables, dead_code)]
                    pub(crate) unsafe fn #name #lt(
                        v: #wire,
                    ) -> ::core::result::Result<::core::option::Option<#inner_ok>, ::std::string::String> {
                        if v.is_null() {
                            return ::core::result::Result::Ok(::core::option::Option::None);
                        }
                        match #inner_conv(#read) {
                            ::core::result::Result::Ok(__x) => {
                                ::core::result::Result::Ok(::core::option::Option::Some(__x))
                            }
                            ::core::result::Result::Err(__e) => ::core::result::Result::Err(__e),
                        }
                    }
                )
            } else {
                syn::parse_quote!(
                    #[allow(non_snake_case, unused_variables, dead_code)]
                    pub(crate) unsafe fn #name #lt(
                        v: #wire,
                    ) -> ::core::option::Option<#inner_ok> {
                        if v.is_null() {
                            ::core::option::Option::None
                        } else {
                            ::core::option::Option::Some(#inner_conv(#read))
                        }
                    }
                )
            };
            return Some(ConverterImpl {
                subs: vec![inner],
                destination: wire,
                function,
                pre_stages: vec![],
                niches: Niches::empty(),
                metadata: (),
            });
        }

        let syn::Type::Reference(rf) = ty else {
            return None;
        };
        let elem = (*rf.elem).clone();
        // `&[E]` slice: marker only — the two-param (`*const E_wire`, `usize`)
        // lowering is done structurally in `emit_inputs`. A scalar `E` crosses as
        // itself (`*const E`); a declared inline-opaque by-value `E` (e.g. a
        // `repr_c_struct`) crosses as `*const E_counterpart` reinterpreted to
        // `&[E]` zero-copy. `subs` marks `E`'s input required so its mirror /
        // prerequisites are emitted.
        if rf.mutability.is_none() {
            if let syn::Type::Slice(s) = &*rf.elem {
                let e = (*s.elem).clone();
                // #170, the slice instance. The two-param lowering builds the
                // `&[E]` zero-copy from C's own block, so there is nowhere to
                // normalise the bytes: `&[bool]` would materialise every
                // element's restricted domain at once. `MaybeUninit<bool>` is
                // not a fix here — the callee wants `&[bool]`, and rebuilding
                // the block would silently drop the zero-copy contract this
                // path exists for. Rejected until a raw-wire lowering exists.
                if is_bool(&e) {
                    panic!(
                        "Cbindgen: `&[bool]` cannot cross IN from C. A `bool` slice is \
                         reinterpreted zero-copy from the caller's block, so a byte outside \
                         `{{0, 1}}` would become a Rust `bool` with no chance to normalise it \
                         (#170). Take the flags as an integer slice, or wrap them in a declared \
                         `opaque_ptr` handle."
                    );
                }
                if is_scalar(&e) {
                    let name =
                        format_ident!("__cbg_inmark_slice_{}", sanitize(&TypeKey::from_type(&e)));
                    let function: syn::ItemFn = syn::parse_quote!(
                        #[allow(non_snake_case, dead_code, unused)]
                        pub(crate) fn #name() {}
                    );
                    return Some(ConverterImpl {
                        subs: vec![e.clone()],
                        destination: syn::parse_quote!(*const #e),
                        function,
                        pre_stages: vec![],
                        niches: Niches::empty(),
                        metadata: (),
                    });
                }
                if let Some(counterpart) = self.value_opaque_ty(&e) {
                    let counterpart = counterpart.clone();
                    let name =
                        format_ident!("__cbg_inmark_slice_{}", sanitize(&TypeKey::from_type(&e)));
                    let function: syn::ItemFn = syn::parse_quote!(
                        #[allow(non_snake_case, dead_code, unused)]
                        pub(crate) fn #name() {}
                    );
                    return Some(ConverterImpl {
                        subs: vec![e],
                        destination: syn::parse_quote!(*const #counterpart),
                        function,
                        pre_stages: vec![],
                        niches: Niches::empty(),
                        metadata: (),
                    });
                }
            }
        }
        // `&str`: borrow a UTF-8 C string directly from the caller.
        if rf.mutability.is_none() && is_str(&elem) {
            let name = Self::in_name(ty);
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
                subs: vec![elem],
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
        if rf.mutability.is_some() {
            // `&mut MaybeUninit<X>` (X value-opaque): out-param into uninitialized
            // memory. Rust writes via the `MaybeUninit` (no drop of the garbage slot).
            if let Some(inner) = maybe_uninit_inner(&elem) {
                let op = self.value_opaque_ty(&inner)?.clone();
                let name = Self::in_name(ty);
                let src = self.src_ty(&inner);
                let short = type_short(&inner);
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
                    subs: vec![inner],
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
            let wire_ty: syn::Type = if self.opaque.contains_key(&TypeKey::from_type(&elem)) {
                let c_struct = self.c_type_ident(&elem);
                syn::parse_quote!(#c_struct)
            } else {
                self.value_opaque_ty(&elem)?.clone()
            };
            let name = Self::in_name(ty);
            let src = self.src_ty(&elem);
            let short = type_short(&elem);
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
                subs: vec![elem],
                destination: syn::parse_quote!(*mut #wire_ty),
                function,
                pre_stages: vec![],
                niches: Niches::empty(),
                metadata: (),
            });
        }
        // `&T` (shared borrow) of an opaque handle or value-opaque type.
        let key1 = TypeKey::from_type(&elem);
        let wire_ty: syn::Type = if self.opaque.contains_key(&key1) {
            let c_struct = self.c_type_ident(&elem);
            syn::parse_quote!(#c_struct)
        } else {
            self.value_opaque_ty(&elem)?.clone()
        };
        let name = Self::in_name(ty);
        let src = self.src_ty(&elem);
        let short = type_short(&elem);
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
            subs: vec![elem],
            destination: syn::parse_quote!(*const #wire_ty),
            function,
            pre_stages: vec![],
            niches: Niches::empty(),
            metadata: (),
        })
    }

    /// `Option<X>`/`Vec<X>`/`&T`/`Result<T,E>` **output** shapes. The composite
    /// markers (`Option`/`Vec`/`Result`) carry a `()` destination — the real
    /// lowering is structural in `emit_function_wrapper` — and exist only to
    /// resolve the entry and make the inner(s) required.
    pub(crate) fn out_wrappers(
        &self,
        ty: &syn::Type,
        r: &impl Conversions<()>,
    ) -> Option<ConverterImpl<()>> {
        // `Option<T>` / `Vec<T>` marker.
        if is_option(ty) || is_vec(ty) {
            let inner = first_type_arg(ty)?;
            r.output_entry(&inner)?;
            let kind = if is_option(ty) { "option" } else { "vec" };
            let name = format_ident!(
                "__cbg_outmark_{}_{}",
                kind,
                sanitize(&TypeKey::from_type(&inner))
            );
            let function: syn::ItemFn = syn::parse_quote!(
                #[allow(non_snake_case, dead_code, unused)]
                pub(crate) fn #name() {}
            );
            return Some(ConverterImpl {
                subs: vec![inner],
                destination: syn::parse_quote!(()),
                function,
                pre_stages: vec![],
                niches: Niches::empty(),
                metadata: (),
            });
        }
        // `Cow<'_, [T]>` marker. The actual C ABI shape is structural in
        // `lower_shape`/`encode_value`, like `Vec<T>`.
        if let Some(inner) = cow_slice_elem(ty) {
            r.output_entry(&inner)?;
            let name = format_ident!(
                "__cbg_outmark_cow_slice_{}",
                sanitize(&TypeKey::from_type(&inner))
            );
            let function: syn::ItemFn = syn::parse_quote!(
                #[allow(non_snake_case, dead_code, unused)]
                pub(crate) fn #name() {}
            );
            return Some(ConverterImpl {
                subs: vec![inner],
                destination: syn::parse_quote!(()),
                function,
                pre_stages: vec![],
                niches: Niches::empty(),
                metadata: (),
            });
        }
        // `&[E]` shared slice borrow (a callback argument): marker only — the real
        // two-component `(*const E_wire, size_t)` lowering of the closure `call`
        // param is structural in `prereq_callback_structs` / `dispatch_fn_input`.
        // `subs: [E]` forces E's output (its `payload_t` mirror / scalar) so the
        // closure wire element type exists; `destination` is unused for the slice
        // (the callback emitter reads the element wire directly).
        if let Some(elem) = self
            .value_opaque_slice_elem(ty)
            .or_else(|| scalar_slice_elem(ty))
        {
            r.output_entry(&elem)?;
            let name = format_ident!(
                "__cbg_outmark_slice_{}",
                sanitize(&TypeKey::from_type(&elem))
            );
            let function: syn::ItemFn = syn::parse_quote!(
                #[allow(non_snake_case, dead_code, unused)]
                pub(crate) fn #name() {}
            );
            return Some(ConverterImpl {
                subs: vec![elem],
                destination: syn::parse_quote!(()),
                function,
                pre_stages: vec![],
                niches: Niches::empty(),
                metadata: (),
            });
        }
        // `&T` shared borrow of an opaque/value-opaque type → non-owning `*const`.
        if let syn::Type::Reference(rf) = ty {
            if rf.mutability.is_none() {
                let elem = (*rf.elem).clone();
                let key = TypeKey::from_type(&elem);
                let wire_ty: syn::Type = if self.opaque.contains_key(&key) {
                    let c_struct = self.c_type_ident(&elem);
                    syn::parse_quote!(#c_struct)
                } else {
                    self.value_opaque_ty(&elem)?.clone()
                };
                let src = self.src_ty(&elem);
                let name = format_ident!("__cbg_out_ref_{}", sanitize(&TypeKey::from_type(&elem)));
                let function: syn::ItemFn = syn::parse_quote!(
                    #[allow(non_snake_case, dead_code, unused)]
                    pub(crate) unsafe fn #name(v: &#src) -> *const #wire_ty {
                        v as *const #src as *const #wire_ty
                    }
                );
                return Some(ConverterImpl {
                    subs: vec![elem],
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
        if is_result(ty) {
            let (ok, err) = result_parts(ty)?;
            let name = format_ident!("__cbg_result_{}", sanitize(&TypeKey::from_type(ty)));
            let function: syn::ItemFn = syn::parse_quote!(
                #[allow(non_snake_case, dead_code, unused)]
                pub(crate) fn #name() {}
            );
            return Some(ConverterImpl {
                subs: vec![ok, err],
                destination: syn::parse_quote!(()),
                function,
                pre_stages: vec![],
                niches: Niches::empty(),
                metadata: (),
            });
        }
        None
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
            .flat_map(|decl| decl.input.iter().chain(decl.output.iter()))
            .filter_map(|spec| match spec {
                ConvertSpec::PrebindgenFn(ident) => Some(ident.clone()),
                ConvertSpec::Trait { .. } => None,
            })
            .filter(|ident| !self.functions.contains_key(ident))
            .collect()
    }
    pub(crate) fn declared_types(&self) -> HashSet<TypeKey> {
        self.opaque
            .keys()
            .chain(self.data.keys())
            .chain(self.value_opaque.keys())
            .chain(self.enums.keys())
            .chain(self.tagged_unions.keys())
            .cloned()
            .collect()
    }
    pub(crate) fn ignored_types(&self) -> HashSet<TypeKey> {
        self.ignored_types.clone()
    }
}
