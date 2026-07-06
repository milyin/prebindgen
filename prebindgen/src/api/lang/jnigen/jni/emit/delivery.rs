//! Output-expansion delivery: unfold plans, leaf encoding, and the
//! error-channel (`ze`) defaults.

use super::*;

/// Language-neutral default classification for one plan leaf — what a
/// **binding** error fills its builder-typed `ze` slot with when the domain
/// value never materialized. One classifier feeds both renderers — the
/// native `__ze_defaults` jvalues ([`default_ze_jvalues`]) and the Kotlin
/// guard literals (`ze_default_kotlin` in render.rs) — so the two sides
/// cannot drift.
pub(crate) enum LeafDefault {
    /// Plan-nullable leaf, or an object kind with no constructible default
    /// (data classes, …): JVM `null`.
    Null,
    /// Raw primitive: zero / `false`.
    Prim(JniPrim),
    /// Non-null `String`: `""`.
    Str,
    /// Byte array or `Copy` value blob: an empty array.
    Bytes,
    /// Collection: an empty list.
    List,
}

/// Classify a leaf's binding-error default — see [`LeafDefault`].
pub(crate) fn leaf_default(
    _ext: &JniGen,
    registry: &Registry<KotlinMeta>,
    leaf: &crate::api::core::unfold::UnfoldLeaf,
) -> LeafDefault {
    if leaf.nullable {
        return LeafDefault::Null;
    }
    let e = registry.output_entry(&leaf.out_ty).unwrap_or_else(|| {
        panic!(
            "leaf_default: leaf `{}` has no registered output converter",
            TypeKey::from_type(&leaf.out_ty)
        )
    });
    if let Some(proj) = &e.metadata.projection {
        return match proj.kind {
            // A handle crosses as a raw jlong — `0L` (no handle) is the
            // default; the receiver's `isClosed()`-style zero check applies.
            ProjectionKind::Handle => LeafDefault::Prim(JniPrim::Long),
            ProjectionKind::ValueBlob => LeafDefault::Bytes,
        };
    }
    if let Some(p) = JniPrim::from_wire(&e.destination) {
        return LeafDefault::Prim(p);
    }
    match jni_field_access(&e.destination) {
        Some(("Ljava/lang/String;", _, _)) => LeafDefault::Str,
        Some(("[B", _, _)) => LeafDefault::Bytes,
        _ => match e
            .metadata
            .kotlin_name
            .as_ref()
            .and_then(|k| k.simple_name())
        {
            Some("List" | "MutableList") => LeafDefault::List,
            _ => LeafDefault::Null,
        },
    }
}

/// The native default-`jvalue` expression per error-plan leaf, rendered from
/// [`leaf_default`]. Each expression evaluates inside the lazy
/// `__ze_defaults` closure (`env` in scope); a zeroed union (`l = null`) is
/// 0 / `false` at every primitive slot. Construction failures fall back to
/// null (cold path, OOM-class only).
pub(crate) fn default_ze_jvalues(
    ext: &JniGen,
    registry: &Registry<KotlinMeta>,
    plan: &crate::api::core::unfold::UnfoldPlan,
) -> Vec<TokenStream> {
    let null_jv = quote!(jni::sys::jvalue {
        l: ::std::ptr::null_mut()
    });
    plan.leaves
        .iter()
        .map(|leaf| match leaf_default(ext, registry, leaf) {
            LeafDefault::Null | LeafDefault::Prim(_) => null_jv.clone(),
            LeafDefault::Str => quote! {
                env.new_string("")
                    .map(|__s| jni::sys::jvalue { l: __s.into_raw() })
                    .unwrap_or(#null_jv)
            },
            LeafDefault::Bytes => quote! {
                env.byte_array_from_slice(&[])
                    .map(|__a| jni::sys::jvalue { l: __a.into_raw() })
                    .unwrap_or(#null_jv)
            },
            LeafDefault::List => quote! {
                env.new_object("java/util/ArrayList", "()V", &[])
                    .map(|__o| jni::sys::jvalue { l: __o.into_raw() })
                    .unwrap_or(#null_jv)
            },
        })
        .collect()
}

/// Emit the output-expansion delivery body (output phase) for a function
/// marked `.expand_output()`. The return value (`__out`) is decomposed by the
/// plan's accessor leaves, each encoded into a JVM `Object`, and all delivered
/// to the foreign builder lambda (`__builder`) in a single `invoke` call whose
/// `JObject` result becomes the wrapper's return.
///
/// **Borrow ordering** (the user's zero-copy rationale): the reference
/// (non-identity) accessor leaves are encoded **first** — each leaf's converter
/// performs the single JVM copy (`&str -> jstring`), ending its borrow into
/// `__out` — then the identity/handle leaf is emitted **last**: an owned `T`
/// return is **moved** into the handle (`Box::into_raw(Box::new(__out))`, no
/// clone) once the borrows are gone; a `&T` return is **cloned** via the
/// borrowed-opaque output converter. The builder args are assembled in declared
/// leaf order regardless of encode order.
///
/// Shape handling: [`UnfoldShape::Base`] decomposes the returned value
/// directly; [`UnfoldShape::Optional`] matches `Some(__inner)` ⇒ decompose the
/// inner, `None` ⇒ null result (builder skipped). Leaf wires may be object
/// (JString/JByteArray/JObject — cast via `.into()`) or primitive (boxed to
/// `java.lang.*` via the cached `box_helper_for_wire` runtime helpers).
///
/// [`UnfoldShape::Base`]: crate::api::core::unfold::UnfoldShape::Base
/// [`UnfoldShape::Optional`]: crate::api::core::unfold::UnfoldShape::Optional
pub(crate) fn emit_unfold_delivery(
    ext: &JniGen,
    registry: &Registry<KotlinMeta>,
    plan: &crate::api::core::unfold::UnfoldPlan,
    call_expr: &TokenStream,
    on_err: &TokenStream,
) -> TokenStream {
    use crate::api::core::unfold::UnfoldShape;

    let n = plan.leaves.len();

    // Builder-arg locals, one per leaf in declared order. The builder is a
    // generated typed `<Source>Builder<out R>` fun interface — its `run`
    // method ID is resolved once per process on the interface class
    // ([`CachedIfaceMethod`]); primitives cross as raw typed jvalues.
    let obj_idents: Vec<syn::Ident> = (0..n).map(|i| format_ident!("__obj{}", i)).collect();

    // Return-site error path: route the message to the error sink, then return
    // the wrapper's sentinel. Threads through the shared leaf encoder.
    let fail = |msg: TokenStream| -> TokenStream {
        quote! {
            let __zd = __ze_defaults(&mut env); signal_error(&mut env, &__error_sink, &__SINK_MID, __SINK_FQN, __SINK_DESCR, ::core::option::Option::Some(&#msg), &__zd);
            return #on_err;
        }
    };

    // Encode a value's leaves (`__out`, a `Some`-bound `__inner`, or a Vec
    // `__elem`) into `__obj0…__objN` (shared with the callback trampoline),
    // yielding the per-leaf typed jvalue arg expressions.
    let encode_leaves = |value: &TokenStream| -> (TokenStream, Vec<TokenStream>) {
        encode_plan_leaves(ext, registry, plan, &obj_idents, value, &fail)
    };

    // Cached-interface call statics for the builder / folder `run`.
    let iface_statics = |spec: &IfaceSpec| -> TokenStream {
        let fqn_lit = syn::LitStr::new(&spec.raw_slash_fqn(), Span::call_site());
        let descr_lit = syn::LitStr::new(&spec.descr, Span::call_site());
        quote! {
            #[allow(non_upper_case_globals)]
            static __CB_MID: ::prebindgen::lang::CachedIfaceMethod =
                ::prebindgen::lang::CachedIfaceMethod::new();
            const __CB_FQN: &str = #fqn_lit;
            const __CB_DESCR: &str = #descr_lit;
        }
    };

    // Common builder-invoke (typed `run`, `Object` return = the erased `R`).
    // Used by `Decompose`/`Optional`; its success arm yields the result
    // `JObject`, error arms route to the sink + return the wrapper's sentinel.
    let builder_invoke = |arg_exprs: &[TokenStream]| -> TokenStream {
        quote! {
            match __CB_MID.call_object(
                &mut env, __CB_FQN, "run", __CB_DESCR, &__builder, &[#(#arg_exprs),*],
            ) {
                ::core::result::Result::Ok(__o) => __o,
                ::core::result::Result::Err(__e) => {
                    // Clears any pending JVM exception so the sink call is safe.
                    let _ = env.exception_describe();
                    let __e2 = <__JniErr as ::core::convert::From<String>>::from(__e.to_string());
                    let __zd = __ze_defaults(&mut env); signal_error(&mut env, &__error_sink, &__SINK_MID, __SINK_FQN, __SINK_DESCR, ::core::option::Option::Some(&__e2.to_string()), &__zd);
                    #on_err
                }
            }
        }
    };

    // Decompose a value into leaves then invoke the builder once (`Decompose`/
    // `Optional`).
    let emit_decompose = |value: &TokenStream| -> TokenStream {
        let (leaves, arg_exprs) = encode_leaves(value);
        let invoke = builder_invoke(&arg_exprs);
        quote! { #leaves #invoke }
    };

    // Iterable (fold) delivery — possibly wrapped in ONE `Optional` layer.
    // `Vec<T>` folds the elements through the typed `<Element>Folder<A>.run(acc,
    // …)`, threading `__acc` and returning the final accumulator; per element the
    // fold args are either the element WHOLE (M4) or its decomposed leaves (M5),
    // with `acc` the erased `A` (`Object`). `Option<Vec<T>>` additionally yields a
    // null result for `None` (the fold is skipped).
    let opt_iterable = match &plan.shape {
        UnfoldShape::Iterable(_) => Some(false),
        UnfoldShape::Optional((), inner) if matches!(**inner, UnfoldShape::Iterable(_)) => {
            Some(true)
        }
        _ => None,
    };
    if let Some(optional) = opt_iterable {
        let statics = iface_statics(
            &folder_iface_for_plan(ext, registry, plan)
                .expect("folder interface spec derivable for a resolved plan"),
        );
        let fold_invoke = |arg_exprs: &[TokenStream]| -> TokenStream {
            quote! {
                __acc = match __CB_MID.call_object(
                    &mut env, __CB_FQN, "run", __CB_DESCR, &__fold,
                    &[jni::sys::jvalue { l: __acc.as_raw() }, #(#arg_exprs),*],
                ) {
                    ::core::result::Result::Ok(__o) => __o,
                    ::core::result::Result::Err(__e) => {
                        let _ = env.exception_describe();
                        let __e2 = <__JniErr as ::core::convert::From<String>>::from(__e.to_string());
                        let __zd = __ze_defaults(&mut env); signal_error(&mut env, &__error_sink, &__SINK_MID, __SINK_FQN, __SINK_DESCR, ::core::option::Option::Some(&__e2.to_string()), &__zd);
                        return #on_err;
                    }
                };
            }
        };

        let loop_body = if let Some(element) = plan.element.as_ref() {
            // Whole-element (M4): encode the element via its own converter —
            // a raw typed jvalue for a primitive-wire element, a JObject
            // otherwise (mirrors `leaf_is_prim`; the folder interface
            // declares the matching typed param).
            let out_entry = registry.output_entry(element).unwrap_or_else(|| {
                panic!(
                    "emit_unfold_delivery: Vec element `{}` has no registered output converter",
                    TypeKey::from_type(element)
                )
            });
            let elem_conv = out_entry.function.sig.ident.clone();
            let elem_wire = out_entry.destination.clone();
            // Primitive-wire elements (including an opaque **handle**, whose wire
            // is `jlong`) cross as a raw typed jvalue; object wires (String /
            // value blob) cross as a `JObject`. Keyed purely on the wire shape —
            // a handle's `Some(Handle)` projection still rides its `jlong`, and
            // the folder interface declares the matching `Long` (raw) param.
            let elem_is_prim = matches!(jni_field_access(&elem_wire), Some((_, _, false)));
            let enc = format_ident!("__enc");
            let (bind_obj, arg_expr) = if elem_is_prim {
                let letter = jni_field_access(&elem_wire).unwrap().1;
                (
                    TokenStream::new(),
                    quote!(jni::sys::jvalue { #letter: __enc }),
                )
            } else {
                let cast = cast_wire_to_jobject(&enc, &elem_wire, &fail);
                (
                    quote! { let __obj: jni::objects::JObject = #cast; },
                    quote!(jni::sys::jvalue { l: __obj.as_raw() }),
                )
            };
            let invoke = fold_invoke(&[arg_expr]);
            quote! {
                let __enc = match #elem_conv(&mut env, __elem) {
                    ::core::result::Result::Ok(__w) => __w,
                    ::core::result::Result::Err(__e) => {
                        let __zd = __ze_defaults(&mut env); signal_error(&mut env, &__error_sink, &__SINK_MID, __SINK_FQN, __SINK_DESCR, ::core::option::Option::Some(&__e.to_string()), &__zd);
                        return #on_err;
                    }
                };
                #bind_obj
                #invoke
            }
        } else {
            // Decomposed (M5): encode each element's leaves, fold over them.
            let (leaves, arg_exprs) = encode_leaves(&quote!(__elem));
            let invoke = fold_invoke(&arg_exprs);
            quote! {
                #leaves
                #invoke
            }
        };
        // Fold the elements of `__vec` into `__acc` (`into_iter()` yields the
        // element type exactly as written — owned `T`, or `&T` for a borrow).
        let fold = quote! {
            let mut __acc = __acc;
            for __elem in __vec.into_iter() {
                #loop_body
            }
            __acc
        };
        // `Option<Vec<T>>`: `None` ⇒ null result; `Some(vec)` ⇒ fold. A bare
        // `Vec<T>` folds the returned value directly.
        return if optional {
            quote! {
                #statics
                let __out = #call_expr;
                match __out {
                    ::core::option::Option::Some(__vec) => { #fold }
                    ::core::option::Option::None => #on_err,
                }
            }
        } else {
            quote! {
                #statics
                let __vec = #call_expr;
                #fold
            }
        };
    }

    match &plan.shape {
        UnfoldShape::Base => {
            let decon = plan
                .decon
                .as_ref()
                .expect("record-built plan carries its DeconId");
            let statics = iface_statics(
                &builder_iface_spec(ext, registry, decon)
                    .expect("builder interface spec derivable for a registered declaration"),
            );
            let body = emit_decompose(&quote!(__out));
            quote! {
                #statics
                let __out = #call_expr;
                #body
            }
        }
        UnfoldShape::Optional((), inner) => {
            match **inner {
                UnfoldShape::Base => {}
                _ => panic!(
                    "emit_unfold_delivery: Optional inner must be Base (scalar) or \
                     Iterable (`Option<Vec<T>>`, handled above)"
                ),
            }
            let decon = plan
                .decon
                .as_ref()
                .expect("record-built plan carries its DeconId");
            let statics = iface_statics(
                &builder_iface_spec(ext, registry, decon)
                    .expect("builder interface spec derivable for a registered declaration"),
            );
            // `None` ⇒ null result (builder skipped); `Some` ⇒ decompose inner.
            let body = emit_decompose(&quote!(__inner));
            quote! {
                #statics
                let __out = #call_expr;
                match __out {
                    ::core::option::Option::Some(__inner) => { #body }
                    ::core::option::Option::None => #on_err,
                }
            }
        }
        UnfoldShape::Iterable(_) => {
            unreachable!("Iterable delivery is handled by the `opt_iterable` branch above")
        }
    }
}

/// Cast an encoded wire local to a `JObject` for the erased `invoke`: object
/// wires pass through / `.into()`; primitive wires box to `java.lang.*`.
/// `fail(msg)` — `msg` an expression yielding `String` — produces the
/// diverging on-error statements (sink + sentinel at a return site, `Err` in
/// the trampoline). Returns an expression yielding `JObject`.
pub(crate) fn cast_wire_to_jobject(
    enc: &syn::Ident,
    wire: &syn::Type,
    fail: &dyn Fn(TokenStream) -> TokenStream,
) -> TokenStream {
    if is_jobject_wire(wire) {
        quote!(#enc)
    } else if matches!(jni_field_access(wire), Some((_, _, true))) {
        quote!(#enc.into())
    } else if let Some(helper) = box_helper_for_wire(wire) {
        let on_fail = fail(quote!(__e));
        quote! {
            match ::prebindgen::lang::#helper(&mut env, #enc) {
                ::core::result::Result::Ok(__o) => __o,
                ::core::result::Result::Err(__e) => {
                    #on_fail
                }
            }
        }
    } else {
        panic!(
            "jnigen unfold: leaf has unsupported wire `{}`",
            wire.to_token_stream()
        )
    }
}

/// Reach a leaf's input by folding its accessor `path` over `base`, then hand
/// the reached expression to `body` (which renders the encode and yields
/// `JObject`). Every `Option`-returning nesting step becomes a `match`: its
/// `None` arm short-circuits the whole leaf to `JObject::null()` (the value is
/// absent ⇒ the leaf is null) — any number of `Option` steps on the path nest.
/// With `unwrap_last == false` the final path element composes directly — a
/// non-identity leaf's converter takes the final accessor's **full** return
/// type (`Option` included), so only the steps *before* it are nesting. An
/// identity leaf (`unwrap_last == true`) delivers the reached value itself, so
/// a final `Option` step unwraps too.
#[allow(clippy::too_many_arguments)]
fn reach_leaf(
    source_module: &syn::Path,
    path: &[syn::Ident],
    returns_option: &dyn Fn(&syn::Ident) -> bool,
    base: TokenStream,
    base_is_ref: bool,
    unwrap_last: bool,
    depth: usize,
    body: &dyn Fn(TokenStream) -> TokenStream,
) -> TokenStream {
    let limit = if unwrap_last {
        path.len()
    } else {
        path.len().saturating_sub(1)
    };
    let mut e = if base_is_ref { base } else { quote!(&#base) };
    match (0..limit).find(|&i| returns_option(&path[i])) {
        // No (more) `Option` nesting steps: compose the rest plainly.
        None => {
            for a in path {
                e = quote!(#source_module::#a(#e));
            }
            body(e)
        }
        Some(k) => {
            for a in &path[..k] {
                e = quote!(#source_module::#a(#e));
            }
            let opt_acc = &path[k];
            let nested = format_ident!("__n{}", depth);
            let inner = reach_leaf(
                source_module,
                &path[k + 1..],
                returns_option,
                quote!(#nested),
                true,
                unwrap_last,
                depth + 1,
                body,
            );
            quote! {
                match #source_module::#opt_acc(#e) {
                    ::core::option::Option::Some(#nested) => { #inner }
                    ::core::option::Option::None => jni::objects::JObject::null(),
                }
            }
        }
    }
}

/// Encode a plan's leaves off `value` (`__out`, a `Some`-bound `__inner`, a Vec
/// `__elem`, an owned callback arg, or a domain error `__de`) into the
/// `obj_idents` locals, in declared-leaf order. Reference (non-identity)
/// leaves are encoded first — ending their borrow into the value — and the
/// identity leaf last (move owned / clone `&T`). Each leaf's value is reached
/// by folding its accessor `path` over `value`; every `Option`-returning
/// nesting step on the path wraps the rest in a `match Some/None` (`None` ⇒ a
/// null leaf) — see [`reach_leaf`]. Error arms are produced by `fail` (see
/// [`cast_wire_to_jobject`]). Shared by the return-delivery site
/// ([`emit_unfold_delivery`]), the callback trampoline, and the domain-error
/// arm of fallible externs (whose `fail` falls back to a binding-error
/// `signal_error` with default ze values).
pub(crate) fn encode_plan_leaves(
    ext: &JniGen,
    registry: &Registry<KotlinMeta>,
    plan: &crate::api::core::unfold::UnfoldPlan,
    obj_idents: &[syn::Ident],
    value: &TokenStream,
    fail: &dyn Fn(TokenStream) -> TokenStream,
) -> (TokenStream, Vec<TokenStream>) {
    let source_module = &ext.source_module;
    let by_ref = plan.by_ref;
    let n = plan.leaves.len();

    // Typed `jvalue` argument expression per leaf, in leaf order: a non-null
    // primitive-wire leaf passes its raw primitive (`__objN` IS the jvalue);
    // every other leaf is a `JObject` local whose raw pointer rides the `l`
    // slot. Matches the descriptor [`crate::api::lang::jnigen::jni::iface`]
    // derives for the same leaf (primitive chunk vs object chunk).
    let mut arg_exprs: Vec<TokenStream> = Vec::with_capacity(n);
    for (idx, leaf) in plan.leaves.iter().enumerate() {
        let obj_ident = &obj_idents[idx];
        if leaf_is_prim(registry, leaf) {
            arg_exprs.push(quote!(#obj_ident));
        } else {
            arg_exprs.push(quote!(jni::sys::jvalue { l: #obj_ident.as_raw() }));
        }
    }

    // True when accessor `acc`'s return type is `Option<…>` (a nullable nesting
    // step on a leaf's path).
    let returns_option = |acc: &syn::Ident| -> bool {
        registry.functions.get(acc).is_some_and(|(f, _)| match &f.sig.output {
            syn::ReturnType::Type(_, t) => matches!(
                &**t,
                syn::Type::Path(tp) if tp.path.segments.last().is_some_and(|s| s.ident == "Option")
            ),
            _ => false,
        })
    };

    let mut stmts = TokenStream::new();
    let mut order: Vec<usize> = (0..n).filter(|&i| !plan.leaves[i].identity).collect();
    order.extend((0..n).filter(|&i| plan.leaves[i].identity));

    for idx in order {
        let leaf = &plan.leaves[idx];
        let obj_ident = &obj_idents[idx];
        let out_entry = registry.output_entry(&leaf.out_ty).unwrap_or_else(|| {
            panic!(
                "jnigen unfold: leaf `{}` has no registered output converter",
                TypeKey::from_type(&leaf.out_ty)
            )
        });
        let conv = out_entry.function.sig.ident.clone();
        let conv_fail = fail(quote!(__e.to_string()));

        // Bind `obj_ident` to a JObject-yielding `expr`.
        let bind_obj = |obj_ident: &syn::Ident, expr: TokenStream| -> TokenStream {
            quote! {
                let #obj_ident: jni::objects::JObject = #expr;
            }
        };

        if leaf.identity {
            // Identity leaf: deliver the value itself. Its projection decides
            // how: a `ptr_class` Handle is cloned (`&T`, reached by the path)
            // or — at the root of an owned value — moved into a fresh Box,
            // and crosses as the RAW `jlong` (the receiver constructs the
            // typed class in bytecode — a native `new_object` would cost a
            // descriptor parse + FindClass + GetMethodID + NewObjectA per
            // delivery). A nullable handle (an `Option` nesting step on the
            // path) boxes to `java.lang.Long` / null. A `value_blob` (`Copy`)
            // is delivered by copy via its value-blob converter
            // (→ `JByteArray`); the Kotlin adapter wraps it (Rust can't box a
            // `@JvmInline value class`). The whole path is `Option`-unwrapped
            // (`unwrap_last`): an optional nesting step makes the leaf null
            // when the value is absent.
            let proj = out_entry.metadata.projection.as_ref().unwrap_or_else(|| {
                panic!(
                    "jnigen unfold: identity leaf `{}` has no projection — \
                     `.accessor_record_id()` requires a ptr_class or value_blob type",
                    TypeKey::from_type(&leaf.out_ty)
                )
            });
            match proj.kind {
                ProjectionKind::Handle => {
                    let handle_ident = format_ident!("__h{}", idx);
                    if leaf.path.is_empty() && !by_ref {
                        // Owned root, non-nullable by construction (nullable
                        // comes from path nesting): move into a Box, raw jlong.
                        stmts.extend(quote! {
                            let #obj_ident: jni::sys::jvalue = jni::sys::jvalue {
                                j: std::boxed::Box::into_raw(std::boxed::Box::new(#value))
                                    as jni::sys::jlong,
                            };
                        });
                    } else if !leaf.nullable {
                        // Reached non-null handle: clone via the converter,
                        // raw jlong (no Option steps on the path).
                        let expr = reach_leaf(
                            source_module,
                            &leaf.path,
                            &returns_option,
                            value.clone(),
                            by_ref,
                            true,
                            0,
                            &|reached| {
                                quote! {{
                                    let #handle_ident: jni::sys::jlong = match #conv(&mut env, #reached) {
                                        ::core::result::Result::Ok(__w) => __w,
                                        ::core::result::Result::Err(__e) => {
                                            #conv_fail
                                        }
                                    };
                                    jni::sys::jvalue { j: #handle_ident }
                                }}
                            },
                        );
                        stmts.extend(quote! {
                            let #obj_ident: jni::sys::jvalue = #expr;
                        });
                    } else {
                        // Nullable handle (Option nesting step): boxed
                        // `java.lang.Long` when present (cached valueOf),
                        // JVM null when absent — matching the `Long?` param.
                        let box_fail = fail(quote!(__e.to_string()));
                        let expr = reach_leaf(
                            source_module,
                            &leaf.path,
                            &returns_option,
                            value.clone(),
                            by_ref,
                            true,
                            0,
                            &|reached| {
                                quote! {{
                                    let #handle_ident: jni::sys::jlong = match #conv(&mut env, #reached) {
                                        ::core::result::Result::Ok(__w) => __w,
                                        ::core::result::Result::Err(__e) => {
                                            #conv_fail
                                        }
                                    };
                                    match ::prebindgen::lang::box_jlong(&mut env, #handle_ident) {
                                        ::core::result::Result::Ok(__o) => __o,
                                        ::core::result::Result::Err(__e) => {
                                            #box_fail
                                        }
                                    }
                                }}
                            },
                        );
                        stmts.extend(bind_obj(obj_ident, expr));
                    }
                }
                ProjectionKind::ValueBlob => {
                    // The value_blob converter takes the value owned (`Copy`).
                    // Owned at the root; reached-by-`&` elsewhere ⇒ deref-copy.
                    let wire = out_entry.destination.clone();
                    let enc_ident = format_ident!("__enc{}", idx);
                    let cast = cast_wire_to_jobject(&enc_ident, &wire, fail);
                    if leaf.path.is_empty() && !by_ref {
                        stmts.extend(bind_obj(
                            obj_ident,
                            quote! {{
                                let #enc_ident = match #conv(&mut env, #value) {
                                    ::core::result::Result::Ok(__w) => __w,
                                    ::core::result::Result::Err(__e) => {
                                        #conv_fail
                                    }
                                };
                                #cast
                            }},
                        ));
                    } else {
                        let expr = reach_leaf(
                            source_module,
                            &leaf.path,
                            &returns_option,
                            value.clone(),
                            by_ref,
                            true,
                            0,
                            &|reached| {
                                quote! {{
                                    let #enc_ident = match #conv(&mut env, *#reached) {
                                        ::core::result::Result::Ok(__w) => __w,
                                        ::core::result::Result::Err(__e) => {
                                            #conv_fail
                                        }
                                    };
                                    #cast
                                }}
                            },
                        );
                        stmts.extend(bind_obj(obj_ident, expr));
                    }
                }
            }
            continue;
        }

        // Leaf reach. An `Accessor` leaf walks its accessor-fn path — unwrapping
        // every `Option` nesting step (`None` ⇒ a null leaf). A `Field` leaf
        // (synthesized `data_class`) reaches a struct field and clones it
        // (`value.a.b.clone()`); the converter (`Option<Box<String>>` → nullable
        // String, …) carries any nullability, so there is no path `Option` to
        // unwrap. `reach(body)` dispatches on the source and feeds the reached
        // Rust expression to `body`.
        use crate::api::core::unfold::LeafSource;
        let reach = |body: &dyn Fn(TokenStream) -> TokenStream| -> TokenStream {
            match leaf.source {
                LeafSource::Accessor => reach_leaf(
                    source_module,
                    &leaf.path,
                    &returns_option,
                    value.clone(),
                    by_ref,
                    false,
                    0,
                    body,
                ),
                LeafSource::Field => {
                    let segs = &leaf.path;
                    body(quote!(#value #(.#segs)*.clone()))
                }
            }
        };

        // A non-null primitive-wire leaf delivers its raw primitive as a typed
        // `jvalue` — no boxing, no JNI call at all (the typed `run` descriptor
        // declares the primitive). Everything else (object wires, and nullable
        // leaves whose `None` arm must yield a JVM null) encodes the reached
        // value with the leaf's output converter and casts to JObject.
        let wire = out_entry.destination.clone();
        let enc_ident = format_ident!("__enc{}", idx);
        if leaf_is_prim(registry, leaf) {
            let letter = jni_field_access(&wire)
                .expect("leaf_is_prim guarantees a primitive wire")
                .1;
            let expr = reach(&|reached| {
                quote! {{
                    let #enc_ident = match #conv(&mut env, #reached) {
                        ::core::result::Result::Ok(__w) => __w,
                        ::core::result::Result::Err(__e) => {
                            #conv_fail
                        }
                    };
                    jni::sys::jvalue { #letter: #enc_ident }
                }}
            });
            stmts.extend(quote! {
                let #obj_ident: jni::sys::jvalue = #expr;
            });
            continue;
        }
        let cast = cast_wire_to_jobject(&enc_ident, &wire, fail);
        let expr = reach(&|reached| {
            quote! {{
                let #enc_ident = match #conv(&mut env, #reached) {
                    ::core::result::Result::Ok(__w) => __w,
                    ::core::result::Result::Err(__e) => {
                        #conv_fail
                    }
                };
                #cast
            }}
        });
        stmts.extend(bind_obj(obj_ident, expr));
    }
    (stmts, arg_exprs)
}

/// True when a plan leaf crosses the typed `run` as a **raw primitive**
/// `jvalue`: non-nullable, no projection (not a handle / value-blob), and a
/// primitive JNI wire. Must agree with the descriptor chunk
/// [`crate::api::lang::jnigen::jni::iface`] derives for the same leaf — a
/// nullable primitive boxes (object chunk), object wires pass as objects.
pub(crate) fn leaf_is_prim(
    registry: &Registry<KotlinMeta>,
    leaf: &crate::api::core::unfold::UnfoldLeaf,
) -> bool {
    if leaf.nullable {
        return false;
    }
    let Some(entry) = registry.output_entry(&leaf.out_ty) else {
        return false;
    };
    // No projection (plain primitive/enum wire) — or an opaque HANDLE, whose
    // converter's wire is the raw `jlong` the typed `run` declares as `Long`
    // (`J`): the receiver constructs the typed class in bytecode. A nullable
    // handle boxes to `java.lang.Long` instead (object chunk). Value blobs
    // stay objects (`[B`).
    let proj_ok = match &entry.metadata.projection {
        None => true,
        Some(p) => p.kind == ProjectionKind::Handle,
    };
    proj_ok && matches!(jni_field_access(&entry.destination), Some((_, _, false)))
}
