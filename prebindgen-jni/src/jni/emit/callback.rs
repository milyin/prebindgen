//! `impl Fn(args)` inputs: the native trampoline calling the typed
//! Kotlin `run`.

use prebindgen_registry::{chain::Chain as _, Conversions};

use super::*;

#[derive(Clone)]
struct JInvokePart {
    prepare: TokenStream,
    arguments: Vec<TokenStream>,
}

impl prebindgen_registry::chain::InvokePart for JInvokePart {
    fn render(
        &self,
        _value: TokenStream,
        _index: usize,
        _emit: &prebindgen_registry::Emit,
    ) -> prebindgen_registry::chain::RenderedInvokePart {
        prebindgen_registry::chain::RenderedInvokePart {
            prepare: self.prepare.clone(),
            arguments: self.arguments.clone(),
            cleanup: TokenStream::new(),
        }
    }
}

#[derive(Clone)]
struct JInvokeBridge {
    name: syn::LitStr,
    descriptor: syn::LitStr,
    frame_capacity: syn::LitInt,
    fold_setups: Vec<TokenStream>,
}

/// A registry-composed callback retained until the final Rust writer runs.
#[derive(Clone)]
pub(crate) struct JInvokePlan {
    name: syn::Ident,
    chain:
        prebindgen_registry::chain::Invoke<crate::jni::chain::JSource, JInvokeBridge, JInvokePart>,
}

impl JInvokePlan {
    pub(crate) fn render(&self, emit: &prebindgen_registry::Emit) -> syn::ItemFn {
        let rendered = self.chain.render(emit);
        let name = &self.name;
        let source = &rendered.source;
        let body = &rendered.body;
        let gen_allow = crate::jni::trait_impl::generated_converter_attr();
        syn::parse_quote!(
            #gen_allow
            pub(crate) unsafe fn #name<'env, 'v>(
                env: &mut jni::JNIEnv<'env>,
                v: &jni::objects::JObject<'v>,
            ) -> ::core::result::Result<#source, __JniErr> {
                ::core::result::Result::Ok(#body)
            }
        )
    }
}

impl prebindgen_registry::chain::InvokeBridge for JInvokeBridge {
    fn intermediate(&self) -> syn::Type {
        syn::parse_quote!(jni::objects::JObject)
    }

    fn argument_name(&self, index: usize) -> syn::Ident {
        format_ident!("__cb_arg{}", index)
    }

    fn capture(&self, value: TokenStream, closure: TokenStream) -> TokenStream {
        let name = &self.name;
        let descriptor = &self.descriptor;
        let fold_setups = &self.fold_setups;
        quote!({
            use std::sync::Arc;
            let java_vm = Arc::new(env.get_java_vm()
                .map_err(|e| <__JniErr as ::core::convert::From<String>>::from(format!("Unable to retrieve JVM: {}", e)))?);
            let callback_global_ref = env.new_global_ref(&#value)
                .map_err(|e| <__JniErr as ::core::convert::From<String>>::from(format!("Unable to global-ref callback: {}", e)))?;
            let __invoke_class = env.get_object_class(&#value)
                .map_err(|e| <__JniErr as ::core::convert::From<String>>::from(format!("Unable to get callback class for {}: {}", #name, e)))?;
            let __invoke_id = env.get_method_id(&__invoke_class, "run", #descriptor)
                .map_err(|e| <__JniErr as ::core::convert::From<String>>::from(format!("Unable to resolve run for {}: {}", #name, e)))?;
            #(#fold_setups)*
            Box::new(#closure)
        })
    }

    fn invoke(&self, arguments: &[TokenStream]) -> TokenStream {
        quote! {
            let __call_res: ::core::result::Result<(), __JniErr> = unsafe {
                env.call_method_unchecked(
                    &callback_global_ref,
                    __invoke_id,
                    jni::signature::ReturnType::Primitive(jni::signature::Primitive::Void),
                    &[#(#arguments),*],
                )
            }
            .map(|_| ())
            .map_err(|e| {
                let _ = env.exception_describe();
                <__JniErr as ::core::convert::From<String>>::from(e.to_string())
            });
            __call_res?;
        }
    }

    fn surround(
        &self,
        prepare: TokenStream,
        invoke: TokenStream,
        cleanup: TokenStream,
    ) -> TokenStream {
        let name = &self.name;
        let frame_capacity = &self.frame_capacity;
        quote!({
            let _ = (|| -> ::core::result::Result<(), __JniErr> {
                let mut env = java_vm
                    .attach_current_thread_as_daemon()
                    .map_err(|e| <__JniErr as ::core::convert::From<String>>::from(format!("Attach thread for {}: {}", #name, e)))?;
                env.push_local_frame(#frame_capacity)
                    .map_err(|e| <__JniErr as ::core::convert::From<String>>::from(format!("push local frame for {}: {}", #name, e)))?;
                let __frame_res = (|| -> ::core::result::Result<(), __JniErr> {
                    #prepare
                    #invoke
                    #cleanup
                    Ok(())
                })();
                let _ = unsafe { env.pop_local_frame(&jni::objects::JObject::null()) };
                __frame_res?;
                Ok(())
            })()
            .map_err(|e| tracing::error!("{} callback error: {e}", #name));
        })
    }

    fn fallible(&self) -> bool {
        true
    }
}

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
    ext: &Declarations,
    source: &prebindgen_registry::flat::TypeRef,
    args: &[prebindgen_registry::flat::TypeRef],
    registry: &impl Conversions,
    arg_fragments: Option<&[&crate::jni::compile::JFrag]>,
    emit: &prebindgen_registry::Emit,
) -> Option<(syn::Type, syn::Type, syn::Expr, JInvokePlan)> {
    // Human-readable tag for attach/log messages.
    let name = format!(
        "Fn({})",
        args.iter()
            .map(|t| t.key().to_string())
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
    let mut part_ranges = Vec::with_capacity(args.len());

    for (i, arg_ty) in args.iter().enumerate() {
        let prelude_start = preludes.len();
        let argument_start = jvalue_exprs.len();
        let cb_arg = &arg_names[i];

        // `&[data_class]` fold arg: instead of building the whole `List` on the
        // Rust side, allocate an empty `ArrayList` and fold each element's raw
        // leaves through the hoisted `__<Folder>Holder.instance` (Kotlin does
        // `fromParts` + `add`), then deliver the assembled list whole to the
        // user callback's `run(List<T>)`. Reuses the OUTPUT fold's folder
        // interface + appender singleton, driven from the trampoline.
        if let Some(plan) = registry
            .callback_arg_plan(&arg_ty.key())
            .filter(|p| super::render::is_iterable_fold(&p.shape))
        {
            // Every leaf converter must already be resolved (deferral safety).
            // A synthesized leaf (a sum's tag) has no converter to wait for.
            for leaf in plan.leaves.iter().filter(|l| l.has_converter()) {
                ext.out_frag(&leaf.out_ty)?;
            }
            let spec = folder_iface_for_plan(ext, registry, plan)?;
            let holder_slash =
                syn::LitStr::new(&spec.singleton_holder_slash_fqn(), Span::call_site());
            let field_lit = syn::LitStr::new(crate::jni::SINGLETON_FIELD, Span::call_site());
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
            let (leaf_stmts, leaf_args, _) = encode_plan_leaves(
                ext,
                registry,
                crate::jni::emit::Delivered::with_chain(
                    plan,
                    arg_fragments
                        .and_then(|fragments| fragments.get(i))
                        .and_then(|fragment| fragment.composed_chain()),
                ),
                &obj_idents,
                &quote!(__cb_elem),
                &fail,
                emit,
            );
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
            part_ranges.push((
                prelude_start,
                preludes.len(),
                argument_start,
                jvalue_exprs.len(),
            ));
            continue;
        }

        // Decomposed arg: deliver the leaves of its type-level canonical
        // output, exactly like a return delivery.
        if let Some(plan) = effective_callback_plan(ext, registry, arg_ty) {
            // Deferral safety: every leaf converter (and identity-leaf
            // projection) must already be resolved — return None so the rank
            // resolver retries this converter later otherwise. A synthesized
            // leaf (a sum's tag) has no converter to wait for: requiring one
            // would make the trampoline wait forever on an `i32` crossing the
            // binding may not have.
            for leaf in plan.leaves.iter().filter(|l| l.has_converter()) {
                let e = ext.out_frag(&leaf.out_ty)?;
                if leaf.identity && e.metadata.projection.is_none() {
                    return None;
                }
            }
            let obj_idents: Vec<syn::Ident> = (0..plan.leaves.len())
                .map(|k| format_ident!("__cb{}_obj{}", i, k))
                .collect();
            let (stmts, mut arg_exprs, present) = encode_plan_leaves(
                ext,
                registry,
                crate::jni::emit::Delivered::with_chain(
                    plan,
                    arg_fragments
                        .and_then(|fragments| fragments.get(i))
                        .and_then(|fragment| fragment.composed_chain()),
                ),
                &obj_idents,
                &quote!(#cb_arg),
                &fail,
                emit,
            );
            let optional = plan.is_optional_base();
            if optional && present.is_none() {
                return None;
            }
            if let Some(present) = present {
                arg_exprs.insert(0, quote!(jni::sys::jvalue { z: #present }));
            }
            preludes.push(stmts);
            total += arg_exprs.len();
            jvalue_exprs.extend(arg_exprs);
            part_ranges.push((
                prelude_start,
                preludes.len(),
                argument_start,
                jvalue_exprs.len(),
            ));
            continue;
        }

        // Whole-value delivery. A by-value arg (`impl Fn(T)`) has a `T` output
        // converter and is passed by move. A borrowed whole-value arg
        // (`impl Fn(&T)` for a type with no accessor plan — e.g. a field-based
        // `data_class` like `Payload`) has no `&T` converter, so fall back to `T`'s
        // converter and clone the borrow (the callback only borrows the value). The
        // `data_class` converter composes the whole object via `fromParts`, so the
        // Kotlin `run(t: T)` receives a ready-made `T`.
        let (cb_val, arg_entry) = match ext.out_frag(arg_ty) {
            Some(e) => (quote!(#cb_arg), e),
            // A borrow: the callback hands out a reference, and the value is
            // cloned for the JVM.
            //
            // That this is a borrow is the model's answer (`borrow_target`) —
            // no spelling is inspected here, which is the point of #229.
            //
            // `(#cb_arg).clone()` is nonetheless only well-typed for a
            // DIRECTLY-spelled `&T`: `#cb_arg` carries the source's spelling,
            // so a transparent wrapper — `Box<&T>`, `Ref` all the same —
            // would clone to `Box<&T>`, which the `T` converter rejects. That
            // case is refused before it reaches here: a callback arg's
            // whole-value output converter is a *required* type, and
            // `output_wrapper_shape`'s borrowed-opaque arm matches
            // `syn::Type::Reference` on `produced` structurally, so `Box<&T>`
            // (a `Type::Path`) never gets one.
            //
            // MEASURED, not assumed — `a_wrapped_borrow_callback_arg_declines`
            // fails, on exactly this clone, if that arm is made
            // spelling-blind. A local re-check here was tried (#279 review)
            // and dropped: it changed no output on `Box<&String>` or
            // `Box<&ZThing>`, and added a classification that never fires.
            None => {
                let core = arg_ty.borrow_target()?;
                (quote!((#cb_arg).clone()), ext.out_frag(core)?)
            }
        };
        arg_entry.activate();
        let arg_wire = arg_entry.destination.clone();
        let enc_ident = format_ident!("__cb{}_enc", i);
        let obj_ident = format_ident!("__cb{}_obj", i);

        // The arg's COMPLETE Rust -> wire chain: the rust-side stages a custom
        // `convert!` declaration inserts (`Duration -> u64`), then the
        // wire-facing converter (`u64 -> jlong`). A whole-value callback arg
        // has no leaf plan, so this is its own encoder path — calling only the
        // wire-facing converter would hand it the semantic value where it
        // expects the representation. Stage locals are keyed on the arg index.
        let conv = {
            let converter = arg_entry.converter_ident();
            if arg_entry.pre_stages.is_empty() {
                quote!(#converter(&mut env, #cb_val)?)
            } else {
                let mut body = TokenStream::new();
                let mut previous = cb_val.clone();
                for (order, (_, stage)) in arg_entry.output_stage_order().enumerate() {
                    let stage_fn = &stage.function.sig.ident;
                    let next = format_ident!("__cb{}_s{}", i, order);
                    body.extend(quote! {
                        let #next = #stage_fn(&mut env, #previous)
                            .map_err(|__e| <__JniErr as ::core::convert::From<String>>::from(
                                __e.to_string()))?;
                    });
                    previous = quote!(#next);
                }
                quote!({ #body #converter(&mut env, #previous)? })
            }
        };

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
                    let #enc_ident = #conv;
                });
                jvalue_exprs.push(quote!(jni::sys::jvalue { j: #enc_ident }));
                total += 1;
                part_ranges.push((
                    prelude_start,
                    preludes.len(),
                    argument_start,
                    jvalue_exprs.len(),
                ));
                continue;
            }
        }

        // Whole-value arg (scalar / String / data-class …):
        // encode with its output converter. A non-`Option` primitive-wire arg
        // passes its raw primitive; everything else casts to JObject. Output
        // converters take the value by move; `cb_arg` is the closure
        // parameter, so pass it directly.
        let arg_is_prim = arg_entry
            .metadata
            .projection
            .as_ref()
            .is_none_or(|p| p.kind == ProjectionKind::Unsigned64)
            && arg_ty.optional_inner().is_none()
            && matches!(jni_field_access(&arg_wire), Some((_, _, false)));
        if arg_is_prim {
            let letter = jni_field_access(&arg_wire).unwrap().1;
            preludes.push(quote! {
                let #enc_ident = #conv;
            });
            jvalue_exprs.push(quote!(jni::sys::jvalue { #letter: #enc_ident }));
            total += 1;
            part_ranges.push((
                prelude_start,
                preludes.len(),
                argument_start,
                jvalue_exprs.len(),
            ));
            continue;
        }
        let cast = cast_wire_to_jobject(&enc_ident, &arg_wire, &fail);
        preludes.push(quote! {
            let #enc_ident = #conv;
            let #obj_ident: jni::objects::JObject = #cast;
        });
        jvalue_exprs.push(quote!(jni::sys::jvalue { l: #obj_ident.as_raw() }));
        total += 1;
        part_ranges.push((
            prelude_start,
            preludes.len(),
            argument_start,
            jvalue_exprs.len(),
        ));
    }

    // Typed `run` descriptor of the generated callback interface — the SAME
    // memoized spec (`SpecKey::Callback`) the wrapper surface and the
    // interface declaration read, so it cannot drift from the jvalues above.
    // The memo key holds `TypeKey`s — `SpecKey` needs `Ord`, which a `TypeRef`
    // cannot give — so it is keyed off each arg's own identity, which
    // `a_callback_identity_is_the_same_from_the_reading_or_the_syntax` pins as
    // the SAME identity the signature-derived key produces.
    let spec = ext.iface_spec(registry, &SpecKey::callback(args))?;
    let descr_lit = syn::LitStr::new(&spec.descr, Span::call_site());
    // Local-frame capacity: roughly an encoded wire + a wrapped object per
    // delivered leaf, plus call temporaries.
    let frame_cap = std::cmp::max(16, 2 * total + 6);
    let frame_cap_lit = syn::LitInt::new(&frame_cap.to_string(), Span::call_site());

    let parts = part_ranges
        .into_iter()
        .map(|(ps, pe, as_, ae)| JInvokePart {
            prepare: {
                let values = &preludes[ps..pe];
                quote!(#(#values)*)
            },
            arguments: jvalue_exprs[as_..ae].to_vec(),
        })
        .collect();
    let chain = prebindgen_registry::chain::Invoke {
        source: source.clone(),
        arguments: args.to_vec(),
        source_policy: crate::jni::chain::JSource {
            wrappers: Vec::new(),
            module: None,
        },
        bridge: JInvokeBridge {
            name: name_lit,
            descriptor: descr_lit,
            frame_capacity: frame_cap_lit,
            fold_setups,
        },
        parts,
    };
    let rendered = chain.render(emit);
    let rust_plan = JInvokePlan {
        name: input_name(&rendered.source.to_token_stream(), &rendered.intermediate),
        chain,
    };

    // The wire type for an `impl Fn(args)` parameter is JObject (the erased
    // Kotlin lambda). The converter returns Box<dyn Fn(args) + Send + Sync>,
    // which coerces to the source's impl-trait param type.
    Some((
        rendered.source,
        rendered.intermediate,
        rendered.body,
        rust_plan,
    ))
}

/// Hard-error guard for `Vec<opaque-handle>` element types. A handle's wire is
/// a `jlong` heap pointer and a `Vec<that>` would yield a collection of
/// closeable native handles the JVM must free one-by-one — unsupported. Detect
/// it by the element's folded [`Projection`] being a [`ProjectionKind::Handle`]
/// and panic with a fix hint, instead of the `Vec<_>` handler silently
/// `return None`-ing (which surfaces as an opaque "unresolved type" error).
pub(crate) fn reject_vec_of_handle(
    inner_projection: &Option<Projection>,
    elem: &prebindgen_registry::flat::TypeRef,
) {
    if let Some(p) = inner_projection {
        if p.kind == ProjectionKind::Handle {
            panic!(
                "JniGen: `Vec<{}>` is unsupported — its elements would be closeable native \
                 handles (jlong) the JVM must free individually. Expose a per-element \
                 accessor instead of returning a `Vec` of handles.",
                elem,
            );
        }
    }
}
