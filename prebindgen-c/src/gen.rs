//! The C adapter's generator: walks the declarations and writes the Rust
//! side of the C ABI.
//!
//! Every decision about how a type crosses is made here, per position:
//!
//! * [`Gen::val`] — a value slot: a data-struct field, a union payload, a
//!   by-value parameter or return of a value type.
//! * [`Gen::param`] — a function parameter (adds borrows, handles, strings,
//!   slices, callbacks).
//! * [`Gen::ret`] — a function result (adds `Option`, `Vec`, `Result`
//!   lowering to out-parameters).
//! * [`Gen::cb_arg`] — an argument handed to a C callback.

use std::{collections::HashMap, rc::Rc};

use prebindgen_tools::{
    flat::{
        flat::{Function, Param, ScalarKind, Type as FlatType, TypeKind, TypeRef},
        Emit, Flat,
    },
    function::{error_ident, result_ident},
    names, ClosureCallbacks, ClosureWriter, FieldCallbacks, FunctionCallbacks, FunctionWriter,
    Input, Output, Qualifier, ResolvedConversion, Return, RustFile, StructWriter, SumWriter, Wire,
};
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};

use crate::{
    builder::{CbindgenBuilder, TypeKind as DeclKind},
    Error,
};

type Res<T> = Result<T, Error>;

fn err<T>(msg: impl Into<String>) -> Res<T> {
    Err(Error(msg.into()))
}

/// A declared type, with its C names.
#[derive(Clone)]
struct Decl {
    kind: DeclKind,
    /// The source type's name.
    name: syn::Ident,
    /// The mirror / opaque struct the C side sees: `calculator_t`.
    c_name: syn::Ident,
    /// Its destructor: `calculator_drop`.
    drop_name: syn::Ident,
}

/// A conversion between a C wire value and a source value, as a function of
/// the value expression.
#[derive(Clone)]
struct Conv {
    f: Rc<dyn Fn(TokenStream) -> TokenStream>,
    fallible: bool,
}

impl Conv {
    fn new(fallible: bool, f: impl Fn(TokenStream) -> TokenStream + 'static) -> Self {
        Self {
            f: Rc::new(f),
            fallible,
        }
    }
    fn identity() -> Self {
        Self::new(false, |v| v)
    }
    fn apply(&self, v: impl ToTokens) -> TokenStream {
        (self.f)(v.to_token_stream())
    }
    fn then(&self, next: &Conv) -> Conv {
        let (a, b) = (self.f.clone(), next.f.clone());
        Conv::new(self.fallible || next.fallible, move |v| b(a(v)))
    }
}

/// How a value slot crosses: its wire type, both conversions, and how to
/// release what the wire owns.
#[derive(Clone)]
struct Val {
    wire: TokenStream,
    input: Option<Conv>,
    output: Option<Conv>,
    /// Statements releasing the owned content of the wire at a place
    /// expression (and leaving it releasable again).
    release: Option<Rc<dyn Fn(TokenStream) -> TokenStream>>,
}

/// How a function result leaves, plus what a failed input returns.
struct CRet {
    ret: Return,
    /// The value returned when an input fails to convert, for a function
    /// with an error slot; `None` when it has none.
    on_error: Option<TokenStream>,
}

pub(crate) struct Gen<'a> {
    b: &'a CbindgenBuilder,
    flat: &'a Flat,
    q: Qualifier<'a>,
    file: RustFile,
    types: HashMap<String, Decl>,
    conversions: HashMap<String, ResolvedConversion>,
    /// Callback closure structs by the key of their `impl Fn` type.
    callbacks: HashMap<String, syn::Ident>,
    callback_bases: HashMap<String, String>,
    needs_free: bool,
    /// The function currently being written, for its failure route.
    current: Option<FnCtx>,
}

#[derive(Clone)]
struct FnCtx {
    panic: bool,
    /// Whether the function's result has an error slot `e`.
    error_slot: bool,
    on_error: Option<TokenStream>,
    name: String,
}

pub(crate) fn generate(b: &CbindgenBuilder, flat: &Flat) -> Res<RustFile> {
    let mut g = Gen {
        b,
        flat,
        q: Qualifier::new(flat).with_default_module(b.source_module.clone()),
        file: RustFile::new(),
        types: HashMap::new(),
        conversions: HashMap::new(),
        callbacks: HashMap::new(),
        callback_bases: HashMap::new(),
        needs_free: false,
        current: None,
    };
    g.run()?;
    Ok(g.file)
}

fn c_int() -> TokenStream {
    quote!(::core::ffi::c_int)
}
fn c_char() -> TokenStream {
    quote!(::core::ffi::c_char)
}
fn c_void() -> TokenStream {
    quote!(::core::ffi::c_void)
}
fn ok(v: impl ToTokens) -> TokenStream {
    quote!(::core::result::Result::Ok(#v))
}

const ALLOW: &str = "#[allow(non_snake_case, non_camel_case_types, unused_variables, unused_mut, unused_unsafe, unused_parens, unused_braces, dead_code, clippy::all)]";

fn allow() -> TokenStream {
    ALLOW.parse().unwrap()
}

impl<'a> Gen<'a> {
    fn run(&mut self) -> Res<()> {
        prebindgen_tools::check_supported(self.flat).map_err(Error)?;
        self.prelude();
        self.index_types()?;
        for c in &self.b.conversions {
            let r = c.resolve(self.flat).map_err(Error)?;
            let key = type_name(&r.target)
                .ok_or_else(|| Error(format!("convert!: `{}` is not a named type", r.target)))?;
            self.conversions.insert(key, r);
        }
        for cb in &self.b.callbacks {
            let t = self
                .flat
                .classify(&cb.ty)
                .map_err(|e| Error(format!("callback `{}`: {e}", cb.ty.to_token_stream())))?;
            if let Some(base) = &cb.base {
                self.callback_bases
                    .insert(t.key().as_str().to_string(), base.clone());
            }
        }
        // Declared types, in declaration order.
        for d in self.b.types.clone() {
            let name = self.decl_name(&d.ty)?;
            self.emit_type(&name)?;
        }
        for cb in self.b.callbacks.clone() {
            let t = self
                .flat
                .classify(&cb.ty)
                .map_err(|e| Error(e.to_string()))?;
            self.closure_struct(&t)?;
        }
        for f in self.b.functions.clone() {
            self.function(&f.name, f.panic)?;
        }
        if self.needs_free && self.b.free_fn.is_none() {
            return err(
                "the binding hands out malloc'd memory (a string or an array); declare \
                 `.free_memory_function(..)` so C can release it",
            );
        }
        self.file.guards(self.flat);
        Ok(())
    }

    fn prelude(&mut self) {
        let allow = allow();
        self.file.push(quote! {
            extern "C" {
                fn malloc(size: usize) -> *mut ::core::ffi::c_void;
                fn free(ptr: *mut ::core::ffi::c_void);
            }
            #allow
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
            #allow
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
        });
        if let Some(f) = &self.b.free_fn {
            let f = names::ident(f);
            self.file.push(quote! {
                #[no_mangle]
                #allow
                pub unsafe extern "C" fn #f(p: *mut ::core::ffi::c_void) {
                    free(p);
                }
            });
        }
    }

    // ── declarations ────────────────────────────────────────────────────

    fn decl_name(&self, ty: &syn::Type) -> Res<String> {
        let t = self
            .flat
            .classify(ty)
            .map_err(|e| Error(format!("`{}`: {e}", ty.to_token_stream())))?;
        type_name(&t)
            .ok_or_else(|| Error(format!("`{}` is not a named type", ty.to_token_stream())))
    }

    fn index_types(&mut self) -> Res<()> {
        for d in &self.b.types {
            let name = self.decl_name(&d.ty)?;
            let base = d.base.clone().unwrap_or_else(|| names::snake(&name));
            let c_name = match &self.b.mangle_type {
                Some(f) => f(&base),
                None => base.clone(),
            };
            let drop_name = match &self.b.mangle_destructor {
                Some(f) => f(&base),
                None => format!("{base}_drop"),
            };
            if name != "String" && self.flat.declared_type(&name).is_none() {
                return err(format!("`{name}` is not a #[prebindgen] type"));
            }
            self.types.insert(
                name.clone(),
                Decl {
                    kind: d.kind.clone(),
                    name: names::ident(&name),
                    c_name: names::ident(&c_name),
                    drop_name: names::ident(&drop_name),
                },
            );
        }
        Ok(())
    }

    fn decl(&self, ty: &TypeRef) -> Option<Decl> {
        type_name(ty).and_then(|n| self.types.get(&n).cloned())
    }

    fn source_type(&self, name: &str) -> TokenStream {
        if name == "String" {
            quote!(::std::string::String)
        } else {
            self.q.path(&names::ident(name))
        }
    }

    fn emit_type(&mut self, name: &str) -> Res<()> {
        let d = self.types[name].clone();
        match d.kind.clone() {
            DeclKind::Opaque => self.opaque(&d),
            DeclKind::OpaqueError { message } => self.opaque_error(&d, &message),
            DeclKind::Enum => self.ensure_enum(&d),
            DeclKind::Union => self.ensure_union(&d),
            DeclKind::Data => self.ensure_data(&d),
            DeclKind::ReprC { assume_valid } => self.ensure_repr_c(&d, assume_valid),
        }
    }

    fn opaque(&mut self, d: &Decl) -> Res<()> {
        if !self.file.claim(format!("opaque:{}", d.name)) {
            return Ok(());
        }
        let (c, drop_name) = (&d.c_name, &d.drop_name);
        let src = self.source_type(&d.name.to_string());
        let allow = allow();
        self.file.push(quote! {
            #[repr(C)]
            #allow
            pub struct #c { _private: [u8; 0] }
            #[no_mangle]
            #allow
            pub unsafe extern "C" fn #drop_name(this_: *mut #c) {
                if !this_.is_null() {
                    drop(::std::boxed::Box::from_raw(this_ as *mut #src));
                }
            }
        });
        Ok(())
    }

    fn opaque_error(&mut self, d: &Decl, message: &syn::Ident) -> Res<()> {
        let key = format!("error:{}", d.name);
        if !self.file.claim(key) {
            return Ok(());
        }
        self.needs_free = true;
        let src = self.source_type(&d.name.to_string());
        let msg = self.q.path(message);
        let f = format_ident!("__cbg_out_{}", d.name);
        let allow = allow();
        self.file.push(quote! {
            #allow
            pub(crate) fn #f(v: #src) -> *mut ::core::ffi::c_char {
                __cbg_alloc_cstr(#msg(&v))
            }
        });
        Ok(())
    }

    fn element(&self, d: &Decl) -> Res<&'a FlatType> {
        self.flat
            .declared_type(&d.name)
            .ok_or_else(|| Error(format!("`{}` is not a #[prebindgen] type", d.name)))
    }

    fn ensure_enum(&mut self, d: &Decl) -> Res<()> {
        if !self.file.claim(format!("enum:{}", d.name)) {
            return Ok(());
        }
        let FlatType::Enum(e) = self.element(d)? else {
            return err(format!("`{}` is not a fieldless enum", d.name));
        };
        let emit = Emit::new();
        let c = &d.c_name;
        let src = self.source_type(&d.name.to_string());
        let values: Vec<TokenStream> = e
            .values
            .iter()
            .map(|v| {
                let n = &v.name;
                match emit.discriminant(v) {
                    Some(dis) => quote!(#n = #dis),
                    None => quote!(#n),
                }
            })
            .collect();
        let names: Vec<&syn::Ident> = e.values.iter().map(|v| &v.name).collect();
        let (fin, fout) = (
            format_ident!("__cbg_in_{}", d.name),
            format_ident!("__cbg_out_{}", d.name),
        );
        let int = c_int();
        let size_msg = format!("`{c}`: a #[repr(C)] enum must have the size of a C `int`");
        let align_msg = format!("`{c}`: a #[repr(C)] enum must have the alignment of a C `int`");
        let bad = format!("invalid discriminant {{}} for `{c}`");
        let allow = allow();
        self.file.push(quote! {
            #[repr(C)]
            #[derive(Copy, Clone, Debug, Eq, PartialEq)]
            #allow
            pub enum #c { #(#values),* }
            #allow
            pub(crate) unsafe fn #fin(v: ::core::mem::MaybeUninit<#c>) -> ::core::result::Result<#src, ::std::string::String> {
                const _: () = {
                    assert!(::core::mem::size_of::<#c>() == ::core::mem::size_of::<#int>(), #size_msg);
                    assert!(::core::mem::align_of::<#c>() == ::core::mem::align_of::<#int>(), #align_msg);
                };
                let __raw: #int = ::core::ptr::read(v.as_ptr() as *const #int);
                #(
                    if __raw == #c::#names as #int {
                        return ::core::result::Result::Ok(#src::#names);
                    }
                )*
                ::core::result::Result::Err(::std::format!(#bad, __raw))
            }
            #allow
            pub(crate) fn #fout(v: #src) -> #c {
                match v { #(#src::#names => #c::#names),* }
            }
        });
        Ok(())
    }

    fn ensure_union(&mut self, d: &Decl) -> Res<()> {
        if !self.file.claim(format!("union:{}", d.name)) {
            return Ok(());
        }
        let FlatType::Variant(v) = self.element(d)? else {
            return err(format!("`{}` is not a data-carrying enum", d.name));
        };
        let c = d.c_name.clone();
        let src = self.source_type(&d.name.to_string());
        let allow = allow();
        let mirror = SumWriter::new(v, &src, c.clone())
            .attr(quote!(#[repr(C)]))
            .attr(allow.clone())
            .write(&mut UnionFields(self))?;
        let n = v.alternatives.len() as i64;
        let int = c_int();
        let size_msg = format!(
            "`{c}`: a #[repr(C)] enum with payload variants must be at least as large as its C `int` discriminant"
        );
        let bad = format!("invalid tag {{}} for `{c}` (expected 0..{n})");
        let (fin, fout) = (
            format_ident!("__cbg_in_{}", d.name),
            format_ident!("__cbg_out_{}", d.name),
        );
        let def = &mirror.def;
        let in_expr = &mirror.input.expr;
        let in_body = if mirror.input.fallible {
            quote!((|| -> ::core::result::Result<#src, ::std::string::String> { ::core::result::Result::Ok(#in_expr) })())
        } else {
            ok(in_expr)
        };
        let out_expr = &mirror.output.expr;
        let check = quote! {
            const _: () = {
                assert!(::core::mem::size_of::<#c>() >= ::core::mem::size_of::<#int>(), #size_msg);
            };
        };
        self.file.push(quote! {
            #def
            #allow
            pub(crate) unsafe fn #fin(v: ::core::mem::MaybeUninit<#c>) -> ::core::result::Result<#src, ::std::string::String> {
                #check
                let __tag: #int = ::core::ptr::read(v.as_ptr() as *const #int);
                if !((__tag as i64) >= 0 && (__tag as i64) < #n) {
                    return ::core::result::Result::Err(::std::format!(#bad, __tag));
                }
                let v = v.assume_init();
                #in_body
            }
            #allow
            pub(crate) fn #fout(v: #src) -> ::core::mem::MaybeUninit<#c> {
                ::core::mem::MaybeUninit::new(#out_expr)
            }
        });
        // The typed drop, when an arm owns anything.
        let mut arms = Vec::new();
        for (alt, wires) in v.alternatives.iter().zip(&mirror.alternatives) {
            let mut releases = Vec::new();
            for (f, w) in alt.fields.iter().zip(wires) {
                let val = self.val(&f.ty)?;
                if let Some(rel) = &val.release {
                    let n = &w.name;
                    releases.push(rel(quote!((*#n))));
                }
            }
            if !releases.is_empty() {
                let aname = &alt.name;
                let binds: Vec<syn::Ident> = wires.iter().map(|w| w.name.clone()).collect();
                let pat = prebindgen_tools::Record::Alt(alt).pattern(&quote!(#c::#aname), &binds);
                arms.push(quote!(#pat => { #(#releases)* }));
            }
        }
        if !arms.is_empty() {
            let drop_name = &d.drop_name;
            self.file.push(quote! {
                #[no_mangle]
                #allow
                pub unsafe extern "C" fn #drop_name(this_: *mut ::core::mem::MaybeUninit<#c>) {
                    if this_.is_null() {
                        return;
                    }
                    #check
                    let __tag: #int = ::core::ptr::read((*this_).as_ptr() as *const #int);
                    if !((__tag as i64) >= 0 && (__tag as i64) < #n) {
                        return;
                    }
                    match (*this_).assume_init_mut() {
                        #(#arms)*
                        _ => {}
                    }
                }
            });
        }
        Ok(())
    }

    fn ensure_data(&mut self, d: &Decl) -> Res<()> {
        if !self.file.claim(format!("data:{}", d.name)) {
            return Ok(());
        }
        let FlatType::Struct(s) = self.element(d)? else {
            return err(format!("`{}` is not a struct", d.name));
        };
        let c = d.c_name.clone();
        let src = self.source_type(&d.name.to_string());
        let allow = allow();
        let mirror = StructWriter::new(s, &src, c.clone())
            .attr(quote!(#[repr(C)]))
            .attr(allow.clone())
            .write(&mut DataFields(self))?;
        let (fin, fout, frel) = (
            format_ident!("__cbg_in_{}", d.name),
            format_ident!("__cbg_out_{}", d.name),
            format_ident!("__cbg_release_{}", d.name),
        );
        let def = &mirror.def;
        let in_expr = &mirror.input.expr;
        let (in_ret, in_body) = if mirror.input.fallible {
            (
                quote!(::core::result::Result<#src, ::std::string::String>),
                quote!((|| -> ::core::result::Result<#src, ::std::string::String> { ::core::result::Result::Ok(#in_expr) })()),
            )
        } else {
            (src.clone(), in_expr.clone())
        };
        let out_expr = &mirror.output.expr;
        let mut releases = Vec::new();
        for (f, w) in s.fields.iter().zip(&mirror.wires) {
            let val = self.val(&f.ty)?;
            if let Some(rel) = &val.release {
                let n = &w.name;
                releases.push(rel(quote!(v.#n)));
            }
        }
        self.file.push(quote! {
            #def
            #allow
            pub(crate) unsafe fn #fin(v: #c) -> #in_ret { #in_body }
            #allow
            pub(crate) fn #fout(v: #src) -> #c { #out_expr }
        });
        if !releases.is_empty() {
            self.file.push(quote! {
                #allow
                pub(crate) unsafe fn #frel(v: &mut #c) { #(#releases)* }
            });
        }
        Ok(())
    }

    fn ensure_repr_c(&mut self, d: &Decl, assume_valid: bool) -> Res<()> {
        if !self.file.claim(format!("reprc:{}", d.name)) {
            return Ok(());
        }
        let FlatType::Struct(s) = self.element(d)? else {
            return err(format!("`{}` is not a struct", d.name));
        };
        let c = d.c_name.clone();
        let src = self.source_type(&d.name.to_string());
        let mut fields = Vec::new();
        for f in &s.fields {
            let Some(fname) = &f.name else {
                return err(format!("repr_c_struct `{}` must have named fields", d.name));
            };
            let wire = self.repr_c_field(&d.name, fname, &f.ty, assume_valid)?;
            fields.push(quote!(pub #fname: #wire));
        }
        let drop_name = &d.drop_name;
        let allow = allow();
        self.file.push(quote! {
            #[repr(C)]
            #allow
            pub struct #c { #(#fields),* }
            const _: () = {
                assert!(::core::mem::size_of::<#src>() == ::core::mem::size_of::<#c>(), "repr_c_struct: Rust type and C mirror differ in size");
                assert!(::core::mem::align_of::<#src>() == ::core::mem::align_of::<#c>(), "repr_c_struct: Rust type and C mirror differ in alignment");
            };
            impl ::prebindgen_c_runtime::Transmute for #c {
                type Rust = #src;
                #[inline]
                fn from_rust(value: Self::Rust) -> Self {
                    let __v = ::core::mem::ManuallyDrop::new(value);
                    unsafe { ::core::ptr::read(&*__v as *const Self::Rust as *const Self) }
                }
                #[inline]
                fn into_rust(self) -> Self::Rust {
                    let __v = ::core::mem::ManuallyDrop::new(self);
                    unsafe { ::core::ptr::read(&*__v as *const Self as *const Self::Rust) }
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
            #[no_mangle]
            #allow
            pub unsafe extern "C" fn #drop_name(this_: *mut #c) {
                if !this_.is_null() {
                    ::core::ptr::drop_in_place(<#c as ::prebindgen_c_runtime::Transmute>::as_rust_mut(&mut *this_));
                }
            }
        });
        Ok(())
    }

    /// The C spelling of a `repr_c_struct` field — the same layout as the
    /// Rust field, so the struct can be reinterpreted.
    fn repr_c_field(
        &mut self,
        owner: &syn::Ident,
        field: &syn::Ident,
        ty: &TypeRef,
        assume_valid: bool,
    ) -> Res<TokenStream> {
        let refuse = |why: &str| {
            err(format!(
                "repr_c_struct `{owner}`: field `{field}` ({ty}) {why}"
            ))
        };
        Ok(match ty.kind() {
            TypeKind::Scalar(ScalarKind::Bool) if !assume_valid => {
                return refuse(
                    "is a `bool`, whose invalid bytes a reinterpretation cannot reject; \
                     declare `.assume_c_field_validity()` or use a data_struct",
                )
            }
            TypeKind::Scalar(k) => {
                let id = names::ident(k.as_str());
                quote!(#id)
            }
            TypeKind::Boxed(inner) => match self.decl(inner) {
                Some(d) if matches!(d.kind, DeclKind::Opaque) => {
                    let c = &d.c_name;
                    return refuse(&format!(
                        "is a non-null owned pointer (`*mut {c}`), so a consumed struct cannot be \
                         left droppable; use `Option<Box<_>>`"
                    ));
                }
                _ => return refuse("is not a representable field"),
            },
            TypeKind::Optional(inner) => match inner.kind() {
                TypeKind::Boxed(b) => match self.decl(b) {
                    Some(d) if matches!(d.kind, DeclKind::Opaque) => {
                        let c = &d.c_name;
                        quote!(*mut #c)
                    }
                    _ => return refuse("is not a representable field"),
                },
                _ => return refuse("is not a representable field"),
            },
            TypeKind::Named { .. } => match self.decl(ty) {
                Some(d) => match d.kind {
                    DeclKind::ReprC { .. } => {
                        self.emit_type(&d.name.to_string())?;
                        let c = &d.c_name;
                        quote!(#c)
                    }
                    DeclKind::Enum if assume_valid => {
                        self.emit_type(&d.name.to_string())?;
                        let c = &d.c_name;
                        quote!(#c)
                    }
                    _ => return refuse("is not a representable field"),
                },
                None => return refuse("names an undeclared type"),
            },
            _ => return refuse("is not a representable field"),
        })
    }

    /// The owned pointer fields of a `repr_c_struct`, nulled after its value
    /// is moved out so the caller's later drop is a no-op.
    fn repr_c_gravestone(&self, name: &str) -> Vec<syn::Ident> {
        let Some(FlatType::Struct(s)) = self.flat.declared_type(name) else {
            return Vec::new();
        };
        s.fields
            .iter()
            .filter(|f| matches!(f.ty.kind(), TypeKind::Optional(_)))
            .filter_map(|f| f.name.clone())
            .collect()
    }

    // ── value slots ─────────────────────────────────────────────────────

    /// A value slot: a data-struct field, a union payload, a by-value
    /// parameter or result of a value type.
    fn val(&mut self, ty: &TypeRef) -> Res<Val> {
        if let Some(conv) = self.conversion(ty) {
            return self.converted_val(ty, &conv);
        }
        Ok(match ty.kind() {
            TypeKind::Scalar(ScalarKind::Bool) => Val {
                wire: quote!(::core::mem::MaybeUninit<bool>),
                input: Some(Conv::new(
                    false,
                    |v| quote!((::core::ptr::read(#v.as_ptr() as *const u8) != 0)),
                )),
                output: Some(Conv::new(
                    false,
                    |v| quote!(::core::mem::MaybeUninit::new(#v)),
                )),
                release: None,
            },
            TypeKind::Scalar(k) => {
                let id = names::ident(k.as_str());
                Val {
                    wire: quote!(#id),
                    input: Some(Conv::identity()),
                    output: Some(Conv::identity()),
                    release: None,
                }
            }
            TypeKind::String if self.decl(ty).is_none() => {
                self.needs_free = true;
                Val {
                    wire: quote!(*mut ::core::ffi::c_char),
                    input: Some(Conv::new(false, |v| {
                        quote!(if #v.is_null() {
                            ::std::string::String::new()
                        } else {
                            ::std::ffi::CStr::from_ptr(#v).to_string_lossy().into_owned()
                        })
                    })),
                    output: Some(Conv::new(false, |v| quote!(__cbg_alloc_cstr(#v)))),
                    release: Some(Rc::new(|p| {
                        quote! {
                            free(#p as *mut ::core::ffi::c_void);
                            #p = ::core::ptr::null_mut();
                        }
                    })),
                }
            }
            TypeKind::Boxed(inner) => {
                let v = self.val(inner)?;
                Val {
                    input: v
                        .input
                        .map(|c| c.then(&Conv::new(false, |v| quote!(::std::boxed::Box::new(#v))))),
                    output: v.output.map(|c| Conv::new(false, |v| quote!(*#v)).then(&c)),
                    ..v
                }
            }
            TypeKind::String | TypeKind::Named { .. } => {
                let Some(d) = self.decl(ty) else {
                    return err(format!("`{ty}` is not declared to the C adapter"));
                };
                self.emit_type(&d.name.to_string())?;
                let (fin, fout, frel) = (
                    format_ident!("__cbg_in_{}", d.name),
                    format_ident!("__cbg_out_{}", d.name),
                    format_ident!("__cbg_release_{}", d.name),
                );
                let c = d.c_name.clone();
                match d.kind {
                    DeclKind::Enum => Val {
                        wire: quote!(::core::mem::MaybeUninit<#c>),
                        input: Some(Conv::new(true, move |v| quote!(#fin(#v)?))),
                        output: Some(Conv::new(
                            false,
                            move |v| quote!(::core::mem::MaybeUninit::new(#fout(#v))),
                        )),
                        release: None,
                    },
                    DeclKind::Union => {
                        let drop_name = d.drop_name.clone();
                        let owns = self.union_owns(&d)?;
                        Val {
                            wire: quote!(::core::mem::MaybeUninit<#c>),
                            input: Some(Conv::new(true, move |v| quote!(#fin(#v)?))),
                            output: Some(Conv::new(false, move |v| quote!(#fout(#v)))),
                            release: owns.then(|| {
                                Rc::new(move |p: TokenStream| quote!(#drop_name(&mut #p);))
                                    as Rc<dyn Fn(TokenStream) -> TokenStream>
                            }),
                        }
                    }
                    DeclKind::Data => {
                        let fallible = self.data_in_fallible(&d)?;
                        let owns = self.data_owns(&d)?;
                        Val {
                            wire: quote!(#c),
                            input: Some(Conv::new(fallible, move |v| {
                                if fallible {
                                    quote!(#fin(#v)?)
                                } else {
                                    quote!(#fin(#v))
                                }
                            })),
                            output: Some(Conv::new(false, move |v| quote!(#fout(#v)))),
                            release: owns.then(|| {
                                Rc::new(move |p: TokenStream| quote!(#frel(&mut #p);))
                                    as Rc<dyn Fn(TokenStream) -> TokenStream>
                            }),
                        }
                    }
                    DeclKind::Opaque => {
                        let src = self.source_type(&d.name.to_string());
                        let drop_name = d.drop_name.clone();
                        let null_msg = format!("null {} handle passed by value", d.name);
                        let c2 = c.clone();
                        Val {
                            wire: quote!(*mut #c),
                            input: Some(Conv::new(true, move |v| {
                                quote!({
                                    let __p = #v;
                                    if __p.is_null() {
                                        return ::core::result::Result::Err(::std::string::String::from(#null_msg));
                                    }
                                    *::std::boxed::Box::from_raw(__p as *mut #src)
                                })
                            })),
                            output: Some(Conv::new(
                                false,
                                move |v| quote!(::std::boxed::Box::into_raw(::std::boxed::Box::new(#v)) as *mut #c2),
                            )),
                            release: Some(Rc::new(move |p| {
                                quote! { #drop_name(#p); #p = ::core::ptr::null_mut(); }
                            })),
                        }
                    }
                    DeclKind::ReprC { .. } => Val {
                        wire: quote!(#c),
                        input: Some(Conv::new(false, {
                            let c = c.clone();
                            move |v| quote!(<#c as ::prebindgen_c_runtime::Transmute>::into_rust(#v))
                        })),
                        output: Some(Conv::new(false, {
                            let c = c.clone();
                            move |v| quote!(<#c as ::prebindgen_c_runtime::Transmute>::from_rust(#v))
                        })),
                        release: {
                            let drop_name = d.drop_name.clone();
                            Some(Rc::new(move |p| quote!(#drop_name(&mut #p);)))
                        },
                    },
                    DeclKind::OpaqueError { .. } => Val {
                        wire: quote!(*mut ::core::ffi::c_char),
                        input: None,
                        output: Some(Conv::new(false, move |v| quote!(#fout(#v)))),
                        release: Some(Rc::new(|p| {
                            quote! {
                                free(#p as *mut ::core::ffi::c_void);
                                #p = ::core::ptr::null_mut();
                            }
                        })),
                    },
                }
            }
            _ => return err(format!("`{ty}` has no C value representation")),
        })
    }

    fn converted_val(&mut self, ty: &TypeRef, conv: &ResolvedConversion) -> Res<Val> {
        let _ = ty;
        let mut out = Val {
            wire: TokenStream::new(),
            input: None,
            output: None,
            release: None,
        };
        let q = self.q.clone();
        let target = conv.target.clone();
        if let Some(stage) = &conv.input {
            let repr = self.val(&stage.repr)?;
            out.wire = repr.wire.clone();
            out.release = repr.release.clone();
            if let Some(rin) = repr.input {
                let (stage, target, q) = (stage.clone(), target.clone(), self.q.clone());
                // The stage body is spelled once here, over a placeholder,
                // so the conversion does not hold the qualifier.
                let placeholder = format_ident!("__cbg_repr");
                let applied = stage.apply(&q, &target, &placeholder.to_token_stream());
                out.input = Some(rin.then(&Conv::new(
                    stage.fallible,
                    move |v| quote!({ let #placeholder = #v; #applied }),
                )));
            }
        }
        if let Some(stage) = &conv.output {
            let repr = self.val(&stage.repr)?;
            out.wire = repr.wire.clone();
            out.release = repr.release.clone();
            if let Some(rout) = repr.output {
                let placeholder = format_ident!("__cbg_value");
                let applied = stage.apply(&q, &target, &placeholder.to_token_stream());
                out.output = Some(
                    Conv::new(
                        stage.fallible,
                        move |v| quote!({ let #placeholder = #v; #applied }),
                    )
                    .then(&rout),
                );
            }
        }
        if out.wire.is_empty() {
            return err(format!(
                "convert!({}) declares neither direction",
                conv.target
            ));
        }
        Ok(out)
    }

    fn conversion(&self, ty: &TypeRef) -> Option<ResolvedConversion> {
        type_name(ty).and_then(|n| self.conversions.get(&n).cloned())
    }

    fn union_owns(&mut self, d: &Decl) -> Res<bool> {
        let FlatType::Variant(v) = self.element(d)? else {
            return Ok(false);
        };
        for alt in &v.alternatives {
            for f in &alt.fields {
                if self.val(&f.ty)?.release.is_some() {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }

    fn data_owns(&mut self, d: &Decl) -> Res<bool> {
        let FlatType::Struct(s) = self.element(d)? else {
            return Ok(false);
        };
        for f in &s.fields {
            if self.val(&f.ty)?.release.is_some() {
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn data_in_fallible(&mut self, d: &Decl) -> Res<bool> {
        let FlatType::Struct(s) = self.element(d)? else {
            return Ok(false);
        };
        for f in &s.fields {
            if self.val(&f.ty)?.input.is_some_and(|c| c.fallible) {
                return Ok(true);
            }
        }
        Ok(false)
    }

    // ── functions ───────────────────────────────────────────────────────

    fn function(&mut self, name: &syn::Ident, panic: bool) -> Res<()> {
        let Some(func) = self.flat.function(name) else {
            return err(format!("`{name}` is not a #[prebindgen] function"));
        };
        let func = func.clone();
        let exported = match &self.b.mangle_function {
            Some(f) => f(&name.to_string()),
            None => name.to_string(),
        };
        self.current = Some(FnCtx {
            panic,
            error_slot: false,
            on_error: None,
            name: name.to_string(),
        });
        let callee = self.q.path(name);
        let prologue = self.alias_preflight(&func)?;
        let item = FunctionWriter::new(&func, callee)
            .name(names::ident(&exported))
            .attr(quote!(#[no_mangle]))
            .attr(allow())
            .prologue(prologue)
            .write(self)?;
        self.current = None;
        self.file.push(item);
        Ok(())
    }

    /// Two parameters naming one resource, where at least one consumes it or
    /// borrows it exclusively, would free it twice or alias a `&mut`: compare
    /// the pointers before any conversion runs.
    fn alias_preflight(&mut self, func: &Function) -> Res<TokenStream> {
        #[derive(Clone, Copy, PartialEq)]
        enum Use {
            Consumed,
            Exclusive,
            Shared,
        }
        let label = |u: Use| match u {
            Use::Consumed => "consumed",
            Use::Exclusive => "exclusively borrowed",
            Use::Shared => "borrowed",
        };
        let mut handles: Vec<(&Param, String, Use)> = Vec::new();
        for p in &func.params {
            let (inner, u) = match p.ty.kind() {
                TypeKind::Ref { mutable, inner, .. } => (
                    &**inner,
                    if *mutable {
                        Use::Exclusive
                    } else {
                        Use::Shared
                    },
                ),
                _ => (&p.ty, Use::Consumed),
            };
            if let Some(d) = self.decl(inner) {
                if matches!(d.kind, DeclKind::Opaque | DeclKind::ReprC { .. }) {
                    handles.push((p, d.name.to_string(), u));
                }
            }
        }
        let mut checks = Vec::new();
        for i in 0..handles.len() {
            for j in i + 1..handles.len() {
                let (a, ta, ua) = &handles[i];
                let (b, tb, ub) = &handles[j];
                if ta != tb || (*ua == Use::Shared && *ub == Use::Shared) {
                    continue;
                }
                let msg = format!(
                    "aliasing arguments: `{}` ({}) and `{}` ({}) are the same `{ta}` — a consumed \
                     or exclusively-borrowed resource may not be named twice in one call",
                    a.name,
                    label(*ua),
                    b.name,
                    label(*ub)
                );
                let (an, bn) = (&a.name, &b.name);
                checks.push((quote!(!(#an as *const ()).is_null() && (#an as *const ()) == (#bn as *const ())), msg));
            }
        }
        if checks.is_empty() {
            return Ok(TokenStream::new());
        }
        let ctx = self.current.clone().expect("in a function");
        if !ctx.panic && !func_has_result(func) {
            return err(format!(
                "function `{}` can be handed aliased resources but has no error channel: declare \
                 it `.panic()` or return `Result`",
                ctx.name
            ));
        }
        let ret = self.c_ret(&func.ret)?;
        let fail = self.fail_fragment(&ret.ret, ret.on_error.as_ref());
        let e = error_ident();
        Ok(checks
            .into_iter()
            .map(|(cond, msg)| {
                quote! {
                    if #cond {
                        let #e = ::std::string::String::from(#msg);
                        #fail
                    }
                }
            })
            .collect())
    }

    fn fail_fragment(&self, _ret: &Return, on_error: Option<&TokenStream>) -> TokenStream {
        let ctx = self.current.as_ref().expect("in a function");
        let e = error_ident();
        let _ = ctx;
        match on_error {
            Some(default) => quote! {{
                if !e.is_null() {
                    *e = __cbg_alloc_cstr(#e);
                }
                return #default;
            }},
            _ => quote!({
                panic!("{}", #e);
            }),
        }
    }

    // ── parameters ──────────────────────────────────────────────────────

    fn param(&mut self, name: &syn::Ident, ty: &TypeRef) -> Res<Input> {
        let null = |what: &str| format!("null {what} pointer");
        match ty.kind() {
            TypeKind::Ref { mutable, inner, .. } => match inner.kind() {
                TypeKind::Str => {
                    let w = Wire::new(name.clone(), quote!(*const ::core::ffi::c_char));
                    Ok(Input::fallible(
                        vec![w],
                        quote!({
                            if #name.is_null() {
                                return ::core::result::Result::Err(::std::string::String::from("null pointer passed for str argument"));
                            }
                            match ::std::ffi::CStr::from_ptr(#name).to_str() {
                                ::core::result::Result::Ok(s) => s,
                                ::core::result::Result::Err(_) => return ::core::result::Result::Err(::std::string::String::from("invalid UTF-8 in str argument")),
                            }
                        }),
                    ))
                }
                TypeKind::Uninit(u) if *mutable => {
                    let d = self.pointee(u)?;
                    let (c, src) = (&d.c_name, self.source_type(&d.name.to_string()));
                    let msg = null(&d.name.to_string());
                    Ok(Input::fallible(
                        vec![Wire::new(name.clone(), quote!(*mut #c))],
                        quote!({
                            if #name.is_null() {
                                return ::core::result::Result::Err(::std::string::String::from(#msg));
                            }
                            &mut *(#name as *mut ::core::mem::MaybeUninit<#src>)
                        }),
                    ))
                }
                TypeKind::Slice(elem) if !*mutable => {
                    let d = self.pointee(elem)?;
                    let (c, src) = (&d.c_name, self.source_type(&d.name.to_string()));
                    let len = names::join(name, "len");
                    Ok(Input::new(
                        vec![
                            Wire::new(name.clone(), quote!(*const #c)),
                            Wire::new(len.clone(), quote!(usize)),
                        ],
                        quote!(if #name.is_null() || #len == 0 {
                            &[][..]
                        } else {
                            ::core::slice::from_raw_parts(#name as *const #src, #len)
                        }),
                    ))
                }
                _ => {
                    if let Some(d) = self.decl(inner) {
                        if matches!(d.kind, DeclKind::Opaque | DeclKind::ReprC { .. }) {
                            self.emit_type(&d.name.to_string())?;
                            let (c, src) = (&d.c_name, self.source_type(&d.name.to_string()));
                            let msg = null(&d.name.to_string());
                            let (ptr, r) = if *mutable {
                                (quote!(*mut #c), quote!(&mut *(#name as *mut #src)))
                            } else {
                                (quote!(*const #c), quote!(&*(#name as *const #src)))
                            };
                            return Ok(Input::fallible(
                                vec![Wire::new(name.clone(), ptr)],
                                quote!({
                                    if #name.is_null() {
                                        return ::core::result::Result::Err(::std::string::String::from(#msg));
                                    }
                                    #r
                                }),
                            ));
                        }
                    }
                    // A borrow of a value type: convert the value, lend it.
                    let v = self.val(inner)?;
                    let input = v
                        .input
                        .ok_or_else(|| Error(format!("`{ty}` cannot cross into Rust")))?;
                    let pass = if *mutable {
                        quote!(&mut #name)
                    } else {
                        quote!(&#name)
                    };
                    let expr = input.apply(name);
                    let w = Wire::new(name.clone(), v.wire);
                    Ok(if input.fallible {
                        Input::fallible(vec![w], expr)
                    } else {
                        Input::new(vec![w], expr)
                    }
                    .with_pass(pass))
                }
            },
            TypeKind::String if self.decl(ty).is_none() => {
                let w = Wire::new(name.clone(), quote!(*const ::core::ffi::c_char));
                Ok(Input::fallible(
                    vec![w],
                    quote!({
                        if #name.is_null() {
                            return ::core::result::Result::Err(::std::string::String::from("null pointer passed for String argument"));
                        }
                        match ::std::ffi::CStr::from_ptr(#name).to_str() {
                            ::core::result::Result::Ok(s) => s.to_owned(),
                            ::core::result::Result::Err(_) => return ::core::result::Result::Err(::std::string::String::from("invalid UTF-8 in String argument")),
                        }
                    }),
                ))
            }
            TypeKind::Callback { args } => {
                let c = self.closure_struct(ty)?;
                let closure = self.closure(name, args)?;
                Ok(Input::new(vec![Wire::new(name.clone(), c)], closure))
            }
            TypeKind::Optional(inner) => {
                if let TypeKind::Ref {
                    mutable: false,
                    inner: t,
                    ..
                } = inner.kind()
                {
                    if let Some(d) = self.decl(t) {
                        if matches!(d.kind, DeclKind::Opaque) {
                            let (c, src) = (&d.c_name, self.source_type(&d.name.to_string()));
                            return Ok(Input::new(
                                vec![Wire::new(name.clone(), quote!(*const #c))],
                                quote!(if #name.is_null() { ::core::option::Option::None } else { ::core::option::Option::Some(&*(#name as *const #src)) }),
                            ));
                        }
                    }
                }
                err(format!(
                    "parameter `{name}`: `{ty}` has no C representation"
                ))
            }
            TypeKind::Named { .. } => {
                if let Some(d) = self.decl(ty) {
                    if let DeclKind::ReprC { .. } = d.kind {
                        self.emit_type(&d.name.to_string())?;
                        let c = &d.c_name;
                        let msg = format!("null {} value passed by value", d.name);
                        let graves = self.repr_c_gravestone(&d.name.to_string());
                        return Ok(Input::fallible(
                            vec![Wire::new(name.clone(), quote!(*mut #c))],
                            quote!({
                                if #name.is_null() {
                                    return ::core::result::Result::Err(::std::string::String::from(#msg));
                                }
                                let __live = <#c as ::prebindgen_c_runtime::Transmute>::into_rust(::core::ptr::read(#name));
                                #( (*#name).#graves = ::core::ptr::null_mut(); )*
                                __live
                            }),
                        ));
                    }
                }
                self.val_param(name, ty)
            }
            _ => self.val_param(name, ty),
        }
    }

    fn pointee(&mut self, ty: &TypeRef) -> Res<Decl> {
        match self.decl(ty) {
            Some(d) if matches!(d.kind, DeclKind::ReprC { .. } | DeclKind::Opaque) => {
                self.emit_type(&d.name.to_string())?;
                Ok(d)
            }
            _ => err(format!(
                "`{ty}` must be a declared repr_c_struct or opaque_ptr here"
            )),
        }
    }

    fn val_param(&mut self, name: &syn::Ident, ty: &TypeRef) -> Res<Input> {
        let v = self.val(ty)?;
        let conv = v
            .input
            .ok_or_else(|| Error(format!("parameter `{name}`: `{ty}` cannot cross into Rust")))?;
        let w = Wire::new(name.clone(), v.wire);
        let expr = conv.apply(name);
        Ok(if conv.fallible {
            Input::fallible(vec![w], expr)
        } else {
            Input::new(vec![w], expr)
        })
    }

    // ── results ─────────────────────────────────────────────────────────

    fn c_ret(&mut self, ty: &TypeRef) -> Res<CRet> {
        let r = result_ident();
        if let TypeKind::Fallible { ok, err: e } = ty.kind() {
            let Some(ed) = self.decl(e) else {
                return err(format!(
                    "`{e}`: the error type must be declared `.opaque_error(..)`"
                ));
            };
            let DeclKind::OpaqueError { .. } = ed.kind else {
                return err(format!(
                    "`{e}`: the error type must be declared `.opaque_error(..)`"
                ));
            };
            self.emit_type(&ed.name.to_string())?;
            let fout = format_ident!("__cbg_out_{}", ed.name);
            let e_wire = Wire::new(format_ident!("e"), quote!(*mut *mut ::core::ffi::c_char));
            let set_e = quote!(if !e.is_null() { *e = #fout(__e); });
            if let TypeKind::Unit = ok.kind() {
                return Ok(CRet {
                    ret: Return {
                        ty: Some(quote!(bool)),
                        wires: vec![e_wire],
                        body: quote!(match #r {
                            ::core::result::Result::Ok(()) => true,
                            ::core::result::Result::Err(__e) => { #set_e false }
                        }),
                    },
                    on_error: Some(quote!(false)),
                });
            }
            if let Some((wire, conv)) = self.pointer_out(ok)? {
                return Ok(CRet {
                    ret: Return {
                        ty: Some(wire.clone()),
                        wires: vec![e_wire],
                        body: {
                            let v = conv.apply(quote!(__v));
                            quote!(match #r {
                                ::core::result::Result::Ok(__v) => #v,
                                ::core::result::Result::Err(__e) => { #set_e ::core::ptr::null_mut() }
                            })
                        },
                    },
                    on_error: Some(quote!(::core::ptr::null_mut())),
                });
            }
            let (wire, conv) = self.ret_value(ok)?;
            let out = Wire::new(format_ident!("out"), quote!(*mut #wire));
            let v = conv.apply(quote!(__v));
            return Ok(CRet {
                ret: Return {
                    ty: Some(quote!(bool)),
                    wires: vec![out, e_wire],
                    body: quote!(match #r {
                        ::core::result::Result::Ok(__v) => {
                            if !out.is_null() { ::core::ptr::write(out, #v); }
                            true
                        }
                        ::core::result::Result::Err(__e) => { #set_e false }
                    }),
                },
                on_error: Some(quote!(false)),
            });
        }
        Ok(CRet {
            ret: self.plain_ret(ty)?,
            on_error: None,
        })
    }

    /// A result delivered as a pointer the C side owns, if `ty` is one.
    fn pointer_out(&mut self, ty: &TypeRef) -> Res<Option<(TokenStream, Conv)>> {
        let opaque = |g: &mut Self, t: &TypeRef| -> Res<Option<Decl>> {
            match g.decl(t) {
                Some(d) if matches!(d.kind, DeclKind::Opaque) => {
                    g.emit_type(&d.name.to_string())?;
                    Ok(Some(d))
                }
                _ => Ok(None),
            }
        };
        if let TypeKind::String = ty.kind() {
            if self.decl(ty).is_none() {
                self.needs_free = true;
                return Ok(Some((
                    quote!(*mut ::core::ffi::c_char),
                    Conv::new(false, |v| quote!(__cbg_alloc_cstr(#v))),
                )));
            }
        }
        if let Some(d) = opaque(self, ty)? {
            let c = d.c_name.clone();
            return Ok(Some((
                quote!(*mut #c),
                Conv::new(
                    false,
                    move |v| quote!(::std::boxed::Box::into_raw(::std::boxed::Box::new(#v)) as *mut #c),
                ),
            )));
        }
        if let TypeKind::Optional(inner) = ty.kind() {
            let (inner, clone) = match inner.kind() {
                TypeKind::Ref {
                    mutable: false,
                    inner,
                    ..
                } => (&**inner, true),
                _ => (&**inner, false),
            };
            if let Some(d) = opaque(self, inner)? {
                let c = d.c_name.clone();
                return Ok(Some((
                    quote!(*mut #c),
                    Conv::new(false, move |v| {
                        let val = if clone {
                            quote!(::core::clone::Clone::clone(__x))
                        } else {
                            quote!(__x)
                        };
                        quote!(match #v {
                            ::core::option::Option::Some(__x) => ::std::boxed::Box::into_raw(::std::boxed::Box::new(#val)) as *mut #c,
                            ::core::option::Option::None => ::core::ptr::null_mut(),
                        })
                    }),
                )));
            }
        }
        Ok(None)
    }

    /// A result delivered by value: the wire type and the conversion into it.
    fn ret_value(&mut self, ty: &TypeRef) -> Res<(TokenStream, Conv)> {
        // A plain `bool` / enum leaves as itself: only an inbound value needs
        // the `MaybeUninit` guard.
        match ty.kind() {
            TypeKind::Scalar(ScalarKind::Bool) => return Ok((quote!(bool), Conv::identity())),
            TypeKind::Named { .. } => {
                if let Some(d) = self.decl(ty) {
                    if let DeclKind::Enum = d.kind {
                        self.emit_type(&d.name.to_string())?;
                        let c = d.c_name.clone();
                        let fout = format_ident!("__cbg_out_{}", d.name);
                        return Ok((quote!(#c), Conv::new(false, move |v| quote!(#fout(#v)))));
                    }
                }
            }
            _ => {}
        }
        if let Some((w, c)) = self.pointer_out(ty)? {
            return Ok((w, c));
        }
        let v = self.val(ty)?;
        let out = v
            .output
            .ok_or_else(|| Error(format!("`{ty}` cannot cross out of Rust")))?;
        if out.fallible {
            return err(format!(
                "`{ty}`: a fallible output conversion needs a Result return"
            ));
        }
        Ok((v.wire, out))
    }

    fn plain_ret(&mut self, ty: &TypeRef) -> Res<Return> {
        let r = result_ident();
        match ty.kind() {
            TypeKind::Unit => Ok(Return {
                ty: None,
                wires: Vec::new(),
                body: quote!(let _ = #r;),
            }),
            TypeKind::Vec(elem) => {
                self.needs_free = true;
                let (wire, conv) = self.array_elem(elem)?;
                let e = conv.apply(quote!(__e));
                Ok(Return {
                    ty: Some(quote!(*mut #wire)),
                    wires: vec![Wire::new(format_ident!("len"), quote!(*mut usize))],
                    body: quote! {
                        let __arr: ::std::vec::Vec<#wire> = #r.into_iter().map(|__e| #e).collect();
                        let (__p, __n) = __cbg_alloc_array(__arr);
                        if !len.is_null() { *len = __n; }
                        __p
                    },
                })
            }
            TypeKind::Optional(inner)
                if !matches!(inner.kind(), TypeKind::Ref { .. })
                    && self.pointer_out(ty)?.is_none() =>
            {
                if let TypeKind::Vec(elem) = inner.kind() {
                    self.needs_free = true;
                    let (wire, conv) = self.array_elem(elem)?;
                    let e = conv.apply(quote!(__e));
                    return Ok(Return {
                        ty: Some(quote!(bool)),
                        wires: vec![
                            Wire::new(format_ident!("out"), quote!(*mut *mut #wire)),
                            Wire::new(format_ident!("out_len"), quote!(*mut usize)),
                        ],
                        body: quote! {
                            match #r {
                                ::core::option::Option::Some(__v) => {
                                    let __arr: ::std::vec::Vec<#wire> = __v.into_iter().map(|__e| #e).collect();
                                    let (__p, __n) = __cbg_alloc_array(__arr);
                                    if !out.is_null() { *out = __p; }
                                    if !out_len.is_null() { *out_len = __n; }
                                    true
                                }
                                ::core::option::Option::None => false,
                            }
                        },
                    });
                }
                let (wire, conv) = self.ret_value(inner)?;
                let v = conv.apply(quote!(__v));
                Ok(Return {
                    ty: Some(quote!(bool)),
                    wires: vec![Wire::new(format_ident!("out"), quote!(*mut #wire))],
                    body: quote! {
                        match #r {
                            ::core::option::Option::Some(__v) => {
                                if !out.is_null() { ::core::ptr::write(out, #v); }
                                true
                            }
                            ::core::option::Option::None => false,
                        }
                    },
                })
            }
            _ => {
                let (wire, conv) = self.ret_value(ty)?;
                let v = conv.apply(&r);
                Ok(Return {
                    ty: Some(wire),
                    wires: Vec::new(),
                    body: v,
                })
            }
        }
    }

    fn array_elem(&mut self, elem: &TypeRef) -> Res<(TokenStream, Conv)> {
        self.ret_value(elem)
    }

    // ── callbacks ───────────────────────────────────────────────────────

    /// The closure struct for a callback type, declared once.
    fn closure_struct(&mut self, ty: &TypeRef) -> Res<syn::Ident> {
        let key = ty.key().as_str().to_string();
        if let Some(c) = self.callbacks.get(&key) {
            return Ok(c.clone());
        }
        let TypeKind::Callback { args } = ty.kind() else {
            return err(format!("`{ty}` is not an `impl Fn(..)` callback"));
        };
        let bases: Vec<String> = match self.callback_bases.get(&key) {
            Some(b) => vec![b.clone()],
            None => args
                .iter()
                .map(|a| names::snake(&names::mangle(a)))
                .collect(),
        };
        let name = match &self.b.mangle_callback {
            Some(f) => f(&bases),
            None => format!("closure_{}", bases.join("_")),
        };
        let c = names::ident(&name);
        self.callbacks.insert(key, c.clone());
        let mut wires = Vec::new();
        for (i, a) in args.iter().enumerate() {
            let out = self.cb_arg(i, a, &quote!(__x))?;
            wires.extend(out.wires.into_iter().map(|w| w.ty));
        }
        let allow = allow();
        self.file.push(quote! {
            #[repr(C)]
            #allow
            pub struct #c {
                pub context: *mut ::core::ffi::c_void,
                pub call: ::core::option::Option<unsafe extern "C" fn(#(#wires,)* *mut ::core::ffi::c_void)>,
                pub drop: ::core::option::Option<unsafe extern "C" fn(*mut ::core::ffi::c_void)>,
            }
        });
        Ok(c)
    }

    fn closure(&mut self, name: &syn::Ident, args: &[TypeRef]) -> Res<TokenStream> {
        let q = self.q.clone();
        let setup = quote! {
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
            let __call = #name.call;
            let __ctx = ::std::sync::Arc::new(__Ctx { context: #name.context, drop: #name.drop });
        };
        ClosureWriter::new(args)
            .setup(setup)
            .write(&q, &mut CbArgs(self), |outs| {
                let values = outs.iter().flat_map(|o| o.wires.iter().map(|w| &w.name));
                quote! {
                    if let ::core::option::Option::Some(__f) = __call {
                        unsafe { __f(#(#values,)* __ctx.context) }
                    }
                }
            })
    }

    /// An argument handed to a C callback.
    fn cb_arg(&mut self, index: usize, ty: &TypeRef, value: &TokenStream) -> Res<Output> {
        let n = format_ident!("__w{}", index);
        if let TypeKind::Ref {
            mutable: false,
            inner,
            ..
        } = ty.kind()
        {
            if let TypeKind::Slice(elem) = inner.kind() {
                let d = self.pointee(elem)?;
                let c = &d.c_name;
                let len = format_ident!("__w{}_len", index);
                return Ok(Output::new(
                    vec![
                        Wire::new(n, quote!(*const #c)),
                        Wire::new(len, quote!(usize)),
                    ],
                    quote!((#value.as_ptr() as *const #c, #value.len())),
                ));
            }
            if let Some(d) = self.decl(inner) {
                if matches!(d.kind, DeclKind::ReprC { .. } | DeclKind::Opaque) {
                    self.emit_type(&d.name.to_string())?;
                    let (c, src) = (&d.c_name, self.source_type(&d.name.to_string()));
                    return Ok(Output::single(
                        Wire::new(n, quote!(*const #c)),
                        quote!(#value as *const #src as *const #c),
                    ));
                }
            }
        }
        let (wire, conv) = self.ret_value(ty)?;
        Ok(Output::single(Wire::new(n, wire), conv.apply(value)))
    }
}

fn func_has_result(f: &Function) -> bool {
    matches!(f.ret.kind(), TypeKind::Fallible { .. })
}

/// The declared name a type is looked up by: `Payload` for a named type,
/// `String` for `String`.
fn type_name(ty: &TypeRef) -> Option<String> {
    match ty.kind() {
        TypeKind::Named { id, .. } => Some(id.name.clone()),
        TypeKind::String => Some("String".to_string()),
        _ => None,
    }
}

impl FunctionCallbacks for Gen<'_> {
    type Error = Error;

    fn param(&mut self, _func: &Function, param: &Param) -> Res<Input> {
        let input = Gen::param(self, &param.name, &param.ty)?;
        let ctx = self.current.as_ref().expect("in a function");
        if input.fallible && ctx.on_error.is_none() && !ctx.panic {
            return err(format!(
                "function `{}`: parameter `{}` ({}) can fail to convert (a null pointer, an \
                 invalid value) and the function has no error channel: declare it `.panic()` or \
                 return `Result`",
                ctx.name, param.name, param.ty
            ));
        }
        Ok(input)
    }

    fn ret(&mut self, func: &Function, ret: &TypeRef) -> Res<Return> {
        let r = self.c_ret(ret)?;
        let ctx = self.current.as_mut().expect("in a function");
        ctx.error_slot = r.on_error.is_some();
        ctx.on_error = r.on_error.clone();
        let _ = func;
        Ok(r.ret)
    }

    fn fail(&mut self, _func: &Function, ret: &Return) -> TokenStream {
        let ctx = self.current.clone().expect("in a function");
        self.fail_fragment(ret, ctx.on_error.as_ref())
    }
}

/// Union payload fields: one wire per field, named by position.
struct UnionFields<'g, 'a>(&'g mut Gen<'a>);

impl FieldCallbacks for UnionFields<'_, '_> {
    type Error = Error;

    fn field_in(
        &mut self,
        owner: &syn::Ident,
        field: &prebindgen_tools::flat::flat::Field,
    ) -> Res<Input> {
        let v = self.0.val(&field.ty)?;
        let name = format_ident!("__f{}", field.index);
        let conv = v.input.ok_or_else(|| {
            Error(format!(
                "`{owner}`: field `{}` cannot cross into Rust",
                field.index
            ))
        })?;
        let expr = conv.apply(&name);
        let w = Wire::new(name, v.wire);
        Ok(if conv.fallible {
            Input::fallible(vec![w], expr)
        } else {
            Input::new(vec![w], expr)
        })
    }

    fn field_out(
        &mut self,
        owner: &syn::Ident,
        field: &prebindgen_tools::flat::flat::Field,
        value: &TokenStream,
    ) -> Res<Output> {
        let v = self.0.val(&field.ty)?;
        let name = format_ident!("__f{}", field.index);
        let conv = v.output.ok_or_else(|| {
            Error(format!(
                "`{owner}`: field `{}` cannot cross out of Rust",
                field.index
            ))
        })?;
        Ok(Output::single(Wire::new(name, v.wire), conv.apply(value)))
    }
}

/// Data-struct fields: one wire per field, named like the field.
struct DataFields<'g, 'a>(&'g mut Gen<'a>);

fn field_wire_name(field: &prebindgen_tools::flat::flat::Field) -> syn::Ident {
    match &field.name {
        Some(n) => n.clone(),
        None => format_ident!("_{}", field.index),
    }
}

impl FieldCallbacks for DataFields<'_, '_> {
    type Error = Error;

    fn field_in(
        &mut self,
        owner: &syn::Ident,
        field: &prebindgen_tools::flat::flat::Field,
    ) -> Res<Input> {
        let v = self.0.val(&field.ty)?;
        let name = field_wire_name(field);
        let conv = v
            .input
            .ok_or_else(|| Error(format!("`{owner}`: field `{name}` cannot cross into Rust")))?;
        let expr = conv.apply(&name);
        let w = Wire::new(name, v.wire);
        Ok(if conv.fallible {
            Input::fallible(vec![w], expr)
        } else {
            Input::new(vec![w], expr)
        })
    }

    fn field_out(
        &mut self,
        owner: &syn::Ident,
        field: &prebindgen_tools::flat::flat::Field,
        value: &TokenStream,
    ) -> Res<Output> {
        let v = self.0.val(&field.ty)?;
        let name = field_wire_name(field);
        let conv = v.output.ok_or_else(|| {
            Error(format!(
                "`{owner}`: field `{name}` cannot cross out of Rust"
            ))
        })?;
        Ok(Output::single(Wire::new(name, v.wire), conv.apply(value)))
    }
}

struct CbArgs<'g, 'a>(&'g mut Gen<'a>);

impl ClosureCallbacks for CbArgs<'_, '_> {
    type Error = Error;

    fn arg(&mut self, index: usize, ty: &TypeRef, value: &TokenStream) -> Res<Output> {
        self.0.cb_arg(index, ty, value)
    }
}

#[allow(dead_code)]
fn c_types_used() -> [TokenStream; 3] {
    [c_int(), c_char(), c_void()]
}
