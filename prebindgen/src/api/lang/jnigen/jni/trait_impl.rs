//! [`Prebindgen`] implementation for [`JniGen`] plus its converter-
//! selector / exception-routing helpers.
//!
//! Carved from the former monolithic JNI module; shares the `jni`
//! namespace via `use super::*`.

use super::*;


// ──────────────────────────────────────────────────────────────────────
// Inherent helpers — wrapper builders (used by both Prebindgen impl
// and consuming-crate wrapper exts like ZenohJniExt).
// ──────────────────────────────────────────────────────────────────────

impl JniGen {
    /// Build the standard JNI input-converter `fn`. Body assumes in-scope
    /// `env: &mut JNIEnv` and `v: &<wire>` (or `v: <wire>` for raw-pointer
    /// wires); produces a value of `rust`. Returned function has its name
    /// already set per the JNI plugin's naming convention.
    ///
    /// `exc` ties the body convention to the bound exception:
    /// * `None` (non-throwing) → signature `Result<rust, __JniErr>` and
    ///   the body is wrapped `Ok(<body>)`; `?` inside propagates the
    ///   framework error.
    /// * `Some(X)` (throwing) → signature `Result<rust, X::rust_type>`
    ///   and the body is emitted as-is — `<body>` already evaluates to
    ///   that `Result`, so no `Ok` wrap (and no cross-type `From`).
    pub(crate) fn build_input_fn(
        &self,
        rust: &syn::Type,
        wire: &syn::Type,
        body: &syn::Expr,
        exc: Option<&ExceptionConfig>,
    ) -> syn::ItemFn {
        let name = input_name(rust, wire);
        let rust_with_lifetime = annotate_borrow_with_lifetime(rust, "env");
        let wire_with_lifetime = annotate_jobject_with_lifetime(wire, "v");
        let err_type = exc
            .map(|e| e.rust_type.clone())
            .unwrap_or_else(default_err_type);
        let ret_body = body_for_exc(body, exc);
        if matches!(wire, syn::Type::Ptr(_)) {
            syn::parse_quote!(
                #[allow(non_snake_case, unused_mut, unused_variables, unused_braces, dead_code)]
                pub(crate) unsafe fn #name<'env>(env: &mut jni::JNIEnv<'env>, v: #wire) -> ::core::result::Result<#rust_with_lifetime, #err_type> {
                    #ret_body
                }
            )
        } else {
            syn::parse_quote!(
                #[allow(non_snake_case, unused_mut, unused_variables, unused_braces, dead_code)]
                pub(crate) unsafe fn #name<'env, 'v>(env: &mut jni::JNIEnv<'env>, v: &#wire_with_lifetime) -> ::core::result::Result<#rust_with_lifetime, #err_type> {
                    #ret_body
                }
            )
        }
    }

    /// Build the standard JNI output-converter `fn`. Body assumes in-scope
    /// `env: &mut JNIEnv` and `v: <rust>` (by value — handles like
    /// `Subscriber<()>` aren't `Clone`, so callers move into the converter).
    ///
    /// `exc` — see [`Self::build_input_fn`]; same body↔exception coupling,
    /// output side.
    pub(crate) fn build_output_fn(
        &self,
        rust: &syn::Type,
        wire: &syn::Type,
        body: &syn::Expr,
        exc: Option<&ExceptionConfig>,
    ) -> syn::ItemFn {
        let name = output_name(rust, wire);
        let wire_with_lifetime = annotate_jobject_with_lifetime(wire, "a");
        let err_type = exc
            .map(|e| e.rust_type.clone())
            .unwrap_or_else(default_err_type);
        let ret_body = body_for_exc(body, exc);
        syn::parse_quote!(
            #[allow(non_snake_case, unused_mut, unused_variables, unused_braces, dead_code)]
            pub(crate) unsafe fn #name<'a>(env: &mut jni::JNIEnv<'a>, v: #rust) -> ::core::result::Result<#wire_with_lifetime, #err_type> {
                #ret_body
            }
        )
    }

    /// Universal "opaque Box-handle as `jlong`" pair — input side.
    ///
    /// Use for any Rust type whose lifecycle is owned by the Java side:
    /// Java holds the raw `Box<T>` pointer as a `Long` and calls Rust
    /// passing the pointer. The converter handles both parameter
    /// shapes, the decision is taken in `on_function` from the
    /// parameter's syntax:
    ///
    /// **`&T` sites (borrow)**: `OwnedObject::from_raw` stores the
    /// pointer without taking ownership of the `Box`; `Deref<Target
    /// = T>` exposes `&*ptr` so the generated call site can borrow it
    /// as `&T`. The wrapper has no `Drop` — nothing is freed, the
    /// heap allocation stays with Java. The Java side must take the
    /// pointer out of its `NativeHandle.withPtr` (read lock) so the
    /// borrow is sequenced against any concurrent consume / close.
    ///
    /// **`T` sites (consume, by-value)**: the call-site emitter
    /// bypasses `OwnedObject` and inlines `*Box::from_raw(ptr)` —
    /// infallible. The Java side must take the pointer out of its
    /// `NativeHandle.consume` (write lock + atomic null) before
    /// invoking this entry point; that write lock drains concurrent
    /// borrows and the atomic-null ensures the same Long cannot be
    /// passed twice. No `T: Clone` bound (Box requires nothing of T),
    /// so non-Clone handles (`Publisher<'a>`, `Subscriber<()>`) can
    /// consume.
    ///
    /// **Convention** (single rule for both input and output):
    /// * Wire: `jni::sys::jlong` — the same width JNI hands across
    ///   the boundary on every platform (`*mut T` would mismatch
    ///   on 32-bit, where ptr size is 4 but jlong is 8).
    /// * Output: `Box::into_raw(Box::new(v)) as i64` — leak the heap
    ///   allocation to Java; sole owner is whoever later calls
    ///   `Box::from_raw` on the same pointer.
    /// * Input: `OwnedObject::from_raw(*v as *const T)` (borrow only).
    /// * Niche: `0i64` / `*v == 0` — `Box::into_raw` never returns 0,
    ///   so `Option<T>` automatically synthesises `0` = `None`,
    ///   matching the legacy "null pointer" ABI for nullable handles.
    pub fn opaque_handle_input(&self, ty: &syn::Type) -> ConverterImpl<KotlinMeta> {
        let wire: syn::Type = syn::parse_quote!(jni::sys::jlong);
        let name = input_name(ty, &wire);
        let function: syn::ItemFn = syn::parse_quote!(
            #[allow(non_snake_case, unused_mut, unused_variables, unused_braces, dead_code)]
            pub(crate) unsafe fn #name<'env, 'v>(
                env: &mut jni::JNIEnv<'env>,
                v: &jni::sys::jlong,
            ) -> ::core::result::Result<OwnedObject<#ty>, __JniErr> {
                Ok(unsafe { OwnedObject::from_raw(*v as *const #ty) })
            }
        );
        ConverterImpl {
            function,
            destination: wire,
            pre_stages: vec![],
            niches: Niches::one(syn::parse_quote!(0i64), syn::parse_quote!(*v == 0)),
            // Opaque handles' value-context Kotlin name stays `"Long"`
            // (the jlong wire mention); the *typed* Kotlin rendering is
            // derived from `handle` below. The wrapper's `?` path surfaces
            // an `OwnedObject::from_raw` failure as the framework
            // `JniBindingError`, so the throws fields point at the
            // framework exception.
            metadata: self.opaque_leaf_meta(ty),
        }
    }

    /// Leaf metadata for an opaque handle: value-context name `"Long"`
    /// plus the [`Projection`] that folds outward through wrappers (owned,
    /// [`FoldStrategy::Direct`]). The single seam where a Rust type is
    /// first marked a closeable native handle.
    fn opaque_leaf_meta(&self, ty: &syn::Type) -> KotlinMeta {
        KotlinMeta {
            projection: Some(Projection {
                leaf_key: TypeKey::from_type(ty).as_str().to_string(),
                owned: true,
                strategy: FoldStrategy::Direct,
                kind: ProjectionKind::Handle,
            }),
            ..self.framework_meta(Some("Long".to_string()))
        }
    }

    /// If the user pinned a Kotlin name for `outer_ty` via
    /// [`Self::data_class`] (or it's an opaque-handle entry that
    /// kept its FQN in `kotlin_name`), use that name; otherwise leave
    /// the auto-derived `inherited` value untouched. Lets handler arms
    /// inherit by default but yield to an explicit user pin when one
    /// exists — same precedence the legacy `KotlinTypeMap.lookup`
    /// fallback chain had.
    pub(crate) fn override_kotlin_name(
        &self,
        outer_ty: &syn::Type,
        inherited: Option<String>,
    ) -> Option<String> {
        let key = TypeKey::from_type(outer_ty);
        if let Some(cfg) = self.types.get(&key) {
            // Opaque-handle entries keep their typed FQN in
            // `kotlin_name` for FQN-consumers, but the value-context
            // name is `"Long"` (set on the rank-0 handler's metadata).
            // Don't let that FQN leak into a wrapper's metadata.
            if cfg.opaque.is_none() {
                if let Some(name) = &cfg.kotlin_name {
                    return Some(name.clone());
                }
            }
        }
        inherited
    }

    /// Auto-derived Kotlin FQN for an `impl Fn(args)` callback. Same
    /// convention `collect_kotlin_callback_fqns` uses, exposed here so
    /// the rank-0/rank-1 callback dispatcher can stamp the FQN into
    /// the converter's [`KotlinMeta`] at creation time. The relative
    /// class name passes through [`Self::mangle_callback`] before
    /// being qualified against
    /// [`Self::kotlin_callback_package`].
    pub(crate) fn auto_callback_fqn(&self, args: &[syn::Type]) -> String {
        let name = derive_callback_name(args);
        self.resolve_callback_fqn(&self.mangle_callback(&name))
    }

    /// Canonical input-converter name for `(rust, wire)` — exposed
    /// for plugin wrapper exts that build `ConverterImpl::function`
    /// manually with a non-standard return type (e.g.
    /// `impl Into<…>` parameters that can't be expressed via
    /// [`Self::input_wrapper`]'s fixed signature shape).
    pub fn input_converter_name(&self, rust: &syn::Type, wire: &syn::Type) -> syn::Ident {
        input_name(rust, wire)
    }

    /// Symmetric to [`Self::input_converter_name`].
    pub fn output_converter_name(&self, rust: &syn::Type, wire: &syn::Type) -> syn::Ident {
        output_name(rust, wire)
    }

    fn emitted_source_type_names(&self) -> std::collections::HashSet<String> {
        let mut names = std::collections::HashSet::new();
        for key in self.types.keys() {
            if let Some(short) = rust_short_name_opt(key) {
                names.insert(short);
            }
        }
        for exc in self.exceptions.iter().skip(1) {
            if let Some(short) = type_last_ident(&exc.rust_type) {
                names.insert(short.to_string());
            }
        }
        names
    }

    /// Walk `item` and prefix every bare single-segment type reference
    /// matching a [`Self::emitted_source_type_names`] name with
    /// [`Self::source_module`]. Applied once per emitted item at write
    /// time via [`Prebindgen::post_process_item`] so converter bodies,
    /// type ascriptions, and casts all stay in sync without each emit
    /// site having to remember to qualify.
    fn qualify_item(&self, item: &mut syn::Item) {
        let source_names = self.emitted_source_type_names();
        if source_names.is_empty() {
            return;
        }
        let mut visitor = QualifyEmittedTypes {
            source_module: &self.source_module,
            source_names: &source_names,
        };
        syn::visit_mut::VisitMut::visit_item_mut(&mut visitor, item);
    }

    /// Output side of [`Self::opaque_handle_input`] — see that method's
    /// docs for the full convention.
    pub fn opaque_handle_output(&self, ty: &syn::Type) -> ConverterImpl<KotlinMeta> {
        let wire: syn::Type = syn::parse_quote!(jni::sys::jlong);
        let body: syn::Expr =
            syn::parse_quote!(std::boxed::Box::into_raw(std::boxed::Box::new(v)) as i64);
        ConverterImpl {
            function: self.build_output_fn(ty, &wire, &body, None),
            destination: wire,
            pre_stages: vec![],
            niches: Niches::one(syn::parse_quote!(0i64), syn::parse_quote!(*v == 0)),
            // Opaque handles' value-context name `"Long"` + folded
            // `Projection` — see [`Self::opaque_handle_input`] /
            // [`Self::opaque_leaf_meta`]. Framework throws because the
            // wrapper's emitted match-arm still has a `JniBindingError`
            // branch reachable via the chain.
            metadata: self.opaque_leaf_meta(ty),
        }
    }

    /// Emit the JObject-typed dispatching input converter for
    /// `impl Into<target> + Send + 'static` given an already-assembled
    /// source list. The caller — typically a
    /// [`Prebindgen::dispatch_into_input`] implementation —
    /// supplies every arm explicitly (including the identity arm
    /// `target → target` if wanted) with each source's borrow/consume
    /// mode.
    ///
    /// Emits an `instanceof` chain over each source `S`: every arm
    /// calls `S`'s already-registered input decoder (wire-narrowed
    /// from the parameter's `JObject`) and converts to `target` via
    /// `TryInto`, so both `From<S> for target` (zero-cost) and
    /// `TryFrom<S> for target` (fallible) work uniformly.
    ///
    /// Per-source mode handling (only relevant for opaque sources —
    /// non-opaque sources have no `Box` slot, so mode is moot):
    /// * [`IntoSourceMode::Borrow`] → decode via
    ///   `OwnedObject::from_raw(...).clone()`. Java's `Box` slot stays
    ///   live; requires `T: Clone`.
    /// * [`IntoSourceMode::Consume`] → bypass `OwnedObject` and inline
    ///   `*Box::from_raw(ptr as *mut T)`. Java's `Box` slot is taken;
    ///   the caller's typed handle must be invalidated (the Kotlin
    ///   wrapper does this via `NativeHandle.consume`). No `T: Clone`
    ///   bound.
    ///
    /// Returns `None` when `sources` is empty or any source lacks a
    /// registered input decoder; the resolver iterates to a fixed
    /// point and will retry on a later round once all decoders exist.
    pub fn emit_into_dispatcher(
        &self,
        target: &syn::Type,
        sources: &[IntoSource],
        registry: &Registry<KotlinMeta>,
    ) -> Option<ConverterImpl<KotlinMeta>> {
        if sources.is_empty() {
            return None;
        }
        let target_key = TypeKey::from_type(target).as_str().to_string();

        // Single-source deterministic path: with exactly one declared source
        // there is nothing to *select* at runtime, so we skip the
        // `find_class` + `is_instance_of` chain entirely. The param decodes
        // the one statically-known source directly and (when the source isn't
        // already the target) converts via `TryInto`. The returned converter
        // carries the SOURCE's real `kotlin_name` + `projection`, so
        // `render_wrapper_fn` classifies the param as an ordinary typed /
        // handle / value param (Borrow/Consume/ValueUnwrap/PassThrough) — no
        // `Any`, no `instanceof`. Multi-source (`len > 1`) keeps the dispatch
        // chain below.
        if sources.len() == 1 {
            let src_ty = &sources[0].source_type;
            let src_entry = registry.input_entry(src_ty)?;
            // Identity (`S == target`): alias the target's own input
            // converter verbatim — its function already yields `target`.
            if TypeKey::from_type(src_ty) == TypeKey::from_type(target) {
                return Some(ConverterImpl {
                    function: src_entry.function.clone(),
                    destination: src_entry.destination.clone(),
                    pre_stages: src_entry.pre_stages.clone(),
                    niches: src_entry.niches.clone(),
                    metadata: src_entry.metadata.clone(),
                });
            }
            // Non-identity: decode `S` (wire-facing `function`), then run a
            // `TryInto::<target>` stage. Mirrors the composed-converter shape
            // in `lookup_input` (stage first, then the source's own stages).
            let body: syn::Expr = syn::parse_quote!({
                ::core::convert::TryInto::try_into(v).map_err(|e| {
                    <__JniErr as ::core::convert::From<String>>::from(format!(
                        "impl Into conversion failed: {}",
                        e
                    ))
                })?
            });
            let stage = Stage {
                function: self.build_output_fn(src_ty, target, &body, None),
                metadata: KotlinMeta::default(),
            };
            let mut pre_stages = vec![stage];
            pre_stages.extend(src_entry.pre_stages.iter().cloned());
            return Some(ConverterImpl {
                function: src_entry.function.clone(),
                destination: src_entry.destination.clone(),
                pre_stages,
                niches: src_entry.niches.clone(),
                metadata: src_entry.metadata.clone(),
            });
        }

        let mut arms: Vec<TokenStream> = Vec::with_capacity(sources.len());
        for src in sources {
            let src_ty = &src.source_type;
            let src_key = TypeKey::from_type(src_ty).as_str().to_string();
            let src_entry = registry.input_entry(src_ty)?;
            let decoder = src_entry.function.sig.ident.clone();
            let wire = src_entry.destination.clone();
            let (java_class, prelude, decoded_ref) =
                jobject_to_wire_adapter(&wire, src_ty, &self.kotlin_type_fqns).unwrap_or_else(
                    || {
                        panic!(
                            "emit_into_dispatcher: source `{}` has wire `{}` which is not a \
                             supported Into-source wire shape (target = `{}`)",
                            src_key,
                            wire.to_token_stream(),
                            target_key
                        )
                    },
                );
            // Opaque sources branch on the declared mode. Non-opaque
            // sources don't own a `Box` slot, so they just decode
            // normally and `mode` has no effect on the emitted code.
            let is_opaque = src_entry.metadata.is_direct_handle();
            let decode_expr: syn::Expr = if is_opaque {
                match src.mode {
                    // Method-call `.clone()` triggers method auto-deref:
                    // OwnedObject<T> has no Clone impl, so dispatch
                    // derefs to `&T` and calls `T::clone`. Requires
                    // `T: Clone`. Java's `Box` slot stays live.
                    IntoSourceMode::Borrow => syn::parse_quote!(
                        unsafe { #decoder(env, #decoded_ref)? }.clone()
                    ),
                    // Bypass the decoder entirely: reconstruct the
                    // unique `Box<T>` from Java's pointer and move `T`
                    // out, freeing the heap allocation. Mirrors the
                    // direct-by-value consume codegen at
                    // `emit_jni_function_wrapper`. Unique-ownership
                    // invariant is upheld by `NativeHandle.consume`
                    // (write lock + atomic null) on the Kotlin side.
                    // `#decoded_ref` is `&__narrowed` for jlong wires;
                    // dereference to recover the `jlong` value.
                    IntoSourceMode::Consume => syn::parse_quote!(
                        unsafe { *std::boxed::Box::from_raw(*#decoded_ref as *mut #src_ty) }
                    ),
                }
            } else {
                syn::parse_quote!(unsafe { #decoder(env, #decoded_ref)? })
            };
            arms.push(quote! {
                {
                    let __class = env
                        .find_class(#java_class)
                        .map_err(|e| <__JniErr as ::core::convert::From<String>>::from(format!("find {}: {}", #java_class, e)))?;
                    let __is = env
                        .is_instance_of(v, &__class)
                        .map_err(|e| <__JniErr as ::core::convert::From<String>>::from(format!("instanceof {}: {}", #java_class, e)))?;
                    if __is {
                        #prelude
                        let __decoded: #src_ty = #decode_expr;
                        let __converted: #target = ::core::convert::TryInto::try_into(__decoded)
                            .map_err(|e| <__JniErr as ::core::convert::From<String>>::from(format!(
                                "convert {} -> {}: {}", #src_key, #target_key, e)))?;
                        return Ok(__converted);
                    }
                }
            });
        }

        let wire: syn::Type = syn::parse_quote!(jni::objects::JObject);
        let pat: syn::Type = syn::parse_quote!(impl Into<#target> + Send + 'static);
        let name = input_name(&pat, &wire);
        let target_label = target_key.clone();
        let function: syn::ItemFn = syn::parse_quote!(
            #[allow(non_snake_case, unused_mut, unused_variables, unused_braces, dead_code)]
            pub(crate) unsafe fn #name<'env, 'v>(
                env: &mut jni::JNIEnv<'env>,
                v: &jni::objects::JObject<'v>,
            ) -> ::core::result::Result<#target, __JniErr> {
                #(#arms)*
                Err(<__JniErr as ::core::convert::From<String>>::from(format!(
                    "impl Into<{}>: no matching source arm for runtime class", #target_label)))
            }
        );

        // JObject wire carries a genuine `null` value that no live
        // source-arm decode ever produces — expose it as a niche so an
        // outer `Option<impl Into<T>>` can carve it (null = None) and
        // stay on the JObject wire (no boxing).
        let niches = default_niches_for_wire(&wire);
        Some(ConverterImpl {
            function,
            destination: wire,
            pre_stages: vec![],
            niches,
            // `impl Into<T>` parameters surface as Kotlin `Any` — the
            // safe wrapper does an `is JNI<X>` chain on the value, and
            // the JNI dispatcher's matching arm uses each source's
            // typed FQN under the hood. The dispatcher's per-arm `?`
            // decode + no-match `Err` fallthrough can fail, so it
            // carries the framework throws.
            metadata: self.framework_meta(Some("Any".to_string())),
        })
    }
}

/// One `pub(crate) fn throw_<short>(...)` item for an exception.
/// Emitted from [`Prebindgen::prerequisites`] so it lands at the
/// top of the same generated file as every other converter — wrapper
/// code below can call it by bare name (`throw_<short>(env, &err)`);
/// hand-written modules in the binding crate reach it via the include
/// module path (e.g. `crate::generated::throw_<short>`). The body
/// finds the JVM class by slash-form FQN and `throw_new`s with
/// `err.to_string()`, logging on either failure.
///
/// The error parameter is generic over `Display` rather than the
/// exception's own Rust type. This decouples the *thrown JVM class*
/// from the *Rust error value*: the unified converter error type
/// (`__JniErr`, the binding's primary domain error) flows through
/// every converter, but each converter chooses which `throw_<short>`
/// to call — so a built-in decode failure carries the domain error
/// value yet surfaces on the JVM as `JniBindingError`. (It also avoids
/// any cross-crate `From` bridge between the framework error type and
/// the domain error type, which the crate layering forbids.)
pub(crate) fn build_throw_fn_item(
    ext: &JniGen,
    registry: &Registry<KotlinMeta>,
    exc: &ExceptionConfig,
) -> syn::Item {
    let throw_fn = &exc.throw_fn_name;
    let class_path_slashes = exc.kotlin_fqn.replace('.', "/");
    // Structured path: when the exception's Rust type has its own
    // registered output converter (i.e. `.data_class(...).throwable()`),
    // construct the JVM object via that converter and throw it as the
    // type's own JVM class — so a structured error carries its fields
    // across the boundary, not just `Display::to_string`. Requires the
    // type to be `Clone` (the converter consumes `v` by value).
    let key = TypeKey::from_type(&exc.rust_type);
    let is_data_class = ext
        .types
        .get(&key)
        .map(|cfg| {
            cfg.kotlin_name.is_some()
                && cfg.opaque.is_none()
                && cfg.enum_cfg.is_none()
                && cfg.callback_kotlin_fqn.is_none()
                && cfg.throwable
        })
        .unwrap_or(false);
    let output_conv = if is_data_class {
        registry
            .output_entry(&exc.rust_type)
            .map(|e| e.function.sig.ident.clone())
    } else {
        None
    };
    if let Some(conv) = output_conv {
        let rust_ty = &exc.rust_type;
        let class_short = &exc.rust_short;
        return syn::parse_quote!(
            #[allow(non_snake_case)]
            pub(crate) fn #throw_fn(env: &mut jni::JNIEnv, err: &#rust_ty) {
                let jobj = match unsafe { #conv(env, err.clone()) } {
                    Ok(o) => o,
                    Err(e) => {
                        tracing::error!(
                            "Failed to encode {} for throw: {}",
                            #class_short,
                            e
                        );
                        return;
                    }
                };
                let throwable = jni::objects::JThrowable::from(jobj);
                if let Err(e) = env.throw(throwable) {
                    tracing::error!("Failed to throw exception: {}", e);
                }
            }
        );
    }
    // Display path: framework `JniBindingError` (no `#[prebindgen]`,
    // no data class — just a class name + a Display message).
    syn::parse_quote!(
        #[allow(non_snake_case)]
        pub(crate) fn #throw_fn(
            env: &mut jni::JNIEnv,
            err: &(impl ::core::fmt::Display + ?Sized),
        ) {
            let exception_class = match env.find_class(#class_path_slashes) {
                Ok(c) => c,
                Err(e) => {
                    tracing::error!("Failed to retrieve exception class: {}", e);
                    return;
                }
            };
            if let Err(e) = env.throw_new(exception_class, err.to_string()) {
                tracing::error!("Failed to throw exception: {}", e);
            }
        }
    )
}

/// One `#[no_mangle] extern "C"` destructor per non-suppressed opaque
/// handle — the Rust counterpart to the `public fun free() = free {
/// freePtr<suffix>(it) }` / `private external fun freePtr<suffix>` pair
/// emitted by [`render_typed_handle_source`]. Each body is the uniform
/// `drop(Box::from_raw(ptr as *mut T))`; the inner `T`'s own `Drop` runs
/// (e.g. `Publisher` network-undeclare) with no special casing.
///
/// Emitted under the same `opaque && !suppress_kotlin_code` condition as
/// the Kotlin shell, so the framework owns *both* halves of the
/// destructor exactly when it owns the typed-handle class. Suppressed
/// handles (hand-written Kotlin) keep their hand-written Rust destructor.
///
/// The symbol follows the documented scheme
/// `Java_<package_underscores>_<class_short>_<mangle_fun("freePtr")>`,
/// where `class_short` is the last segment of the typed-handle FQN
/// (`TypeConfig::kotlin_name`) and the `freePtr` name passes through
/// [`JniGen::mangle_fun`] — exact symmetry with the Kotlin
/// `external fun <mangle_fun("freePtr")>` declaration in
/// [`render_typed_handle_source`]. `ext.types` is a `HashMap`, so the
/// items are sorted by symbol to keep generated output deterministic.
///
/// Emission is gated on the resolved `registry`: a destructor is only
/// emitted for an opaque handle whose type a scanned `#[prebindgen]` fn
/// actually references (as input or output). This mirrors converter
/// emission and keeps feature-gated handles (e.g. `zenoh-ext`-only types
/// whose declare/undeclare fns are `#[cfg]`'d out of the scan) from
/// producing destructors that reference types not in scope.
pub(crate) fn build_handle_destructor_items(ext: &JniGen, registry: &Registry<KotlinMeta>) -> Vec<syn::Item> {
    let free_ptr = ext.mangle_fun("freePtr");
    let mut named: Vec<(String, syn::Item)> = Vec::new();
    for (key, cfg) in &ext.types {
        let Some(opaque) = &cfg.opaque else { continue };
        if opaque.suppress_kotlin_code {
            continue;
        }
        // Skip handles the (feature-aware) scan never references — their
        // type may not be in scope in the generated module.
        let ty = key.to_type();
        if registry.input_entry(&ty).is_none() && registry.output_entry(&ty).is_none() {
            continue;
        }
        let class_short = cfg
            .kotlin_name
            .as_deref()
            .and_then(|fqn| fqn.rsplit('.').next())
            .unwrap_or_else(|| {
                panic!(
                    "build_handle_destructor_items: opaque handle `{}` has no \
                     kotlin_name to derive a destructor symbol from",
                    key.as_str()
                )
            });
        let class_pkg = cfg
            .kotlin_name
            .as_deref()
            .and_then(|fqn| fqn.rsplit_once('.').map(|(pkg, _)| pkg))
            .unwrap_or("")
            .replace('.', "_");
        let symbol = if class_pkg.is_empty() {
            format!("Java_{class_short}_{free_ptr}")
        } else {
            format!("Java_{class_pkg}_{class_short}_{free_ptr}")
        };
        let ident = syn::Ident::new(&symbol, Span::call_site());
        let item: syn::Item = syn::parse_quote!(
            #[no_mangle]
            #[allow(non_snake_case, unused_variables)]
            pub(crate) unsafe extern "C" fn #ident(
                _env: jni::JNIEnv,
                _class: jni::objects::JClass,
                ptr: jni::sys::jlong,
            ) {
                if ptr != 0 {
                    drop(Box::from_raw(ptr as *mut #ty));
                }
            }
        );
        named.push((symbol, item));
    }
    named.sort_by(|a, b| a.0.cmp(&b.0));
    named.into_iter().map(|(_, item)| item).collect()
}

// ──────────────────────────────────────────────────────────────────────
// Prebindgen impl
// ──────────────────────────────────────────────────────────────────────

impl Prebindgen for JniGen {
    /// Cross-language extras every JNI converter carries — currently
    /// the Kotlin value-context type name. Filled by the rank-N
    /// handlers at the same point they build the wire/body; the
    /// resolver propagates it into [`crate::api::core::registry::TypeEntry::metadata`];
    /// the Kotlin emitter reads it back to drive every wrapper /
    /// typed-handle / `JNIWrappers` signature.
    type Metadata = KotlinMeta;

    /// Union of every per-class `.method(...)` / `.companion_method(...)`
    /// list and every `.function(...)` list across all
    /// [`Self::package`] contexts. Each entry is a
    /// `#[prebindgen]` fn ident the user explicitly hooked into the
    /// binding; functions not in this set are skipped by the registry's
    /// signature scan and by the per-item emitter.
    fn declared_functions(&self) -> std::collections::HashSet<syn::Ident> {
        let mut out = std::collections::HashSet::new();
        for cfg in self.types.values() {
            for m in &cfg.instance_methods {
                out.insert(m.rust_ident.clone());
            }
            for m in &cfg.companion_methods {
                out.insert(m.rust_ident.clone());
            }
        }
        for pkg in self.packages.values() {
            for m in &pkg.functions {
                out.insert(m.rust_ident.clone());
            }
        }
        out
    }

    /// Every type registered via `.ptr_class`,
    /// `.data_class`, or `.enum_class` — anything in
    /// [`Self::types`]. These are the only structs/enums the
    /// per-item emitter walks; bodies of undeclared types are
    /// skipped.
    fn declared_types(&self) -> std::collections::HashSet<TypeKey> {
        self.types.keys().cloned().collect()
    }

    /// Emit the `OwnedObject<T>` borrow wrapper used by
    /// [`Self::opaque_handle_input`] into the destination file.
    /// The struct is referenced by an unqualified `OwnedObject` from
    /// the same generated file, so no `use` paths leak into the host
    /// crate's source tree.
    fn prerequisites(&self, registry: &Registry<KotlinMeta>) -> Vec<syn::Item> {
        // `__JniErr` is the **framework** error type alias — always the
        // pre-registered `JniBindingError`, never a user-declared
        // application exception. Built-in converter bodies compose
        // their `?` failures into this type via its `From<String>`
        // impl, so a built-in decode failure surfaces as
        // `JniBindingError` on the JVM. Throwing converters
        // (closures returning `Some(parse_quote!(<full path>))` in the middle slot of
        // `input_wrapper` / `output_wrapper`) instead emit functions
        // typed `Result<…, X>` — they bypass `__JniErr` entirely so no
        // cross-type bridge between the framework error and a domain
        // error is needed (the orphan rule forbids one).
        let error_type = &self.framework_exception().rust_type;
        let alias: syn::Item = syn::parse_quote!(
            #[allow(dead_code)]
            pub(crate) type __JniErr = #error_type;
        );
        let mut items = vec![alias];
        items.extend(owned_object_prerequisite_items());
        // Throw fns — one `pub(crate) fn throw_<short>(env, &err)` per
        // registered throwable class (via `.throwable()`). Emitted as prerequisites
        // (above the converters) so the wrappers below can reference
        // them by bare name; the binding crate references them as
        // `<include_module>::throw_<short>` from outside the file.
        items.extend(
            self.exceptions
                .iter()
                .map(|exc| build_throw_fn_item(self, registry, exc)),
        );
        // Handle destructors — one `extern "C" freePtr<suffix>` per
        // non-suppressed opaque handle (the Rust half of the typed-handle
        // `free()` pair the Kotlin emitter generates).
        items.extend(build_handle_destructor_items(self, registry));
        // Compile-time `Copy` assertion per `value_blob` type — the blob
        // converters reinterpret raw bytes by value, which is only sound for
        // `Copy` types. A mis-declared non-`Copy` type fails to compile here
        // (at the include site) with a clear bound error rather than at a
        // converter use. The bare type name is qualified against
        // `source_module` by `post_process_item` like every other body.
        for (key, cfg) in &self.types {
            if cfg.value_blob {
                let ty = key.to_type();
                items.push(syn::parse_quote!(
                    const _: () = {
                        const fn __assert_copy<T: ::core::marker::Copy>() {}
                        __assert_copy::<#ty>();
                    };
                ));
            }
        }
        items
    }

    fn post_process_item(&self, item: &mut syn::Item) {
        self.qualify_item(item);
    }

    // ── Item methods ─────────────────────────────────────────────────

    fn on_function(&self, f: &syn::ItemFn, registry: &Registry<KotlinMeta>) -> TokenStream {
        emit_jni_function_wrapper(self, f, registry)
    }

    fn on_struct(&self, _s: &syn::ItemStruct, _registry: &Registry<KotlinMeta>) -> TokenStream {
        // Struct converter bodies are emitted by the resolver via
        // on_input_type_rank_0 / on_output_type_rank_0 below; no separate
        // per-struct item is needed.
        TokenStream::new()
    }

    fn on_enum(&self, _e: &syn::ItemEnum, _registry: &Registry<KotlinMeta>) -> TokenStream {
        TokenStream::new()
    }

    fn on_const(&self, c: &syn::ItemConst, _registry: &Registry<KotlinMeta>) -> TokenStream {
        c.to_token_stream()
    }

    // ── Input converters ─────────────────────────────────────────────

    fn on_input_type_rank_0(
        &self,
        ty: &syn::Type,
        registry: &Registry<KotlinMeta>,
    ) -> Option<ConverterImpl<KotlinMeta>> {
        // Structured-config overrides first (opaque handles, then user-
        // registered rank-0 wrappers, then built-ins).
        let key = TypeKey::from_type(ty);
        if let Some(cfg) = self.types.get(&key) {
            if cfg.opaque.is_some() {
                return Some(self.opaque_handle_input(ty));
            }
        }
        // `value_blob`-declared `Copy` types: decode the raw memory blob out
        // of a `JByteArray` (length-checked, `read_unaligned` since the byte
        // array isn't aligned to the type). Returns owned `T`, so `&T` /
        // by-value / `Vec<T>` / `Option<T>` all compose through the existing
        // handlers. `T: Copy` ⇒ reading the value out is sound (no double
        // drop); the `Copy` bound itself is enforced by the assertion in
        // `prerequisites`.
        if self.types.get(&key).map(|c| c.value_blob).unwrap_or(false) {
            let wire: syn::Type = syn::parse_quote!(jni::objects::JByteArray);
            let body: syn::Expr = syn::parse_quote!({
                let __bytes = env.convert_byte_array(v).map_err(|e| {
                    <__JniErr as ::core::convert::From<String>>::from(format!(
                        "value-blob decode: {}",
                        e
                    ))
                })?;
                if __bytes.len() != ::core::mem::size_of::<#ty>() {
                    return ::core::result::Result::Err(
                        <__JniErr as ::core::convert::From<String>>::from(
                            "value-blob decode: wrong byte length".to_string(),
                        ),
                    );
                }
                unsafe { ::core::ptr::read_unaligned(__bytes.as_ptr() as *const #ty) }
            });
            let niches = default_niches_for_wire(&wire);
            return Some(ConverterImpl {
                pre_stages: vec![],
                function: self.build_input_fn(ty, &wire, &body, None),
                destination: wire,
                niches,
                metadata: KotlinMeta {
                    projection: Some(Projection {
                        leaf_key: key.as_str().to_string(),
                        owned: false,
                        strategy: FoldStrategy::Direct,
                        kind: ProjectionKind::ValueBlob,
                    }),
                    ..self.framework_meta(Some("ByteArray".to_string()))
                },
            });
        }
        // `enum_class`-declared enums: jint wire, `TryFrom<i32>` decode.
        // Registered before the user-wrapper lookup so a stray
        // `input_wrapper` registration on the same key would have to be
        // intentional. The rank-0 enum arm produces a terminal converter
        // (jint → Rust enum) with the configured Kotlin FQN in metadata.
        if let Some(cfg) = self.types.get(&key) {
            if cfg.enum_cfg.is_some() {
                if let Some(name) = bare_path_ident(ty) {
                    if let Some((e, _)) = registry.enums.get(&name) {
                        let (wire, body) = enum_input_body(self, e);
                        let niches = default_niches_for_wire(&wire);
                        let kotlin_name = cfg.kotlin_name.clone();
                        return Some(ConverterImpl {
                            pre_stages: vec![],
                            function: self.build_input_fn(ty, &wire, &body, None),
                            destination: wire,
                            niches,
                            metadata: self.framework_meta(kotlin_name),
                        });
                    }
                }
            }
        }
        if let Some(conv) = self.lookup_input(ty, &[], registry) {
            return Some(conv);
        }
        // `str` is unsized, so converters can't return it directly.
        // Still register a rank-0 entry to satisfy resolution for
        // borrowed `&str` parameters: decode `JString` to owned `String`
        // and let call sites borrow as needed.
        if TypeKey::from_type(ty).as_str() == "str" {
            let wire: syn::Type = syn::parse_quote!(jni::objects::JString);
            let body: syn::Expr = syn::parse_quote!({
                let s = env.get_string(v).map_err(|e| {
                    <__JniErr as ::core::convert::From<String>>::from(format!(
                        "decode_string: {}",
                        e
                    ))
                })?;
                s.into()
            });
            let rust_ty: syn::Type = syn::parse_quote!(String);
            let kotlin_name = self.override_kotlin_name(ty, Some("String".to_string()));
            let niches = default_niches_for_wire(&wire);
            return Some(ConverterImpl {
                pre_stages: vec![],
                function: self.build_input_fn(&rust_ty, &wire, &body, None),
                destination: wire,
                niches,
                metadata: self.framework_meta(kotlin_name),
            });
        }
        if let Some((wire, body)) = primitive_input(ty) {
            let niches = default_niches_for_wire(&wire);
            let kotlin_name = kotlin_for_wire(&wire);
            return Some(ConverterImpl {
                pre_stages: vec![],
                function: self.build_input_fn(ty, &wire, &body, None),
                destination: wire,
                niches,
                metadata: self.framework_meta(kotlin_name),
            });
        }
        if let Some(name) = bare_path_ident(ty) {
            if let Some((s, _)) = registry.structs.get(&name) {
                // Value-class leaf (input mirror of the output branch in
                // `on_output_type_rank_0`): a `@JvmInline value class` is erased
                // to its single inner field, so decode that field's wire
                // directly and construct the value class, tagging a `ValueClass`
                // projection. This makes a value-class *parameter* render as
                // `ValueUnwrap` (the `external fun` declares the erased inner —
                // e.g. `ByteArray` — and the wrapper passes `<name>.<field>`),
                // which is required: a `@JvmInline value class` in an
                // `external fun` signature triggers Kotlin's value-class name
                // mangling (`name-<hash>`) and breaks JNI linkage against the
                // unmangled native symbol.
                if self.types.get(&key).map(|c| c.value_class).unwrap_or(false) {
                    if let Some((inner_ident, inner_ty)) = value_class_inner_field(s) {
                        let inner_entry = registry.input_entry(&inner_ty)?;
                        let inner_conv = inner_entry.function.sig.ident.clone();
                        let wire = inner_entry.destination.clone();
                        // Qualify the struct literal against the source module
                        // (`zenoh_flat::ZBytes { .. }`) — a struct-literal path is
                        // an expression, not a type, so `post_process_item`'s
                        // type-qualifier wouldn't reach it (mirrors
                        // `struct_input_body`'s `#struct_module::#ident { .. }`).
                        let struct_module = struct_module_path(self, s);
                        let struct_ident = &s.ident;
                        let body: syn::Expr = syn::parse_quote!({
                            let __inner = #inner_conv(env, v)?;
                            #struct_module::#struct_ident { #inner_ident: __inner }
                        });
                        return Some(ConverterImpl {
                            pre_stages: vec![],
                            function: self.build_input_fn(ty, &wire, &body, None),
                            destination: wire,
                            niches: inner_entry.niches.clone(),
                            metadata: KotlinMeta {
                                projection: Some(Projection {
                                    leaf_key: key.as_str().to_string(),
                                    owned: false,
                                    strategy: FoldStrategy::Direct,
                                    kind: ProjectionKind::ValueClass,
                                }),
                                ..self.framework_meta(inner_entry.metadata.kotlin_name.clone())
                            },
                        });
                    }
                }
                let (wire, body) = struct_input_body(self, s, registry)?;
                let niches = default_niches_for_wire(&wire);
                // Auto-generated struct: the value-context Kotlin name is
                // whatever the user pinned via `data_class`. If
                // they didn't, leave `kotlin_name = None` — emitter
                // surfaces this as a build-time hard error.
                let kotlin_name = self.types.get(&key).and_then(|c| c.kotlin_name.clone());
                return Some(ConverterImpl {
                    pre_stages: vec![],
                    function: self.build_input_fn(ty, &wire, &body, None),
                    destination: wire,
                    niches,
                    metadata: self.framework_meta(kotlin_name),
                });
            }
            // Bare-ident enum: leave to the consuming crate to override
            // (today's CongestionControl etc. fall here — caller's wrapper
            // ext returns Some in its own on_input_type_rank_0).
        }
        None
    }

    fn on_input_type_rank_1(
        &self,
        pat: &syn::Type,
        t1: &syn::Type,
        registry: &Registry<KotlinMeta>,
    ) -> Option<ConverterImpl<KotlinMeta>> {
        if let Some(conv) = self.lookup_input(pat, &[t1.clone()], registry) {
            return Some(conv);
        }
        // `& _` borrow: a free-fn converter can't return `&T` (no borrow
        // source), so we *share* T's resolved converter — `&T`'s entry
        // points at the same `ItemFn`. The fn returns owned `T`; the
        // call site in `emit_jni_function_wrapper` adds `&decoded` when
        // the original param was `&T`. write.rs's dedup-by-name keeps
        // the function emitted exactly once.
        //
        // This handler exists to make the wildcard-substitution machinery
        // fire: it returns subs=[t1] (via the resolver), so propagation
        // marks T as required transitively from `&T`.
        if pat_match(pat, "& _") || pat_match(pat, "& mut _") {
            let inner = registry.input_entry(t1)?;
            let outer_ty: syn::Type = if pat_match(pat, "& mut _") {
                syn::parse_quote!(&mut #t1)
            } else {
                syn::parse_quote!(&#t1)
            };
            // `&T` / `&mut T` are Kotlin-side no-ops — inherit the inner
            // type's name, unless the user pinned an explicit override
            // on the outer form itself (rare but legal).
            let kotlin_name =
                self.override_kotlin_name(&outer_ty, inner.metadata.kotlin_name.clone());
            // The outer form shares T's converter function verbatim, so it
            // inherits T's throws behaviour (whatever exception T's
            // converter is bound to). Copy the inner's throws metadata.
            // A borrowed handle (mut or not) is still opaque (param
            // classification needs to see it), but the holder doesn't own
            // it — mark `owned: false` so `close()` emission skips it.
            let projection = inner
                .metadata
                .projection
                .clone()
                .map(|h| Projection { owned: false, ..h });
            return Some(ConverterImpl {
                destination: inner.destination.clone(),
                function: inner.function.clone(),
                pre_stages: vec![],
                niches: inner.niches.clone(),
                metadata: KotlinMeta {
                    kotlin_name,
                    throws: inner.metadata.throws.clone(),
                    throws_action: inner.metadata.throws_action.clone(),
                    value_rust_key: None,
                    projection,
                },
            });
        }
        // `Option<&T>` / `Option<&mut T>` for opaque T: the general
        // `Option<_>` handler below treats the inner type opaquely and
        // would generate `Option<&T>` with no lifetime + a buggy
        // `*const &T` cast. Route opaque borrows through their own path
        // that returns `Option<OwnedObject<T>>`; the call site
        // `.as_deref()` / `.as_deref_mut()` coerces back to `Option<&T>`
        // / `Option<&mut T>` per OwnedObject's Deref / DerefMut impls.
        //
        // Falls through for non-opaque inners — the general handler
        // produces sensible code (returns `Option<T>` and the call site
        // adds `.as_ref()` if needed; out of scope here).
        if pat_match(pat, "Option < & _ >") || pat_match(pat, "Option < & mut _ >") {
            let inner = registry.input_entry(t1)?;
            if inner.metadata.is_direct_handle() {
                let is_mut = pat_match(pat, "Option < & mut _ >");
                let inner_wire = inner.destination.clone();
                let inner_conv = inner.function.sig.ident.clone();
                let outer_ty: syn::Type = if is_mut {
                    syn::parse_quote!(Option<&mut #t1>)
                } else {
                    syn::parse_quote!(Option<&#t1>)
                };
                let name = input_name(&outer_ty, &inner_wire);
                let function: syn::ItemFn = syn::parse_quote!(
                    #[allow(non_snake_case, unused_mut, unused_variables, unused_braces, dead_code)]
                    pub(crate) unsafe fn #name<'env, 'v>(
                        env: &mut jni::JNIEnv<'env>,
                        v: &#inner_wire,
                    ) -> ::core::result::Result<Option<OwnedObject<#t1>>, __JniErr> {
                        Ok({
                            if *v == 0 { None } else { Some(#inner_conv(env, v)?) }
                        })
                    }
                );
                let kotlin_name =
                    self.override_kotlin_name(&outer_ty, inner.metadata.kotlin_name.clone());
                let projection = inner.metadata.projection.clone().map(|h| Projection {
                    owned: false,
                    // `Option<&Handle>` always rides the inner's `*v == 0` niche
                    // (body is `if *v == 0 { None } else { ... }` above), so
                    // null is the `0i64` sentinel — never JVM boxed.
                    strategy: FoldStrategy::Nullable {
                        kind: NullableKind::Niche,
                        inner: Box::new(h.strategy),
                    },
                    ..h
                });
                return Some(ConverterImpl {
                    pre_stages: vec![],
                    function,
                    destination: inner_wire,
                    niches: Niches::empty(),
                    metadata: KotlinMeta {
                        kotlin_name,
                        throws: inner.metadata.throws.clone(),
                        throws_action: inner.metadata.throws_action.clone(),
                        value_rust_key: None,
                        projection,
                    },
                });
            }
            // Non-opaque: let the general `Option<_>` handler below take it.
        }
        // `Vec<T>` (input side): wire is `JObject` carrying a Java
        // `List<InnerWire>`; we iterate, decode each element via the
        // inner converter, collect into a `Vec`. `Vec<u8>` is already
        // handled at rank-0 (special-cased in `primitive_input` to a
        // `JByteArray` wire) so rank-1 never gets it. Non-opaque inners
        // whose wire is a non-jobject primitive (e.g. `Vec<i32>`) aren't
        // covered by this handler — extend if needed.
        if pat_match(pat, "Vec < _ >") {
            let inner = registry.input_entry(t1)?;
            reject_vec_of_handle(&inner.metadata.projection, t1);
            let inner_wire = inner.destination.clone();
            if !is_jobject_shaped_wire(&inner_wire) {
                return None;
            }
            let inner_conv = inner.function.sig.ident.clone();
            let outer_ty: syn::Type = syn::parse_quote!(Vec<#t1>);
            let wire: syn::Type = syn::parse_quote!(jni::objects::JObject);
            let body: syn::Expr = syn::parse_quote!({
                let __list = jni::objects::JList::from_env(env, v)
                    .map_err(|e| <__JniErr as ::core::convert::From<String>>::from(format!("Vec<_>: list-from-env: {}", e)))?;
                let mut __it = __list.iter(env)
                    .map_err(|e| <__JniErr as ::core::convert::From<String>>::from(format!("Vec<_>: list-iter: {}", e)))?;
                let mut __out: Vec<#t1> = Vec::new();
                while let Some(__obj) = __it.next(env)
                    .map_err(|e| <__JniErr as ::core::convert::From<String>>::from(format!("Vec<_>: list-next: {}", e)))?
                {
                    let __elem_wire: #inner_wire = __obj.into();
                    let __elem: #t1 = #inner_conv(env, &__elem_wire)?;
                    __out.push(__elem);
                }
                __out
            });
            let inner_kotlin = inner.metadata.kotlin_name.clone()?;
            let kotlin_name = self.override_kotlin_name(
                &outer_ty,
                // `List` is auto-imported in Kotlin (default imports), so we
                // skip the FQN to avoid `register_fqn` treating the generic
                // as part of the import path.
                Some(format!("List<{}>", inner_kotlin)),
            );
            return Some(ConverterImpl {
                pre_stages: vec![],
                function: self.build_input_fn(&outer_ty, &wire, &body, None),
                destination: wire,
                niches: Niches::empty(),
                metadata: KotlinMeta {
                    kotlin_name,
                    throws: inner.metadata.throws.clone(),
                    throws_action: inner.metadata.throws_action.clone(),
                    value_rust_key: None,
                    projection: None,
                },
            });
        }
        if pat_match(pat, "Option < _ >") {
            let outer_ty: syn::Type = syn::parse_quote!(Option<#t1>);
            let (wire, body, niches) = option_input(t1, registry)?;
            // Inherit the inner's name; user pins on `Option<T>` win.
            // The nullability marker (`?`) is added by the use site.
            let inherited = registry
                .input_entry(t1)
                .and_then(|e| e.metadata.kotlin_name.clone());
            let kotlin_name = self.override_kotlin_name(&outer_ty, inherited);
            // Fold a Nullable layer over the inner projection (if any). The
            // kind mirrors which path `option_input` took: when it consumed
            // an inner niche, the wire stays identical to the inner's
            // destination (e.g. `jlong` for handles, `JByteArray` for
            // ByteArray-shaped value classes) and `None` is the niche slot
            // sentinel; the boxed fallback widens the wire to `JObject`. The
            // renderer reads `kind` so the Kotlin declared wire and wrap
            // shape match the runtime ABI.
            let nullable_kind = nullable_kind_for(&wire, t1, registry);
            let projection = registry
                .input_entry(t1)
                .and_then(|e| e.metadata.projection.clone())
                .map(|h| Projection {
                    strategy: FoldStrategy::Nullable {
                        kind: nullable_kind,
                        inner: Box::new(h.strategy),
                    },
                    ..h
                });
            return Some(ConverterImpl {
                pre_stages: vec![],
                function: self.build_input_fn(&outer_ty, &wire, &body, None),
                destination: wire,
                niches,
                metadata: KotlinMeta {
                    projection,
                    ..self.framework_meta(kotlin_name)
                },
            });
        }
        None
    }

    fn dispatch_into_input(
        &self,
        target: &syn::Type,
        sources: &[IntoSource],
        registry: &Registry<KotlinMeta>,
    ) -> Option<ConverterImpl<KotlinMeta>> {
        self.emit_into_dispatcher(target, sources, registry)
    }

    fn dispatch_fn_input(
        &self,
        args: &[syn::Type],
        registry: &Registry<KotlinMeta>,
    ) -> Option<ConverterImpl<KotlinMeta>> {
        let outer_ty = build_fn_type(args);
        let (wire, body) = callback_input(self, args, registry)?;
        let niches = default_niches_for_wire(&wire);
        // Kotlin sees `impl Fn(...)` as the matching mangled
        // fun-interface (zenoh-jni: `JNIOn<Args>`). Use the
        // registration-stamped FQN when set; fall back to the
        // auto-derived name.
        let outer_key = TypeKey::from_type(&outer_ty);
        let kotlin_name = self
            .types
            .get(&outer_key)
            .and_then(|c| c.callback_kotlin_fqn.clone())
            .or_else(|| Some(self.auto_callback_fqn(args)));
        Some(ConverterImpl {
            pre_stages: vec![],
            function: self.build_input_fn(&outer_ty, &wire, &body, None),
            destination: wire,
            niches,
            metadata: self.framework_meta(kotlin_name),
        })
    }

    fn on_input_type_rank_2(
        &self,
        pat: &syn::Type,
        t1: &syn::Type,
        t2: &syn::Type,
        registry: &Registry<KotlinMeta>,
    ) -> Option<ConverterImpl<KotlinMeta>> {
        let _ = registry;
        self.lookup_input(pat, &[t1.clone(), t2.clone()], registry)
    }

    fn on_input_type_rank_3(
        &self,
        pat: &syn::Type,
        t1: &syn::Type,
        t2: &syn::Type,
        t3: &syn::Type,
        registry: &Registry<KotlinMeta>,
    ) -> Option<ConverterImpl<KotlinMeta>> {
        let _ = registry;
        self.lookup_input(pat, &[t1.clone(), t2.clone(), t3.clone()], registry)
    }

    // ── Output converters ────────────────────────────────────────────

    fn on_output_type_rank_0(
        &self,
        ty: &syn::Type,
        registry: &Registry<KotlinMeta>,
    ) -> Option<ConverterImpl<KotlinMeta>> {
        // Structured-config overrides first (opaque handles, then the
        // unified user-registered wrapper table, then built-ins).
        let key = TypeKey::from_type(ty);
        if let Some(cfg) = self.types.get(&key) {
            if cfg.opaque.is_some() {
                return Some(self.opaque_handle_output(ty));
            }
        }
        // `value_blob`-declared `Copy` types: encode the value's raw memory
        // bytes into a fresh `JByteArray` (the value-level peer of an opaque
        // handle's `jlong`). `v: #ty` is owned and `Copy`, so reading its
        // bytes and letting it drop normally is sound. Wire is `JByteArray`
        // (jobject-shaped), so `Vec<T>` / `Option<T>` compose through the
        // existing handlers — `Vec<value-blob>` surfaces as `List<ByteArray>`.
        if self.types.get(&key).map(|c| c.value_blob).unwrap_or(false) {
            let wire: syn::Type = syn::parse_quote!(jni::objects::JByteArray);
            let body: syn::Expr = syn::parse_quote!({
                let __bytes: &[u8] = unsafe {
                    ::core::slice::from_raw_parts(
                        (&v as *const #ty) as *const u8,
                        ::core::mem::size_of::<#ty>(),
                    )
                };
                env.byte_array_from_slice(__bytes).map_err(|e| {
                    <__JniErr as ::core::convert::From<String>>::from(format!(
                        "value-blob encode: {}",
                        e
                    ))
                })?
            });
            let niches = default_niches_for_wire(&wire);
            return Some(ConverterImpl {
                pre_stages: vec![],
                function: self.build_output_fn(ty, &wire, &body, None),
                destination: wire,
                niches,
                metadata: KotlinMeta {
                    projection: Some(Projection {
                        leaf_key: key.as_str().to_string(),
                        owned: false,
                        strategy: FoldStrategy::Direct,
                        kind: ProjectionKind::ValueBlob,
                    }),
                    ..self.framework_meta(Some("ByteArray".to_string()))
                },
            });
        }
        // `enum_class`-declared enums: jint wire, `as jni::sys::jint`
        // encode. Symmetric to the input arm above; relies on
        // `#[repr(i32)]` (or any repr that supports the cast) on the
        // declared enum so the discriminant value round-trips identically.
        if let Some(cfg) = self.types.get(&key) {
            if cfg.enum_cfg.is_some() {
                if let Some(name) = bare_path_ident(ty) {
                    if let Some((e, _)) = registry.enums.get(&name) {
                        let (wire, body) = enum_output_body(self, e);
                        let niches = default_niches_for_wire(&wire);
                        let kotlin_name = cfg.kotlin_name.clone();
                        return Some(ConverterImpl {
                            pre_stages: vec![],
                            function: self.build_output_fn(ty, &wire, &body, None),
                            destination: wire,
                            niches,
                            metadata: self.framework_meta(kotlin_name),
                        });
                    }
                }
            }
        }
        if let Some(conv) = self.lookup_output(ty, &[], registry) {
            return Some(conv);
        }
        // `()` — identity converter so `fn foo()` and `fn foo() -> ()`
        // funnel through the same uniform output path as everything else.
        // Wire is `()`. Body just returns `v`. No Kotlin name — Unit
        // returns are dropped from emitted signatures, so metadata stays
        // empty.
        if pat_match(ty, "()") {
            let wire: syn::Type = syn::parse_quote!(());
            let body: syn::Expr = syn::parse_quote!(v);
            return Some(ConverterImpl {
                function: self.build_output_fn(ty, &wire, &body, None),
                destination: wire,
                pre_stages: vec![],
                niches: Niches::empty(),
                metadata: KotlinMeta::default(),
            });
        }
        if let Some((wire, body)) = primitive_output(ty) {
            let niches = default_niches_for_wire(&wire);
            let kotlin_name = kotlin_for_wire(&wire);
            return Some(ConverterImpl {
                pre_stages: vec![],
                function: self.build_output_fn(ty, &wire, &body, None),
                destination: wire,
                niches,
                metadata: self.framework_meta(kotlin_name),
            });
        }
        if let Some(name) = bare_path_ident(ty) {
            if let Some((s, _)) = registry.structs.get(&name) {
                // Value-class leaf: a `@JvmInline value class` is erased to its
                // single inner field, so the converter delegates to the inner
                // field's converter (wire + descriptor + value-context Kotlin
                // name all come from it) and tags a `ValueClass` projection.
                // Every typed-surface emitter then wraps `W(inner)` and folds
                // through Option/Vec uniformly — same machinery opaque handles
                // ride, no value-class special cases in the struct encoder.
                if self.types.get(&key).map(|c| c.value_class).unwrap_or(false) {
                    if let Some(inner) = value_class_inner_field(s) {
                        let (inner_ident, inner_ty) = inner;
                        let inner_entry = registry.output_entry(&inner_ty)?;
                        let inner_conv = inner_entry.function.sig.ident.clone();
                        let wire = inner_entry.destination.clone();
                        let body: syn::Expr =
                            syn::parse_quote!({ #inner_conv(env, v.#inner_ident)? });
                        return Some(ConverterImpl {
                            pre_stages: vec![],
                            function: self.build_output_fn(ty, &wire, &body, None),
                            destination: wire,
                            niches: inner_entry.niches.clone(),
                            metadata: KotlinMeta {
                                projection: Some(Projection {
                                    leaf_key: key.as_str().to_string(),
                                    owned: false,
                                    strategy: FoldStrategy::Direct,
                                    kind: ProjectionKind::ValueClass,
                                }),
                                ..self.framework_meta(inner_entry.metadata.kotlin_name.clone())
                            },
                        });
                    }
                }
                let (wire, body) = struct_output_body(self, s, registry)?;
                let niches = default_niches_for_wire(&wire);
                let kotlin_name = self.types.get(&key).and_then(|c| c.kotlin_name.clone());
                return Some(ConverterImpl {
                    pre_stages: vec![],
                    function: self.build_output_fn(ty, &wire, &body, None),
                    destination: wire,
                    niches,
                    metadata: self.framework_meta(kotlin_name),
                });
            }
        }
        None
    }

    fn on_output_type_rank_1(
        &self,
        pat: &syn::Type,
        t1: &syn::Type,
        registry: &Registry<KotlinMeta>,
    ) -> Option<ConverterImpl<KotlinMeta>> {
        if let Some(conv) = self.lookup_output(pat, &[t1.clone()], registry) {
            return Some(conv);
        }
        // Borrowed opaque-handle output (`&T` / `&'static T` where `T` is a
        // declared opaque handle). Canonical zenoh-flat's `z_*` accessors
        // return *borrowed* handles for the C tier's zero-copy borrows, but
        // the JVM keeps its handle past the call — so the only sound lowering
        // is to clone the referent into a fresh owned `Box`-handle (every such
        // zenoh handle type is `Clone`). This mirrors `opaque_handle_output`
        // with a `.clone()`; `Option<&T>` then composes through the `Option`
        // arm below (it looks up this `&T` entry as its inner). Matched
        // structurally so the lifetime variant `&'static _` is covered too.
        if let syn::Type::Reference(r) = pat {
            if r.mutability.is_none()
                && self
                    .types
                    .get(&TypeKey::from_type(t1))
                    .map(|c| c.opaque.is_some())
                    .unwrap_or(false)
            {
                let mut ref_ty = r.clone();
                *ref_ty.elem = t1.clone();
                let outer_ty = syn::Type::Reference(ref_ty);
                let wire: syn::Type = syn::parse_quote!(jni::sys::jlong);
                let body: syn::Expr = syn::parse_quote!(std::boxed::Box::into_raw(
                    std::boxed::Box::new(v.clone())
                ) as i64);
                return Some(ConverterImpl {
                    function: self.build_output_fn(&outer_ty, &wire, &body, None),
                    destination: wire,
                    pre_stages: vec![],
                    niches: Niches::one(syn::parse_quote!(0i64), syn::parse_quote!(*v == 0)),
                    metadata: self.opaque_leaf_meta(t1),
                });
            }
        }
        // `Result<_, _>` is handled as a built-in rank-2 wrapper registered
        // in `JniGen::new`. Bindings just declare the Err type via
        // `.throwable()`. Per-error overrides are possible by registering a
        // more specific rank-1 `output_wrapper(Result<_, ConcreteErr>, …)`
        // — rank-1 fires before rank-2 in resolve and short-circuits here.
        if pat_match(pat, "Option < _ >") {
            let outer_ty: syn::Type = syn::parse_quote!(Option<#t1>);
            let (wire, body, niches) = option_output(t1, registry)?;
            let inherited = registry
                .output_entry(t1)
                .and_then(|e| e.metadata.kotlin_name.clone());
            let kotlin_name = self.override_kotlin_name(&outer_ty, inherited);
            // Fold a Nullable layer over the inner projection (if any). The
            // kind reflects which path `option_output` took (see
            // [`nullable_kind_for`]): niche-fulfilled keeps the inner wire
            // and treats the slot value as `None`; boxed widens to `JObject`
            // and uses JVM null.
            let nullable_kind = nullable_kind_for_output(&wire, t1, registry);
            let projection = registry
                .output_entry(t1)
                .and_then(|e| e.metadata.projection.clone())
                .map(|h| Projection {
                    strategy: FoldStrategy::Nullable {
                        kind: nullable_kind,
                        inner: Box::new(h.strategy),
                    },
                    ..h
                });
            return Some(ConverterImpl {
                pre_stages: vec![],
                function: self.build_output_fn(&outer_ty, &wire, &body, None),
                destination: wire,
                niches,
                metadata: KotlinMeta {
                    projection,
                    ..self.framework_meta(kotlin_name)
                },
            });
        }
        // `Vec<T>` (output side): encode as a `java.util.ArrayList<InnerWire>`.
        // Symmetric to the input handler. `Vec<u8>` is special-cased at
        // rank-0 (primitive_output → JByteArray) so rank-1 never sees it.
        if pat_match(pat, "Vec < _ >") {
            let inner = registry.output_entry(t1)?;
            reject_vec_of_handle(&inner.metadata.projection, t1);
            let inner_wire = inner.destination.clone();
            if !is_jobject_shaped_wire(&inner_wire) {
                return None;
            }
            let inner_conv = inner.function.sig.ident.clone();
            let outer_ty: syn::Type = syn::parse_quote!(Vec<#t1>);
            let wire: syn::Type = syn::parse_quote!(jni::objects::JObject);
            let body: syn::Expr = syn::parse_quote!({
                let __list_obj = env
                    .new_object("java/util/ArrayList", "()V", &[])
                    .map_err(|e| <__JniErr as ::core::convert::From<String>>::from(format!("Vec<_>: new ArrayList: {}", e)))?;
                let __list = jni::objects::JList::from_env(env, &__list_obj)
                    .map_err(|e| <__JniErr as ::core::convert::From<String>>::from(format!("Vec<_>: list-from-env: {}", e)))?;
                for __elem in v.into_iter() {
                    let __elem_wire = #inner_conv(env, __elem)?;
                    let __elem_obj: jni::objects::JObject = __elem_wire.into();
                    __list.add(env, &__elem_obj)
                        .map_err(|e| <__JniErr as ::core::convert::From<String>>::from(format!("Vec<_>: list-add: {}", e)))?;
                }
                __list_obj
            });
            let inner_kotlin = inner.metadata.kotlin_name.clone()?;
            let kotlin_name = self.override_kotlin_name(
                &outer_ty,
                // `List` is auto-imported in Kotlin (default imports), so we
                // skip the FQN to avoid `register_fqn` treating the generic
                // as part of the import path. When the inner carries a
                // projection, this wire-context name still drives non-
                // projection consumers; projection-aware sites (classify_return,
                // data-class fields) prefer `projection` and render the typed
                // `List<TypedShort>` instead.
                Some(format!("List<{}>", inner_kotlin)),
            );
            // Fold an Iterable layer over the inner projection (if any), so
            // `Vec<Handle>` / `Vec<ValueClass>` carry the full strategy.
            let projection = inner.metadata.projection.clone().map(|h| Projection {
                strategy: FoldStrategy::Iterable(Box::new(h.strategy)),
                ..h
            });
            return Some(ConverterImpl {
                pre_stages: vec![],
                function: self.build_output_fn(&outer_ty, &wire, &body, None),
                destination: wire,
                niches: Niches::empty(),
                metadata: KotlinMeta {
                    kotlin_name,
                    throws: inner.metadata.throws.clone(),
                    throws_action: inner.metadata.throws_action.clone(),
                    value_rust_key: None,
                    projection,
                },
            });
        }
        None
    }

    fn on_output_type_rank_2(
        &self,
        pat: &syn::Type,
        t1: &syn::Type,
        t2: &syn::Type,
        registry: &Registry<KotlinMeta>,
    ) -> Option<ConverterImpl<KotlinMeta>> {
        self.lookup_output(pat, &[t1.clone(), t2.clone()], registry)
    }

    fn on_output_type_rank_3(
        &self,
        pat: &syn::Type,
        t1: &syn::Type,
        t2: &syn::Type,
        t3: &syn::Type,
        registry: &Registry<KotlinMeta>,
    ) -> Option<ConverterImpl<KotlinMeta>> {
        self.lookup_output(pat, &[t1.clone(), t2.clone(), t3.clone()], registry)
    }

    fn into_sources(&self, target: &syn::Type) -> Vec<IntoSource> {
        let key = TypeKey::from_type(target);
        self.into_sources_map.get(&key).cloned().unwrap_or_default()
    }
}
