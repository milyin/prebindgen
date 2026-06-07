//! JNI `extern "C"` wrapper and converter-body emission (free fns).
//!
//! Carved from the former monolithic JNI module; shares the `jni`
//! namespace via `use super::*`.

use super::*;


// ──────────────────────────────────────────────────────────────────────
// Function-wrapper emission (JNI extern "C")
// ──────────────────────────────────────────────────────────────────────

pub(crate) fn emit_jni_function_wrapper(
    ext: &JniGen,
    f: &syn::ItemFn,
    registry: &Registry<KotlinMeta>,
) -> TokenStream {
    let original_ident = &f.sig.ident;
    let wrapper_ident = mangle_jni_name(ext, original_ident);
    let source_module = &ext.source_module;

    let mut wire_params: Vec<TokenStream> = Vec::new();
    // Each entry is a per-input decode statement. Fallible decodes are
    // `match`-arms that, on `Err`, call `signal_error(&mut env,
    // &__error_sink, &__e)` (invoking the caller's Kotlin sink instead of
    // throwing a JVM exception) and `return <sentinel>;`.
    let mut prelude: Vec<TokenStream> = Vec::new();
    let mut call_args: Vec<TokenStream> = Vec::new();

    // Output is resolved first so the per-input `match`-arms can splice
    // the function's sentinel into their early-`return` path.
    let return_ty: syn::Type = match &f.sig.output {
        syn::ReturnType::Default => syn::parse_quote!(()),
        syn::ReturnType::Type(_, ty) => (**ty).clone(),
    };
    let output_entry = registry.output_entry(&return_ty).unwrap_or_else(|| {
        panic!(
            "JniGen::on_function: return type `{}` of `{}` has no registered output \
             converter — register one via `JniGen::output_wrapper(pat, |…| Some((ty, exc, body)))` \
             (exc = `None` for non-throwing, `Some(parse_quote!(<full path>))` \
              to bind a domain exception)",
            TypeKey::from_type(&return_ty),
            original_ident,
        )
    });
    let wire_return_ty = output_entry.destination.clone();
    let conv_out = output_entry.function.sig.ident.clone();
    let wire_return_lt = annotate_jobject_with_lifetime(&wire_return_ty, "a");
    let wire_return = wire_return_lt.to_token_stream();
    let on_err: TokenStream = sentinel_for_wire(&wire_return_ty);

    // Input parameters: look up converter for the param type AS WRITTEN.
    // No strip — a `&T` param looks up `&T`'s entry (which the `& _`
    // rank-1 handler resolved by sharing `T`'s function). Call site adds
    // `&decoded` only for `&T`-shaped originals; that's a Rust call-
    // convention concern, not a converter concern.
    for input in &f.sig.inputs {
        let syn::FnArg::Typed(pt) = input else {
            continue;
        };
        let syn::Pat::Ident(pat_id) = &*pt.pat else {
            continue;
        };
        let arg_ident = &pat_id.ident;
        let arg_ty = &*pt.ty;

        // Constructor-expansion: this parameter's wire form is the fold plan's
        // flattened leaves. Decode each leaf with its own converter, run the
        // (pure-Rust) fold to build the value, then pass it to the call.
        if let Some(plan) = registry
            .expansion_plans
            .get(&(original_ident.clone(), arg_ident.clone()))
        {
            let (wp, pre, call_arg) =
                emit_expanded_param(ext, registry, plan, arg_ident, &on_err);
            wire_params.extend(wp);
            prelude.extend(pre);
            call_args.push(call_arg);
            continue;
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
            let (decode, call_arg) = render_flat_input_decode(&plan, arg_ident, &on_err);
            prelude.push(decode);
            call_args.push(call_arg);
            continue;
        }

        let wire = &entry.destination;
        let conv = entry.function.sig.ident.clone();
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
            call_args.push(quote!(#arg_ident));
            continue;
        }

        let wire_with_lifetime = annotate_jobject_with_lifetime(wire, "a");
        wire_params.push(quote!(#wire_ident: #wire_with_lifetime));
        // Input wrapper takes wires by ref except for raw pointers. The
        // converter returns `Result<T, __JniErr>`; on `Err` we throw via
        // this input's own throw fn and bail with the function sentinel.
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
                        signal_error(&mut env, &__error_sink, &__e);
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
                        signal_error(&mut env, &__error_sink, &__e);
                        return #on_err;
                    }
                };
            ));
            let mut prev = stage0_ident;
            // pre_stages[0] is closest to rust → iterated last; walk
            // back from the function-adjacent end.
            let n = entry.pre_stages.len();
            for (idx, stage) in entry.pre_stages.iter().enumerate().rev() {
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
                            signal_error(&mut env, &__error_sink, &__e);
                            return #on_err;
                        }
                    };
                ));
                prev = out_ident;
            }
        }
        match arg_ty {
            syn::Type::Reference(r) if r.mutability.is_some() => {
                call_args.push(quote!(&mut #arg_ident));
            }
            syn::Type::Reference(_) => {
                call_args.push(quote!(&#arg_ident));
            }
            // `Option<&T>` / `Option<&mut T>` for opaque inner: the input
            // converter produced `Option<OwnedObject<T>>` (see rank-1
            // handler above). `.as_deref()` / `.as_deref_mut()` coerces
            // back to `Option<&T>` / `Option<&mut T>` via OwnedObject's
            // Deref / DerefMut impls.
            _ if matches!(option_inner_ref_mutability(arg_ty), Some(false)) => {
                call_args.push(quote!(#arg_ident.as_deref()));
            }
            _ if matches!(option_inner_ref_mutability(arg_ty), Some(true)) => {
                call_args.push(quote!(#arg_ident.as_deref_mut()));
            }
            _ => {
                call_args.push(quote!(#arg_ident));
            }
        }
    }

    let call_expr = quote!(#source_module::#original_ident(#(#call_args),*));

    // Output phase. Every output converter now returns
    // `Result<wire, <err_type>>` — the bare-wire shape is gone.
    // Unwrap and dispatch to the converter's `throws_action`
    // (framework `throw_JniBindingError` for plain wrappers, a domain
    // throw fn for throws-marked wrappers).
    //
    // Pre_stages run in forward order BEFORE the wire-facing function:
    // rust → pre_stages[0] → … → pre_stages[N-1] → function → wire. Each
    // stage's `Err` arm routes to the per-call `signal_error` sink.
    let mut output_phase: TokenStream = quote! { let __out = #call_expr; };
    let mut prev_out: TokenStream = quote!(__out);
    for (i, stage) in output_entry.pre_stages.iter().enumerate() {
        let stage_fn = &stage.function.sig.ident;
        let next_ident = format_ident!("__out_s{}", i);
        output_phase.extend(quote! {
            let #next_ident = match #stage_fn(&mut env, #prev_out) {
                ::core::result::Result::Ok(__v) => __v,
                ::core::result::Result::Err(__e) => {
                    signal_error(&mut env, &__error_sink, &__e);
                    return #on_err;
                }
            };
        });
        prev_out = quote!(#next_ident);
    }
    output_phase.extend(quote! {
        match #conv_out(&mut env, #prev_out) {
            ::core::result::Result::Ok(__w) => __w,
            ::core::result::Result::Err(__e) => {
                signal_error(&mut env, &__error_sink, &__e);
                #on_err
            }
        }
    });

    // The trailing error-sink param: a Kotlin `ErrorSink` instance (JObject
    // wire). Declared last so the wire param order matches the Kotlin
    // `external fun` (which appends `errorSink: Any`).
    quote! {
        #[no_mangle]
        #[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
        pub unsafe extern "C" fn #wrapper_ident<'a>(
            mut env: jni::JNIEnv<'a>,
            _class: jni::objects::JClass<'a>,
            #(#wire_params,)*
            __error_sink: jni::objects::JObject<'a>,
        ) -> #wire_return {
            #(#prelude)*
            #output_phase
        }
    }
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
    let source_module = &ext.source_module;
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
                        signal_error(&mut env, &__error_sink, &__e);
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
                        signal_error(&mut env, &__error_sink, &__e);
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
                            signal_error(&mut env, &__error_sink, &__e);
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
    let qualify = |id: &syn::Ident| -> syn::Path { syn::parse_quote!(#source_module::#id) };
    let fold_expr = crate::api::core::expand::emit_fold(plan, &leaf_locals, &qualify);
    let folded = format_ident!("__folded_{}", orig_param);
    prelude.push(quote!(
        let #folded = match #fold_expr {
            ::core::result::Result::Ok(__v) => __v,
            ::core::result::Result::Err(__e) => {
                let __je = <__JniErr as ::core::convert::From<::std::string::String>>::from(__e);
                signal_error(&mut env, &__error_sink, &__je);
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

/// Last-segment ident of a `TypeKey` — e.g. `"Publisher<'static>"` →
/// `"Publisher"`, `"AdvancedSubscriber<()>"` → `"AdvancedSubscriber"`. Used by
/// the structured builders ([`JniGen::ptr_class`],
/// [`JniGen::data_class`]) to derive a default Kotlin class name from
/// the Rust type-key. Panics for non-path types (e.g. closures, references) —
/// the per-kind `kotlin_*_name_mangle` closures see only path-shaped
/// shorts. For verbatim Kotlin expressions on non-path types, chain
/// [`JniGen::with_kotlin_type`] after the structured builder.
pub(crate) fn rust_short_name(key: &TypeKey) -> String {
    rust_short_name_opt(key).unwrap_or_else(|| {
        panic!(
            "rust_short_name: cannot derive Kotlin name from type-key `{}` — \
             only path-shaped types are supported here; use \
             `with_kotlin_type(\"<verbatim>\")` to set the name explicitly",
            key.as_str()
        )
    })
}

/// Fallible variant of [`rust_short_name`] — returns `None` for
/// non-path types instead of panicking. Used by
/// [`JniGen::note_wrapper_registration`] which is called for rank-0
/// wrapper patterns including non-path shapes like `()` where there
/// is no Kotlin short name to derive.
pub(crate) fn rust_short_name_opt(key: &TypeKey) -> Option<String> {
    let ty = key.to_type();
    if let syn::Type::Path(tp) = &ty {
        if let Some(last) = tp.path.segments.last() {
            return Some(last.ident.to_string());
        }
    }
    None
}


/// `VisitMut` that prefixes every bare single-segment `Type::Path` whose
/// ident lives in `source_names` with `source_module`. Walks the full
/// AST — function signatures, generic args, type ascriptions, casts,
/// turbofish — so any emitted item passes through one universal pass
/// instead of each emit site having to remember to qualify.
pub(crate) struct QualifyEmittedTypes<'a> {
    pub(crate) source_module: &'a syn::Path,
    pub(crate) source_names: &'a std::collections::HashSet<String>,
}

impl<'a> syn::visit_mut::VisitMut for QualifyEmittedTypes<'a> {
    fn visit_type_path_mut(&mut self, tp: &mut syn::TypePath) {
        if tp.qself.is_none() && tp.path.leading_colon.is_none() && tp.path.segments.len() == 1 {
            let ident = tp.path.segments[0].ident.to_string();
            if self.source_names.contains(&ident) {
                let mut qualified = self.source_module.clone();
                qualified.segments.push(tp.path.segments[0].clone());
                tp.path = qualified;
            }
        }
        syn::visit_mut::visit_type_path_mut(self, tp);
    }
}

pub(crate) fn mangle_jni_name(ext: &JniGen, ident: &syn::Ident) -> syn::Ident {
    let camel = snake_to_camel(&ident.to_string());
    let mangled = ext.mangle_fun(&camel);
    let mut name = ext.jni_class_path.clone();
    name.push('_');
    name.push_str(&mangled);
    syn::Ident::new(&name, Span::call_site())
}

/// Sentinel value to return through the wrapper signature when the inner
/// closure errors. Must compile against any wire type we emit.
pub(crate) fn sentinel_for_wire(wire: &syn::Type) -> TokenStream {
    // Unit wire (void-returning wrappers): the value *is* the sentinel.
    if let syn::Type::Tuple(t) = wire {
        if t.elems.is_empty() {
            return quote!(());
        }
    }
    if let syn::Type::Path(tp) = wire {
        if let Some(last) = tp.path.segments.last() {
            let name = last.ident.to_string();
            return match name.as_str() {
                "jboolean" | "jbyte" | "jchar" | "jshort" | "jint" | "jlong" => quote!(0 as #wire),
                "jfloat" | "jdouble" => quote!(0.0 as #wire),
                "JObject" | "JString" | "JByteArray" | "JClass" => {
                    quote!(jni::objects::JObject::null().into())
                }
                _ => quote!(unsafe { std::mem::zeroed::<#wire>() }),
            };
        }
    }
    if matches!(wire, syn::Type::Ptr(_)) {
        return quote!(std::ptr::null());
    }
    quote!(unsafe { std::mem::zeroed::<#wire>() })
}

// ──────────────────────────────────────────────────────────────────────
// Primitive bodies
// ──────────────────────────────────────────────────────────────────────

pub(crate) fn primitive_input(ty: &syn::Type) -> Option<(syn::Type, syn::Expr)> {
    let key = TypeKey::from_type(ty).as_str().to_string();
    // Bodies receive `v: &<wire>`; primitives are Copy so `*v` works.
    Some(match key.as_str() {
        "bool" => (
            syn::parse_quote!(jni::sys::jboolean),
            syn::parse_quote!(*v != 0),
        ),
        "i32" => (syn::parse_quote!(jni::sys::jint), syn::parse_quote!(*v)),
        "i64" => (syn::parse_quote!(jni::sys::jlong), syn::parse_quote!(*v)),
        "f64" => (syn::parse_quote!(jni::sys::jdouble), syn::parse_quote!(*v)),
        "Duration" | "std :: time :: Duration" => (
            syn::parse_quote!(jni::sys::jlong),
            syn::parse_quote!(std::time::Duration::from_millis(*v as u64)),
        ),
        "String" => (
            syn::parse_quote!(jni::objects::JString),
            syn::parse_quote!({
                let s = env.get_string(v).map_err(|e| {
                    <__JniErr as ::core::convert::From<String>>::from(format!(
                        "decode_string: {}",
                        e
                    ))
                })?;
                s.into()
            }),
        ),
        "Vec < u8 >" => (
            syn::parse_quote!(jni::objects::JByteArray),
            syn::parse_quote!({
                env.convert_byte_array(v).map_err(|e| {
                    <__JniErr as ::core::convert::From<String>>::from(format!(
                        "decode_byte_array: {}",
                        e
                    ))
                })?
            }),
        ),
        _ => return None,
    })
}

pub(crate) fn primitive_output(ty: &syn::Type) -> Option<(syn::Type, syn::Expr)> {
    let key = TypeKey::from_type(ty).as_str().to_string();
    // Output wrappers take v by value (move). Primitives are Copy, so
    // `v as wire` works. String/Vec consume v.
    Some(match key.as_str() {
        "bool" => (
            syn::parse_quote!(jni::sys::jboolean),
            syn::parse_quote!(v as jni::sys::jboolean),
        ),
        "i32" => (
            syn::parse_quote!(jni::sys::jint),
            syn::parse_quote!(v as jni::sys::jint),
        ),
        "i64" => (
            syn::parse_quote!(jni::sys::jlong),
            syn::parse_quote!(v as jni::sys::jlong),
        ),
        "f64" => (
            syn::parse_quote!(jni::sys::jdouble),
            syn::parse_quote!(v as jni::sys::jdouble),
        ),
        "String" => (
            syn::parse_quote!(jni::objects::JString),
            syn::parse_quote!({
                env.new_string(v.as_str()).map_err(|e| {
                    <__JniErr as ::core::convert::From<String>>::from(format!(
                        "encode_string: {}",
                        e
                    ))
                })?
            }),
        ),
        "Vec < u8 >" => (
            syn::parse_quote!(jni::objects::JByteArray),
            syn::parse_quote!({
                env.byte_array_from_slice(v.as_slice()).map_err(|e| {
                    <__JniErr as ::core::convert::From<String>>::from(format!(
                        "encode_byte_array: {}",
                        e
                    ))
                })?
            }),
        ),
        _ => return None,
    })
}

// ──────────────────────────────────────────────────────────────────────
// Option<_> wrappers
// ──────────────────────────────────────────────────────────────────────

/// Build `Option<T>`'s input converter.
///
/// Two paths, picked in this order:
///
/// 1. **Niche path** (preferred). If `T`'s converter exposes any niche
///    slots, carve the first one and use it as the `None` discriminator.
///    The wrapper keeps `T`'s wire unchanged — no boxing, no extra
///    allocation, ABI-identical to a hand-written `if v == sentinel`.
///    The `rest` of the niche set is re-exported on the wrapper so an
///    enclosing wrapper (e.g. `Option<Option<T>>`) can keep carving.
///
/// 2. **Boxed-primitive fallback**. If `T`'s wire is a JNI primitive
///    (`jlong`, `jint`, …) and there is no niche, the wrapper widens
///    the wire to `JObject` carrying a Java boxed type (`java.lang.Long`,
///    `java.lang.Integer`, …). `null` denotes `None`. The wrapper
///    exposes no further niches — every `JObject` value already carries
///    meaning (null = None, non-null = Some).
///
/// If neither path applies (non-primitive wire, no niche), the wrap
/// fails and the resolver falls through to other rank-1 attempts.
pub(crate) fn option_input(
    t1: &syn::Type,
    registry: &Registry<KotlinMeta>,
) -> Option<(syn::Type, syn::Expr, Niches)> {
    let inner_entry = registry.input_entry(t1)?;
    let inner_wire = inner_entry.destination.clone();
    let inner_conv = inner_entry.function.sig.ident.clone();

    // 1. Niche path.
    if let Some((slot, rest)) = inner_entry.niches.clone().carve() {
        let pred = &slot.matches;
        let returns_owned_object = inner_entry.metadata.is_direct_handle();
        let body: syn::Expr = if returns_owned_object {
            // Borrow semantics: the Java side still owns the boxed value
            // (its `close()` will free the original Box later via the typed
            // handle's `freePtr`). Cloning the inner T keeps the pointer
            // live across this call — using `Box::from_raw` here would
            // consume the box, leaving the Java slot dangling and causing
            // a double-free the next time the same data-class instance is
            // decoded. Requires `T: Clone`.
            syn::parse_quote!({
                if #pred {
                    None
                } else {
                    Some(unsafe { OwnedObject::from_raw(*v as *const #t1).clone() })
                }
            })
        } else {
            syn::parse_quote!({
                if #pred { None } else { Some(#inner_conv(env, v)?) }
            })
        };
        return Some((inner_wire, body, rest));
    }

    // 2. Boxed-primitive fallback.
    if is_jni_primitive(&inner_wire) {
        let unbox_method = jni_unbox_method(&inner_wire);
        let unbox_sig = jni_unbox_sig(&inner_wire);
        let getter = jni_unbox_getter(&inner_wire);
        let getter_id = format_ident!("{}", getter);
        let body: syn::Expr = syn::parse_quote!({
            if !v.is_null() {
                let __unboxed: #inner_wire = env
                    .call_method(&v, #unbox_method, #unbox_sig, &[])
                    // `JValue::z()` yields a Rust `bool`, every other accessor
                    // yields its matching `jni::sys` type; the `as #inner_wire`
                    // coerces `bool → jboolean` and is an identity cast for the
                    // numeric accessors.
                    .and_then(|val| val.#getter_id())
                    .map(|__x| __x as #inner_wire)
                    .map_err(|e| <__JniErr as ::core::convert::From<String>>::from(format!("Option unbox: {}", e)))?;
                Some(#inner_conv(env, &__unboxed)?)
            } else {
                None
            }
        });
        let wire: syn::Type = syn::parse_quote!(jni::objects::JObject);
        return Some((wire, body, Niches::empty()));
    }

    None
}

/// Build `Option<T>`'s output converter — symmetric to [`option_input`].
pub(crate) fn option_output(
    t1: &syn::Type,
    registry: &Registry<KotlinMeta>,
) -> Option<(syn::Type, syn::Expr, Niches)> {
    let inner_entry = registry.output_entry(t1)?;
    let inner_wire = inner_entry.destination.clone();
    let inner_conv = inner_entry.function.sig.ident.clone();

    // 1. Niche path.
    if let Some((slot, rest)) = inner_entry.niches.clone().carve() {
        let none_value = &slot.value;
        let body: syn::Expr = syn::parse_quote!({
            match v {
                Some(value) => #inner_conv(env, value)?,
                None => #none_value,
            }
        });
        return Some((inner_wire, body, rest));
    }

    // 2. Boxed-primitive fallback.
    if is_jni_primitive(&inner_wire) {
        let java_class = jni_box_class(&inner_wire);
        let box_sig = jni_box_sig(&inner_wire);
        let variant = jni_box_variant(&inner_wire);
        let variant_id = format_ident!("{}", variant);
        let body: syn::Expr = syn::parse_quote!({
            match v {
                Some(value) => {
                    let __raw: #inner_wire = #inner_conv(env, value)?;
                    env.call_static_method(
                        #java_class,
                        "valueOf",
                        #box_sig,
                        &[jni::objects::JValue::#variant_id(__raw)],
                    )
                    .and_then(|val| val.l())
                    .map_err(|e| <__JniErr as ::core::convert::From<String>>::from(format!("Option box: {}", e)))?
                }
                None => jni::objects::JObject::null(),
            }
        });
        let wire: syn::Type = syn::parse_quote!(jni::objects::JObject);
        return Some((wire, body, Niches::empty()));
    }

    None
}

// ──────────────────────────────────────────────────────────────────────
// Callback wrappers — impl Fn(args) -> JObject (Kotlin fun-interface)
// ──────────────────────────────────────────────────────────────────────

pub(crate) fn callback_input(
    ext: &JniGen,
    args: &[syn::Type],
    registry: &Registry<KotlinMeta>,
) -> Option<(syn::Type, syn::Expr)> {
    let name = derive_callback_name(args);

    // Per-arg: encode call + JNI signature chunk.
    let mut arg_idents: Vec<syn::Ident> = Vec::new();
    let mut arg_preludes: Vec<TokenStream> = Vec::new();
    let mut jvalue_exprs: Vec<TokenStream> = Vec::new();
    // Opaque-handle args wrapped into a typed handle object; closed after
    // the callback returns so the per-invocation `Box` is freed.
    let mut handle_obj_idents: Vec<syn::Ident> = Vec::new();
    let mut sig = String::from("(");

    for (i, arg_ty) in args.iter().enumerate() {
        let raw_ident = format_ident!("__arg{}", i);
        let enc_ident = format_ident!("__arg{}_encoded", i);
        let obj_ident = format_ident!("__arg{}_obj", i);

        // Args are output-direction (encoded outbound). Look up output entry.
        let arg_entry = registry.output_entry(arg_ty)?;
        let arg_wire = arg_entry.destination.clone();
        let conv = arg_entry.function.sig.ident.clone();

        // Opaque-handle arg: the output converter produces a `jlong`
        // (`Box::into_raw`), but the callback's `run` takes the typed handle
        // class, not a `Long`. Push the typed FQN slot; the wrapped object is
        // built in the by-value prelude loop below and `close()`-d after the
        // callback returns (see the body).
        if let Some(h) = &arg_entry.metadata.projection {
            let java_path = handle_field_fqn(ext, h).replace('.', "/");
            sig.push_str(&format!("L{};", java_path));
            jvalue_exprs.push(quote!(jni::objects::JValue::Object(&#obj_ident)));
            handle_obj_idents.push(obj_ident);
            arg_idents.push(raw_ident);
            continue;
        }

        // Data-class arg: flatten into the `run` signature so native makes ONE
        // crossing with leaf wires (no built `jni.<Struct>` object, no
        // round-trip). The slots' idents/descriptors are access-independent, so
        // here (sig + JValue list) we use a throwaway access; the matching
        // preludes that bind those idents are emitted in the second loop from
        // the closure param. Prefix `cb{i}` keeps idents unique per arg and
        // distinct from the `__cb_arg{i}` closure params.
        if let Some(st) = callback_arg_data_class(ext, registry, arg_ty) {
            let prefix = format!("cb{}", i);
            let (_pre, slots) =
                flatten_struct_encode(ext, registry, &st, &quote!(__unused), &prefix, 0, &quote!(env))?;
            for sl in &slots {
                sig.push_str(&sl.descriptor);
                let id = &sl.ident;
                if sl.is_object {
                    jvalue_exprs.push(quote!(jni::objects::JValue::Object(&#id)));
                } else {
                    jvalue_exprs.push(quote!(jni::objects::JValue::from(#id)));
                }
            }
            arg_idents.push(raw_ident);
            continue;
        }

        match jni_field_access(&arg_wire) {
            Some((s, _, false)) => {
                sig.push_str(s);
                arg_preludes.push(quote! {
                    let #raw_ident = &__cb_args.#i;
                    let #enc_ident = #conv(&mut env, #raw_ident)?;
                });
                jvalue_exprs.push(quote!(jni::objects::JValue::from(#enc_ident)));
            }
            Some((s, _, true)) => {
                sig.push_str(s);
                arg_preludes.push(quote! {
                    let #raw_ident = &__cb_args.#i;
                    let #enc_ident = #conv(&mut env, #raw_ident)?;
                    let #obj_ident: jni::objects::JObject = #enc_ident.into();
                });
                jvalue_exprs.push(quote!(jni::objects::JValue::Object(&#obj_ident)));
            }
            None if is_jobject_wire(&arg_wire) => {
                // The callback's `run` method takes the Kotlin equivalent
                // of this Rust arg type, not the callback interface itself.
                // Look up the registered FQN and slash-encode it for the
                // JVM method descriptor.
                let arg_key = TypeKey::from_type(arg_ty).as_str().to_string();
                let arg_fqn = ext
                    .kotlin_fqn(&arg_key)
                    .map(|v| v.replace('.', "/"))
                    .unwrap_or_else(|| "java/lang/Object".to_string());
                sig.push_str(&format!("L{};", arg_fqn));
                arg_preludes.push(quote! {
                    let #enc_ident = #conv(&mut env, &__cb_args.#i)?;
                    let #obj_ident: jni::objects::JObject = #enc_ident;
                });
                jvalue_exprs.push(quote!(jni::objects::JValue::Object(&#obj_ident)));
            }
            None => return None, // unsupported wire form
        }
        arg_idents.push(raw_ident);
    }
    sig.push_str(")V");

    // Tuple destructure for closure args.
    let arg_pat_ty: Vec<TokenStream> = args.iter().map(|t| quote!(#t)).collect();
    let arg_pat_ident: Vec<TokenStream> = (0..args.len())
        .map(|i| {
            let ident = format_ident!("__cb_arg{}", i);
            quote!(#ident)
        })
        .collect();
    let _ = arg_pat_ident;

    let name_lit = syn::LitStr::new(&name, Span::call_site());
    let sig_lit = syn::LitStr::new(&sig, Span::call_site());

    // Body: capture global ref, return a Box<dyn Fn(args)>.
    // The wrapper takes the raw JObject `v` (the Kotlin callback ref).
    let arg_indices: Vec<syn::Index> = (0..args.len()).map(syn::Index::from).collect();
    let _ = arg_indices;

    // Build the Fn closure body.
    let arg_names: Vec<syn::Ident> = (0..args.len())
        .map(|i| format_ident!("__cb_arg{}", i))
        .collect();

    // Convert (self.0, .1, ...) tuple field accesses into __cb_arg0, _arg1.
    // Replace `__cb_args.0` with `__cb_arg0` etc. in arg_preludes by
    // re-rendering: easier to just rebuild here.
    let mut fixed_preludes: Vec<TokenStream> = Vec::new();
    for (i, arg_ty) in args.iter().enumerate() {
        let raw_ident = format_ident!("__arg{}", i);
        let enc_ident = format_ident!("__arg{}_encoded", i);
        let obj_ident = format_ident!("__arg{}_obj", i);
        let cb_arg = &arg_names[i];
        let arg_entry = registry.output_entry(arg_ty)?;
        let arg_wire = arg_entry.destination.clone();
        let conv = arg_entry.function.sig.ident.clone();
        // Opaque-handle arg: encode to `jlong` then wrap into the typed
        // handle class via its `(J)V` ctor. By-value non-optional, so no
        // null guard. The box is freed after the callback via `close()`
        // in the body below.
        if let Some(h) = &arg_entry.metadata.projection {
            let java_path = handle_field_fqn(ext, h).replace('.', "/");
            let java_path_lit = syn::LitStr::new(&java_path, Span::call_site());
            fixed_preludes.push(quote! {
                let #enc_ident = #conv(&mut env, #cb_arg)?;
                let #obj_ident: jni::objects::JObject = env
                    .new_object(#java_path_lit, "(J)V", &[jni::objects::JValue::from(#enc_ident)])
                    .map_err(|e| <__JniErr as ::core::convert::From<String>>::from(format!("wrap typed handle {}: {}", #java_path_lit, e)))?;
            });
            let _ = raw_ident;
            continue;
        }
        // Data-class arg: emit the flatten preludes that encode the struct's
        // leaf wires from the closure param (`#cb_arg`). Same prefix `cb{i}` as
        // loop 1, so the bound idents match the `JValue` list built there.
        if let Some(st) = callback_arg_data_class(ext, registry, arg_ty) {
            let prefix = format!("cb{}", i);
            let access = quote!(#cb_arg);
            let (pre, _slots) =
                flatten_struct_encode(ext, registry, &st, &access, &prefix, 0, &quote!(&mut env))?;
            fixed_preludes.push(pre);
            let _ = raw_ident;
            continue;
        }
        // Output wrappers take rust by value (move). cb_arg is the
        // closure parameter (by value), so pass it directly.
        match jni_field_access(&arg_wire) {
            Some((_, _, false)) => fixed_preludes.push(quote! {
                let #enc_ident = #conv(&mut env, #cb_arg)?;
            }),
            Some((_, _, true)) => fixed_preludes.push(quote! {
                let #enc_ident = #conv(&mut env, #cb_arg)?;
                let #obj_ident: jni::objects::JObject = #enc_ident.into();
            }),
            None if is_jobject_wire(&arg_wire) => fixed_preludes.push(quote! {
                let #enc_ident = #conv(&mut env, #cb_arg)?;
                let #obj_ident: jni::objects::JObject = #enc_ident;
            }),
            None => return None,
        }
        let _ = raw_ident; // unused with by-value flow
    }

    let body: syn::Expr = syn::parse_quote!({
        use std::sync::Arc;
        let java_vm = Arc::new(env.get_java_vm()
            .map_err(|e| <__JniErr as ::core::convert::From<String>>::from(format!("Unable to retrieve JVM: {}", e)))?);
        let callback_global_ref = env.new_global_ref(&v)
            .map_err(|e| <__JniErr as ::core::convert::From<String>>::from(format!("Unable to global-ref callback: {}", e)))?;
        Box::new(move |#(#arg_names: #arg_pat_ty),*| {
            let _ = (|| -> ::core::result::Result<(), __JniErr> {
                let mut env = java_vm
                    .attach_current_thread_as_daemon()
                    .map_err(|e| <__JniErr as ::core::convert::From<String>>::from(format!("Attach thread for {}: {}", #name_lit, e)))?;
                // The callback fires on a daemon-attached zenoh RX thread that
                // never returns through a JNI stack frame, so the JNI local
                // refs each invocation creates (encoded args, wrapped handle
                // objects, call temporaries) would otherwise accumulate for
                // the thread's lifetime and exhaust the JVM heap
                // (OutOfMemoryError). Bracket each invocation in an explicit
                // local frame so every local is released when the frame pops —
                // popped unconditionally below so an early `?`/error path
                // still frees it.
                env.push_local_frame(16)
                    .map_err(|e| <__JniErr as ::core::convert::From<String>>::from(format!("push local frame for {}: {}", #name_lit, e)))?;
                let __frame_res = (|| -> ::core::result::Result<(), __JniErr> {
                    #(#fixed_preludes)*
                    let __call_res: ::core::result::Result<(), __JniErr> = env.call_method(
                        &callback_global_ref,
                        "run",
                        #sig_lit,
                        &[#(#jvalue_exprs),*],
                    )
                    .map(|_| ())
                    .map_err(|e| {
                        // `exception_describe` also clears the pending exception,
                        // so subsequent JNI calls (the handle closes below) are safe.
                        let _ = env.exception_describe();
                        <__JniErr as ::core::convert::From<String>>::from(e.to_string())
                    });
                    // Free each opaque-handle arg's per-invocation `Box` once the
                    // callback returns — a no-op if the consumer `take()`-ed the
                    // handle (its slot is then already 0). Runs even when the
                    // callback threw, so a throwing consumer never leaks.
                    #(let _ = env.call_method(&#handle_obj_idents, "close", "()V", &[]);)*
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

    // The destination type for an `impl Fn(args)` parameter is JObject (the
    // Kotlin callback object). We return Box<dyn Fn(args) + Send + Sync>
    // wrapped in a generic so it satisfies the impl-trait param type.
    // Actually the SOURCE (rust) type IS `impl Fn(args) + Send + Sync + 'static`,
    // so the wrapper's return type is that. Box<dyn Fn> coerces.
    Some((syn::parse_quote!(jni::objects::JObject), body))
}

pub(crate) fn derive_callback_name(args: &[syn::Type]) -> String {
    let mut s = String::new();
    for a in args {
        s.push_str(&type_short_ident(a));
    }
    s.push_str("Callback");
    s
}

pub(crate) fn type_short_ident(ty: &syn::Type) -> String {
    if let syn::Type::Path(tp) = ty {
        if let Some(last) = tp.path.segments.last() {
            return last.ident.to_string();
        }
    }
    "Unknown".into()
}

pub(crate) fn is_jobject_wire(wire: &syn::Type) -> bool {
    if let syn::Type::Path(tp) = wire {
        if let Some(last) = tp.path.segments.last() {
            return last.ident == "JObject";
        }
    }
    false
}

/// True if `wire` is a JNI handle (`JObject`, `JString`, `JByteArray`,
/// `JClass`) that natively supports a `null` discriminator. These types
/// all impl `is_null()` and accept `JObject::null().into()` for
/// construction.
pub(crate) fn is_jobject_shaped_wire(wire: &syn::Type) -> bool {
    if let syn::Type::Path(tp) = wire {
        if let Some(last) = tp.path.segments.last() {
            return matches!(
                last.ident.to_string().as_str(),
                "JObject" | "JString" | "JByteArray" | "JClass"
            );
        }
    }
    false
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
                 it as a value-blob via `.value_blob(...)` so the Vec surfaces as \
                 `List<ByteArray>`; otherwise expose a per-element accessor instead of \
                 returning a `Vec` of handles.",
                elem.to_token_stream(),
                elem.to_token_stream(),
            );
        }
    }
}

/// Default niche set for a JNI wrapper wire: every `J*` handle has a
/// genuine `null` value that no live conversion ever produces, so wrap
/// it as a single niche; everything else (`jlong`, `jint`, `()`, …) has
/// no implicit niche.
///
/// Plugins are free to declare *additional* niches on top of this for
/// pointer-shape primitives like `Box::into_raw`-as-`jlong`.
pub(crate) fn default_niches_for_wire(wire: &syn::Type) -> Niches {
    if is_jobject_shaped_wire(wire) {
        Niches::one(
            syn::parse_quote!(jni::objects::JObject::null().into()),
            syn::parse_quote!(v.is_null()),
        )
    } else {
        Niches::empty()
    }
}

// ──────────────────────────────────────────────────────────────────────
// Struct rank-0 bodies
// ──────────────────────────────────────────────────────────────────────

/// Resolve the typed-handle Kotlin FQN for a handle-bearing struct field
/// and assert its folded strategy is one the struct encode/decode bridge
/// supports. Today only scalar handle slots (`Direct`, optionally wrapped
/// in `Nullable`) are encodable as a single `L<FQN>;` ctor arg; a
/// collection layer (`Iterable`, i.e. `Vec<Handle>`) would need array
/// codegen and is a loud build-time error until implemented.
pub(crate) fn handle_field_fqn(ext: &JniGen, h: &Projection) -> String {
    fn assert_scalar(s: &FoldStrategy) {
        match s {
            FoldStrategy::Direct => {}
            FoldStrategy::Nullable { inner, .. } => assert_scalar(inner),
            FoldStrategy::Iterable(_) => panic!(
                "struct handle field: collection (Vec<Handle>) layers are not yet \
                 supported by the struct encode/decode bridge — add array codegen \
                 to struct_output_body/struct_input_body to lift this guard"
            ),
        }
    }
    assert_scalar(&h.strategy);
    ext.kotlin_fqn(&h.leaf_key)
        .map(|v| v.to_string())
        .unwrap_or_else(|| {
            panic!(
                "struct handle field: leaf `{}` has no Kotlin FQN registered \
                 (ptr_class)",
                h.leaf_key
            )
        })
}

pub(crate) fn struct_input_body(
    ext: &JniGen,
    s: &syn::ItemStruct,
    registry: &Registry<KotlinMeta>,
) -> Option<(syn::Type, syn::Expr)> {
    let struct_name = s.ident.to_string();
    let struct_module = struct_module_path(ext, s);
    let struct_ident = &s.ident;

    let syn::Fields::Named(named) = &s.fields else {
        return None;
    };

    let mut field_preludes: Vec<TokenStream> = Vec::new();
    let mut field_init: Vec<TokenStream> = Vec::new();

    for field in &named.named {
        let fname_ident = field.ident.as_ref().unwrap().clone();
        let fname = fname_ident.to_string();
        let camel = snake_to_camel(&fname);
        let err_prefix = format!("{struct_name}.{camel}: {{}}");
        let raw_ident = format_ident!("__{}_raw", fname_ident);

        // Defer if any field's input converter isn't resolved yet — the
        // fixed-point loop will retry on the next iteration.
        let field_entry = registry.input_entry(&field.ty)?;
        let field_wire = field_entry.destination.clone();
        let field_conv = field_entry.function.sig.ident.clone();

        // Projection fields — mirror of `struct_output_body`'s kind branch:
        //  * Handle: read the JNINativeHandle object from the JVM slot,
        //    `peek()` the raw jlong, then run the per-field input converter
        //    (jlong-keyed; null handle ⇒ jlong 0 ⇒ `None` via the niche path).
        //  * ValueBlob: the class is JVM-erased to its `bytes: ByteArray`, so
        //    the slot is the `[B` descriptor; read it as a JObject, coerce to
        //    the inner wire, and run the per-field converter. (Without this
        //    branch a value-blob field would be mis-decoded as a handle —
        //    peeking a non-handle object.)
        if let Some(proj) = &field_entry.metadata.projection {
            match proj.kind {
                ProjectionKind::Handle => {
                    let java_path = handle_field_fqn(ext, proj).replace('.', "/");
                    let sig = format!("L{};", java_path);
                    let tmp_ident = format_ident!("__{}_jobj", fname_ident);
                    // Struct fields are owned, so a non-`Option` handle field
                    // owns its native object: decode by consuming
                    // (`Box::from_raw` → owned `T`), mirroring
                    // `struct_output_body`'s `Box::into_raw`. The borrow
                    // converter would yield `OwnedObject<T>`, which can't
                    // populate an owned field. `Option<_>` handle fields keep
                    // the niche-aware converter (jlong 0 ⇒ `None`).
                    let field_ty = &field.ty;
                    let field_is_option = matches!(
                        field_ty,
                        syn::Type::Path(p) if p.path.segments.last()
                            .map(|s| s.ident == "Option").unwrap_or(false)
                    );
                    let decode = if field_is_option {
                        quote! { let #fname_ident = #field_conv(env, &#raw_ident)?; }
                    } else {
                        quote! {
                            let #fname_ident: #field_ty = unsafe {
                                *std::boxed::Box::from_raw(#raw_ident as *mut #field_ty)
                            };
                        }
                    };
                    field_preludes.push(quote! {
                        let #tmp_ident: jni::objects::JObject = env.get_field(v, #camel, #sig)
                            .and_then(|val| val.l())
                            .map_err(|e| <__JniErr as ::core::convert::From<String>>::from(format!(#err_prefix, e)))?;
                        let #raw_ident: jni::sys::jlong = if #tmp_ident.is_null() {
                            0
                        } else {
                            env.call_method(&#tmp_ident, "peek", "()J", &[])
                                .and_then(|val| val.j())
                                .map_err(|e| <__JniErr as ::core::convert::From<String>>::from(format!(#err_prefix, e)))?
                        };
                        #decode
                    });
                }
                ProjectionKind::ValueBlob => {
                    let descriptor = "[B";
                    let tmp_ident = format_ident!("__{}_jobj", fname_ident);
                    field_preludes.push(quote! {
                        let #tmp_ident: jni::objects::JObject = env.get_field(v, #camel, #descriptor)
                            .and_then(|val| val.l())
                            .map_err(|e| <__JniErr as ::core::convert::From<String>>::from(format!(#err_prefix, e)))?;
                        let #raw_ident: #field_wire = #tmp_ident.into();
                        let #fname_ident = #field_conv(env, &#raw_ident)?;
                    });
                }
            }
            field_init.push(quote!(#fname_ident));
            continue;
        }

        match jni_field_access(&field_wire) {
            Some((sig, accessor, false)) => {
                field_preludes.push(quote! {
                    let #raw_ident: #field_wire = env.get_field(v, #camel, #sig)
                        .and_then(|val| val.#accessor())
                        .map_err(|e| <__JniErr as ::core::convert::From<String>>::from(format!(#err_prefix, e)))? as _;
                    let #fname_ident = #field_conv(env, &#raw_ident)?;
                });
            }
            Some((sig, _, true)) => {
                let tmp_ident = format_ident!("__{}_jobj", fname_ident);
                field_preludes.push(quote! {
                    let #tmp_ident: jni::objects::JObject = env.get_field(v, #camel, #sig)
                        .and_then(|val| val.l())
                        .map_err(|e| <__JniErr as ::core::convert::From<String>>::from(format!(#err_prefix, e)))?;
                    let #raw_ident: #field_wire = #tmp_ident.into();
                    let #fname_ident = #field_conv(env, &#raw_ident)?;
                });
            }
            None => {
                // Wire is JObject — fetch via .l() and pass by reference.
                field_preludes.push(quote! {
                    let #raw_ident: jni::objects::JObject = env.get_field(v, #camel, "Ljava/lang/Object;")
                        .and_then(|val| val.l())
                        .map_err(|e| <__JniErr as ::core::convert::From<String>>::from(format!(#err_prefix, e)))?;
                    let #fname_ident = #field_conv(env, &#raw_ident)?;
                });
            }
        }
        field_init.push(quote!(#fname_ident));
    }

    let body: syn::Expr = syn::parse_quote!({
        #(#field_preludes)*
        #struct_module::#struct_ident { #(#field_init),* }
    });
    Some((syn::parse_quote!(jni::objects::JObject), body))
}

// ──────────────────────────────────────────────────────────────────────
// Struct input flattening (pass a data_class param as its leaf fields)
// ──────────────────────────────────────────────────────────────────────

/// One flattened leaf of a struct **input** param. The mirror of
/// [`EncSlot`] for the input direction: instead of reading the field with
/// `env.get_field(...)` out of a single `JObject`, the leaf crosses the JNI
/// boundary as its own wrapper parameter. Carries every fact the three
/// coordinated sites (native wrapper signature, `JNINative` extern decl,
/// Kotlin call-site destructure) need so they cannot drift in order, type, or
/// nullability.
pub(crate) struct FlatLeaf {
    /// Native wrapper parameter ident — also the decode source.
    pub native_ident: syn::Ident,
    /// Native wire type (lifetime-annotated for object wires).
    pub native_wire_ty: TokenStream,
    /// Kotlin `external fun` parameter name (camelCase).
    pub kt_name: String,
    /// Kotlin `external fun` parameter type (incl. a trailing `?`).
    pub kt_wire_ty: String,
    /// Kotlin call-site destructure expression feeding this leaf.
    pub kt_access: String,
    /// Per-field input converter ident (`None` for the synthetic present flag).
    pub conv: Option<syn::Ident>,
    /// Struct field this leaf populates (`None` for the present flag).
    pub field: Option<syn::Ident>,
    /// `true` for the synthetic `<param>Present: Boolean` gate leaf emitted
    /// for an `Option<struct>` param.
    pub is_present_flag: bool,
}

/// A flattened plan for one struct input parameter. Built once by
/// [`build_flat_input_plan`] and consumed by all three codegen sites.
pub(crate) struct FlatInputPlan {
    pub leaves: Vec<FlatLeaf>,
    /// Module path the struct lives under (`zenoh_flat`).
    pub struct_module: syn::Path,
    /// Struct ident (`Encoding`).
    pub struct_ident: syn::Ident,
    /// `true` when the original param was `Option<…>` — leaves are gated on a
    /// `present` flag and decoded lazily.
    pub optional: bool,
    /// `true` when the source fn takes `&Struct` — the call site passes `&arg`.
    pub by_ref: bool,
    /// The present-flag param ident (`Some` iff `optional`).
    pub present_ident: Option<syn::Ident>,
}

/// Extract `S` from an `impl Into<S> + …` parameter type.
pub(crate) fn impl_into_target(ty: &syn::Type) -> Option<syn::Type> {
    let syn::Type::ImplTrait(it) = ty else {
        return None;
    };
    for b in &it.bounds {
        if let syn::TypeParamBound::Trait(tb) = b {
            if let Some(seg) = tb.path.segments.last() {
                if seg.ident == "Into" {
                    if let syn::PathArguments::AngleBracketed(ab) = &seg.arguments {
                        if let Some(syn::GenericArgument::Type(t)) = ab.args.first() {
                            return Some(t.clone());
                        }
                    }
                }
            }
        }
    }
    None
}

/// Peel a leading `&`/`&mut` then an `Option<…>` to expose the inner type used
/// for enum/struct detection (`&Priority`, `Option<Priority>` → `Priority`).
pub(crate) fn flat_probe_inner(ty: &syn::Type) -> syn::Type {
    let stripped = match ty {
        syn::Type::Reference(r) => (*r.elem).clone(),
        other => other.clone(),
    };
    option_inner_type(&stripped).unwrap_or(stripped)
}

/// Kotlin literal that fills a leaf slot when its `Option<struct>` parent is
/// absent (the `present` flag tells Rust to ignore it). `None` for nullable
/// leaves, which simply ride a JVM `null`. Mirrors
/// [`primitive_default_for_descriptor`] on the Rust side.
pub(crate) fn kt_leaf_default(sig: &str, nullable: bool) -> Option<String> {
    if nullable {
        return None;
    }
    Some(
        match sig {
            "Z" => "false",
            "B" | "S" | "I" => "0",
            "C" => "'\\u0000'",
            "J" => "0L",
            "F" => "0.0f",
            "D" => "0.0",
            "Ljava/lang/String;" => "\"\"",
            "[B" => "ByteArray(0)",
            _ => "null",
        }
        .to_string(),
    )
}

/// Build a [`FlatInputPlan`] for a struct input parameter, or `None` to keep
/// the existing single-`JObject` path. Returns `None` (safe fallback) for any
/// shape outside the conservative v1 leaf set — handle/value projections,
/// enums, nested data classes, boxed `Option<primitive>`, `Vec<non-u8>`,
/// converters with `pre_stages`, and `impl Into<S>` dispatch (`Any`). This is
/// the single source of truth shared by the native wrapper signature, the
/// `JNINative` extern declaration, and the Kotlin call-site destructure.
pub(crate) fn build_flat_input_plan(
    ext: &JniGen,
    registry: &Registry<KotlinMeta>,
    param_name: &syn::Ident,
    arg_ty: &syn::Type,
    kt_base: &str,
) -> Option<FlatInputPlan> {
    // 1. Resolve the struct target through `&`, `Option<…>`, and `impl Into<S>`.
    let (by_ref, t1) = match arg_ty {
        syn::Type::Reference(r) => (true, (*r.elem).clone()),
        other => (false, other.clone()),
    };
    let (optional, inner) = match option_inner_type(&t1) {
        Some(i) => (true, i),
        None => (false, t1.clone()),
    };
    let struct_ty = impl_into_target(&inner).unwrap_or_else(|| inner.clone());
    let name = bare_path_ident(&struct_ty)?;
    let (st, _) = registry.structs.get(&name)?;
    let key = TypeKey::from_type(&struct_ty);
    let cfg = ext.types.get(&key);
    // Exclude value-blob / enum structs — they have their own
    // erasure and are not field-flattened here.
    if cfg.map(|c| c.value_blob).unwrap_or(false) {
        return None;
    }
    if ext.is_kotlin_enum(&struct_ty) {
        return None;
    }
    // Identity / pass-through guard: the resolved param must decode to the
    // struct itself, not an opaque handle / value projection (`projection`
    // present) and not a multi-source / non-identity `impl Into<S>` (which
    // surfaces as `"Any"` Dispatch or a foreign source type). The resolved
    // param's Kotlin type (compared by short name, since metadata carries the
    // FQN) must equal the struct's data-class name.
    let entry = registry.input_entry(arg_ty)?;
    if entry.metadata.projection.is_some() {
        return None;
    }
    let dc_short = cfg
        .and_then(|c| c.kotlin_name.clone())
        .map(|fqn| fqn.rsplit('.').next().unwrap_or(&fqn).to_string())
        .unwrap_or_else(|| name.to_string());
    let entry_short = entry
        .metadata
        .kotlin_name
        .as_deref()
        .map(|s| s.rsplit('.').next().unwrap_or(s));
    if entry_short != Some(dc_short.as_str()) {
        return None;
    }

    // 2. Named fields only.
    let syn::Fields::Named(named) = &st.fields else {
        return None;
    };

    // 3. Classify every field as a simple leaf, else fall back.
    let struct_module = struct_module_path(ext, st);
    // `kt_base` is the Kotlin expression for the object at the call site —
    // normally the camelCase param name, or `this` for a promoted instance
    // receiver. The native param idents / extern names stay keyed on
    // `param_name` so the wire signature is independent of the call form.
    let kt_param = kt_base.to_string();
    let mut leaves: Vec<FlatLeaf> = Vec::new();

    // Present gate for `Option<struct>` (first leaf, mirrors the output
    // `Option<nested>` `present: jboolean` slot).
    let present_ident = if optional {
        let id = format_ident!("{}_present", param_name);
        leaves.push(FlatLeaf {
            native_ident: id.clone(),
            native_wire_ty: quote!(jni::sys::jboolean),
            kt_name: snake_to_camel(&format!("{}_present", param_name)),
            kt_wire_ty: "Boolean".to_string(),
            kt_access: format!("{kt_param} != null"),
            conv: None,
            field: None,
            is_present_flag: true,
        });
        Some(id)
    } else {
        None
    };

    for field in &named.named {
        let fident = field.ident.clone()?;
        let fcamel = snake_to_camel(&fident.to_string());
        let fentry = registry.input_entry(&field.ty)?;
        // Reject anything outside the simple-leaf set (keeps the object path).
        if !fentry.pre_stages.is_empty() {
            return None;
        }
        if fentry.metadata.projection.is_some() {
            return None;
        }
        if ext.is_kotlin_enum(&flat_probe_inner(&field.ty)) {
            return None;
        }
        let wire = &fentry.destination;
        let (sig, _accessor, _is_obj) = jni_field_access(wire)?;
        let f_opt = option_inner_type(&field.ty).is_some();
        let kt = fentry.metadata.kotlin_name.clone()?;
        let kt_wire_ty = format!("{}{}", kt, if f_opt { "?" } else { "" });
        let native_ident = format_ident!("{}_{}", param_name, fident);
        let native_wire_ty = annotate_jobject_with_lifetime(wire, "a").to_token_stream();
        let kt_name = snake_to_camel(&format!("{}_{}", param_name, fident));

        // Destructure expression. Under an absent `Option<struct>` parent the
        // leaf still needs a value on the wire (`present` makes Rust ignore
        // it): nullable leaves ride JVM null, non-null leaves a typed default.
        let kt_access = if optional {
            let base = format!("{kt_param}?.{fcamel}");
            match kt_leaf_default(sig, f_opt) {
                Some(def) => format!("{base} ?: {def}"),
                None => base,
            }
        } else {
            format!("{kt_param}.{fcamel}")
        };

        leaves.push(FlatLeaf {
            native_ident,
            native_wire_ty,
            kt_name,
            kt_wire_ty,
            kt_access,
            conv: Some(fentry.function.sig.ident.clone()),
            field: Some(fident),
            is_present_flag: false,
        });
    }

    Some(FlatInputPlan {
        leaves,
        struct_module,
        struct_ident: st.ident.clone(),
        optional,
        by_ref,
        present_ident,
    })
}

/// Render the native reconstruct for a [`FlatInputPlan`]: decode each leaf
/// param with its per-field converter (lazily, inside the `present` branch for
/// an `Option<struct>`) and bind the rebuilt struct to `arg_ident`. Each decode
/// failure routes through `signal_error` (the per-call sink) and returns the
/// function `on_err` sentinel. Returns the prelude statements and the call
/// argument (`arg` or `&arg`).
pub(crate) fn render_flat_input_decode(
    plan: &FlatInputPlan,
    arg_ident: &syn::Ident,
    on_err: &TokenStream,
) -> (TokenStream, TokenStream) {
    let module = &plan.struct_module;
    let sid = &plan.struct_ident;
    let mut field_decodes: Vec<TokenStream> = Vec::new();
    let mut field_inits: Vec<TokenStream> = Vec::new();
    for leaf in &plan.leaves {
        if leaf.is_present_flag {
            continue;
        }
        let conv = leaf.conv.as_ref().unwrap();
        let wid = &leaf.native_ident;
        let fid = leaf.field.clone().unwrap();
        let tmp = format_ident!("__{}_{}", arg_ident, fid);
        field_decodes.push(quote! {
            let #tmp = match #conv(&mut env, &#wid) {
                ::core::result::Result::Ok(__v) => __v,
                ::core::result::Result::Err(__e) => {
                    signal_error(&mut env, &__error_sink, &__e);
                    return #on_err;
                }
            };
        });
        field_inits.push(quote!(#fid: #tmp));
    }
    let build = quote!(#module::#sid { #(#field_inits),* });
    let prelude = if plan.optional {
        let present = plan.present_ident.as_ref().unwrap();
        quote! {
            let #arg_ident = if #present != 0u8 {
                #(#field_decodes)*
                Some(#build)
            } else {
                None
            };
        }
    } else {
        quote! {
            #(#field_decodes)*
            let #arg_ident = #build;
        }
    };
    let call_arg = if plan.by_ref {
        quote!(&#arg_ident)
    } else {
        quote!(#arg_ident)
    };
    (prelude, call_arg)
}

/// One flattened leaf wire slot of a struct's recursive `fromParts` encode
/// (see [`flatten_struct_encode`]). `ident` holds the encoded wire after the
/// preludes run; `default` is the value used for this slot when it sits under
/// an absent `Option<nested>` parent.
pub(crate) struct EncSlot {
    ident: proc_macro2::Ident,
    wire_ty: TokenStream,
    descriptor: String,
    is_object: bool,
    default: TokenStream,
}

/// Zero/null wire value for a JVM descriptor — used to fill an absent
/// `Option<nested>`'s leaf slots (the Kotlin `present` flag tells the factory
/// to ignore them).
pub(crate) fn primitive_default_for_descriptor(sig: &str) -> TokenStream {
    match sig {
        "Z" => quote!(0u8),
        "B" => quote!(0i8),
        "C" => quote!(0u16),
        "S" => quote!(0i16),
        "I" => quote!(0i32),
        "J" => quote!(0i64),
        "F" => quote!(0.0f32),
        "D" => quote!(0.0f64),
        _ => quote!(jni::objects::JObject::null()),
    }
}

/// Recursively flatten a struct's output encode into a list of leaf wire slots
/// + the preludes that compute them, so the whole object graph can be built by
/// a **single** Kotlin `fromParts` call (no per-nested-struct
/// `call_static_method`). Nested non-optional data-class fields are inlined;
/// nested `Option<data-class>` fields emit a `present` `jboolean` slot followed
/// by the child's leaves (encoded in the `Some` arm, defaulted in the `None`
/// arm). Leaves (primitives, handles→`jlong`, value classes/blobs→`ByteArray`,
/// enums→`jint`, strings, `Vec`) terminate the recursion. `access` is the Rust
/// expression yielding the current struct value (`v`, `v.field`, or the matched
/// `__cN` under an Option); `prefix` namespaces the generated idents.
#[allow(clippy::too_many_arguments)]
pub(crate) fn flatten_struct_encode(
    ext: &JniGen,
    registry: &Registry<KotlinMeta>,
    s: &syn::ItemStruct,
    access: &TokenStream,
    prefix: &str,
    depth: usize,
    env_expr: &TokenStream,
) -> Option<(TokenStream, Vec<EncSlot>)> {
    assert!(
        depth <= 16,
        "flatten_struct_encode: recursion too deep at struct `{}` (cyclic data_class?)",
        s.ident
    );
    let syn::Fields::Named(named) = &s.fields else {
        return None;
    };
    let mut preludes = TokenStream::new();
    let mut slots: Vec<EncSlot> = Vec::new();

    for field in &named.named {
        let fname = field.ident.as_ref().unwrap().clone();
        let effective_ty = field.ty.clone();
        let field_entry = registry.output_entry(&effective_ty)?;
        let field_wire = field_entry.destination.clone();
        let field_conv = field_entry.function.sig.ident.clone();
        let value_expr = quote! { #field_conv(#env_expr, #access.#fname.clone())? };
        let base = format!("{}_{}", prefix, fname);
        let id = format_ident!("__{}", base);

        // Projection leaf (opaque handle → jlong, value class / blob → ByteArray).
        if let Some(proj) = &field_entry.metadata.projection {
            match proj.kind {
                ProjectionKind::Handle => {
                    preludes.extend(quote! { let #id: jni::sys::jlong = #value_expr; });
                    slots.push(EncSlot {
                        ident: id,
                        wire_ty: quote!(jni::sys::jlong),
                        descriptor: "J".to_string(),
                        is_object: false,
                        default: quote!(0i64),
                    });
                }
                ProjectionKind::ValueBlob => {
                    preludes
                        .extend(quote! { let #id: jni::objects::JObject = { #value_expr }.into(); });
                    slots.push(EncSlot {
                        ident: id,
                        wire_ty: quote!(jni::objects::JObject),
                        descriptor: "[B".to_string(),
                        is_object: true,
                        default: quote!(jni::objects::JObject::null()),
                    });
                }
            }
            continue;
        }
        // Enum leaf → jint discriminant (Kotlin `fromParts` calls `fromInt`).
        if ext.is_kotlin_enum(&effective_ty) {
            if let Some(name) = bare_path_ident(&effective_ty) {
                if ext.kotlin_fqn(&name.to_string()).is_some() {
                    preludes.extend(quote! { let #id: jni::sys::jint = #value_expr; });
                    slots.push(EncSlot {
                        ident: id,
                        wire_ty: quote!(jni::sys::jint),
                        descriptor: "I".to_string(),
                        is_object: false,
                        default: quote!(0i32),
                    });
                    continue;
                }
            }
        }
        // Nested data-class field (not a projection / not an enum, and its
        // option-stripped bare type is a registered non-value-class struct):
        // recurse and inline its leaves instead of building the child via its
        // own `fromParts` call.
        let inner_ty = option_inner_type(&effective_ty).unwrap_or_else(|| effective_ty.clone());
        let nested_child = bare_path_ident(&inner_ty).and_then(|name| {
            let is_struct = registry.structs.contains_key(&name);
            let is_vc = ext
                .types
                .get(&TypeKey::from_type(&inner_ty))
                .map(|c| c.value_blob)
                .unwrap_or(false);
            if is_struct && !is_vc && !ext.is_kotlin_enum(&inner_ty) {
                registry.structs.get(&name).map(|(st, _)| st.clone())
            } else {
                None
            }
        });
        if let Some(child) = nested_child {
            if pat_match_top(&effective_ty, "Vec") {
                panic!(
                    "flatten_struct_encode: `Vec<{}>` data-class field (`{}.{}`) is not \
                     supported by the fromParts flatten (variable arity)",
                    inner_ty.to_token_stream(),
                    s.ident,
                    fname
                );
            }
            if option_inner_type(&effective_ty).is_none() {
                let child_access = quote! { #access.#fname };
                let (child_pre, child_slots) = flatten_struct_encode(
                    ext,
                    registry,
                    &child,
                    &child_access,
                    &base,
                    depth + 1,
                    env_expr,
                )?;
                preludes.extend(child_pre);
                slots.extend(child_slots);
            } else {
                // `Option<nested>`: a `present` flag + the child's leaves,
                // encoded in the `Some` arm and defaulted in the `None` arm.
                let cbind = format_ident!("__c{}", depth);
                let child_access = quote! { #cbind };
                let (child_pre, child_slots) = flatten_struct_encode(
                    ext,
                    registry,
                    &child,
                    &child_access,
                    &base,
                    depth + 1,
                    env_expr,
                )?;
                let flag_id = format_ident!("__{}_present", base);
                let outer_ids: Vec<proc_macro2::Ident> = (0..child_slots.len())
                    .map(|i| format_ident!("__{}_o{}", base, i))
                    .collect();
                let outer_tys: Vec<TokenStream> =
                    child_slots.iter().map(|sl| sl.wire_ty.clone()).collect();
                let inner_ids: Vec<proc_macro2::Ident> =
                    child_slots.iter().map(|sl| sl.ident.clone()).collect();
                let defaults: Vec<TokenStream> =
                    child_slots.iter().map(|sl| sl.default.clone()).collect();
                preludes.extend(quote! {
                    let #flag_id: jni::sys::jboolean;
                    #( let #outer_ids: #outer_tys; )*
                    match &#access.#fname {
                        Some(#cbind) => {
                            #child_pre
                            #flag_id = 1u8;
                            #( #outer_ids = #inner_ids; )*
                        }
                        None => {
                            #flag_id = 0u8;
                            #( #outer_ids = #defaults; )*
                        }
                    }
                });
                slots.push(EncSlot {
                    ident: flag_id,
                    wire_ty: quote!(jni::sys::jboolean),
                    descriptor: "Z".to_string(),
                    is_object: false,
                    default: quote!(0u8),
                });
                for (i, sl) in child_slots.iter().enumerate() {
                    slots.push(EncSlot {
                        ident: outer_ids[i].clone(),
                        wire_ty: sl.wire_ty.clone(),
                        descriptor: sl.descriptor.clone(),
                        is_object: sl.is_object,
                        default: sl.default.clone(),
                    });
                }
            }
            continue;
        }
        // Leaf primitive / object (string, byte array, Vec, ...).
        match jni_field_access(&field_wire) {
            Some((sig, _, false)) => {
                preludes.extend(quote! { let #id: #field_wire = #value_expr; });
                slots.push(EncSlot {
                    ident: id,
                    wire_ty: quote!(#field_wire),
                    descriptor: sig.to_string(),
                    is_object: false,
                    default: primitive_default_for_descriptor(sig),
                });
            }
            Some((sig, _, true)) => {
                preludes.extend(quote! { let #id: jni::objects::JObject = #value_expr.into(); });
                slots.push(EncSlot {
                    ident: id,
                    wire_ty: quote!(jni::objects::JObject),
                    descriptor: sig.to_string(),
                    is_object: true,
                    default: quote!(jni::objects::JObject::null()),
                });
            }
            None => {
                // Object-shaped wire with no primitive descriptor; the JVM slot
                // must be the field's actual declared type (Option-stripped).
                let slot_ty =
                    option_inner_type(&effective_ty).unwrap_or_else(|| effective_ty.clone());
                let typed_slot = registry
                    .output_entry(&slot_ty)
                    .and_then(|e| jni_field_access(&e.destination))
                    .map(|(sig, _, _)| sig.to_string())
                    .or_else(|| {
                        bare_path_ident(&slot_ty).and_then(|name| {
                            ext.kotlin_fqn(&name.to_string())
                                .map(|v| format!("L{};", v.replace('.', "/")))
                        })
                    })
                    .or_else(|| {
                        if pat_match_top(&slot_ty, "Vec") {
                            Some("Ljava/util/List;".to_string())
                        } else if let syn::Type::Path(tp) = &field_wire {
                            tp.path.segments.last().and_then(|seg| {
                                match seg.ident.to_string().as_str() {
                                    "JString" => Some("Ljava/lang/String;".to_string()),
                                    "JByteArray" => Some("[B".to_string()),
                                    _ => None,
                                }
                            })
                        } else {
                            None
                        }
                    })
                    .unwrap_or_else(|| "Ljava/lang/Object;".to_string());
                preludes.extend(quote! { let #id: jni::objects::JObject = #value_expr; });
                slots.push(EncSlot {
                    ident: id,
                    wire_ty: quote!(jni::objects::JObject),
                    descriptor: typed_slot,
                    is_object: true,
                    default: quote!(jni::objects::JObject::null()),
                });
            }
        }
    }
    Some((preludes, slots))
}

/// If `arg_ty` is a registered **data_class** (not a handle / value class /
/// enum / external alias like `ZSample`), return its `ItemStruct` so a callback
/// arg of that type can be flattened into the `run(...)` signature
/// (`flatten_struct_encode`) instead of crossing as a built object. Returns
/// `None` for everything else (those keep their single-slot callback path).
pub(crate) fn callback_arg_data_class(
    ext: &JniGen,
    registry: &Registry<KotlinMeta>,
    arg_ty: &syn::Type,
) -> Option<syn::ItemStruct> {
    let name = bare_path_ident(arg_ty)?;
    if !registry.structs.contains_key(&name) {
        return None;
    }
    let is_vc = ext
        .types
        .get(&TypeKey::from_type(arg_ty))
        .map(|c| c.value_blob)
        .unwrap_or(false);
    if is_vc || ext.is_kotlin_enum(arg_ty) {
        return None;
    }
    registry.structs.get(&name).map(|(st, _)| st.clone())
}

pub(crate) fn struct_output_body(
    ext: &JniGen,
    s: &syn::ItemStruct,
    registry: &Registry<KotlinMeta>,
) -> Option<(syn::Type, syn::Expr)> {
    let struct_name = s.ident.to_string();
    // Prefer the registered Kotlin FQN (`io.zenoh.jni.JniSample`) so the
    // mangle closure flows through; fall back to the bare struct ident
    // qualified with the package when no `data_class` /
    // `ptr_class` declaration exists for this Rust type.
    let struct_ident = &s.ident;
    let struct_ty: syn::Type = syn::parse_quote!(#struct_ident);
    let registered_fqn = ext
        .types
        .get(&TypeKey::from_type(&struct_ty))
        .and_then(|cfg| cfg.kotlin_name.clone());
    let java_class_name = if let Some(fqn) = registered_fqn {
        fqn.replace('.', "/")
    } else if ext.java_class_prefix.is_empty() {
        struct_name.clone()
    } else {
        format!("{}/{}", ext.java_class_prefix, struct_name)
    };

    // Recursively flatten the whole object graph into leaf wires, then build it
    // with ONE `call_static_method("fromParts", …)` — no per-nested-struct JNI
    // crossing. The Kotlin `fromParts` factory (recursively flattened the same
    // way in `render_data_class_source`) reassembles the graph in bytecode.
    let access = quote!(v);
    let (preludes, slots) = flatten_struct_encode(ext, registry, s, &access, "", 0, &quote!(env))?;

    let mut sig = String::from("(");
    let mut args: Vec<TokenStream> = Vec::new();
    for sl in &slots {
        sig.push_str(&sl.descriptor);
        let id = &sl.ident;
        if sl.is_object {
            args.push(quote!(jni::objects::JValue::Object(&#id)));
        } else {
            args.push(quote!(jni::objects::JValue::from(#id)));
        }
    }
    sig.push_str(&format!(")L{};", java_class_name));
    let factory_sig_lit = syn::LitStr::new(&sig, Span::call_site());

    let body: syn::Expr = syn::parse_quote!({
        #preludes
        let __obj = env.call_static_method(
            #java_class_name,
            "fromParts",
            #factory_sig_lit,
            &[#(#args),*],
        )
        .and_then(|__v| __v.l())
        .map_err(|e| <__JniErr as ::core::convert::From<String>>::from(format!("encode struct via fromParts: {}", e)))?;
        __obj
    });
    Some((syn::parse_quote!(jni::objects::JObject), body))
}

pub(crate) fn struct_module_path(ext: &JniGen, s: &syn::ItemStruct) -> syn::Path {
    // Place the struct under <source_module>::<file_stem>::<Name>. Today's
    // pipeline derives the module from the source file stem; here we ride
    // on the same convention by inspecting the SourceLocation. Without a
    // location handy at this stage we fall back to <source_module>::<Name>.
    // In practice the actual file stem is added in the compose step at the
    // call site by the consuming crate when needed.
    let _ = s;
    ext.source_module.clone()
}

// ──────────────────────────────────────────────────────────────────────
// Enum rank-0 bodies
// ──────────────────────────────────────────────────────────────────────

/// `jint → Rust enum` decoder body for a `enum_class`-declared enum.
/// Wire is `jni::sys::jint`. The framework builds the decode `match`
/// directly from the enum's own discriminants — no `TryFrom<i32>` impl
/// is required on the flat enum (the enum declaration is the single
/// source of truth for the int↔variant mapping, shared with the Kotlin
/// `value(N)` constants via [`enum_discriminant_values`]). An unknown
/// discriminant surfaces as the framework `__JniErr`.
///
/// The arms use the bare ident — same shape as the wrapper function's
/// `v: <ident>` signature — so binding crates can pick whichever
/// upstream type a bare `<ident>` resolves to in their include-site
/// `use` statements. Pairs with output body below.
pub(crate) fn enum_input_body(_ext: &JniGen, e: &syn::ItemEnum) -> (syn::Type, syn::Expr) {
    assert_only_unit_variants(e);
    let ident = &e.ident;
    let ident_name = ident.to_string();
    let arms = crate::api::lang::jnigen::util::enum_discriminant_values(e)
        .into_iter()
        .map(|(variant, value)| {
            let lit = proc_macro2::Literal::i64_unsuffixed(value);
            quote! { #lit => #ident::#variant, }
        });
    let body: syn::Expr = syn::parse_quote!({
        match *v as i64 {
            #(#arms)*
            other => {
                return ::core::result::Result::Err(
                    <__JniErr as ::core::convert::From<String>>::from(
                        format!("invalid {} discriminant: {}", #ident_name, other)
                    )
                );
            }
        }
    });
    (syn::parse_quote!(jni::sys::jint), body)
}

/// `Rust enum → jint` encoder body for a `enum_class`-declared enum.
/// Wire is `jni::sys::jint`. Relies on the declared enum's repr
/// supporting an `as` cast (i.e. C-like enum, no fields); the
/// [`assert_only_unit_variants`] check below catches violations
/// upstream of the cast. The body works without naming the enum type
/// at all — `v` is already typed via the wrapper signature, so the
/// `as` cast picks up the right type by inference.
pub(crate) fn enum_output_body(_ext: &JniGen, e: &syn::ItemEnum) -> (syn::Type, syn::Expr) {
    assert_only_unit_variants(e);
    let body: syn::Expr = syn::parse_quote!({ v as jni::sys::jint });
    (syn::parse_quote!(jni::sys::jint), body)
}

/// Hard error on any enum that's not C-like (unit variants only).
/// `enum_class`'s discriminant-keyed Kotlin emission and `as jint`
/// encode both depend on unit variants — bail loudly at build time
/// rather than emitting wrong code.
pub(crate) fn assert_only_unit_variants(e: &syn::ItemEnum) {
    for variant in &e.variants {
        if !matches!(variant.fields, syn::Fields::Unit) {
            panic!(
                "enum_class only supports C-like enums (unit variants), \
                 but `{}::{}` has fields",
                e.ident, variant.ident
            );
        }
    }
}


/// If `ty` is a `&T` borrow with no explicit lifetime, splice in `'<life>`.
/// Otherwise return `ty` unchanged.
pub(crate) fn annotate_borrow_with_lifetime(ty: &syn::Type, life: &str) -> syn::Type {
    if let syn::Type::Reference(r) = ty {
        if r.lifetime.is_none() {
            let mut new = r.clone();
            new.lifetime = Some(syn::Lifetime::new(
                &format!("'{}", life),
                proc_macro2::Span::call_site(),
            ));
            return syn::Type::Reference(new);
        }
    }
    ty.clone()
}

/// If `ty` is `JObject` / `JString` / `JByteArray` (no explicit angle args),
/// splice in `<'<life>>`. Otherwise return `ty` unchanged.
pub(crate) fn annotate_jobject_with_lifetime(ty: &syn::Type, life: &str) -> syn::Type {
    if let syn::Type::Path(tp) = ty {
        if let Some(last) = tp.path.segments.last() {
            let name = last.ident.to_string();
            if matches!(
                name.as_str(),
                "JObject" | "JString" | "JByteArray" | "JClass"
            ) {
                if matches!(last.arguments, syn::PathArguments::None) {
                    let mut new = tp.clone();
                    if let Some(last) = new.path.segments.last_mut() {
                        let lt = syn::Lifetime::new(
                            &format!("'{}", life),
                            proc_macro2::Span::call_site(),
                        );
                        last.arguments = syn::PathArguments::AngleBracketed(
                            syn::AngleBracketedGenericArguments {
                                colon2_token: None,
                                lt_token: syn::token::Lt::default(),
                                args: syn::punctuated::Punctuated::from_iter(std::iter::once(
                                    syn::GenericArgument::Lifetime(lt),
                                )),
                                gt_token: syn::token::Gt::default(),
                            },
                        );
                    }
                    return syn::Type::Path(new);
                }
            }
        }
    }
    ty.clone()
}

// ──────────────────────────────────────────────────────────────────────
// Helpers
// ──────────────────────────────────────────────────────────────────────

pub(crate) fn pat_match(ty: &syn::Type, pat: &str) -> bool {
    ty.to_token_stream().to_string() == pat
}

/// `true` if `ty` is a path whose final segment is `name` (e.g. `Vec<_>` for
/// `name = "Vec"`, `Option<&T>` for `name = "Option"`). Ignores generic args.
pub(crate) fn pat_match_top(ty: &syn::Type, name: &str) -> bool {
    if let syn::Type::Path(tp) = ty {
        if let Some(last) = tp.path.segments.last() {
            return last.ident == name;
        }
    }
    false
}

/// If `ty` is `Option<&T>` or `Option<&mut T>`, return `Some(is_mut)`.
/// Returns `None` for any other shape. Used by `emit_jni_function_wrapper`
/// to decide whether the call site needs `.as_deref()` / `.as_deref_mut()`
/// when the input converter produced `Option<OwnedObject<T>>`.
pub(crate) fn option_inner_ref_mutability(ty: &syn::Type) -> Option<bool> {
    let syn::Type::Path(tp) = ty else { return None };
    let seg = tp.path.segments.last()?;
    if seg.ident != "Option" {
        return None;
    }
    let syn::PathArguments::AngleBracketed(ab) = &seg.arguments else {
        return None;
    };
    let syn::GenericArgument::Type(inner) = ab.args.first()? else {
        return None;
    };
    let syn::Type::Reference(r) = inner else {
        return None;
    };
    Some(r.mutability.is_some())
}

pub(crate) fn bare_path_ident(ty: &syn::Type) -> Option<syn::Ident> {
    if let syn::Type::Path(tp) = ty {
        if let Some(last) = tp.path.segments.last() {
            if matches!(last.arguments, syn::PathArguments::None) {
                return Some(last.ident.clone());
            }
        }
    }
    None
}

/// If `ty` is `Option<Inner>`, return `Inner`. Used by the struct encoder to
/// derive the JVM ctor slot descriptor of an optional field: the value is
/// encoded as a nullable JObject, but the Kotlin constructor expects `Inner`'s
/// concrete erased type, not `Ljava/lang/Object;`.
pub(crate) fn option_inner_type(ty: &syn::Type) -> Option<syn::Type> {
    let syn::Type::Path(tp) = ty else { return None };
    let seg = tp.path.segments.last()?;
    if seg.ident != "Option" {
        return None;
    }
    let syn::PathArguments::AngleBracketed(ab) = &seg.arguments else {
        return None;
    };
    match ab.args.first()? {
        syn::GenericArgument::Type(inner) => Some(inner.clone()),
        _ => None,
    }
}

/// Inline-class field name for a value projection identified by its folded
/// [`Projection::leaf_key`] (e.g. `"ZZenohId"`) rather than by a raw param type.
/// Used for `Option<value-blob>` params where the written type isn't the bare
/// value class but the projection still resolves the leaf — so the wrapper
/// knows which inline field to unwrap (`<name>.bytes`).
pub(crate) fn value_projection_field_for_leaf(ext: &JniGen, leaf_key: &str) -> Option<String> {
    let key = TypeKey::parse(leaf_key);
    let cfg = ext.types.get(&key)?;
    if cfg.value_blob {
        return Some("bytes".to_string());
    }
    None
}

/// Decide which [`NullableKind`] to fold for an `Option<_>` wrapper, given
/// the wrapper's destination wire and the registry-resolved inner. The
/// detection mirrors the two paths in [`option_input`] / [`option_output`]:
/// the niche path keeps the inner's wire untouched (e.g. `jlong` stays
/// `jlong`, `JByteArray` stays `JByteArray`), while the boxed-primitive
/// fallback widens the wire to `JObject`. So `outer_wire == inner.destination`
/// uniquely identifies the niche path.
///
/// Symmetric `_input` / `_output` flavors only differ in which registry side
/// they consult — the comparison is identical.
pub(crate) fn nullable_kind_for(
    outer_wire: &syn::Type,
    inner_ty: &syn::Type,
    registry: &Registry<KotlinMeta>,
) -> NullableKind {
    let inner_dest = registry
        .input_entry(inner_ty)
        .map(|e| e.destination.clone())
        .expect(
            "nullable_kind_for: Option<_> input handler reached here only after option_input \
             returned Some, so the inner's input entry must exist",
        );
    if outer_wire == &inner_dest {
        NullableKind::Niche
    } else {
        NullableKind::Boxed
    }
}

pub(crate) fn nullable_kind_for_output(
    outer_wire: &syn::Type,
    inner_ty: &syn::Type,
    registry: &Registry<KotlinMeta>,
) -> NullableKind {
    let inner_dest = registry
        .output_entry(inner_ty)
        .map(|e| e.destination.clone())
        .expect(
            "nullable_kind_for_output: Option<_> output handler reached here only after \
             option_output returned Some, so the inner's output entry must exist",
        );
    if outer_wire == &inner_dest {
        NullableKind::Niche
    } else {
        NullableKind::Boxed
    }
}

// ──────────────────────────────────────────────────────────────────────
// JNI-internal naming convention. Hand-written code in zenoh-jni
// (e.g. liveliness.rs, advanced_subscriber.rs) calls auto-generated
// converters by these computed names — so the convention is part of the
// JNI plugin's public contract, not a private implementation detail.
// ──────────────────────────────────────────────────────────────────────

/// INPUT: wire → rust. Format `<wire_id>_to_<rust_id>_<hash>`. Special
/// case: `impl Fn(...)` keeps the legacy `process_kotlin_<Name>_callback`
/// name so existing hand-written call sites continue to resolve. With
/// the current [`derive_callback_name`] algorithm `<Name>` is
/// concatenated arg shorts + `"Callback"` (e.g. `process_kotlin_SampleCallback_callback`).
pub(crate) fn input_name(rust: &syn::Type, wire: &syn::Type) -> syn::Ident {
    if let Some(args) = extract_fn_trait_args(rust) {
        let name = derive_callback_name(&args);
        let s = format!("process_kotlin_{}_callback", name);
        return syn::Ident::new(&s, Span::call_site());
    }
    let rust_id = sanitize_for_ident(&rust.to_token_stream().to_string());
    let wire_id = wire_short(wire);
    let h = hash_pair(rust, wire);
    let s = format!("{}_to_{}_{:08x}", wire_id, rust_id, h & 0xffff_ffff);
    syn::Ident::new(&s, Span::call_site())
}

/// OUTPUT: rust → wire. Format `<rust_id>_to_<wire_id>_<hash>`.
pub(crate) fn output_name(rust: &syn::Type, wire: &syn::Type) -> syn::Ident {
    let rust_id = sanitize_for_ident(&rust.to_token_stream().to_string());
    let wire_id = wire_short(wire);
    let h = hash_pair(rust, wire);
    let s = format!("{}_to_{}_{:08x}", rust_id, wire_id, h & 0xffff_ffff);
    syn::Ident::new(&s, Span::call_site())
}

pub(crate) fn sanitize_for_ident(s: &str) -> String {
    // Special-case the empty tuple — the all-punctuation token stream
    // would sanitize to a meaningless fallback. `unit` is recognisable.
    if s.trim() == "()" {
        return "unit".to_string();
    }
    let mut out = String::with_capacity(s.len());
    let mut prev_underscore = false;
    for c in s.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c);
            prev_underscore = false;
        } else if !prev_underscore {
            out.push('_');
            prev_underscore = true;
        }
    }
    while out.starts_with('_') {
        out.remove(0);
    }
    while out.ends_with('_') {
        out.pop();
    }
    if out.is_empty() {
        out.push_str("ty");
    }
    if out.chars().next().is_some_and(|c| c.is_ascii_digit()) {
        out.insert(0, '_');
    }
    out
}

pub(crate) fn wire_short(wire: &syn::Type) -> String {
    if let syn::Type::Path(tp) = wire {
        if let Some(last) = tp.path.segments.last() {
            return sanitize_for_ident(&last.ident.to_string());
        }
    }
    sanitize_for_ident(&wire.to_token_stream().to_string())
}

pub(crate) fn hash_pair(rust: &syn::Type, wire: &syn::Type) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut h = DefaultHasher::new();
    rust.to_token_stream().to_string().hash(&mut h);
    "::".hash(&mut h);
    wire.to_token_stream().to_string().hash(&mut h);
    h.finish()
}

/// Reconstruct the `impl Fn(args...) + Send + Sync + 'static` syn::Type
/// from a flat slice of arg types. Used by the rank-1/2/3 callback impls
/// to feed `input_wrapper` the original outer type.
pub(crate) fn build_fn_type(args: &[syn::Type]) -> syn::Type {
    let arg_iter = args.iter();
    syn::parse_quote!(impl Fn( #(#arg_iter),* ) + Send + Sync + 'static)
}

/// `OwnedObject<T>` definition emitted into the destination Rust file.
///
/// A non-owning borrow wrapper around a `*const T` whose backing
/// `Box<T>` lives on the Java side. The Java side hands Rust the
/// pointer under its `NativeHandle.withPtr` read lock; for the
/// duration of the JNI call the heap allocation is guaranteed live,
/// so `Deref<Target = T>` exposing `&*ptr` is sound. The wrapper has
/// no `Drop`: nothing is freed here, the Box stays with Java.
///
/// By-value `T` extraction is intentionally NOT through this wrapper.
/// Consume call sites use `*Box::from_raw(ptr)` inline, taking
/// ownership of Java's slot; `NativeHandle.consume` (write-lock +
/// atomic null) sequences that against any concurrent borrow.
///
/// Co-locating the definition with the converters keeps the generated
/// file self-contained — no `use` statement or runtime-support module
/// is required from the host crate.
pub(crate) fn owned_object_prerequisite_items() -> Vec<syn::Item> {
    vec![
        syn::parse_quote!(
            /// See module-level docs at [`owned_object_prerequisite_items`].
            #[allow(dead_code)]
            pub(crate) struct OwnedObject<T: ?Sized> {
                ptr: *const T,
            }
        ),
        syn::parse_quote!(
            impl<T: ?Sized> std::ops::Deref for OwnedObject<T> {
                type Target = T;
                fn deref(&self) -> &Self::Target {
                    unsafe { &*self.ptr }
                }
            }
        ),
        syn::parse_quote!(
            // `&mut OwnedObject<T>` coerces to `&mut T` via this impl,
            // letting source fns that take `&mut T` opaque-handle params
            // be called from generated wrappers. The pointer originated
            // from `Box::into_raw` (which produces `*mut T`); the
            // `*const T → *mut T` cast just restores the original
            // mutability. Sequencing against concurrent borrow / consume
            // is upheld by `NativeHandle.withPtr` on the JVM side, same
            // as `Deref`.
            impl<T: ?Sized> std::ops::DerefMut for OwnedObject<T> {
                fn deref_mut(&mut self) -> &mut Self::Target {
                    unsafe { &mut *(self.ptr as *mut T) }
                }
            }
        ),
        syn::parse_quote!(
            impl<T: ?Sized> OwnedObject<T> {
                /// Borrow a `T` whose backing `Box<T>` lives on the
                /// Java side. Stores only the pointer; the wrapper
                /// does not own the heap allocation and never frees
                /// it on drop.
                ///
                /// # Safety
                ///
                /// `ptr` must be the result of an earlier
                /// `Box::into_raw(Box::new(v))` and the allocation
                /// must still be live (Java still owns it). The Java
                /// side is responsible for sequencing this call
                /// against any concurrent free or consume (via
                /// `NativeHandle.withPtr` read-lock vs `consume` /
                /// `close` write-lock) so the borrow cannot race a
                /// deallocation on the same pointer.
                #[allow(dead_code)]
                pub(crate) unsafe fn from_raw(ptr: *const T) -> Self {
                    Self { ptr }
                }
            }
        ),
    ]
}

// ──────────────────────────────────────────────────────────────────────
// Tests
// ──────────────────────────────────────────────────────────────────────
//
// These tests exercise the niche cascade by hand-building registry
// entries with deliberate niche shapes, then driving `option_input` /
// `option_output` directly. They mirror the documented `Niches`
// semantics: each `Option<_>` layer carves one slot and re-exports the
// rest; once the rest is exhausted, the next layer falls back to the
// boxed-Java-primitive scheme.

