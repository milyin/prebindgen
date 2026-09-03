//! `impl Fn(args)` inputs: the native trampoline calling the typed
//! Kotlin `run`.

use prebindgen_registry::{chain::Chain as _, Conversions};

use super::*;

#[derive(Clone)]
enum JInvokePart {
    Fold {
        delivery: FrozenDelivery,
        fold_obj: syn::Ident,
        fold_id: syn::Ident,
        element_frame: syn::LitInt,
    },
    Decomposed {
        delivery: FrozenDelivery,
        optional: bool,
    },
    Whole {
        /// The argument's own output ABI, shared with the site that names it
        /// rather than copied — the same property `FrozenDelivery` holds for
        /// the two decomposing arms (#622 review).
        abi: std::rc::Rc<crate::jni::compile::OutValueAbi>,
        wire: syn::Type,
        primitive: bool,
    },
}

impl JInvokePart {
    /// The converters this argument crosses through.
    fn calls(&self, out: &mut Vec<prebindgen_registry::write::ArtifactKey>) {
        match self {
            Self::Fold { delivery, .. } | Self::Decomposed { delivery, .. } => delivery.calls(out),
            Self::Whole { abi, .. } => abi.pipeline.calls(out),
        }
    }
}

impl prebindgen_registry::chain::InvokePart for JInvokePart {
    fn render(
        &self,
        value: &syn::Ident,
        index: usize,
        emit: &prebindgen_registry::RustWriter,
    ) -> prebindgen_registry::chain::RenderedInvokePart {
        assert_eq!(value, &callback_argument_name(index));
        let fail = |msg: TokenStream| -> TokenStream {
            quote! {
                return ::core::result::Result::Err(
                    <__JniErr as ::core::convert::From<String>>::from(#msg));
            }
        };
        let (prepare, arguments) = match self {
            Self::Fold {
                delivery,
                fold_obj,
                fold_id,
                element_frame,
            } => {
                let obj_idents: Vec<syn::Ident> = (0..delivery.wire_count())
                    .map(|part| format_ident!("__cbfold{}_obj{}", index, part))
                    .collect();
                let (leaf_stmts, leaf_args, _) = encode_plan_leaves(
                    delivery,
                    delivery.delivered(),
                    &obj_idents,
                    &quote!(__cb_elem),
                    &fail,
                    emit,
                );
                let acc = format_ident!("__fold{}_acc", index);
                (
                    quote! {
                        let #acc: jni::objects::JObject = env
                            .new_object("java/util/ArrayList", "()V", &[])
                            .map_err(|e| <__JniErr as ::core::convert::From<String>>::from(format!("fold: new ArrayList: {}", e)))?;
                        for __cb_elem in #value.iter() {
                            env.push_local_frame(#element_frame)
                                .map_err(|e| <__JniErr as ::core::convert::From<String>>::from(format!("fold: push frame: {}", e)))?;
                            let __fold_res = (|| -> ::core::result::Result<(), __JniErr> {
                                #leaf_stmts
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
                    },
                    vec![quote!(jni::sys::jvalue { l: #acc.as_raw() })],
                )
            }
            Self::Decomposed { delivery, optional } => {
                let obj_idents: Vec<syn::Ident> = (0..delivery.wire_count())
                    .map(|part| format_ident!("__cb{}_obj{}", index, part))
                    .collect();
                let (stmts, mut arguments, present) = encode_plan_leaves(
                    delivery,
                    delivery.delivered(),
                    &obj_idents,
                    &quote!(#value),
                    &fail,
                    emit,
                );
                if *optional {
                    let present = present.expect("a frozen optional callback delivery has a gate");
                    arguments.insert(0, quote!(jni::sys::jvalue { z: #present }));
                }
                (stmts, arguments)
            }
            Self::Whole {
                abi,
                wire,
                primitive,
            } => {
                let (pipeline, projection) = (&abi.pipeline, &abi.projection);
                let enc = format_ident!("__cb{}_enc", index);
                let call = pipeline.invoke_output(quote!(#value), emit);
                if projection
                    .as_ref()
                    .is_some_and(|projection| projection.kind == ProjectionKind::Handle)
                {
                    (
                        quote!(let #enc = #call?;),
                        vec![quote!(jni::sys::jvalue { j: #enc })],
                    )
                } else if *primitive {
                    let letter = jni_field_access(wire)
                        .expect("a primitive callback wire has a jvalue member")
                        .1;
                    (
                        quote!(let #enc = #call?;),
                        vec![quote!(jni::sys::jvalue { #letter: #enc })],
                    )
                } else {
                    let object = format_ident!("__cb{}_obj", index);
                    let cast = cast_wire_to_jobject(&enc, wire, &fail);
                    (
                        quote! {
                            let #enc = #call?;
                            let #object: jni::objects::JObject = #cast;
                        },
                        vec![quote!(jni::sys::jvalue { l: #object.as_raw() })],
                    )
                }
            }
        };
        prebindgen_registry::chain::RenderedInvokePart {
            prepare,
            arguments,
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

fn callback_argument_name(index: usize) -> syn::Ident {
    format_ident!("__cb_arg{}", index)
}

/// Freeze one callback argument's deconstructed delivery before the retained
/// Invoke plan is rendered. A composable recipe contributes its exact child
/// wires and chain; an irregular registry-owned unfold walk freezes each
/// already-resolved leaf crossing once at this resolution seam.
fn freeze_callback_delivery(
    ext: &Declarations,
    plan: &crate::unfold::UnfoldPlan,
    fragment: &crate::jni::compile::JFrag,
) -> Option<(
    Vec<crate::jni::compile::OutWire>,
    Option<crate::jni::compile::ComposedChain>,
)> {
    let expected = crate::jni::compile::OutWire::from_leaves(&plan.leaves);
    let composed = fragment.out_wires.clone().filter(|wires| {
        wires.len() == expected.len()
            && wires
                .iter()
                .zip(&expected)
                .all(|(left, right)| left.same_delivery(right))
    });
    let wires = match composed {
        Some(wires) => wires,
        None => expected
            .into_iter()
            .map(|mut wire| {
                wire.abi = Some(if wire.is_tag() {
                    crate::jni::compile::OutAbi::Tag
                } else if matches!(wire.from, crate::jni::compile::OutFrom::Present) {
                    crate::jni::compile::OutAbi::Present
                } else {
                    ext.out_frag(&wire.out_ty)?.output_abi()
                });
                if wire.identity
                    && matches!(
                        wire.abi,
                        Some(crate::jni::compile::OutAbi::Value(ref value))
                            if value.projection.is_none()
                    )
                {
                    return None;
                }
                Some(wire)
            })
            .collect::<Option<Vec<_>>>()?,
    };
    wires
        .iter()
        .for_each(crate::jni::compile::OutWire::activate);
    let chain = fragment.composed_chain();
    if let Some(chain) = &chain {
        chain.activate();
    }
    Some((wires, chain))
}

/// A registry-composed callback retained until the final Rust writer runs.
#[derive(Clone)]
pub(crate) struct JInvokePlan {
    operation: prebindgen_registry::OperationId,
    chain:
        prebindgen_registry::chain::Invoke<crate::jni::chain::JSource, JInvokeBridge, JInvokePart>,
}

impl JInvokePlan {
    pub(crate) fn operation_id(&self) -> &prebindgen_registry::OperationId {
        &self.operation
    }

    /// The ABI one delivered argument occupies, as the trampoline finalized
    /// it. What a `Role::CallbackArg` site names: the same allocation the
    /// rendered part holds, so the site states the delivery rather than a
    /// second derivation of it.
    pub(crate) fn arg_abi(&self, index: usize) -> Option<crate::jni::compile::JAbiLeaves> {
        Some(match self.chain.parts.get(index)? {
            JInvokePart::Fold { delivery, .. } | JInvokePart::Decomposed { delivery, .. } => {
                crate::jni::compile::JAbiLeaves::Decomposed(delivery.wires())
            }
            JInvokePart::Whole { abi, .. } => crate::jni::compile::JAbiLeaves::Invoked(abi.clone()),
        })
    }

    /// The converters this callback's Invoke helper calls, argument by
    /// argument.
    pub(crate) fn calls(&self, out: &mut Vec<prebindgen_registry::write::ArtifactKey>) {
        for part in &self.chain.parts {
            part.calls(out);
        }
    }

    pub(crate) fn render(&self, emit: &prebindgen_registry::RustWriter) -> syn::ItemFn {
        let rendered = self.chain.render(emit);
        let name = emit.operation_ident("jni", &self.operation);
        let source = &rendered.source;
        let body = &rendered.body;
        let gen_allow = crate::jni::trait_impl::generated_converter_attr();
        syn::parse_quote!(
            #gen_allow
            pub(crate) unsafe fn #name<'env, 'v>(
                env: &mut jni::JNIEnv<'env>,
                v: &jni::objects::JObject<'v>,
            ) -> ::core::result::Result<#source, __JniErr> {
                Ok(#body)
            }
        )
    }
}

impl prebindgen_registry::chain::InvokeBridge for JInvokeBridge {
    fn intermediate(&self) -> syn::Type {
        syn::parse_quote!(jni::objects::JObject)
    }

    fn argument_name(&self, index: usize) -> syn::Ident {
        callback_argument_name(index)
    }

    fn capture(&self, value: TokenStream, closure: TokenStream) -> TokenStream {
        let name = &self.name;
        let descriptor = &self.descriptor;
        let fold_setups = &self.fold_setups;
        // Resolve the typed callback interface's `run` method once, while the
        // trampoline is created. `JNIEnv::call_method` reparses the descriptor
        // and resolves the method on every call; that measured at roughly 33%
        // of subscriber hot-path delivery time. `JMethodID` is Copy + Send +
        // Sync and remains valid because the global ref pins the callback and
        // therefore its class for the closure's lifetime.
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
        // SAFETY: `__invoke_id` was resolved on this callback object's exact
        // class with the exact descriptor represented by `arguments`; the
        // global ref keeps that class loaded. `run` returns void, primitives
        // occupy their matching raw jvalue fields, and exception handling is
        // the same checked JNI path used by `call_method`.
        //
        // A plan-less owned-handle argument's per-invocation Box is closed by
        // Kotlin's `asRaw` proxy in `finally { close() }` (a no-op if taken), so
        // the Rust invocation has no matching post-call close.
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
        // Callbacks may run on daemon-attached Zenoh receive threads that never
        // return through a JNI stack frame. Give every invocation its own local
        // frame so encoded leaves, handle wrappers and call temporaries cannot
        // accumulate until `OutOfMemoryError`; pop it unconditionally after the
        // inner Result, including every early `?`/error path.
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
    operation: prebindgen_registry::OperationId,
    source: &prebindgen_registry::flat::TypeRef,
    args: &[prebindgen_registry::flat::TypeRef],
    registry: &(impl Conversions + ?Sized),
    arg_fragments: &[&crate::jni::compile::JFrag],
) -> Option<(syn::Type, JInvokePlan)> {
    // Human-readable tag for attach/log messages.
    let name = format!(
        "Fn({})",
        args.iter()
            .map(|t| t.key().to_string())
            .collect::<Vec<_>>()
            .join(", ")
    );
    let name_lit = syn::LitStr::new(&name, Span::call_site());

    let mut parts = Vec::with_capacity(args.len());
    let mut total: usize = 0;
    // One-time setup statements (folder singleton + method id for an
    // `&[data_class]` fold arg), spliced before the `Box::new` so the move
    // closure captures them.
    let mut fold_setups: Vec<TokenStream> = Vec::new();
    for (i, arg_ty) in args.iter().enumerate() {
        // `&[data_class]` fold arg: instead of building the whole `List` on the
        // Rust side, allocate an empty `ArrayList` and fold each element's raw
        // leaves through the hoisted `__<Folder>Holder.instance` (Kotlin does
        // `fromParts` + `add`), then deliver the assembled list whole to the
        // user callback's `run(List<T>)`. Reuses the OUTPUT fold's folder
        // interface + appender singleton, driven from the trampoline.
        if let Some(plan) = ext
            .unfolded()
            .callback_arg_plans
            .get(&arg_ty.key())
            .filter(|p| super::render::is_iterable_fold(&p.shape))
        {
            let fragment = *arg_fragments.get(i)?;
            let (wires, chain) = freeze_callback_delivery(ext, plan, fragment)?;
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
            let elem_frame = std::cmp::max(16, 2 * plan.leaves.len() + 6);
            parts.push(JInvokePart::Fold {
                delivery: FrozenDelivery::new(ext, registry, plan, std::rc::Rc::new(wires), chain),
                fold_obj,
                fold_id,
                element_frame: syn::LitInt::new(&elem_frame.to_string(), Span::call_site()),
            });
            total += 1;
            continue;
        }

        // Decomposed arg: deliver the leaves of its type-level canonical
        // output, exactly like a return delivery.
        if let Some(plan) = effective_callback_plan(ext, arg_ty) {
            let fragment = *arg_fragments.get(i)?;
            let (wires, chain) = freeze_callback_delivery(ext, plan, fragment)?;
            let optional = plan.is_optional_base();
            if optional
                && !matches!(
                    chain.as_ref().map(|chain| &chain.layout),
                    Some(crate::jni::compile::JLayout::Optional(inner))
                        if inner.leaf_count() == plan.leaves.len()
                )
            {
                return None;
            }
            total += plan.leaves.len() + usize::from(optional);
            parts.push(JInvokePart::Decomposed {
                delivery: FrozenDelivery::new(ext, registry, plan, std::rc::Rc::new(wires), chain),
                optional,
            });
            continue;
        }

        // Whole-value delivery consumes the exact deconstruct fragment the
        // registry's Invoke recipe compiled for this argument. Its retained
        // pipeline already decides move versus borrow cloning, stage order,
        // projection, and the final JNI wire; rendering does not look the type
        // up again or reconstruct any of those decisions.
        let fragment = *arg_fragments.get(i)?;
        let arg_abi = crate::jni::compile::output_abi_of(&fragment.freeze())?;
        arg_abi.activate();
        let crate::jni::compile::OutAbi::Value(arg_value) = arg_abi else {
            unreachable!("a whole callback argument is not a synthesized selector")
        };
        let arg_value = std::rc::Rc::new(*arg_value);
        let arg_wire = arg_value.pipeline.wire().clone();
        let arg_is_prim = arg_value
            .projection
            .as_ref()
            .is_none_or(|p| p.kind == ProjectionKind::Unsigned64)
            && arg_ty.optional_inner().is_none()
            && matches!(jni_field_access(&arg_wire), Some((_, _, false)));
        parts.push(JInvokePart::Whole {
            abi: arg_value,
            wire: arg_wire,
            primitive: arg_is_prim,
        });
        total += 1;
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

    let chain = prebindgen_registry::chain::Invoke {
        source: source.clone(),
        arguments: args.to_vec(),
        source_policy: crate::jni::chain::JSource {
            wrappers: Vec::new(),
        },
        bridge: JInvokeBridge {
            name: name_lit,
            descriptor: descr_lit,
            frame_capacity: frame_cap_lit,
            fold_setups,
        },
        parts,
    };
    let intermediate: syn::Type = syn::parse_quote!(jni::objects::JObject);
    let rust_plan = JInvokePlan { operation, chain };

    // The wire type for an `impl Fn(args)` parameter is JObject (the erased
    // Kotlin lambda). The converter returns Box<dyn Fn(args) + Send + Sync>,
    // which coerces to the source's impl-trait param type.
    Some((intermediate, rust_plan))
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
