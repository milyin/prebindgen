//! `impl Fn(args)` inputs: the native trampoline calling the typed
//! Kotlin `run`.

use super::*;

/// Build the input-converter body for an `impl Fn(args)` parameter: a
/// trampoline that wraps the Kotlin **lambda** (`(leaves…) -> Unit`, erased to
/// `Any` at the extern tier) as a `Box<dyn Fn(args) + Send + Sync>`.
///
/// Each callback arg is delivered the same way a *return* of its type would
/// be: a type with a canonical-output plan ([`Registry::callback_arg_plans`])
/// is decomposed into its leaves via the shared [`encode_plan_leaves`] (the
/// trampoline owns the value — identity-leaf handles transfer to the lambda,
/// never closed by Rust); a plan-less opaque-handle type is boxed into a fresh
/// typed handle that is `close()`-d after the invoke (no-op if `take()`-ed);
/// anything else crosses whole through its output converter. All objects feed
/// one erased `invoke(Object…)` — a single JNI crossing per invocation.
///
/// Errors cannot reach a caller-side error sink (the declaring call already
/// returned), so they are converted to `__JniErr` and logged via `tracing`.
pub(crate) fn callback_input(
    ext: &JniGen,
    args: &[syn::Type],
    registry: &Registry<KotlinMeta>,
) -> Option<(syn::Type, syn::Expr)> {
    // Human-readable tag for attach/log messages.
    let name = format!(
        "Fn({})",
        args.iter()
            .map(|t| TypeKey::from_type(t).to_string())
            .collect::<Vec<_>>()
            .join(", ")
    );
    let name_lit = syn::LitStr::new(&name, Span::call_site());

    // Trampoline error path for the shared leaf encoder: convert the message
    // to `__JniErr` inside the per-invocation `Result` closure.
    let fail = |msg: TokenStream| -> TokenStream {
        quote! {
            return ::core::result::Result::Err(
                <__JniErr as ::core::convert::From<String>>::from(#msg));
        }
    };

    let arg_names: Vec<syn::Ident> = (0..args.len())
        .map(|i| format_ident!("__cb_arg{}", i))
        .collect();
    let arg_pat_ty: Vec<TokenStream> = args.iter().map(|t| quote!(#t)).collect();

    // Per-arg encode preludes binding the typed `run`'s args in declared
    // order (a decomposed arg contributes one arg per leaf). Each entry of
    // `jvalue_exprs` is a typed `jvalue`: raw primitives for primitive-wire
    // leaves, `{ l: obj.as_raw() }` for object leaves — matching the
    // descriptor of the generated callback interface's `run`.
    let mut preludes: Vec<TokenStream> = Vec::new();
    let mut jvalue_exprs: Vec<TokenStream> = Vec::new();
    let mut total: usize = 0;
    // One-time setup statements (folder singleton + method id for an
    // `&[data_class]` fold arg), spliced before the `Box::new` so the move
    // closure captures them.
    let mut fold_setups: Vec<TokenStream> = Vec::new();

    for (i, arg_ty) in args.iter().enumerate() {
        let cb_arg = &arg_names[i];

        // `&[data_class]` fold arg: instead of building the whole `List` on the
        // Rust side, allocate an empty `ArrayList` and fold each element's raw
        // leaves through the hoisted `__<Folder>Holder.instance` (Kotlin does
        // `fromParts` + `add`), then deliver the assembled list whole to the
        // user callback's `run(List<T>)`. Reuses the OUTPUT fold's folder
        // interface + appender singleton, driven from the trampoline.
        if let Some(plan) = registry
            .callback_arg_plans
            .get(&TypeKey::from_type(arg_ty))
            .filter(|p| super::render::is_iterable_fold(&p.shape))
        {
            // Every leaf converter must already be resolved (deferral safety).
            for leaf in &plan.leaves {
                registry.output_entry(&leaf.out_ty)?;
            }
            let spec = folder_iface_for_plan(ext, registry, plan)?;
            let holder_slash =
                syn::LitStr::new(&spec.singleton_holder_slash_fqn(), Span::call_site());
            let field_lit = syn::LitStr::new(
                crate::api::lang::jnigen::jni::SINGLETON_FIELD,
                Span::call_site(),
            );
            let field_sig =
                syn::LitStr::new(&format!("L{};", spec.raw_slash_fqn()), Span::call_site());
            let run_cls = syn::LitStr::new(&spec.raw_slash_fqn(), Span::call_site());
            let run_descr = syn::LitStr::new(&spec.descr, Span::call_site());
            let fold_obj = format_ident!("__fold{}_obj", i);
            let fold_id = format_ident!("__fold{}_id", i);
            // Setup once (captured): fetch the appender singleton (a `@JvmField`
            // in its holder object) as a global ref, and resolve its `run`
            // method id on the folder interface class.
            fold_setups.push(quote! {
                let #fold_obj = {
                    let __cls = env.find_class(#holder_slash)
                        .map_err(|e| <__JniErr as ::core::convert::From<String>>::from(format!("find folder holder {}: {}", #holder_slash, e)))?;
                    let __field = env.get_static_field(&__cls, #field_lit, #field_sig)
                        .and_then(|__v| __v.l())
                        .map_err(|e| <__JniErr as ::core::convert::From<String>>::from(format!("fetch folder singleton {}.{}: {}", #holder_slash, #field_lit, e)))?;
                    env.new_global_ref(&__field)
                        .map_err(|e| <__JniErr as ::core::convert::From<String>>::from(format!("global-ref folder singleton: {}", e)))?
                };
                let #fold_id = {
                    let __cls = env.find_class(#run_cls)
                        .map_err(|e| <__JniErr as ::core::convert::From<String>>::from(format!("find folder iface {}: {}", #run_cls, e)))?;
                    env.get_method_id(&__cls, "run", #run_descr)
                        .map_err(|e| <__JniErr as ::core::convert::From<String>>::from(format!("resolve folder run {}: {}", #run_cls, e)))?
                };
            });
            // Per fire: one `ArrayList`, fold each element's leaves through the
            // appender (which mutates the list in place and returns it — the
            // return is ignored). Each element's leaf locals live in a nested
            // local frame so they are freed per element (the daemon-thread
            // local-ref discipline — only the `acc` ref crosses iterations).
            let acc = format_ident!("__fold{}_acc", i);
            let obj_idents: Vec<syn::Ident> = (0..plan.leaves.len())
                .map(|k| format_ident!("__cbfold{}_obj{}", i, k))
                .collect();
            let (leaf_stmts, leaf_args) =
                encode_plan_leaves(ext, registry, plan, &obj_idents, &quote!(__cb_elem), &fail);
            let elem_frame = std::cmp::max(16, 2 * plan.leaves.len() + 6);
            let elem_frame_lit = syn::LitInt::new(&elem_frame.to_string(), Span::call_site());
            preludes.push(quote! {
                let #acc: jni::objects::JObject = env
                    .new_object("java/util/ArrayList", "()V", &[])
                    .map_err(|e| <__JniErr as ::core::convert::From<String>>::from(format!("fold: new ArrayList: {}", e)))?;
                for __cb_elem in #cb_arg.iter() {
                    env.push_local_frame(#elem_frame_lit)
                        .map_err(|e| <__JniErr as ::core::convert::From<String>>::from(format!("fold: push frame: {}", e)))?;
                    let __fold_res = (|| -> ::core::result::Result<(), __JniErr> {
                        #leaf_stmts
                        // The appender returns the same list it mutates, so the
                        // result is discarded; `#acc` (an outer-frame ref) stays
                        // valid across the nested frame.
                        let _ = unsafe {
                            env.call_method_unchecked(
                                &#fold_obj,
                                #fold_id,
                                jni::signature::ReturnType::Object,
                                &[jni::sys::jvalue { l: #acc.as_raw() }, #(#leaf_args),*],
                            )
                        }
                        .map_err(|e| {
                            let _ = env.exception_describe();
                            <__JniErr as ::core::convert::From<String>>::from(format!("fold run: {}", e))
                        })?;
                        ::core::result::Result::Ok(())
                    })();
                    let _ = unsafe { env.pop_local_frame(&jni::objects::JObject::null()) };
                    __fold_res?;
                }
            });
            jvalue_exprs.push(quote!(jni::sys::jvalue { l: #acc.as_raw() }));
            total += 1;
            continue;
        }

        // Decomposed arg: deliver the leaves of its type-level canonical
        // output, exactly like a return delivery.
        if let Some(plan) = registry.callback_arg_plans.get(&TypeKey::from_type(arg_ty)) {
            // Deferral safety: every leaf converter (and identity-leaf
            // projection) must already be resolved — return None so the rank
            // resolver retries this converter later otherwise.
            for leaf in &plan.leaves {
                let e = registry.output_entry(&leaf.out_ty)?;
                if leaf.identity && e.metadata.projection.is_none() {
                    return None;
                }
            }
            let obj_idents: Vec<syn::Ident> = (0..plan.leaves.len())
                .map(|k| format_ident!("__cb{}_obj{}", i, k))
                .collect();
            let (stmts, arg_exprs) =
                encode_plan_leaves(ext, registry, plan, &obj_idents, &quote!(#cb_arg), &fail);
            preludes.push(stmts);
            total += arg_exprs.len();
            jvalue_exprs.extend(arg_exprs);
            continue;
        }

        // Whole-value delivery. A by-value arg (`impl Fn(T)`) has a `T` output
        // converter and is passed by move. A borrowed whole-value arg
        // (`impl Fn(&T)` for a type with no accessor plan — e.g. a field-based
        // `data_class` like `Payload`) has no `&T` converter, so fall back to `T`'s
        // converter and clone the borrow (the callback only borrows the value). The
        // `data_class` converter composes the whole object via `fromParts`, so the
        // Kotlin `run(t: T)` receives a ready-made `T`.
        let (cb_val, arg_entry) = match registry.output_entry(arg_ty) {
            Some(e) => (quote!(#cb_arg), e),
            None => match arg_ty {
                syn::Type::Reference(r) => {
                    let core = (*r.elem).clone();
                    (quote!((#cb_arg).clone()), registry.output_entry(&core)?)
                }
                _ => return None,
            },
        };
        let arg_wire = arg_entry.destination.clone();
        let conv = arg_entry.function.sig.ident.clone();
        let enc_ident = format_ident!("__cb{}_enc", i);
        let obj_ident = format_ident!("__cb{}_obj", i);

        // Plan-less opaque-handle arg: encode to a raw `jlong` (`Box::into_raw`)
        // and deliver it as-is. The typed handle class is constructed Kotlin-side
        // by the generated `asRaw` proxy (`WrapKind::HandleOwned`), which also
        // `close()`s it after `run` (close-unless-taken) — so no Rust
        // `new_object` and no post-invoke close. The Kotlin wrap lets a queryable
        // consumer reply through the handle inside the callback (a consuming
        // reply zeroes the slot, making the proxy's `close` a no-op). See
        // `owned_handle_iface_param`.
        if let Some(h) = &arg_entry.metadata.projection {
            if matches!(h.kind, ProjectionKind::Handle) {
                preludes.push(quote! {
                    let #enc_ident = #conv(&mut env, #cb_val)?;
                });
                jvalue_exprs.push(quote!(jni::sys::jvalue { j: #enc_ident }));
                total += 1;
                continue;
            }
        }

        // Whole-value arg (scalar / String / data-class / value-blob …):
        // encode with its output converter. A non-`Option` primitive-wire arg
        // passes its raw primitive; everything else casts to JObject. Output
        // converters take the value by move; `cb_arg` is the closure
        // parameter, so pass it directly.
        let arg_is_prim = arg_entry.metadata.projection.is_none()
            && !is_option_type(arg_ty)
            && matches!(jni_field_access(&arg_wire), Some((_, _, false)));
        if arg_is_prim {
            let letter = jni_field_access(&arg_wire).unwrap().1;
            preludes.push(quote! {
                let #enc_ident = #conv(&mut env, #cb_val)?;
            });
            jvalue_exprs.push(quote!(jni::sys::jvalue { #letter: #enc_ident }));
            total += 1;
            continue;
        }
        let cast = cast_wire_to_jobject(&enc_ident, &arg_wire, &fail);
        preludes.push(quote! {
            let #enc_ident = #conv(&mut env, #cb_val)?;
            let #obj_ident: jni::objects::JObject = #cast;
        });
        jvalue_exprs.push(quote!(jni::sys::jvalue { l: #obj_ident.as_raw() }));
        total += 1;
    }

    // Typed `run` descriptor of the generated callback interface — derived
    // from the same plans/leaf classification as the jvalues above.
    let spec = callback_iface_spec(ext, registry, args)?;
    let descr_lit = syn::LitStr::new(&spec.descr, Span::call_site());
    // Local-frame capacity: roughly an encoded wire + a wrapped object per
    // delivered leaf, plus call temporaries.
    let frame_cap = std::cmp::max(16, 2 * total + 6);
    let frame_cap_lit = syn::LitInt::new(&frame_cap.to_string(), Span::call_site());

    let body: syn::Expr = syn::parse_quote!({
        use std::sync::Arc;
        let java_vm = Arc::new(env.get_java_vm()
            .map_err(|e| <__JniErr as ::core::convert::From<String>>::from(format!("Unable to retrieve JVM: {}", e)))?);
        let callback_global_ref = env.new_global_ref(&v)
            .map_err(|e| <__JniErr as ::core::convert::From<String>>::from(format!("Unable to global-ref callback: {}", e)))?;
        // Resolve the typed callback interface's `run` method ID ONCE, here
        // at trampoline creation. The safe `JNIEnv::call_method` re-parses
        // the descriptor string (a `combine`-parser run) and re-resolves the
        // method through the JVM symbol table on EVERY call — measured at
        // ~33% of per-message delivery time on the subscriber hot path. The
        // `JMethodID` is `Copy + Send + Sync` and stays valid for the
        // closure's lifetime: the global ref pins the callback instance and
        // therefore its class.
        let __invoke_class = env.get_object_class(&v)
            .map_err(|e| <__JniErr as ::core::convert::From<String>>::from(format!("Unable to get callback class for {}: {}", #name_lit, e)))?;
        let __invoke_id = env.get_method_id(&__invoke_class, "run", #descr_lit)
            .map_err(|e| <__JniErr as ::core::convert::From<String>>::from(format!("Unable to resolve run for {}: {}", #name_lit, e)))?;
        // One-time fold setup (folder singleton global ref + `run` method id),
        // captured by the move closure below.
        #(#fold_setups)*
        Box::new(move |#(#arg_names: #arg_pat_ty),*| {
            let _ = (|| -> ::core::result::Result<(), __JniErr> {
                let mut env = java_vm
                    .attach_current_thread_as_daemon()
                    .map_err(|e| <__JniErr as ::core::convert::From<String>>::from(format!("Attach thread for {}: {}", #name_lit, e)))?;
                // The callback fires on a daemon-attached zenoh RX thread that
                // never returns through a JNI stack frame, so the JNI local
                // refs each invocation creates (encoded leaves, wrapped handle
                // objects, call temporaries) would otherwise accumulate for
                // the thread's lifetime and exhaust the JVM heap
                // (OutOfMemoryError). Bracket each invocation in an explicit
                // local frame so every local is released when the frame pops —
                // popped unconditionally below so an early `?`/error path
                // still frees it.
                env.push_local_frame(#frame_cap_lit)
                    .map_err(|e| <__JniErr as ::core::convert::From<String>>::from(format!("push local frame for {}: {}", #name_lit, e)))?;
                let __frame_res = (|| -> ::core::result::Result<(), __JniErr> {
                    #(#preludes)*
                    // SAFETY: `__invoke_id` was resolved on this exact
                    // callback object's class with this exact descriptor, and
                    // the global ref keeps the class loaded. Exception-check
                    // semantics are identical to the safe `call_method` (both
                    // route through the same checked JNI invoke). `run`
                    // returns void; primitives ride the jvalues raw.
                    let __call_res: ::core::result::Result<(), __JniErr> = unsafe {
                        env.call_method_unchecked(
                            &callback_global_ref,
                            __invoke_id,
                            jni::signature::ReturnType::Primitive(jni::signature::Primitive::Void),
                            &[#(#jvalue_exprs),*],
                        )
                    }
                    .map(|_| ())
                    .map_err(|e| {
                        // `exception_describe` also clears the pending exception.
                        let _ = env.exception_describe();
                        <__JniErr as ::core::convert::From<String>>::from(e.to_string())
                    });
                    // A plan-less opaque-handle arg's per-invocation `Box` is
                    // freed Kotlin-side by the `asRaw` proxy's `finally { close() }`
                    // (close-unless-taken), so there is no Rust-side close here.
                    __call_res?;
                    Ok(())
                })();
                // Pop the frame unconditionally so locals are freed even when
                // the body above returned `Err` early.
                let _ = unsafe { env.pop_local_frame(&jni::objects::JObject::null()) };
                __frame_res?;
                Ok(())
            })()
            .map_err(|e| tracing::error!("{} callback error: {e}", #name_lit));
        })
    });

    // The wire type for an `impl Fn(args)` parameter is JObject (the erased
    // Kotlin lambda). The converter returns Box<dyn Fn(args) + Send + Sync>,
    // which coerces to the source's impl-trait param type.
    Some((syn::parse_quote!(jni::objects::JObject), body))
}

/// Hard-error guard for `Vec<opaque-handle>` element types. A handle's wire is
/// a `jlong` heap pointer and a `Vec<that>` would yield a collection of
/// closeable native handles the JVM must free one-by-one — unsupported. Detect
/// it by the element's folded [`Projection`] being a [`ProjectionKind::Handle`]
/// and panic with a fix hint, instead of the `Vec<_>` handler silently
/// `return None`-ing (which surfaces as an opaque "unresolved type" error).
pub(crate) fn reject_vec_of_handle(inner_projection: &Option<Projection>, elem: &syn::Type) {
    if let Some(p) = inner_projection {
        if p.kind == ProjectionKind::Handle {
            panic!(
                "JniGen: `Vec<{}>` is unsupported — its elements would be closeable native \
                 handles (jlong) the JVM must free individually. If `{}` is `Copy`, declare \
                 it as a value class via `.value_class(...)` so the Vec surfaces as \
                 `List<ByteArray>`; otherwise expose a per-element accessor instead of \
                 returning a `Vec` of handles.",
                elem.to_token_stream(),
                elem.to_token_stream(),
            );
        }
    }
}

/// Reconstruct the `impl Fn(args...) + Send + Sync + 'static` syn::Type
/// from a flat slice of arg types. Used by the rank-1/2/3 callback impls
/// to feed `input_wrapper` the original outer type.
pub(crate) fn build_fn_type(args: &[syn::Type]) -> syn::Type {
    let arg_iter = args.iter();
    syn::parse_quote!(impl Fn( #(#arg_iter),* ) + Send + Sync + 'static)
}
