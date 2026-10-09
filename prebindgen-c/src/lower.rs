//! How each type crosses the C boundary, by place.
//!
//! A value crosses as a [`Crossing`] that [`resolve`] builds from the plan's
//! ways and choices; this module answers what it asks the C boundary
//! ([`Lower`]), by the kind of place:
//!
//! * a **value slot** — a struct field, a union payload, a representation —
//!   is one wire ([`Plan::value_in`], [`Plan::value_out`]), released by
//!   [`Plan::release`];
//! * a **parameter** can also be a pointer to what the caller keeps, or a
//!   slice lent in place ([`Plan::param`]);
//! * a **result** can be returned, written to an out-parameter, or handed
//!   over as memory C frees ([`Plan::ret`]);
//! * a **callback argument** is lent for the duration of the call
//!   ([`Plan::callback_arg`]).

use prebindgen_flat::flat::{ScalarKind, Type as FlatType, TypeKind, TypeRef, Variant};
use prebindgen_tools::{
    names, resolve, resolve_arm, shape, Access, Arm, Choices, Code, Crossing, Decode, Direction,
    Encode, In, Input, Lower, Out, Output, Place, Presence, Seg, Sequence, SequenceKind, Shape,
    TextKind, Ways, Whole, Wire, WireType, Wrapper,
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

/// A parameter's crossing, plus how the wrapper passes the bound value to
/// the callee when not as is — `&s` for a borrow of a converted local.
pub(crate) struct Param<'f> {
    pub crossing: Crossing<'f, CWire, In>,
    pub pass: Option<TokenStream>,
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
    fn value_in(&self, ty: &TypeRef, w: &syn::Ident) -> Res<Option<Whole<CWire, In>>> {
        let one = |t: CWire| Wire::new(w.clone(), t);
        let wire = |t: CWire, e: TokenStream| read(one(t), Code::new(e));
        let fallible = |t: CWire, e: TokenStream| read(one(t), Code::fallible(e));
        Ok(Some(match self.shape(ty)? {
            Shape::Scalar(ScalarKind::Bool) => wire(
                CWire::BoolSlot,
                quote!((::core::ptr::read(#w.as_ptr() as *const u8) != 0)),
            ),
            Shape::Scalar(k) => wire(CWire::Scalar(k), quote!(#w)),
            Shape::Str {
                kind: TextKind::String,
                access: Access::Owned,
            } => wire(
                CWire::c_str(true),
                quote!(if #w.is_null() {
                    ::std::string::String::new()
                } else {
                    ::std::ffi::CStr::from_ptr(#w).to_string_lossy().into_owned()
                }),
            ),
            // Crossed as what it boxes.
            Shape::Boxed(_) => return Ok(None),
            Shape::Declared { declaration, .. } if Access::of(ty) == Access::Owned => {
                let name = declared_name(ty);
                match declaration {
                    Setting::Converted(_) => {
                        return err(format!("`{ty}`: its conversion declares no input"))
                    }
                    Setting::Type(t) => {
                        let (c, fin) = (&t.c, format_ident!("__cbg_in_{}", t.rust));
                        match &t.kind {
                            Kind::Enum | Kind::Union => {
                                fallible(CWire::Uninit(c.clone()), quote!(#fin(#w)?))
                            }
                            Kind::Data if self.data_in_fallible(name)? => {
                                fallible(CWire::Struct(c.clone()), quote!(#fin(#w)?))
                            }
                            Kind::Data => wire(CWire::Struct(c.clone()), quote!(#fin(#w))),
                            Kind::Opaque => {
                                let src = self.source(name);
                                let msg = format!("null {name} handle passed by value");
                                fallible(
                                    CWire::ptr(true, CWire::Struct(c.clone())),
                                    quote!({
                                        if #w.is_null() {
                                            return ::core::result::Result::Err(::std::string::String::from(#msg));
                                        }
                                        *::std::boxed::Box::from_raw(#w as *mut #src)
                                    }),
                                )
                            }
                            Kind::ReprC { .. } => wire(
                                CWire::Struct(c.clone()),
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
        }))
    }

    /// A value of `ty` → a value slot `w`.
    fn value_out(&self, ty: &TypeRef, w: &syn::Ident) -> Res<Option<Whole<CWire, Out>>> {
        let one = |t: CWire, e: &dyn Fn(&TokenStream) -> TokenStream| {
            Whole::<CWire, Out>::new(Wire::new(w.clone(), t), |v| Code::new(e(v)))
        };
        Ok(Some(match self.shape(ty)? {
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
            // Crossed as what it boxes.
            Shape::Boxed(_) => return Ok(None),
            Shape::Declared { declaration, .. } if Access::of(ty) == Access::Owned => {
                let name = declared_name(ty);
                match declaration {
                    Setting::Converted(_) => {
                        return err(format!("`{ty}`: its conversion declares no output"))
                    }
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
        }))
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
        let slot = Place::new(name).at(Seg::Field("__w".into()));
        for t in self.fields_of(name)? {
            if self.input(t, slot.clone())?.decode(self)?.is_fallible() {
                return Ok(true);
            }
        }
        Ok(false)
    }

    // ── parameters ──────────────────────────────────────────────────────

    /// A parameter `name` of type `ty` crossing whole, or `None` for a
    /// slice C lends in place.
    fn param_whole(&self, name: &syn::Ident, ty: &TypeRef) -> Res<Option<Whole<CWire, In>>> {
        let null = |what: &str| format!("null {what} pointer");
        let fail = |msg: &str| quote!(return ::core::result::Result::Err(::std::string::String::from(#msg)));
        // A borrow of a value: the value, which the call lends ([`Self::pass`]).
        let borrowed_value = || -> Res<Option<Whole<CWire, In>>> {
            let TypeKind::Ref { inner, .. } = ty.kind() else {
                unreachable!("borrowed shape without a borrowed type")
            };
            self.value_in(inner, name)
        };
        Ok(Some(match self.shape(ty)? {
            Shape::Str {
                kind: TextKind::Str,
                access: Access::Shared,
            } => {
                let (null, bad) = (
                    fail("null pointer passed for str argument"),
                    fail("invalid UTF-8 in str argument"),
                );
                read(
                    Wire::new(name.clone(), CWire::c_str(false)),
                    Code::fallible(quote!({
                        if #name.is_null() { #null; }
                        match ::std::ffi::CStr::from_ptr(#name).to_str() {
                            ::core::result::Result::Ok(s) => s,
                            ::core::result::Result::Err(_) => #bad,
                        }
                    })),
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
                read(
                    Wire::new(name.clone(), CWire::c_str(false)),
                    Code::fallible(quote!({
                        if #name.is_null() { #null; }
                        match ::std::ffi::CStr::from_ptr(#name).to_str() {
                            ::core::result::Result::Ok(s) => s.to_owned(),
                            ::core::result::Result::Err(_) => #bad,
                        }
                    })),
                )
            }
            Shape::Out(slot) => {
                let t = self.pointee(slot)?;
                let (c, src) = (&t.c, self.source(&t.rust.to_string()));
                let null = fail(&null(&t.rust.to_string()));
                read(
                    Wire::new(name.clone(), CWire::ptr(true, CWire::Struct(c.clone()))),
                    Code::fallible(quote!({
                        if #name.is_null() { #null; }
                        &mut *(#name as *mut ::core::mem::MaybeUninit<#src>)
                    })),
                )
            }
            // Lent in place: a sequence of its elements ([`Lower::sequence`]).
            Shape::Seq {
                kind: SequenceKind::Slice,
                access: Access::Shared,
                ..
            } => return Ok(None),
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
                read(
                    Wire::new(name.clone(), ptr),
                    Code::fallible(quote!({
                        if #name.is_null() { #null; }
                        #r
                    })),
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
                read(
                    Wire::new(name.clone(), CWire::ptr(true, CWire::Struct(c.clone()))),
                    Code::fallible(quote!({
                        if #name.is_null() { #null; }
                        let __live = <#c as ::prebindgen_c_runtime::Transmute>::into_rust(::core::ptr::read(#name));
                        #( (*#name).#graves = ::core::ptr::null_mut(); )*
                        __live
                    })),
                )
            }
            // A borrow of a value: convert the value, lend it.
            Shape::Declared {
                ty: declared_ty, ..
            } if Access::of(declared_ty) != Access::Owned => return borrowed_value(),
            Shape::Str {
                kind: TextKind::String,
                access: Access::Shared | Access::Exclusive,
            }
            | Shape::Ref { .. } => return borrowed_value(),
            Shape::Callback(args) => {
                let key = ty.key().as_str().to_string();
                let c = self.closures[&key].clone();
                let closure = self.closure(name, args)?;
                read(
                    Wire::new(name.clone(), CWire::Struct(c)),
                    Code::new(closure),
                )
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
                    read(
                        Wire::new(name.clone(), CWire::ptr(false, CWire::Struct(c.clone()))),
                        Code::new(
                            quote!(if #name.is_null() { ::core::option::Option::None } else { ::core::option::Option::Some(&*(#name as *const #src)) }),
                        ),
                    )
                }
                _ => {
                    return err(format!(
                        "parameter `{name}`: `{ty}` has no C representation"
                    ))
                }
            },
            _ => {
                return self
                    .value_in(ty, name)
                    .map_err(|e| crate::Error(format!("parameter `{name}`: {}", e.0)))
            }
        }))
    }

    /// How the call passes parameter `name` of type `ty`: a borrow of the
    /// converted local when the type borrows a value, else the local.
    fn pass(&self, name: &syn::Ident, ty: &TypeRef) -> Res<Option<TokenStream>> {
        let lend = |access: Access| match access {
            Access::Exclusive => Some(quote!(&mut #name)),
            _ => Some(quote!(&#name)),
        };
        Ok(match self.shape(ty)? {
            Shape::Declared {
                declaration:
                    Setting::Type(CType {
                        kind: Kind::Opaque | Kind::ReprC { .. },
                        ..
                    }),
                ..
            } => None,
            Shape::Declared {
                ty: declared_ty, ..
            } if Access::of(declared_ty) != Access::Owned => lend(Access::of(declared_ty)),
            Shape::Str {
                kind: TextKind::String,
                access: access @ (Access::Shared | Access::Exclusive),
            }
            | Shape::Ref { access, .. } => lend(access),
            _ => None,
        })
    }

    /// Parameter `name` of `func`, of type `ty`: how it crosses, and how the
    /// call passes it.
    pub(crate) fn param(&self, func: &str, name: &syn::Ident, ty: &TypeRef) -> Res<Param<'f>> {
        let place = Place::new(func).at(Seg::Param(names::bare(name)));
        Ok(Param {
            crossing: self.input(ty, place)?,
            pass: self.pass(name, ty)?,
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
        let slot = self.output(ty, Place::new("result").at(Seg::Field("__w".into())))?;
        let out = slot.encode(self)?;
        if out.is_fallible() {
            return err(format!(
                "`{ty}`: a fallible output conversion needs a Result return"
            ));
        }
        let wire = slot.wires()[0].ty.clone();
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

    /// Callback argument `n` of type `ty` crossing whole, or `None` for a
    /// slice lent in place.
    fn arg_whole(&self, n: &syn::Ident, ty: &TypeRef) -> Res<Option<Whole<CWire, Out>>> {
        Ok(Some(match self.shape(ty)? {
            // Lent in place: a sequence of its elements ([`Lower::sequence`]).
            Shape::Seq {
                kind: SequenceKind::Slice,
                access: Access::Shared,
                ..
            } => return Ok(None),
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
                Whole::<CWire, Out>::new(
                    Wire::new(n.clone(), CWire::ptr(false, CWire::Struct(c.clone()))),
                    |v| Code::new(quote!(#v as *const #src as *const #c)),
                )
            }
            _ => {
                // Delivered like a by-value result, over the argument.
                let (wire, e) = self.ret_value(ty, &quote!(__arg))?;
                Whole::<CWire, Out>::new(Wire::new(n.clone(), wire), |v| {
                    Code::new(quote!({ let __arg = #v; #e }))
                })
            }
        }))
    }

    /// Argument `index` of a C callback, of type `ty`.
    pub(crate) fn callback_arg(&self, index: usize, ty: &TypeRef) -> Res<Crossing<'f, CWire, Out>> {
        self.output(ty, Place::new("callback").at(Seg::Arg(index)))
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
        let mut binds = TokenStream::new();
        for (o, n) in outs.iter().zip(&names) {
            let out = o.encode(self)?;
            // A callback has no error channel for a conversion to fail into.
            if out.is_fallible() {
                return err(format!(
                    "callback argument `{}`: a fallible output conversion needs a Result return",
                    o.ty()
                ));
            }
            binds.extend(out.bind(n));
        }
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

// ── crossings ───────────────────────────────────────────────────────────

/// The C boundary in direction `D`: value slots, parameters, callback
/// arguments and slices lent in place, by place.
struct Boundary<'a, 'f, D: Direction> {
    plan: &'a Plan<'f>,
    choices: &'a Choices<D>,
    /// The sum whose arms are being mirrored: its fields are named by
    /// position.
    sum: Option<&'f Variant>,
}

impl<D: Direction> Boundary<'_, '_, D> {
    /// The name of the slot at `place`: the parameter's, the callback
    /// argument's, or the field's in the mirror.
    fn slot(&self, place: &Place) -> syn::Ident {
        let mut alt = None;
        let mut slot = names::ident("value");
        for seg in &place.path {
            match seg {
                Seg::Param(n) => slot = names::ident(n),
                Seg::Arg(i) => slot = format_ident!("__w{}", i),
                Seg::Alt(a) => alt = Some(a.as_str()),
                Seg::Field(n) => {
                    slot = match (self.sum, alt) {
                        (Some(v), Some(a)) => {
                            let index = v
                                .alternatives
                                .iter()
                                .find(|x| names::bare(&x.name) == a)
                                .and_then(|x| x.fields.iter().find(|f| Seg::field(f) == *seg))
                                .map(|f| f.index)
                                .expect("a field of the mirrored arm");
                            format_ident!("__f{}", index)
                        }
                        _ if n.starts_with(|c: char| c.is_ascii_digit()) => {
                            format_ident!("_{}", n)
                        }
                        _ => names::ident(n),
                    }
                }
                Seg::Return | Seg::Some | Seg::Elem | Seg::Repr | Seg::Inner => {}
            }
        }
        slot
    }
}

impl<'f, D: Direction> Lower<'f, D> for Boundary<'_, 'f, D> {
    type Wire = CWire;
    type Leaf = ();
    type Error = crate::Error;

    fn ways(&self) -> &Ways<'f, ()> {
        &self.plan.ways
    }

    fn choices(&self) -> &Choices<D> {
        self.choices
    }

    fn whole(
        &self,
        ty: &TypeRef,
        _: Shape<'_, &()>,
        place: &Place,
    ) -> Res<Option<Whole<CWire, D>>> {
        let name = self.slot(place);
        if let [.., Seg::Elem] = place.path.as_slice() {
            // One element of a slice lent in place: the C struct itself.
            let c = &self.plan.pointee(ty)?.c;
            let elem = Wire::new(format_ident!("__elem"), CWire::Struct(c.clone()));
            let same = |e: &dyn ToTokens| Code::new(e.to_token_stream());
            return Ok(Some(Whole::either(elem, |w| same(w), |v| same(v))));
        }
        let root = |kind: fn(&Seg) -> bool| matches!(place.path.as_slice(), [seg] if kind(seg));
        Whole::by_direction(
            || match root(|s| matches!(s, Seg::Param(_))) {
                true => self.plan.param_whole(&name, ty),
                false => self.plan.value_in(ty, &name),
            },
            || match root(|s| matches!(s, Seg::Arg(_))) {
                true => self.plan.arg_whole(&name, ty),
                false => self.plan.value_out(ty, &name),
            },
        )
    }

    fn presence(&self, inner: &Crossing<'f, CWire, D>, place: &Place) -> Res<Presence<CWire, D>> {
        err(format!(
            "{place}: `Option<{}>` has no C representation here",
            inner.ty()
        ))
    }

    fn sequence(&self, elem: &Crossing<'f, CWire, D>, place: &Place) -> Res<Vec<Wire<CWire>>> {
        let c = &self.plan.pointee(elem.ty())?.c;
        let name = self.slot(place);
        let len = names::join(&name, "len");
        Ok(vec![
            Wire::new(name, CWire::ptr(false, CWire::Struct(c.clone()))),
            Wire::new(len, CWire::Scalar(ScalarKind::Usize)),
        ])
    }

    fn tag(&self, place: &Place) -> Res<Wire<CWire>> {
        err(format!(
            "{place}: C mirrors a sum rather than taking it apart"
        ))
    }
}

impl<'f> Plan<'f> {
    fn boundary<'a, D: Direction>(
        &'a self,
        choices: &'a Choices<D>,
        sum: Option<&'f Variant>,
    ) -> Boundary<'a, 'f, D> {
        Boundary {
            plan: self,
            choices,
            sum,
        }
    }

    /// How the value of `ty` at `place` crosses into Rust.
    pub(crate) fn input(&self, ty: &TypeRef, place: Place) -> Res<Crossing<'f, CWire, In>> {
        resolve(&self.boundary(&self.inputs, None), ty, place)
    }

    /// How the value of `ty` at `place` crosses out of Rust.
    pub(crate) fn output(&self, ty: &TypeRef, place: Place) -> Res<Crossing<'f, CWire, Out>> {
        resolve(&self.boundary(&self.outputs, None), ty, place)
    }

    /// Alternative `alt` of the sum `v`, mirrored at `place`, both ways.
    pub(crate) fn mirror_arm(
        &self,
        v: &'f Variant,
        alt: &'f prebindgen_flat::flat::Alternative,
        place: &Place,
    ) -> Res<(Arm<'f, CWire, In>, Arm<'f, CWire, Out>)> {
        Ok((
            resolve_arm(&self.boundary(&self.inputs, Some(v)), v, alt, place)?,
            resolve_arm(&self.boundary(&self.outputs, Some(v)), v, alt, place)?,
        ))
    }
}

impl<'f> Decode<'f, CWire> for Plan<'f> {
    type Error = crate::Error;

    fn sequence(
        &self,
        _: &Crossing<'f, CWire, In>,
        seq: &Sequence<'f, CWire, In>,
        _: Input,
    ) -> Res<Code> {
        let [name, len] = seq.wires() else {
            unreachable!("a slice lent in place is a pointer and a length")
        };
        let (name, len) = (&name.name, &len.name);
        let src = self.source(&self.pointee(seq.elem().ty())?.rust.to_string());
        Ok(Code::new(quote!(if #name.is_null() || #len == 0 {
            &[][..]
        } else {
            ::core::slice::from_raw_parts(#name as *const #src, #len)
        })))
    }

    /// A borrow passes the converted local, which the call lends
    /// ([`Plan::pass`]).
    fn wrapped(&self, _: &Crossing<'f, CWire, In>, wrapper: Wrapper, inner: Input) -> Res<Input> {
        Ok(match wrapper {
            Wrapper::Box => inner.map(|e| quote!(::std::boxed::Box::new(#e))),
            Wrapper::Cow => inner.map(|e| quote!(::std::borrow::Cow::Owned(#e))),
            Wrapper::Ref(_) => inner,
        })
    }
}

impl<'f> Encode<'f, CWire> for Plan<'f> {
    type Error = crate::Error;

    fn sequence(
        &self,
        _: &Crossing<'f, CWire, Out>,
        seq: &Sequence<'f, CWire, Out>,
        _: &Output,
        v: &TokenStream,
    ) -> Res<Code> {
        let c = &self.pointee(seq.elem().ty())?.c;
        Ok(Code::new(quote!((#v.as_ptr() as *const #c, #v.len()))))
    }
}

/// A leaf into Rust whose code `code` reads its wire.
fn read(wire: Wire<CWire>, code: Code) -> Whole<CWire, In> {
    Whole::<CWire, In>::new(wire, |_| code)
}
