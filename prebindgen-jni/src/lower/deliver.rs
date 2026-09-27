//! Deliveries: a value handed to Kotlin as a *list of parameters* — a
//! builder's, a callback's, an error handler's.
//!
//! A value delivers as one typed parameter unless its type has an output
//! expansion (`expand_return!`): then it delivers as its fields, each field
//! itself a delivery — spliced when the field's type has an expansion of its
//! own, inlined when it comes from a value form (`fields!`) and is a data
//! class, typed otherwise. An optional value with an expansion delivers its
//! fields gated by one presence flag: every one of them `null` when absent.

use prebindgen_flat::flat::{Struct, Type as FlatType, TypeKind, TypeRef};
use prebindgen_tools::{names, ClosureCallbacks, ClosureWriter, Output, Record, Shape, Wire};
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};

use super::{
    field_seg, kt_ident,
    leaf::{join, method_desc, Leaf, LeafTy, Prim},
    leaf_ident, Dir,
};
use crate::{
    decl::{ExpandReturnDecl, FieldsDecl, ReturnField},
    plan::{err, ClassKind, Plan, Res, Setting},
};

/// One Kotlin-visible parameter of a delivery.
#[derive(Clone, Debug)]
pub(crate) struct DParam {
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
    pub params: Vec<DParam>,
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

impl Plan<'_> {
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

    /// Deliver `value` (of `ty`) as parameters named under `name`, with raw
    /// leaves under `root`. `gates` are the presence flags every parameter
    /// hangs on; `inline` spreads a data class's fields.
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
        // An expansion with no fields drops the value: nothing crosses.
        if explicit.is_some_and(|e| e.fields.is_empty()) {
            return Ok(Delivery {
                params: Vec::new(),
                leaves: Vec::new(),
                output: Output::none(value),
            });
        }
        let (core, v, borrowed) = peel(ty, value.clone());
        // An optional value whose inner type expands: one presence gate.
        if let TypeKind::Optional(inner) = core.kind() {
            let (inner_core, _, _) = peel(inner, quote!(__unused));
            if self.expansion(inner_core, explicit).is_some() {
                let present = Leaf::new(LeafTy::Prim(Prim::Z)).under(&join(root, "_present"));
                let x = format_ident!("__x{}", depth);
                let x_value = if borrowed { quote!(&#x) } else { quote!(#x) };
                let mut g = gates.to_vec();
                g.push(raw_name(&present));
                let inner_d =
                    self.deliver(inner, x_value, root, name, explicit, &g, false, depth + 1)?;
                let defaults = inner_d.leaves.iter().map(Leaf::rs_default);
                let bind = inner_d.output.bind();
                let names: Vec<&syn::Ident> =
                    inner_d.output.wires.iter().map(|w| &w.name).collect();
                let mut wires = vec![Wire::new(leaf_ident(root, "_present"), present.rs())];
                wires.extend(inner_d.output.wires.clone());
                let scrutinee = if borrowed { quote!(#v.as_ref()) } else { v };
                let output = Output {
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
                    output,
                });
            }
        }
        if let Some(e) = self.expansion(core, explicit) {
            return self.expanded(core, e, v, borrowed, root, name, gates, depth);
        }
        if inline {
            if let Shape::Declared {
                setting: Setting::Class(c),
                ..
            } = self.shape(core)?
            {
                if let ClassKind::Data { .. } = c.kind {
                    let v = if borrowed {
                        quote!(::core::clone::Clone::clone(#v))
                    } else {
                        v
                    };
                    return self.fields_of(self.struct_of(c)?, None, v, root, name, gates, depth);
                }
            }
        }
        self.plain(ty, value, root, name, gates, depth)
    }

    /// Whether `ty` is an enum (`Some(false)`) or an optional enum
    /// (`Some(true)`), under borrows and boxes.
    fn enum_value(&self, ty: &TypeRef) -> Res<Option<bool>> {
        let is_enum = |t: &TypeRef| {
            let (core, _, _) = peel(t, TokenStream::new());
            matches!(core.kind(), TypeKind::Named { id, .. }
                if matches!(self.class(&id.name).map(|c| &c.kind), Some(ClassKind::Enum)))
        };
        let (core, _, _) = peel(ty, TokenStream::new());
        if is_enum(core) {
            return Ok(Some(false));
        }
        if let TypeKind::Optional(inner) = core.kind() {
            if is_enum(inner) {
                return Ok(Some(true));
            }
        }
        Ok(None)
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
        // An enum reaches a delivered parameter as its value, an `Int`.
        let as_int = self.enum_value(ty)?;
        let decode = match as_int {
            Some(_) => gate(gates, raws[0].clone()),
            None => gate(gates, self.kt_decode(ty, &raws, gated, depth)?),
        };
        let pname = kt_ident(&names::camel(if name.is_empty() { "value" } else { name }));
        let close = self.kt_close(ty, &pname)?.map(|c| {
            if gated && !c.contains("?.") {
                c.replacen(&format!("{pname}."), &format!("{pname}?."), 1)
            } else {
                c
            }
        });
        Ok(Delivery {
            params: vec![DParam {
                name: pname,
                kt: match as_int {
                    Some(true) => "Int?".to_string(),
                    Some(false) => nullable("Int".to_string(), gated),
                    None => nullable(self.kt_type(ty)?, gated),
                },
                decode,
                close,
            }],
            leaves,
            output: self.rs_encode(ty, value, root, depth)?,
        })
    }

    /// The fields of a struct value, each delivered (inlining data classes),
    /// with the per-field expansions and names of a value form.
    #[allow(clippy::too_many_arguments)]
    fn fields_of(
        &self,
        s: &Struct,
        form: Option<&FieldsDecl>,
        value: TokenStream,
        root: &str,
        name: &str,
        gates: &[String],
        depth: usize,
    ) -> Res<Delivery> {
        let head = self.q.path(&s.name);
        let binds: Vec<syn::Ident> = (0..s.fields.len())
            .map(|i| format_ident!("__f{}_{}", depth, i))
            .collect();
        let pat = Record::Struct(s).pattern(&head, &binds);
        let mut parts = Vec::new();
        for (f, b) in s.fields.iter().zip(&binds) {
            let seg = field_seg(f);
            let explicit = form.and_then(|d| field_entry(&d.overrides, &seg));
            let pname = match form.and_then(|d| field_entry(&d.names, &seg)) {
                Some(n) => n.clone(),
                None => join(name, &seg),
            };
            parts.push(self.deliver(
                &f.ty,
                b.to_token_stream(),
                &join(root, &seg),
                &pname,
                explicit,
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
                    let func = g.fun.resolve(self.flat).map_err(crate::Error)?;
                    let callee = g.fun.callee(&self.q);
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
                    let (r, n) = (join(root, &seg), join(name, &seg));
                    // A field of the expanded type itself is that value, not
                    // another expansion of it.
                    parts.push(
                        if core_name(&func.ret).as_deref() == Some(type_name.as_str()) {
                            self.plain(&func.ret, quote!(#callee(#arg)), &r, &n, gates, depth + 1)?
                        } else {
                            let call = quote!(#callee(#arg));
                            self.deliver(&func.ret, call, &r, &n, None, gates, false, depth + 1)?
                        },
                    );
                }
                ReturnField::Handle => {
                    let handle = if borrowed {
                        quote!(::core::clone::Clone::clone(#v))
                    } else {
                        quote!(#v)
                    };
                    let (r, n) = (join(root, "handle"), join(name, "handle"));
                    deferred.push(parts.len());
                    parts.push(self.plain(ty, handle, &r, &n, gates, depth + 1)?);
                }
                ReturnField::Form { form, consume } => {
                    let fun = &form.fun;
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
                    parts.push(self.fields_of(
                        s,
                        Some(form),
                        sv.to_token_stream(),
                        root,
                        name,
                        gates,
                        depth + 1,
                    )?);
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
            .map(|(i, a)| self.callback_arg(i, a, &format_ident!("__a{}", i).to_token_stream()))
            .collect()
    }

    fn callback_arg(&self, i: usize, ty: &TypeRef, value: &TokenStream) -> Res<Delivery> {
        let name = match self.shape(ty) {
            Ok(Shape::Declared { name, .. }) => names::snake(name),
            Ok(Shape::Scalar(k)) => k.as_str().to_string(),
            _ => format!("arg{i}"),
        };
        self.deliver(
            ty,
            value.clone(),
            &format!("a{i}"),
            &name,
            None,
            &[],
            false,
            1,
        )
    }

    fn callback_base(&self, ty: &TypeRef) -> Res<String> {
        let TypeKind::Callback { args } = ty.kind() else {
            return err(format!("`{ty}` is not a callback"));
        };
        if args.is_empty() {
            return Ok("Void".to_string());
        }
        Ok(args.iter().map(type_base).collect())
    }

    /// The package of a callback type's interfaces: the class of its one
    /// argument, else the base package.
    fn callback_pkg(&self, ty: &TypeRef) -> String {
        if let TypeKind::Callback { args } = ty.kind() {
            if let [arg] = args.as_slice() {
                if let Some(c) = core_name(arg).and_then(|n| self.class(&n)) {
                    return c.pkg.clone();
                }
            }
        }
        self.base_pkg.clone()
    }

    /// The user-facing callback interface of `ty`.
    pub(crate) fn callback_fqn(&self, ty: &TypeRef) -> Res<String> {
        let (pkg, base) = (self.callback_pkg(ty), self.callback_base(ty)?);
        Ok(format!("{pkg}.{base}Callback"))
    }

    /// The raw callback interface of `ty`, the one Rust invokes.
    pub(crate) fn callback_raw_fqn(&self, ty: &TypeRef) -> Res<String> {
        let (pkg, base) = (self.callback_pkg(ty), self.callback_base(ty)?);
        Ok(format!("{pkg}.{base}CallbackRaw"))
    }

    /// The Rust closure a callback object `w` becomes: every call delivers
    /// the arguments through one upcall.
    pub(crate) fn callback_closure(
        &self,
        ty: &TypeRef,
        args: &[TypeRef],
        w: &syn::Ident,
    ) -> Res<TokenStream> {
        let leaves: Vec<Leaf> = self
            .callback_args(args)?
            .into_iter()
            .flat_map(|d| d.leaves)
            .collect();
        let desc = method_desc(&leaves, "V");
        let frame = 32 + leaves.len() as i32 * 2;
        let what = format!("callback {ty}");
        ClosureWriter::new(args)
            .setup(quote!(let __up = ::prebindgen_jni_runtime::Upcall::new(env, &#w, "run", #desc, #frame)?;))
            .on_error(quote!(::prebindgen_jni_runtime::report_callback_error(#what, &__err);))
            .write(&self.q, &mut CallbackArgs(self), |binds, outs| {
                let values = outs
                    .iter()
                    .flat_map(|o| &o.wires)
                    .zip(&leaves)
                    .map(|(wr, l)| l.jvalue(&wr.name.to_token_stream()));
                quote! {
                    __up.call_void(|env| {
                        #binds
                        ::core::result::Result::Ok(::std::vec![#(#values),*])
                    })?;
                }
            })
    }
}

/// A callback's arguments, each delivered.
struct CallbackArgs<'p, 'f>(&'p Plan<'f>);

impl ClosureCallbacks for CallbackArgs<'_, '_> {
    type Error = crate::Error;

    fn arg(&mut self, index: usize, ty: &TypeRef, value: &TokenStream) -> Res<Output> {
        Ok(self.0.callback_arg(index, ty, value)?.output)
    }
}

/// The entry a value form declares for field `seg`.
fn field_entry<'a, T>(list: &'a [(String, T)], seg: &str) -> Option<&'a T> {
    list.iter().find(|(n, _)| n == seg).map(|(_, v)| v)
}

/// `ty` under its transparent layers and borrows: the core type, the value
/// expression reaching it, and whether that value is borrowed.
fn peel(ty: &TypeRef, value: TokenStream) -> (&TypeRef, TokenStream, bool) {
    match ty.kind() {
        TypeKind::Boxed(inner) => peel(inner, quote!((*#value))),
        TypeKind::Ref { inner, .. } => {
            let (t, v, _) = peel(inner, value);
            (t, v, true)
        }
        _ => (ty, value, false),
    }
}

/// The base a callback interface is named from.
fn type_base(ty: &TypeRef) -> String {
    match ty.kind() {
        TypeKind::Named { id, .. } => id.name.clone(),
        TypeKind::Scalar(k) => k.as_str().to_string(),
        TypeKind::String | TypeKind::Str => "String".to_string(),
        TypeKind::Ref { inner, .. } | TypeKind::Boxed(inner) | TypeKind::Cow { inner, .. } => {
            type_base(inner)
        }
        TypeKind::Vec(e) | TypeKind::Slice(e) => format!("{}List", type_base(e)),
        TypeKind::Optional(t) => format!("Optional{}", type_base(t)),
        TypeKind::Array { elem, .. } => format!("{}Array", type_base(elem)),
        _ => "Value".to_string(),
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
    let mut binds = Vec::new();
    let mut late = Vec::new();
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
        // A handle delivered as a field of a larger value is the receiver's
        // to keep; only a whole delivered value is closed after a callback.
        params.extend(p.params.into_iter().map(|p| DParam { close: None, ..p }));
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
