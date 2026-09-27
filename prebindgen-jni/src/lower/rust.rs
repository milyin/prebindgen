//! The Rust side of the codec: wires rebuilt into a value, and a value
//! taken apart into wires.

use prebindgen_flat::flat::{ScalarKind, TypeKind, TypeRef};
use prebindgen_tools::{
    names, record_in, record_out, Access, Input, Output, Record, SequenceKind, Shape, TextKind,
    Wire,
};
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};

use super::{
    alt_seg, array_prim, field_seg, is_u8,
    leaf::{join, Leaf, Prim},
    leaf_ident, scalar_prim, Dir,
};
use crate::plan::{err, ClassKind, Conv, Plan, Res, Setting};

fn rt() -> TokenStream {
    quote!(::prebindgen_jni_runtime)
}

impl Plan<'_> {
    /// Wires → a value of `ty`. The wires are the input leaves of `ty`, named
    /// under `root`; `depth` keeps nested locals apart.
    pub(crate) fn rs_decode(&self, ty: &TypeRef, root: &str, depth: usize) -> Res<Input> {
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
    ) -> Res<Input> {
        let rt = rt();
        let wires = || -> Res<Vec<Wire>> {
            Ok(self
                .leaves(ty, Dir::In)?
                .iter()
                .map(|l| Wire::new(leaf_ident(root, &l.name), l.rs()))
                .collect())
        };
        let w = leaf_ident(root, "");
        let access = Access::of(ty);
        Ok(match shape {
            Shape::Unit => Input::new(Vec::new(), quote!(())),
            Shape::Scalar(k) => {
                let wires = wires()?;
                let range = |t: TokenStream, name: &str| {
                    let msg = format!("{name} input out of range: {{}}");
                    quote!(<#t as ::core::convert::TryFrom<_>>::try_from(#w).map_err(|_| ::std::format!(#msg, #w))?)
                };
                match k {
                    ScalarKind::Bool => Input::new(wires, quote!((#w != 0))),
                    ScalarKind::U8 => Input::fallible(wires, range(quote!(u8), "u8")),
                    ScalarKind::U16 => Input::fallible(wires, range(quote!(u16), "u16")),
                    ScalarKind::U32 => Input::fallible(wires, range(quote!(u32), "u32")),
                    ScalarKind::Usize => Input::fallible(wires, range(quote!(usize), "usize")),
                    ScalarKind::U64 => Input::new(wires, quote!((#w as u64))),
                    ScalarKind::Isize => Input::new(wires, quote!((#w as isize))),
                    _ => Input::new(wires, quote!(#w)),
                }
            }
            Shape::Str { .. } => Input::fallible(wires()?, quote!(#rt::read_string(env, &#w)?)),
            Shape::Seq { elem, .. } if is_u8(elem) => {
                Input::fallible(wires()?, quote!(#rt::read_u8s(env, &#w)?))
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
                Input::fallible(
                    wires()?,
                    quote!(#rt::fixed::<#t, #len>(#rt::#read(env, &#w)?.into_iter().map(|__x| #conv).collect())?),
                )
            }
            Shape::Declared { declaration, .. } => match declaration {
                Setting::Converted(c) => {
                    let stage = c.resolved.input.as_ref().ok_or_else(|| {
                        crate::Error(format!("convert!({}) has no input", c.name))
                    })?;
                    let repr = self.rs_decode(&stage.repr, root, depth)?;
                    let r = format_ident!("__r{}", depth);
                    let check = domain_check(c, &r);
                    stage.decode(&self.q, &c.resolved.target, repr, &r, check)
                }
                Setting::Class(c) => {
                    let t = self.q.path(&names::ident(&c.rust));
                    match &c.kind {
                        ClassKind::Ptr { .. } => {
                            let f = match access {
                                Access::Owned => quote!(take_handle),
                                Access::Shared => quote!(borrow_handle),
                                Access::Exclusive => quote!(borrow_handle_mut),
                            };
                            Input::fallible(wires()?, quote!(#rt::#f::<#t>(#w)?))
                        }
                        ClassKind::Enum => {
                            let arms = self.enum_arms(c)?;
                            let pats = arms
                                .iter()
                                .map(|(n, d)| quote!(#d => ::core::result::Result::Ok(#t::#n)));
                            let msg = format!("invalid value {{}} for enum `{}`", c.rust);
                            Input::fallible(
                                wires()?,
                                quote!((match #w { #(#pats,)* __v => ::core::result::Result::Err(::std::format!(#msg, __v)) })?),
                            )
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
                            record_in(Record::Struct(s), &t, parts)
                        }
                        ClassKind::Sealed { .. } => {
                            let tag = leaf_ident(root, "_tag");
                            let mut wires = vec![Wire::new(tag.clone(), Prim::I.rs())];
                            let mut arms = Vec::new();
                            for (i, alt) in self.variant_of(c)?.alternatives.iter().enumerate() {
                                let aseg = alt_seg(alt);
                                let mut parts = Vec::new();
                                for f in &alt.fields {
                                    let r = join(root, &join(&aseg, &field_seg(f)));
                                    parts.push(self.rs_decode(&f.ty, &r, depth)?);
                                }
                                let an = &alt.name;
                                let built = record_in(Record::Alt(alt), &quote!(#t::#an), parts);
                                wires.extend(built.wires);
                                let e = built.expr;
                                let i = i as i32;
                                arms.push(quote!(#i => ::core::result::Result::Ok(#e)));
                            }
                            let msg = format!("{}: invalid tag {{}}", c.name);
                            Input::fallible(
                                wires,
                                quote!((match #tag { #(#arms,)* __t => ::core::result::Result::Err(::std::format!(#msg, __t)) })?),
                            )
                        }
                    }
                }
            },
            Shape::Option(inner) => {
                let inner_leaves = self.leaves(inner, Dir::In)?;
                let i = self.rs_decode(inner, root, depth)?;
                if inner_leaves.len() == 1 && inner_leaves[0].is_obj() {
                    let n = i.wires[0].name.clone();
                    Input::optional(None, quote!(!#n.is_null()), i)
                } else {
                    let present = leaf_ident(root, "_present");
                    Input::optional(
                        Some(Wire::new(present.clone(), Prim::Z.rs())),
                        quote!((#present != 0)),
                        i,
                    )
                }
            }
            Shape::Seq { elem, .. } => {
                let n = leaf_ident(root, "_n");
                let er = format!("__e{depth}");
                let elem_in = self.rs_decode(elem, &er, depth + 1)?;
                let mut wires = vec![Wire::new(n.clone(), Prim::I.rs())];
                let mut reads = Vec::new();
                let mut binds = Vec::new();
                let mut drops = Vec::new();
                for (k, l) in self.leaves(elem, Dir::In)?.iter().enumerate() {
                    let col = leaf_ident(root, &l.name);
                    wires.push(Wire::new(col.clone(), l.column().rs()));
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
                Input::fallible(
                    wires,
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
            Shape::Callback(args) => {
                Input::fallible(wires()?, self.callback_closure(ty, args, &w)?)
            }
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
    ) -> Res<Output> {
        self.rs_encode_shape(ty, self.shape(ty)?, value, root, depth)
    }

    fn rs_encode_shape(
        &self,
        ty: &TypeRef,
        shape: Shape<'_, &Setting>,
        value: TokenStream,
        root: &str,
        depth: usize,
    ) -> Res<Output> {
        let rt = rt();
        let single = |e: TokenStream, fallible: bool| -> Res<Output> {
            let leaves = self.leaves(ty, Dir::Out)?;
            let w = Wire::new(leaf_ident(root, &leaves[0].name), leaves[0].rs());
            Ok(if fallible {
                Output::fallible(vec![w], e)
            } else {
                Output::single(w, e)
            })
        };
        let access = Access::of(ty);
        Ok(match shape {
            Shape::Unit => Output::none(value),
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
                Setting::Converted(c) => {
                    let stage = c.resolved.output.as_ref().ok_or_else(|| {
                        crate::Error(format!("convert!({}) has no output", c.name))
                    })?;
                    let r = format_ident!("__r{}", depth);
                    let repr = self.rs_encode(&stage.repr, r.to_token_stream(), root, depth + 1)?;
                    stage.encode(
                        &self.q,
                        &c.resolved.target,
                        &value,
                        &r,
                        domain_check(c, &r),
                        repr,
                    )
                }
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
                            record_out(Record::Struct(self.struct_of(c)?), &t, &value, |f, b| {
                                self.rs_encode(
                                    &f.ty,
                                    b.clone(),
                                    &join(root, &field_seg(f)),
                                    depth + 1,
                                )
                            })?
                        }
                        ClassKind::Sealed { .. } => self.encode_sum(c, &t, value, root, depth)?,
                    }
                }
            },
            Shape::Option(inner) => {
                let inner_leaves = self.leaves(inner, Dir::Out)?;
                let x = format_ident!("__x{}", depth);
                let inner_out = self.rs_encode(inner, x.to_token_stream(), root, depth + 1)?;
                let e = &inner_out.expr;
                if inner_leaves.len() == 1 && inner_leaves[0].is_obj() {
                    Output {
                        expr: quote!(match #value {
                            ::core::option::Option::Some(#x) => #e,
                            ::core::option::Option::None => #rt::jni::objects::JObject::null(),
                        }),
                        ..inner_out
                    }
                } else if inner_leaves.len() == 1 {
                    let p = inner_leaves[0].prim().expect("a primitive leaf");
                    let boxer = format_ident!("{}", p.box_helper());
                    let w = Wire::new(leaf_ident(root, ""), quote!(#rt::jni::objects::JObject<'a>));
                    Output::fallible(
                        vec![w],
                        quote!(match #value {
                            ::core::option::Option::Some(#x) => #rt::#boxer(env, #e)?,
                            ::core::option::Option::None => #rt::jni::objects::JObject::null(),
                        }),
                    )
                } else {
                    let present = leaf_ident(root, "_present");
                    let mut wires = vec![Wire::new(present, Prim::Z.rs())];
                    wires.extend(inner_out.wires.clone());
                    let defaults: Vec<TokenStream> =
                        inner_leaves.iter().map(Leaf::rs_default).collect();
                    let bind = inner_out.bind();
                    let names: Vec<&syn::Ident> = inner_out.wires.iter().map(|w| &w.name).collect();
                    Output {
                        wires,
                        expr: quote!(match #value {
                            ::core::option::Option::Some(#x) => { #bind (1u8, #(#names),*) }
                            ::core::option::Option::None => (0u8, #(#defaults),*),
                        }),
                        fallible: inner_out.fallible,
                    }
                }
            }
            Shape::Seq { elem, access, .. } => {
                let n = leaf_ident(root, "_n");
                let er = format!("__e{depth}");
                let x = format_ident!("__x{}", depth);
                let elem_out = self.rs_encode(elem, x.to_token_stream(), &er, depth + 1)?;
                let mut wires = vec![Wire::new(n, Prim::I.rs())];
                let mut setup = Vec::new();
                let mut pushes = Vec::new();
                let mut finals = vec![quote!(__n as i32)];
                for (k, l) in self.leaves(elem, Dir::Out)?.iter().enumerate() {
                    wires.push(Wire::new(leaf_ident(root, &l.name), l.column().rs()));
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
                Output::fallible(
                    wires,
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

    /// A sum → its tag and every alternative's wires; an arm fills its own
    /// and defaults the rest.
    fn encode_sum(
        &self,
        c: &crate::plan::Class,
        head: &TokenStream,
        value: TokenStream,
        root: &str,
        depth: usize,
    ) -> Res<Output> {
        let v = self.variant_of(c)?;
        let tag = leaf_ident(root, "_tag");
        let mut groups: Vec<Vec<Leaf>> = Vec::new();
        for alt in &v.alternatives {
            let aseg = alt_seg(alt);
            let mut g = Vec::new();
            for f in &alt.fields {
                let seg = join(root, &join(&aseg, &field_seg(f)));
                g.extend(
                    self.leaves(&f.ty, Dir::Out)?
                        .into_iter()
                        .map(|l| l.under(&seg)),
                );
            }
            groups.push(g);
        }
        let mut wires = vec![Wire::new(tag, Prim::I.rs())];
        for g in &groups {
            wires.extend(g.iter().map(|l| Wire::new(names::ident(&l.name), l.rs())));
        }
        let mut arms = Vec::new();
        let mut fallible = false;
        for (i, alt) in v.alternatives.iter().enumerate() {
            let aseg = alt_seg(alt);
            let record = Record::Alt(alt);
            let binds = record.binds();
            let an = &alt.name;
            let pat = record.pattern(&quote!(#head::#an), &binds);
            let mut outs = Vec::new();
            for (f, b) in alt.fields.iter().zip(&binds) {
                let seg = join(root, &join(&aseg, &field_seg(f)));
                outs.push(self.rs_encode(&f.ty, b.to_token_stream(), &seg, depth + 1)?);
            }
            fallible |= outs.iter().any(|o| o.fallible);
            let out_binds = outs.iter().map(Output::bind);
            let tag = i as i32;
            let mut values = vec![quote!(#tag)];
            for (j, g) in groups.iter().enumerate() {
                for l in g {
                    values.push(if j == i {
                        names::ident(&l.name).to_token_stream()
                    } else {
                        l.rs_default()
                    });
                }
            }
            arms.push(quote!(#pat => { #(#out_binds)* (#(#values),*) }));
        }
        Ok(Output {
            wires,
            expr: quote!(match #value { #(#arms),* }),
            fallible,
        })
    }
}

/// The check a converted value's declared domain puts on its
/// representation, bound to `r`.
fn domain_check(c: &Conv, r: &syn::Ident) -> TokenStream {
    match c.range {
        Some((lo, hi)) => {
            let name = &c.name;
            quote!(let #r = ::prebindgen_jni_runtime::check_domain(#r, #lo, #hi, #name)?;)
        }
        None => TokenStream::new(),
    }
}
