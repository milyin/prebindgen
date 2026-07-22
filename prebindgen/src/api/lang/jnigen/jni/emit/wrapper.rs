//! Extern `"C"` JNI wrapper functions: signature lowering, input
//! params, and the expanded-param path.

use super::*;
use crate::api::core::types_util::result_ok_type;

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

/// A const whose (peeled) type is a declared opaque handle is rejected: a
/// shared closeable `val` is semantically wrong (whose `close()` is it?).
/// Expose a factory function instead — the established idiom (e.g. zenoh's
/// `encoding_const_*` companion factories).
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

    let mut wire_params: Vec<TokenStream> = Vec::new();
    // Each entry is a per-input decode statement. Fallible decodes are
    // `match`-arms that, on `Err`, call `signal_error(&mut env,
    // &__error_sink, &__e)` (invoking the caller's Kotlin sink instead of
    // throwing a JVM exception) and `return <sentinel>;`.
    let mut prelude: Vec<TokenStream> = Vec::new();
    let mut call_args: Vec<TokenStream> = Vec::new();

    // The lowered plan classifies both sides ONCE — the same classification
    // the Kotlin wrapper and `external fun` renderers consume; this site
    // renders the Rust decode/encode for each kind. The output is classified
    // first (inside `build`) so the per-input `match`-arms can splice the
    // function's sentinel into their early-`return` path.
    // Backstop only — `validate_resolved` reports every plan failure before
    // any writer runs, so this panic is unreachable through the write paths.
    let plan = ext
        .fn_plan(registry, f)
        .unwrap_or_else(|e| panic!("{}", e.message(original_ident)));
    let wrapper_ident = syn::Ident::new(&plan.native_symbol, Span::call_site());
    // Output (data) expansion: when output expansion was declared for this
    // function, the return value is decomposed by the deconstructor. Two
    // deliveries:
    //   * `Callback` (`deconstruct_output`, `FnOutputPlan::Unfold`): the
    //     leaves are delivered to a foreign builder/fold lambda — the
    //     wrapper's wire return is the lambda's `JObject` result (no
    //     `output_entry`; see `emit_unfold_delivery`).
    //   * `Return` (`convert_output`, `is_convert`): the single decomposed
    //     value is **returned** directly through its ordinary output
    //     converter — the wrapper behaves exactly like a normal function
    //     whose return type is `convert_out_ty`.
    let unfold_plan = registry.unfold_plans.get(original_ident);
    // Error-position expansion: when the fn returns `Result<T, E>` and an error
    // plan is declared, the **`?`** is applied here — the extern peels the
    // `Result` (Err arm decomposes `E` into the `ze` leaves and invokes the
    // typed DOMAIN handler), and the success path uses `T`'s converter (not the
    // `Result<T, E>` rank-2 wrapper).
    let error_plan = registry.error_plans.get(original_ident);
    let is_convert = matches!(&plan.output, FnOutputPlan::Value(v) if v.is_convert);
    // The output converter entry (`None` for callback delivery). The lookup
    // was validated at plan build; re-resolving here keeps the plan free of
    // registry borrows for the future build-once stage.
    let output_entry = match &plan.output {
        FnOutputPlan::Value(v) => Some(
            registry
                .output_entry(&v.target_ty)
                .expect("output entry validated at plan build"),
        ),
        FnOutputPlan::Unfold(_) => None,
    };
    let wire_ty = plan.output.wire_ty();
    let wire_return = annotate_jobject_with_lifetime(&wire_ty, "a").to_token_stream();
    let on_err = sentinel_for_wire(&wire_ty);

    for param in &plan.params {
        let (wp, pre, call_arg) = emit_input_param(ext, registry, original_ident, param, &on_err);
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
        let uplan = unfold_plan.expect("is_convert ⇒ plan");
        let leaf = &uplan.leaves[0];
        let by_ref = uplan.by_ref;
        let compose = |base: TokenStream, base_is_ref: bool| -> TokenStream {
            let mut e = if base_is_ref { base } else { quote!(&#base) };
            for a in &leaf.path {
                let m = ext.fn_module(registry, a);
                e = quote!(#m::#a(#e));
            }
            e
        };
        match &uplan.shape {
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
        // wraps, Option-nested accessor unwrap) — and invoke the typed DOMAIN
        // handler (no `je`, no defaults), then return the sentinel. A failure
        // while ENCODING the error itself degrades to the BINDING channel
        // (`signal_binding_error`). The success `T` flows into the normal
        // output phase.
        let eze_idents: Vec<syn::Ident> = (0..ep.leaves.len())
            .map(|i| format_ident!("__eze{}", i))
            .collect();
        let ze_fail = |msg: TokenStream| -> TokenStream {
            quote! {
                signal_binding_error(&mut env, &__error_sink, &__SINK_MID, __SINK_FQN, __SINK_DESCR, &#msg);
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
                    signal_domain_error(
                        &mut env, &__domain_sink,
                        &__DSINK_MID, __DSINK_FQN, __DSINK_DESCR,
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
    let output_phase: TokenStream = if let FnOutputPlan::Unfold(u) = &plan.output {
        // Iterable folds: two params (`__acc` accumulator + `__fold` callback).
        // Decompose/Optional: a single `__builder` callback.
        let uplan = unfold_plan.expect("Unfold output ⇒ unfold plan present");
        builder_param = Some(unfold_builder_param(u.iterable_fold));
        emit_unfold_delivery(
            ext,
            registry,
            uplan,
            u.iface.as_deref(),
            &call_expr,
            &on_err,
        )
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
                        signal_binding_error(&mut env, &__error_sink, &__SINK_MID, __SINK_FQN, __SINK_DESCR, &__e.to_string());
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
                    signal_binding_error(&mut env, &__error_sink, &__SINK_MID, __SINK_FQN, __SINK_DESCR, &__e.to_string());
                    #on_err
                }
            }
        });
        phase
    };

    // Error sinks. Both channels are typed `fun interface`s whose `run` method
    // ID is resolved once per process on the interface class (the sink instance
    // differs per call). The BINDING channel (`__error_sink` + `__SINK_*`, the
    // base `JniErrorHandler`) is always present — every wrapper can hit a
    // binding/marshalling failure. The DOMAIN channel (`__domain_sink` +
    // `__DSINK_*`, the typed `<Src>Handler`) is present only for a fallible fn
    // with a declared error plan; its `Err(E)` decomposition delivers the real
    // leaves (no `je`, no fabricated defaults).
    let error_ifaces = plan.onerror_iface.as_ref().unwrap_or_else(|| {
        panic!(
            "jnigen: cannot derive the onError handler interface for `{}`",
            original_ident
        )
    });
    let bsink_fqn_lit = syn::LitStr::new(&error_ifaces.binding.raw_slash_fqn(), Span::call_site());
    let bsink_descr_lit = syn::LitStr::new(&error_ifaces.binding.descr, Span::call_site());
    let (domain_setup, domain_sink_param) = match &error_ifaces.domain {
        Some(dsink) => {
            let dfqn = syn::LitStr::new(&dsink.raw_slash_fqn(), Span::call_site());
            let ddescr = syn::LitStr::new(&dsink.descr, Span::call_site());
            (
                quote! {
                    #[allow(non_upper_case_globals)]
                    static __DSINK_MID: ::prebindgen::lang::CachedIfaceMethod =
                        ::prebindgen::lang::CachedIfaceMethod::new();
                    const __DSINK_FQN: &str = #dfqn;
                    const __DSINK_DESCR: &str = #ddescr;
                },
                quote!(__domain_sink: jni::objects::JObject<'a>,),
            )
        }
        None => (quote!(), quote!()),
    };
    let sinks_setup = quote! {
        #[allow(non_upper_case_globals)]
        static __SINK_MID: ::prebindgen::lang::CachedIfaceMethod =
            ::prebindgen::lang::CachedIfaceMethod::new();
        const __SINK_FQN: &str = #bsink_fqn_lit;
        const __SINK_DESCR: &str = #bsink_descr_lit;
        #domain_setup
    };

    // Trailing sink params: `__error_sink` (binding) always, then
    // `__domain_sink` (typed domain error) for a fallible fn — a capture is
    // passed for each. Declared after the wire params + builder so the order
    // matches the Kotlin `external fun`.
    quote! {
        #[no_mangle]
        #[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
        pub unsafe extern "C" fn #wrapper_ident<'a>(
            mut env: jni::JNIEnv<'a>,
            _class: jni::objects::JClass<'a>,
            #(#wire_params,)*
            #builder_param
            __error_sink: jni::objects::JObject<'a>,
            #domain_sink_param
        ) -> #wire_return {
            #sinks_setup
            #(#prelude)*
            #output_phase
        }
    }
}

fn unfold_builder_param(iterable_fold: bool) -> TokenStream {
    // An `Iterable` fold (incl. `Option<Vec<T>>`) takes `(acc, fold)`; every
    // other delivery takes a single `build`.
    if iterable_fold {
        quote!(__acc: jni::objects::JObject<'a>, __fold: jni::objects::JObject<'a>,)
    } else {
        quote!(__builder: jni::objects::JObject<'a>,)
    }
}

/// Render the Rust-side decode for one source-fn parameter from its lowered
/// [`PlanParam`]: the wire params, prelude decode statements, and the call
/// argument. The classification (which crossing form) lives in the plan; this
/// site only renders each [`InputKind`]'s decode.
#[allow(clippy::type_complexity)]
fn emit_input_param(
    ext: &JniGen,
    registry: &Registry<KotlinMeta>,
    original_ident: &syn::Ident,
    param: &PlanParam,
    on_err: &TokenStream,
) -> (Vec<TokenStream>, Vec<TokenStream>, TokenStream) {
    // Constructor-expansion: this parameter's wire form is the fold plan's
    // flattened leaves. Decode each leaf with its own converter, run the
    // (pure-Rust) fold to build the value, then pass it to the call.
    let leaf = match &param.form {
        ParamForm::Expanded(leaves) => {
            let fold = registry
                .expansion_plans
                .get(&(original_ident.clone(), param.ident.clone()))
                .expect("ParamForm::Expanded ⇒ expansion plan present");
            return emit_expanded_param(ext, registry, fold, leaves, &param.ident, on_err);
        }
        ParamForm::Single(leaf) => &**leaf,
    };
    let arg_ident = &param.ident;
    let arg_ty = &param.ty;

    let mut wire_params: Vec<TokenStream> = Vec::new();
    let mut prelude: Vec<TokenStream> = Vec::new();

    match &leaf.kind {
        // Flattenable data_class param: cross its fields as separate wire
        // params and reconstruct the struct inline — no per-call
        // `env.get_field(...)` reflection. The `JNINative` extern and the
        // Kotlin call-site destructure read the same plan so the three
        // sites can't drift.
        InputKind::FlattenStruct(plan) => {
            for leaf in &plan.leaves {
                let pid = &leaf.native_ident;
                let pty = &leaf.native_wire_ty;
                wire_params.push(quote!(#pid: #pty));
            }
            let (decode, call_arg) = render_flat_input_decode(plan, arg_ident, on_err);
            prelude.push(decode);
            (wire_params, prelude, call_arg)
        }

        // Bare `Option<primitive>` / `Option<enum>` param: cross as a
        // `(present: jboolean, value: <wire>)` pair instead of a boxed
        // `java.lang.*` `JObject`. The Rust side rebuilds the `Option` from
        // two raw scalars — no `env.call_method("intValue", …)` unbox.
        InputKind::OptionScalar(sp) => {
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
                            signal_binding_error(&mut env, &__error_sink, &__SINK_MID, __SINK_FQN, __SINK_DESCR, &__e.to_string());
                            return #on_err;
                        }
                    };
                    ::core::option::Option::Some(#tmp)
                } else {
                    ::core::option::Option::None
                };
            });
            (wire_params, prelude, quote!(#arg_ident))
        }

        // Slice / `Vec` of a flattenable data_class: the param crosses as a
        // single `jlong` handle to a Rust-side `Vec<T>` that the Kotlin
        // wrapper builds by pushing each element's decoupled leaves in a loop
        // (see `build_vec_build_helper_items` + `ParamMode::VecBuild`) — no
        // per-element `env.get_field(...)`. `&[T]` borrows the boxed Vec;
        // by-value `Vec<T>` moves it out with `mem::take` (leaving an empty
        // Vec the Kotlin `finally` frees). Decode is infallible, like the
        // by-value-handle consume below.
        InputKind::VecBuild { elem, by_ref } => {
            let handle_ident = format_ident!("{}_handle", arg_ident);
            wire_params.push(quote!(#handle_ident: jni::sys::jlong));
            if *by_ref {
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
            (wire_params, prelude, quote!(#arg_ident))
        }

        // By-value `T` opaque-handle parameter: emit the consume
        // converter inline, bypassing `OwnedObject`. The Java side
        // holds the handle's monitor and passes the pointer here;
        // `Box::from_raw` reconstructs the unique owner and `*box`
        // moves `T` out, dropping the heap allocation. The
        // unique-ownership invariant is upheld by the Kotlin wrapper
        // (monitor + tag-bit close in `finally`), which ensures the
        // same live pointer cannot be passed twice. No `T: Clone`
        // bound, so non-Clone handles (e.g. `Publisher<'a>`) work too.
        // A null or tagged (closed) pointer — a close that raced past
        // the pre-lock guard — is rejected before any dereference.
        InputKind::Handle { direct: true } if !matches!(arg_ty, syn::Type::Reference(_)) => {
            let entry = registry
                .input_entry(arg_ty)
                .expect("plan classified Handle ⇒ entry present");
            let wire_ident = if matches!(&entry.destination, syn::Type::Ptr(_)) {
                format_ident!("{}_ptr", arg_ident)
            } else {
                arg_ident.clone()
            };
            wire_params.push(quote!(#wire_ident: jni::sys::jlong));
            prelude.push(quote!(
                if #wire_ident == 0 || (#wire_ident & 1) == 1 {
                    signal_binding_error(&mut env, &__error_sink, &__SINK_MID, __SINK_FQN, __SINK_DESCR, "Operation on a closed native handle.");
                    return #on_err;
                }
                let #arg_ident: #arg_ty = unsafe {
                    *std::boxed::Box::from_raw(#wire_ident as *mut #arg_ty)
                };
            ));
            (wire_params, prelude, quote!(#arg_ident))
        }

        // Everything else — borrowed/composed handles, value projections,
        // callbacks, plain types — decodes through the resolved entry's
        // ordinary converter chain.
        InputKind::Callback { .. }
        | InputKind::Handle { .. }
        | InputKind::ValueUnwrap { .. }
        | InputKind::Unsigned64
        | InputKind::Plain => {
            let entry = registry.input_entry(arg_ty).unwrap_or_else(|| {
                panic!(
                    "JniGen::on_function: input type `{}` for `{}` is unresolved",
                    TypeKey::from_type(arg_ty),
                    original_ident,
                )
            });
            emit_plain_decode(entry, arg_ident, arg_ty, on_err)
        }
    }
}

/// The ordinary converter-chain decode shared by every pass-through kind:
/// wire param + staged decode prelude + the call argument (`&decoded` /
/// `.as_deref()` per the source param's Rust shape).
fn emit_plain_decode(
    entry: &crate::api::core::registry::TypeEntry<KotlinMeta>,
    arg_ident: &syn::Ident,
    arg_ty: &syn::Type,
    on_err: &TokenStream,
) -> (Vec<TokenStream>, Vec<TokenStream>, TokenStream) {
    let mut wire_params: Vec<TokenStream> = Vec::new();
    let mut prelude: Vec<TokenStream> = Vec::new();
    let wire = &entry.destination;
    let conv = entry.converter_ident().clone();
    let wire_ident = if matches!(wire, syn::Type::Ptr(_)) {
        format_ident!("{}_ptr", arg_ident)
    } else {
        arg_ident.clone()
    };

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
                    signal_binding_error(&mut env, &__error_sink, &__SINK_MID, __SINK_FQN, __SINK_DESCR, &__e.to_string());
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
                    signal_binding_error(&mut env, &__error_sink, &__SINK_MID, __SINK_FQN, __SINK_DESCR, &__e.to_string());
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
                        signal_binding_error(&mut env, &__error_sink, &__SINK_MID, __SINK_FQN, __SINK_DESCR, &__e.to_string());
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
    (wire_params, prelude, call_arg)
}

/// Emit the wire params, decode prelude, and call argument for one
/// constructor-expanded parameter. Each classified leaf is decoded with its
/// own resolved input converter (reusing the by-value-handle consume fast
/// path where the leaf is a direct owned handle); the leaves then feed
/// [`crate::api::core::expand::emit_fold`], whose `Result<_, String>` is routed
/// through the same error sink as any fallible input. The returned call
/// argument is the built value (`&value` when the original parameter was `&T`).
pub(crate) fn emit_expanded_param(
    ext: &JniGen,
    registry: &Registry<KotlinMeta>,
    plan: &crate::api::core::expand::FoldPlan,
    leaves: &[PlanLeaf],
    orig_param: &syn::Ident,
    on_err: &TokenStream,
) -> (Vec<TokenStream>, Vec<TokenStream>, TokenStream) {
    let mut wire_params: Vec<TokenStream> = Vec::new();
    let mut prelude: Vec<TokenStream> = Vec::new();
    let mut leaf_locals: Vec<syn::Ident> = Vec::new();

    debug_assert_eq!(plan.leaves.len(), leaves.len());
    for (leaf, classified) in plan.leaves.iter().zip(leaves) {
        let leaf_ty = &leaf.ty;
        let lookup_entry = || {
            registry.input_entry(leaf_ty).unwrap_or_else(|| {
                panic!(
                    "JniGen expand: leaf type `{}` (parameter `{}`) is unresolved",
                    TypeKey::from_type(leaf_ty),
                    orig_param,
                )
            })
        };
        let local = format_ident!("__exp_{}", leaf.name);

        // `Option<scalar>` / `Option<enum>` leaf (only produced by a
        // selector-dispatched constructor variant, where each arm's args are
        // `Option`-wrapped by presence): cross as a decoupled
        // `(present: jboolean, value: <wire>)` pair instead of a boxed
        // `java.lang.*` `JObject`. The Kotlin extern and call site consume
        // the same classified plan, so the JNI arity/types agree on both
        // sides of the wire.
        if let InputKind::OptionScalar(sp) = &classified.kind {
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
                            signal_binding_error(&mut env, &__error_sink, &__SINK_MID, __SINK_FQN, __SINK_DESCR, &__e.to_string());
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
        // jlong handle inline, mirroring the normal by-value-handle path —
        // including its null/tagged (closed) pointer guard.
        let is_consume = matches!(classified.kind, InputKind::Handle { direct: true })
            && !matches!(leaf_ty, syn::Type::Reference(_));
        if is_consume {
            let wire_ident = format_ident!("{}_ptr", leaf.name);
            wire_params.push(quote!(#wire_ident: jni::sys::jlong));
            prelude.push(quote!(
                if #wire_ident == 0 || (#wire_ident & 1) == 1 {
                    signal_binding_error(&mut env, &__error_sink, &__SINK_MID, __SINK_FQN, __SINK_DESCR, "Operation on a closed native handle.");
                    return #on_err;
                }
                let #local: #leaf_ty = unsafe {
                    *std::boxed::Box::from_raw(#wire_ident as *mut #leaf_ty)
                };
            ));
            leaf_locals.push(local);
            continue;
        }

        let entry = lookup_entry();
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
                        signal_binding_error(&mut env, &__error_sink, &__SINK_MID, __SINK_FQN, __SINK_DESCR, &__e.to_string());
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
                        signal_binding_error(&mut env, &__error_sink, &__SINK_MID, __SINK_FQN, __SINK_DESCR, &__e.to_string());
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
                            signal_binding_error(&mut env, &__error_sink, &__SINK_MID, __SINK_FQN, __SINK_DESCR, &__e.to_string());
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
                signal_binding_error(&mut env, &__error_sink, &__SINK_MID, __SINK_FQN, __SINK_DESCR, &__je.to_string());
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
