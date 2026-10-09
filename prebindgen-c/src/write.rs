//! Writing the plan: C definitions and wrappers in declaration order.

use prebindgen_flat::{
    flat::{Field, Function, ScalarKind, Type as FlatType, TypeKind, TypeRef},
    Emit,
};
use prebindgen_tools::{
    code::result_expr,
    legacy::{Input, Output},
    names, Access, Record, RustFile, Seg, Shape, Wire, WireType,
};
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};

use crate::{
    lower::{CRet, Param},
    plan::{declared_name, err, CType, Item, Kind, Plan, Res, Setting},
    wire::CWire,
    Error,
};

const ALLOW: &str = "#[allow(non_snake_case, non_camel_case_types, unused_variables, unused_mut, unused_unsafe, unused_parens, unused_braces, dead_code, clippy::all)]";

fn allow() -> TokenStream {
    ALLOW.parse().unwrap()
}

/// Write every element of the plan.
pub(crate) fn write(plan: &Plan) -> Res<RustFile> {
    let mut file = RustFile::new();
    file.push(prelude(plan));
    for item in &plan.items {
        file.push(match item {
            Item::Type(name) => type_item(plan, name)?,
            Item::Closure { ty, name } => closure_struct(plan, ty, name)?,
            Item::Function {
                func,
                exported,
                panic,
            } => function(plan, func, exported, *panic)?,
        });
    }
    file.guards(plan.flat);
    Ok(file)
}

/// The allocation helpers every binding uses, and the free function.
fn prelude(plan: &Plan) -> TokenStream {
    let allow = allow();
    let free = plan.free_fn.as_ref().map(|f| {
        quote! {
            #[no_mangle]
            #allow
            pub unsafe extern "C" fn #f(p: *mut ::core::ffi::c_void) {
                free(p);
            }
        }
    });
    quote! {
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
        #free
    }
}

// ── types ───────────────────────────────────────────────────────────────

fn type_item(plan: &Plan, name: &str) -> Res<TokenStream> {
    let t = plan.ctype(name).expect("a planned type");
    match &t.kind {
        Kind::Opaque => Ok(opaque(plan, t)),
        Kind::Error { message } => Ok(opaque_error(plan, t, message)),
        Kind::Enum => enum_mirror(plan, t),
        Kind::Union => union_mirror(plan, t),
        Kind::Data => data_mirror(plan, t),
        Kind::ReprC { assume_valid } => repr_c(plan, t, *assume_valid),
    }
}

/// A handle: an opaque struct and its destructor.
fn opaque(plan: &Plan, t: &CType) -> TokenStream {
    let (c, drop) = (&t.c, &t.drop);
    let src = plan.source(&t.rust.to_string());
    let allow = allow();
    quote! {
        #[repr(C)]
        #allow
        pub struct #c { _private: [u8; 0] }
        #[no_mangle]
        #allow
        pub unsafe extern "C" fn #drop(this_: *mut #c) {
            if !this_.is_null() {
                drop(::std::boxed::Box::from_raw(this_ as *mut #src));
            }
        }
    }
}

/// An error type: handed to C as its message.
fn opaque_error(plan: &Plan, t: &CType, message: &syn::Ident) -> TokenStream {
    let src = plan.source(&t.rust.to_string());
    let msg = plan.source(&message.to_string());
    let f = format_ident!("__cbg_out_{}", t.rust);
    let allow = allow();
    quote! {
        #allow
        pub(crate) fn #f(v: #src) -> *mut ::core::ffi::c_char {
            __cbg_alloc_cstr(#msg(&v))
        }
    }
}

/// A fieldless enum: a `#[repr(C)]` mirror, validated on the way in.
fn enum_mirror(plan: &Plan, t: &CType) -> Res<TokenStream> {
    let FlatType::Enum(e) = plan.element(&t.rust.to_string())? else {
        return err(format!("`{}` is not a fieldless enum", t.rust));
    };
    let emit = Emit::new();
    let c = &t.c;
    let src = plan.source(&t.rust.to_string());
    let values = e.values.iter().map(|v| {
        let n = &v.name;
        match emit.discriminant(v) {
            Some(d) => quote!(#n = #d),
            None => quote!(#n),
        }
    });
    let names: Vec<&syn::Ident> = e.values.iter().map(|v| &v.name).collect();
    let (fin, fout) = (
        format_ident!("__cbg_in_{}", t.rust),
        format_ident!("__cbg_out_{}", t.rust),
    );
    let size_msg = format!("`{c}`: a #[repr(C)] enum must have the size of a C `int`");
    let align_msg = format!("`{c}`: a #[repr(C)] enum must have the alignment of a C `int`");
    let bad = format!("invalid discriminant {{}} for `{c}`");
    let allow = allow();
    Ok(quote! {
        #[repr(C)]
        #[derive(Copy, Clone, Debug, Eq, PartialEq)]
        #allow
        pub enum #c { #(#values),* }
        #allow
        pub(crate) unsafe fn #fin(v: ::core::mem::MaybeUninit<#c>) -> ::core::result::Result<#src, ::std::string::String> {
            const _: () = {
                assert!(::core::mem::size_of::<#c>() == ::core::mem::size_of::<::core::ffi::c_int>(), #size_msg);
                assert!(::core::mem::align_of::<#c>() == ::core::mem::align_of::<::core::ffi::c_int>(), #align_msg);
            };
            let __raw: ::core::ffi::c_int = ::core::ptr::read(v.as_ptr() as *const ::core::ffi::c_int);
            #(
                if __raw == #c::#names as ::core::ffi::c_int {
                    return ::core::result::Result::Ok(#src::#names);
                }
            )*
            ::core::result::Result::Err(::std::format!(#bad, __raw))
        }
        #allow
        pub(crate) fn #fout(v: #src) -> #c {
            match v { #(#src::#names => #c::#names),* }
        }
    })
}

/// A struct or union field: one value slot.
struct Fields<'p, 'f> {
    plan: &'p Plan<'f>,
    owner: &'p syn::Ident,
    /// The slot's name for a field.
    wire: fn(&Field) -> syn::Ident,
}

impl Fields<'_, '_> {
    fn field_in(&mut self, field: &Field) -> Res<Input<CWire>> {
        let w = (self.wire)(field);
        self.plan
            .value_in(&field.ty, &w)
            .map_err(|e| Error(format!("`{}`: field `{w}`: {}", self.owner, e.0)))
    }

    fn field_out(&mut self, field: &Field) -> Res<Output<CWire>> {
        let w = (self.wire)(field);
        self.plan
            .value_out(&field.ty, &w)
            .map_err(|e| Error(format!("`{}`: field `{w}`: {}", self.owner, e.0)))
    }
}

fn named_wire(f: &Field) -> syn::Ident {
    match &f.name {
        Some(n) => n.clone(),
        None => format_ident!("_{}", f.index),
    }
}

fn positional_wire(f: &Field) -> syn::Ident {
    format_ident!("__f{}", f.index)
}

/// A data-carrying enum: a `#[repr(C)]` enum mirror, its tag validated on
/// the way in, and a typed drop when an arm owns memory.
fn union_mirror(plan: &Plan, t: &CType) -> Res<TokenStream> {
    let FlatType::Variant(v) = plan.element(&t.rust.to_string())? else {
        return err(format!("`{}` is not a data-carrying enum", t.rust));
    };
    let c = &t.c;
    let src = plan.source(&t.rust.to_string());
    let allow = allow();
    let mirror = sum_mirror(
        v,
        &src,
        c,
        vec![quote!(#[repr(C)]), allow.clone()],
        &mut Fields {
            plan,
            owner: &t.rust,
            wire: positional_wire,
        },
    )?;
    let n = v.alternatives.len() as i64;
    let size_msg = format!(
        "`{c}`: a #[repr(C)] enum with payload variants must be at least as large as its C `int` discriminant"
    );
    let bad = format!("invalid tag {{}} for `{c}` (expected 0..{n})");
    let (fin, fout) = (
        format_ident!("__cbg_in_{}", t.rust),
        format_ident!("__cbg_out_{}", t.rust),
    );
    let def = &mirror.def;
    let in_body = mirror.input.result();
    let out_expr = &mirror.output.expr;
    let check = quote! {
        const _: () = {
            assert!(::core::mem::size_of::<#c>() >= ::core::mem::size_of::<::core::ffi::c_int>(), #size_msg);
        };
    };
    // The tag C wrote, read before the enum is: C can write any `int`.
    let tag = |p: TokenStream| {
        quote! {
            let __tag: ::core::ffi::c_int = ::core::ptr::read(#p as *const ::core::ffi::c_int);
            let __valid = (__tag as i64) >= 0 && (__tag as i64) < #n;
        }
    };
    let tag_in = tag(quote!(v.as_ptr()));
    let tag_drop = tag(quote!((*this_).as_ptr()));
    let mut arms = Vec::new();
    for (alt, wires) in v.alternatives.iter().zip(&mirror.alternatives) {
        let mut releases = Vec::new();
        for (f, w) in alt.fields.iter().zip(wires) {
            let n = &w.name;
            if let Some(r) = plan.release(&f.ty, &quote!((*#n)))? {
                releases.push(r);
            }
        }
        if !releases.is_empty() {
            let aname = &alt.name;
            let binds: Vec<syn::Ident> = wires.iter().map(|w| w.name.clone()).collect();
            let pat = Record::Alt(alt).pattern(&quote!(#c::#aname), &binds);
            arms.push(quote!(#pat => { #(#releases)* }));
        }
    }
    let drop = (!arms.is_empty()).then(|| {
        let d = &t.drop;
        quote! {
            #[no_mangle]
            #allow
            pub unsafe extern "C" fn #d(this_: *mut ::core::mem::MaybeUninit<#c>) {
                if this_.is_null() {
                    return;
                }
                #check
                #tag_drop
                if !__valid {
                    return;
                }
                match (*this_).assume_init_mut() {
                    #(#arms)*
                    _ => {}
                }
            }
        }
    });
    Ok(quote! {
        #def
        #allow
        pub(crate) unsafe fn #fin(v: ::core::mem::MaybeUninit<#c>) -> ::core::result::Result<#src, ::std::string::String> {
            #check
            #tag_in
            if !__valid {
                return ::core::result::Result::Err(::std::format!(#bad, __tag));
            }
            let v = v.assume_init();
            #in_body
        }
        #allow
        pub(crate) fn #fout(v: #src) -> ::core::mem::MaybeUninit<#c> {
            ::core::mem::MaybeUninit::new(#out_expr)
        }
        #drop
    })
}

/// A struct crossing field by field: a mirror, both conversions, and a
/// release of the memory its fields own.
fn data_mirror(plan: &Plan, t: &CType) -> Res<TokenStream> {
    let FlatType::Struct(s) = plan.element(&t.rust.to_string())? else {
        return err(format!("`{}` is not a struct", t.rust));
    };
    let c = &t.c;
    let src = plan.source(&t.rust.to_string());
    let allow = allow();
    let mirror = struct_mirror(
        s,
        &src,
        c,
        vec![quote!(#[repr(C)]), allow.clone()],
        &mut Fields {
            plan,
            owner: &t.rust,
            wire: named_wire,
        },
    )?;
    let (fin, fout, frel) = (
        format_ident!("__cbg_in_{}", t.rust),
        format_ident!("__cbg_out_{}", t.rust),
        format_ident!("__cbg_release_{}", t.rust),
    );
    let def = &mirror.def;
    let (in_ret, in_body) = if mirror.input.fallible {
        (
            quote!(::core::result::Result<#src, ::std::string::String>),
            mirror.input.result(),
        )
    } else {
        (src.clone(), mirror.input.expr.clone())
    };
    let out_expr = &mirror.output.expr;
    let mut releases = Vec::new();
    for (f, w) in s.fields.iter().zip(&mirror.wires) {
        let n = &w.name;
        if let Some(r) = plan.release(&f.ty, &quote!(v.#n))? {
            releases.push(r);
        }
    }
    let release = (!releases.is_empty()).then(|| {
        quote! {
            #allow
            pub(crate) unsafe fn #frel(v: &mut #c) { #(#releases)* }
        }
    });
    Ok(quote! {
        #def
        #allow
        pub(crate) unsafe fn #fin(v: #c) -> #in_ret { #in_body }
        #allow
        pub(crate) fn #fout(v: #src) -> #c { #out_expr }
        #release
    })
}

/// A `#[repr(C)]` struct: a mirror with the same layout, reinterpreted.
fn repr_c(plan: &Plan, t: &CType, assume_valid: bool) -> Res<TokenStream> {
    let FlatType::Struct(s) = plan.element(&t.rust.to_string())? else {
        return err(format!("`{}` is not a struct", t.rust));
    };
    let c = &t.c;
    let src = plan.source(&t.rust.to_string());
    let mut fields = Vec::new();
    for f in &s.fields {
        let Some(fname) = &f.name else {
            return err(format!("repr_c_struct `{}` must have named fields", t.rust));
        };
        let wire = repr_c_field(plan, &t.rust, fname, &f.ty, assume_valid)?;
        fields.push(quote!(pub #fname: #wire));
    }
    let drop = &t.drop;
    let allow = allow();
    Ok(quote! {
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
        pub unsafe extern "C" fn #drop(this_: *mut #c) {
            if !this_.is_null() {
                ::core::ptr::drop_in_place(<#c as ::prebindgen_c_runtime::Transmute>::as_rust_mut(&mut *this_));
            }
        }
    })
}

/// The C spelling of a `repr_c_struct` field — the same layout as the Rust
/// field, so the struct can be reinterpreted.
fn repr_c_field(
    plan: &Plan,
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
    let opaque = |t: &TypeRef| match plan.shape(t) {
        Ok(Shape::Declared {
            ty: declared_ty,
            declaration:
                Setting::Type(CType {
                    kind: Kind::Opaque,
                    c,
                    ..
                }),
            ..
        }) if Access::of(declared_ty) == Access::Owned => Some(c.clone()),
        _ => None,
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
        TypeKind::Boxed(inner) => match opaque(inner) {
            Some(c) => {
                return refuse(&format!(
                    "is a non-null owned pointer (`*mut {c}`), so a consumed struct cannot be \
                     left droppable; use `Option<Box<_>>`"
                ))
            }
            None => return refuse("is not a representable field"),
        },
        TypeKind::Optional(inner) => match inner.kind() {
            TypeKind::Boxed(b) => match opaque(b) {
                Some(c) => quote!(*mut #c),
                None => return refuse("is not a representable field"),
            },
            _ => return refuse("is not a representable field"),
        },
        TypeKind::Named { .. } => match plan.shape(ty)? {
            Shape::Declared {
                ty: declared_ty,
                declaration: Setting::Type(t),
                ..
            } if Access::of(declared_ty) == Access::Owned => match t.kind {
                Kind::ReprC { .. } => t.c.to_token_stream(),
                Kind::Enum if assume_valid => t.c.to_token_stream(),
                _ => return refuse("is not a representable field"),
            },
            Shape::Undeclared(_) => return refuse("names an undeclared type"),
            _ => return refuse("is not a representable field"),
        },
        _ => return refuse("is not a representable field"),
    })
}

/// A callback's closure struct: `{ context, call, drop }`.
fn closure_struct(plan: &Plan, ty: &TypeRef, name: &syn::Ident) -> Res<TokenStream> {
    let TypeKind::Callback { args } = ty.kind() else {
        return err(format!("`{ty}` is not an `impl Fn(..)` callback"));
    };
    let mut wires = Vec::new();
    for (i, a) in args.iter().enumerate() {
        let out = plan.callback_arg(i, a)?;
        wires.extend(out.wires().into_iter().map(|w| w.ty.rust()));
    }
    let allow = allow();
    Ok(quote! {
        #[repr(C)]
        #allow
        pub struct #name {
            pub context: *mut ::core::ffi::c_void,
            pub call: ::core::option::Option<unsafe extern "C" fn(#(#wires,)* *mut ::core::ffi::c_void)>,
            pub drop: ::core::option::Option<unsafe extern "C" fn(*mut ::core::ffi::c_void)>,
        }
    })
}

// ── functions ───────────────────────────────────────────────────────────

/// An exported function.
fn function(plan: &Plan, func: &Function, exported: &syn::Ident, panic: bool) -> Res<TokenStream> {
    let ret = plan.ret(&func.ret)?;
    let fail = fail(&ret);
    let prologue = alias_preflight(plan, func, &fail)?;
    if !prologue.is_empty() && !panic && ret.on_error.is_none() {
        return err(format!(
            "function `{}` can be handed aliased resources but has no error channel: declare \
             it `.panic()` or return `Result`",
            func.name
        ));
    }
    let callee = &func.name;
    let mut params = Vec::new();
    let mut stmts = Vec::new();
    let mut args = Vec::new();
    for p in &func.params {
        let name = &p.name;
        let Param { input, pass } = plan.param(name, &p.ty)?;
        if input.fallible && ret.on_error.is_none() && !panic {
            return err(format!(
                "function `{}`: parameter `{name}` ({}) can fail to convert (a null pointer, an invalid value) and the function has no error channel: declare it `.panic()` or return `Result`",
                func.name, p.ty
            ));
        }
        params.extend(input.wires().into_iter().map(Wire::decl));
        if input.fallible {
            let r = input.result();
            stmts.push(quote!(let #name = match #r {
                ::core::result::Result::Ok(__v) => __v,
                ::core::result::Result::Err(__err) => { #fail }
            };));
        } else {
            let e = &input.expr;
            stmts.push(quote!(let #name = #e;));
        }
        args.push(pass.unwrap_or_else(|| quote!(#name)));
    }
    params.extend(ret.ret.wires.iter().map(Wire::decl));
    let body = &ret.ret.body;
    let ret_ty = ret.ret.ty.as_ref().map(|t| {
        let t = t.rust();
        quote!(-> #t)
    });
    let allow = allow();
    Ok(quote! {
        #[no_mangle]
        #allow
        pub unsafe extern "C" fn #exported(#(#params),*) #ret_ty {
            #prologue
            #(#stmts)*
            let __result = #callee(#(#args),*);
            #body
        }
    })
}

/// What a failed input does: report through the error slot and return the
/// failure value, or abort.
fn fail(ret: &CRet) -> TokenStream {
    let e = format_ident!("__err");
    match &ret.on_error {
        Some(default) => quote! {{
            if !e.is_null() {
                *e = __cbg_alloc_cstr(#e);
            }
            return #default;
        }},
        None => quote!({
            panic!("{}", #e);
        }),
    }
}

/// Two parameters naming one resource, where at least one consumes it or
/// borrows it exclusively, would free it twice or alias a `&mut`: compare the
/// pointers before any conversion runs.
fn alias_preflight(plan: &Plan, func: &Function, fail: &TokenStream) -> Res<TokenStream> {
    let label = |a: Access| match a {
        Access::Owned => "consumed",
        Access::Exclusive => "exclusively borrowed",
        Access::Shared => "borrowed",
    };
    let mut handles = Vec::new();
    for p in &func.params {
        if let Shape::Declared {
            ty,
            declaration:
                Setting::Type(CType {
                    kind: Kind::Opaque | Kind::ReprC { .. },
                    ..
                }),
        } = plan.shape(&p.ty)?
        {
            handles.push((&p.name, declared_name(ty), Access::of(ty)));
        }
    }
    let e = format_ident!("__err");
    let mut checks = Vec::new();
    for (i, (a, ta, ua)) in handles.iter().enumerate() {
        for (b, tb, ub) in &handles[i + 1..] {
            if ta != tb || (*ua == Access::Shared && *ub == Access::Shared) {
                continue;
            }
            let msg = format!(
                "aliasing arguments: `{a}` ({}) and `{b}` ({}) are the same `{ta}` — a consumed \
                 or exclusively-borrowed resource may not be named twice in one call",
                label(*ua),
                label(*ub)
            );
            checks.push(quote! {
                if !(#a as *const ()).is_null() && (#a as *const ()) == (#b as *const ()) {
                    let #e = ::std::string::String::from(#msg);
                    #fail
                }
            });
        }
    }
    Ok(checks.into_iter().collect())
}

fn struct_mirror(
    source: &prebindgen_flat::flat::Struct,
    source_path: &TokenStream,
    name: &syn::Ident,
    attrs: Vec<TokenStream>,
    cb: &mut Fields<'_, '_>,
) -> Res<StructMirror> {
    let record = Record::Struct(source);
    let mut ins = Vec::new();
    let mut outs = Vec::new();
    let binds = record.binds();
    for f in &source.fields {
        ins.push(cb.field_in(f)?);
        outs.push(cb.field_out(f)?);
    }

    let wires: Vec<Wire<CWire>> = ins
        .iter()
        .flat_map(|i| i.wires().into_iter().cloned())
        .collect();
    let decls = wires.iter().map(|w| {
        let d = w.decl();
        quote!(pub #d)
    });

    let def = quote! {
        #(#attrs)*
        pub struct #name { #(#decls),* }
    };
    let wire_names: Vec<&syn::Ident> = wires.iter().map(|w| &w.name).collect();
    let head = source_path;
    let rebuilt = Input::record(source, ins);
    let input = Code {
        expr: {
            let e = &rebuilt.expr;
            quote!({ let #name { #(#wire_names),* } = v; #e })
        },
        fallible: rebuilt.fallible,
    };
    let pat = record.pattern(head, &binds);
    let fallible = outs.iter().any(|o| o.fallible);
    let out_binds: Vec<TokenStream> = outs.iter().zip(&binds).map(|(o, b)| o.bind(b)).collect();
    let output = Code {
        expr: quote!({ let #pat = v; #(#out_binds)* #name { #(#wire_names),* } }),
        fallible,
    };
    Ok(StructMirror {
        def,
        wires,
        input,
        output,
    })
}

fn sum_mirror(
    source: &prebindgen_flat::flat::Variant,
    source_path: &TokenStream,
    name: &syn::Ident,
    attrs: Vec<TokenStream>,
    cb: &mut Fields<'_, '_>,
) -> Res<SumMirror> {
    let src = source_path;
    let mut variants = Vec::new();
    let mut alternatives = Vec::new();
    let mut in_arms = Vec::new();
    let mut out_arms = Vec::new();
    let mut in_fallible = false;
    let mut out_fallible = false;
    for alt in &source.alternatives {
        let record = Record::Alt(alt);
        let aname = &alt.name;
        let binds = record.binds();
        let mut ins = Vec::new();
        let mut outs = Vec::new();
        for f in &alt.fields {
            ins.push(cb.field_in(f)?);
            outs.push(cb.field_out(f)?);
        }
        // A mirror alternative has one mirror field per source field; a
        // field that needs several wires is kept whole as a tuple would
        // lose the per-field names C sees, so it is refused here by
        // construction: the adapter hands one wire per field.
        let wires: Vec<Wire<CWire>> = ins
            .iter()
            .flat_map(|i| i.wires().into_iter().cloned())
            .collect();
        let mirror_head = quote!(#name::#aname);
        let source_head = quote!(#src::#aname);
        // Declaration: same delimiters as the source, wire types in place.
        let tys: Vec<TokenStream> = wires.iter().map(|w| w.ty.rust()).collect();
        let decl = record.construct(&quote!(#aname), &tys);
        variants.push(decl);
        // In: match the mirror alternative, binding its wires by name.
        let wire_binds: Vec<syn::Ident> = wires.iter().map(|w| w.name.clone()).collect();
        let mirror_pat = record.pattern(&mirror_head, &wire_binds);
        let parts = alt.fields.iter().map(Seg::field).zip(ins).collect();
        let rebuilt = Input::parts(source.type_ref(), parts, |values| {
            record.construct(&source_head, &values)
        });
        in_fallible |= rebuilt.fallible;
        let e = &rebuilt.expr;
        in_arms.push(quote!(#mirror_pat => #e));
        // Out: match the source alternative, produce the mirror one.
        let source_pat = record.pattern(&source_head, &binds);
        out_fallible |= outs.iter().any(|o| o.fallible);
        let out_binds: Vec<TokenStream> = outs.iter().zip(&binds).map(|(o, b)| o.bind(b)).collect();
        let values: Vec<TokenStream> = wires.iter().map(|w| w.name.to_token_stream()).collect();
        let rebuilt_mirror = record.construct(&mirror_head, &values);
        out_arms.push(quote!(#source_pat => { #(#out_binds)* #rebuilt_mirror }));
        alternatives.push(wires);
    }

    let def = quote! {
        #(#attrs)*
        pub enum #name { #(#variants),* }
    };
    Ok(SumMirror {
        def,
        alternatives,
        input: Code {
            expr: quote!(match v { #(#in_arms),* }),
            fallible: in_fallible,
        },
        output: Code {
            expr: quote!(match v { #(#out_arms),* }),
            fallible: out_fallible,
        },
    })
}

/// A mirror's conversion to or from its source type: an expression over
/// the mirror value `v`, which may use `?` on `Result<_, String>`.
struct Code {
    expr: TokenStream,
    fallible: bool,
}

impl Code {
    fn result(&self) -> TokenStream {
        result_expr(&self.expr, self.fallible)
    }
}

struct StructMirror {
    def: TokenStream,
    wires: Vec<Wire<CWire>>,
    input: Code,
    output: Code,
}
struct SumMirror {
    alternatives: Vec<Vec<Wire<CWire>>>,
    def: TokenStream,
    input: Code,
    output: Code,
}
