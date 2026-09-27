//! Output deliveries: a value handed to Kotlin as a *list of parameters* —
//! a builder's, a callback's, an error handler's.
//!
//! A value delivers as one typed parameter unless its type has an output
//! expansion (`expand_return!`): then it delivers as its fields, each field
//! itself a delivery — spliced when the field's type has an expansion of its
//! own, inlined when it comes from a value form (`fields!`) and is a data
//! class, typed otherwise. An optional value with an expansion delivers its
//! fields gated by one presence flag: every one of them `null` when absent.

use prebindgen_tools::{
    flat::flat::{Struct, Type as FlatType, TypeKind, TypeRef},
    names, Output, Record, Wire,
};
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};

use super::{
    codec::{kt_ident, Dir, Kind},
    err,
    leaf::{join, method_desc, Leaf},
    Gen, Res,
};
use crate::decl::{ExpandReturnDecl, ReturnField};

/// One Kotlin-visible parameter of a delivery.
#[derive(Clone, Debug)]
pub(crate) struct Param {
    pub name: String,
    pub kt: String,
    /// Builds the parameter's value from the raw leaves (named by
    /// [`raw_name`]).
    pub decode: String,
    /// Closes the value after a callback returns, when it owns handles.
    pub close: Option<String>,
}

/// A delivered value: its parameters, its raw leaves and the Rust encoding.
pub(crate) struct Delivery {
    pub params: Vec<Param>,
    pub leaves: Vec<Leaf>,
    pub output: Output,
}

/// The Kotlin name of a raw leaf parameter.
pub(crate) fn raw_name(l: &Leaf) -> String {
    let n = names::camel(&l.name);
    kt_ident(if n.is_empty() { "value" } else { &n })
}

fn gate(gates: &[String], decode: String) -> String {
    if gates.is_empty() {
        decode
    } else {
        format!("(if ({}) {decode} else null)", gates.join(" && "))
    }
}

fn nullable(kt: String, gated: bool) -> String {
    if gated && !kt.ends_with('?') {
        format!("{kt}?")
    } else {
        kt
    }
}

impl<'a> Gen<'a> {
    /// The expansion that applies to a value of `ty`: the explicit one, else
    /// the type's own; `None` when it would only hand over the value itself.
    pub(crate) fn expansion<'e>(
        &'e self,
        ty: &TypeRef,
        explicit: Option<&'e ExpandReturnDecl>,
    ) -> Option<&'e ExpandReturnDecl> {
        let e = match explicit {
            Some(e) => e,
            None => match ty.kind() {
                TypeKind::Named { id, .. } => self.ret_exp.get(&id.name)?,
                _ => return None,
            },
        };
        let identity = e.fields.len() == 1 && matches!(e.fields[0], ReturnField::Handle);
        (!identity).then_some(e)
    }

    /// `ty` under its transparent layers and borrows: the core type, the
    /// value expression reaching it, and whether that value is borrowed.
    fn peel<'t>(&self, ty: &'t TypeRef, value: TokenStream) -> (&'t TypeRef, TokenStream, bool) {
        match ty.kind() {
            TypeKind::Boxed(inner) => {
                let (t, v, b) = self.peel(inner, quote!((*#value)));
                (t, v, b)
            }
            TypeKind::Ref { inner, .. } => {
                let (t, v, _) = self.peel(inner, value);
                (t, v, true)
            }
            _ => (ty, value, false),
        }
    }

    /// Deliver `value` (of `ty`) as parameters named under `name`, with raw
    /// leaves under `root`.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn deliver(
        &self,
        ty: &TypeRef,
        value: TokenStream,
        root: &str,
        name: &str,
        explicit: Option<&ExpandReturnDecl>,
        gates: &[String],
        inline: bool,
        depth: usize,
    ) -> Res<Delivery> {
        let (core, v, borrowed) = self.peel(ty, value.clone());
        // An optional value whose inner type expands: one presence gate.
        if let TypeKind::Optional(inner) = core.kind() {
            let (inner_core, _, _) = self.peel(inner, quote!(__unused));
            if self.expansion(inner_core, explicit).is_some() {
                let present = Leaf::new(super::leaf::LeafTy::Prim(super::leaf::Prim::Z))
                    .under(&join(root, "_present"));
                let x = format_ident!("__x{}", depth);
                let x_value = if borrowed { quote!(&#x) } else { quote!(#x) };
                let mut g = gates.to_vec();
                g.push(raw_name(&present));
                let inner_d =
                    self.deliver(inner, x_value, root, name, explicit, &g, false, depth + 1)?;
                let defaults: Vec<TokenStream> =
                    inner_d.leaves.iter().map(Leaf::rs_default).collect();
                let bind = inner_d.output.bind();
                let names: Vec<&syn::Ident> =
                    inner_d.output.wires.iter().map(|w| &w.name).collect();
                let pw = super::codec::leaf_ident(root, "_present");
                let mut wires = vec![Wire::new(pw, present.rs())];
                wires.extend(inner_d.output.wires.clone());
                let scrutinee = if borrowed {
                    quote!(#v.as_ref())
                } else {
                    v.clone()
                };
                let out = Output {
                    wires,
                    expr: quote!(match #scrutinee {
                        ::core::option::Option::Some(#x) => { #bind (1u8, #(#names),*) }
                        ::core::option::Option::None => (0u8, #(#defaults),*),
                    }),
                    fallible: inner_d.output.fallible,
                };
                let mut leaves = vec![present];
                leaves.extend(inner_d.leaves);
                return Ok(Delivery {
                    params: inner_d.params,
                    leaves,
                    output: out,
                });
            }
        }
        if let Some(e) = self.expansion(core, explicit) {
            return self.expanded(core, e, v, borrowed, root, name, gates, depth);
        }
        if inline {
            if let Ok(Kind::Record(_, s)) = self.kind(core) {
                let v = if borrowed {
                    quote!(::core::clone::Clone::clone(#v))
                } else {
                    v
                };
                return self.fields_of(core, s, v, root, name, gates, depth);
            }
        }
        self.plain(ty, value, root, name, gates, depth)
    }

    /// One typed parameter.
    fn plain(
        &self,
        ty: &TypeRef,
        value: TokenStream,
        root: &str,
        name: &str,
        gates: &[String],
        depth: usize,
    ) -> Res<Delivery> {
        let gated = !gates.is_empty();
        let leaves: Vec<Leaf> = self
            .leaves(ty, Dir::Out)?
            .into_iter()
            .map(|mut l| {
                l.nullable |= gated && l.is_obj();
                l.under(root)
            })
            .collect();
        let raws: Vec<String> = leaves.iter().map(raw_name).collect();
        let decode = gate(gates, self.kt_decode(ty, &raws, gated, depth)?);
        let pname = kt_ident(&names::camel(if name.is_empty() { "value" } else { name }));
        let close = self.kt_close(ty, &pname)?.map(|c| {
            if gated && !c.contains("?.") {
                c.replacen(&format!("{pname}."), &format!("{pname}?."), 1)
            } else {
                c
            }
        });
        Ok(Delivery {
            params: vec![Param {
                name: pname,
                kt: nullable(self.kt_type(ty)?, gated),
                decode,
                close,
            }],
            leaves,
            output: self.rs_encode(ty, value, root, depth)?,
        })
    }

    /// The fields of a struct value, each delivered (inlining data classes).
    #[allow(clippy::too_many_arguments)]
    fn fields_of(
        &self,
        ty: &TypeRef,
        s: &Struct,
        value: TokenStream,
        root: &str,
        name: &str,
        gates: &[String],
        depth: usize,
    ) -> Res<Delivery> {
        let head = self.q.path(&s.name);
        let _ = ty;
        let record = Record::Struct(s);
        let binds: Vec<syn::Ident> = (0..s.fields.len())
            .map(|i| format_ident!("__f{}_{}", depth, i))
            .collect();
        let pat = record.pattern(&head, &binds);
        let mut parts = Vec::new();
        for (f, b) in s.fields.iter().zip(&binds) {
            let seg = super::codec::field_seg(f);
            parts.push(self.deliver(
                &f.ty,
                b.to_token_stream(),
                &join(root, &seg),
                &join(name, &seg),
                None,
                gates,
                true,
                depth + 1,
            )?);
        }
        Ok(concat(quote!(let #pat = #value;), parts, Vec::new()))
    }

    /// A value delivered through its output expansion.
    #[allow(clippy::too_many_arguments)]
    fn expanded(
        &self,
        ty: &TypeRef,
        e: &ExpandReturnDecl,
        value: TokenStream,
        borrowed: bool,
        root: &str,
        name: &str,
        gates: &[String],
        depth: usize,
    ) -> Res<Delivery> {
        let v = format_ident!("__v{}", depth);
        let type_name = core_name(ty).unwrap_or_default();
        let type_snake = names::snake(&type_name);
        let mut parts = Vec::new();
        let mut deferred = Vec::new();
        let mut prelude = quote!(let #v = #value;);
        for field in &e.fields {
            match field {
                ReturnField::Getter(g) => {
                    let (func, callee) = self.resolve_fn(&g.fun)?;
                    let fname = g.name.clone().unwrap_or_else(|| {
                        let n = names::bare(&func.name);
                        n.strip_prefix(&format!("{type_snake}_"))
                            .unwrap_or(&n)
                            .to_string()
                    });
                    let arg = match func.params.first().map(|p| p.ty.kind()) {
                        Some(TypeKind::Ref { .. }) if borrowed => quote!(#v),
                        Some(TypeKind::Ref { .. }) => quote!(&#v),
                        Some(_) if borrowed => quote!(::core::clone::Clone::clone(#v)),
                        Some(_) => quote!(::core::clone::Clone::clone(&#v)),
                        None => return err(format!("getter `{}` takes no argument", func.name)),
                    };
                    let seg = names::snake(&fname);
                    // A field of the expanded type itself is that value, not
                    // another expansion of it.
                    if core_name(&func.ret).as_deref() == Some(type_name.as_str()) {
                        parts.push(self.plain(
                            &func.ret,
                            quote!(#callee(#arg)),
                            &join(root, &seg),
                            &join(name, &seg),
                            gates,
                            depth + 1,
                        )?);
                        continue;
                    }
                    parts.push(self.deliver(
                        &func.ret,
                        quote!(#callee(#arg)),
                        &join(root, &seg),
                        &join(name, &seg),
                        None,
                        gates,
                        false,
                        depth + 1,
                    )?);
                }
                ReturnField::Handle => {
                    let handle = if borrowed {
                        quote!(::core::clone::Clone::clone(#v))
                    } else {
                        quote!(#v)
                    };
                    let d = self.plain(
                        ty,
                        handle,
                        &join(root, "handle"),
                        &join(name, "handle"),
                        gates,
                        depth + 1,
                    )?;
                    deferred.push(parts.len());
                    parts.push(d);
                }
                ReturnField::Form { fun, consume } => {
                    let f = self.flat.function(fun).ok_or_else(|| {
                        crate::Error(format!("`{fun}` is not a #[prebindgen] function"))
                    })?;
                    let callee = self.q.path(fun);
                    let arg = match (consume, borrowed) {
                        (true, true) => quote!(::core::clone::Clone::clone(#v)),
                        (true, false) => quote!(#v),
                        (false, true) => quote!(#v),
                        (false, false) => quote!(&#v),
                    };
                    let s = match f.ret.kind() {
                        TypeKind::Named { id, .. } => match self.flat.declared_type(&id.name) {
                            Some(FlatType::Struct(s)) => s,
                            _ => return err(format!("value form `{fun}` must return a struct")),
                        },
                        _ => return err(format!("value form `{fun}` must return a struct")),
                    };
                    let sv = format_ident!("__s{}", depth);
                    prelude.extend(quote!(let #sv = #callee(#arg);));
                    let d = self.fields_of(
                        &f.ret,
                        s,
                        sv.to_token_stream(),
                        root,
                        name,
                        gates,
                        depth + 1,
                    )?;
                    parts.push(d);
                }
            }
        }
        Ok(concat(prelude, parts, deferred))
    }

    // ── callbacks ───────────────────────────────────────────────────────

    /// The deliveries of a callback's arguments, bound to `__a0`, `__a1`, …
    pub(crate) fn callback_args(&self, args: &[TypeRef]) -> Res<Vec<Delivery>> {
        args.iter()
            .enumerate()
            .map(|(i, a)| {
                let name = self.arg_name(a, i);
                self.deliver(
                    a,
                    format_ident!("__a{}", i).to_token_stream(),
                    &format!("a{i}"),
                    &name,
                    None,
                    &[],
                    false,
                    1,
                )
            })
            .collect()
    }

    fn arg_name(&self, ty: &TypeRef, i: usize) -> String {
        match self.kind(ty) {
            Ok(
                Kind::Handle { class, .. }
                | Kind::Enum(class)
                | Kind::Record(class, _)
                | Kind::Sum(class, _),
            ) => names::snake(&class.rust),
            Ok(Kind::Converted(c)) => names::snake(&c.name),
            Ok(Kind::Scalar(k)) => k.as_str().to_string(),
            _ => format!("arg{i}"),
        }
    }

    /// The base a callback interface is named from.
    fn type_base(&self, ty: &TypeRef) -> String {
        match ty.kind() {
            TypeKind::Named { id, .. } => id.name.clone(),
            TypeKind::Scalar(k) => k.as_str().to_string(),
            TypeKind::String | TypeKind::Str => "String".to_string(),
            TypeKind::Ref { inner, .. } | TypeKind::Boxed(inner) | TypeKind::Cow { inner, .. } => {
                self.type_base(inner)
            }
            TypeKind::Vec(e) | TypeKind::Slice(e) => format!("{}List", self.type_base(e)),
            TypeKind::Optional(t) => format!("Optional{}", self.type_base(t)),
            TypeKind::Array { elem, .. } => format!("{}Array", self.type_base(elem)),
            _ => "Value".to_string(),
        }
    }

    fn callback_base(&self, ty: &TypeRef) -> Res<String> {
        let TypeKind::Callback { args } = ty.kind() else {
            return err(format!("`{ty}` is not a callback"));
        };
        if args.is_empty() {
            return Ok("Unit".to_string());
        }
        Ok(args
            .iter()
            .map(|a| self.type_base(a))
            .collect::<Vec<_>>()
            .join(""))
    }

    /// The user-facing callback interface of `ty`.
    pub(crate) fn callback_fqn(&self, ty: &TypeRef) -> Res<String> {
        Ok(format!(
            "{}.{}Callback",
            self.base_pkg,
            self.callback_base(ty)?
        ))
    }

    /// The raw callback interface of `ty`, the one Rust invokes.
    pub(crate) fn callback_raw_fqn(&self, ty: &TypeRef) -> Res<String> {
        Ok(format!(
            "{}.{}CallbackRaw",
            self.base_pkg,
            self.callback_base(ty)?
        ))
    }

    /// The Rust closure a callback object becomes.
    pub(crate) fn callback_closure(
        &self,
        ty: &TypeRef,
        args: &[TypeRef],
        w: &syn::Ident,
    ) -> Res<TokenStream> {
        let deliveries = self.callback_args(args)?;
        let leaves: Vec<Leaf> = deliveries.iter().flat_map(|d| d.leaves.clone()).collect();
        let desc = method_desc(&leaves, "V");
        let frame = 32 + leaves.len() as i32 * 2;
        let arg_names: Vec<syn::Ident> =
            (0..args.len()).map(|i| format_ident!("__a{}", i)).collect();
        let arg_tys: Vec<TokenStream> = args.iter().map(|a| self.q.ty_elided(a)).collect();
        let binds: Vec<TokenStream> = deliveries.iter().map(|d| d.output.bind()).collect();
        let values: Vec<TokenStream> = deliveries
            .iter()
            .flat_map(|d| {
                d.output
                    .wires
                    .iter()
                    .zip(&d.leaves)
                    .map(|(wr, l)| l.jvalue(&wr.name.to_token_stream()))
            })
            .collect();
        let what = format!("callback {ty}");
        Ok(quote!({
            let __up = ::prebindgen_jni_runtime::Upcall::new(env, &#w, "run", #desc, #frame)?;
            move |#(#arg_names: #arg_tys),*| {
                let __r = __up.call_void(|env| {
                    #(#binds)*
                    ::core::result::Result::Ok(::std::vec![#(#values),*])
                });
                if let ::core::result::Result::Err(__e) = __r {
                    ::prebindgen_jni_runtime::report_callback_error(#what, &__e);
                }
            }
        }))
    }

    /// Emit the Kotlin interfaces of a callback type, once.
    pub(crate) fn ensure_callback(&mut self, ty: &TypeRef) -> Res<()> {
        let TypeKind::Callback { args } = ty.kind() else {
            return Ok(());
        };
        let fqn = self.callback_fqn(ty)?;
        if !self.kt_claimed.insert(fqn.clone()) {
            return Ok(());
        }
        let name = fqn.rsplit('.').next().unwrap().to_string();
        let raw = format!("{name}Raw");
        let deliveries = self.callback_args(args)?;
        let params: Vec<&Param> = deliveries.iter().flat_map(|d| &d.params).collect();
        let leaves: Vec<&Leaf> = deliveries.iter().flat_map(|d| &d.leaves).collect();
        let sig = params
            .iter()
            .map(|p| format!("{}: {}", p.name, p.kt))
            .collect::<Vec<_>>()
            .join(", ");
        let raw_sig = leaves
            .iter()
            .map(|l| format!("{}: {}", raw_name(l), l.kt_raw()))
            .collect::<Vec<_>>()
            .join(", ");
        let raw_args = leaves
            .iter()
            .map(|l| raw_name(l))
            .collect::<Vec<_>>()
            .join(", ");
        let locals: Vec<String> = params
            .iter()
            .enumerate()
            .map(|(i, p)| format!("        val __p{i} = {}", p.decode))
            .collect();
        let call_args = (0..params.len())
            .map(|i| format!("__p{i}"))
            .collect::<Vec<_>>()
            .join(", ");
        let closes: Vec<String> = params
            .iter()
            .enumerate()
            .filter_map(|(i, p)| {
                p.close
                    .as_ref()
                    .map(|c| c.replacen(&p.name, &format!("__p{i}"), 1))
            })
            .collect();
        let body = if closes.is_empty() {
            format!("{}\n        run({call_args})", locals.join("\n"))
        } else {
            format!(
                "{}\n        try {{\n            run({call_args})\n        }} finally {{\n            {}\n        }}",
                locals.join("\n"),
                closes.join("\n            ")
            )
        };
        let lambda_params = if raw_args.is_empty() {
            String::new()
        } else {
            format!(" {raw_args} ->")
        };
        let body = body.replace("\n        ", "\n            ");
        let text = format!(
            "public fun interface {raw} {{\n    public fun run({raw_sig})\n}}\n\n\
             public fun interface {name} {{\n    public fun run({sig})\n\n\
             \x20   public fun asRaw(): {raw} =\n        {raw} {{{lambda_params}\n    {body}\n        }}\n}}\n"
        );
        let pkg = self.base_pkg.clone();
        self.kt_push(&pkg, text);
        Ok(())
    }
}

/// The named type under `Option`, `Box` and borrows.
fn core_name(ty: &TypeRef) -> Option<String> {
    match ty.kind() {
        TypeKind::Named { id, .. } => Some(id.name.clone()),
        TypeKind::Optional(t) | TypeKind::Boxed(t) | TypeKind::Ref { inner: t, .. } => core_name(t),
        _ => None,
    }
}

/// Deliveries side by side, bound in order except `deferred` (bound last —
/// they consume the value the others read), wires in declaration order.
fn concat(prelude: TokenStream, parts: Vec<Delivery>, deferred: Vec<usize>) -> Delivery {
    let fallible = parts.iter().any(|p| p.output.fallible);
    let mut binds: Vec<TokenStream> = Vec::new();
    let mut late: Vec<TokenStream> = Vec::new();
    for (i, p) in parts.iter().enumerate() {
        if deferred.contains(&i) {
            late.push(p.output.bind());
        } else {
            binds.push(p.output.bind());
        }
    }
    let mut params = Vec::new();
    let mut leaves = Vec::new();
    let mut wires = Vec::new();
    for p in parts {
        params.extend(p.params);
        leaves.extend(p.leaves);
        wires.extend(p.output.wires);
    }
    let names: Vec<&syn::Ident> = wires.iter().map(|w| &w.name).collect();
    let tuple = if names.len() == 1 {
        let n = names[0];
        quote!(#n)
    } else {
        quote!((#(#names),*))
    };
    let expr = quote!({ #prelude #(#binds)* #(#late)* #tuple });
    Delivery {
        params,
        leaves,
        output: Output {
            wires,
            expr,
            fallible,
        },
    }
}
