//! Extern `"C"` JNI wrapper functions: signature lowering, input
//! params, and the expanded-param path.

use super::*;
use crate::api::{
    core::{registry::Conversions, types_util::result_ok_type},
    lang::jnigen::jni::trait_impl::{
        build_through_erased_wrappers, build_through_wrappers, read_through_erased_wrappers,
    },
};

pub(crate) fn emit_jni_function_wrapper(
    ext: &Declarations,
    f: &crate::api::core::flat::Function,
    registry: &Registry<KotlinMeta>,
    emit: &crate::api::core::emit::Emit,
) -> TokenStream {
    emit_jni_function_wrapper_with_callee(ext, f, registry, None, emit)
}

/// The synthetic nullary getter signature a declared const is emitted
/// through: `pub fn const_get_<ident_lower>() -> <const ty>`. Both sides —
/// the Rust extern ([`Declarations::on_const`] via
/// [`emit_jni_function_wrapper_with_callee`]) and the Kotlin `val`
/// initializer (`render_const_val`) — derive the extern symbol from this one
/// ident, so they stay in sync by construction. The body is never used.
pub(crate) fn const_getter_fn(
    c: &crate::api::core::flat::Constant,
) -> crate::api::core::flat::Function {
    let ident = format_ident!("const_get_{}", c.name.to_string().to_lowercase());
    // No lookup: a constant element carries its own `TypeRef`.
    crate::api::core::flat::Function::synthetic_getter(ident, c.ty.clone())
}

/// A const whose (peeled) type is a declared opaque handle is rejected: a
/// shared closeable `val` is semantically wrong (whose `close()` is it?).
/// Expose a factory function instead — the established idiom (e.g. zenoh's
/// `encoding_const_*` companion factories).
pub(crate) fn reject_handle_const(ext: &Declarations, c: &crate::api::core::flat::Constant) {
    // Off the element: a constant's type is a reading, so the peel is the
    // model's (`util::head_type`) and no node is fetched to reach it.
    reject_handle_key(
        ext,
        &crate::api::lang::jnigen::util::head_type(&c.ty).key(),
        "const",
        &c.name.to_string(),
    );
}

/// The constant-value handle check shared by both constant kinds: peel
/// `&`/`Option`/`Vec` layers off `ty` and reject if what remains is a
/// declared opaque handle. `what`/`ident` shape the error message
/// (`const MAX_LEN` / `constant fn encoding_const_x_str`).
pub(crate) fn reject_handle_constant_type(
    ext: &Declarations,
    ty: &crate::api::core::flat::TypeRef,
    what: &str,
    name: &str,
) {
    // The same three layers, off the classification. The node loop peeled a
    // `Type::Reference` and then re-read the last path segment's ident twice
    // (`option_inner_type`, `vec_inner_type`) to decide what it had.
    let mut ty = ty;
    loop {
        if let Some(inner) = ty
            .borrow_target()
            .or_else(|| ty.optional_inner())
            .or_else(|| ty.sequence_elem())
        {
            ty = inner;
            continue;
        }
        break;
    }
    reject_handle_key(ext, &ty.key(), what, name);
}

/// The refusal itself, once: a declared opaque handle cannot be a shared
/// closeable Kotlin `val`.
///
/// Split from the peel because the two callers peel differently — an element's
/// type is a reading and peels off the kind, while a **build-script-supplied**
/// expression type has no reading until something interns it and peels off the
/// spelling. One assertion, two ways to reach it.
fn reject_handle_key(ext: &Declarations, key: &TypeKey, what: &str, name: &str) {
    let is_handle = ext.types.get(key).is_some_and(|cfg| cfg.is_opaque());
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
pub(crate) fn validate_constant_fn(ext: &Declarations, f: &crate::api::core::flat::Function) {
    // The ELEMENT: a signature is a parameter list and a return, both already
    // classified. This walked `sig.inputs` for the arity, matched
    // `ReturnType::Type` for the return — an elided one the element already
    // normalizes to `Unit` — and ran `result_ok_type` over a path to re-derive
    // the fallibility `TypeKind::Fallible` states.
    assert!(
        f.params.is_empty(),
        "constant fn `{}`: takes {} parameter(s) — a function-backed constant must be nullary \
         (declare it with `.fun(...)` instead if it is a real function)",
        f.name,
        f.params.len()
    );
    assert!(
        f.ret.fallible_parts().is_none(),
        "constant fn `{}`: returns a `Result` — a function-backed constant must be \
         infallible (declare it with `.fun(...)` instead if it can fail)",
        f.name
    );
    reject_handle_constant_type(ext, &f.ret, "constant fn", &f.name.to_string());
}

/// The synthetic nullary getter signature an **expression constant**
/// ([`ConstExprDecl`](crate::lang::ConstExprDecl)) is emitted through:
/// `pub fn const_get_<val_name_lower>() -> <ty>` — the same convention as
/// const-backed getters, so both sides derive the extern symbol from the one
/// val name. The body is never used.
pub(crate) fn const_expr_getter_fn(
    kotlin_name: &str,
    ty: &syn::Type,
    registry: &impl Conversions<KotlinMeta>,
) -> crate::api::core::flat::Function {
    let ident = format_ident!("const_get_{}", kotlin_name.to_lowercase());
    // The one lookup this path needs: the type is named by a build script, so
    // no element carries it. A miss means the declared type never entered the
    // pipeline, which is a binding error worth naming rather than a `None` to
    // absorb.
    let ret = registry.reading_of(ty).unwrap_or_else(|| {
        panic!(
            "constant_expr `{kotlin_name}`: type `{}` is not a type this binding crosses — \
             declare it, or name one that is",
            quote::ToTokens::to_token_stream(ty),
        )
    });
    crate::api::core::flat::Function::synthetic_getter(ident, ret)
}
/// Validates an expression constant's declared value type (checked on both
/// write paths): not a `Result` (a domain-fallible value is not a constant),
/// not (peeled to) a declared opaque handle.
pub(crate) fn validate_constant_expr(ext: &Declarations, kotlin_name: &str, ty: &syn::Type) {
    assert!(
        result_ok_type(ty).is_none(),
        "constant expr `{kotlin_name}`: type is a `Result` — an expression constant must be \
         infallible (declare a real function with `.fun(...)` instead if it can fail)"
    );
    // The peel, on the SPELLING. Unlike the fn path above there is no reading
    // to ask: a `const_expr!` type is written by the BUILD SCRIPT and names no
    // captured item, so the model never classified it (#280) — the ledger's
    // documented adapter-owned category, and the reason this one node walk
    // stays where its peer's could go.
    let mut ty = ty.clone();
    loop {
        if let syn::Type::Reference(r) = &ty {
            ty = (*r.elem).clone();
            continue;
        }
        if let Some(inner) = option_inner_type(&ty).or_else(|| vec_inner_type(&ty)) {
            ty = inner;
            continue;
        }
        break;
    }
    reject_handle_key(ext, &TypeKey::from_type(&ty), "constant expr", kotlin_name);
}

/// [`emit_jni_function_wrapper`] with the raw callee expression overridable:
/// `None` = the ordinary `<origin module>::<fn ident>(args)` call; `Some(e)`
/// splices `e` verbatim as the value the output phase converts. Used by the
/// const getter emission (`Declarations::on_const`), whose synthetic nullary `f`
/// carries the signature while the value comes from
/// `<origin module>::<CONST_IDENT>` — a path, not a call.
pub(crate) fn emit_jni_function_wrapper_with_callee(
    ext: &Declarations,
    f: &crate::api::core::flat::Function,
    registry: &Registry<KotlinMeta>,
    callee: Option<syn::Expr>,
    emit: &crate::api::core::emit::Emit,
) -> TokenStream {
    let original_ident = &f.name;

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
    let unfold_plan = registry.unfold_plans().get(original_ident);
    // Error-position expansion: when the fn returns `Result<T, E>` and an error
    // plan is declared, the **`?`** is applied here — the extern peels the
    // `Result` (Err arm decomposes `E` into the `ze` leaves and invokes the
    // typed DOMAIN handler), and the success path uses `T`'s converter (not the
    // `Result<T, E>` rank-2 wrapper).
    let error_plan = registry.error_plans().get(original_ident);
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
        let (wp, pre, call_arg) =
            emit_input_param(ext, registry, original_ident, param, &on_err, emit);
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
        // One derivation, shared with the multi-leaf encoder — the value forms
        // are bound by the same [`bind_hoists`] and the leaf reached by the
        // same [`reach_leaf_flat`]. Deriving either a second time here is what
        // let the two drift apart: this shortcut used to compose its reach
        // straight off the raw value, which for a value form declared with
        // `.fields_self_into(..)` emitted `f(&v)` against a by-value receiver.
        let qualify = |id: &syn::Ident| -> syn::Path { ext.fn_module(registry, id) };
        // `None` when the reach is the IDENTITY of `base` and no value form had
        // to be bound — the leaf IS the value, so there is nothing to compose.
        // Reported separately from the composed form because the two want
        // different code: wrapping identity in a block emits `{ __cvsrc }`
        // (`unused_braces`), and mapping it over an `Option` emits
        // `.map(|__inner| __inner)` (`clippy::map_identity`). Generated code
        // runs through the consumer's own lints, where both are denials.
        let compose = |base: TokenStream, base_is_ref: bool| -> Option<TokenStream> {
            let hoisted = bind_hoists(&qualify, &uplan.hoists, &base, base_is_ref);
            let stmts = &hoisted.stmts;
            let reached = match hoisted.rebase(&leaf.path) {
                Some((local, rest, consuming)) => {
                    reach_leaf_flat(&qualify, leaf, &rest, quote!(#local), false, consuming)
                }
                None => {
                    reach_leaf_flat(&qualify, leaf, &leaf.path, base.clone(), base_is_ref, false)
                }
            };
            if stmts.is_empty() && reached.to_string() == base.to_string() {
                return None;
            }
            Some(quote!({ #stmts #reached }))
        };
        match &uplan.shape {
            UnfoldShape::Optional((), _) => match compose(quote!(__inner), by_ref) {
                Some(inner) => quote!({
                    let __cvsrc = #raw_call;
                    __cvsrc.map(|__inner| #inner)
                }),
                None => raw_call.clone(),
            },
            _ => match compose(quote!(__cvsrc), by_ref) {
                Some(v) => quote!({
                    let __cvsrc = #raw_call;
                    #v
                }),
                None => raw_call.clone(),
            },
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
        let (ze_stmts, ze_args) = encode_plan_leaves(
            ext,
            registry,
            ep,
            &eze_idents,
            &quote!(__de),
            &ze_fail,
            emit,
        );
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
        // The delivery **binds** the returned value and matches it against the
        // canonical shape its `kind` names (`Option`, then a run). Conversion
        // follows the SYNTAX, and this position takes no converter — nothing
        // between the source call and the match re-spells anything — so the
        // wrappers the classification erased have to come off here, at the
        // emitter's own binding, or the match is an `E0308` on a spelling the
        // model deliberately reads as optional (#292).
        //
        // The value delivered is the `Ok` side when the error plan applied the
        // `?`, and the return itself otherwise — the wrappers questioned are
        // those over whatever `call_expr` actually yields.
        let delivered = match error_plan {
            Some(_) => f.ret.fallible_parts().map_or(&f.ret, |(ok, _)| ok),
            None => &f.ret,
        };
        let call_expr =
            read_through_erased_wrappers(delivered, call_expr.clone()).unwrap_or_else(|| {
                panic!(
                    "`{original_ident}` returns `{}`, whose leaves are delivered to a builder: \
                     the value has to be moved out of `{}` to be decomposed, and that wrapper \
                     does not permit it (a `Cow` payload cannot be moved through `Deref`). \
                     Reserved rather than refused for good: rebuilding through every \
                     transparent wrapper is #292 item 3 — until then, spell the return \
                     without it.",
                    delivered,
                    delivered.erased_wrappers().join("<"),
                )
            });
        emit_unfold_delivery(
            ext,
            registry,
            uplan,
            u.iface.as_deref(),
            &call_expr,
            &on_err,
            emit,
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
                    static __DSINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod =
                        ::prebindgen_jni_runtime::CachedIfaceMethod::new();
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
        static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod =
            ::prebindgen_jni_runtime::CachedIfaceMethod::new();
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
    ext: &Declarations,
    registry: &Registry<KotlinMeta>,
    original_ident: &syn::Ident,
    param: &PlanParam,
    on_err: &TokenStream,
    emit: &crate::api::core::emit::Emit,
) -> (Vec<TokenStream>, Vec<TokenStream>, TokenStream) {
    // Constructor-expansion: this parameter's wire form is the fold plan's
    // flattened leaves. Decode each leaf with its own converter, run the
    // (pure-Rust) fold to build the value, then pass it to the call.
    let leaf = match &param.form {
        ParamForm::Expanded(leaves) => {
            let fold = registry
                .expansion_plans()
                .get(&(original_ident.clone(), param.ident.clone()))
                .expect("ParamForm::Expanded ⇒ expansion plan present");
            return emit_expanded_param(ext, registry, fold, leaves, &param.ident, on_err, emit);
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
            let (decode, call_arg) = render_flat_input_decode(plan, arg_ident, on_err, emit);
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
            // The rebuilt `Option`, then the wrappers the parameter's spelling
            // adds over it — `Box<Option<T>>` gets its `Box` back here, because
            // nothing between this and the source call re-spells the value.
            // The plan only exists when the build resolves, so this cannot fail.
            let built = build_through_wrappers(
                &sp.arg_wrappers,
                quote! {
                    if #pid != 0u8 {
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
                    }
                },
            )
            .expect("an option-scalar plan is built only for a buildable spelling");
            prelude.push(quote! {
                let #arg_ident = #built;
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
            // Generated Rust spells the reading's own tokens.
            let elem = emit.spell(elem);
            let handle_ident = format_ident!("{}_handle", arg_ident);
            wire_params.push(quote!(#handle_ident: jni::sys::jlong));
            if *by_ref {
                // `vec_build_elem` refuses a wrapped run on this path, so the
                // borrow is the parameter's own spelling and there is nothing
                // to put back.
                prelude.push(quote!(
                    let #arg_ident: &[#elem] =
                        unsafe { &*(#handle_ident as *const Vec<#elem>) };
                ));
            } else {
                // By value the local is owned, so the run's wrappers go back on
                // for free — `Box<Vec<T>>` is `Box::new(mem::take(..))`. The
                // ascription is dropped rather than restated: the wrapped
                // spelling is what the expression now produces, and naming it
                // here would be the same fact written twice.
                let taken = build_through_erased_wrappers(
                    &leaf.reading,
                    quote!(unsafe {
                        ::core::mem::take(&mut *(#handle_ident as *mut Vec<#elem>))
                    }),
                )
                .expect("vec_build_elem accepted this run spelling");
                prelude.push(quote!(let #arg_ident = #taken;));
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
        InputKind::Handle { direct: true }
            if !matches!(arg_ty.kind(), crate::api::core::flat::TypeKind::Ref { .. }) =>
        {
            let entry = registry
                .input_entry(arg_ty)
                .expect("plan classified Handle ⇒ entry present");
            let wire_ident = if matches!(&entry.destination, syn::Type::Ptr(_)) {
                format_ident!("{}_ptr", arg_ident)
            } else {
                arg_ident.clone()
            };
            wire_params.push(quote!(#wire_ident: jni::sys::jlong));
            let arg_ty = emit.spell(arg_ty);
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
        | InputKind::Unsigned64 { .. }
        | InputKind::Plain => {
            // The leaf's reading — for `ParamForm::Single` it is the very
            // reading `param.ty` was spelled from, so this is the same lookup
            // without the round trip. The panic now CALLS the shared message
            // instead of restating it, which is what `PlanError::message`'s doc
            // has always claimed and hand-duplication did not deliver.
            let entry = registry.input_entry(&leaf.reading).unwrap_or_else(|| {
                panic!(
                    "{}",
                    PlanError::Unresolved {
                        ty: Box::new(leaf.reading.clone())
                    }
                    .message(original_ident)
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
    arg_ty: &crate::api::core::flat::TypeRef,
    on_err: &TokenStream,
) -> (Vec<TokenStream>, Vec<TokenStream>, TokenStream) {
    use crate::api::core::flat::TypeKind;
    /// `&mut T`, read off the kind — and off `kind()` rather than through
    /// `is_exclusive_borrow`, which also sees through a `Box` and refuses an
    /// out-parameter's slot. This is the borrow the source wrote.
    fn is_mut_ref(t: &crate::api::core::flat::TypeRef) -> bool {
        matches!(t.kind(), TypeKind::Ref { mutable: true, .. })
    }
    /// `Option<&T>` / `Option<&mut T>` → `Some(is_mut)`, the shape
    /// `option_inner_ref_mutability` used to fish out of a path.
    fn opt_ref_mut(t: &crate::api::core::flat::TypeRef) -> Option<bool> {
        let TypeKind::Optional(inner) = t.kind() else {
            return None;
        };
        match inner.kind() {
            TypeKind::Ref { mutable, .. } => Some(*mutable),
            _ => None,
        }
    }
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
    let arg_mut: TokenStream = if is_mut_ref(arg_ty) || matches!(opt_ref_mut(arg_ty), Some(true)) {
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
    let call_arg = match arg_ty.kind() {
        TypeKind::Ref { mutable: true, .. } => quote!(&mut #arg_ident),
        TypeKind::Ref { .. } => quote!(&#arg_ident),
        // `Option<&T>` / `Option<&mut T>` for opaque inner: the input
        // converter produced `Option<OwnedObject<T>>` (see rank-1
        // handler above). `.as_deref()` / `.as_deref_mut()` coerces
        // back to `Option<&T>` / `Option<&mut T>` via OwnedObject's
        // Deref / DerefMut impls.
        _ if matches!(opt_ref_mut(arg_ty), Some(false)) => {
            quote!(#arg_ident.as_deref())
        }
        _ if matches!(opt_ref_mut(arg_ty), Some(true)) => {
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
    ext: &Declarations,
    registry: &Registry<KotlinMeta>,
    plan: &crate::api::core::expand::FoldPlan,
    leaves: &[PlanLeaf],
    orig_param: &syn::Ident,
    on_err: &TokenStream,
    emit: &crate::api::core::emit::Emit,
) -> (Vec<TokenStream>, Vec<TokenStream>, TokenStream) {
    let mut wire_params: Vec<TokenStream> = Vec::new();
    let mut prelude: Vec<TokenStream> = Vec::new();
    let mut leaf_locals: Vec<syn::Ident> = Vec::new();

    debug_assert_eq!(plan.leaves.len(), leaves.len());
    for (leaf, classified) in plan.leaves.iter().zip(leaves) {
        let leaf_ty = &leaf.ty;
        // The ascription generated Rust writes for this leaf's local.
        let leaf_ty_tokens = emit.spell(leaf_ty);
        let lookup_entry = || {
            // The leaf's own reading goes straight to the entry: spelling it and
            // looking the same reading back up is the round trip #286 removed.
            registry.input_entry(&leaf.ty).unwrap_or_else(|| {
                // Shared wording, not restated — see the sibling backstop above.
                panic!(
                    "{}",
                    PlanError::UnresolvedLeaf {
                        ty: Box::new(leaf.ty.clone()),
                        param: orig_param.clone(),
                    }
                    .message(orig_param)
                )
            })
        };
        let local = format_ident!("__exp_{}", leaf.name);

        // An expansion leaf can itself be a data class. Reuse the recursive
        // plan instead of allowing that leaf to fall back to a JObject, so
        // expansion and ordinary parameters have the same boundary rule.
        if let InputKind::FlattenStruct(flat) = &classified.kind {
            for flat_leaf in &flat.leaves {
                let ident = &flat_leaf.native_ident;
                let wire = &flat_leaf.native_wire_ty;
                wire_params.push(quote!(#ident: #wire));
            }
            let (decode, _) = render_flat_input_decode(flat, &local, on_err, emit);
            prelude.push(decode);
            leaf_locals.push(local);
            continue;
        }

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
            // The local is ascribed the leaf's own SPELLING (`leaf_ty`), so the
            // rebuilt `Option` has to be wrapped back up to match it — the same
            // rule as the parameter path above, and the reason the ascription
            // can stay as written rather than being weakened to the stripped
            // type.
            let built = build_through_wrappers(
                &sp.arg_wrappers,
                quote! {
                    if #present_ident != 0u8 {
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
                    }
                },
            )
            .expect("an option-scalar plan is built only for a buildable spelling");
            prelude.push(quote!(
                let #local: #leaf_ty_tokens = #built;
            ));
            leaf_locals.push(local);
            continue;
        }

        // Direct owned-handle leaf (e.g. an identity-variant `T`): consume the
        // jlong handle inline, mirroring the normal by-value-handle path —
        // including its null/tagged (closed) pointer guard.
        let is_consume = matches!(classified.kind, InputKind::Handle { direct: true })
            && !matches!(leaf_ty.kind(), crate::api::core::flat::TypeKind::Ref { .. });
        if is_consume {
            let wire_ident = format_ident!("{}_ptr", leaf.name);
            wire_params.push(quote!(#wire_ident: jni::sys::jlong));
            prelude.push(quote!(
                if #wire_ident == 0 || (#wire_ident & 1) == 1 {
                    signal_binding_error(&mut env, &__error_sink, &__SINK_MID, __SINK_FQN, __SINK_DESCR, "Operation on a closed native handle.");
                    return #on_err;
                }
                let #local: #leaf_ty_tokens = unsafe {
                    *std::boxed::Box::from_raw(#wire_ident as *mut #leaf_ty_tokens)
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
