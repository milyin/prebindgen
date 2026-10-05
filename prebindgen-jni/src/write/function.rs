//! One bound function: its Rust extern, its `external fun` declaration and
//! its Kotlin wrapper — all three read from the same [`Binding`], so the
//! extern's parameter list and the declaration cannot disagree.

use prebindgen_flat::flat::{TypeKind, TypeRef};
use prebindgen_tools::{names, Access, Output, Shape, Wire};
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};

use super::{jni_symbol, Out};
use crate::{
    lower::{
        deliver::raw_name,
        kotlin::{HandleSite, KtEnc},
        kt_ident,
        leaf::{method_desc, Leaf, LeafTy, Prim},
        pack,
        select::SigParam,
        Dir, Param,
    },
    plan::{err, Binding, Callee, EPlan, PPlan, Placement, Plan, RPlan, Res},
};

/// A written Kotlin wrapper: its text, and its header for an interface.
pub(crate) struct Wrapper {
    pub text: String,
    pub header: String,
}

/// Write one binding: the extern into `out`, the wrapper returned.
pub(crate) fn function(plan: &Plan, out: &mut Out, b: &Binding) -> Res<Wrapper> {
    out.rust.push(extern_rust(plan, b)?);
    out.externs.push(extern_kotlin(plan, b)?);
    wrapper(plan, b)
}

// ── the Rust extern ─────────────────────────────────────────────────────

fn extern_rust(plan: &Plan, b: &Binding) -> Res<TokenStream> {
    let symbol = jni_symbol(&plan.harness_fqn(), &b.ext_name);
    let jobject = quote!(::prebindgen_jni_runtime::jni::objects::JObject<'a>);
    let callee = match &b.callee {
        Callee::Call(c) | Callee::Value(c) => c.clone(),
    };
    let mut cb = RustBoundary { plan, b };
    let ret = cb.ret()?;
    let fail = cb.fail();
    let mut params = vec![
        quote!(__env: ::prebindgen_jni_runtime::jni::JNIEnv<'a>),
        quote!(_class: ::prebindgen_jni_runtime::jni::objects::JClass<'a>),
    ];
    let mut stmts = Vec::new();
    let mut args = Vec::new();
    for p in &b.func.params {
        let name = &p.name;
        let Param { input, pass } = cb.param(name)?;
        params.extend(input.wires().into_iter().map(Wire::decl));
        if input.fallible {
            let r = input.result();
            stmts.push(quote!(let #name = match #r {
                ::core::result::Result::Ok(__v) => __v,
                ::core::result::Result::Err(__err) => { #fail }
            };));
        } else {
            let e = &input.expr;
            stmts.push(quote!(let #name = #e;));
        }
        args.push(pass.unwrap_or_else(|| quote!(#name)));
    }
    params.extend(ret.wires.iter().map(Wire::decl));
    params.push(quote!(__error_sink: #jobject));
    if let EPlan::Domain { .. } = b.err {
        params.push(quote!(__domain_sink: #jobject));
    }
    let call = match b.callee {
        Callee::Value(_) => callee,
        Callee::Call(_) => quote!(#callee(#(#args),*)),
    };
    let name = format_ident!("{}", symbol);
    let ret_ty = ret.ty.as_ref().map(|t| quote!(-> #t));
    let body = ret.body;
    Ok(quote! {
        #[no_mangle]
        #[allow(non_snake_case, unused_mut, unused_variables, unused_braces, unused_parens, unused_unsafe, dead_code, clippy::all)]
        pub unsafe extern "system" fn #name<'a>(#(#params),*) #ret_ty {
            let mut __env = __env; let env = &mut __env;
            #(#stmts)*
            let __result = #call;
            #body
        }
    })
}

struct Return {
    ty: Option<TokenStream>,
    wires: Vec<Wire<Leaf>>,
    body: TokenStream,
}

/// The JNI adapter's answers for one extern.
struct RustBoundary<'p, 'f> {
    plan: &'p Plan<'f>,
    b: &'p Binding,
}

impl RustBoundary<'_, '_> {
    fn ret(&mut self) -> Res<Return> {
        let (plan, b) = (self.plan, self.b);
        let r = format_ident!("__result");
        let whole = match &b.err {
            EPlan::None => rust_ret_value(plan, &b.ret, r.to_token_stream())?,
            e => {
                let ok = rust_ret_value(plan, &b.ret, quote!(__ok))?;
                let on_err = rust_error(plan, e)?;
                quote!(match #r {
                    ::core::result::Result::Ok(__ok) => #ok,
                    ::core::result::Result::Err(__e) => #on_err,
                })
            }
        };
        let (ty, default) = ret_wire(&b.ret);
        // The object a sink, builder or folder result is handed to.
        let sink = match &b.ret {
            RPlan::Sink { iface, .. } => Some(iface.clone()),
            RPlan::Builder { iface, .. } => Some(format!("{iface}Raw")),
            RPlan::Fold { columns_iface, .. } => Some(columns_iface.clone()),
            RPlan::Unit | RPlan::Direct { .. } => None,
        };
        let wires = sink
            .map(|fqn| Wire::new(format_ident!("__sink"), Leaf::new(LeafTy::Callback(fqn))))
            .into_iter()
            .collect();
        let rty = ty.clone().unwrap_or(quote!(()));
        let e = format_ident!("__err");
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

    fn param(&mut self, name: &syn::Ident) -> Res<Param> {
        let plan = self.plan;
        let p = self
            .b
            .params
            .iter()
            .find(|p| match p {
                PPlan::Receiver(fp) | PPlan::Value(fp, _) => fp.name == *name,
                PPlan::Selector(s, _) => s.param.name == *name,
            })
            .expect("a planned parameter");
        let (fp, packed) = match p {
            PPlan::Selector(s, _) => return plan.selector_input(s),
            PPlan::Receiver(fp) => (fp, false),
            PPlan::Value(fp, packed) => (fp, *packed),
        };
        let root = names::bare(&fp.name);
        let mut input = plan.rs_decode(&fp.ty, &root, 0)?;
        if packed {
            let leaves: Vec<Leaf> = plan
                .leaves(&fp.ty, Dir::In)?
                .into_iter()
                .map(|l| l.under(&root))
                .collect();
            input = pack::pack_input(&root, &leaves, input);
        }
        // A borrowed parameter lends the decoded value.
        let pass = match plan.shape(&fp.ty)? {
            Shape::Str {
                access: Access::Shared,
                ..
            }
            | Shape::Seq {
                access: Access::Shared,
                ..
            }
            | Shape::Ref {
                access: prebindgen_tools::Access::Shared,
                ..
            } => Some(quote!(&#name)),
            Shape::Ref { .. }
            | Shape::Str {
                access: Access::Exclusive,
                ..
            }
            | Shape::Seq {
                access: Access::Exclusive,
                ..
            } => {
                return err(format!(
                    "`{name}`: a `&mut` value parameter cannot cross from Kotlin"
                ))
            }
            _ => None,
        };
        Ok(Param { input, pass })
    }

    fn fail(&mut self) -> TokenStream {
        let e = format_ident!("__err");
        let (_, default) = ret_wire(&self.b.ret);
        quote!({
            __jni_signal(env, &__error_sink, &#e);
            return #default;
        })
    }
}

fn uses_sink(ret: &RPlan) -> bool {
    matches!(
        ret,
        RPlan::Sink { .. } | RPlan::Builder { .. } | RPlan::Fold { .. }
    )
}

/// The extern's return type, and the value it returns on failure.
fn ret_wire(ret: &RPlan) -> (Option<TokenStream>, TokenStream) {
    let object = (
        Some(quote!(::prebindgen_jni_runtime::jni::sys::jobject)),
        quote!(::core::ptr::null_mut()),
    );
    match ret {
        RPlan::Unit => (None, quote!(())),
        RPlan::Direct { leaf, .. } => match leaf.prim() {
            Some(p) => (Some(p.rs()), p.rs_default()),
            None => object,
        },
        _ => object,
    }
}

/// The encoding of a successful value per the return plan, as statements
/// ending in `Ok(<return wire>)`.
fn rust_ret_value(plan: &Plan, ret: &RPlan, value: TokenStream) -> Res<TokenStream> {
    Ok(match ret {
        RPlan::Unit => quote!({ let _ = #value; ::core::result::Result::Ok(()) }),
        RPlan::Direct { ty, leaf } => {
            let out = plan.rs_encode(ty, value, "r", 1)?;
            let bind = out.bind();
            let n = &out.wires()[0].name;
            let v = if leaf.is_obj() {
                quote!(#n.into_raw())
            } else {
                quote!(#n)
            };
            quote!({ #bind ::core::result::Result::Ok(#v) })
        }
        RPlan::Sink {
            ty, leaves, iface, ..
        } => call_sink(&plan.rs_encode(ty, value, "r", 1)?, leaves, iface),
        RPlan::Builder {
            whole,
            exp,
            iface,
            leaves,
            ..
        } => {
            let iface = format!("{iface}Raw");
            layered(whole, value, &mut |t, v| {
                let x = format_ident!("__x");
                let d = plan.deliver(t, x.to_token_stream(), "r", "", Some(exp), &[], false, 1)?;
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
        } => layered(whole, value, &mut |t, v| {
            let TypeKind::Vec(elem) = t.kind() else {
                return err(format!("`{t}`: a folded result must be a `Vec`"));
            };
            let e = format_ident!("__x");
            let d = plan.deliver(elem, e.to_token_stream(), "r", "", Some(exp), &[], false, 1)?;
            let cols = columns(t, &d.output, leaves, &e, quote!(__items));
            let mut col_leaves = vec![Leaf::new(LeafTy::Prim(Prim::I))];
            col_leaves.extend(leaves.iter().map(Leaf::column));
            let call = call_sink(&cols, &col_leaves, columns_iface);
            Ok(
                quote!({ let __items: ::std::vec::Vec<_> = ::core::iter::IntoIterator::into_iter(#v).collect(); #call }),
            )
        })?,
    })
}

/// Walk the layers a builder or folder absorbs — `Box`, the first `Option`
/// (absent: a null result) — down to the value `core` delivers (the `Vec`
/// itself, for a fold).
fn layered(
    ty: &TypeRef,
    value: TokenStream,
    core: &mut dyn FnMut(&TypeRef, TokenStream) -> Res<TokenStream>,
) -> Res<TokenStream> {
    match ty.kind() {
        TypeKind::Boxed(t) => layered(t, quote!((*#value)), core),
        TypeKind::Optional(t) => {
            let inner = unbox(t, quote!(__some), core)?;
            Ok(quote!(match #value {
                ::core::option::Option::Some(__some) => #inner,
                ::core::option::Option::None => ::core::result::Result::Ok(::core::ptr::null_mut()),
            }))
        }
        _ => core(ty, value),
    }
}

/// Below the `Option`: only `Box` layers remain.
fn unbox(
    ty: &TypeRef,
    value: TokenStream,
    core: &mut dyn FnMut(&TypeRef, TokenStream) -> Res<TokenStream>,
) -> Res<TokenStream> {
    match ty.kind() {
        TypeKind::Boxed(t) => unbox(t, quote!((*#value)), core),
        _ => core(ty, value),
    }
}

/// Columns of a sequence of deliveries: a count and one array per leaf.
fn columns(
    ty: &TypeRef,
    elem: &Output<Leaf>,
    leaves: &[Leaf],
    x: &syn::Ident,
    items: TokenStream,
) -> Output<Leaf> {
    let rt = quote!(::prebindgen_jni_runtime);
    let mut setup = Vec::new();
    let mut pushes = Vec::new();
    let mut finals = vec![quote!(__n as i32)];
    let mut wires = vec![Wire::new(
        format_ident!("__cn"),
        Leaf::new(LeafTy::Prim(Prim::I)),
    )];
    for (k, (l, w)) in leaves.iter().zip(elem.wires()).enumerate() {
        let col = format_ident!("__col{}", k);
        let local = &w.name;
        wires.push(Wire::new(format_ident!("__colw{}", k), l.column()));
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
    let bind = elem.bind();
    Output::seq(
        ty,
        wires,
        elem.form.clone(),
        quote!({
            let __n = #items.len();
            #(#setup)*
            for (__i, #x) in #items.into_iter().enumerate() {
                #bind
                #(#pushes)*
            }
            (#(#finals),*)
        }),
    )
    .mark_fallible()
}

/// What an `Err` does: a message for the binding-error channel, or a
/// delivery to the typed handler (the empty message then says it was
/// delivered).
fn rust_error(plan: &Plan, e: &EPlan) -> Res<TokenStream> {
    Ok(match e {
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
            let d = plan.deliver(ty, quote!(__e), "e", "", None, &[], false, 1)?;
            let bind = d.output.bind();
            let values = d
                .output
                .wires()
                .into_iter()
                .zip(leaves)
                .map(|(w, l)| l.jvalue(&w.name.to_token_stream()));
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

/// Encode `out`'s wires as `jvalue`s and call the sink interface's `run`,
/// returning its result object.
fn call_sink(out: &Output<Leaf>, leaves: &[Leaf], iface: &str) -> TokenStream {
    let bind = out.bind();
    let values = out
        .wires()
        .into_iter()
        .zip(leaves)
        .map(|(w, l)| l.jvalue(&w.name.to_token_stream()));
    let desc = method_desc(leaves, "Ljava/lang/Object;");
    let fqn = iface.replace('.', "/");
    quote!({
        #bind
        static __S: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
        let __o = __S.call_object(env, #fqn, "run", #desc, &__sink, &[#(#values),*])?;
        ::core::result::Result::Ok(__o.into_raw())
    })
}

// ── the `external fun` ──────────────────────────────────────────────────

/// The extern's trailing object parameters: sink, error sink, domain sink.
fn trailing(b: &Binding) -> Vec<&'static str> {
    let mut v = Vec::new();
    if uses_sink(&b.ret) {
        v.push("sink");
    }
    v.push("errorSink");
    if let EPlan::Domain { .. } = b.err {
        v.push("domainSink");
    }
    v
}

fn extern_kotlin(plan: &Plan, b: &Binding) -> Res<String> {
    let mut params = Vec::new();
    for p in &b.params {
        for l in plan.param_leaves(p)? {
            params.push(format!("{}: {}", raw_name(&l), l.kt_raw()));
        }
    }
    params.extend(trailing(b).iter().map(|n| format!("{n}: Any")));
    let ret = match &b.ret {
        RPlan::Unit => String::new(),
        RPlan::Direct { leaf, .. } => format!(": {}", leaf.kt_raw()),
        _ => ": Any?".to_string(),
    };
    Ok(format!(
        "    external fun {}({}){ret}",
        b.ext_name,
        params.join(", ")
    ))
}

// ── the Kotlin wrapper ──────────────────────────────────────────────────

/// The Kotlin type the wrapper returns, and its type parameters.
fn kt_ret(plan: &Plan, ret: &RPlan) -> Res<(String, String)> {
    let generic = |optional: bool, t: &str| {
        let r = if optional {
            format!("{t}?")
        } else {
            t.to_string()
        };
        (r, format!("<{t}> "))
    };
    Ok(match ret {
        RPlan::Unit => ("Unit".to_string(), String::new()),
        RPlan::Direct { ty, .. } | RPlan::Sink { ty, .. } => (plan.kt_type(ty)?, String::new()),
        RPlan::Builder { optional, .. } => generic(*optional, "R"),
        RPlan::Fold { optional, .. } => generic(*optional, "A"),
    })
}

fn wrapper(plan: &Plan, b: &Binding) -> Res<Wrapper> {
    let base = &plan.base_pkg;
    let (ret, generics) = kt_ret(plan, &b.ret)?;
    let mut cx = KtEnc::default();
    let mut sig: Vec<SigParam> = Vec::new();
    let mut args: Vec<String> = Vec::new();
    for p in &b.params {
        match p {
            PPlan::Receiver(fp) => {
                args.extend(plan.kt_encode(&fp.ty, "this", false, &mut cx, true)?);
            }
            PPlan::Value(fp, packed) => {
                let n = kt_ident(&names::camel(&names::bare(&fp.name)));
                sig.push(SigParam::Plain(n.clone(), plan.kt_type(&fp.ty)?));
                let exprs = plan.kt_encode(&fp.ty, &n, false, &mut cx, true)?;
                if *packed {
                    args.extend(pack::pack_kotlin(&plan.leaves(&fp.ty, Dir::In)?, &exprs));
                } else {
                    args.extend(exprs);
                }
            }
            PPlan::Selector(s, split) => {
                args.extend(plan.selector_encode(s, &mut cx)?);
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
    if let RPlan::Fold { .. } = b.ret {
        tail.push(("acc".to_string(), "A".to_string()));
    }
    let je = format!("{base}.JniErrorHandler");
    match &b.err {
        EPlan::Domain { handler, .. } => {
            tail.push(("onBindingError".to_string(), format!("{je}<{ret}>")));
            tail.push(("onError".to_string(), format!("{handler}<{ret}>")));
        }
        _ => tail.push(("onError".to_string(), format!("{je}<{ret}>"))),
    }
    match &b.ret {
        RPlan::Builder { iface, .. } => tail.push(("build".to_string(), format!("{iface}<R>"))),
        RPlan::Fold { iface, .. } => tail.push(("fold".to_string(), format!("{iface}<A>"))),
        _ => {}
    }
    let binding_handler = match b.err {
        EPlan::Domain { .. } => "onBindingError",
        _ => "onError",
    };

    // The call.
    let mut call_args = args;
    match &b.ret {
        RPlan::Sink { kt_sink, .. } => call_args.push(kt_sink.clone()),
        RPlan::Builder { .. } => call_args.push("build.asRaw()".to_string()),
        RPlan::Fold {
            columns_iface,
            params,
            leaves,
            ..
        } => {
            let raws: Vec<String> = leaves.iter().map(raw_name).collect();
            // The parameters decode over the raw leaf names: bind each to the
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
    if let EPlan::Domain { .. } = b.err {
        call_args.push("__dcap".to_string());
    }
    let call = format!(
        "{}.{}({})",
        plan.harness_fqn(),
        b.ext_name,
        call_args.join(", ")
    );
    let (pre, call) = guard_handles(plan, &cx.handles, call, binding_handler);

    // Body.
    let mut body = pre;
    body.extend(cx.prelude);
    body.push(format!(
        "val __bcap = {base}.JniErrorHandlerCapture.acquire()"
    ));
    if let EPlan::Domain { capture, .. } = &b.err {
        body.push(format!("val __dcap = {capture}.acquire()"));
    }
    body.push(format!("val __ret = {call}"));
    body.push(format!(
        "if (__bcap.failed) return {binding_handler}.run(__bcap.ze0)"
    ));
    if let EPlan::Domain { params, leaves, .. } = &b.err {
        // The handler's parameters decode over the raw leaf names: read them
        // off the capture.
        let rebinds: Vec<String> = leaves
            .iter()
            .map(|l| {
                let r = raw_name(l);
                if l.is_obj() && l.nullable {
                    format!("val {r} = __dcap.{r}")
                } else {
                    format!("val {r} = __dcap.{r}!!")
                }
            })
            .collect();
        let decodes: Vec<String> = params.iter().map(|p| p.decode.clone()).collect();
        body.push(format!(
            "if (__dcap.failed) {{\n        {}\n        return onError.run({})\n    }}",
            rebinds.join("\n        "),
            decodes.join(", ")
        ));
    }
    let ret_expr = match &b.ret {
        RPlan::Unit => "Unit".to_string(),
        RPlan::Direct { ty, .. } => plan.kt_decode(ty, &["__ret".to_string()], false, 0)?,
        _ => format!("__ret as {ret}"),
    };
    body.push(format!("return {ret_expr}"));

    // Signature.
    let vis = match &b.placement {
        Placement::Method(c) if c.iface.is_some() => "public override",
        _ => "public",
    };
    let suppress = if uses_sink(&b.ret) {
        "@Suppress(\"UNCHECKED_CAST\")\n"
    } else {
        ""
    };
    let mut sig_strs: Vec<String> = Vec::new();
    for p in &sig {
        match p {
            SigParam::Plain(n, t) => sig_strs.push(format!("{n}: {t}")),
            SigParam::Selector(s) | SigParam::Split(s) => {
                for (n, t) in plan.selector_params(s)? {
                    sig_strs.push(format!("{n}: {t}"));
                }
            }
        }
    }
    sig_strs.extend(tail.iter().map(|(n, t)| format!("{n}: {t}")));
    let mut text = format!(
        "{suppress}{vis} fun {generics}{}({}): {ret} {{\n    {}\n}}\n",
        b.kt_name,
        sig_strs.join(", "),
        body.join("\n    ")
    );
    // Split overloads.
    let splits = sig
        .iter()
        .filter(|p| matches!(p, SigParam::Split(_)))
        .count();
    if splits > 0 {
        let head = format!("public fun {generics}");
        for o in plan.split_overloads(&head, &b.kt_name, &sig, &tail, &ret, splits > 1)? {
            text.push('\n');
            text.push_str(&o);
        }
    }
    let header = format!(
        "fun {generics}{}({}): {ret}",
        b.kt_name,
        sig_strs.join(", ")
    );
    Ok(Wrapper { text, header })
}

/// The checks and locks around a call that passes handles: each must be
/// open, a consumed one may not appear twice, every one is locked (in
/// address order) for the call, and a consumed one is marked afterwards.
/// Returns the statements to run first and the guarded call.
fn guard_handles(
    plan: &Plan,
    handles: &[HandleSite],
    mut call: String,
    binding_handler: &str,
) -> (Vec<String>, String) {
    let base = &plan.base_pkg;
    let fail = |msg: &str| format!("return {binding_handler}.run(\"{msg}\")");
    let ptr = |h: &HandleSite| {
        if h.nullable {
            format!("({}?.ptr ?: 0L)", h.expr)
        } else {
            format!("{}.ptr", h.expr)
        }
    };
    let mut pre = Vec::new();
    let consumed: Vec<&HandleSite> = handles.iter().filter(|h| h.consumed).collect();
    for (i, a) in consumed.iter().enumerate() {
        for c in &consumed[i + 1..] {
            if a.class.rust != c.class.rust {
                continue;
            }
            let (pa, pc) = (ptr(a), ptr(c));
            pre.push(format!(
                "if (({pa} and -2L) != 0L && ({pa} and -2L) == ({pc} and -2L)) return {binding_handler}.run(\"Aliasing arguments: '{}' and '{}' are the same native resource; a consumed handle may not be passed twice in one call.\")",
                a.expr, c.expr
            ));
        }
    }
    for h in handles {
        let closed = if h.nullable {
            format!("{}?.isClosed() == true", h.expr)
        } else {
            format!("{}.isClosed()", h.expr)
        };
        pre.push(format!(
            "if ({closed}) {}",
            fail("Operation on a closed native handle.")
        ));
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
        call = format!(
            "try {{\n            {call}\n        }} finally {{\n            {marks}\n        }}"
        );
    }
    if plan.handle_locks && !handles.is_empty() {
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
    (pre, call)
}
