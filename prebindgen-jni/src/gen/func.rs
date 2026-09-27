//! One bound function: its plan, its Rust extern and its Kotlin wrapper.
//!
//! The plan is made once and read by both sides, so the extern's parameter
//! list and the `external fun` declaration cannot disagree.

use std::rc::Rc;

use prebindgen_tools::{
    flat::flat::{Function, Param as FlatParam, TypeKind, TypeRef},
    function::{error_ident, result_ident},
    names, FunctionCallbacks, FunctionWriter, Input, Output, Return, Wire,
};
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};

use super::{
    codec::{kt_ident, Dir, Form, HandleSite, Kind, KtEnc},
    deliver::{raw_name, Delivery, Param},
    err,
    leaf::{method_desc, Leaf, LeafTy},
    rust::jni_symbol,
    select::{Selector, SigParam},
    Bound, Class, ClassKind, Gen, Placement, Res,
};
use crate::decl::ExpandReturnDecl;

/// How one source parameter crosses.
pub(crate) enum PPlan {
    /// The receiver of a member: `this`.
    Receiver(FlatParam),
    /// A value; `true` when its leaves travel packed.
    Value(FlatParam, bool),
    Selector(Selector, bool),
}

/// How the (successful) result crosses.
pub(crate) enum RPlan {
    Unit,
    /// One leaf, returned as the extern's own result.
    Direct {
        ty: TypeRef,
        leaf: Leaf,
    },
    /// Several leaves, handed to a sink whose result the extern returns.
    Sink {
        ty: TypeRef,
        leaves: Vec<Leaf>,
        iface: String,
        kt_sink: String,
    },
    /// Delivered to a caller-supplied builder (`R`), `null` when absent.
    Builder {
        whole: TypeRef,
        exp: Option<ExpandReturnDecl>,
        optional: bool,
        iface: String,
        leaves: Vec<Leaf>,
    },
    /// A sequence folded by a caller-supplied folder (`A`), `null` when absent.
    Fold {
        whole: TypeRef,
        exp: Option<ExpandReturnDecl>,
        optional: bool,
        iface: String,
        columns_iface: String,
        params: Vec<Param>,
        leaves: Vec<Leaf>,
    },
}

/// How an `Err` crosses.
#[allow(clippy::large_enum_variant)] // one per bound function
pub(crate) enum EPlan {
    None,
    /// Displayed into the binding-error channel.
    Binding,
    /// Delivered to a typed error handler.
    Domain {
        ty: TypeRef,
        handler: String,
        capture: String,
        raw_iface: String,
        params: Vec<Param>,
        leaves: Vec<Leaf>,
    },
}

pub(crate) struct FnPlan {
    pub params: Vec<PPlan>,
    pub ret: RPlan,
    pub err: EPlan,
}

impl Gen<'_> {
    /// Plan a bound function.
    pub(crate) fn plan(&mut self, b: &Bound) -> Res<FnPlan> {
        let f = &b.func;
        let receiver = matches!(b.placement, Placement::Method(_));
        if receiver && f.params.is_empty() {
            return err(format!("method `{}` has no receiver parameter", f.name));
        }
        let mut params = Vec::new();
        for (i, p) in f.params.iter().enumerate() {
            if receiver && i == 0 {
                params.push(PPlan::Receiver(p.clone()));
                continue;
            }
            let pname = names::bare(&p.name);
            let explicit = b
                .decl
                .params
                .iter()
                .find(|(n, _)| *n == pname)
                .map(|(_, d)| d);
            match self.selector(p, explicit)? {
                Some(s) => {
                    let split = b.decl.splits.contains(&pname);
                    params.push(PPlan::Selector(s, split));
                }
                None => {
                    if let TypeKind::Callback { .. } = p.ty.kind() {
                        self.ensure_callback(&p.ty)?;
                    }
                    let packed = matches!(
                        self.kind(&p.ty),
                        Ok(Kind::Record(c, _)) if matches!(c.kind, ClassKind::Data { packed: true })
                    ) || matches!(
                        self.kind(&p.ty),
                        Ok(Kind::Ref { inner, .. }) if matches!(
                            self.kind(inner),
                            Ok(Kind::Record(c, _)) if matches!(c.kind, ClassKind::Data { packed: true })
                        )
                    );
                    params.push(PPlan::Value(p.clone(), packed));
                }
            }
        }
        for s in &b.decl.splits {
            if !params
                .iter()
                .any(|p| matches!(p, PPlan::Selector(sel, _) if names::bare(&sel.param.name) == *s))
            {
                return err(format!(
                    "`{}`: `.split_on_param(\"{s}\")` names no selector parameter",
                    f.name
                ));
            }
        }
        // The JVM caps a method's arguments at 255 slots (the holder object
        // is one): pack the largest values until the extern fits.
        loop {
            let mut slots = 1 + 3;
            let mut largest: Option<(usize, usize)> = None;
            for (i, p) in params.iter().enumerate() {
                let n: usize = self.param_leaves(p)?.iter().map(Leaf::slots).sum();
                slots += n;
                if let PPlan::Value(_, false) = p {
                    if largest.is_none_or(|(_, m)| n > m) {
                        largest = Some((i, n));
                    }
                }
            }
            match largest {
                Some((i, _)) if slots > 255 => {
                    if let PPlan::Value(_, packed) = &mut params[i] {
                        *packed = true;
                    }
                }
                _ => break,
            }
        }
        let (ok, err_ty) = match f.ret.kind() {
            TypeKind::Fallible { ok, err } => ((**ok).clone(), Some((**err).clone())),
            _ => (f.ret.clone(), None),
        };
        let ret = self.ret_plan(b, &ok)?;
        let err = match err_ty {
            None => EPlan::None,
            Some(e) => self.err_plan(&e)?,
        };
        Ok(FnPlan { params, ret, err })
    }

    fn ret_plan(&mut self, b: &Bound, ty: &TypeRef) -> Res<RPlan> {
        if let TypeKind::Unit = ty.kind() {
            return Ok(RPlan::Unit);
        }
        let explicit = b.decl.ret.as_ref();
        // A constructor returns its own class as itself, never expanded.
        if let Placement::Constructor(c) = &b.placement {
            let own =
                |t: &TypeRef| matches!(t.kind(), TypeKind::Named { id, .. } if id.name == c.rust);
            if explicit.is_none() && own(ty) {
                let leaves = self.leaves(ty, Dir::Out)?;
                return Ok(RPlan::Direct {
                    ty: ty.clone(),
                    leaf: leaves.into_iter().next().expect("a handle leaf"),
                });
            }
        }
        // Layers a builder or folder absorbs: `Box`, `Option`, `Vec`.
        let mut core = ty;
        let mut optional = false;
        let mut seq = false;
        loop {
            match core.kind() {
                TypeKind::Boxed(t) => core = t,
                TypeKind::Optional(t) if !optional && !seq => {
                    optional = true;
                    core = t
                }
                TypeKind::Vec(t) if !seq => {
                    seq = true;
                    core = t
                }
                _ => break,
            }
        }
        let bare = match core.kind() {
            TypeKind::Ref { inner, .. } => &**inner,
            _ => core,
        };
        let exp_on = self.expansion(bare, explicit).cloned();
        if let Some(e) = exp_on {
            let tname = match bare.kind() {
                TypeKind::Named { id, .. } => id.name.clone(),
                _ => return err(format!("`{ty}`: an output expansion needs a named type")),
            };
            let suffix = if explicit.is_some() {
                names::pascal(&names::bare(&b.func.name))
            } else {
                String::new()
            };
            let d = self.deliver(core, quote!(__x), "r", "", Some(&e), &[], false, 1)?;
            if seq {
                let iface = format!("{}.{tname}{suffix}Folder", self.base_pkg);
                let columns_iface = format!("{}.{tname}{suffix}FolderColumns", self.base_pkg);
                self.ensure_folder(&iface, &columns_iface, &d)?;
                return Ok(RPlan::Fold {
                    whole: ty.clone(),
                    exp: Some(e),
                    optional,
                    iface,
                    columns_iface,
                    params: d.params,
                    leaves: d.leaves,
                });
            }
            let iface = format!("{}.{tname}{suffix}Builder", self.base_pkg);
            self.ensure_builder(&iface, &d)?;
            return Ok(RPlan::Builder {
                whole: ty.clone(),
                exp: Some(e),
                optional,
                iface,
                leaves: d.leaves,
            });
        }
        // A typed value.
        let leaves = self.leaves(ty, Dir::Out)?;
        if leaves.len() == 1 {
            return Ok(RPlan::Direct {
                ty: ty.clone(),
                leaf: leaves.into_iter().next().unwrap(),
            });
        }
        let (iface, kt_sink) = self.ensure_sink(ty)?;
        Ok(RPlan::Sink {
            ty: ty.clone(),
            leaves,
            iface,
            kt_sink,
        })
    }

    fn err_plan(&mut self, e: &TypeRef) -> Res<EPlan> {
        let TypeKind::Named { id, .. } = e.kind() else {
            return Ok(EPlan::Binding);
        };
        if self.expansion(e, None).is_none() {
            return Ok(EPlan::Binding);
        }
        let Some(class) = self.classes.get(&id.name).cloned() else {
            return err(format!(
                "`{}`: an error with an output expansion needs a declared class",
                id.name
            ));
        };
        let handler = format!("{}.{}Handler", class.pkg, id.name);
        let raw_iface = format!("{handler}Raw");
        let capture = format!("{handler}Capture");
        let d = self.deliver(e, quote!(__e), "e", "", None, &[], false, 1)?;
        self.ensure_error_handler(&class, &handler, &raw_iface, &capture, &d)?;
        Ok(EPlan::Domain {
            ty: e.clone(),
            handler,
            capture,
            raw_iface,
            params: d.params,
            leaves: d.leaves,
        })
    }

    // ── Kotlin support interfaces ───────────────────────────────────────

    /// A typed value's sink: an interface Rust calls with the leaves, and a
    /// singleton that assembles the value.
    fn ensure_sink(&mut self, ty: &TypeRef) -> Res<(String, String)> {
        let base = names::mangle(ty);
        let iface = format!("{}.__Sink_{base}", self.base_pkg);
        let single = format!("{}.__sink_{base}", self.base_pkg);
        if self.kt_claimed.insert(iface.clone()) {
            let leaves = self.leaves(ty, Dir::Out)?;
            let raws: Vec<String> = leaves.iter().map(raw_name).collect();
            let sig = leaves
                .iter()
                .zip(&raws)
                .map(|(l, n)| format!("{n}: {}", l.kt_raw()))
                .collect::<Vec<_>>()
                .join(", ");
            let decode = self.kt_decode(ty, &raws, false, 0)?;
            let text = format!(
                "public fun interface __Sink_{base} {{\n    public fun run({sig}): Any?\n}}\n\n\
                 internal val __sink_{base}: __Sink_{base} = __Sink_{base} {{ {} ->\n    {decode}\n}}\n",
                raws.join(", ")
            );
            let pkg = self.base_pkg.clone();
            self.kt_push(&pkg, text);
        }
        Ok((iface, single))
    }

    fn ensure_builder(&mut self, iface: &str, d: &Delivery) -> Res<()> {
        if !self.kt_claimed.insert(iface.to_string()) {
            return Ok(());
        }
        let name = iface.rsplit('.').next().unwrap();
        let sig = d
            .params
            .iter()
            .map(|p| format!("{}: {}", p.name, p.kt))
            .collect::<Vec<_>>()
            .join(", ");
        let raws: Vec<String> = d.leaves.iter().map(raw_name).collect();
        let raw_sig = d
            .leaves
            .iter()
            .zip(&raws)
            .map(|(l, n)| format!("{n}: {}", l.kt_raw()))
            .collect::<Vec<_>>()
            .join(", ");
        let args = d
            .params
            .iter()
            .map(|p| p.decode.clone())
            .collect::<Vec<_>>()
            .join(", ");
        let lambda = if raws.is_empty() {
            String::new()
        } else {
            format!(" {} ->", raws.join(", "))
        };
        let text = format!(
            "public fun interface {name}Raw<out R> {{\n    public fun run({raw_sig}): R\n}}\n\n\
             public fun interface {name}<out R> {{\n    public fun run({sig}): R\n\n\
             \x20   public fun asRaw(): {name}Raw<R> =\n        {name}Raw<R> {{{lambda}\n            run({args})\n        }}\n}}\n"
        );
        let pkg = self.base_pkg.clone();
        self.kt_push(&pkg, text);
        Ok(())
    }

    fn ensure_folder(&mut self, iface: &str, columns: &str, d: &Delivery) -> Res<()> {
        if !self.kt_claimed.insert(iface.to_string()) {
            return Ok(());
        }
        let name = iface.rsplit('.').next().unwrap();
        let cname = columns.rsplit('.').next().unwrap();
        let sig = d
            .params
            .iter()
            .map(|p| format!("{}: {}", p.name, p.kt))
            .collect::<Vec<_>>()
            .join(", ");
        let cols = d
            .leaves
            .iter()
            .map(|l| format!("{}: {}", raw_name(l), l.column().kt_raw()))
            .collect::<Vec<_>>()
            .join(", ");
        let sep = if cols.is_empty() { "" } else { ", " };
        let text = format!(
            "public fun interface {name}<A> {{\n    public fun run(acc: A, {sig}): A\n}}\n\n\
             public fun interface {cname} {{\n    public fun run(n: Int{sep}{cols}): Any?\n}}\n"
        );
        let pkg = self.base_pkg.clone();
        self.kt_push(&pkg, text);
        Ok(())
    }

    fn ensure_error_handler(
        &mut self,
        class: &Class,
        handler: &str,
        raw_iface: &str,
        capture: &str,
        d: &Delivery,
    ) -> Res<()> {
        if !self.kt_claimed.insert(handler.to_string()) {
            return Ok(());
        }
        let name = handler.rsplit('.').next().unwrap();
        let raw = raw_iface.rsplit('.').next().unwrap();
        let cap = capture.rsplit('.').next().unwrap();
        let sig = d
            .params
            .iter()
            .map(|p| format!("{}: {}", p.name, p.kt))
            .collect::<Vec<_>>()
            .join(", ");
        let raws: Vec<String> = d.leaves.iter().map(raw_name).collect();
        let raw_sig = d
            .leaves
            .iter()
            .zip(&raws)
            .map(|(l, n)| format!("{n}: {}", l.kt_raw()))
            .collect::<Vec<_>>()
            .join(", ");
        let fields = d
            .leaves
            .iter()
            .zip(&raws)
            .map(|(l, n)| {
                let t = l.kt_raw();
                let nt = if t.ends_with('?') { t } else { format!("{t}?") };
                format!("    @JvmField var {n}: {nt} = null")
            })
            .collect::<Vec<_>>()
            .join("\n");
        let assigns = raws
            .iter()
            .map(|n| format!("this.{n} = {n}"))
            .collect::<Vec<_>>()
            .join("; ");
        let resets = raws
            .iter()
            .map(|n| format!("c.{n} = null"))
            .collect::<Vec<_>>()
            .join("; ");
        let text = format!(
            "public fun interface {name}<out R> {{\n    public fun run({sig}): R\n}}\n\n\
             public fun interface {raw} {{\n    public fun run({raw_sig})\n}}\n\n\
             internal class {cap} : {raw} {{\n    @JvmField var failed: Boolean = false\n{fields}\n\
             \x20   override fun run({raw_sig}) {{ failed = true; {assigns} }}\n\
             \x20   companion object {{\n\
             \x20       private val TL: ThreadLocal<{cap}> = ThreadLocal.withInitial {{ {cap}() }}\n\
             \x20       @JvmStatic fun acquire(): {cap} {{\n\
             \x20           val c = TL.get()\n\
             \x20           c.failed = false; {resets}\n\
             \x20           return c\n\
             \x20       }}\n\
             \x20   }}\n}}\n"
        );
        let pkg = class.pkg.clone();
        self.kt_push(&pkg, text);
        Ok(())
    }

    // ── the whole function ──────────────────────────────────────────────

    /// Emit a bound function's extern, and return its Kotlin wrapper(s).
    pub(crate) fn function(&mut self, b: &Bound) -> Res<String> {
        let plan = self.plan(b)?;
        self.extern_rust(b, &plan)?;
        self.extern_kotlin(b, &plan)?;
        self.wrapper(b, &plan)
    }

    /// The raw parameter leaves of the extern, per source parameter.
    fn param_leaves(&self, p: &PPlan) -> Res<Vec<Leaf>> {
        match p {
            PPlan::Receiver(fp) | PPlan::Value(fp, false) => Ok(self
                .leaves(&fp.ty, Dir::In)?
                .into_iter()
                .map(|l| l.under(&names::bare(&fp.name)))
                .collect()),
            PPlan::Value(fp, true) => {
                let root = names::bare(&fp.name);
                let leaves: Vec<Leaf> = self
                    .leaves(&fp.ty, Dir::In)?
                    .into_iter()
                    .map(|l| l.under(&root))
                    .collect();
                Ok(super::pack::packed_leaves(&root, &leaves))
            }
            PPlan::Selector(s, _) => self.selector_leaves(s),
        }
    }

    /// The extern's trailing object parameters: sink, error sink, domain sink.
    fn trailing(&self, plan: &FnPlan) -> Vec<(&'static str, bool)> {
        let sink = matches!(
            plan.ret,
            RPlan::Sink { .. } | RPlan::Builder { .. } | RPlan::Fold { .. }
        );
        let mut v = Vec::new();
        if sink {
            v.push(("sink", true));
        }
        v.push(("errorSink", true));
        if matches!(plan.err, EPlan::Domain { .. }) {
            v.push(("domainSink", true));
        }
        v
    }

    fn extern_kotlin(&mut self, b: &Bound, plan: &FnPlan) -> Res<()> {
        let mut params = Vec::new();
        for p in &plan.params {
            for l in self.param_leaves(p)? {
                params.push(format!("{}: {}", raw_name(&l), l.kt_raw()));
            }
        }
        for (n, _) in self.trailing(plan) {
            params.push(format!("{n}: Any"));
        }
        let ret = match &plan.ret {
            RPlan::Unit => String::new(),
            RPlan::Direct { leaf, .. } => format!(": {}", leaf.kt_raw()),
            _ => ": Any?".to_string(),
        };
        self.externs.push(format!(
            "    external fun {}({}){ret}",
            b.ext_name,
            params.join(", ")
        ));
        Ok(())
    }

    fn extern_rust(&mut self, b: &Bound, plan: &FnPlan) -> Res<()> {
        let symbol = jni_symbol(&self.harness_fqn(), &b.ext_name);
        let valued = self.valued;
        let mut cb = RustCb { g: self, plan };
        let mut w = FunctionWriter::new(&b.func, b.callee.clone())
            .name(format_ident!("{}", symbol))
            .abi(Some("system"))
            .generics(quote!(<'a>))
            .attr(quote!(#[no_mangle]))
            .attr(quote!(#[allow(non_snake_case, unused_mut, unused_variables, unused_braces, unused_parens, unused_unsafe, dead_code, clippy::all)]))
            .leading(Wire::new(format_ident!("__env"), quote!(::prebindgen_jni_runtime::jni::JNIEnv<'a>)))
            .leading(Wire::new(
                format_ident!("_class"),
                quote!(::prebindgen_jni_runtime::jni::objects::JClass<'a>),
            ))
            .prologue(quote!(let mut __env = __env; let env = &mut __env;));
        if valued {
            w = w.call_with(|callee, _| callee.clone());
        }
        w = w.trailing(Wire::new(
            format_ident!("__error_sink"),
            quote!(::prebindgen_jni_runtime::jni::objects::JObject<'a>),
        ));
        if matches!(plan.err, EPlan::Domain { .. }) {
            w = w.trailing(Wire::new(
                format_ident!("__domain_sink"),
                quote!(::prebindgen_jni_runtime::jni::objects::JObject<'a>),
            ));
        }
        let item = w.write(&mut cb)?;
        self.rust.push(item);
        Ok(())
    }

    /// The Rust encoding of a successful value per the return plan, as
    /// statements ending in `Ok(<return wire>)` inside a `Result` closure.
    fn rust_ret_value(&self, plan: &RPlan, value: TokenStream) -> Res<TokenStream> {
        Ok(match plan {
            RPlan::Unit => quote!({ let _ = #value; ::core::result::Result::Ok(()) }),
            RPlan::Direct { ty, leaf } => {
                let out = self.rs_encode(ty, value, "r", 1)?;
                let bind = out.bind();
                let n = &out.wires[0].name;
                let v = if leaf.is_obj() {
                    quote!(#n.into_raw())
                } else {
                    quote!(#n)
                };
                quote!({ #bind ::core::result::Result::Ok(#v) })
            }
            RPlan::Sink { ty, leaves, iface, .. } => {
                let out = self.rs_encode(ty, value, "r", 1)?;
                call_sink(&out, leaves, iface)
            }
            RPlan::Builder {
                whole,
                exp,
                iface,
                leaves,
                ..
            } => {
                let iface = format!("{iface}Raw");
                self.layered(whole, value, &mut |g, t, v| {
                    let x = format_ident!("__x");
                    let d = g.deliver(t, x.to_token_stream(), "r", "", exp.as_ref(), &[], false, 1)?;
                    let call = call_sink(&d.output, leaves, &iface);
                    Ok(quote!({ let #x = #v; #call }))
                })?
            }
            RPlan::Fold {
                whole,
                exp,
                columns_iface,
                leaves,
                ..
            } => self.layered(whole, value, &mut |g, t, v| {
                let TypeKind::Vec(elem) = t.kind() else {
                    return err(format!("`{t}`: a folded result must be a `Vec`"));
                };
                let e = format_ident!("__x");
                let d = g.deliver(elem, e.to_token_stream(), "r", "", exp.as_ref(), &[], false, 1)?;
                let cols = g.columns(&d.output, leaves, &e, quote!(__items))?;
                let mut col_leaves = vec![Leaf::new(LeafTy::Prim(super::leaf::Prim::I))];
                col_leaves.extend(leaves.iter().map(Leaf::column));
                let call = call_sink(&cols, &col_leaves, columns_iface);
                Ok(quote!({ let __items: ::std::vec::Vec<_> = ::core::iter::IntoIterator::into_iter(#v).collect(); #call }))
            })?,
        })
    }

    /// Walk the layers a builder or folder absorbs — `Box`, the first
    /// `Option` (absent: a null result) — down to the value `core` delivers
    /// (the `Vec` itself, for a fold).
    fn layered(
        &self,
        ty: &TypeRef,
        value: TokenStream,
        core: &mut dyn FnMut(&Self, &TypeRef, TokenStream) -> Res<TokenStream>,
    ) -> Res<TokenStream> {
        match ty.kind() {
            TypeKind::Boxed(t) => self.layered(t, quote!((*#value)), core),
            TypeKind::Optional(t) => {
                let inner = self.layered_inner(t, quote!(__some), core)?;
                Ok(quote!(match #value {
                    ::core::option::Option::Some(__some) => #inner,
                    ::core::option::Option::None => ::core::result::Result::Ok(::core::ptr::null_mut()),
                }))
            }
            _ => core(self, ty, value),
        }
    }

    /// [`Self::layered`] below the `Option`: only `Box` layers remain.
    fn layered_inner(
        &self,
        ty: &TypeRef,
        value: TokenStream,
        core: &mut dyn FnMut(&Self, &TypeRef, TokenStream) -> Res<TokenStream>,
    ) -> Res<TokenStream> {
        match ty.kind() {
            TypeKind::Boxed(t) => self.layered_inner(t, quote!((*#value)), core),
            _ => core(self, ty, value),
        }
    }

    /// Columns of a sequence of deliveries: a count and one array per leaf.
    fn columns(
        &self,
        elem: &Output,
        leaves: &[Leaf],
        x: &syn::Ident,
        items: TokenStream,
    ) -> Res<Output> {
        let rt = quote!(::prebindgen_jni_runtime);
        let mut setup = Vec::new();
        let mut pushes = Vec::new();
        let mut finals = vec![quote!(__n as i32)];
        let mut wires = vec![Wire::new(format_ident!("__cn"), super::leaf::Prim::I.rs())];
        for (k, (l, w)) in leaves.iter().zip(&elem.wires).enumerate() {
            let col = format_ident!("__col{}", k);
            let local = &w.name;
            wires.push(Wire::new(format_ident!("__colw{}", k), l.column().rs()));
            match l.prim() {
                Some(p) => {
                    let (_, write) = p.array_helpers();
                    let write = format_ident!("{}", write);
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
        let bind = elem.bind();
        Ok(Output::fallible(
            wires,
            quote!({
                let __n = #items.len();
                #(#setup)*
                for (__i, #x) in #items.into_iter().enumerate() {
                    #bind
                    #(#pushes)*
                }
                (#(#finals),*)
            }),
        ))
    }

    fn rust_error(&self, plan: &EPlan) -> Res<TokenStream> {
        Ok(match plan {
            EPlan::None => quote!(unreachable!()),
            EPlan::Binding => quote!(::core::result::Result::Err(
                ::std::string::ToString::to_string(&__e)
            )),
            EPlan::Domain {
                ty,
                raw_iface,
                leaves,
                ..
            } => {
                let d = self.deliver(ty, quote!(__e), "e", "", None, &[], false, 1)?;
                let bind = d.output.bind();
                let values: Vec<TokenStream> = d
                    .output
                    .wires
                    .iter()
                    .zip(leaves)
                    .map(|(w, l)| l.jvalue(&w.name.to_token_stream()))
                    .collect();
                let desc = method_desc(leaves, "V");
                let fqn = raw_iface.replace('.', "/");
                quote!({
                    #bind
                    static __D: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
                    __D.call_void(env, #fqn, "run", #desc, &__domain_sink, &[#(#values),*])?;
                    ::core::result::Result::Err(::std::string::String::new())
                })
            }
        })
    }

    // ── Kotlin wrapper ──────────────────────────────────────────────────

    /// The Kotlin type the wrapper returns, and its type parameters.
    fn kt_ret(&self, plan: &RPlan) -> Res<(String, String)> {
        Ok(match plan {
            RPlan::Unit => ("Unit".to_string(), String::new()),
            RPlan::Direct { ty, .. } | RPlan::Sink { ty, .. } => (self.kt_type(ty)?, String::new()),
            RPlan::Builder { optional, .. } => (
                if *optional {
                    "R?".to_string()
                } else {
                    "R".to_string()
                },
                "<R> ".to_string(),
            ),
            RPlan::Fold { optional, .. } => (
                if *optional {
                    "A?".to_string()
                } else {
                    "A".to_string()
                },
                "<A> ".to_string(),
            ),
        })
    }

    fn wrapper(&mut self, b: &Bound, plan: &FnPlan) -> Res<String> {
        let (ret, generics) = self.kt_ret(&plan.ret)?;
        let mut cx = KtEnc::default();
        let mut sig: Vec<SigParam> = Vec::new();
        let mut args: Vec<String> = Vec::new();
        let mut this_class: Option<Rc<Class>> = None;
        if let Placement::Method(c) = &b.placement {
            this_class = Some(c.clone());
        }
        for p in &plan.params {
            match p {
                PPlan::Receiver(fp) => {
                    args.extend(self.kt_encode(&fp.ty, "this", false, &mut cx, true)?);
                }
                PPlan::Value(fp, packed) => {
                    let n = kt_ident(&names::camel(&names::bare(&fp.name)));
                    sig.push(SigParam::Plain(n.clone(), self.kt_type(&fp.ty)?));
                    let exprs = self.kt_encode(&fp.ty, &n, false, &mut cx, true)?;
                    if *packed {
                        let leaves = self.leaves(&fp.ty, Dir::In)?;
                        args.extend(super::pack::pack_kotlin(&leaves, &exprs));
                    } else {
                        args.extend(exprs);
                    }
                }
                PPlan::Selector(s, split) => {
                    args.extend(self.selector_encode(s, &mut cx)?);
                    sig.push(if *split {
                        SigParam::Split(s.clone())
                    } else {
                        SigParam::Selector(s.clone())
                    });
                }
            }
        }
        // Tail parameters: accumulator, error handlers, builder / folder.
        let mut tail: Vec<(String, String)> = Vec::new();
        if let RPlan::Fold { .. } = plan.ret {
            tail.push(("acc".to_string(), "A".to_string()));
        }
        let je = format!("{}.JniErrorHandler", self.base_pkg);
        match &plan.err {
            EPlan::Domain { handler, .. } => {
                tail.push(("onBindingError".to_string(), format!("{je}<{ret}>")));
                tail.push(("onError".to_string(), format!("{handler}<{ret}>")));
            }
            _ => tail.push(("onError".to_string(), format!("{je}<{ret}>"))),
        }
        match &plan.ret {
            RPlan::Builder { iface, .. } => tail.push(("build".to_string(), format!("{iface}<R>"))),
            RPlan::Fold { iface, .. } => tail.push(("fold".to_string(), format!("{iface}<A>"))),
            _ => {}
        }
        let binding_handler = if matches!(plan.err, EPlan::Domain { .. }) {
            "onBindingError"
        } else {
            "onError"
        };

        // The call.
        let mut call_args = args.clone();
        match &plan.ret {
            RPlan::Sink { kt_sink, .. } => call_args.push(kt_sink.clone()),
            RPlan::Builder { .. } => call_args.push("build.asRaw()".to_string()),
            RPlan::Fold {
                columns_iface,
                params,
                leaves,
                ..
            } => {
                let raws: Vec<String> = leaves.iter().map(raw_name).collect();
                // The param decodes read the raw leaf names: bind each to the
                // element of its column.
                let rebinds: Vec<String> = leaves
                    .iter()
                    .zip(&raws)
                    .map(|(l, r)| match l.prim() {
                        Some(_) => format!("val {r} = __c_{r}[__i]"),
                        None => format!("val {r} = __c_{r}[__i] as {}", l.kt_raw()),
                    })
                    .collect();
                let decodes: Vec<String> = params.iter().map(|p| p.decode.clone()).collect();
                let mut col_params = vec!["n".to_string()];
                col_params.extend(raws.iter().map(|r| format!("__c_{r}")));
                call_args.push(format!(
                    "{columns_iface} {{ {} ->\n            var __a = acc\n            for (__i in 0 until n) {{\n                {}\n                __a = fold.run(__a{}{})\n            }}\n            __a\n        }}",
                    col_params.join(", "),
                    rebinds.join("\n                "),
                    if decodes.is_empty() { "" } else { ", " },
                    decodes.join(", ")
                ));
            }
            _ => {}
        }
        call_args.push("__bcap".to_string());
        if let EPlan::Domain { .. } = plan.err {
            call_args.push("__dcap".to_string());
        }
        let harness = self.harness_fqn();
        let mut call = format!("{harness}.{}({})", b.ext_name, call_args.join(", "));

        // Handles: closed checks, alias checks, locks, consumption.
        let mut pre = Vec::new();
        let handles: Vec<HandleSite> = cx.handles.clone();
        let fail = |msg: &str| format!("return {binding_handler}.run(\"{msg}\")");
        for h in &handles {
            if h.nullable {
                pre.push(format!(
                    "if ({}?.isClosed() == true) {}",
                    h.expr,
                    fail("Operation on a closed native handle.")
                ));
            } else {
                pre.push(format!(
                    "if ({}.isClosed()) {}",
                    h.expr,
                    fail("Operation on a closed native handle.")
                ));
            }
        }
        let consumed: Vec<&HandleSite> = handles.iter().filter(|h| h.consumed).collect();
        for (i, a) in consumed.iter().enumerate() {
            for c in consumed.iter().skip(i + 1) {
                if a.class.rust != c.class.rust {
                    continue;
                }
                let pa = if a.nullable {
                    format!("({}?.ptr ?: 0L)", a.expr)
                } else {
                    format!("{}.ptr", a.expr)
                };
                let pc = if c.nullable {
                    format!("({}?.ptr ?: 0L)", c.expr)
                } else {
                    format!("{}.ptr", c.expr)
                };
                pre.insert(
                    0,
                    format!(
                        "if (({pa} and -2L) != 0L && ({pa} and -2L) == ({pc} and -2L)) return {binding_handler}.run(\"Aliasing arguments: '{}' and '{}' are the same native resource; a consumed handle may not be passed twice in one call.\")",
                        a.label, c.label
                    ),
                );
            }
        }
        if !consumed.is_empty() {
            let marks = consumed
                .iter()
                .map(|h| {
                    if h.nullable {
                        format!("{}?.markConsumed()", h.expr)
                    } else {
                        format!("{}.markConsumed()", h.expr)
                    }
                })
                .collect::<Vec<_>>()
                .join("; ");
            call = format!("try {{\n            {call}\n        }} finally {{\n            {marks}\n        }}");
        }
        if self.b.handle_locks && !handles.is_empty() {
            let base = &self.base_pkg;
            if handles.iter().all(|h| !h.nullable) && handles.len() <= 3 {
                let hs = handles
                    .iter()
                    .map(|h| h.expr.clone())
                    .collect::<Vec<_>>()
                    .join(", ");
                call = format!("{base}.withSortedHandleLocks({hs}) {{\n        {call}\n    }}");
            } else {
                let adds = handles
                    .iter()
                    .map(|h| {
                        if h.nullable {
                            format!("{}?.let {{ __locks.add(it) }}", h.expr)
                        } else {
                            format!("__locks.add({})", h.expr)
                        }
                    })
                    .collect::<Vec<_>>()
                    .join("; ");
                call = format!(
                    "run {{\n        val __locks = ArrayList<{base}.NativeHandle>(); {adds}\n        {base}.withSortedHandleLocks(__locks) {{\n            {call}\n        }}\n    }}"
                );
            }
        }

        // Body.
        let mut body = pre;
        body.extend(cx.prelude.iter().cloned());
        body.push(format!(
            "val __bcap = {}.JniErrorHandlerCapture.acquire()",
            self.base_pkg
        ));
        if let EPlan::Domain { capture, .. } = &plan.err {
            body.push(format!("val __dcap = {capture}.acquire()"));
        }
        body.push(format!("val __ret = {call}"));
        body.push(format!(
            "if (__bcap.failed) return {binding_handler}.run(__bcap.ze0)"
        ));
        if let EPlan::Domain { params, leaves, .. } = &plan.err {
            let raws: Vec<String> = leaves.iter().map(raw_name).collect();
            let mut decodes: Vec<String> = params.iter().map(|p| p.decode.clone()).collect();
            // The handler params decode over the raw leaf names; read them
            // off the capture.
            let rebinds = raws
                .iter()
                .zip(leaves)
                .map(|(r, l)| {
                    if l.is_obj() && !l.nullable {
                        format!("val {r} = __dcap.{r}!!")
                    } else if l.is_obj() {
                        format!("val {r} = __dcap.{r}")
                    } else {
                        format!("val {r} = __dcap.{r}!!")
                    }
                })
                .collect::<Vec<_>>();
            for d in &mut decodes {
                *d = d.clone();
            }
            body.push(format!(
                "if (__dcap.failed) {{\n        {}\n        return onError.run({})\n    }}",
                rebinds.join("\n        "),
                decodes.join(", ")
            ));
        }
        let ret_expr = match &plan.ret {
            RPlan::Unit => "Unit".to_string(),
            RPlan::Direct { ty, .. } => self.kt_decode(ty, &["__ret".to_string()], false, 0)?,
            RPlan::Sink { .. } | RPlan::Builder { .. } | RPlan::Fold { .. } => {
                format!("__ret as {ret}")
            }
        };
        body.push(format!("return {ret_expr}"));

        // Signature.
        let is_member = !matches!(b.placement, Placement::Package);
        let overriding = match &b.placement {
            Placement::Method(c) => c.iface.is_some(),
            _ => false,
        };
        let vis = if overriding {
            "public override"
        } else {
            "public"
        };
        let suppress = if matches!(
            plan.ret,
            RPlan::Sink { .. } | RPlan::Builder { .. } | RPlan::Fold { .. }
        ) {
            "@Suppress(\"UNCHECKED_CAST\")\n"
        } else {
            ""
        };
        let mut sig_strs: Vec<String> = Vec::new();
        for p in &sig {
            match p {
                SigParam::Plain(n, t) => sig_strs.push(format!("{n}: {t}")),
                SigParam::Selector(s) | SigParam::Split(s) => {
                    for (n, t) in self.selector_params(s)? {
                        sig_strs.push(format!("{n}: {t}"));
                    }
                }
            }
        }
        for (n, t) in &tail {
            sig_strs.push(format!("{n}: {t}"));
        }
        let mut text = format!(
            "{suppress}{vis} fun {generics}{}({}): {ret} {{\n    {}\n}}\n",
            b.kt_name,
            sig_strs.join(", "),
            body.join("\n    ")
        );
        // Split overloads.
        if sig.iter().any(|p| matches!(p, SigParam::Split(_))) {
            let splits = sig
                .iter()
                .filter(|p| matches!(p, SigParam::Split(_)))
                .count();
            let head = format!("public fun {generics}");
            for o in self.split_overloads(&head, &b.kt_name, &sig, &tail, &ret, splits > 1)? {
                text.push('\n');
                text.push_str(&o);
            }
        }
        if let Some(c) = this_class {
            // Record the interface member signature.
            if c.iface.is_some() {
                let header = format!(
                    "fun {generics}{}({}): {ret}",
                    b.kt_name,
                    sig_strs.join(", ")
                );
                self.iface_members
                    .entry(c.rust.clone())
                    .or_default()
                    .push(header);
            }
        }
        let _ = is_member;
        Ok(text)
    }
}

/// Encode `out`'s wires as `jvalue`s and call the sink interface's `run`,
/// returning its result object.
fn call_sink(out: &Output, leaves: &[Leaf], iface: &str) -> TokenStream {
    let bind = out.bind();
    let values: Vec<TokenStream> = out
        .wires
        .iter()
        .zip(leaves)
        .map(|(w, l)| l.jvalue(&w.name.to_token_stream()))
        .collect();
    let desc = method_desc(leaves, "Ljava/lang/Object;");
    let fqn = iface.replace('.', "/");
    quote!({
        #bind
        static __S: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
        let __o = __S.call_object(env, #fqn, "run", #desc, &__sink, &[#(#values),*])?;
        ::core::result::Result::Ok(__o.into_raw())
    })
}

/// The Rust side of a function wrapper.
struct RustCb<'g, 'a> {
    g: &'g mut Gen<'a>,
    plan: &'g FnPlan,
}

impl FunctionCallbacks for RustCb<'_, '_> {
    type Error = crate::Error;

    fn param(&mut self, _func: &Function, param: &FlatParam) -> Res<Input> {
        let idx = self
            .plan
            .params
            .iter()
            .position(|p| match p {
                PPlan::Receiver(fp) | PPlan::Value(fp, _) => fp.name == param.name,
                PPlan::Selector(s, _) => s.param.name == param.name,
            })
            .expect("a planned parameter");
        match &self.plan.params[idx] {
            PPlan::Selector(s, _) => self.g.selector_input(s),
            PPlan::Receiver(fp) | PPlan::Value(fp, _) => {
                let root = names::bare(&fp.name);
                let mut input = self.g.rs_decode(&fp.ty, &root, 0)?;
                if let PPlan::Value(_, true) = &self.plan.params[idx] {
                    let leaves: Vec<Leaf> = self
                        .g
                        .leaves(&fp.ty, Dir::In)?
                        .into_iter()
                        .map(|l| l.under(&root))
                        .collect();
                    input = super::pack::pack_input(&root, &leaves, input);
                }
                let name = &fp.name;
                Ok(match self.g.kind(&fp.ty)? {
                    Kind::Str(Form::Slice)
                    | Kind::Bytes(Form::Slice)
                    | Kind::Bytes(Form::RefVec) => input.with_pass(quote!(&#name)),
                    Kind::Seq {
                        form: Form::Slice | Form::RefVec,
                        ..
                    } => input.with_pass(quote!(&#name)),
                    Kind::Ref { mutable: false, .. } => input.with_pass(quote!(&#name)),
                    Kind::Ref { mutable: true, .. } => {
                        return err(format!(
                            "`{}`: a `&mut` value parameter cannot cross from Kotlin",
                            fp.name
                        ))
                    }
                    _ => input,
                })
            }
        }
    }

    fn ret(&mut self, _func: &Function, _ret: &TypeRef) -> Res<Return> {
        let r = result_ident();
        let ok = self.g.rust_ret_value(&self.plan.ret, quote!(__ok))?;
        let whole = match &self.plan.err {
            EPlan::None => self.g.rust_ret_value(&self.plan.ret, r.to_token_stream())?,
            e => {
                let on_err = self.g.rust_error(e)?;
                quote!(match #r {
                    ::core::result::Result::Ok(__ok) => #ok,
                    ::core::result::Result::Err(__e) => #on_err,
                })
            }
        };
        let (ty, default) = match &self.plan.ret {
            RPlan::Unit => (None, quote!(())),
            RPlan::Direct { leaf, .. } => match leaf.prim() {
                Some(p) => (Some(p.rs()), p.rs_default()),
                None => (
                    Some(quote!(::prebindgen_jni_runtime::jni::sys::jobject)),
                    quote!(::core::ptr::null_mut()),
                ),
            },
            _ => (
                Some(quote!(::prebindgen_jni_runtime::jni::sys::jobject)),
                quote!(::core::ptr::null_mut()),
            ),
        };
        let wires = if matches!(
            self.plan.ret,
            RPlan::Sink { .. } | RPlan::Builder { .. } | RPlan::Fold { .. }
        ) {
            vec![Wire::new(
                format_ident!("__sink"),
                quote!(::prebindgen_jni_runtime::jni::objects::JObject<'a>),
            )]
        } else {
            Vec::new()
        };
        let rty = ty.clone().unwrap_or(quote!(()));
        let e = error_ident();
        Ok(Return {
            ty,
            wires,
            body: quote! {
                let __r: ::core::result::Result<#rty, ::std::string::String> = (|| #whole)();
                match __r {
                    ::core::result::Result::Ok(__v) => __v,
                    ::core::result::Result::Err(#e) => {
                        if !#e.is_empty() {
                            __jni_signal(env, &__error_sink, &#e);
                        }
                        #default
                    }
                }
            },
        })
    }

    fn fail(&mut self, _func: &Function, ret: &Return) -> TokenStream {
        let e = error_ident();
        let default = match &ret.ty {
            None => quote!(()),
            Some(t) if t.to_string().contains("jobject") => quote!(::core::ptr::null_mut()),
            Some(t) if t.to_string().contains("jdouble") || t.to_string().contains("jfloat") => {
                quote!(0.0)
            }
            Some(_) => quote!(0),
        };
        quote!({
            __jni_signal(env, &__error_sink, &#e);
            return #default;
        })
    }
}

#[allow(dead_code)]
fn unused(_: ClassKind, _: Delivery, _: Param) {}
