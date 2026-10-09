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
    names, shape, Access, Form, FormKind, Input, Output, SequenceKind, Shape, TextKind, Wire,
    WireType,
};
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};

use crate::{
    plan::{declared_name, err, CType, Kind, Plan, Res, Setting},
    wire::CWire,
};

pub(crate) struct Return {
    pub ty: Option<CWire>,
    pub wires: Vec<Wire<CWire>>,
    pub body: TokenStream,
}

/// A parameter's input, plus how the wrapper passes the bound value to the
/// callee when not as is — `&s` for a borrow of a converted local.
pub(crate) struct Param {
    pub input: Input<CWire>,
    pub pass: Option<TokenStream>,
}

/// The form of one element of a slice C lends in place.
fn in_place(elem: &TypeRef, c: &syn::Ident) -> Form<CWire> {
    Form {
        ty: elem.clone(),
        kind: FormKind::Wire(Wire::new(format_ident!("__elem"), CWire::Struct(c.clone()))),
    }
}

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
            match self.flat.element(name).and_then(|e| e.name()) {
                Some(n) => n.to_token_stream(),
                None => names::ident(name).to_token_stream(),
            }
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
    pub(crate) fn value_in(&self, ty: &TypeRef, w: &syn::Ident) -> Res<Input<CWire>> {
        let one = |t: CWire| Wire::new(w.clone(), t);
        Ok(match self.shape(ty)? {
            Shape::Scalar(ScalarKind::Bool) => Input::wire(
                ty,
                one(CWire::BoolSlot),
                quote!((::core::ptr::read(#w.as_ptr() as *const u8) != 0)),
            ),
            Shape::Scalar(k) => Input::wire(ty, one(CWire::Scalar(k)), w),
            Shape::Str {
                kind: TextKind::String,
                access: Access::Owned,
            } => Input::wire(
                ty,
                one(CWire::c_str(true)),
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
                    Setting::Converted(c) => Input::via(c, |repr| self.value_in(repr, w))?,
                    Setting::Type(t) => {
                        let (c, fin) = (&t.c, format_ident!("__cbg_in_{}", t.rust));
                        match &t.kind {
                            Kind::Enum | Kind::Union => {
                                Input::wire(ty, one(CWire::Uninit(c.clone())), quote!(#fin(#w)?))
                                    .mark_fallible()
                            }
                            Kind::Data if self.data_in_fallible(name)? => {
                                Input::wire(ty, one(CWire::Struct(c.clone())), quote!(#fin(#w)?))
                                    .mark_fallible()
                            }
                            Kind::Data => {
                                Input::wire(ty, one(CWire::Struct(c.clone())), quote!(#fin(#w)))
                            }
                            Kind::Opaque => {
                                let src = self.source(name);
                                let msg = format!("null {name} handle passed by value");
                                Input::wire(
                                    ty,
                                    one(CWire::ptr(true, CWire::Struct(c.clone()))),
                                    quote!({
                                        if #w.is_null() {
                                            return ::core::result::Result::Err(::std::string::String::from(#msg));
                                        }
                                        *::std::boxed::Box::from_raw(#w as *mut #src)
                                    }),
                                )
                                .mark_fallible()
                            }
                            Kind::ReprC { .. } => Input::wire(
                                ty,
                                one(CWire::Struct(c.clone())),
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

    /// A value of `ty` → a value slot `w`.
    pub(crate) fn value_out(&self, ty: &TypeRef, w: &syn::Ident) -> Res<Output<CWire>> {
        let one = |t: CWire, e: &dyn Fn(&TokenStream) -> TokenStream| {
            Output::wire(ty, Wire::new(w.clone(), t), e)
        };
        Ok(match self.shape(ty)? {
            Shape::Scalar(ScalarKind::Bool) => one(
                CWire::BoolSlot,
                &|v| quote!(::core::mem::MaybeUninit::new(#v)),
            ),
            Shape::Scalar(k) => one(CWire::Scalar(k), &|v| v.clone()),
            Shape::Str {
                kind: TextKind::String,
                access: Access::Owned,
            } => {
                self.require_free()?;
                one(CWire::c_str(true), &|v| quote!(__cbg_alloc_cstr(#v)))
            }
            Shape::Boxed(inner) => self.value_out(inner, w)?.before(|v| quote!((*#v))),
            Shape::Declared { declaration, .. } if Access::of(ty) == Access::Owned => {
                let name = declared_name(ty);
                match declaration {
                    Setting::Converted(c) => Output::via(c, |repr| self.value_out(repr, w))?,
                    Setting::Type(t) => {
                        let (c, fout) = (&t.c, format_ident!("__cbg_out_{}", t.rust));
                        match &t.kind {
                            Kind::Enum => one(
                                CWire::Uninit(c.clone()),
                                &|v| quote!(::core::mem::MaybeUninit::new(#fout(#v))),
                            ),
                            Kind::Union => one(CWire::Uninit(c.clone()), &|v| quote!(#fout(#v))),
                            Kind::Data => one(CWire::Struct(c.clone()), &|v| quote!(#fout(#v))),
                            Kind::Opaque => one(
                                CWire::ptr(true, CWire::Struct(c.clone())),
                                &|v| quote!(::std::boxed::Box::into_raw(::std::boxed::Box::new(#v)) as *mut #c),
                            ),
                            Kind::ReprC { .. } => one(
                                CWire::Struct(c.clone()),
                                &|v| quote!(<#c as ::prebindgen_c_runtime::Transmute>::from_rust(#v)),
                            ),
                            Kind::Error { .. } => {
                                self.require_free()?;
                                let _ = name;
                                one(CWire::c_str(true), &|v| quote!(#fout(#v)))
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
                    Setting::Converted(c) => self.release(c.repr(), place)?,
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
    pub(crate) fn param(&self, name: &syn::Ident, ty: &TypeRef) -> Res<Param> {
        let null = |what: &str| format!("null {what} pointer");
        let fail = |msg: &str| quote!(return ::core::result::Result::Err(::std::string::String::from(#msg)));
        let borrowed_value = |access: Access| -> Res<Param> {
            let TypeKind::Ref { inner, .. } = ty.kind() else {
                unreachable!("borrowed shape without a borrowed type")
            };
            let input = self.value_in(inner, name)?;
            let pass = if access == Access::Exclusive {
                quote!(&mut #name)
            } else {
                quote!(&#name)
            };
            Ok(Param {
                input,
                pass: Some(pass),
            })
        };
        let input = match self.shape(ty)? {
            Shape::Str {
                kind: TextKind::Str,
                access: Access::Shared,
            } => {
                let (null, bad) = (
                    fail("null pointer passed for str argument"),
                    fail("invalid UTF-8 in str argument"),
                );
                Input::wire(
                    ty,
                    Wire::new(name.clone(), CWire::c_str(false)),
                    quote!({
                        if #name.is_null() { #null; }
                        match ::std::ffi::CStr::from_ptr(#name).to_str() {
                            ::core::result::Result::Ok(s) => s,
                            ::core::result::Result::Err(_) => #bad,
                        }
                    }),
                )
                .mark_fallible()
            }
            Shape::Str {
                kind: TextKind::String,
                access: Access::Owned,
            } => {
                let (null, bad) = (
                    fail("null pointer passed for String argument"),
                    fail("invalid UTF-8 in String argument"),
                );
                Input::wire(
                    ty,
                    Wire::new(name.clone(), CWire::c_str(false)),
                    quote!({
                        if #name.is_null() { #null; }
                        match ::std::ffi::CStr::from_ptr(#name).to_str() {
                            ::core::result::Result::Ok(s) => s.to_owned(),
                            ::core::result::Result::Err(_) => #bad,
                        }
                    }),
                )
                .mark_fallible()
            }
            Shape::Out(slot) => {
                let t = self.pointee(slot)?;
                let (c, src) = (&t.c, self.source(&t.rust.to_string()));
                let null = fail(&null(&t.rust.to_string()));
                Input::wire(
                    ty,
                    Wire::new(name.clone(), CWire::ptr(true, CWire::Struct(c.clone()))),
                    quote!({
                        if #name.is_null() { #null; }
                        &mut *(#name as *mut ::core::mem::MaybeUninit<#src>)
                    }),
                )
                .mark_fallible()
            }
            Shape::Seq {
                elem,
                kind: SequenceKind::Slice,
                access: Access::Shared,
            } => {
                let t = self.pointee(elem)?;
                let (c, src) = (&t.c, self.source(&t.rust.to_string()));
                let len = names::join(name, "len");
                Input::seq(
                    ty,
                    vec![
                        Wire::new(name.clone(), CWire::ptr(false, CWire::Struct(c.clone()))),
                        Wire::new(len.clone(), CWire::Scalar(ScalarKind::Usize)),
                    ],
                    in_place(elem, c),
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
                let exclusive = access == Access::Exclusive;
                let r = if exclusive {
                    quote!(&mut *(#name as *mut #src))
                } else {
                    quote!(&*(#name as *const #src))
                };
                let ptr = CWire::ptr(exclusive, CWire::Struct(c.clone()));
                Input::wire(
                    ty,
                    Wire::new(name.clone(), ptr),
                    quote!({
                        if #name.is_null() { #null; }
                        #r
                    }),
                )
                .mark_fallible()
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
                Input::wire(
                    ty,
                    Wire::new(name.clone(), CWire::ptr(true, CWire::Struct(c.clone()))),
                    quote!({
                        if #name.is_null() { #null; }
                        let __live = <#c as ::prebindgen_c_runtime::Transmute>::into_rust(::core::ptr::read(#name));
                        #( (*#name).#graves = ::core::ptr::null_mut(); )*
                        __live
                    }),
                )
                .mark_fallible()
            }
            // A borrow of a value: convert the value, lend it.
            Shape::Declared {
                ty: declared_ty, ..
            } if Access::of(declared_ty) != Access::Owned => {
                return borrowed_value(Access::of(declared_ty))
            }
            Shape::Str {
                kind: TextKind::String,
                access: access @ (Access::Shared | Access::Exclusive),
            }
            | Shape::Ref { access, .. } => return borrowed_value(access),
            Shape::Callback(args) => {
                let key = ty.key().as_str().to_string();
                let c = self.closures[&key].clone();
                let closure = self.closure(name, args)?;
                Input::wire(ty, Wire::new(name.clone(), CWire::Struct(c)), closure)
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
                    Input::wire(
                        ty,
                        Wire::new(name.clone(), CWire::ptr(false, CWire::Struct(c.clone()))),
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
        };
        Ok(Param { input, pass: None })
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
        let r = format_ident!("__result");
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
        let e_wire = Wire::new(format_ident!("e"), CWire::ptr(true, CWire::c_str(true)));
        let set_e = quote!(if !e.is_null() { *e = #fout(__e); });
        if let TypeKind::Unit = ok.kind() {
            return Ok(CRet {
                ret: Return {
                    ty: Some(CWire::Bool),
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
        let out = Wire::new(format_ident!("out"), CWire::ptr(true, wire));
        Ok(CRet {
            ret: Return {
                ty: Some(CWire::Bool),
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
    fn pointer_out(&self, ty: &TypeRef, v: &TokenStream) -> Res<Option<(CWire, TokenStream)>> {
        Ok(match self.shape(ty)? {
            Shape::Str {
                kind: TextKind::String,
                access: Access::Owned,
            } => {
                self.require_free()?;
                Some((CWire::c_str(true), quote!(__cbg_alloc_cstr(#v))))
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
                CWire::ptr(true, CWire::Struct(c.clone())),
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
                        CWire::ptr(true, CWire::Struct(c.clone())),
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
    fn ret_value(&self, ty: &TypeRef, v: &TokenStream) -> Res<(CWire, TokenStream)> {
        // A plain `bool` / enum leaves as itself: only an inbound value needs
        // the `MaybeUninit` guard.
        match self.shape(ty)? {
            Shape::Scalar(ScalarKind::Bool) => return Ok((CWire::Bool, v.clone())),
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
                return Ok((CWire::Struct(c.clone()), quote!(#fout(#v))));
            }
            _ => {}
        }
        if let Some(p) = self.pointer_out(ty, v)? {
            return Ok(p);
        }
        let out = self.value_out(ty, &format_ident!("__w"))?;
        if out.fallible {
            return err(format!(
                "`{ty}`: a fallible output conversion needs a Result return"
            ));
        }
        let wire = out.wires()[0].ty.clone();
        Ok((wire, out.apply(v)))
    }

    fn plain_ret(&self, ty: &TypeRef) -> Res<Return> {
        let r = format_ident!("__result");
        let array = |elem: &TypeRef| -> Res<(CWire, TokenStream)> {
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
                let elem_ty = wire.rust();
                Return {
                    ty: Some(CWire::ptr(true, wire)),
                    wires: vec![Wire::new(
                        format_ident!("len"),
                        CWire::ptr(true, CWire::Scalar(ScalarKind::Usize)),
                    )],
                    body: quote! {
                        let __arr: ::std::vec::Vec<#elem_ty> = #r.into_iter().map(|__e| #e).collect();
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
                    let elem_ty = wire.rust();
                    return Ok(Return {
                        ty: Some(CWire::Bool),
                        wires: vec![
                            Wire::new(
                                format_ident!("out"),
                                CWire::ptr(true, CWire::ptr(true, wire)),
                            ),
                            Wire::new(
                                format_ident!("out_len"),
                                CWire::ptr(true, CWire::Scalar(ScalarKind::Usize)),
                            ),
                        ],
                        body: quote! {
                            match #r {
                                ::core::option::Option::Some(__v) => {
                                    let __arr: ::std::vec::Vec<#elem_ty> = __v.into_iter().map(|__e| #e).collect();
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
                    ty: Some(CWire::Bool),
                    wires: vec![Wire::new(format_ident!("out"), CWire::ptr(true, wire))],
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
    pub(crate) fn callback_arg(&self, index: usize, ty: &TypeRef) -> Res<Output<CWire>> {
        let n = format_ident!("__w{}", index);
        match self.shape(ty)? {
            Shape::Seq {
                elem,
                kind: SequenceKind::Slice,
                access: Access::Shared,
            } => {
                let c = &self.pointee(elem)?.c;
                let len = format_ident!("__w{}_len", index);
                Ok(Output::seq(
                    ty,
                    vec![
                        Wire::new(n, CWire::ptr(false, CWire::Struct(c.clone()))),
                        Wire::new(len, CWire::Scalar(ScalarKind::Usize)),
                    ],
                    in_place(elem, c),
                    |v| quote!((#v.as_ptr() as *const #c, #v.len())),
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
                Ok(Output::wire(
                    ty,
                    Wire::new(n, CWire::ptr(false, CWire::Struct(c.clone()))),
                    |v| quote!(#v as *const #src as *const #c),
                ))
            }
            _ => {
                // Delivered like a by-value result, over the argument.
                let (wire, e) = self.ret_value(ty, &quote!(__arg))?;
                Ok(Output::wire(
                    ty,
                    Wire::new(n, wire),
                    |v| quote!({ let __arg = #v; #e }),
                ))
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
        let names: Vec<_> = (0..args.len()).map(|i| format_ident!("__a{}", i)).collect();
        let tys = prebindgen_tools::callback_arg_types(self.flat, args);
        let outs = args
            .iter()
            .enumerate()
            .map(|(i, ty)| self.callback_arg(i, ty))
            .collect::<Res<Vec<_>>>()?;
        let binds: TokenStream = outs.iter().zip(&names).map(|(o, n)| o.bind(n)).collect();
        let values: Vec<syn::Ident> = outs
            .iter()
            .flat_map(|o| o.wires().into_iter().map(|w| w.name.clone()))
            .collect();
        Ok(quote! {{
            #setup
            move |#(#names: #tys),*| {
                let __res = (|| -> ::core::result::Result<(), ::std::string::String> {
                    #binds
                    if let ::core::option::Option::Some(__f) = __call {
                        unsafe { __f(#(#values,)* __ctx.context) }
                    }
                    ::core::result::Result::Ok(())
                })();
                if let ::core::result::Result::Err(__err) = __res { }
            }
        }})
    }
}
