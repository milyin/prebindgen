//! Extern `"C"` JNI wrapper functions: signature lowering, input
//! params, and the expanded-param path.

use super::*;
use crate::api::core::types_util::result_ok_type;

struct OutputLowering<'a> {
    entry: Option<&'a crate::api::core::registry::TypeEntry<KotlinMeta>>,
    wire_return: TokenStream,
    on_err: TokenStream,
}

pub(crate) fn emit_jni_function_wrapper(
    ext: &JniGen,
    f: &syn::ItemFn,
    registry: &Registry<KotlinMeta>,
) -> TokenStream {
    emit_jni_function_wrapper_with_callee(ext, f, registry, None)
}

/// The synthetic nullary getter signature a declared const is emitted
/// through: `pub fn const_get_<ident_lower>() -> <const ty>`. Both sides —
/// the Rust extern ([`JniGen::on_const`] via
/// [`emit_jni_function_wrapper_with_callee`]) and the Kotlin `val`
/// initializer (`render_const_val`) — derive the extern symbol from this one
/// ident, so they stay in sync by construction. The body is never used.
pub(crate) fn const_getter_fn(c: &syn::ItemConst) -> syn::ItemFn {
    let ident = format_ident!("const_get_{}", c.ident.to_string().to_lowercase());
    let ty = &c.ty;
    syn::parse_quote! {
        pub fn #ident() -> #ty {
            unimplemented!()
        }
    }
}

/// A const whose (peeled) type is a declared opaque handle is rejected: an
/// eagerly-initialized shared closeable `val` is semantically wrong (whose
/// `close()` is it?). Expose a factory function instead — the established
/// idiom (e.g. zenoh's `encoding_const_*` companion factories).
pub(crate) fn reject_handle_const(ext: &JniGen, c: &syn::ItemConst) {
    reject_handle_constant_type(ext, &c.ty, "const", &c.ident.to_string());
}

/// The constant-value handle check shared by both constant kinds: peel
/// `&`/`Option`/`Vec` layers off `ty` and reject if what remains is a
/// declared opaque handle. `what`/`ident` shape the error message
/// (`const MAX_LEN` / `constant fn encoding_const_x_str`).
pub(crate) fn reject_handle_constant_type(ext: &JniGen, ty: &syn::Type, what: &str, name: &str) {
    let mut ty = ty.clone();
    loop {
        if let syn::Type::Reference(r) = &ty {
            ty = (*r.elem).clone();
            continue;
        }
        if let Some(inner) = option_inner_type(&ty) {
            ty = inner;
            continue;
        }
        if let Some(inner) = vec_inner_type(&ty) {
            ty = inner;
            continue;
        }
        break;
    }
    let key = TypeKey::from_type(&ty);
    let is_handle = ext
        .types
        .get(&key)
        .and_then(|cfg| cfg.opaque.as_ref())
        .is_some();
    assert!(
        !is_handle,
        "{what} `{name}`: type `{}` is a declared opaque handle — a shared closeable Kotlin `val` is \
         not supported. Expose a `#[prebindgen]` factory function returning the constant and \
         declare it as a companion constructor instead.",
        key.as_str()
    );
}

/// Validates a [`ConstDecl::fun`] declaration against the real
/// signature: the fn must be **nullary** (a constant has no inputs), must
/// not return a `Result` (a domain-fallible value is not a constant — and
/// the `val` initializer's throwing `JniErrorHandler` only fits the
/// infallible wrapper shape), and its return type must not peel to a
/// declared opaque handle (same rationale as [`reject_handle_const`]).
pub(crate) fn validate_constant_fn(ext: &JniGen, f: &syn::ItemFn) {
    assert!(
        f.sig.inputs.is_empty(),
        "constant fn `{}`: takes {} parameter(s) — a function-backed constant must be nullary \
         (declare it with `.fun(...)` instead if it is a real function)",
        f.sig.ident,
        f.sig.inputs.len()
    );
    if let syn::ReturnType::Type(_, ty) = &f.sig.output {
        assert!(
            result_ok_type(ty).is_none(),
            "constant fn `{}`: returns a `Result` — a function-backed constant must be \
             infallible (declare it with `.fun(...)` instead if it can fail)",
            f.sig.ident
        );
        reject_handle_constant_type(ext, ty, "constant fn", &f.sig.ident.to_string());
    }
}

/// The synthetic nullary getter signature an **expression constant**
/// ([`ConstExprDecl`](crate::lang::ConstExprDecl)) is emitted through:
/// `pub fn const_get_<val_name_lower>() -> <ty>` — the same convention as
/// const-backed getters, so both sides derive the extern symbol from the one
/// val name. The body is never used.
pub(crate) fn const_expr_getter_fn(kotlin_name: &str, ty: &syn::Type) -> syn::ItemFn {
    let ident = format_ident!("const_get_{}", kotlin_name.to_lowercase());
    syn::parse_quote! {
        pub fn #ident() -> #ty {
            unimplemented!()
        }
    }
}

/// Validates an expression constant's declared value type (checked on both
/// write paths): not a `Result` (a domain-fallible value is not a constant),
/// not (peeled to) a declared opaque handle.
pub(crate) fn validate_constant_expr(ext: &JniGen, kotlin_name: &str, ty: &syn::Type) {
    assert!(
        result_ok_type(ty).is_none(),
        "constant expr `{kotlin_name}`: type is a `Result` — an expression constant must be \
         infallible (declare a real function with `.fun(...)` instead if it can fail)"
    );
    reject_handle_constant_type(ext, ty, "constant expr", kotlin_name);
}

/// [`emit_jni_function_wrapper`] with the raw callee expression overridable:
/// `None` = the ordinary `<origin module>::<fn ident>(args)` call; `Some(e)`
/// splices `e` verbatim as the value the output phase converts. Used by the
/// const getter emission (`JniGen::on_const`), whose synthetic nullary `f`
/// carries the signature while the value comes from
/// `<origin module>::<CONST_IDENT>` — a path, not a call.
pub(crate) fn emit_jni_function_wrapper_with_callee(
    ext: &JniGen,
    f: &syn::ItemFn,
    registry: &Registry<KotlinMeta>,
    callee: Option<syn::Expr>,
) -> TokenStream {
    let original_ident = &f.sig.ident;
    let wrapper_ident = mangle_jni_name(ext, original_ident);

    let mut wire_params: Vec<TokenStream> = Vec::new();
    // Each entry is a per-input decode statement. Fallible decodes are
    // `match`-arms that, on `Err`, call `signal_error(&mut env,
    // &__error_sink, &__e)` (invoking the caller's Kotlin sink instead of
    // throwing a JVM exception) and `return <sentinel>;`.
    let mut prelude: Vec<TokenStream> = Vec::new();
    let mut call_args: Vec<TokenStream> = Vec::new();

    // Output is resolved first so the per-input `match`-arms can splice
    // the function's sentinel into their early-`return` path.
    let return_ty = fn_return_type(f);
    // Output (data) expansion: when output expansion was declared for this
    // function, the return value is decomposed by the deconstructor. Two
    // deliveries:
    //   * `Callback` (`deconstruct_output`): the leaves are delivered to a
    //     foreign builder/fold lambda — the wrapper's wire return is the
    //     lambda's `JObject` result (no `output_entry`; see `emit_unfold_delivery`).
    //   * `Return` (`convert_output`): the single decomposed value is **returned**
    //     directly through its ordinary output converter — the wrapper behaves
    //     exactly like a normal function whose return type is `convert_out_ty`.
    use crate::api::core::unfold::Delivery;
    let unfold_plan = registry.unfold_plans.get(original_ident);
    let is_convert = unfold_plan.is_some_and(|p| p.delivery == Delivery::Return);
    // Error-position expansion: when the fn returns `Result<T, E>` and an error
    // plan is declared, the **`?`** is applied here — the extern peels the
    // `Result` (Err arm decomposes `E` into the `ze` leaves and invokes the
    // error callback), and the success path uses `T`'s converter (not the
    // `Result<T, E>` rank-2 wrapper). `n_ze` = the error leaf count (the callback
    // arity after the fixed `je`).
    let error_plan = registry.error_plans.get(original_ident);
    let n_ze = error_plan.map_or(0, |p| p.leaves.len());
    let output = lower_output(
        registry,
        original_ident,
        &return_ty,
        unfold_plan,
        error_plan,
    );
    let output_entry = output.entry;
    let wire_return = output.wire_return;
    let on_err = output.on_err;

    // Input parameters: look up converter for the param type AS WRITTEN.
    // No strip — a `&T` param looks up `&T`'s entry (which the `& _`
    // rank-1 handler resolved by sharing `T`'s function). Call site adds
    // `&decoded` only for `&T`-shaped originals; that's a Rust call-
    // convention concern, not a converter concern.
    for input in &f.sig.inputs {
        let Some((wp, pre, call_arg)) =
            emit_input_param(ext, registry, original_ident, input, &on_err)
        else {
            continue;
        };
        wire_params.extend(wp);
        prelude.extend(pre);
        call_args.push(call_arg);
    }

    let raw_call = match &callee {
        Some(e) => quote!(#e),
        None => {
            let call_module = ext.fn_module(registry, original_ident);
            quote!(#call_module::#original_ident(#(#call_args),*))
        }
    };
    // For `convert_output` (Return), the value the output converter sees is the
    // **deconstructed** single value (the converter's accessor applied to the
    // raw return, lifted through the shape) — not the raw return. Build that
    // block so the normal output phase converts it. `Decompose` ⇒ `acc(raw)`;
    // `Optional` ⇒ `raw.map(|inner| acc(inner))`.
    let call_expr: TokenStream = if is_convert {
        use crate::api::core::unfold::UnfoldShape;
        let plan = unfold_plan.expect("is_convert ⇒ plan");
        let leaf = &plan.leaves[0];
        let by_ref = plan.by_ref;
        let compose = |base: TokenStream, base_is_ref: bool| -> TokenStream {
            let mut e = if base_is_ref { base } else { quote!(&#base) };
            for a in &leaf.path {
                let m = ext.fn_module(registry, a);
                e = quote!(#m::#a(#e));
            }
            e
        };
        match &plan.shape {
            UnfoldShape::Optional((), _) => {
                let inner = compose(quote!(__inner), by_ref);
                quote!({
                    let __cvsrc = #raw_call;
                    __cvsrc.map(|__inner| #inner)
                })
            }
            _ => {
                let v = compose(quote!(__cvsrc), by_ref);
                quote!({
                    let __cvsrc = #raw_call;
                    #v
                })
            }
        }
    } else if let Some(ep) = error_plan {
        // `Result<T, E>` peel (the automatic `?`): success ⇒ `T`; on `Err(e)`,
        // decompose `e` into the `ze` leaves — through the SAME shared leaf
        // encoder every output/callback delivery uses (typed jvalues, handle
        // wraps, Option-nested accessor unwrap) — and invoke the error
        // callback with `je = None` (a domain error, not a binding one), then
        // return the sentinel. A failure while ENCODING the error itself
        // degrades to a binding error: `je` = message, ze = defaults. The
        // success `T` flows into the normal output phase.
        let eze_idents: Vec<syn::Ident> = (0..ep.leaves.len())
            .map(|i| format_ident!("__eze{}", i))
            .collect();
        let ze_fail = |msg: TokenStream| -> TokenStream {
            quote! {
                let __zd = __ze_defaults(&mut env);
                signal_error(
                    &mut env, &__error_sink,
                    &__SINK_MID, __SINK_FQN, __SINK_DESCR,
                    ::core::option::Option::Some(&#msg),
                    &__zd,
                );
                return #on_err;
            }
        };
        let (ze_stmts, ze_args) =
            encode_plan_leaves(ext, registry, ep, &eze_idents, &quote!(__de), &ze_fail);
        quote! {
            match #raw_call {
                ::core::result::Result::Ok(__v) => __v,
                ::core::result::Result::Err(__de) => {
                    #ze_stmts
                    signal_error(
                        &mut env, &__error_sink,
                        &__SINK_MID, __SINK_FQN, __SINK_DESCR,
                        ::core::option::Option::None,
                        &[#(#ze_args),*],
                    );
                    return #on_err;
                }
            }
        }
    } else {
        raw_call
    };

    // Output phase. Three shapes:
    //   * `Callback` output expansion: decompose the return value and deliver the
    //     leaves to the foreign builder/fold (`__builder` / `__acc`+`__fold`).
    //   * `Return` output expansion (convert) and normal returns: every output
    //     converter returns `Result<wire, <err_type>>`; run pre_stages then the
    //     wire-facing converter, routing each `Err` through `signal_error`. (For
    //     convert, `call_expr` above already deconstructed the value.)
    let mut builder_param: Option<TokenStream> = None;
    let output_phase: TokenStream = if let (Some(plan), false) = (unfold_plan, is_convert) {
        // Iterable folds: two params (`__acc` accumulator + `__fold` callback).
        // Decompose/Optional: a single `__builder` callback.
        builder_param = Some(unfold_builder_param(plan));
        emit_unfold_delivery(ext, registry, plan, &call_expr, &on_err)
    } else {
        let output_entry = output_entry.expect("normal path has an output entry");
        let mut phase: TokenStream = quote! { let __out = #call_expr; };
        let mut prev_out: TokenStream = quote!(__out);
        // Pre_stages run in forward order BEFORE the wire-facing function:
        // rust → pre_stages[0] → … → pre_stages[N-1] → function → wire.
        for (i, stage) in output_entry.output_stage_order() {
            let stage_fn = &stage.function.sig.ident;
            let next_ident = format_ident!("__out_s{}", i);
            phase.extend(quote! {
                let #next_ident = match #stage_fn(&mut env, #prev_out) {
                    ::core::result::Result::Ok(__v) => __v,
                    ::core::result::Result::Err(__e) => {
                        let __zd = __ze_defaults(&mut env); signal_error(&mut env, &__error_sink, &__SINK_MID, __SINK_FQN, __SINK_DESCR, ::core::option::Option::Some(&__e.to_string()), &__zd);
                        return #on_err;
                    }
                };
            });
            prev_out = quote!(#next_ident);
        }
        let conv_out = output_entry.converter_ident().clone();
        phase.extend(quote! {
            match #conv_out(&mut env, #prev_out) {
                ::core::result::Result::Ok(__w) => __w,
                ::core::result::Result::Err(__e) => {
                    let __zd = __ze_defaults(&mut env); signal_error(&mut env, &__error_sink, &__SINK_MID, __SINK_FQN, __SINK_DESCR, ::core::option::Option::Some(&__e.to_string()), &__zd);
                    #on_err
                }
            }
        });
        phase
    };

    // `__ze_defaults` — the typed default ze values passed at **binding**
    // error sites (a `JniError`, where `je` carries the message). The handler
    // interface types its ze exactly like a builder's leaves, so the defaults
    // must be valid at those types: zeroed jvalue for raw primitives, `""`,
    // an empty byte array, a closed (`ptr = 0`) handle instance, JVM null for
    // plan-nullable leaves. Built LAZILY (the closure runs only on the cold
    // error path); in scope for the prelude + every helper-generated
    // `signal_error` call.
    let ze_default_exprs: Vec<TokenStream> = error_plan
        .map(|ep| default_ze_jvalues(ext, registry, ep))
        .unwrap_or_default();
    debug_assert_eq!(ze_default_exprs.len(), n_ze);
    // The error sink is a typed `<Err>Handler` / `JniErrorHandler` fun
    // interface; its `run` method ID is resolved once per process on the
    // interface class (the sink instance differs per call). The trio is in
    // scope for every `signal_error` call the prelude/output phases emit.
    let sink_spec = onerror_iface_spec(ext, registry, original_ident).unwrap_or_else(|| {
        panic!(
            "jnigen: cannot derive the onError handler interface for `{}`",
            original_ident
        )
    });
    let sink_fqn_lit = syn::LitStr::new(&sink_spec.raw_slash_fqn(), Span::call_site());
    let sink_descr_lit = syn::LitStr::new(&sink_spec.descr, Span::call_site());
    let ze_defaults_setup = quote! {
        #[allow(unused_variables)]
        let __ze_defaults = |env: &mut jni::JNIEnv| -> ::std::vec::Vec<jni::sys::jvalue> {
            ::std::vec![#(#ze_default_exprs),*]
        };
        #[allow(non_upper_case_globals)]
        static __SINK_MID: ::prebindgen::lang::CachedIfaceMethod =
            ::prebindgen::lang::CachedIfaceMethod::new();
        const __SINK_FQN: &str = #sink_fqn_lit;
        const __SINK_DESCR: &str = #sink_descr_lit;
    };

    // The trailing `__error_sink` param is the foreign **error callback** (a
    // function type `(je: String?, ze…) -> R`); the wrapper passes a capture.
    // Declared last so the wire param order matches the Kotlin `external fun`.
    quote! {
        #[no_mangle]
        #[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
        pub unsafe extern "C" fn #wrapper_ident<'a>(
            mut env: jni::JNIEnv<'a>,
            _class: jni::objects::JClass<'a>,
            #(#wire_params,)*
            #builder_param
            __error_sink: jni::objects::JObject<'a>,
        ) -> #wire_return {
            #ze_defaults_setup
            #(#prelude)*
            #output_phase
        }
    }
}

fn fn_return_type(f: &syn::ItemFn) -> syn::Type {
    match &f.sig.output {
        syn::ReturnType::Default => syn::parse_quote!(()),
        syn::ReturnType::Type(_, ty) => (**ty).clone(),
    }
}

fn lower_output<'a>(
    registry: &'a Registry<KotlinMeta>,
    original_ident: &syn::Ident,
    return_ty: &syn::Type,
    unfold_plan: Option<&crate::api::core::unfold::UnfoldPlan>,
    error_plan: Option<&crate::api::core::unfold::UnfoldPlan>,
) -> OutputLowering<'a> {
    let ok_ty = error_plan.and_then(|_| result_ok_type(return_ty));
    // The output converter to route through: the converted single value for
    // `Return`, the `Result` Ok type when peeling, the function's own return for
    // a normal fn, none for `Callback`.
    let target_ty = output_target_type(return_ty, unfold_plan, ok_ty.as_ref());
    let entry = target_ty.as_ref().map(|ty| {
        registry.output_entry(ty).unwrap_or_else(|| {
            panic!(
                "JniGen::on_function: return type `{}` of `{}` has no registered output \
                 converter — register one via `JniGen::output_wrapper(pat, |…| Some((ty, exc, body)))` \
                 (exc = `None` for non-throwing, `Some(parse_quote!(<full path>))` \
                  to bind a domain exception)",
                TypeKey::from_type(ty),
                original_ident,
            )
        })
    });
    let wire_ty = match entry {
        Some(e) => e.destination.clone(),
        None => syn::parse_quote!(jni::objects::JObject),
    };
    let wire_return = annotate_jobject_with_lifetime(&wire_ty, "a").to_token_stream();
    let on_err = sentinel_for_wire(&wire_ty);
    OutputLowering {
        entry,
        wire_return,
        on_err,
    }
}

fn output_target_type(
    return_ty: &syn::Type,
    unfold_plan: Option<&crate::api::core::unfold::UnfoldPlan>,
    ok_ty: Option<&syn::Type>,
) -> Option<syn::Type> {
    use crate::api::core::unfold::Delivery;

    match unfold_plan {
        Some(p) if p.delivery == Delivery::Return => Some(
            p.convert_out_ty
                .clone()
                .expect("Return delivery carries convert_out_ty"),
        ),
        Some(_) => None,
        None => Some(ok_ty.cloned().unwrap_or_else(|| return_ty.clone())),
    }
}

fn unfold_builder_param(plan: &crate::api::core::unfold::UnfoldPlan) -> TokenStream {
    // An `Iterable` fold (incl. `Option<Vec<T>>`) takes `(acc, fold)`; every
    // other delivery takes a single `build`.
    if super::render::is_iterable_fold(&plan.shape) {
        quote!(__acc: jni::objects::JObject<'a>, __fold: jni::objects::JObject<'a>,)
    } else {
        quote!(__builder: jni::objects::JObject<'a>,)
    }
}

/// Decode one source-fn input parameter: look up its converter for the type AS
/// WRITTEN (no strip — a `&T` param looks up `&T`'s entry, which the `& _`
/// rank-1 handler resolved by sharing `T`'s function), then emit its wire
/// params, prelude decode statements, and the call argument. Returns `None` for
/// a non-`Typed`/non-`Ident` arg (`self`, patterns), which the caller skips.
///
/// Reads only `ext`/`registry`/`original_ident`/`on_err` — independent of any
/// other input — so the per-input handling stays a self-contained unit.
#[allow(clippy::type_complexity)]
fn emit_input_param(
    ext: &JniGen,
    registry: &Registry<KotlinMeta>,
    original_ident: &syn::Ident,
    input: &syn::FnArg,
    on_err: &TokenStream,
) -> Option<(Vec<TokenStream>, Vec<TokenStream>, TokenStream)> {
    let syn::FnArg::Typed(pt) = input else {
        return None;
    };
    let syn::Pat::Ident(pat_id) = &*pt.pat else {
        return None;
    };
    let arg_ident = &pat_id.ident;
    let arg_ty = &*pt.ty;

    let mut wire_params: Vec<TokenStream> = Vec::new();
    let mut prelude: Vec<TokenStream> = Vec::new();

    // Constructor-expansion: this parameter's wire form is the fold plan's
    // flattened leaves. Decode each leaf with its own converter, run the
    // (pure-Rust) fold to build the value, then pass it to the call.
    if let Some(plan) = registry
        .expansion_plans
        .get(&(original_ident.clone(), arg_ident.clone()))
    {
        let (wp, pre, call_arg) = emit_expanded_param(ext, registry, plan, arg_ident, on_err);
        wire_params.extend(wp);
        prelude.extend(pre);
        return Some((wire_params, prelude, call_arg));
    }

    let entry = registry.input_entry(arg_ty).unwrap_or_else(|| {
        panic!(
            "JniGen::on_function: input type `{}` for `{}` is unresolved",
            TypeKey::from_type(arg_ty),
            original_ident,
        )
    });

    // Flattenable data_class param: cross its fields as separate wire
    // params and reconstruct the struct inline — no per-call
    // `env.get_field(...)` reflection. Falls back (None) to the
    // single-`JObject` path for any shape outside the conservative leaf
    // set (handles, nested structs, enums, …). The `JNINative` extern and
    // the Kotlin call-site destructure read the same plan so the three
    // sites can't drift.
    if let Some(plan) = build_flat_input_plan(ext, registry, arg_ident, arg_ty, "") {
        for leaf in &plan.leaves {
            let pid = &leaf.native_ident;
            let pty = &leaf.native_wire_ty;
            wire_params.push(quote!(#pid: #pty));
        }
        let (decode, call_arg) = render_flat_input_decode(&plan, arg_ident, on_err);
        prelude.push(decode);
        return Some((wire_params, prelude, call_arg));
    }

    // Bare `Option<primitive>` / `Option<enum>` param: cross as a
    // `(present: jboolean, value: <wire>)` pair instead of a boxed
    // `java.lang.*` `JObject`. The Rust side rebuilds the `Option` from two
    // raw scalars — no `env.call_method("intValue", …)` unbox. The `JNINative`
    // extern decl and the Kotlin call site read the same plan (see
    // `ParamMode::OptionScalar`), so the three sites can't drift.
    if let Some(sp) = build_option_scalar_input_plan(ext, registry, arg_ident, arg_ty) {
        let pid = &sp.present_ident;
        let vid = &sp.value_ident;
        let vwire = &sp.value_wire;
        wire_params.push(quote!(#pid: jni::sys::jboolean));
        wire_params.push(quote!(#vid: #vwire));
        let conv = &sp.inner_conv;
        let tmp = format_ident!("__{}_val", arg_ident);
        prelude.push(quote! {
            let #arg_ident = if #pid != 0u8 {
                let #tmp = match #conv(&mut env, &#vid) {
                    ::core::result::Result::Ok(__v) => __v,
                    ::core::result::Result::Err(__e) => {
                        let __zd = __ze_defaults(&mut env);
                        signal_error(&mut env, &__error_sink, &__SINK_MID, __SINK_FQN, __SINK_DESCR, ::core::option::Option::Some(&__e.to_string()), &__zd);
                        return #on_err;
                    }
                };
                ::core::option::Option::Some(#tmp)
            } else {
                ::core::option::Option::None
            };
        });
        return Some((wire_params, prelude, quote!(#arg_ident)));
    }

    // Slice / `Vec` of a flattenable data_class: the param crosses as a single
    // `jlong` handle to a Rust-side `Vec<T>` that the Kotlin wrapper builds by
    // pushing each element's decoupled leaves in a loop (see
    // `build_vec_build_helper_items` + `ParamMode::VecBuild`) — no per-element
    // `env.get_field(...)`. `&[T]` borrows the boxed Vec; by-value `Vec<T>`
    // moves it out with `mem::take` (leaving an empty Vec the Kotlin `finally`
    // frees). Decode is infallible, like the by-value-handle consume below.
    if let Some((elem, by_ref)) = vec_build_elem(ext, registry, arg_ty) {
        let handle_ident = format_ident!("{}_handle", arg_ident);
        wire_params.push(quote!(#handle_ident: jni::sys::jlong));
        if by_ref {
            prelude.push(quote!(
                let #arg_ident: &[#elem] =
                    unsafe { &*(#handle_ident as *const Vec<#elem>) };
            ));
        } else {
            prelude.push(quote!(
                let #arg_ident: Vec<#elem> =
                    unsafe { ::core::mem::take(&mut *(#handle_ident as *mut Vec<#elem>)) };
            ));
        }
        return Some((wire_params, prelude, quote!(#arg_ident)));
    }

    let wire = &entry.destination;
    let conv = entry.converter_ident().clone();
    let wire_ident = if matches!(wire, syn::Type::Ptr(_)) {
        format_ident!("{}_ptr", arg_ident)
    } else {
        arg_ident.clone()
    };

    // By-value `T` opaque-handle parameter: emit the consume
    // converter inline, bypassing `OwnedObject`. The Java side
    // takes the pointer out of its `NativeHandle.consume` under
    // the write lock and passes it here; `Box::from_raw`
    // reconstructs the unique owner and `*box` moves `T` out,
    // dropping the heap allocation. The unique-ownership
    // invariant is upheld by `NativeHandle.consume` (write-lock
    // + atomic pointer take), which drains all in-flight borrows
    // and ensures no live borrow can outlive this point. No
    // `T: Clone` bound, so non-Clone handles (e.g. `Publisher<'a>`)
    // work too. This decode is infallible — no `match` needed.
    let is_consume =
        !matches!(arg_ty, syn::Type::Reference(_)) && entry.metadata.is_direct_handle();
    if is_consume {
        wire_params.push(quote!(#wire_ident: jni::sys::jlong));
        prelude.push(quote!(
            let #arg_ident: #arg_ty = unsafe {
                *std::boxed::Box::from_raw(#wire_ident as *mut #arg_ty)
            };
        ));
        return Some((wire_params, prelude, quote!(#arg_ident)));
    }

    let wire_with_lifetime = annotate_jobject_with_lifetime(wire, "a");
    wire_params.push(quote!(#wire_ident: #wire_with_lifetime));
    // Input wrapper takes wires by ref except for raw pointers. The
    // converter returns `Result<T, __JniErr>`; on `Err` we signal the
    // error sink and bail with the function sentinel (no JVM throw).
    let decode_call = if matches!(wire, syn::Type::Ptr(_)) {
        quote!(#conv(&mut env, #wire_ident))
    } else {
        quote!(#conv(&mut env, &#wire_ident))
    };
    // Binding for the final `arg_ident` needs `mut` when the source
    // fn takes `&mut T` — the call site below emits `&mut arg_ident`,
    // which requires a mutable binding. Also for `Option<&mut T>`
    // where the call site needs `.as_deref_mut()`. Intermediate stage
    // bindings (`__{ident}_sN`) don't need it.
    let arg_mut: TokenStream = if matches!(arg_ty, syn::Type::Reference(r) if r.mutability.is_some())
        || matches!(option_inner_ref_mutability(arg_ty), Some(true))
    {
        quote!(mut)
    } else {
        quote!()
    };
    // Stage 0: wire-facing function. Pre_stages then run in REVERSE
    // (rust-side last). Even with no pre_stages this collapses to a
    // single `let #arg_ident = match decode_call { ... }`, byte-
    // identical to the pre-chain emission.
    if entry.pre_stages.is_empty() {
        prelude.push(quote!(
            let #arg_mut #arg_ident = match #decode_call {
                ::core::result::Result::Ok(__v) => __v,
                ::core::result::Result::Err(__e) => {
                    let __zd = __ze_defaults(&mut env); signal_error(&mut env, &__error_sink, &__SINK_MID, __SINK_FQN, __SINK_DESCR, ::core::option::Option::Some(&__e.to_string()), &__zd);
                    return #on_err;
                }
            };
        ));
    } else {
        // Multi-stage: introduce a temporary for the function's
        // result, then thread each pre_stage in reverse onto it.
        let stage0_ident = format_ident!("__{}_s0", arg_ident);
        prelude.push(quote!(
            let #stage0_ident = match #decode_call {
                ::core::result::Result::Ok(__v) => __v,
                ::core::result::Result::Err(__e) => {
                    let __zd = __ze_defaults(&mut env); signal_error(&mut env, &__error_sink, &__SINK_MID, __SINK_FQN, __SINK_DESCR, ::core::option::Option::Some(&__e.to_string()), &__zd);
                    return #on_err;
                }
            };
        ));
        let mut prev = stage0_ident;
        // pre_stages[0] is closest to rust → iterated last; walk
        // back from the function-adjacent end.
        let n = entry.pre_stages.len();
        for (idx, stage) in entry.input_stage_order() {
            let stage_fn = &stage.function.sig.ident;
            let is_last = idx == 0;
            let out_ident = if is_last {
                arg_ident.clone()
            } else {
                format_ident!("__{}_s{}", arg_ident, n - idx)
            };
            // Final binding gets `mut` if the source fn takes `&mut`.
            let bind_mut: TokenStream = if is_last { arg_mut.clone() } else { quote!() };
            prelude.push(quote!(
                let #bind_mut #out_ident = match #stage_fn(&mut env, #prev) {
                    ::core::result::Result::Ok(__v) => __v,
                    ::core::result::Result::Err(__e) => {
                        let __zd = __ze_defaults(&mut env); signal_error(&mut env, &__error_sink, &__SINK_MID, __SINK_FQN, __SINK_DESCR, ::core::option::Option::Some(&__e.to_string()), &__zd);
                        return #on_err;
                    }
                };
            ));
            prev = out_ident;
        }
    }
    let call_arg = match arg_ty {
        syn::Type::Reference(r) if r.mutability.is_some() => quote!(&mut #arg_ident),
        syn::Type::Reference(_) => quote!(&#arg_ident),
        // `Option<&T>` / `Option<&mut T>` for opaque inner: the input
        // converter produced `Option<OwnedObject<T>>` (see rank-1
        // handler above). `.as_deref()` / `.as_deref_mut()` coerces
        // back to `Option<&T>` / `Option<&mut T>` via OwnedObject's
        // Deref / DerefMut impls.
        _ if matches!(option_inner_ref_mutability(arg_ty), Some(false)) => {
            quote!(#arg_ident.as_deref())
        }
        _ if matches!(option_inner_ref_mutability(arg_ty), Some(true)) => {
            quote!(#arg_ident.as_deref_mut())
        }
        _ => quote!(#arg_ident),
    };
    Some((wire_params, prelude, call_arg))
}

/// Emit the wire params, decode prelude, and call argument for one
/// constructor-expanded parameter. Each leaf is decoded with its own resolved
/// input converter (reusing the by-value-handle consume fast path where the
/// leaf is a direct owned handle); the leaves then feed
/// [`crate::api::core::expand::emit_fold`], whose `Result<_, String>` is routed
/// through the same error sink as any fallible input. The returned call
/// argument is the built value (`&value` when the original parameter was `&T`).
pub(crate) fn emit_expanded_param(
    ext: &JniGen,
    registry: &Registry<KotlinMeta>,
    plan: &crate::api::core::expand::FoldPlan,
    orig_param: &syn::Ident,
    on_err: &TokenStream,
) -> (Vec<TokenStream>, Vec<TokenStream>, TokenStream) {
    let mut wire_params: Vec<TokenStream> = Vec::new();
    let mut prelude: Vec<TokenStream> = Vec::new();
    let mut leaf_locals: Vec<syn::Ident> = Vec::new();

    for leaf in &plan.leaves {
        let leaf_ty = &leaf.ty;
        let entry = registry.input_entry(leaf_ty).unwrap_or_else(|| {
            panic!(
                "JniGen expand: leaf type `{}` (parameter `{}`) is unresolved",
                TypeKey::from_type(leaf_ty),
                orig_param,
            )
        });
        let local = format_ident!("__exp_{}", leaf.name);

        // `Option<scalar>` / `Option<enum>` leaf (only produced by a
        // selector-dispatched constructor variant, where each arm's args are
        // `Option`-wrapped by presence): cross as a decoupled
        // `(present: jboolean, value: <wire>)` pair instead of a boxed
        // `java.lang.*` `JObject`. This matches the Kotlin extern — which
        // applies the same `build_option_scalar_input_plan` per expanded leaf in
        // `render_extern_decl` — and the bare top-level `Option<scalar>` param
        // path, so the JNI arity/types agree on both sides of the wire.
        if let Some(sp) = build_option_scalar_input_plan(ext, registry, &leaf.name, leaf_ty) {
            let present_ident = &sp.present_ident;
            let value_ident = &sp.value_ident;
            let value_wire = &sp.value_wire;
            let inner_conv = &sp.inner_conv;
            wire_params.push(quote!(#present_ident: jni::sys::jboolean));
            wire_params.push(quote!(#value_ident: #value_wire));
            prelude.push(quote!(
                let #local: #leaf_ty = if #present_ident != 0u8 {
                    let __v = match #inner_conv(&mut env, &#value_ident) {
                        ::core::result::Result::Ok(__v) => __v,
                        ::core::result::Result::Err(__e) => {
                            let __zd = __ze_defaults(&mut env);
                            signal_error(&mut env, &__error_sink, &__SINK_MID, __SINK_FQN, __SINK_DESCR, ::core::option::Option::Some(&__e.to_string()), &__zd);
                            return #on_err;
                        }
                    };
                    ::core::option::Option::Some(__v)
                } else {
                    ::core::option::Option::None
                };
            ));
            leaf_locals.push(local);
            continue;
        }

        // Direct owned-handle leaf (e.g. an identity-variant `T`): consume the
        // jlong handle inline, mirroring the normal by-value-handle path.
        let is_consume =
            !matches!(leaf_ty, syn::Type::Reference(_)) && entry.metadata.is_direct_handle();
        if is_consume {
            let wire_ident = format_ident!("{}_ptr", leaf.name);
            wire_params.push(quote!(#wire_ident: jni::sys::jlong));
            prelude.push(quote!(
                let #local: #leaf_ty = unsafe {
                    *std::boxed::Box::from_raw(#wire_ident as *mut #leaf_ty)
                };
            ));
            leaf_locals.push(local);
            continue;
        }

        let wire = &entry.destination;
        let conv = entry.function.sig.ident.clone();
        let wire_ident = if matches!(wire, syn::Type::Ptr(_)) {
            format_ident!("{}_ptr", leaf.name)
        } else {
            leaf.name.clone()
        };
        let wire_with_lifetime = annotate_jobject_with_lifetime(wire, "a");
        wire_params.push(quote!(#wire_ident: #wire_with_lifetime));
        let decode_call = if matches!(wire, syn::Type::Ptr(_)) {
            quote!(#conv(&mut env, #wire_ident))
        } else {
            quote!(#conv(&mut env, &#wire_ident))
        };
        // Compose any pre_stages (rust-side, reverse order) onto the decode.
        if entry.pre_stages.is_empty() {
            prelude.push(quote!(
                let #local = match #decode_call {
                    ::core::result::Result::Ok(__v) => __v,
                    ::core::result::Result::Err(__e) => {
                        let __zd = __ze_defaults(&mut env); signal_error(&mut env, &__error_sink, &__SINK_MID, __SINK_FQN, __SINK_DESCR, ::core::option::Option::Some(&__e.to_string()), &__zd);
                        return #on_err;
                    }
                };
            ));
        } else {
            let stage0 = format_ident!("{}_s0", local);
            prelude.push(quote!(
                let #stage0 = match #decode_call {
                    ::core::result::Result::Ok(__v) => __v,
                    ::core::result::Result::Err(__e) => {
                        let __zd = __ze_defaults(&mut env); signal_error(&mut env, &__error_sink, &__SINK_MID, __SINK_FQN, __SINK_DESCR, ::core::option::Option::Some(&__e.to_string()), &__zd);
                        return #on_err;
                    }
                };
            ));
            let n = entry.pre_stages.len();
            let mut prev = stage0;
            for (idx, stage) in entry.pre_stages.iter().enumerate().rev() {
                let stage_fn = &stage.function.sig.ident;
                let out_ident = if idx == 0 {
                    local.clone()
                } else {
                    format_ident!("{}_s{}", local, n - idx)
                };
                prelude.push(quote!(
                    let #out_ident = match #stage_fn(&mut env, #prev) {
                        ::core::result::Result::Ok(__v) => __v,
                        ::core::result::Result::Err(__e) => {
                            let __zd = __ze_defaults(&mut env); signal_error(&mut env, &__error_sink, &__SINK_MID, __SINK_FQN, __SINK_DESCR, ::core::option::Option::Some(&__e.to_string()), &__zd);
                            return #on_err;
                        }
                    };
                ));
                prev = out_ident;
            }
        }
        leaf_locals.push(local);
    }

    // The fold itself (language-agnostic). Its `Err(String)` is lifted into
    // `__JniErr` and routed through the same sink as fallible inputs.
    let qualify = |id: &syn::Ident| -> syn::Path {
        let m = ext.fn_module(registry, id);
        syn::parse_quote!(#m::#id)
    };
    let fold_expr = crate::api::core::expand::emit_fold(plan, &leaf_locals, &qualify);
    let folded = format_ident!("__folded_{}", orig_param);
    prelude.push(quote!(
        let #folded = match #fold_expr {
            ::core::result::Result::Ok(__v) => __v,
            ::core::result::Result::Err(__e) => {
                let __je = <__JniErr as ::core::convert::From<::std::string::String>>::from(__e);
                let __zd = __ze_defaults(&mut env); signal_error(&mut env, &__error_sink, &__SINK_MID, __SINK_FQN, __SINK_DESCR, ::core::option::Option::Some(&__je.to_string()), &__zd);
                return #on_err;
            }
        };
    ));

    // `Option<&T>` ⇒ `folded.as_ref()`; `&T` ⇒ `&folded`; by-value (incl.
    // `Option<T>`) ⇒ `folded`.
    let call_arg = match (plan.produces_option(), plan.by_ref) {
        (true, true) => quote!(#folded.as_ref()),
        (false, true) => quote!(&#folded),
        (_, false) => quote!(#folded),
    };
    (wire_params, prelude, call_arg)
}
