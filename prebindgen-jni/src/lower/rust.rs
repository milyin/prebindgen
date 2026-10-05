//! The Rust side of the codec: wires rebuilt into a value, and a value
//! taken apart into wires.

use prebindgen_flat::flat::{ScalarKind, TypeKind, TypeRef};
use prebindgen_tools::{names, Access, Input, Output, SequenceKind, Shape, TextKind, Wire};
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};

use super::{
    alt_seg, array_prim, field_seg, is_u8,
    leaf::{join, Leaf, LeafTy, Prim},
    leaf_ident, scalar_prim, Dir,
};
use crate::plan::{err, ClassKind, Conv, Plan, Res, Setting};

fn rt() -> TokenStream {
    quote!(::prebindgen_jni_runtime)
}

impl Plan<'_> {
    /// Wires → a value of `ty`. The wires are the input leaves of `ty`, named
    /// under `root`; `depth` keeps nested locals apart.
    pub(crate) fn rs_decode(&self, ty: &TypeRef, root: &str, depth: usize) -> Res<Input<Leaf>> {
        self.rs_decode_shape(ty, self.shape(ty)?, root, depth)
    }

    // The Cow container chooses an owned codec for its unsized child; `ty`
    // remains the actual boundary type used to obtain its wire list.
    fn rs_decode_shape(
        &self,
        ty: &TypeRef,
        shape: Shape<'_, &Setting>,
        root: &str,
        depth: usize,
    ) -> Res<Input<Leaf>> {
        let rt = rt();
        // The one wire `ty` arrives on, when it takes one leaf.
        let wire = || -> Res<Wire<Leaf>> {
            let l = self.leaves(ty, Dir::In)?.remove(0);
            Ok(Wire::new(leaf_ident(root, &l.name), l))
        };
        let one = |e: TokenStream| -> Res<Input<Leaf>> { Ok(Input::wire(ty, wire()?, e)) };
        let w = leaf_ident(root, "");
        let access = Access::of(ty);
        Ok(match shape {
            Shape::Unit => Input::parts(ty, Vec::new(), |_| quote!(())),
            Shape::Scalar(k) => {
                let range = |t: TokenStream, name: &str| {
                    let msg = format!("{name} input out of range: {{}}");
                    quote!(<#t as ::core::convert::TryFrom<_>>::try_from(#w).map_err(|_| ::std::format!(#msg, #w))?)
                };
                match k {
                    ScalarKind::Bool => one(quote!((#w != 0)))?,
                    ScalarKind::U8 => one(range(quote!(u8), "u8"))?.mark_fallible(),
                    ScalarKind::U16 => one(range(quote!(u16), "u16"))?.mark_fallible(),
                    ScalarKind::U32 => one(range(quote!(u32), "u32"))?.mark_fallible(),
                    ScalarKind::Usize => one(range(quote!(usize), "usize"))?.mark_fallible(),
                    ScalarKind::U64 => one(quote!((#w as u64)))?,
                    ScalarKind::Isize => one(quote!((#w as isize)))?,
                    _ => one(quote!(#w))?,
                }
            }
            Shape::Str { .. } => one(quote!(#rt::read_string(env, &#w)?))?.mark_fallible(),
            Shape::Seq { elem, .. } if is_u8(elem) => {
                one(quote!(#rt::read_u8s(env, &#w)?))?.mark_fallible()
            }
            Shape::Array { elem, len } => {
                let TypeKind::Scalar(k) = elem.kind() else {
                    return err(format!("`{ty}`: only arrays of primitives cross"));
                };
                let read = format_ident!("{}", array_prim(*k).array_helpers().0);
                let t = names::ident(k.as_str());
                let conv = if *k == ScalarKind::Bool {
                    quote!(__x != 0)
                } else {
                    quote!(__x as #t)
                };
                one(quote!(#rt::fixed::<#t, #len>(#rt::#read(env, &#w)?.into_iter().map(|__x| #conv).collect())?))?
                    .mark_fallible()
            }
            Shape::Declared { declaration, .. } => match declaration {
                Setting::Converted(c) => Input::via(&self.q, &c.resolved, |repr| {
                    let repr = self.rs_decode(repr, root, depth)?;
                    Ok::<_, crate::Error>(match domain_check(c, &quote!(__checked)) {
                        Some(check) => {
                            repr.and_then(|e| quote!({ let __checked = #e; #check __checked }))
                        }
                        None => repr,
                    })
                })?,
                Setting::Class(c) => {
                    let t = self.q.path(&names::ident(&c.rust));
                    match &c.kind {
                        ClassKind::Ptr { .. } => {
                            let f = match access {
                                Access::Owned => quote!(take_handle),
                                Access::Shared => quote!(borrow_handle),
                                Access::Exclusive => quote!(borrow_handle_mut),
                            };
                            one(quote!(#rt::#f::<#t>(#w)?))?.mark_fallible()
                        }
                        ClassKind::Enum => {
                            let arms = self.enum_arms(c)?;
                            let pats = arms
                                .iter()
                                .map(|(n, d)| quote!(#d => ::core::result::Result::Ok(#t::#n)));
                            let msg = format!("invalid value {{}} for enum `{}`", c.rust);
                            one(quote!((match #w { #(#pats,)* __v => ::core::result::Result::Err(::std::format!(#msg, __v)) })?))?
                                .mark_fallible()
                        }
                        ClassKind::Data { .. } => {
                            let s = self.struct_of(c)?;
                            let mut parts = Vec::new();
                            for f in &s.fields {
                                parts.push(self.rs_decode(
                                    &f.ty,
                                    &join(root, &field_seg(f)),
                                    depth,
                                )?);
                            }
                            Input::record(&self.q, s, parts)
                        }
                        ClassKind::Sealed { .. } => {
                            let v = self.variant_of(c)?;
                            let tag = Wire::new(
                                leaf_ident(root, "_tag"),
                                Leaf::new(LeafTy::Prim(Prim::I)),
                            );
                            let mut alts = Vec::new();
                            for alt in &v.alternatives {
                                let aseg = alt_seg(alt);
                                let mut parts = Vec::new();
                                for f in &alt.fields {
                                    let r = join(root, &join(&aseg, &field_seg(f)));
                                    parts.push(self.rs_decode(&f.ty, &r, depth)?);
                                }
                                alts.push(parts);
                            }
                            Input::sum(&self.q, v, tag, alts)
                        }
                    }
                }
            },
            Shape::Option(inner) => {
                let inner_leaves = self.leaves(inner, Dir::In)?;
                let i = self.rs_decode(inner, root, depth)?;
                if inner_leaves.len() == 1 && inner_leaves[0].is_obj() {
                    let n = i.wires()[0].name.clone();
                    Input::optional(ty, None, quote!(!#n.is_null()), i)
                } else {
                    let present = leaf_ident(root, "_present");
                    Input::optional(
                        ty,
                        Some(Wire::new(present.clone(), Leaf::new(LeafTy::Prim(Prim::Z)))),
                        quote!((#present != 0)),
                        i,
                    )
                }
            }
            Shape::Seq { elem, .. } => {
                let n = leaf_ident(root, "_n");
                let er = format!("__e{depth}");
                let elem_in = self.rs_decode(elem, &er, depth + 1)?;
                let mut wires = vec![Wire::new(n.clone(), Leaf::new(LeafTy::Prim(Prim::I)))];
                let mut reads = Vec::new();
                let mut binds = Vec::new();
                let mut drops = Vec::new();
                for (k, l) in self.leaves(elem, Dir::In)?.iter().enumerate() {
                    let col = leaf_ident(root, &l.name);
                    wires.push(Wire::new(col.clone(), l.column()));
                    let local = leaf_ident(&er, &l.name);
                    match l.prim() {
                        Some(p) => {
                            let read = format_ident!("{}", p.array_helpers().0);
                            let buf = format_ident!("__c{}_{}", depth, k);
                            reads.push(quote!(let #buf = #rt::#read(env, &#col)?;));
                            binds.push(quote!(let #local = #buf[__i];));
                        }
                        None => {
                            binds.push(
                                quote!(let #local = #rt::object_array_get(env, &#col, __i)?;),
                            );
                            drops.push(quote!(#rt::drop_local(env, #local);));
                        }
                    }
                }
                let e = elem_in.result();
                Input::seq(
                    ty,
                    wires,
                    elem_in.form,
                    quote!({
                        let __n = #n as usize;
                        #(#reads)*
                        let mut __v = ::std::vec::Vec::with_capacity(__n);
                        for __i in 0..__n {
                            #(#binds)*
                            let __x = #e;
                            #(#drops)*
                            __v.push(__x?);
                        }
                        __v
                    }),
                )
                .mark_fallible()
            }
            Shape::Ref { inner, .. } => self.rs_decode(inner, root, depth)?,
            Shape::Boxed(inner) => self
                .rs_decode(inner, root, depth)?
                .map(|e| quote!(::std::boxed::Box::new(#e))),
            Shape::Cow(inner) => {
                let input = match inner.kind() {
                    TypeKind::Str => self.rs_decode_shape(
                        ty,
                        Shape::Str {
                            kind: TextKind::String,
                            access: Access::Owned,
                        },
                        root,
                        depth,
                    )?,
                    TypeKind::Slice(elem) => self.rs_decode_shape(
                        ty,
                        Shape::Seq {
                            elem,
                            kind: SequenceKind::Vec,
                            access: Access::Owned,
                        },
                        root,
                        depth,
                    )?,
                    _ => self.rs_decode(inner, root, depth)?,
                };
                input.map(|e| quote!(::std::borrow::Cow::Owned(#e)))
            }
            Shape::Callback(args) => one(self.callback_closure(ty, args, &w)?)?.mark_fallible(),
            Shape::Undeclared(_) | Shape::Result { .. } | Shape::Out(_) => {
                unreachable!("refused by shape")
            }
        })
    }

    /// A value of `ty` (the expression `value`, owned) → its output wires,
    /// named under `root`.
    pub(crate) fn rs_encode(
        &self,
        ty: &TypeRef,
        value: TokenStream,
        root: &str,
        depth: usize,
    ) -> Res<Output<Leaf>> {
        self.rs_encode_shape(ty, self.shape(ty)?, value, root, depth)
    }

    fn rs_encode_shape(
        &self,
        ty: &TypeRef,
        shape: Shape<'_, &Setting>,
        value: TokenStream,
        root: &str,
        depth: usize,
    ) -> Res<Output<Leaf>> {
        let rt = rt();
        let single = |e: TokenStream, fallible: bool| -> Res<Output<Leaf>> {
            let l = self.leaves(ty, Dir::Out)?.remove(0);
            let out = Output::wire(ty, Wire::new(leaf_ident(root, &l.name), l), e);
            Ok(if fallible { out.mark_fallible() } else { out })
        };
        let access = Access::of(ty);
        Ok(match shape {
            Shape::Unit => Output::unit(ty, value),
            Shape::Scalar(k) => {
                let p = scalar_prim(k).rs();
                let e = match k {
                    ScalarKind::Bool => quote!((#value as u8)),
                    _ => quote!((#value as #p)),
                };
                single(e, false)?
            }
            Shape::Str { .. } => single(
                quote!(#rt::new_string(env, ::core::convert::AsRef::<str>::as_ref(&#value))?),
                true,
            )?,
            Shape::Seq { elem, .. } if is_u8(elem) => single(
                quote!(#rt::write_u8s(env, ::core::convert::AsRef::<[u8]>::as_ref(&#value))?),
                true,
            )?,
            Shape::Array { elem, .. } => {
                let TypeKind::Scalar(k) = elem.kind() else {
                    return err(format!("`{ty}`: only arrays of primitives cross"));
                };
                let p = array_prim(*k);
                let write = format_ident!("{}", p.array_helpers().1);
                let pt = p.rs();
                let conv = if *k == ScalarKind::Bool {
                    quote!(u8::from(*__x))
                } else {
                    quote!(*__x as #pt)
                };
                single(
                    quote!(#rt::#write(env, &#value.iter().map(|__x| #conv).collect::<::std::vec::Vec<_>>())?),
                    true,
                )?
            }
            Shape::Declared { declaration, .. } => match declaration {
                Setting::Converted(c) => Output::via(&self.q, &c.resolved, &value, |repr, r| {
                    let (r, checked) = match domain_check(c, &r) {
                        Some(check) => (quote!({ #check #r }), true),
                        None => (r, false),
                    };
                    let out = self.rs_encode(repr, r, root, depth + 1)?;
                    Ok::<_, crate::Error>(if checked { out.mark_fallible() } else { out })
                })?,
                Setting::Class(c) => {
                    let t = self.q.path(&names::ident(&c.rust));
                    match &c.kind {
                        ClassKind::Ptr { .. } => {
                            let e = match access {
                                Access::Owned => quote!(#rt::new_handle(#value)),
                                _ => quote!(#rt::new_handle(::core::clone::Clone::clone(#value))),
                            };
                            single(e, false)?
                        }
                        ClassKind::Enum => {
                            let arms = self.enum_arms(c)?;
                            let pats = arms.iter().map(|(n, d)| quote!(#t::#n => #d));
                            single(quote!((match #value { #(#pats),* } as i32)), false)?
                        }
                        ClassKind::Data { .. } => {
                            Output::record(&self.q, self.struct_of(c)?, &value, |f, b| {
                                self.rs_encode(&f.ty, b, &join(root, &field_seg(f)), depth + 1)
                            })?
                        }
                        ClassKind::Sealed { .. } => {
                            let tag = Wire::new(
                                leaf_ident(root, "_tag"),
                                Leaf::new(LeafTy::Prim(Prim::I)),
                            );
                            Output::sum(&self.q, self.variant_of(c)?, tag, &value, |alt, f, b| {
                                let seg = join(root, &join(&alt_seg(alt), &field_seg(f)));
                                self.rs_encode(&f.ty, b, &seg, depth + 1)
                            })?
                        }
                    }
                }
            },
            Shape::Option(inner) => {
                let inner_leaves = self.leaves(inner, Dir::Out)?;
                let inner_out = |x: TokenStream| self.rs_encode(inner, x, root, depth + 1);
                if inner_leaves.len() == 1 && inner_leaves[0].is_obj() {
                    // A single object leaf is null for `None`.
                    Output::optional(ty, None, &value, inner_out)?
                } else if inner_leaves.len() == 1 {
                    // A single primitive leaves boxed, so `None` can be null.
                    let p = inner_leaves[0].prim().expect("a primitive leaf");
                    let boxer = format_ident!("{}", p.box_helper());
                    let x = format_ident!("__x{}", depth);
                    let e = inner_out(x.to_token_stream())?.expr;
                    let w = Wire::new(leaf_ident(root, ""), Leaf::new(LeafTy::Boxed(p)));
                    Output::wire(
                        ty,
                        w,
                        quote!(match #value {
                            ::core::option::Option::Some(#x) => #rt::#boxer(env, #e)?,
                            ::core::option::Option::None => #rt::jni::objects::JObject::null(),
                        }),
                    )
                    .mark_fallible()
                } else {
                    let present = Wire::new(
                        leaf_ident(root, "_present"),
                        Leaf::new(LeafTy::Prim(Prim::Z)),
                    );
                    Output::optional(ty, Some((present, quote!(1u8))), &value, inner_out)?
                }
            }
            Shape::Seq { elem, access, .. } => {
                let n = leaf_ident(root, "_n");
                let er = format!("__e{depth}");
                let x = format_ident!("__x{}", depth);
                let elem_out = self.rs_encode(elem, x.to_token_stream(), &er, depth + 1)?;
                let mut wires = vec![Wire::new(n, Leaf::new(LeafTy::Prim(Prim::I)))];
                let mut setup = Vec::new();
                let mut pushes = Vec::new();
                let mut finals = vec![quote!(__n as i32)];
                for (k, l) in self.leaves(elem, Dir::Out)?.iter().enumerate() {
                    wires.push(Wire::new(leaf_ident(root, &l.name), l.column()));
                    let local = leaf_ident(&er, &l.name);
                    let col = format_ident!("__c{}_{}", depth, k);
                    match l.prim() {
                        Some(p) => {
                            let write = format_ident!("{}", p.array_helpers().1);
                            setup.push(quote!(let mut #col = ::std::vec::Vec::with_capacity(__n);));
                            pushes.push(quote!(#col.push(#local);));
                            finals.push(quote!(#rt::#write(env, &#col)?));
                        }
                        None => {
                            setup.push(quote!(let #col = #rt::new_object_array(env, __n)?;));
                            pushes.push(quote!(#rt::object_array_set(env, &#col, __i, #local)?;));
                            finals.push(quote!(#col));
                        }
                    }
                }
                let iter = match access {
                    Access::Owned => quote!(::core::iter::IntoIterator::into_iter(#value)),
                    Access::Shared | Access::Exclusive => quote!(#value.iter().cloned()),
                };
                let bind = elem_out.bind();
                Output::seq(
                    ty,
                    wires,
                    elem_out.form,
                    quote!({
                        let __items: ::std::vec::Vec<_> = #iter.collect();
                        let __n = __items.len();
                        #(#setup)*
                        for (__i, #x) in __items.into_iter().enumerate() {
                            #bind
                            #(#pushes)*
                        }
                        (#(#finals),*)
                    }),
                )
                .mark_fallible()
            }
            Shape::Ref { inner, .. } => self.rs_encode(
                inner,
                quote!(::core::clone::Clone::clone(#value)),
                root,
                depth,
            )?,
            Shape::Boxed(inner) => self.rs_encode(inner, quote!((*#value)), root, depth)?,
            Shape::Cow(inner) => match inner.kind() {
                TypeKind::Str => self.rs_encode_shape(
                    ty,
                    Shape::Str {
                        kind: TextKind::Str,
                        access: Access::Shared,
                    },
                    value,
                    root,
                    depth,
                )?,
                TypeKind::Slice(elem) => {
                    // Bytes can be read directly through AsRef; other elements
                    // are consumed from the owned Cow payload, as before.
                    let value = if is_u8(elem) {
                        value
                    } else {
                        quote!(#value.into_owned())
                    };
                    self.rs_encode_shape(
                        ty,
                        Shape::Seq {
                            elem,
                            kind: SequenceKind::Vec,
                            access: Access::Owned,
                        },
                        value,
                        root,
                        depth,
                    )?
                }
                _ => self.rs_encode(inner, quote!(#value.into_owned()), root, depth)?,
            },
            Shape::Callback(_) => return err(format!("`{ty}`: a callback cannot leave Rust")),
            Shape::Undeclared(_) | Shape::Result { .. } | Shape::Out(_) => {
                unreachable!("refused by shape")
            }
        })
    }
}

/// The statement checking that the representation bound to `r` lies in a
/// converted value's declared domain; `None` when it declares none.
fn domain_check(c: &Conv, r: &TokenStream) -> Option<TokenStream> {
    let (lo, hi) = c.range?;
    let name = &c.name;
    Some(quote!(let #r = ::prebindgen_jni_runtime::check_domain(#r, #lo, #hi, #name)?;))
}
