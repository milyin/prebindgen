//! How each type crosses the C boundary, by place.
//!
//! Every function here reads one layer of a type with
//! [`shape`](prebindgen_tools::shape) and recurses into what it holds. The
//! place decides the form:
//!
//! * a **value slot** — a struct field, a union payload, a by-value
//!   parameter, an array element — is one wire ([`Plan::value_in`],
//!   [`Plan::value_out`]), released by [`Plan::release`];
//! * a **parameter** can also be a pointer to what the caller keeps
//!   ([`Plan::param`]);
//! * a **result** can be returned, written to an out-parameter, or handed
//!   over as memory C frees ([`Plan::ret`]);
//! * a **callback argument** is lent for the duration of the call
//!   ([`Plan::callback_arg`]).

use prebindgen_flat::flat::{ScalarKind, Type as FlatType, TypeKind, TypeRef};
use prebindgen_tools::{
    function::result_ident, names, shape, Access, Input, Output, Return, SequenceKind, Shape,
    TextKind, Wire,
};
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};

use crate::plan::{declared_name, err, CType, Kind, Plan, Res, Setting};

/// A result's delivery, plus what the wrapper returns when an input fails.
pub(crate) struct CRet {
    pub ret: Return,
    /// The value returned on failure, for a function with an error slot;
    /// `None` when it has none.
    pub on_error: Option<TokenStream>,
}

impl<'f> Plan<'f> {
    pub(crate) fn shape<'t>(&self, ty: &'t TypeRef) -> Res<Shape<'t, &Setting>> {
        shape(ty, |candidate| match candidate.kind() {
            TypeKind::Named { id, .. } => self.setting(&id.name),
            TypeKind::String => self.setting("String"),
            _ => None,
        })
        .map_err(|e| crate::Error(format!("`{ty}`: {e}")))
    }

    /// The source type a declared name spells.
    pub(crate) fn source(&self, name: &str) -> TokenStream {
        if name == "String" {
            quote!(::std::string::String)
        } else {
            self.q.path(&names::ident(name))
        }
    }

    fn require_free(&self) -> Res<()> {
        if self.free_fn.is_none() {
            return err(
                "the binding hands out malloc'd memory (a string or an array); declare \
                 `.free_memory_function(..)` so C can release it",
            );
        }
        Ok(())
    }

    // ── value slots ─────────────────────────────────────────────────────

    /// A value slot `w` → a value of `ty`.
    pub(crate) fn value_in(&self, ty: &TypeRef, w: &syn::Ident) -> Res<Input> {
        let one = |t: TokenStream| vec![Wire::new(w.clone(), t)];
        Ok(match self.shape(ty)? {
            Shape::Scalar(ScalarKind::Bool) => Input::new(
                one(quote!(::core::mem::MaybeUninit<bool>)),
                quote!((::core::ptr::read(#w.as_ptr() as *const u8) != 0)),
            ),
            Shape::Scalar(k) => {
                let id = names::ident(k.as_str());
                Input::identity(Wire::new(w.clone(), quote!(#id)))
            }
            Shape::Str {
                kind: TextKind::String,
                access: Access::Owned,
            } => Input::new(
                one(quote!(*mut ::core::ffi::c_char)),
                quote!(if #w.is_null() {
                    ::std::string::String::new()
                } else {
                    ::std::ffi::CStr::from_ptr(#w).to_string_lossy().into_owned()
                }),
            ),
            Shape::Boxed(inner) => self
                .value_in(inner, w)?
                .map(|e| quote!(::std::boxed::Box::new(#e))),
            Shape::Declared { declaration, .. } if Access::of(ty) == Access::Owned => {
                let name = declared_name(ty);
                match declaration {
                    Setting::Converted(c) => {
                        let Some(stage) = &c.input else {
                            return err(format!("`{ty}` cannot cross into Rust"));
                        };
                        let repr = self.value_in(&stage.repr, w)?;
                        stage.decode(
                            &self.q,
                            &c.target,
                            repr,
                            &format_ident!("__cbg_repr"),
                            TokenStream::new(),
                        )
                    }
                    Setting::Type(t) => {
                        let (c, fin) = (&t.c, format_ident!("__cbg_in_{}", t.rust));
                        match &t.kind {
                            Kind::Enum | Kind::Union => Input::fallible(
                                one(quote!(::core::mem::MaybeUninit<#c>)),
                                quote!(#fin(#w)?),
                            ),
                            Kind::Data if self.data_in_fallible(name)? => {
                                Input::fallible(one(quote!(#c)), quote!(#fin(#w)?))
                            }
                            Kind::Data => Input::new(one(quote!(#c)), quote!(#fin(#w))),
                            Kind::Opaque => {
                                let src = self.source(name);
                                let msg = format!("null {name} handle passed by value");
                                Input::fallible(
                                    one(quote!(*mut #c)),
                                    quote!({
                                        if #w.is_null() {
                                            return ::core::result::Result::Err(::std::string::String::from(#msg));
                                        }
                                        *::std::boxed::Box::from_raw(#w as *mut #src)
                                    }),
                                )
                            }
                            Kind::ReprC { .. } => Input::new(
                                one(quote!(#c)),
                                quote!(<#c as ::prebindgen_c_runtime::Transmute>::into_rust(#w)),
                            ),
                            Kind::Error { .. } => {
                                return err(format!("`{ty}` cannot cross into Rust"))
                            }
                        }
                    }
                }
            }
            Shape::Undeclared(name) => {
                return err(format!("`{name}` is not declared to the C adapter"))
            }
            _ => return err(format!("`{ty}` has no C value representation")),
        })
    }

    /// A value of `ty` (the expression `v`) → a value slot `w`.
    pub(crate) fn value_out(&self, ty: &TypeRef, v: &TokenStream, w: &syn::Ident) -> Res<Output> {
        let one = |t: TokenStream, e: TokenStream| Output::single(Wire::new(w.clone(), t), e);
        Ok(match self.shape(ty)? {
            Shape::Scalar(ScalarKind::Bool) => one(
                quote!(::core::mem::MaybeUninit<bool>),
                quote!(::core::mem::MaybeUninit::new(#v)),
            ),
            Shape::Scalar(k) => {
                let id = names::ident(k.as_str());
                one(quote!(#id), v.clone())
            }
            Shape::Str {
                kind: TextKind::String,
                access: Access::Owned,
            } => {
                self.require_free()?;
                one(
                    quote!(*mut ::core::ffi::c_char),
                    quote!(__cbg_alloc_cstr(#v)),
                )
            }
            Shape::Boxed(inner) => self.value_out(inner, &quote!((*#v)), w)?,
            Shape::Declared { declaration, .. } if Access::of(ty) == Access::Owned => {
                let name = declared_name(ty);
                match declaration {
                    Setting::Converted(c) => {
                        let Some(stage) = &c.output else {
                            return err(format!("`{ty}` cannot cross out of Rust"));
                        };
                        let r = format_ident!("__cbg_value");
                        let repr = self.value_out(&stage.repr, &r.to_token_stream(), w)?;
                        stage.encode(&self.q, &c.target, v, &r, TokenStream::new(), repr)
                    }
                    Setting::Type(t) => {
                        let (c, fout) = (&t.c, format_ident!("__cbg_out_{}", t.rust));
                        match &t.kind {
                            Kind::Enum => one(
                                quote!(::core::mem::MaybeUninit<#c>),
                                quote!(::core::mem::MaybeUninit::new(#fout(#v))),
                            ),
                            Kind::Union => {
                                one(quote!(::core::mem::MaybeUninit<#c>), quote!(#fout(#v)))
                            }
                            Kind::Data => one(quote!(#c), quote!(#fout(#v))),
                            Kind::Opaque => one(
                                quote!(*mut #c),
                                quote!(::std::boxed::Box::into_raw(::std::boxed::Box::new(#v)) as *mut #c),
                            ),
                            Kind::ReprC { .. } => one(
                                quote!(#c),
                                quote!(<#c as ::prebindgen_c_runtime::Transmute>::from_rust(#v)),
                            ),
                            Kind::Error { .. } => {
                                self.require_free()?;
                                let _ = name;
                                one(quote!(*mut ::core::ffi::c_char), quote!(#fout(#v)))
                            }
                        }
                    }
                }
            }
            Shape::Undeclared(name) => {
                return err(format!("`{name}` is not declared to the C adapter"))
            }
            _ => return err(format!("`{ty}` has no C value representation")),
        })
    }

    /// Statements releasing what a value slot of `ty` at `place` owns, and
    /// leaving it releasable again; `None` when it owns nothing.
    pub(crate) fn release(&self, ty: &TypeRef, place: &TokenStream) -> Res<Option<TokenStream>> {
        let free = quote! {
            free(#place as *mut ::core::ffi::c_void);
            #place = ::core::ptr::null_mut();
        };
        Ok(match self.shape(ty)? {
            Shape::Str {
                kind: TextKind::String,
                access: Access::Owned,
            } => Some(free),
            Shape::Boxed(inner) => self.release(inner, place)?,
            Shape::Declared { declaration, .. } if Access::of(ty) == Access::Owned => {
                let name = declared_name(ty);
                match declaration {
                    Setting::Converted(c) => match c.output.as_ref().or(c.input.as_ref()) {
                        Some(stage) => self.release(&stage.repr, place)?,
                        None => None,
                    },
                    Setting::Type(t) => {
                        let (d, frel) = (&t.drop, format_ident!("__cbg_release_{}", t.rust));
                        match &t.kind {
                            Kind::Enum => None,
                            Kind::Union => self.owns(name)?.then(|| quote!(#d(&mut #place);)),
                            Kind::Data => self.owns(name)?.then(|| quote!(#frel(&mut #place);)),
                            Kind::Opaque => {
                                Some(quote! { #d(#place); #place = ::core::ptr::null_mut(); })
                            }
                            Kind::ReprC { .. } => Some(quote!(#d(&mut #place);)),
                            Kind::Error { .. } => Some(free),
                        }
                    }
                }
            }
            _ => None,
        })
    }

    fn fields_of(&self, name: &str) -> Res<Vec<&'f TypeRef>> {
        Ok(match self.element(name)? {
            FlatType::Struct(s) => s.fields.iter().map(|f| &f.ty).collect(),
            FlatType::Variant(v) => v
                .alternatives
                .iter()
                .flat_map(|a| a.fields.iter().map(|f| &f.ty))
                .collect(),
            _ => Vec::new(),
        })
    }

    /// Whether a struct or union value owns memory its fields release.
    pub(crate) fn owns(&self, name: &str) -> Res<bool> {
        let place = quote!(__p);
        for t in self.fields_of(name)? {
            if self.release(t, &place)?.is_some() {
                return Ok(true);
            }
        }
        Ok(false)
    }

    pub(crate) fn data_in_fallible(&self, name: &str) -> Res<bool> {
        let w = format_ident!("__w");
        for t in self.fields_of(name)? {
            if self.value_in(t, &w)?.fallible {
                return Ok(true);
            }
        }
        Ok(false)
    }

    // ── parameters ──────────────────────────────────────────────────────

    /// A parameter `name` of type `ty`.
    pub(crate) fn param(&self, name: &syn::Ident, ty: &TypeRef) -> Res<Input> {
        let null = |what: &str| format!("null {what} pointer");
        let fail = |msg: &str| quote!(return ::core::result::Result::Err(::std::string::String::from(#msg)));
        let borrowed_value = |access: Access| -> Res<Input> {
            let TypeKind::Ref { inner, .. } = ty.kind() else {
                unreachable!("borrowed shape without a borrowed type")
            };
            let input = self.value_in(inner, name)?;
            let pass = if access == Access::Exclusive {
                quote!(&mut #name)
            } else {
                quote!(&#name)
            };
            Ok(input.with_pass(pass))
        };
        Ok(match self.shape(ty)? {
            Shape::Str {
                kind: TextKind::Str,
                access: Access::Shared,
            } => {
                let (null, bad) = (
                    fail("null pointer passed for str argument"),
                    fail("invalid UTF-8 in str argument"),
                );
                Input::fallible(
                    vec![Wire::new(name.clone(), quote!(*const ::core::ffi::c_char))],
                    quote!({
                        if #name.is_null() { #null; }
                        match ::std::ffi::CStr::from_ptr(#name).to_str() {
                            ::core::result::Result::Ok(s) => s,
                            ::core::result::Result::Err(_) => #bad,
                        }
                    }),
                )
            }
            Shape::Str {
                kind: TextKind::String,
                access: Access::Owned,
            } => {
                let (null, bad) = (
                    fail("null pointer passed for String argument"),
                    fail("invalid UTF-8 in String argument"),
                );
                Input::fallible(
                    vec![Wire::new(name.clone(), quote!(*const ::core::ffi::c_char))],
                    quote!({
                        if #name.is_null() { #null; }
                        match ::std::ffi::CStr::from_ptr(#name).to_str() {
                            ::core::result::Result::Ok(s) => s.to_owned(),
                            ::core::result::Result::Err(_) => #bad,
                        }
                    }),
                )
            }
            Shape::Out(slot) => {
                let t = self.pointee(slot)?;
                let (c, src) = (&t.c, self.source(&t.rust.to_string()));
                let null = fail(&null(&t.rust.to_string()));
                Input::fallible(
                    vec![Wire::new(name.clone(), quote!(*mut #c))],
                    quote!({
                        if #name.is_null() { #null; }
                        &mut *(#name as *mut ::core::mem::MaybeUninit<#src>)
                    }),
                )
            }
            Shape::Seq {
                elem,
                kind: SequenceKind::Slice,
                access: Access::Shared,
            } => {
                let t = self.pointee(elem)?;
                let (c, src) = (&t.c, self.source(&t.rust.to_string()));
                let len = names::join(name, "len");
                Input::new(
                    vec![
                        Wire::new(name.clone(), quote!(*const #c)),
                        Wire::new(len.clone(), quote!(usize)),
                    ],
                    quote!(if #name.is_null() || #len == 0 {
                        &[][..]
                    } else {
                        ::core::slice::from_raw_parts(#name as *const #src, #len)
                    }),
                )
            }
            Shape::Declared {
                ty: declared_ty,
                declaration:
                    Setting::Type(
                        t @ CType {
                            kind: Kind::Opaque | Kind::ReprC { .. },
                            ..
                        },
                    ),
            } if Access::of(declared_ty) != Access::Owned => {
                let tname = declared_name(declared_ty);
                let access = Access::of(declared_ty);
                let (c, src) = (&t.c, self.source(tname));
                let null = fail(&null(tname));
                let (ptr, r) = if access == Access::Exclusive {
                    (quote!(*mut #c), quote!(&mut *(#name as *mut #src)))
                } else {
                    (quote!(*const #c), quote!(&*(#name as *const #src)))
                };
                Input::fallible(
                    vec![Wire::new(name.clone(), ptr)],
                    quote!({
                        if #name.is_null() { #null; }
                        #r
                    }),
                )
            }
            Shape::Declared {
                ty: declared_ty,
                declaration:
                    Setting::Type(
                        t @ CType {
                            kind: Kind::ReprC { .. },
                            ..
                        },
                    ),
            } if Access::of(declared_ty) == Access::Owned => {
                let tname = declared_name(declared_ty);
                let c = &t.c;
                let null = fail(&format!("null {tname} value passed by value"));
                let graves = self.repr_c_graves(tname);
                Input::fallible(
                    vec![Wire::new(name.clone(), quote!(*mut #c))],
                    quote!({
                        if #name.is_null() { #null; }
                        let __live = <#c as ::prebindgen_c_runtime::Transmute>::into_rust(::core::ptr::read(#name));
                        #( (*#name).#graves = ::core::ptr::null_mut(); )*
                        __live
                    }),
                )
            }
            // A borrow of a value: convert the value, lend it.
            Shape::Declared {
                ty: declared_ty, ..
            } if Access::of(declared_ty) != Access::Owned => {
                borrowed_value(Access::of(declared_ty))?
            }
            Shape::Str {
                kind: TextKind::String,
                access: access @ (Access::Shared | Access::Exclusive),
            }
            | Shape::Ref { access, .. } => borrowed_value(access)?,
            Shape::Callback(args) => {
                let key = ty.key().as_str().to_string();
                let c = self.closures[&key].clone();
                let closure = self.closure(name, args)?;
                Input::new(vec![Wire::new(name.clone(), c)], closure)
            }
            Shape::Option(inner) => match self.shape(inner)? {
                Shape::Declared {
                    ty: declared_ty,
                    declaration:
                        Setting::Type(
                            t @ CType {
                                kind: Kind::Opaque, ..
                            },
                        ),
                } if Access::of(declared_ty) == Access::Shared => {
                    let tname = declared_name(declared_ty);
                    let (c, src) = (&t.c, self.source(tname));
                    Input::new(
                        vec![Wire::new(name.clone(), quote!(*const #c))],
                        quote!(if #name.is_null() { ::core::option::Option::None } else { ::core::option::Option::Some(&*(#name as *const #src)) }),
                    )
                }
                _ => {
                    return err(format!(
                        "parameter `{name}`: `{ty}` has no C representation"
                    ))
                }
            },
            _ => self
                .value_in(ty, name)
                .map_err(|e| crate::Error(format!("parameter `{name}`: {}", e.0)))?,
        })
    }

    /// A type C passes by pointer: a declared `repr_c_struct` or opaque
    /// handle.
    fn pointee(&self, ty: &TypeRef) -> Res<&CType> {
        match self.shape(ty)? {
            Shape::Declared {
                ty: declared_ty,
                declaration:
                    Setting::Type(
                        t @ CType {
                            kind: Kind::ReprC { .. } | Kind::Opaque,
                            ..
                        },
                    ),
                ..
            } if Access::of(declared_ty) == Access::Owned => Ok(t),
            _ => err(format!(
                "`{ty}` must be a declared repr_c_struct or opaque_ptr here"
            )),
        }
    }

    /// The owned pointer fields of a `repr_c_struct`, nulled after its
    /// value is moved out so the caller's later drop is a no-op.
    fn repr_c_graves(&self, name: &str) -> Vec<syn::Ident> {
        let Some(FlatType::Struct(s)) = self.flat.declared_type(name) else {
            return Vec::new();
        };
        s.fields
            .iter()
            .filter(|f| matches!(f.ty.kind(), TypeKind::Optional(_)))
            .filter_map(|f| f.name.clone())
            .collect()
    }

    // ── results ─────────────────────────────────────────────────────────

    /// How a result of type `ty` leaves the wrapper.
    pub(crate) fn ret(&self, ty: &TypeRef) -> Res<CRet> {
        let r = result_ident();
        let Shape::Result { ok, err: e } = self.shape(ty)? else {
            return Ok(CRet {
                ret: self.plain_ret(ty)?,
                on_error: None,
            });
        };
        let ename = match self.shape(e)? {
            Shape::Declared {
                ty: declared_ty,
                declaration:
                    Setting::Type(CType {
                        kind: Kind::Error { .. },
                        ..
                    }),
            } if Access::of(declared_ty) == Access::Owned => declared_name(declared_ty),
            _ => {
                return err(format!(
                    "`{e}`: the error type must be declared `.opaque_error(..)`"
                ))
            }
        };
        self.require_free()?;
        let fout = format_ident!("__cbg_out_{}", ename);
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
        if let Some((wire, v)) = self.pointer_out(ok, &quote!(__v))? {
            return Ok(CRet {
                ret: Return {
                    ty: Some(wire),
                    wires: vec![e_wire],
                    body: quote!(match #r {
                        ::core::result::Result::Ok(__v) => #v,
                        ::core::result::Result::Err(__e) => { #set_e ::core::ptr::null_mut() }
                    }),
                },
                on_error: Some(quote!(::core::ptr::null_mut())),
            });
        }
        let (wire, v) = self.ret_value(ok, &quote!(__v))?;
        let out = Wire::new(format_ident!("out"), quote!(*mut #wire));
        Ok(CRet {
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
        })
    }

    /// A result C receives as a pointer it owns, if `ty` is one: the wire
    /// type and the expression producing it from `v`.
    fn pointer_out(
        &self,
        ty: &TypeRef,
        v: &TokenStream,
    ) -> Res<Option<(TokenStream, TokenStream)>> {
        Ok(match self.shape(ty)? {
            Shape::Str {
                kind: TextKind::String,
                access: Access::Owned,
            } => {
                self.require_free()?;
                Some((
                    quote!(*mut ::core::ffi::c_char),
                    quote!(__cbg_alloc_cstr(#v)),
                ))
            }
            Shape::Declared {
                ty: declared_ty,
                declaration:
                    Setting::Type(CType {
                        kind: Kind::Opaque,
                        c,
                        ..
                    }),
                ..
            } if Access::of(declared_ty) == Access::Owned => Some((
                quote!(*mut #c),
                quote!(::std::boxed::Box::into_raw(::std::boxed::Box::new(#v)) as *mut #c),
            )),
            Shape::Option(inner) => match self.shape(inner)? {
                Shape::Declared {
                    ty: declared_ty,
                    declaration:
                        Setting::Type(CType {
                            kind: Kind::Opaque,
                            c,
                            ..
                        }),
                    ..
                } if Access::of(declared_ty) != Access::Exclusive => {
                    let access = Access::of(declared_ty);
                    let val = if access == Access::Shared {
                        quote!(::core::clone::Clone::clone(__x))
                    } else {
                        quote!(__x)
                    };
                    Some((
                        quote!(*mut #c),
                        quote!(match #v {
                            ::core::option::Option::Some(__x) => ::std::boxed::Box::into_raw(::std::boxed::Box::new(#val)) as *mut #c,
                            ::core::option::Option::None => ::core::ptr::null_mut(),
                        }),
                    ))
                }
                _ => None,
            },
            _ => None,
        })
    }

    /// A result delivered by value: its wire type, and the expression
    /// producing it from `v`.
    fn ret_value(&self, ty: &TypeRef, v: &TokenStream) -> Res<(TokenStream, TokenStream)> {
        // A plain `bool` / enum leaves as itself: only an inbound value needs
        // the `MaybeUninit` guard.
        match self.shape(ty)? {
            Shape::Scalar(ScalarKind::Bool) => return Ok((quote!(bool), v.clone())),
            Shape::Declared {
                ty: declared_ty,
                declaration:
                    Setting::Type(CType {
                        kind: Kind::Enum,
                        c,
                        rust,
                        ..
                    }),
                ..
            } if Access::of(declared_ty) == Access::Owned => {
                let fout = format_ident!("__cbg_out_{}", rust);
                return Ok((quote!(#c), quote!(#fout(#v))));
            }
            _ => {}
        }
        if let Some(p) = self.pointer_out(ty, v)? {
            return Ok(p);
        }
        let out = self.value_out(ty, v, &format_ident!("__w"))?;
        if out.fallible {
            return err(format!(
                "`{ty}`: a fallible output conversion needs a Result return"
            ));
        }
        Ok((out.wires[0].ty.clone(), out.expr))
    }

    fn plain_ret(&self, ty: &TypeRef) -> Res<Return> {
        let r = result_ident();
        let array = |elem: &TypeRef| -> Res<(TokenStream, TokenStream)> {
            self.require_free()?;
            self.ret_value(elem, &quote!(__e))
        };
        Ok(match self.shape(ty)? {
            Shape::Unit => Return {
                ty: None,
                wires: Vec::new(),
                body: quote!(let _ = #r;),
            },
            Shape::Seq {
                elem,
                kind: SequenceKind::Vec,
                access: Access::Owned,
            } => {
                let (wire, e) = array(elem)?;
                Return {
                    ty: Some(quote!(*mut #wire)),
                    wires: vec![Wire::new(format_ident!("len"), quote!(*mut usize))],
                    body: quote! {
                        let __arr: ::std::vec::Vec<#wire> = #r.into_iter().map(|__e| #e).collect();
                        let (__p, __n) = __cbg_alloc_array(__arr);
                        if !len.is_null() { *len = __n; }
                        __p
                    },
                }
            }
            Shape::Option(inner)
                if !matches!(inner.kind(), TypeKind::Ref { .. })
                    && self.pointer_out(ty, &r.to_token_stream())?.is_none() =>
            {
                if let Shape::Seq {
                    elem,
                    kind: SequenceKind::Vec,
                    access: Access::Owned,
                } = self.shape(inner)?
                {
                    let (wire, e) = array(elem)?;
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
                let (wire, v) = self.ret_value(inner, &quote!(__v))?;
                Return {
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
                }
            }
            _ => {
                let (wire, v) = self.ret_value(ty, &r.to_token_stream())?;
                Return {
                    ty: Some(wire),
                    wires: Vec::new(),
                    body: v,
                }
            }
        })
    }

    // ── callbacks ───────────────────────────────────────────────────────

    /// Argument `index` of a C callback, of type `ty`, held in `value`.
    pub(crate) fn callback_arg(
        &self,
        index: usize,
        ty: &TypeRef,
        value: &TokenStream,
    ) -> Res<Output> {
        let n = format_ident!("__w{}", index);
        match self.shape(ty)? {
            Shape::Seq {
                elem,
                kind: SequenceKind::Slice,
                access: Access::Shared,
            } => {
                let c = &self.pointee(elem)?.c;
                let len = format_ident!("__w{}_len", index);
                Ok(Output::new(
                    vec![
                        Wire::new(n, quote!(*const #c)),
                        Wire::new(len, quote!(usize)),
                    ],
                    quote!((#value.as_ptr() as *const #c, #value.len())),
                ))
            }
            Shape::Declared {
                ty: declared_ty,
                declaration:
                    Setting::Type(CType {
                        kind: Kind::ReprC { .. } | Kind::Opaque,
                        c,
                        ..
                    }),
            } if Access::of(declared_ty) == Access::Shared => {
                let name = declared_name(declared_ty);
                let src = self.source(name);
                Ok(Output::single(
                    Wire::new(n, quote!(*const #c)),
                    quote!(#value as *const #src as *const #c),
                ))
            }
            _ => {
                let (wire, e) = self.ret_value(ty, value)?;
                Ok(Output::single(Wire::new(n, wire), e))
            }
        }
    }

    /// The closure an `impl Fn(..)` parameter `name` becomes, over the closure
    /// struct C passes.
    fn closure(&self, name: &syn::Ident, args: &[TypeRef]) -> Res<TokenStream> {
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
        prebindgen_tools::ClosureWriter::new(args)
            .setup(setup)
            .write(&self.q, &mut CallbackArgs(self), |binds, outs| {
                let values = outs.iter().flat_map(|o| o.wires.iter().map(|w| &w.name));
                quote! {
                    #binds
                    if let ::core::option::Option::Some(__f) = __call {
                        unsafe { __f(#(#values,)* __ctx.context) }
                    }
                }
            })
    }
}

struct CallbackArgs<'p, 'f>(&'p Plan<'f>);

impl prebindgen_tools::ClosureCallbacks for CallbackArgs<'_, '_> {
    type Error = crate::Error;

    fn arg(&mut self, index: usize, ty: &TypeRef, value: &TokenStream) -> Res<Output> {
        self.0.callback_arg(index, ty, value)
    }
}
