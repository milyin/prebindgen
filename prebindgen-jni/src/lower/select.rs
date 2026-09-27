//! Input selectors: a parameter supplied in one of several ways, chosen at
//! run time (`expand_param!(T).variant(fun!(ctor)).variant_self()`).
//!
//! Kotlin passes a selector `Int` plus every variant's arguments side by
//! side, the inactive ones `null`: a constructor's arguments, each nullable,
//! or the handle itself. Rust builds the value with the constructor, or
//! takes (or borrows) the handle. `split_on_param` adds one typed overload
//! per variant.

use std::rc::Rc;

use prebindgen_tools::{
    flat::flat::{Function, Param as FlatParam, TypeKind},
    names, shape, Access, Input, Shape, Wire,
};
use proc_macro2::TokenStream;
use quote::quote;

use super::{
    kotlin::{HandleSite, KtEnc},
    kt_ident,
    leaf::{Leaf, LeafTy, Prim},
    leaf_ident, Dir,
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
        let (name, access) = match shape(core, |n| self.types.get(n)) {
            Ok(Shape::Declared { name, access, .. }) => (name, access),
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
        let mut out = vec![(format!("{p}Sel"), "Int".to_string())];
        for (i, v) in s.variants.iter().enumerate() {
            match v {
                SelVariant::Build { func, .. } => {
                    for (j, fp) in func.params.iter().enumerate() {
                        let t = self.kt_type(&fp.ty)?;
                        out.push((format!("{p}{i}{j}"), nullable(t)));
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
        let mut out = vec![Leaf::new(LeafTy::Prim(Prim::I)).under(&format!("{root}_sel"))];
        for (i, v) in s.variants.iter().enumerate() {
            match v {
                SelVariant::Build { func, .. } => {
                    for (j, fp) in func.params.iter().enumerate() {
                        let opt = fp.ty.optional();
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
        let mut out = vec![format!("{p}Sel")];
        for (i, v) in s.variants.iter().enumerate() {
            match v {
                SelVariant::Build { func, .. } => {
                    for (j, fp) in func.params.iter().enumerate() {
                        let opt = fp.ty.optional();
                        out.extend(self.kt_encode(&opt, &format!("{p}{i}{j}"), false, cx, true)?);
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

    /// The Rust input of a selector parameter.
    pub(crate) fn selector_input(&self, s: &Selector) -> Res<Input> {
        let root = names::bare(&s.param.name);
        let sel = leaf_ident(&root, "sel");
        let t = self.q.path(&names::ident(&s.class.rust));
        let mut wires = vec![Wire::new(sel.clone(), Prim::I.rs())];
        let mut arms = Vec::new();
        for (i, v) in s.variants.iter().enumerate() {
            let body = match v {
                SelVariant::Build { func, callee } => {
                    let mut args = Vec::new();
                    for (j, fp) in func.params.iter().enumerate() {
                        let opt = fp.ty.optional();
                        let r = format!("{root}_{i}{j}");
                        let input = self.rs_decode(&opt, &r, 0)?;
                        wires.extend(input.wires.clone());
                        let e = input.result();
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
                    match s.access {
                        Access::Owned => built,
                        _ => quote!(::prebindgen_jni_runtime::MaybeOwned::Owned(#built)),
                    }
                }
                SelVariant::Handle => {
                    let w = leaf_ident(&root, &i.to_string());
                    wires.push(Wire::new(w.clone(), Prim::J.rs()));
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
        let name = &s.param.name;
        let expr = quote!(match #sel {
            #(#arms,)*
            __s => return ::core::result::Result::Err(::std::format!(#msg, __s)),
        });
        let input = Input::fallible(wires, expr);
        Ok(match (s.access, s.optional) {
            (Access::Owned, _) => input,
            (_, false) => input.with_pass(quote!(&*#name)),
            (_, true) => input.with_pass(quote!(#name.as_deref())),
        })
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
    ) -> Res<Vec<String>> {
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
            let mut args = Vec::new();
            for (pi, p) in params.iter().enumerate() {
                match p {
                    SigParam::Plain(n, t) => {
                        sig.push(format!("{n}: {t}"));
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
                                            sig.push(format!("{n}: {}", self.kt_type(&fp.ty)?));
                                            args.push(n);
                                        } else {
                                            args.push("null".to_string());
                                        }
                                    }
                                }
                                SelVariant::Handle => {
                                    if i == vi {
                                        sig.push(format!("{base}: {}", s.class.fqn()));
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
                            args.push(n);
                        }
                    }
                }
            }
            for (n, t) in tail {
                sig.push(format!("{n}: {t}"));
                args.push(n.clone());
            }
            out.push(format!(
                "{head}{name}({}): {ret} =\n    {name}({})\n",
                sig.join(", "),
                args.join(", ")
            ));
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
