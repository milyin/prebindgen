//! Input selectors: a parameter supplied in one of several ways, chosen at
//! run time (`expand_param!(T).variant(fun!(ctor)).variant_self()`).
//!
//! Kotlin passes a selector `Int` plus every variant's arguments side by
//! side, the inactive ones `null`: a constructor's arguments, each nullable,
//! or the handle itself. Rust builds the value with the constructor, or
//! takes (or borrows) the handle. `split_on_param` adds one typed overload
//! per variant.
//!
//! An expansion with a single constructor and no handle variant is
//! *direct*: there is nothing to choose, so Kotlin passes the constructor's
//! arguments as the parameter and no selector crosses. An optional
//! parameter is direct only when its absence shows in the arguments — the
//! constructor takes at least one, and none is itself optional; otherwise it
//! keeps the selector, whose `-1` means `None`.

use std::rc::Rc;

use prebindgen_flat::flat::{Function, Param as FlatParam, TypeKind};
use prebindgen_tools::{names, shape, Access, Form, FormKind, Input, Seg, Shape, Wire};
use proc_macro2::TokenStream;
use quote::quote;

use super::{
    kotlin::{HandleSite, KtEnc},
    kt_ident,
    leaf::{Leaf, LeafTy, Prim},
    leaf_ident, Dir, Param,
};
use crate::{
    decl::{ExpandParamDecl, ParamVariant},
    plan::{err, Class, Plan, Res},
};

/// A resolved selector parameter.
#[derive(Clone)]
pub(crate) struct Selector {
    pub param: FlatParam,
    pub class: Rc<Class>,
    pub variants: Vec<SelVariant>,
    /// `Option<..>`: selector `-1` is `None`.
    pub optional: bool,
    /// How the callee takes the value.
    pub access: Access,
}

#[derive(Clone)]
#[allow(clippy::large_enum_variant)] // one per selector variant
pub(crate) enum SelVariant {
    Build { func: Function, callee: TokenStream },
    Handle,
}

impl Selector {
    /// A single constructor and nothing else, and — for an optional
    /// parameter — arguments that can say the value is absent: no selector
    /// crosses.
    pub(crate) fn is_direct(&self) -> bool {
        let [SelVariant::Build { func, .. }] = self.variants.as_slice() else {
            return false;
        };
        !self.optional
            || (!func.params.is_empty()
                && func
                    .params
                    .iter()
                    .all(|p| !matches!(p.ty.kind(), TypeKind::Optional(_))))
    }
}

/// The Kotlin name of argument `j` of variant `i` of selector parameter
/// `p`: `p1` for a one-argument constructor, `p01`, `p02` otherwise.
fn arg_name(p: &str, i: usize, j: usize, arity: usize) -> String {
    if arity == 1 {
        format!("{p}{i}")
    } else {
        format!("{p}{i}{j}")
    }
}

impl Plan<'_> {
    /// The selector for `param`, if an input expansion applies to it.
    pub(crate) fn selector(
        &self,
        param: &FlatParam,
        explicit: Option<&ExpandParamDecl>,
    ) -> Res<Option<Selector>> {
        let (core, optional) = match param.ty.kind() {
            TypeKind::Optional(inner) => (&**inner, true),
            _ => (&param.ty, false),
        };
        let (name, access) = match shape(core, |candidate| {
            super::bare_declared_name(candidate).and_then(|name| self.types.get(name))
        }) {
            Ok(Shape::Declared { ty, .. }) => (
                super::declared_name(ty).expect("a declared class name"),
                Access::of(ty),
            ),
            Ok(Shape::Undeclared(name)) => (name, Access::Owned),
            _ => return Ok(None),
        };
        if access == Access::Exclusive {
            return Ok(None);
        }
        let decl = match explicit {
            Some(d) => d,
            None => match self.param_exp.get(name) {
                Some(d) => d,
                None => return Ok(None),
            },
        };
        if decl.variants.len() == 1 && matches!(decl.variants[0], ParamVariant::Handle) {
            return Ok(None);
        }
        let Some(class) = self.class(name).cloned() else {
            return err(format!(
                "`{name}`: an input expansion needs a declared class"
            ));
        };
        let mut variants = Vec::new();
        for v in &decl.variants {
            variants.push(match v {
                ParamVariant::Build(f) => SelVariant::Build {
                    func: f.fun.resolve(self.flat).map_err(crate::Error)?,
                    callee: f.fun.callee(&self.q),
                },
                ParamVariant::Handle => SelVariant::Handle,
            });
        }
        Ok(Some(Selector {
            param: param.clone(),
            class,
            variants,
            optional,
            access,
        }))
    }

    /// The Kotlin parameters of a selector: `pSel`, then each variant's.
    pub(crate) fn selector_params(&self, s: &Selector) -> Res<Vec<(String, String)>> {
        let p = kt_ident(&names::camel(&names::bare(&s.param.name)));
        if let (true, [SelVariant::Build { func, .. }]) = (s.is_direct(), s.variants.as_slice()) {
            let n = func.params.len();
            return func
                .params
                .iter()
                .enumerate()
                .map(|(j, fp)| {
                    let name = if n == 1 { p.clone() } else { format!("{p}{j}") };
                    let t = self.kt_type(&fp.ty)?;
                    Ok((name, if s.optional { nullable(t) } else { t }))
                })
                .collect();
        }
        let mut out = vec![(format!("{p}Sel"), "Int".to_string())];
        for (i, v) in s.variants.iter().enumerate() {
            match v {
                SelVariant::Build { func, .. } => {
                    let n = func.params.len();
                    for (j, fp) in func.params.iter().enumerate() {
                        let t = self.kt_type(&fp.ty)?;
                        out.push((arg_name(&p, i, j, n), nullable(t)));
                    }
                }
                SelVariant::Handle => out.push((format!("{p}{i}"), format!("{}?", s.class.fqn()))),
            }
        }
        Ok(out)
    }

    /// The raw leaves of a selector.
    pub(crate) fn selector_leaves(&self, s: &Selector) -> Res<Vec<Leaf>> {
        let root = names::bare(&s.param.name);
        let mut out = Vec::new();
        if !s.is_direct() {
            out.push(Leaf::new(LeafTy::Prim(Prim::I)).under(&format!("{root}_sel")));
        }
        for (i, v) in s.variants.iter().enumerate() {
            match v {
                SelVariant::Build { func, .. } => {
                    for (j, fp) in func.params.iter().enumerate() {
                        let opt = self.variant_arg_ty(s, &fp.ty);
                        out.extend(
                            self.leaves(&opt, Dir::In)?
                                .into_iter()
                                .map(|l| l.under(&format!("{root}_{i}{j}"))),
                        );
                    }
                }
                SelVariant::Handle => {
                    out.push(Leaf::new(LeafTy::Prim(Prim::J)).under(&format!("{root}_{i}")))
                }
            }
        }
        Ok(out)
    }

    /// Kotlin leaf expressions for a selector, from its Kotlin parameters.
    pub(crate) fn selector_encode(&self, s: &Selector, cx: &mut KtEnc) -> Res<Vec<String>> {
        let p = kt_ident(&names::camel(&names::bare(&s.param.name)));
        if s.is_direct() {
            let names: Vec<String> = self
                .selector_params(s)?
                .into_iter()
                .map(|(n, _)| n)
                .collect();
            let [SelVariant::Build { func, .. }] = s.variants.as_slice() else {
                unreachable!("a direct selector");
            };
            let mut out = Vec::new();
            for (fp, n) in func.params.iter().zip(&names) {
                let ty = self.variant_arg_ty(s, &fp.ty);
                out.extend(self.kt_encode(&ty, n, false, cx, true)?);
            }
            return Ok(out);
        }
        let mut out = vec![format!("{p}Sel")];
        for (i, v) in s.variants.iter().enumerate() {
            match v {
                SelVariant::Build { func, .. } => {
                    let n = func.params.len();
                    for (j, fp) in func.params.iter().enumerate() {
                        let opt = self.variant_arg_ty(s, &fp.ty);
                        out.extend(self.kt_encode(
                            &opt,
                            &arg_name(&p, i, j, n),
                            false,
                            cx,
                            true,
                        )?);
                    }
                }
                SelVariant::Handle => {
                    let e = format!("{p}{i}");
                    out.extend(self.kt_encode_handle(&s.class, &e, s.access == Access::Owned, cx));
                }
            }
        }
        Ok(out)
    }

    fn kt_encode_handle(
        &self,
        class: &Rc<Class>,
        expr: &str,
        consumed: bool,
        cx: &mut KtEnc,
    ) -> Vec<String> {
        cx.handles.push(HandleSite {
            expr: expr.to_string(),
            nullable: true,
            consumed,
            class: class.clone(),
        });
        vec![format!("({expr}?.ptr ?: 0L)")]
    }

    /// How argument type `ty` of a variant crosses: optional, unless the
    /// selector is direct and not optional itself.
    fn variant_arg_ty(
        &self,
        s: &Selector,
        ty: &prebindgen_flat::flat::TypeRef,
    ) -> prebindgen_flat::flat::TypeRef {
        let already = matches!(ty.kind(), TypeKind::Optional(_));
        if already || (s.is_direct() && !s.optional) {
            ty.clone()
        } else {
            ty.optional()
        }
    }

    /// The Rust input of a direct selector: the constructor called on its
    /// arguments (`None` when an optional parameter's are absent).
    fn direct_input(&self, s: &Selector, func: &Function, callee: &TokenStream) -> Res<Param> {
        let root = names::bare(&s.param.name);
        let mut parts = Vec::new();
        let mut args = Vec::new();
        let mut binds = Vec::new();
        for (j, fp) in func.params.iter().enumerate() {
            let ty = self.variant_arg_ty(s, &fp.ty);
            let input = self.rs_decode(&ty, &format!("{root}_0{j}"), 0)?;
            let e = input.result();
            parts.push((Seg::Param(names::bare(&fp.name)), input.form));
            let a = quote::format_ident!("__a{}", j);
            binds.push(quote!(let #a = #e?;));
            args.push(a);
        }
        let call = |args: &[syn::Ident]| match func.ret.kind() {
            TypeKind::Fallible { .. } => {
                quote!(#callee(#(#args),*).map_err(|__e| ::std::string::ToString::to_string(&__e))?)
            }
            _ => quote!(#callee(#(#args),*)),
        };
        let built = call(&args);
        let wrap = |b: TokenStream| match s.access {
            Access::Owned => b,
            _ => quote!(::prebindgen_jni_runtime::MaybeOwned::Owned(#b)),
        };
        let expr = if s.optional {
            let some = wrap(built);
            quote!({
                #(#binds)*
                match (#(#args,)*) {
                    (#(::core::option::Option::Some(#args),)*) => ::core::option::Option::Some(#some),
                    _ => ::core::option::Option::None,
                }
            })
        } else {
            let v = wrap(built);
            quote!({ #(#binds)* #v })
        };
        // The parameter is built from the constructor's arguments.
        let form = Form {
            ty: s.param.ty.clone(),
            kind: FormKind::Parts(parts),
        };
        Ok(selector_param(s, form, expr))
    }

    /// The Rust input of a selector parameter.
    pub(crate) fn selector_input(&self, s: &Selector) -> Res<Param> {
        if let (true, [SelVariant::Build { func, callee }]) = (s.is_direct(), s.variants.as_slice())
        {
            return self.direct_input(s, func, callee);
        }
        let root = names::bare(&s.param.name);
        let sel = leaf_ident(&root, "sel");
        let t = self.q.path(&names::ident(&s.class.rust));
        let tag = Wire::new(sel.clone(), Leaf::new(LeafTy::Prim(Prim::I)));
        let mut alts = Vec::new();
        let mut arms = Vec::new();
        for (i, v) in s.variants.iter().enumerate() {
            let body = match v {
                SelVariant::Build { func, callee } => {
                    let mut args = Vec::new();
                    let mut parts = Vec::new();
                    for (j, fp) in func.params.iter().enumerate() {
                        let r = format!("{root}_{i}{j}");
                        let input = self.rs_decode(&self.variant_arg_ty(s, &fp.ty), &r, 0)?;
                        let e = input.result();
                        parts.push((Seg::Param(names::bare(&fp.name)), input.form));
                        // An optional argument's `None` is its value, not a
                        // missing argument.
                        if let TypeKind::Optional(_) = fp.ty.kind() {
                            args.push(quote!(#e?));
                            continue;
                        }
                        let pname = names::bare(&fp.name);
                        let msg = format!("missing argument `{pname}` for `{root}` variant {i}");
                        args.push(quote!(#e?.ok_or_else(|| ::std::string::String::from(#msg))?));
                    }
                    let built = match func.ret.kind() {
                        TypeKind::Fallible { .. } => {
                            quote!(#callee(#(#args),*).map_err(|__e| ::std::string::ToString::to_string(&__e))?)
                        }
                        _ => quote!(#callee(#(#args),*)),
                    };
                    alts.push(parts);
                    match s.access {
                        Access::Owned => built,
                        _ => quote!(::prebindgen_jni_runtime::MaybeOwned::Owned(#built)),
                    }
                }
                SelVariant::Handle => {
                    let w = leaf_ident(&root, &i.to_string());
                    // The value itself, as an existing handle.
                    let handle = Wire::new(w.clone(), Leaf::new(LeafTy::Prim(Prim::J)));
                    let form = Form {
                        ty: s.param.ty.clone(),
                        kind: FormKind::Wire(handle),
                    };
                    alts.push(vec![(Seg::Param("self".into()), form)]);
                    match s.access {
                        Access::Owned => quote!(::prebindgen_jni_runtime::take_handle::<#t>(#w)?),
                        _ => quote!(::prebindgen_jni_runtime::MaybeOwned::Borrowed(
                            ::prebindgen_jni_runtime::borrow_handle::<#t>(#w)?
                        )),
                    }
                }
            };
            let i = i as i32;
            arms.push(if s.optional {
                quote!(#i => ::core::option::Option::Some(#body))
            } else {
                quote!(#i => #body)
            });
        }
        if s.optional {
            arms.push(quote!(-1 => ::core::option::Option::None));
        }
        let msg = format!("invalid selector {{}} for `{root}`");
        let expr = quote!(match #sel {
            #(#arms,)*
            __s => return ::core::result::Result::Err(::std::format!(#msg, __s)),
        });
        // A selector is a tag choosing among the variants.
        let form = Form {
            ty: s.param.ty.clone(),
            kind: FormKind::Sum { tag, alts },
        };
        Ok(selector_param(s, form, expr))
    }

    /// The typed overloads of a selector-form function, one per combination
    /// of the split parameters' variants.
    pub(crate) fn split_overloads(
        &self,
        head: &str,
        name: &str,
        params: &[SigParam],
        tail: &[(String, String)],
        ret: &str,
        prefix_names: bool,
    ) -> Res<Vec<(String, Vec<String>)>> {
        let mut combos: Vec<Vec<(usize, usize)>> = vec![Vec::new()];
        for (pi, p) in params.iter().enumerate() {
            if let SigParam::Split(s) = p {
                let mut next = Vec::new();
                for c in &combos {
                    for vi in 0..s.variants.len() {
                        let mut c2 = c.clone();
                        c2.push((pi, vi));
                        next.push(c2);
                    }
                }
                combos = next;
            }
        }
        let mut out = Vec::new();
        for combo in combos {
            let mut sig = Vec::new();
            let mut types = Vec::new();
            let mut args = Vec::new();
            for (pi, p) in params.iter().enumerate() {
                match p {
                    SigParam::Plain(n, t) => {
                        sig.push(format!("{n}: {t}"));
                        types.push(t.clone());
                        args.push(n.clone());
                    }
                    SigParam::Split(s) => {
                        let vi = combo.iter().find(|(i, _)| *i == pi).unwrap().1;
                        let base = kt_ident(&names::camel(&names::bare(&s.param.name)));
                        args.push(vi.to_string());
                        for (i, v) in s.variants.iter().enumerate() {
                            match v {
                                SelVariant::Build { func, .. } => {
                                    for fp in &func.params {
                                        if i == vi {
                                            let pn = names::bare(&fp.name);
                                            let n = if prefix_names {
                                                kt_ident(&names::camel(&format!(
                                                    "{}_{pn}",
                                                    names::bare(&s.param.name)
                                                )))
                                            } else {
                                                kt_ident(&names::camel(&pn))
                                            };
                                            let t = self.kt_type(&fp.ty)?;
                                            sig.push(format!("{n}: {t}"));
                                            types.push(t);
                                            args.push(n);
                                        } else {
                                            args.push("null".to_string());
                                        }
                                    }
                                }
                                SelVariant::Handle => {
                                    if i == vi {
                                        sig.push(format!("{base}: {}", s.class.fqn()));
                                        types.push(s.class.fqn());
                                        args.push(base.clone());
                                    } else {
                                        args.push("null".to_string());
                                    }
                                }
                            }
                        }
                    }
                    SigParam::Selector(s) => {
                        for (n, t) in self.selector_params(s)? {
                            sig.push(format!("{n}: {t}"));
                            types.push(t);
                            args.push(n);
                        }
                    }
                }
            }
            for (n, t) in tail {
                sig.push(format!("{n}: {t}"));
                types.push(t.clone());
                args.push(n.clone());
            }
            let text = format!(
                "{head}{name}({}): {ret} =\n    {name}({})\n",
                sig.join(", "),
                args.join(", ")
            );
            out.push((text, types));
        }
        Ok(out)
    }
}

/// A Kotlin wrapper parameter: plain, a selector group, or a selector group
/// that is split into overloads.
pub(crate) enum SigParam {
    Plain(String, String),
    Selector(Selector),
    Split(Selector),
}

fn nullable(t: String) -> String {
    if t.ends_with('?') {
        t
    } else {
        format!("{t}?")
    }
}

/// A selector parameter's input over `form`, and how its value is lent.
fn selector_param(s: &Selector, form: Form<Leaf>, expr: TokenStream) -> Param {
    let name = &s.param.name;
    let pass = match (s.access, s.optional) {
        (Access::Owned, _) => None,
        (_, false) => Some(quote!(&*#name)),
        (_, true) => Some(quote!(#name.as_deref())),
    };
    Param {
        input: Input {
            form,
            expr,
            fallible: true,
        },
        pass,
    }
}
