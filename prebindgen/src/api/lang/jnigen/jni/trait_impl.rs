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
    /// `exc` ties the body convention to the `Result`'s Rust error type:
    /// * `None` → signature `Result<rust, __JniErr>` and the body is
    ///   wrapped `Ok(<body>)`; `?` inside propagates the framework error.
    /// * `Some(E)` → signature `Result<rust, E>` and the body is emitted
    ///   as-is — `<body>` already evaluates to that `Result`, so no `Ok`
    ///   wrap. `E` is the raw error type peeled from a `Result<T, E>`.
    pub(crate) fn build_input_fn(
        &self,
        rust: &syn::Type,
        wire: &syn::Type,
        body: &syn::Expr,
        exc: Option<&syn::Type>,
    ) -> syn::ItemFn {
        let name = input_name(rust, wire);
        let rust_with_lifetime = annotate_borrow_with_lifetime(rust, "env");
        let wire_with_lifetime = annotate_jobject_with_lifetime(wire, "v");
        let err_type = exc.cloned().unwrap_or_else(default_err_type);
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
        exc: Option<&syn::Type>,
    ) -> syn::ItemFn {
        let name = output_name(rust, wire);
        let wire_with_lifetime = annotate_jobject_with_lifetime(wire, "a");
        let err_type = exc.cloned().unwrap_or_else(default_err_type);
        let ret_body = body_for_exc(body, exc);
        syn::parse_quote!(
            #[allow(non_snake_case, unused_mut, unused_variables, unused_braces, dead_code)]
            pub(crate) unsafe fn #name<'a>(env: &mut jni::JNIEnv<'a>, v: #rust) -> ::core::result::Result<#wire_with_lifetime, #err_type> {
                #ret_body
            }
        )
    }

    /// Borrowed string-slice output converter (`&str → jstring`, a single
    /// copy — the dual of the `str` input arm). Shared by two resolver arms so
    /// they emit the SAME-named fn (write.rs dedups by `sig.ident`):
    /// * the rank-1 `&str` arm — the converter actually used for a reference
    ///   accessor leaf (`f(&T) -> &str`, output expansion);
    /// * the rank-0 `str` arm — resolves the unsized `str` reached as the sub
    ///   of `&str` (so required-propagation doesn't flag `str` unresolved).
    ///
    /// Surfaces as Kotlin `String`. Built from a normalized (lifetime-free)
    /// `&str` so both arms produce an identical [`output_name`].
    fn str_ref_output(&self) -> ConverterImpl<KotlinMeta> {
        let outer_ty: syn::Type = syn::parse_quote!(&str);
        let wire: syn::Type = syn::parse_quote!(jni::objects::JString);
        let body: syn::Expr = syn::parse_quote!({
            env.new_string(v).map_err(|e| {
                <__JniErr as ::core::convert::From<String>>::from(format!("encode_str: {}", e))
            })?
        });
        let kotlin_name = self.override_kotlin_name(&outer_ty, Some("String".to_string()));
        let niches = default_niches_for_wire(&wire);
        ConverterImpl {
            pre_stages: vec![],
            function: self.build_output_fn(&outer_ty, &wire, &body, None),
            destination: wire,
            niches,
            metadata: self.framework_meta(kotlin_name),
        }
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

}

/// The single `signal_error` free function: the one error channel every
/// generated extern uses. Instead of throwing a JVM exception, it invokes
/// the per-call Kotlin `ErrorSink.onError(message: String)` callback with the
/// error's `Display` string. The caller's wrapper installs a default sink
/// that captures the message and rethrows it as a Kotlin exception after the
/// native call returns (so SDK `try/catch` keeps working), but a caller may
/// pass any sink and do anything else. This is the seed of the unified
/// callback return-channel: a later step can add an `onValue(...)` leg so
/// success values flow through the same sink.
///
/// `err` is generic over `Display`, so both the framework `__JniErr`
/// (`JniBindingError`, a `String` wrapper) and a domain `Result<T, E>`'s `E`
/// funnel through one function with no per-type routing.
pub(crate) fn build_signal_error_item() -> syn::Item {
    syn::parse_quote!(
        #[allow(non_snake_case, dead_code)]
        pub(crate) fn signal_error(
            env: &mut jni::JNIEnv,
            sink: &jni::objects::JObject,
            err: &(impl ::core::fmt::Display + ?Sized),
        ) {
            let __msg = match env.new_string(err.to_string()) {
                Ok(s) => s,
                Err(e) => {
                    tracing::error!("signal_error: new_string failed: {}", e);
                    return;
                }
            };
            if let Err(e) = env.call_method(
                sink,
                "onError",
                "(Ljava/lang/String;)V",
                &[jni::objects::JValue::Object(&__msg)],
            ) {
                tracing::error!("signal_error: onError call failed: {}", e);
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

    /// Hand the registry this back-end's constructor-expansion declarations so
    /// `write_rust` can resolve `.expand`s into fold plans before resolution.
    fn expansions(&self) -> Option<&crate::api::core::expand::Expansions> {
        Some(&self.expansions)
    }

    /// Hand the registry this back-end's output-expansion declarations so
    /// `write_rust` can resolve them into unfold plans before resolution.
    fn deconstructors(&self) -> Option<&crate::api::core::unfold::Deconstructors> {
        Some(&self.deconstructors)
    }

    /// Union of every `.package_fun(...)` list across all
    /// [`Self::package`] contexts. Each entry is a
    /// `#[prebindgen]` fn ident the user explicitly hooked into the
    /// binding; functions not in this set are skipped by the registry's
    /// signature scan and by the per-item emitter.
    fn declared_functions(&self) -> std::collections::HashSet<syn::Ident> {
        let mut out = std::collections::HashSet::new();
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
        // `JniBindingError` String-wrapper. Built-in converter bodies compose
        // their `?` failures into this type via its `From<String>` impl. A
        // `Result<T, E>` return instead binds its own raw `E`; both funnel to
        // the per-call `signal_error` sink (generic over `Display`).
        let error_type = framework_error_type();
        let alias: syn::Item = syn::parse_quote!(
            #[allow(dead_code)]
            pub(crate) type __JniErr = #error_type;
        );
        let mut items = vec![alias];
        items.extend(owned_object_prerequisite_items());
        // The single `signal_error` channel fn the extern bodies call on any
        // `Err`. Emitted above the converters so wrapper code references it by
        // bare name; the binding crate reaches it as
        // `<include_module>::signal_error` from outside the file.
        items.push(build_signal_error_item());
        let _ = registry;
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
                    value_rust_key: None,
                    projection: None,
                },
            });
        }
        // `Option<T>` for a direct opaque handle, BY VALUE (consume): wire is
        // `jlong` with `0` = `None`; when present the `Box` is reconstructed
        // and `T` moved out (owned), mirroring the by-value `T` consume path.
        // Produces `Option<T>` (not `Option<OwnedObject<T>>`) so the source
        // fn's `Option<T>` parameter type matches. The Kotlin side nulls the
        // handle's `ptr` slot after the call (see `ConsumeNullable`).
        if pat_match(pat, "Option < _ >") {
            let inner = registry.input_entry(t1)?;
            if inner.metadata.is_direct_handle() {
                let inner_wire = inner.destination.clone();
                let outer_ty: syn::Type = syn::parse_quote!(Option<#t1>);
                let name = input_name(&outer_ty, &inner_wire);
                let function: syn::ItemFn = syn::parse_quote!(
                    #[allow(non_snake_case, unused_mut, unused_variables, unused_braces, dead_code)]
                    pub(crate) unsafe fn #name<'env, 'v>(
                        env: &mut jni::JNIEnv<'env>,
                        v: &#inner_wire,
                    ) -> ::core::result::Result<Option<#t1>, __JniErr> {
                        Ok({
                            if *v == 0 {
                                None
                            } else {
                                Some(*std::boxed::Box::from_raw(*v as *mut #t1))
                            }
                        })
                    }
                );
                let kotlin_name =
                    self.override_kotlin_name(&outer_ty, inner.metadata.kotlin_name.clone());
                let projection = inner.metadata.projection.clone().map(|h| Projection {
                    owned: true,
                    // Rides the inner's `*v == 0` niche, so the wire stays
                    // `jlong` and `None` is the `0` sentinel (never JVM boxed).
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
                        value_rust_key: None,
                        projection,
                    },
                });
            }
            // Non-opaque inner: fall through to the general Option handler.
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
        // `str` is unsized, so it has no by-value output converter — but it is
        // reached as the sub of a `&str` reference accessor leaf. Resolve it to
        // the same `&str → jstring` fn the rank-1 `&str` arm uses (deduped by
        // name) so required-propagation doesn't flag it unresolved.
        if TypeKey::from_type(ty).as_str() == "str" {
            return Some(self.str_ref_output());
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
        // Borrowed string slice output (`&str` / `&'a str`): the converter used
        // for a zero-copy reference accessor return (`f(&T) -> &str`, output
        // expansion). The single copy into the JVM is `&str → jstring` (no
        // intermediate owned `String`). The unsized `str` sub resolves via the
        // rank-0 arm to the same fn (see [`Self::str_ref_output`]).
        if let syn::Type::Reference(r) = pat {
            if r.mutability.is_none() && TypeKey::from_type(t1).as_str() == "str" {
                return Some(self.str_ref_output());
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
            // A **non-projection** `Option<T>` return (`Option<String>`,
            // `Option<i64>`, …) surfaces directly as a nullable Kotlin type, so
            // its value-context name carries the `?`. Projection options get the
            // `?` from `render_handle_type(Nullable …)` at the use site instead,
            // so leave those untouched here.
            let kotlin_name = if projection.is_none() {
                kotlin_name.map(|n| if n.ends_with('?') { n } else { format!("{n}?") })
            } else {
                kotlin_name
            };
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

}
