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
        let kotlin_name = self.override_kotlin_name(&outer_ty, Some(kt::KtType::string()));
        let niches = default_niches_for_wire(&wire);
        ConverterImpl {
            subs: vec![],
            pre_stages: vec![],
            function: self.build_output_fn(&outer_ty, &wire, &body, None),
            destination: wire,
            niches,
            metadata: self.framework_meta(kotlin_name),
        }
    }

    /// `Cow<[u8]>` output converter (any lifetime form) — see the call site
    /// in [`Self::output_terminal`]. `None` when `ty` isn't a
    /// `Cow<…, [u8]>` path.
    fn cow_bytes_output(&self, ty: &syn::Type) -> Option<ConverterImpl<KotlinMeta>> {
        let syn::Type::Path(tp) = ty else { return None };
        let seg = tp.path.segments.last()?;
        if seg.ident != "Cow" {
            return None;
        }
        let syn::PathArguments::AngleBracketed(ab) = &seg.arguments else {
            return None;
        };
        let inner_is_bytes = ab.args.iter().any(|a| {
            matches!(a, syn::GenericArgument::Type(t) if TypeKey::from_type(t).as_str() == "[u8]")
        });
        if !inner_is_bytes {
            return None;
        }
        // The generated fn's param type must be resolvable without imports —
        // normalize whatever path form the accessor wrote to the full one.
        let norm_ty: syn::Type = syn::parse_quote!(::std::borrow::Cow<'_, [u8]>);
        let wire: syn::Type = syn::parse_quote!(jni::objects::JByteArray);
        let body: syn::Expr = syn::parse_quote!({
            env.byte_array_from_slice(&v).map_err(|e| {
                <__JniErr as ::core::convert::From<String>>::from(format!(
                    "encode_byte_array: {}",
                    e
                ))
            })?
        });
        let kotlin_name = self.override_kotlin_name(ty, Some(kt::KtType::byte_array()));
        let niches = default_niches_for_wire(&wire);
        Some(ConverterImpl {
            subs: vec![],
            pre_stages: vec![],
            function: self.build_output_fn(&norm_ty, &wire, &body, None),
            destination: wire,
            niches,
            metadata: self.framework_meta(kotlin_name),
        })
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
            subs: vec![],
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
    /// [`FoldStrategy::Base`]). The single seam where a Rust type is
    /// first marked a closeable native handle.
    fn opaque_leaf_meta(&self, ty: &syn::Type) -> KotlinMeta {
        KotlinMeta {
            projection: Some(Projection {
                leaf_key: TypeKey::from_type(ty).as_str().to_string(),
                owned: true,
                strategy: FoldStrategy::Base,
                kind: ProjectionKind::Handle,
            }),
            ..self.framework_meta(Some(kt::KtType::cls("Long")))
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
        inherited: Option<kt::KtType>,
    ) -> Option<kt::KtType> {
        let key = TypeKey::from_type(outer_ty);
        if let Some(cfg) = self.types.get(&key) {
            // Opaque-handle entries keep their typed FQN in
            // `name_spec` for FQN-consumers, but the value-context
            // name is `"Long"` (set on the rank-0 handler's metadata).
            // Don't let that FQN leak into a wrapper's metadata.
            if cfg.opaque.is_none() {
                if let Some(spec) = &cfg.name_spec {
                    return Some(kt::KtType::cls(self.fqn_of(spec)));
                }
            }
        }
        inherited
    }

    /// Canonical input-converter name for `(rust, wire)` — exposed
    /// for plugin wrapper exts that build `ConverterImpl::function`
    /// manually with a non-standard return type (e.g.
    /// `impl Into<…>` parameters that can't be expressed via
    /// `input_wrapper_shape`'s fixed signature shape).
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
            subs: vec![],
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
/// Strip a single leading `&` (one level) from a type, leaving non-references
/// unchanged. Used to reach a `Vec`/slice element's bare type for nomination.
fn peel_leading_ref(ty: &syn::Type) -> syn::Type {
    match ty {
        syn::Type::Reference(r) => (*r.elem).clone(),
        other => other.clone(),
    }
}

/// True for the `String` builtin (final path segment `String`) — the one
/// undeclared type that crosses as a single JObject-shaped leaf (`JString`).
fn is_string_type(ty: &syn::Type) -> bool {
    matches!(ty, syn::Type::Path(tp)
        if tp.path.segments.last().is_some_and(|s| s.ident == "String"))
}

pub(crate) fn build_signal_error_item() -> syn::Item {
    syn::parse_quote!(
        #[allow(non_snake_case, dead_code)]
        pub(crate) fn signal_error(
            env: &mut jni::JNIEnv,
            sink: &jni::objects::JObject,
            mid: &::prebindgen::lang::CachedIfaceMethod,
            fqn: &str,
            descr: &str,
            je: ::core::option::Option<&str>,
            ze: &[jni::sys::jvalue],
        ) {
            // If a JVM exception is already pending (a Java upcall threw during a
            // converter), let it propagate untouched — do NOT invoke the error
            // callback over it (and do not clear/describe it: that would swallow
            // the real exception). The extern returns its sentinel and the pending
            // exception surfaces when control returns to the JVM.
            if env.exception_check().unwrap_or(false) {
                return;
            }
            // `je` (binding message, `Some` only for a `JniError`) crosses as the
            // fixed first `String?`; the `ze` library-error leaves follow as
            // pre-encoded object-slot jvalues — all nullable object params of
            // the sink's typed `<Err>Handler.run` (`mid`/`fqn`/`descr` are the
            // per-extern cached interface method).
            let __je: jni::objects::JObject = match je {
                ::core::option::Option::Some(__m) => match env.new_string(__m) {
                    Ok(s) => s.into(),
                    Err(e) => {
                        tracing::error!("signal_error: new_string failed: {}", e);
                        return;
                    }
                },
                ::core::option::Option::None => jni::objects::JObject::null(),
            };
            let mut __args: ::std::vec::Vec<jni::sys::jvalue> =
                ::std::vec::Vec::with_capacity(1 + ze.len());
            __args.push(jni::sys::jvalue { l: __je.as_raw() });
            __args.extend_from_slice(ze);
            // On failure leave any pending exception in place (don't describe/
            // clear it) so it propagates rather than being swallowed.
            if let Err(e) = mid.call_object(env, fqn, "run", descr, sink, &__args) {
                tracing::error!("signal_error: error-callback invoke failed: {}", e);
            }
        }
    )
}

/// One `#[no_mangle] extern "C"` destructor per opaque handle — the Rust
/// counterpart to the `public fun free() = free {
/// freePtr<suffix>(it) }` / `private external fun freePtr<suffix>` pair
/// emitted by [`render_typed_handle_source`] — so the framework owns *both*
/// halves of the destructor for every typed-handle class. Each body is the
/// uniform `drop(Box::from_raw(ptr as *mut T))`; the inner `T`'s own `Drop`
/// runs (e.g. `Publisher` network-undeclare) with no special casing.
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
pub(crate) fn build_handle_destructor_items(
    ext: &JniGen,
    registry: &Registry<KotlinMeta>,
) -> Vec<syn::Item> {
    let free_ptr = ext.mangle_fun("freePtr");
    let mut named: Vec<(String, syn::Item)> = Vec::new();
    for (key, cfg) in &ext.types {
        if cfg.opaque.is_none() {
            continue;
        }
        // Skip handles the (feature-aware) scan never references — their
        // type may not be in scope in the generated module.
        let ty = key.to_type();
        if registry.input_entry(&ty).is_none() && registry.output_entry(&ty).is_none() {
            continue;
        }
        let class_fqn = cfg
            .name_spec
            .as_ref()
            .map(|s| ext.fqn_of(s))
            .unwrap_or_else(|| {
                panic!(
                    "build_handle_destructor_items: opaque handle `{}` has no \
                     name spec to derive a destructor symbol from",
                    key.as_str()
                )
            });
        let class_short = class_fqn.rsplit('.').next().unwrap_or(&class_fqn);
        let class_pkg = class_fqn
            .rsplit_once('.')
            .map(|(pkg, _)| pkg)
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

/// Per-shape **input** wrapper converter builders (`&`/`Option<&>`/`Vec`/
/// `Option`). Each returns `Some(ConverterImpl)` only for the wildcard pattern
/// it claims; [`JniGen::input_wrapper_shape`] chains them in priority order.
/// Because [`pat_match`] is an exact match, the patterns are disjoint — except
/// the two `Option<_>` sub-cases (direct-handle-by-value vs general), which
/// share a pattern and so live together in [`JniGen::input_option`] to keep
/// their original fall-through.
impl JniGen {
    /// `& _` / `& mut _` borrow: share T's resolved converter — `&T`'s entry
    /// points at the same `ItemFn` (the fn returns owned `T`; the call site in
    /// `emit_jni_function_wrapper` adds `&decoded`). Exists so the
    /// wildcard-substitution machinery marks T required transitively from `&T`.
    fn input_borrow(
        &self,
        pat: &syn::Type,
        t1: &syn::Type,
        registry: &Registry<KotlinMeta>,
    ) -> Option<ConverterImpl<KotlinMeta>> {
        if !(pat_match(pat, "& _") || pat_match(pat, "& mut _")) {
            return None;
        }
        let inner = registry.input_entry(t1)?;
        let outer_ty: syn::Type = if pat_match(pat, "& mut _") {
            syn::parse_quote!(&mut #t1)
        } else {
            syn::parse_quote!(&#t1)
        };
        // `&T` / `&mut T` are Kotlin-side no-ops — inherit the inner
        // type's name, unless the user pinned an explicit override
        // on the outer form itself (rare but legal).
        let kotlin_name = self.override_kotlin_name(&outer_ty, inner.metadata.kotlin_name.clone());
        // The outer form shares T's converter function verbatim, so it
        // inherits T's throws behaviour. A borrowed handle (mut or not) is
        // still opaque (param classification needs to see it), but the holder
        // doesn't own it — mark `owned: false` so `close()` emission skips it.
        let projection = inner
            .metadata
            .projection
            .clone()
            .map(|h| Projection { owned: false, ..h });
        Some(ConverterImpl {
            subs: vec![],
            destination: inner.destination.clone(),
            function: inner.function.clone(),
            pre_stages: vec![],
            niches: inner.niches.clone(),
            metadata: KotlinMeta {
                kotlin_name,
                value_rust_key: None,
                projection,
            },
        })
    }

    /// `Option<&T>` / `Option<&mut T>` for opaque T: returns
    /// `Option<OwnedObject<T>>` (the call site `.as_deref()` coerces back).
    /// `None` for non-opaque inners — the resolver then offers `Option<_>`
    /// over `&T` and the general handler takes it.
    fn input_option_ref(
        &self,
        pat: &syn::Type,
        t1: &syn::Type,
        registry: &Registry<KotlinMeta>,
    ) -> Option<ConverterImpl<KotlinMeta>> {
        if !(pat_match(pat, "Option < & _ >") || pat_match(pat, "Option < & mut _ >")) {
            return None;
        }
        let inner = registry.input_entry(t1)?;
        if !inner.metadata.is_direct_handle() {
            // Non-opaque: let the general `Option<_>` handler take it.
            return None;
        }
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
        let kotlin_name = self.override_kotlin_name(&outer_ty, inner.metadata.kotlin_name.clone());
        let projection = inner.metadata.projection.clone().map(|h| Projection {
            owned: false,
            // `Option<&Handle>` always rides the inner's `*v == 0` niche
            // (body is `if *v == 0 { None } else { ... }` above), so
            // null is the `0i64` sentinel — never JVM boxed.
            strategy: FoldStrategy::Optional(NullableKind::Niche, Box::new(h.strategy)),
            ..h
        });
        Some(ConverterImpl {
            subs: vec![],
            pre_stages: vec![],
            function,
            destination: inner_wire,
            niches: Niches::empty(),
            metadata: KotlinMeta {
                kotlin_name,
                value_rust_key: None,
                projection,
            },
        })
    }

    /// `Vec<T>` (input side): wire is `JObject` carrying a Java
    /// `List<InnerWire>`; iterate, decode each element via the inner converter,
    /// collect into a `Vec`. (`Vec<u8>` is special-cased at rank-0.)
    fn input_vec(
        &self,
        pat: &syn::Type,
        t1: &syn::Type,
        registry: &Registry<KotlinMeta>,
    ) -> Option<ConverterImpl<KotlinMeta>> {
        if !pat_match(pat, "Vec < _ >") {
            return None;
        }
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
            // `List` is auto-imported in Kotlin (default imports).
            Some(kt::KtType::generic("List", [inner_kotlin])),
        );
        Some(ConverterImpl {
            subs: vec![],
            pre_stages: vec![],
            function: self.build_input_fn(&outer_ty, &wire, &body, None),
            destination: wire,
            niches: Niches::empty(),
            metadata: KotlinMeta {
                kotlin_name,
                value_rust_key: None,
                projection: None,
            },
        })
    }

    /// `Option<T>`: first the direct-opaque-handle by-value consume (wire
    /// `jlong`, `0` = `None`, `Box` reconstructed and `T` moved out), then —
    /// when the inner isn't a direct handle — the general nullable fold. The
    /// two share the `Option<_>` pattern, so they stay in one method to keep
    /// the original sequential fall-through.
    fn input_option(
        &self,
        pat: &syn::Type,
        t1: &syn::Type,
        registry: &Registry<KotlinMeta>,
    ) -> Option<ConverterImpl<KotlinMeta>> {
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
                    strategy: FoldStrategy::Optional(NullableKind::Niche, Box::new(h.strategy)),
                    ..h
                });
                return Some(ConverterImpl {
                    subs: vec![],
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
            // destination and `None` is the niche slot sentinel; the boxed
            // fallback widens the wire to `JObject`.
            let nullable_kind = nullable_kind_for(&wire, t1, registry);
            let projection = registry
                .input_entry(t1)
                .and_then(|e| e.metadata.projection.clone())
                .map(|h| Projection {
                    strategy: FoldStrategy::Optional(nullable_kind, Box::new(h.strategy)),
                    ..h
                });
            return Some(ConverterImpl {
                subs: vec![],
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

    /// True when `elem` crosses the boundary as a **single leaf** the foreign
    /// side can reassemble from one wire value — a value blob (→ `ByteArray`) or
    /// the `String` builtin (→ `JString`). Multi-field `data_class` elements
    /// (whose output is a `fromParts` object), enums, and opaque handles are
    /// excluded. Drives [`Self::leaf_vec_fold_elements`].
    ///
    /// Classified from the adapter's declared [`TypeConfig`] table (and the
    /// `String` builtin), not the resolver's output converters — this runs
    /// **before** type resolution, exactly like [`Self::value_struct_decons`].
    fn is_leaf_vec_element(&self, elem: &syn::Type) -> bool {
        match self.types.get(&TypeKey::from_type(elem)) {
            // A declared value blob crosses as a single `ByteArray` leaf; a
            // declared opaque handle crosses as a single `jlong` (pointer) leaf
            // that the Kotlin folder wraps into its typed handle class. Enums
            // and multi-field data classes are not leaf-folded — data classes go
            // through `value_struct_decons`.
            Some(cfg) => cfg.value_blob || cfg.opaque.is_some(),
            // Undeclared: only the `String` builtin crosses as a single
            // JObject-shaped leaf (`JString`); other builtins are primitives.
            None => is_string_type(elem),
        }
    }
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

    // ── Structural type resolution ──────────────────────────────────────
    // Try the terminal categories, then the user-wrapper table (`match_user_*`,
    // any depth, specificity-ordered), then the built-in wrapper shapes — peel
    // `ty`'s outermost layer and dispatch to `{input,output}_wrapper_shape` with
    // the reconstructed canonical pattern. `subs` = the captured inner(s).

    fn on_input_type(
        &self,
        ty: &syn::Type,
        registry: &Registry<KotlinMeta>,
    ) -> Option<ConverterImpl<KotlinMeta>> {
        self.select_input_type(ty, registry)
    }

    fn on_output_type(
        &self,
        ty: &syn::Type,
        registry: &Registry<KotlinMeta>,
    ) -> Option<ConverterImpl<KotlinMeta>> {
        self.select_output_type(ty, registry)
    }

    /// Hand the registry this back-end's constructor-expansion declarations so
    /// `write_rust` can resolve `.expand`s into fold plans before resolution.
    /// Assembled on demand from the per-fn overrides plus the raw type-level
    /// [`ParamExpandDecl`]s (see [`JniGen::build_expansions`]).
    fn expansions(&self) -> Option<crate::api::core::expand::Expansions> {
        Some(self.build_expansions())
    }

    /// Hand the registry this back-end's output-expansion declarations so
    /// `write_rust` can resolve them into unfold plans before resolution.
    /// Assembled on demand — field names (member inheritance) resolve here,
    /// against the complete declaration set (see
    /// [`JniGen::build_deconstructors`]).
    fn deconstructors(&self) -> Option<crate::api::core::unfold::Deconstructors> {
        Some(self.build_deconstructors())
    }

    /// Synthesize a field-decomposition for every `.data_class` type whose
    /// fields the fixed builder can forward verbatim (see
    /// [`synth_value_struct_leaves`]). The result drives
    /// [`crate::api::core::unfold::apply_value_structs`] so such a struct
    /// crosses Rust→Kotlin as decoupled leaves (reassembled by the generated
    /// `fromParts` builder singleton) instead of a `JObject` built on the Rust
    /// side via `call_static_method`. Types the synthesizer declines (enums /
    /// projections / `Option`/`Vec`-nested) keep the whole-value
    /// [`struct_output_body`] path.
    fn value_struct_decons(
        &self,
        registry: &Registry<KotlinMeta>,
    ) -> Vec<crate::api::core::unfold::ValueDecon> {
        let mut out = Vec::new();
        for (ident, (item_struct, _loc)) in &registry.structs {
            let source: syn::Type = syn::parse_quote!(#ident);
            let key = TypeKey::from_type(&source);
            // A `data_class` is a registered type that is neither an opaque
            // handle, an enum, nor a value blob.
            let is_data_class = matches!(
                self.type_kind(registry, &source),
                TypeKind::DataStruct { cfg: Some(c), .. } if c.name_spec.is_some()
            );
            if !is_data_class {
                continue;
            }
            if let Some(leaves) = crate::api::lang::jnigen::jni::synth_value_struct_leaves(
                self,
                registry,
                item_struct,
                &[],
                "",
                0,
            ) {
                if !leaves.is_empty() {
                    out.push(crate::api::core::unfold::ValueDecon {
                        key,
                        source,
                        leaves,
                    });
                }
            }
        }
        out
    }

    /// Nominate every **single-leaf** element type that appears in a `Vec<T>` /
    /// `Option<Vec<T>>` return or an `impl Fn(&[T])` callback arg, so
    /// [`crate::api::core::unfold::apply_leaf_vec_folds`] routes the collection
    /// through a foreign-built fold (no Rust `ArrayList`). A single-leaf element
    /// is a value blob (→ `ByteArray`), an opaque handle (→ a `jlong` pointer
    /// the Kotlin folder wraps into its typed handle class), or a non-`data_class`
    /// builtin with a JObject-shaped output wire (e.g. String). Multi-field
    /// `data_class` elements are excluded — they go through
    /// [`Self::value_struct_decons`].
    fn leaf_vec_fold_elements(&self, registry: &Registry<KotlinMeta>) -> Vec<syn::Type> {
        let mut seen = std::collections::HashSet::new();
        let mut out = Vec::new();
        let mut consider = |bare: syn::Type| {
            if seen.insert(TypeKey::from_type(&bare)) && self.is_leaf_vec_element(&bare) {
                out.push(bare);
            }
        };
        for (item_fn, _loc) in registry.functions.values() {
            // `Vec<T>` / `Option<Vec<T>>` return.
            if let syn::ReturnType::Type(_, ret) = &item_fn.sig.output {
                let after_opt =
                    crate::api::core::types_util::option_inner_type(ret).unwrap_or((**ret).clone());
                if let Some(elem) = crate::api::core::types_util::vec_inner_type(&after_opt) {
                    consider(peel_leading_ref(&elem));
                }
            }
            // `impl Fn(&[T])` / `impl Fn([T])` callback arg.
            for input in &item_fn.sig.inputs {
                let syn::FnArg::Typed(pt) = input else {
                    continue;
                };
                let Some(args) = crate::api::core::registry::extract_fn_trait_args(&pt.ty) else {
                    continue;
                };
                for arg in args {
                    if let syn::Type::Slice(s) = &peel_leading_ref(&arg) {
                        consider(peel_leading_ref(&s.elem));
                    }
                }
            }
        }
        out
    }

    /// Union of every `.fun(...)` list across all
    /// [`Self::package`] subpackage contexts. Each entry is a
    /// `#[prebindgen]` fn ident the user explicitly hooked into the
    /// binding; functions not in this set are skipped by the registry's
    /// signature scan and by the per-item emitter.
    fn declared_functions(&self) -> std::collections::HashSet<syn::Ident> {
        let mut out = std::collections::HashSet::new();
        for pkg in self.packages.values() {
            for m in &pkg.functions {
                out.insert(m.rust_ident.clone());
            }
            // Function-backed constants (`constant_fun`) are ordinary
            // declared functions on the Rust/extern side; only their Kotlin
            // surface differs (an eagerly-initialized top-level `val`).
            for m in &pkg.constant_functions {
                out.insert(m.rust_ident.clone());
            }
        }
        // Class members (accessor/method/constructor) are declared via
        // `.accessor`/`.method`/`.constructor` (not `.fun`) but are still real
        // `#[prebindgen]` wrappers: they need a Rust extern + JNINative
        // `external fun` + JSONL inclusion. Only their Kotlin surface differs
        // (an instance method or companion factory instead of a free fn).
        out.extend(
            self.class_members
                .values()
                .flatten()
                .map(|m| m.rust_ident.clone()),
        );
        out
    }

    /// Functions ever referenced as a named leaf in a `return_expand!` `.field(fun!(...))`/
    /// `.return_expand(...)` record — see
    /// `accessor_record_fns`'s doc (`jni/mod.rs`). Usage-derived, not tied to
    /// `.fun()` class-member declarations: a function need not also be
    /// exposed as an instance method to be referenced this way.
    fn accessor_functions(&self) -> std::collections::HashSet<syn::Ident> {
        let mut set = self.accessor_record_fns.clone();
        for decl in &self.return_expand_decls {
            for f in &decl.fields {
                if let LocalField::Named(func, _) = f {
                    set.insert(func.clone());
                }
            }
        }
        set
    }

    /// Fun members (`.fun`) — their fn ident mapped to the owning class's
    /// `TypeKey`, so input-flattening can skip the receiver parameter.
    fn method_receivers(&self) -> std::collections::HashMap<syn::Ident, TypeKey> {
        self.class_members
            .iter()
            .flat_map(|(key, ms)| {
                ms.iter()
                    .filter(|m| m.kind == MemberKind::Fun)
                    .map(move |m| (m.rust_ident.clone(), key.clone()))
            })
            .collect()
    }

    /// Every type registered via one of the four **class declarators**
    /// (`.ptr_class` / `.enum_class` / `.data_class` / `.value_class`).
    /// These are the only structs/enums the per-item emitter walks, and the
    /// scan requires them in BOTH directions (their converters always resolve
    /// both ways). Wrapper-only registrations are deliberately excluded: a
    /// wrapper type is required per **usage** direction, so an output-only
    /// wrapper needs no input twin.
    fn declared_types(&self) -> std::collections::HashSet<TypeKey> {
        self.types
            .iter()
            .filter(|(_, c)| c.class_decl)
            .map(|(k, _)| k.clone())
            .collect()
    }

    /// Union of every `.constant(...)` list across all
    /// [`Self::package`] subpackage contexts. `Some` even when empty — JniGen
    /// HAS a const declaration mechanism, so const emission is declared-only
    /// and undeclared consts get the skip warning (see
    /// [`Prebindgen::declared_consts`]).
    /// The declared value types of every expression constant
    /// (`PackageDecl::constant_expr`) — they have no `#[prebindgen]` item to
    /// scan, so the resolver is told directly to produce their output
    /// converters.
    fn required_output_types(&self) -> Vec<syn::Type> {
        self.packages
            .values()
            .flat_map(|p| p.constant_exprs.iter().map(|e| e.ty.clone()))
            .collect()
    }

    fn declared_consts(&self) -> Option<std::collections::HashSet<syn::Ident>> {
        let mut out = std::collections::HashSet::new();
        for pkg in self.packages.values() {
            for c in &pkg.constants {
                out.insert(c.rust_ident.clone());
            }
        }
        Some(out)
    }

    /// Consts acknowledged-but-unexposed via [`JniGen::ignore_const`].
    fn ignored_consts(&self) -> std::collections::HashSet<syn::Ident> {
        self.ignored_const_idents.clone()
    }

    /// Fns acknowledged-but-unbound via [`JniGen::ignore_fun`] — suppresses
    /// the registry's "skipping undeclared" warning, emits nothing.
    fn ignored_functions(&self) -> std::collections::HashSet<syn::Ident> {
        self.ignored_fns.clone()
    }

    /// Types acknowledged-but-undeclared via [`JniGen::ignore_class`].
    fn ignored_types(&self) -> std::collections::HashSet<TypeKey> {
        self.ignored_class_types.clone()
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
        // Slice/Vec input helpers — a `…VecNew/Push/Free` trio per flattenable
        // element type a scanned `&[T]`/`Vec<T>` param takes. Kotlin builds the
        // Rust-side `Vec` by pushing each element's decoupled leaves, then passes
        // the handle (see `ParamMode::VecBuild`), avoiding per-element
        // `env.get_field(...)` upcalls on the Rust side.
        items.extend(build_vec_build_helper_items(self, registry));
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
        // Expression constants — one nullary JNI getter extern per
        // `PackageDecl::constant_expr`, its value the binding-defined
        // expression evaluated with `use <source_module>::*;` in scope (so
        // it composes the source crate's items without qualification). The
        // getter reuses the whole function-wrapper pipeline via the
        // synthetic signature, exactly like a const-backed getter.
        let source_module = &self.source_module;
        for decl in self.packages.values().flat_map(|p| &p.constant_exprs) {
            validate_constant_expr(self, &decl.kotlin_name, &decl.ty);
            let getter = const_expr_getter_fn(&decl.kotlin_name, &decl.ty);
            let expr = &decl.expr;
            let callee: syn::Expr = syn::parse_quote!({
                #[allow(unused_imports)]
                use #source_module::*;
                #expr
            });
            let wrapper =
                emit_jni_function_wrapper_with_callee(self, &getter, registry, Some(callee));
            items.push(syn::parse2::<syn::Item>(wrapper).expect(
                "constant_expr: generated getter wrapper is a single item by construction",
            ));
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
        // input_terminal / output_terminal below; no separate
        // per-struct item is needed.
        TokenStream::new()
    }

    fn on_enum(&self, _e: &syn::ItemEnum, _registry: &Registry<KotlinMeta>) -> TokenStream {
        TokenStream::new()
    }

    fn source_module(&self) -> Option<&syn::Path> {
        Some(&self.source_module)
    }

    /// Declared consts only reach here (write gating via
    /// [`Prebindgen::declared_consts`]): re-emit the const as a path-alias
    /// to its source-of-truth (initializer tokens are never copied — they
    /// may reference source-crate internals) AND emit its nullary JNI getter
    /// extern. The getter reuses the whole function-wrapper pipeline (so the
    /// const's type flows through the ordinary output-converter machinery);
    /// only the callee expression differs — a path to the const, not a call.
    fn on_const(&self, c: &syn::ItemConst, registry: &Registry<KotlinMeta>) -> TokenStream {
        // Unnamed infrastructure consts (`const _`, e.g. the injected
        // `konst::assertc_eq!` feature guard) pass through verbatim — no
        // getter, no Kotlin surface.
        if c.ident == "_" {
            return c.to_token_stream();
        }
        reject_handle_const(self, c);
        let getter = const_getter_fn(c);
        let const_ident = &c.ident;
        let source_module = &self.source_module;
        let callee: syn::Expr = syn::parse_quote!(#source_module::#const_ident);
        let wrapper = emit_jni_function_wrapper_with_callee(self, &getter, registry, Some(callee));
        let alias = crate::api::core::const_path_alias(c, source_module);
        quote! {
            #alias
            #wrapper
        }
    }

    fn dispatch_fn_input(
        &self,
        args: &[syn::Type],
        registry: &Registry<KotlinMeta>,
    ) -> Option<ConverterImpl<KotlinMeta>> {
        let outer_ty = build_fn_type(args);
        let (wire, body) = callback_input(self, args, registry)?;
        let niches = default_niches_for_wire(&wire);
        // `impl Fn(...)` crosses the extern tier as the erased lambda object
        // (`Any`) — same as the unfold builder / error-sink params. The typed
        // wrapper-level lambda signature is computed at render time from the
        // arg types' callback plans, not carried in metadata.
        Some(ConverterImpl {
            subs: vec![],
            pre_stages: vec![],
            function: self.build_input_fn(&outer_ty, &wire, &body, None),
            destination: wire,
            niches,
            metadata: self.framework_meta(Some(kt::KtType::any())),
        })
    }
}

/// Structural converter builders — the rank-0 terminal chains and the rank-1
/// wrapper-shape handlers, now inherent helpers called by the structural
/// [`Prebindgen::on_input_type`] / [`Prebindgen::on_output_type`].
impl JniGen {
    // ── Input converters ─────────────────────────────────────────────

    /// Whole-type **input** terminal categories (opaque handle, value-blob,
    /// enum, the rank-0 user table, `str`, primitive, struct) — depends on
    /// nothing, `subs` empty.
    pub(crate) fn input_terminal(
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
                subs: vec![],
                pre_stages: vec![],
                function: self.build_input_fn(ty, &wire, &body, None),
                destination: wire,
                niches,
                metadata: KotlinMeta {
                    projection: Some(Projection {
                        leaf_key: key.as_str().to_string(),
                        owned: false,
                        strategy: FoldStrategy::Base,
                        kind: ProjectionKind::ValueBlob,
                    }),
                    ..self.framework_meta(Some(kt::KtType::cls("ByteArray")))
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
                        let kotlin_name = cfg
                            .name_spec
                            .as_ref()
                            .map(|s| kt::KtType::cls(self.fqn_of(s)));
                        return Some(ConverterImpl {
                            subs: vec![],
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
            let kotlin_name = self.override_kotlin_name(ty, Some(kt::KtType::string()));
            let niches = default_niches_for_wire(&wire);
            return Some(ConverterImpl {
                subs: vec![],
                pre_stages: vec![],
                function: self.build_input_fn(&rust_ty, &wire, &body, None),
                destination: wire,
                niches,
                metadata: self.framework_meta(kotlin_name),
            });
        }
        // `Box<String>`: a heap string carried as an opaque-pointer struct field
        // (e.g. an FFI-safe `#[repr(C)]` struct's `Option<Box<String>>`). Decode
        // the `JString` to an owned `String` and box it; surfaces as Kotlin
        // `String` (and `Option<Box<String>>` composes to `String?` via the
        // `Option<_>` wrapper). Dual of the `Box<String>` output arm.
        if TypeKey::from_type(ty).as_str() == "Box < String >" {
            let wire: syn::Type = syn::parse_quote!(jni::objects::JString);
            let body: syn::Expr = syn::parse_quote!({
                let s = env.get_string(v).map_err(|e| {
                    <__JniErr as ::core::convert::From<String>>::from(format!(
                        "decode_string: {}",
                        e
                    ))
                })?;
                ::std::boxed::Box::new(::std::string::String::from(s))
            });
            let rust_ty: syn::Type = syn::parse_quote!(::std::boxed::Box<::std::string::String>);
            let kotlin_name = self.override_kotlin_name(ty, Some(kt::KtType::string()));
            let niches = default_niches_for_wire(&wire);
            return Some(ConverterImpl {
                subs: vec![],
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
                subs: vec![],
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
                let kotlin_name = self
                    .types
                    .get(&key)
                    .and_then(|c| c.name_spec.as_ref())
                    .map(|s| kt::KtType::cls(self.fqn_of(s)));
                return Some(ConverterImpl {
                    subs: vec![],
                    pre_stages: vec![],
                    function: self.build_input_fn(ty, &wire, &body, None),
                    destination: wire,
                    niches,
                    metadata: self.framework_meta(kotlin_name),
                });
            }
            // Bare-ident enum: leave to the consuming crate to override
            // (today's CongestionControl etc. fall here — caller's wrapper
            // ext returns Some in its own input_terminal).
        }
        None
    }

    /// **Input** wrapper shape (`pat` = the reconstructed canonical pattern,
    /// `t1` = its captured inner): the rank-1 user table, then the built-in
    /// `&`/`Option<&>`/`Vec`/`Option` handlers.
    pub(crate) fn input_wrapper_shape(
        &self,
        pat: &syn::Type,
        t1: &syn::Type,
        registry: &Registry<KotlinMeta>,
    ) -> Option<ConverterImpl<KotlinMeta>> {
        if let Some(conv) = self.lookup_input(pat, std::slice::from_ref(t1), registry) {
            return Some(conv);
        }
        // Disjoint wildcard patterns (see the `impl JniGen` block above), tried
        // in priority order. The borrow/option-ref/vec patterns are exact and
        // mutually exclusive; the two `Option<_>` sub-cases share a method.
        self.input_borrow(pat, t1, registry)
            .or_else(|| self.input_option_ref(pat, t1, registry))
            .or_else(|| self.input_vec(pat, t1, registry))
            .or_else(|| self.input_option(pat, t1, registry))
    }

    // ── Output converters ────────────────────────────────────────────

    /// Whole-type **output** terminal categories (the dual of
    /// [`Self::input_terminal`]: opaque handle, value-blob, enum, user table,
    /// `str`, `Cow<[u8]>`, unit, primitive, struct) — `subs` empty.
    pub(crate) fn output_terminal(
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
                subs: vec![],
                pre_stages: vec![],
                function: self.build_output_fn(ty, &wire, &body, None),
                destination: wire,
                niches,
                metadata: KotlinMeta {
                    projection: Some(Projection {
                        leaf_key: key.as_str().to_string(),
                        owned: false,
                        strategy: FoldStrategy::Base,
                        kind: ProjectionKind::ValueBlob,
                    }),
                    ..self.framework_meta(Some(kt::KtType::cls("ByteArray")))
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
                        let kotlin_name = cfg
                            .name_spec
                            .as_ref()
                            .map(|s| kt::KtType::cls(self.fqn_of(s)));
                        return Some(ConverterImpl {
                            subs: vec![],
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
        // `Box<String>`: read the heap string through the box and encode it as a
        // `JString`; surfaces as Kotlin `String` (and `Option<Box<String>>` →
        // `String?` via the `Option<_>` wrapper). Dual of the `Box<String>`
        // input arm — together they let an opaque-pointer `String` struct field
        // map to a plain Kotlin `String`.
        if TypeKey::from_type(ty).as_str() == "Box < String >" {
            let wire: syn::Type = syn::parse_quote!(jni::objects::JString);
            let body: syn::Expr = syn::parse_quote!({
                env.new_string(v.as_str()).map_err(|e| {
                    <__JniErr as ::core::convert::From<String>>::from(format!("encode_str: {}", e))
                })?
            });
            let rust_ty: syn::Type = syn::parse_quote!(::std::boxed::Box<::std::string::String>);
            let kotlin_name = self.override_kotlin_name(ty, Some(kt::KtType::string()));
            let niches = default_niches_for_wire(&wire);
            return Some(ConverterImpl {
                subs: vec![],
                pre_stages: vec![],
                function: self.build_output_fn(&rust_ty, &wire, &body, None),
                destination: wire,
                niches,
                metadata: self.framework_meta(kotlin_name),
            });
        }
        // `Cow<'_, [u8]>` (any lifetime): a borrow-or-owned byte container —
        // one copy into the JVM array straight off the `Deref<[u8]>`, no
        // intermediate owned `Vec` (the zero-copy dual of the `Vec<u8>`
        // output, for accessors like `zenoh::ZBytes::to_bytes()` that borrow
        // when the payload is contiguous). Surfaces as Kotlin `ByteArray`.
        if let Some(conv) = self.cow_bytes_output(ty) {
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
                subs: vec![],
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
                subs: vec![],
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
                let kotlin_name = self
                    .types
                    .get(&key)
                    .and_then(|c| c.name_spec.as_ref())
                    .map(|s| kt::KtType::cls(self.fqn_of(s)));
                return Some(ConverterImpl {
                    subs: vec![],
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

    /// **Output** wrapper shape (the dual of [`Self::input_wrapper_shape`]):
    /// the rank-1 user table, then the built-in `&Handle`/`&str`/`Option`/`Vec`
    /// handlers. An `Option<&Handle>` resolves via the shallow `Option<_>`.
    pub(crate) fn output_wrapper_shape(
        &self,
        pat: &syn::Type,
        t1: &syn::Type,
        registry: &Registry<KotlinMeta>,
    ) -> Option<ConverterImpl<KotlinMeta>> {
        if let Some(conv) = self.lookup_output(pat, std::slice::from_ref(t1), registry) {
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
                    subs: vec![],
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
                    strategy: FoldStrategy::Optional(nullable_kind, Box::new(h.strategy)),
                    ..h
                });
            // A **non-projection** `Option<T>` return (`Option<String>`,
            // `Option<i64>`, …) surfaces directly as a nullable Kotlin type, so
            // its value-context name carries the `?`. Projection options get the
            // `?` from `handle_kt_type(Nullable …)` at the use site instead,
            // so leave those untouched here.
            let kotlin_name = if projection.is_none() {
                kotlin_name.map(|n| if n.is_nullable() { n } else { n.nullable() })
            } else {
                kotlin_name
            };
            return Some(ConverterImpl {
                subs: vec![],
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
            // `Vec<opaque-handle>` output is delivered by the Kotlin-side leaf
            // fold (`apply_leaf_vec_folds` → typed-handle wrap), so this
            // whole-`ArrayList` converter is bypassed for it. A handle's `jlong`
            // wire isn't JObject-shaped, so it returns `None` below; the
            // fold-covered return is de-required, so the `None` is not an error.
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
                // `List` is auto-imported in Kotlin (default imports). When
                // the inner carries a projection, this wire-context name
                // still drives non-projection consumers; projection-aware
                // sites (classify_return, data-class fields) prefer
                // `projection` and render the typed `List<TypedShort>`
                // instead.
                Some(kt::KtType::generic("List", [inner_kotlin])),
            );
            // Fold an Iterable layer over the inner projection (if any), so
            // `Vec<Handle>` / `Vec<ValueClass>` carry the full strategy.
            let projection = inner.metadata.projection.clone().map(|h| Projection {
                strategy: FoldStrategy::Iterable(Box::new(h.strategy)),
                ..h
            });
            // The list conversion always builds a fresh non-null `ArrayList`, so
            // `JObject` null is a free niche — lets `Option<Vec<T>>` ride it
            // (None ⇒ null list) instead of needing a boxed wrapper.
            let niches = default_niches_for_wire(&wire);
            return Some(ConverterImpl {
                subs: vec![],
                pre_stages: vec![],
                function: self.build_output_fn(&outer_ty, &wire, &body, None),
                destination: wire,
                niches,
                metadata: KotlinMeta {
                    kotlin_name,
                    value_rust_key: None,
                    projection,
                },
            });
        }
        None
    }

    /// `&[T]` borrowed-slice output (used for a **callback argument** that crosses
    /// native→JVM, e.g. `impl Fn(&[Payload])`). The borrowed dual of the `Vec<T>`
    /// output handler above: build a `java.util.ArrayList<InnerWire>` by iterating
    /// the slice **by reference** and cloning each element through its output
    /// converter (`v.iter()` + `Clone::clone` instead of `into_iter()`). Surfaces
    /// as Kotlin `List<T>`. The element must have a JObject-shaped output wire
    /// (struct / String / …) — scalar slices are not handled here.
    pub(crate) fn output_slice(
        &self,
        elem: &syn::Type,
        registry: &Registry<KotlinMeta>,
    ) -> Option<ConverterImpl<KotlinMeta>> {
        let inner = registry.output_entry(elem)?;
        // A `&[opaque-handle]` callback arg is delivered by the Kotlin-side leaf
        // fold (typed-handle wrap), bypassing this whole-`ArrayList` converter; a
        // handle's `jlong` wire isn't JObject-shaped, so it returns `None` here.
        let inner_wire = inner.destination.clone();
        if !is_jobject_shaped_wire(&inner_wire) {
            return None;
        }
        let inner_conv = inner.function.sig.ident.clone();
        let outer_ty: syn::Type = syn::parse_quote!(&[#elem]);
        let wire: syn::Type = syn::parse_quote!(jni::objects::JObject);
        let body: syn::Expr = syn::parse_quote!({
            let __list_obj = env
                .new_object("java/util/ArrayList", "()V", &[])
                .map_err(|e| <__JniErr as ::core::convert::From<String>>::from(format!("&[_]: new ArrayList: {}", e)))?;
            let __list = jni::objects::JList::from_env(env, &__list_obj)
                .map_err(|e| <__JniErr as ::core::convert::From<String>>::from(format!("&[_]: list-from-env: {}", e)))?;
            for __elem in v.iter() {
                let __elem_wire = #inner_conv(env, ::core::clone::Clone::clone(__elem))?;
                let __elem_obj: jni::objects::JObject = __elem_wire.into();
                __list.add(env, &__elem_obj)
                    .map_err(|e| <__JniErr as ::core::convert::From<String>>::from(format!("&[_]: list-add: {}", e)))?;
            }
            __list_obj
        });
        let inner_kotlin = inner.metadata.kotlin_name.clone()?;
        let kotlin_name =
            self.override_kotlin_name(&outer_ty, Some(kt::KtType::generic("List", [inner_kotlin])));
        let projection = inner.metadata.projection.clone().map(|h| Projection {
            strategy: FoldStrategy::Iterable(Box::new(h.strategy)),
            ..h
        });
        let niches = default_niches_for_wire(&wire);
        Some(ConverterImpl {
            subs: vec![elem.clone()],
            pre_stages: vec![],
            function: self.build_output_fn(&outer_ty, &wire, &body, None),
            destination: wire,
            niches,
            metadata: KotlinMeta {
                kotlin_name,
                value_rust_key: None,
                projection,
            },
        })
    }
}
