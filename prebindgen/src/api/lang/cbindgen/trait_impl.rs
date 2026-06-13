use super::builder::callback_fn_type;
use super::*;

/// Per-category rank-0 **input** converter builders. Each returns
/// `Some(ConverterImpl)` only for the type category it claims (and `None`
/// otherwise); [`Cbindgen::on_input_type_rank_0`] chains them in priority
/// order. The categories are mutually exclusive, so the chain's fall-through
/// is equivalent to the original sequential `if … return` blocks.
impl Cbindgen {
    /// Opaque handle, by-value consume: `*Box::from_raw(v)` — fallible (null
    /// handle → message). The wire is the bare handle pointer `*mut #c_struct`.
    fn in_opaque_handle(&self, ty: &syn::Type) -> Option<ConverterImpl<()>> {
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
            destination: syn::parse_quote!(*mut #c_struct),
            function,
            pre_stages: vec![],
            niches: Niches::empty(),
            metadata: (),
        })
    }

    /// Data struct: decode each field from its C wire — infallible.
    fn in_data_struct(&self, ty: &syn::Type, r: &Registry<()>) -> Option<ConverterImpl<()>> {
        let key = TypeKey::from_type(ty);
        if !self.data.contains_key(&key) {
            return None;
        }
        let fields = self.struct_fields(r, ty)?;
        let name = Self::in_name(ty);
        let c_struct = self.c_type_ident(ty);
        let src = self.src_ty(ty);
        let mut inits: Vec<TokenStream> = Vec::new();
        for (fname, fty) in &fields {
            if is_string(fty) {
                inits.push(quote!(#fname: if v.#fname.is_null() {
                    ::std::string::String::new()
                } else {
                    ::std::ffi::CStr::from_ptr(v.#fname).to_string_lossy().into_owned()
                }));
            } else {
                inits.push(quote!(#fname: v.#fname));
            }
        }
        let function: syn::ItemFn = syn::parse_quote!(
            #[allow(non_snake_case, unused_variables, dead_code)]
            pub(crate) unsafe fn #name(v: #c_struct) -> #src {
                #src { #(#inits),* }
            }
        );
        Some(ConverterImpl {
            destination: syn::parse_quote!(#c_struct),
            function,
            pre_stages: vec![],
            niches: Niches::empty(),
            metadata: (),
        })
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
    fn in_value_opaque(&self, ty: &syn::Type) -> Option<ConverterImpl<()>> {
        let opaque = self.value_opaque_ty(ty)?.clone();
        let owned = self.opaque_kind(ty) == Some(OpaqueKind::Owned);
        let name = Self::in_name(ty);
        let src = self.src_ty(ty);
        let short = type_short(ty);
        let null_msg = format!("null {short} value passed by value");
        let writeback = owned.then(|| {
            quote!(::core::ptr::write(v, <#opaque as ::prebindgen::Gravestone>::gravestone());)
        });
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
            destination: syn::parse_quote!(*mut #opaque),
            function,
            pre_stages: vec![],
            niches: Niches::empty(),
            metadata: (),
        })
    }

    /// Enum input: `match` the C enum back to the source enum — infallible.
    fn in_enum(&self, ty: &syn::Type, r: &Registry<()>) -> Option<ConverterImpl<()>> {
        let key = TypeKey::from_type(ty);
        if !self.enums.contains_key(&key) {
            return None;
        }
        let e = enum_item(r, ty)?;
        assert_unit_variants(e);
        let name = Self::in_name(ty);
        let cname = self.c_type_ident(ty);
        let src = self.src_ty(ty);
        let arms = e.variants.iter().map(|v| {
            let id = &v.ident;
            quote!(#cname::#id => #src::#id,)
        });
        let function: syn::ItemFn = syn::parse_quote!(
            #[allow(non_snake_case, unused_variables, dead_code)]
            pub(crate) fn #name(v: #cname) -> #src {
                match v { #(#arms)* }
            }
        );
        Some(ConverterImpl {
            destination: syn::parse_quote!(#cname),
            function,
            pre_stages: vec![],
            niches: Niches::empty(),
            metadata: (),
        })
    }

    /// `String` input: `*const c_char` → owned `String` — fallible.
    fn in_string(&self, ty: &syn::Type) -> Option<ConverterImpl<()>> {
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
            destination: syn::parse_quote!(*const ::core::ffi::c_char),
            function,
            pre_stages: vec![],
            niches: Niches::empty(),
            metadata: (),
        })
    }

    /// Bare `str` never crosses the C ABI directly, but resolving `&str`
    /// inputs requires its inner node to have a filled rank-0 cell.
    fn in_str(&self, ty: &syn::Type) -> Option<ConverterImpl<()>> {
        if !is_str(ty) {
            return None;
        }
        let name = Self::in_name(ty);
        let function: syn::ItemFn = syn::parse_quote!(
            #[allow(non_snake_case, dead_code, unused_variables)]
            pub(crate) fn #name() {}
        );
        Some(ConverterImpl {
            destination: syn::parse_quote!(*const ::core::ffi::c_char),
            function,
            pre_stages: vec![],
            niches: Niches::empty(),
            metadata: (),
        })
    }

    /// FFI-safe scalar (`bool`, integers, floats): identity pass-through.
    fn in_scalar(&self, ty: &syn::Type) -> Option<ConverterImpl<()>> {
        if !is_scalar(ty) {
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
            destination: ty.clone(),
            function,
            pre_stages: vec![],
            niches: Niches::empty(),
            metadata: (),
        })
    }
}

/// Per-section [`Cbindgen::prerequisites`] emitters. Each returns the runtime-
/// support items for one concern; the trait method concatenates them in order,
/// so the emitted preamble is identical to the former single function.
impl Cbindgen {
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
            pub(crate) fn __cbg_alloc_cstr(
                s: ::std::string::String,
            ) -> *mut ::core::ffi::c_char {
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
            pub(crate) unsafe fn __cbg_alloc_array<W>(
                v: ::std::vec::Vec<W>,
            ) -> (*mut W, usize) {
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
                let wire = c_field_wire(fty).unwrap_or_else(|| {
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
                let writeback = (cfg.kind == OpaqueKind::Owned).then(|| {
                    quote!(::core::ptr::write(
                        src,
                        <#opaque as ::prebindgen::Gravestone>::gravestone(),
                    );)
                });
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

    /// Enums: `#[repr(C)]` mirror (variant idents + explicit discriminants).
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
            assert_unit_variants(e);
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
            let arg_wires: Vec<syn::Type> = args
                .iter()
                .enumerate()
                .map(|(i, a)| {
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
                        syn::parse_quote!(*mut #wire)
                    } else {
                        wire
                    }
                })
                .collect();
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

impl Prebindgen for Cbindgen {
    type Metadata = ();

    fn declared_functions(&self) -> HashSet<syn::Ident> {
        self.functions.keys().cloned().collect()
    }

    fn ignored_functions(&self) -> HashSet<syn::Ident> {
        self.ignored_functions.clone()
    }

    fn declared_types(&self) -> HashSet<TypeKey> {
        self.opaque
            .keys()
            .chain(self.data.keys())
            .chain(self.value_opaque.keys())
            .chain(self.enums.keys())
            .cloned()
            .collect()
    }

    fn ignored_types(&self) -> HashSet<TypeKey> {
        self.ignored_types.clone()
    }

    fn prerequisites(&self, registry: &Registry<()>) -> Vec<syn::Item> {
        // C-string data memory (string returns + `String` fields of data structs)
        // is malloc'd raw and freed by the single universal `free_memory_function`.
        // Array returns (`Vec<T>`) also hand out a malloc'd block freed via the
        // same function (per element through the `z_free_array` macro), so the
        // allocator/freer prelude is needed for them too. Each section's emitter
        // lives in the `impl Cbindgen` block above; order is significant.
        let produces_array = self.produces_array(registry);
        let mut items: Vec<syn::Item> = Vec::new();
        items.extend(self.prereq_alloc_free(registry, produces_array));
        items.extend(self.prereq_array_builder(produces_array));
        items.extend(self.prereq_opaque_handles(registry));
        items.extend(self.prereq_data_structs(registry));
        items.extend(self.prereq_value_opaque(registry));
        items.extend(self.prereq_enums(registry));
        items.extend(self.prereq_callback_structs(registry));
        items
    }

    // ── Item emission ──────────────────────────────────────────────────

    fn on_function(&self, f: &syn::ItemFn, registry: &Registry<()>) -> TokenStream {
        self.emit_function_wrapper(f, registry)
    }

    fn on_struct(&self, _s: &syn::ItemStruct, _registry: &Registry<()>) -> TokenStream {
        // The `#[repr(C)]` mirror + converters come from prerequisites /
        // on_output_type_rank_0; the original (non-FFI-safe) struct is dropped.
        TokenStream::new()
    }

    fn on_enum(&self, _e: &syn::ItemEnum, _registry: &Registry<()>) -> TokenStream {
        TokenStream::new()
    }

    // ── Input direction (wire → rust) ──────────────────────────────────

    fn on_input_type_rank_0(&self, ty: &syn::Type, r: &Registry<()>) -> Option<ConverterImpl<()>> {
        // Mutually-exclusive type categories, tried in priority order. See the
        // `impl Cbindgen` block above for each category's converter shape.
        self.in_opaque_handle(ty)
            .or_else(|| self.in_data_struct(ty, r))
            .or_else(|| self.in_value_opaque(ty))
            .or_else(|| self.in_enum(ty, r))
            .or_else(|| self.in_string(ty))
            .or_else(|| self.in_str(ty))
            .or_else(|| self.in_scalar(ty))
    }

    fn on_input_type_rank_1(
        &self,
        pat: &syn::Type,
        t1: &syn::Type,
        _r: &Registry<()>,
    ) -> Option<ConverterImpl<()>> {
        // `Option<T>` input: a single nullable C param, NULL = `None` (the input
        // mirror of the output null-niche rule). A pointer-wire inner (opaque
        // handle, `char*`) reuses its wire directly; a value/scalar inner is
        // boxed behind a `*const` pointer. The inner's own converter does the
        // non-null decode, so its fallibility is preserved.
        //
        // Only the `Option<_>` pattern (wildcard directly the argument) is
        // handled here, so `t1` is the full inner type and its own converter
        // (e.g. the `&T` borrow) is reused verbatim. Patterns like `Option<&_>`
        // or `Option<Vec<_>>` are rejected — the resolver also enumerates the
        // `Option<_>` shape with the concrete inner in `t1`, which is the one
        // that resolves correctly (otherwise an `Option<&ZConfig>` would bind to
        // the *owned* `ZConfig` converter, dropping the reference).
        if is_option(pat) && matches!(first_type_arg(pat), Some(syn::Type::Infer(_))) {
            let inner = _r.input_entry(t1)?;
            let inner_wire = inner.destination.clone();
            let inner_conv = inner.function.sig.ident.clone();
            // Inner Ok type + fallibility from its converter's return type.
            let (inner_ok, fallible): (syn::Type, bool) = match &inner.function.sig.output {
                syn::ReturnType::Type(_, ty) if is_result(ty) => {
                    let (ok, _e) = result_parts(ty).expect("is_result ⇒ result_parts");
                    (ok, true)
                }
                syn::ReturnType::Type(_, ty) => ((**ty).clone(), false),
                syn::ReturnType::Default => (syn::parse_quote!(()), false),
            };
            let is_ptr = matches!(inner_wire, syn::Type::Ptr(_));
            let wire: syn::Type = if is_ptr {
                inner_wire.clone()
            } else {
                syn::parse_quote!(*const #inner_wire)
            };
            // Read the inner wire value out of `v` for the non-null branch.
            let read = if is_ptr { quote!(v) } else { quote!(*v) };
            let name = format_ident!("__cbg_in_option_{}", sanitize(&TypeKey::from_type(t1)));
            // A borrow inner (`Option<&T>`) carries the `'a` of its decoded
            // reference into `inner_ok`, so the wrapper must declare it.
            let lt: TokenStream = if matches!(t1, syn::Type::Reference(_)) {
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
                destination: wire,
                function,
                pre_stages: vec![],
                niches: Niches::empty(),
                metadata: (),
            });
        }

        // `&str`: borrow a UTF-8 C string directly from the caller.
        let syn::Type::Reference(r) = pat else {
            return None;
        };
        // `&[E]` slice (scalar `E`): marker only — the two-param (`*const E`,
        // `usize`) lowering is done structurally in `emit_inputs`. `pat` is the
        // wildcard `&[_]`; `t1` is the element type.
        if r.mutability.is_none() {
            if let syn::Type::Slice(s) = &*r.elem {
                if matches!(&*s.elem, syn::Type::Infer(_)) && is_scalar(t1) {
                    let name =
                        format_ident!("__cbg_inmark_slice_{}", sanitize(&TypeKey::from_type(t1)));
                    let function: syn::ItemFn = syn::parse_quote!(
                        #[allow(non_snake_case, dead_code, unused)]
                        pub(crate) fn #name() {}
                    );
                    return Some(ConverterImpl {
                        destination: syn::parse_quote!(*const #t1),
                        function,
                        pre_stages: vec![],
                        niches: Niches::empty(),
                        metadata: (),
                    });
                }
            }
        }
        if r.mutability.is_none() && is_str(t1) {
            let name = Self::in_name(pat);
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
                        ::core::result::Result::Err(_) => {
                            ::core::result::Result::Err(
                                ::std::string::String::from("invalid UTF-8 in str argument"),
                            )
                        }
                    }
                }
            );
            return Some(ConverterImpl {
                destination: syn::parse_quote!(*const ::core::ffi::c_char),
                function,
                pre_stages: vec![],
                niches: Niches::empty(),
                metadata: (),
            });
        }

        // `&mut T` (mutable borrow) of an opaque handle: wire `*mut <C struct>`,
        // decode to `&mut <src>` borrowed from the C-owned `Box`. Fallible
        // (null checks).
        if r.mutability.is_some() {
            if !self.opaque.contains_key(&TypeKey::from_type(t1)) {
                return None;
            }
            let ref_ty: syn::Type = syn::parse_quote!(&mut #t1);
            let name = Self::in_name(&ref_ty);
            let c_struct = self.c_type_ident(t1);
            let src = self.src_ty(t1);
            let short = type_short(t1);
            let null_ptr_msg = format!("null {short} pointer");
            let function: syn::ItemFn = syn::parse_quote!(
                #[allow(non_snake_case, unused_variables, dead_code)]
                pub(crate) unsafe fn #name<'a>(
                    v: *mut #c_struct,
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
                destination: syn::parse_quote!(*mut #c_struct),
                function,
                pre_stages: vec![],
                niches: Niches::empty(),
                metadata: (),
            });
        }

        // `&T` (shared borrow) of an opaque handle or value-opaque type: wire
        // `*const <C struct>` (the pointer-target for `opaque_ptr`, the opaque
        // counterpart for `value_opaque`), decode to `&<src>` reinterpreting the
        // C-owned storage. Fallible (null checks). Other inners fall through.
        let key1 = TypeKey::from_type(t1);
        let wire_ty: syn::Type = if self.opaque.contains_key(&key1) {
            let c_struct = self.c_type_ident(t1);
            syn::parse_quote!(#c_struct)
        } else if let Some(op) = self.value_opaque_ty(t1) {
            op.clone()
        } else {
            return None;
        };
        let ref_ty: syn::Type = syn::parse_quote!(&#t1);
        let name = Self::in_name(&ref_ty);
        let src = self.src_ty(t1);
        let short = type_short(t1);
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
            destination: syn::parse_quote!(*const #wire_ty),
            function,
            pre_stages: vec![],
            niches: Niches::empty(),
            metadata: (),
        })
    }

    fn on_input_type_rank_2(
        &self,
        _pat: &syn::Type,
        _t1: &syn::Type,
        _t2: &syn::Type,
        _r: &Registry<()>,
    ) -> Option<ConverterImpl<()>> {
        None
    }

    fn on_input_type_rank_3(
        &self,
        _pat: &syn::Type,
        _t1: &syn::Type,
        _t2: &syn::Type,
        _t3: &syn::Type,
        _r: &Registry<()>,
    ) -> Option<ConverterImpl<()>> {
        None
    }

    /// `impl Fn(Args...) + Send + Sync + 'static` callback input. The C wire is a
    /// by-value closure struct (`{ void *context; call; drop }`, emitted in
    /// `prerequisites`); the converter rebuilds a Rust closure that, on each
    /// invocation, encodes its args through their **output** converters (the
    /// args travel Rust→C when the callback fires — they're owned handles the C
    /// `call` is responsible for dropping) and invokes the C function pointer.
    /// An `Arc<Ctx>` carries the `void *context` + `drop`, releasing it (once,
    /// `Send + Sync`) when the Rust closure is dropped. Only signatures declared
    /// via [`Cbindgen::callback`] are handled.
    fn dispatch_fn_input(
        &self,
        args: &[syn::Type],
        registry: &Registry<()>,
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
            destination: syn::parse_quote!(#c_struct),
            function,
            pre_stages: vec![],
            niches: Niches::empty(),
            metadata: (),
        })
    }

    // ── Output direction (rust → wire) ─────────────────────────────────

    fn on_output_type_rank_0(
        &self,
        ty: &syn::Type,
        _r: &Registry<()>,
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
                destination: syn::parse_quote!(()),
                function,
                pre_stages: vec![],
                niches: Niches::empty(),
                metadata: (),
            });
        }

        // `String` output: encode into the owning `cbg_string_t` helper so C
        // callers get an explicit destructor instead of a raw `char **`.
        if is_string(ty) {
            let name = Self::out_name(ty);
            let function: syn::ItemFn = syn::parse_quote!(
                #[allow(non_snake_case, unused_variables, dead_code)]
                pub(crate) fn #name(v: ::std::string::String) -> *mut ::core::ffi::c_char {
                    __cbg_alloc_cstr(v)
                }
            );
            return Some(ConverterImpl {
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
                destination: syn::parse_quote!(*mut #c_struct),
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
            for (fname, fty) in &fields {
                if is_string(fty) {
                    inits.push(quote!(#fname: __cbg_alloc_cstr(v.#fname)));
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
            assert_unit_variants(e);
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
                destination: syn::parse_quote!(#cname),
                function,
                pre_stages: vec![],
                niches: Niches::empty(),
                metadata: (),
            });
        }

        None
    }

    fn on_output_type_rank_1(
        &self,
        pat: &syn::Type,
        t1: &syn::Type,
        _r: &Registry<()>,
    ) -> Option<ConverterImpl<()>> {
        // Composite output layers (`Option<T>`, `Vec<T>`): the structural
        // lowering happens in `emit_function_wrapper` via `lower_value`. These
        // markers exist only so the resolver marks the composite type resolved
        // and propagates required-ness to the inner/element type (resolved here
        // first, deepest-first). The marker fn is never called.
        if is_option(pat) || is_vec(pat) {
            _r.output_entry(t1)?;
            let kind = if is_option(pat) { "option" } else { "vec" };
            let name = format_ident!(
                "__cbg_outmark_{}_{}",
                kind,
                sanitize(&TypeKey::from_type(t1))
            );
            let function: syn::ItemFn = syn::parse_quote!(
                #[allow(non_snake_case, dead_code, unused)]
                pub(crate) fn #name() {}
            );
            return Some(ConverterImpl {
                destination: syn::parse_quote!(()),
                function,
                pre_stages: vec![],
                niches: Niches::empty(),
                metadata: (),
            });
        }

        // `&T` (any shared borrow — `&'static`, `&'a`, or elided) of an opaque
        // handle → `*const <C struct>`: a const, **non-owning** pointer that
        // reinterprets the borrow with no allocation. Signals to C callers that
        // the value must NOT be freed (it is loaned from the receiver / a static).
        // Composes under `Option<&T>` (NULL niche) for nullable loaned returns.
        let syn::Type::Reference(r) = pat else {
            return None;
        };
        if r.mutability.is_some() {
            return None;
        }
        let key = TypeKey::from_type(t1);
        // Wire C type: pointer-target for `opaque_ptr`, opaque counterpart for
        // `value_opaque`. Other inners fall through.
        let wire_ty: syn::Type = if self.opaque.contains_key(&key) {
            let c_struct = self.c_type_ident(t1);
            syn::parse_quote!(#c_struct)
        } else if let Some(op) = self.value_opaque_ty(t1) {
            op.clone()
        } else {
            return None;
        };
        let src = self.src_ty(t1);
        // Name off the concrete inner `t1` (not the `&_` wildcard pattern), so
        // distinct borrowed-return types don't collide on one converter ident.
        let name = format_ident!("__cbg_out_ref_{}", sanitize(&TypeKey::from_type(t1)));
        // Elided input lifetime: the raw-pointer output carries no lifetime, so
        // this accepts a borrow of any lifetime (a `&'static` coerces in).
        let function: syn::ItemFn = syn::parse_quote!(
            #[allow(non_snake_case, dead_code, unused)]
            pub(crate) unsafe fn #name(v: &#src) -> *const #wire_ty {
                v as *const #src as *const #wire_ty
            }
        );
        Some(ConverterImpl {
            destination: syn::parse_quote!(*const #wire_ty),
            function,
            pre_stages: vec![],
            niches: Niches::empty(),
            metadata: (),
        })
    }

    fn on_output_type_rank_2(
        &self,
        pat: &syn::Type,
        t1: &syn::Type,
        _t2: &syn::Type,
        _r: &Registry<()>,
    ) -> Option<ConverterImpl<()>> {
        // `Result<T, E>` return: the resolver needs *some* converter so the
        // entry resolves and its inner T / E become required. The real lowering
        // (bool + out-param + error-param) happens in `on_function`; this marker
        // function is never called.
        if !is_result(pat) {
            return None;
        }
        let _ = t1;
        let name = format_ident!("__cbg_result_{}", sanitize(&TypeKey::from_type(pat)));
        let function: syn::ItemFn = syn::parse_quote!(
            #[allow(non_snake_case, dead_code, unused)]
            pub(crate) fn #name() {}
        );
        // Marker only — `emit_function_wrapper` does the real lowering via
        // `lower_value`. Destination is unused (the success type may itself be a
        // composite like `Option<Vec<_>>` with no single C type).
        Some(ConverterImpl {
            destination: syn::parse_quote!(()),
            function,
            pre_stages: vec![],
            niches: Niches::empty(),
            metadata: (),
        })
    }

    fn on_output_type_rank_3(
        &self,
        _pat: &syn::Type,
        _t1: &syn::Type,
        _t2: &syn::Type,
        _t3: &syn::Type,
        _r: &Registry<()>,
    ) -> Option<ConverterImpl<()>> {
        None
    }
}
