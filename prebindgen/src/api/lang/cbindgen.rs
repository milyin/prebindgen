//! `Cbindgen` — the C / cbindgen language adapter.
//!
//! A [`Prebindgen`] back-end that turns a "flat" `#[prebindgen]` library into a
//! Rust file suitable for [`cbindgen`](https://github.com/mozilla/cbindgen) to
//! parse into a C header plus a static / dynamic library.
//!
//! Items are **opt-in**: nothing is converted unless it is explicitly declared
//! with [`Cbindgen::function`] / [`Cbindgen::opaque`] / [`Cbindgen::data_struct`]
//! / [`Cbindgen::enum_`].
//!
//! ## C ABI conventions
//!
//! * **Opaque handle** (declared with [`Cbindgen::opaque`]): a Rust value whose
//!   lifecycle is owned by the C side. Represented as `#[repr(C)] struct T { _0:
//!   *mut c_void }` wrapping a `Box::into_raw` pointer. A `<name>_drop`
//!   destructor is generated per handle.
//! * **Data struct** (declared with [`Cbindgen::data_struct`]): a by-value
//!   `#[repr(C)]` struct whose fields are mapped to C-ABI wire types
//!   (`String` → `*mut c_char`). Heap-owning structs get a `<name>_drop` too.
//! * **`Result<T, E>` return**: lowered to the C out-param idiom —
//!   `bool fn(T *out, <inputs>, E *e)`. `e` may be `NULL`, in which case the
//!   error value is dropped. Returns `true` on `Ok`, `false` on `Err`.
//!
//! References to the original Rust types in generated bodies are written
//! fully-qualified against [`Cbindgen::source_module`] so the generated file can
//! define its own identically-named `#[repr(C)]` wrapper structs without
//! colliding with the source crate's types.
//!
//! This first increment hardcodes one assumption: the framework error type is
//! the source crate's `Error` struct with a single `message: String` field
//! (aliased as `__CErr` in the generated file). Generalising the error type is
//! future work.

use std::collections::{HashMap, HashSet};

use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};

use crate::api::core::niches::Niches;
use crate::api::core::prebindgen::{ConverterImpl, Prebindgen};
use crate::api::core::registry::{Registry, TypeKey};

/// Per-opaque-handle / per-data-struct configuration.
#[derive(Clone, Default)]
struct TypeCfg {
    /// Pinned C base name for the generated `<name>_drop` destructor. Defaults
    /// to `snake_case(short)` when `None`.
    c_name: Option<String>,
}

/// C / cbindgen language adapter. Build it with [`Cbindgen::new`], declare the
/// items to convert with the fluent methods, then drive it through
/// [`Registry::write_rust`](crate::core::Registry::write_rust).
#[derive(Default)]
pub struct Cbindgen {
    /// Module path the original `#[prebindgen]` items live under. Used to
    /// fully-qualify bare references to source types in generated bodies.
    source_module: Option<syn::Path>,
    /// Idents of `#[prebindgen]` functions explicitly declared for conversion.
    functions: HashSet<syn::Ident>,
    /// Opaque-handle types (`Box` + `void*` lifecycle, auto `_drop`).
    opaque: HashMap<TypeKey, TypeCfg>,
    /// By-value `#[repr(C)]` data structs.
    data: HashMap<TypeKey, TypeCfg>,
    /// Enum types.
    enums: HashMap<TypeKey, TypeCfg>,
}

impl Cbindgen {
    /// Create an adapter with no declarations (emits an empty library).
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the module path the original `#[prebindgen]` items live under
    /// (e.g. `syn::parse_quote!(zenoh_flat)`).
    pub fn source_module(mut self, p: syn::Path) -> Self {
        self.source_module = Some(p);
        self
    }

    /// Declare a `#[prebindgen]` function to convert into the C layer.
    pub fn function(mut self, ident: syn::Ident) -> Self {
        self.functions.insert(ident);
        self
    }

    /// Declare an opaque-handle type. Its C struct + `snake_case(short)_drop`
    /// destructor are generated.
    pub fn opaque(mut self, ty: syn::Type) -> Self {
        self.opaque.insert(TypeKey::from_type(&ty), TypeCfg::default());
        self
    }

    /// Like [`Self::opaque`] but pins the C base name for the destructor
    /// (e.g. `.opaque_named(syn::parse_quote!(ZKeyExpr), "z_keyexpr")` →
    /// `z_keyexpr_drop`), since naive snake_case of `ZKeyExpr` is `z_key_expr`.
    pub fn opaque_named(mut self, ty: syn::Type, c_name: &str) -> Self {
        self.opaque.insert(
            TypeKey::from_type(&ty),
            TypeCfg { c_name: Some(c_name.to_string()) },
        );
        self
    }

    /// Declare a by-value `#[repr(C)]` data struct (e.g. `Error`).
    pub fn data_struct(mut self, ty: syn::Type) -> Self {
        self.data.insert(TypeKey::from_type(&ty), TypeCfg::default());
        self
    }

    /// Like [`Self::data_struct`] but pins the C base name used for the
    /// generated `_drop` destructor of heap-owning structs.
    pub fn data_struct_named(mut self, ty: syn::Type, c_name: &str) -> Self {
        self.data.insert(
            TypeKey::from_type(&ty),
            TypeCfg { c_name: Some(c_name.to_string()) },
        );
        self
    }

    /// Declare an enum (by type) to convert. (Codegen for enums is future work.)
    pub fn enum_(mut self, ty: syn::Type) -> Self {
        self.enums.insert(TypeKey::from_type(&ty), TypeCfg::default());
        self
    }

    // ── Internal helpers ───────────────────────────────────────────────

    /// Fully-qualify a bare single-segment source type against
    /// [`Self::source_module`] (e.g. `ZKeyExpr` → `zenoh_flat::ZKeyExpr`).
    /// Anything already qualified, or with no `source_module` set, is returned
    /// unchanged.
    fn src_ty(&self, ty: &syn::Type) -> syn::Type {
        if let (Some(m), syn::Type::Path(tp)) = (&self.source_module, ty) {
            if tp.qself.is_none() && tp.path.leading_colon.is_none() && tp.path.segments.len() == 1 {
                let mut path = m.clone();
                path.segments.push(tp.path.segments[0].clone());
                return syn::Type::Path(syn::TypePath { qself: None, path });
            }
        }
        ty.clone()
    }

    /// Path to a source function (e.g. `zenoh_flat::z_keyexpr_try_from`).
    fn src_fn(&self, ident: &syn::Ident) -> syn::Path {
        match &self.source_module {
            Some(m) => {
                let mut p = m.clone();
                p.segments.push(syn::PathSegment::from(ident.clone()));
                p
            }
            None => syn::Path::from(ident.clone()),
        }
    }

    /// The framework error type (the source crate's `Error`).
    fn err_ty(&self) -> syn::Type {
        self.src_ty(&syn::parse_quote!(Error))
    }

    /// Construct a framework error value carrying `msg`.
    fn err_value(&self, msg: &str) -> syn::Expr {
        let err = self.err_ty();
        syn::parse_quote!(#err { message: ::std::string::String::from(#msg) })
    }

    fn in_name(ty: &syn::Type) -> syn::Ident {
        format_ident!("__cbg_in_{}", sanitize(&TypeKey::from_type(ty)))
    }

    fn out_name(ty: &syn::Type) -> syn::Ident {
        format_ident!("__cbg_out_{}", sanitize(&TypeKey::from_type(ty)))
    }

    /// C base name for a declared type's destructor: pinned `c_name`, else
    /// `snake_case(short)`.
    fn c_base_name(cfg: &TypeCfg, ty: &syn::Type) -> String {
        cfg.c_name
            .clone()
            .unwrap_or_else(|| snake_case(&type_short(ty)))
    }
}

impl Prebindgen for Cbindgen {
    type Metadata = ();

    fn declared_functions(&self) -> HashSet<syn::Ident> {
        self.functions.clone()
    }

    fn declared_types(&self) -> HashSet<TypeKey> {
        self.opaque
            .keys()
            .chain(self.data.keys())
            .chain(self.enums.keys())
            .cloned()
            .collect()
    }

    fn prerequisites(&self, registry: &Registry<()>) -> Vec<syn::Item> {
        let mut items: Vec<syn::Item> = Vec::new();

        // Nothing declared → emit nothing (keeps an unconfigured adapter inert).
        if self.opaque.is_empty() && self.data.is_empty() && self.functions.is_empty() {
            return items;
        }

        // Framework error alias. Built-in fallible converters return
        // `Result<_, __CErr>`.
        let err = self.err_ty();
        items.push(syn::parse_quote!(
            #[allow(dead_code)]
            pub(crate) type __CErr = #err;
        ));

        // Opaque handles: `#[repr(C)] struct T { _0: *mut c_void }` + `_drop`.
        for (key, cfg) in sorted_by_key(&self.opaque) {
            let ty = key.to_type();
            if registry.input_entry(&ty).is_none() && registry.output_entry(&ty).is_none() {
                continue;
            }
            let c_struct = format_ident!("{}", type_short(&ty));
            items.push(syn::parse_quote!(
                #[repr(C)]
                pub struct #c_struct {
                    _0: *mut ::core::ffi::c_void,
                }
            ));
            let src = self.src_ty(&ty);
            let drop_ident = format_ident!("{}_drop", Self::c_base_name(cfg, &ty));
            items.push(syn::parse_quote!(
                #[no_mangle]
                #[allow(non_snake_case, unused_variables)]
                pub unsafe extern "C" fn #drop_ident(this_: *mut #c_struct) {
                    if !this_.is_null() && !(*this_)._0.is_null() {
                        drop(::std::boxed::Box::from_raw((*this_)._0 as *mut #src));
                        (*this_)._0 = ::core::ptr::null_mut();
                    }
                }
            ));
        }

        // Data structs: `#[repr(C)]` mirror + `_drop` for heap fields.
        for (key, cfg) in sorted_by_key(&self.data) {
            let ty = key.to_type();
            if registry.input_entry(&ty).is_none() && registry.output_entry(&ty).is_none() {
                continue;
            }
            let Some(fields) = self.struct_fields(registry, &ty) else {
                continue;
            };
            let c_struct = format_ident!("{}", type_short(&ty));
            let mut field_defs: Vec<TokenStream> = Vec::new();
            let mut drop_stmts: Vec<TokenStream> = Vec::new();
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
                if is_string(fty) {
                    drop_stmts.push(quote!(
                        if !(*this_).#fname.is_null() {
                            drop(::std::ffi::CString::from_raw((*this_).#fname));
                            (*this_).#fname = ::core::ptr::null_mut();
                        }
                    ));
                }
            }
            items.push(syn::parse_quote!(
                #[repr(C)]
                pub struct #c_struct {
                    #(#field_defs,)*
                }
            ));
            if !drop_stmts.is_empty() {
                let drop_ident = format_ident!("{}_drop", Self::c_base_name(cfg, &ty));
                items.push(syn::parse_quote!(
                    #[no_mangle]
                    #[allow(non_snake_case, unused_variables)]
                    pub unsafe extern "C" fn #drop_ident(this_: *mut #c_struct) {
                        if !this_.is_null() {
                            #(#drop_stmts)*
                        }
                    }
                ));
            }
        }

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

    fn on_input_type_rank_0(&self, ty: &syn::Type, _r: &Registry<()>) -> Option<ConverterImpl<()>> {
        let key = TypeKey::from_type(ty);

        // Opaque handle, by-value consume: `*Box::from_raw(v._0)`.
        if self.opaque.contains_key(&key) {
            let name = Self::in_name(ty);
            let c_struct = format_ident!("{}", type_short(ty));
            let src = self.src_ty(ty);
            let err = self.err_value("null opaque handle passed by value");
            let function: syn::ItemFn = syn::parse_quote!(
                #[allow(non_snake_case, unused_variables, dead_code)]
                pub(crate) unsafe fn #name(v: #c_struct) -> ::core::result::Result<#src, __CErr> {
                    if v._0.is_null() {
                        return ::core::result::Result::Err(#err);
                    }
                    ::core::result::Result::Ok(*::std::boxed::Box::from_raw(v._0 as *mut #src))
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

        // Data struct: decode each field from its C wire.
        if self.data.contains_key(&key) {
            let fields = self.struct_fields(_r, ty)?;
            let name = Self::in_name(ty);
            let c_struct = format_ident!("{}", type_short(ty));
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
                pub(crate) unsafe fn #name(v: #c_struct) -> ::core::result::Result<#src, __CErr> {
                    ::core::result::Result::Ok(#src { #(#inits),* })
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

        // `String` input: `*const c_char` → owned `String`.
        if is_string(ty) {
            let name = Self::in_name(ty);
            let null_err = self.err_value("null pointer passed for String argument");
            let utf8_err = self.err_value("invalid UTF-8 in String argument");
            let function: syn::ItemFn = syn::parse_quote!(
                #[allow(non_snake_case, unused_variables, dead_code)]
                pub(crate) unsafe fn #name(
                    v: *const ::core::ffi::c_char,
                ) -> ::core::result::Result<::std::string::String, __CErr> {
                    if v.is_null() {
                        return ::core::result::Result::Err(#null_err);
                    }
                    match ::std::ffi::CStr::from_ptr(v).to_str() {
                        ::core::result::Result::Ok(s) => {
                            ::core::result::Result::Ok(s.to_owned())
                        }
                        ::core::result::Result::Err(_) => {
                            ::core::result::Result::Err(#utf8_err)
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

        None
    }

    fn on_input_type_rank_1(
        &self,
        _pat: &syn::Type,
        _t1: &syn::Type,
        _r: &Registry<()>,
    ) -> Option<ConverterImpl<()>> {
        None
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

    // ── Output direction (rust → wire) ─────────────────────────────────

    fn on_output_type_rank_0(&self, ty: &syn::Type, _r: &Registry<()>) -> Option<ConverterImpl<()>> {
        let key = TypeKey::from_type(ty);

        // Opaque handle output: `Box::into_raw` into the wire struct.
        if self.opaque.contains_key(&key) {
            let name = Self::out_name(ty);
            let c_struct = format_ident!("{}", type_short(ty));
            let src = self.src_ty(ty);
            let function: syn::ItemFn = syn::parse_quote!(
                #[allow(non_snake_case, unused_variables, dead_code)]
                pub(crate) fn #name(v: #src) -> #c_struct {
                    #c_struct {
                        _0: ::std::boxed::Box::into_raw(::std::boxed::Box::new(v))
                            as *mut ::core::ffi::c_void,
                    }
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

        // Data struct output: encode each field into its C wire.
        if self.data.contains_key(&key) {
            let fields = self.struct_fields(_r, ty)?;
            let name = Self::out_name(ty);
            let c_struct = format_ident!("{}", type_short(ty));
            let src = self.src_ty(ty);
            let mut inits: Vec<TokenStream> = Vec::new();
            for (fname, fty) in &fields {
                if is_string(fty) {
                    inits.push(quote!(#fname: ::std::ffi::CString::new(v.#fname)
                        .unwrap_or_default()
                        .into_raw()));
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

        None
    }

    fn on_output_type_rank_1(
        &self,
        _pat: &syn::Type,
        _t1: &syn::Type,
        _r: &Registry<()>,
    ) -> Option<ConverterImpl<()>> {
        None
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
        let name = format_ident!("__cbg_result_{}", sanitize(&TypeKey::from_type(pat)));
        let function: syn::ItemFn = syn::parse_quote!(
            #[allow(non_snake_case, dead_code, unused)]
            pub(crate) fn #name() {}
        );
        // Destination is the success wire (only used if some outer wrapper reads
        // it; `on_function` does not).
        let t_short = format_ident!("{}", type_short(t1));
        Some(ConverterImpl {
            destination: syn::parse_quote!(#t_short),
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

impl Cbindgen {
    /// Fields (`name`, `type`) of a declared data struct, looked up from the
    /// registry's indexed structs. `None` if the type isn't an indexed named
    /// struct.
    fn struct_fields(
        &self,
        registry: &Registry<()>,
        ty: &syn::Type,
    ) -> Option<Vec<(syn::Ident, syn::Type)>> {
        let ident = type_path_tail(ty)?;
        let (item, _) = registry.structs.get(&ident)?;
        if let syn::Fields::Named(named) = &item.fields {
            Some(
                named
                    .named
                    .iter()
                    .map(|f| (f.ident.clone().unwrap(), f.ty.clone()))
                    .collect(),
            )
        } else {
            None
        }
    }

    /// Assemble the `#[no_mangle] extern "C"` wrapper for one declared fn.
    fn emit_function_wrapper(&self, f: &syn::ItemFn, registry: &Registry<()>) -> TokenStream {
        let orig = &f.sig.ident;
        let call_path = self.src_fn(orig);

        let return_ty: syn::Type = match &f.sig.output {
            syn::ReturnType::Default => syn::parse_quote!(()),
            syn::ReturnType::Type(_, ty) => (**ty).clone(),
        };

        if let Some((ok_ty, err_ty)) = result_parts(&return_ty) {
            self.emit_fallible_wrapper(orig, &call_path, f, &ok_ty, &err_ty, registry)
        } else {
            self.emit_infallible_wrapper(orig, &call_path, f, &return_ty, registry)
        }
    }

    /// `Result<T, E>` → `bool fn(T *out, <inputs>, E *e)`.
    fn emit_fallible_wrapper(
        &self,
        orig: &syn::Ident,
        call_path: &syn::Path,
        f: &syn::ItemFn,
        ok_ty: &syn::Type,
        err_ty: &syn::Type,
        registry: &Registry<()>,
    ) -> TokenStream {
        let ok_entry = registry.output_entry(ok_ty).unwrap_or_else(|| {
            panic!(
                "Cbindgen::on_function: success type `{}` of `{}` has no output converter",
                TypeKey::from_type(ok_ty),
                orig
            )
        });
        let err_entry = registry.output_entry(err_ty).unwrap_or_else(|| {
            panic!(
                "Cbindgen::on_function: error type `{}` of `{}` has no output converter",
                TypeKey::from_type(err_ty),
                orig
            )
        });
        let ok_wire = &ok_entry.destination;
        let ok_conv = &ok_entry.function.sig.ident;
        let err_wire = &err_entry.destination;
        let err_conv = &err_entry.function.sig.ident;

        let (params, decodes, call_args) =
            self.emit_inputs(orig, f, registry, Some(err_conv));

        quote! {
            #[no_mangle]
            #[allow(non_snake_case, unused_mut, unused_variables, unused_unsafe, dead_code)]
            pub unsafe extern "C" fn #orig(
                out: *mut #ok_wire,
                #(#params,)*
                e: *mut #err_wire,
            ) -> bool {
                #(#decodes)*
                match #call_path(#(#call_args),*) {
                    ::core::result::Result::Ok(__v) => {
                        *out = #ok_conv(__v);
                        true
                    }
                    ::core::result::Result::Err(__err) => {
                        if !e.is_null() {
                            *e = #err_conv(__err);
                        }
                        false
                    }
                }
            }
        }
    }

    /// Non-`Result` return: natural shape (`void` for unit, wire by value
    /// otherwise). Input-decode failures have no error channel here (none occur
    /// among the currently-declared fns); this path is filled out in a later
    /// increment.
    fn emit_infallible_wrapper(
        &self,
        orig: &syn::Ident,
        call_path: &syn::Path,
        f: &syn::ItemFn,
        return_ty: &syn::Type,
        registry: &Registry<()>,
    ) -> TokenStream {
        let (params, decodes, call_args) = self.emit_inputs(orig, f, registry, None);
        let call = quote!(#call_path(#(#call_args),*));

        if is_unit(return_ty) {
            quote! {
                #[no_mangle]
                #[allow(non_snake_case, unused_mut, unused_variables, unused_unsafe, dead_code)]
                pub unsafe extern "C" fn #orig(#(#params),*) {
                    #(#decodes)*
                    #call;
                }
            }
        } else {
            let out_entry = registry.output_entry(return_ty).unwrap_or_else(|| {
                panic!(
                    "Cbindgen::on_function: return type `{}` of `{}` has no output converter",
                    TypeKey::from_type(return_ty),
                    orig
                )
            });
            let wire = &out_entry.destination;
            let conv = &out_entry.function.sig.ident;
            quote! {
                #[no_mangle]
                #[allow(non_snake_case, unused_mut, unused_variables, unused_unsafe, dead_code)]
                pub unsafe extern "C" fn #orig(#(#params),*) -> #wire {
                    #(#decodes)*
                    #conv(#call)
                }
            }
        }
    }

    /// Build the wire param list, per-input decode statements, and call-site
    /// argument expressions. When `err_conv` is `Some`, decode failures write
    /// `*e` and `return false`; otherwise they abort (no error channel).
    fn emit_inputs(
        &self,
        orig: &syn::Ident,
        f: &syn::ItemFn,
        registry: &Registry<()>,
        err_conv: Option<&syn::Ident>,
    ) -> (Vec<TokenStream>, Vec<TokenStream>, Vec<TokenStream>) {
        let mut params = Vec::new();
        let mut decodes = Vec::new();
        let mut call_args = Vec::new();

        for input in &f.sig.inputs {
            let syn::FnArg::Typed(pt) = input else { continue };
            let syn::Pat::Ident(pat_id) = &*pt.pat else { continue };
            let ident = &pat_id.ident;
            let arg_ty = &*pt.ty;

            let entry = registry.input_entry(arg_ty).unwrap_or_else(|| {
                panic!(
                    "Cbindgen::on_function: input type `{}` of `{}` has no input converter",
                    TypeKey::from_type(arg_ty),
                    orig
                )
            });
            let wire = &entry.destination;
            let conv = &entry.function.sig.ident;

            params.push(quote!(#ident: #wire));

            let on_err = match err_conv {
                Some(ec) => quote!(if !e.is_null() { *e = #ec(__err); } return false;),
                None => quote!(::std::process::abort();),
            };
            decodes.push(quote!(
                let #ident = match #conv(#ident) {
                    ::core::result::Result::Ok(__v) => __v,
                    ::core::result::Result::Err(__err) => { #on_err }
                };
            ));

            match arg_ty {
                syn::Type::Reference(r) if r.mutability.is_some() => {
                    call_args.push(quote!(&mut #ident))
                }
                syn::Type::Reference(_) => call_args.push(quote!(&#ident)),
                _ => call_args.push(quote!(#ident)),
            }
        }

        (params, decodes, call_args)
    }
}

// ── Free helpers ───────────────────────────────────────────────────────

/// Iterate a `TypeKey`-keyed map in deterministic (key-string) order.
fn sorted_by_key(map: &HashMap<TypeKey, TypeCfg>) -> Vec<(&TypeKey, &TypeCfg)> {
    let mut entries: Vec<(&TypeKey, &TypeCfg)> = map.iter().collect();
    entries.sort_by(|a, b| a.0.as_str().cmp(b.0.as_str()));
    entries
}

/// Turn a `TypeKey` into a valid ident fragment (non-alphanumerics → `_`).
fn sanitize(key: &TypeKey) -> String {
    key.as_str()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '_' })
        .collect()
}

/// Last path-segment ident of a type as a `String` (e.g. `ZKeyExpr`).
fn type_short(ty: &syn::Type) -> String {
    type_path_tail(ty)
        .map(|i| i.to_string())
        .unwrap_or_else(|| sanitize(&TypeKey::from_type(ty)))
}

/// Last path-segment ident of a path type.
fn type_path_tail(ty: &syn::Type) -> Option<syn::Ident> {
    if let syn::Type::Path(tp) = ty {
        tp.path.segments.last().map(|s| s.ident.clone())
    } else {
        None
    }
}

/// PascalCase → snake_case (`ZKeyExpr` → `z_key_expr`).
fn snake_case(s: &str) -> String {
    let mut out = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if i != 0 {
                out.push('_');
            }
            out.extend(c.to_lowercase());
        } else {
            out.push(c);
        }
    }
    out
}

fn is_string(ty: &syn::Type) -> bool {
    type_path_tail(ty).map(|i| i == "String").unwrap_or(false)
}

fn is_unit(ty: &syn::Type) -> bool {
    matches!(ty, syn::Type::Tuple(t) if t.elems.is_empty())
}

fn is_result(ty: &syn::Type) -> bool {
    type_path_tail(ty).map(|i| i == "Result").unwrap_or(false)
}

/// If `ty` is `Result<T, E>`, return `(T, E)`.
fn result_parts(ty: &syn::Type) -> Option<(syn::Type, syn::Type)> {
    let syn::Type::Path(tp) = ty else { return None };
    let seg = tp.path.segments.last()?;
    if seg.ident != "Result" {
        return None;
    }
    let syn::PathArguments::AngleBracketed(ab) = &seg.arguments else {
        return None;
    };
    let mut tys = ab.args.iter().filter_map(|a| match a {
        syn::GenericArgument::Type(t) => Some(t.clone()),
        _ => None,
    });
    let t = tys.next()?;
    let e = tys.next()?;
    Some((t, e))
}

/// C-ABI wire type for a struct field. `String` → `*mut c_char`; FFI-safe
/// scalars pass through. `None` for anything else (unsupported this increment).
fn c_field_wire(ty: &syn::Type) -> Option<syn::Type> {
    if is_string(ty) {
        return Some(syn::parse_quote!(*mut ::core::ffi::c_char));
    }
    let short = type_path_tail(ty)?.to_string();
    match short.as_str() {
        "bool" | "i8" | "i16" | "i32" | "i64" | "isize" | "u8" | "u16" | "u32" | "u64"
        | "usize" | "f32" | "f64" => Some(ty.clone()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SourceLocation;

    /// An adapter with no declarations writes an empty (whitespace-only) file.
    #[test]
    fn empty_adapter_writes_empty_file() {
        let dir = std::env::temp_dir().join(format!("cbindgen_scaffold_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let out = dir.join("empty.rs");

        let cbindgen = Cbindgen::new();
        let mut registry: Registry<()> = Registry::default();
        let path = registry.write_rust(&cbindgen, &out).expect("write_rust");

        let contents = std::fs::read_to_string(&path).unwrap();
        assert!(
            contents.trim().is_empty(),
            "expected empty output, got:\n{contents}"
        );
    }

    /// `z_keyexpr_try_from(s: String) -> Result<ZKeyExpr, Error>` lowers to the
    /// C out-param + bool convention, with auto-generated drops.
    #[test]
    fn keyexpr_try_from_lowering() {
        let loc = SourceLocation::default();
        let func: syn::ItemFn = syn::parse_quote!(
            pub fn z_keyexpr_try_from(s: String) -> Result<ZKeyExpr, Error> {
                unimplemented!()
            }
        );
        let error_struct: syn::ItemStruct = syn::parse_quote!(
            pub struct Error {
                pub message: String,
            }
        );

        let mut registry = Registry::<()>::from_items([
            (syn::Item::Fn(func), loc.clone()),
            (syn::Item::Struct(error_struct), loc.clone()),
        ])
        .expect("index items");

        let cbindgen = Cbindgen::new()
            .source_module(syn::parse_quote!(zenoh_flat))
            .opaque_named(syn::parse_quote!(ZKeyExpr), "z_keyexpr")
            .data_struct_named(syn::parse_quote!(Error), "z_error")
            .function(syn::parse_quote!(z_keyexpr_try_from));

        let dir = std::env::temp_dir().join(format!("cbindgen_keyexpr_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let out = dir.join("keyexpr.rs");
        let path = registry.write_rust(&cbindgen, &out).expect("write_rust");
        let src = std::fs::read_to_string(&path).unwrap();

        // Wrapper signature.
        assert!(src.contains("extern \"C\" fn z_keyexpr_try_from"), "{src}");
        assert!(src.contains("out : * mut ZKeyExpr") || src.contains("out: *mut ZKeyExpr"), "{src}");
        assert!(src.contains("-> bool"), "{src}");
        // repr(C) wrapper structs.
        assert!(src.contains("struct ZKeyExpr"), "{src}");
        assert!(src.contains("struct Error"), "{src}");
        // Auto-generated destructors with pinned names.
        assert!(src.contains("fn z_keyexpr_drop"), "{src}");
        assert!(src.contains("fn z_error_drop"), "{src}");
        // Source call is fully qualified.
        assert!(src.contains("zenoh_flat :: z_keyexpr_try_from") || src.contains("zenoh_flat::z_keyexpr_try_from"), "{src}");
    }
}
