//! [`Prebindgen`] implementation for [`JniGenBuilder`] plus its converter-
//! selector / exception-routing helpers.
//!
//! Carved from the former monolithic JNI module; shares the `jni`
//! namespace via `use super::*`.

use super::*;
use crate::api::core::registry::{Building, Conversions, Crossing, RegistryBuilder};

/// The `#[allow(...)]` carried by every generated converter `fn`.
///
/// Generated converters are uniform templates, not hand-written idiomatic Rust,
/// so beyond the name / unused suppressions this allows the clippy lints those
/// templates inherently trip — none of which flag a real issue in generated
/// code, and which are only avoidable by contorting the emitted code:
/// * `needless_question_mark` — a range-checked input's `Ok(try_from(..)?)`;
/// * `let_and_return` — a multi-stage conversion fold's trailing `let`;
/// * `nonminimal_bool` / `eq_op` — a representation-domain guard whose bounds
///   are the scalar's min/max (`true &&`) or which has no exclusions (`!(false)`).
fn generated_converter_attr() -> syn::Attribute {
    syn::parse_quote!(#[allow(
        non_snake_case,
        unused_mut,
        unused_variables,
        unused_braces,
        // A representation-agnostic converter says the same thing for every
        // spelling, so the plain spelling gets the degenerate form of it: a
        // reflexive `.into()`, a deref that is a no-op, parens around a value
        // that needed none. Suppressing per-shape would mean asking which
        // spelling this is, which is the guessing #270 removed.
        unused_parens,
        dead_code,
        clippy::useless_conversion,
        clippy::needless_question_mark,
        clippy::let_and_return,
        clippy::nonminimal_bool,
        clippy::eq_op
    )])
}

// ──────────────────────────────────────────────────────────────────────
// Inherent helpers — wrapper builders (used by both Prebindgen impl
// and consuming-crate wrapper exts like ZenohJniExt).
// ──────────────────────────────────────────────────────────────────────

impl Declarations {
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
        let gen_allow = generated_converter_attr();
        if matches!(wire, syn::Type::Ptr(_)) {
            syn::parse_quote!(
                #gen_allow
                pub(crate) unsafe fn #name<'env>(env: &mut jni::JNIEnv<'env>, v: #wire) -> ::core::result::Result<#rust_with_lifetime, #err_type> {
                    #ret_body
                }
            )
        } else {
            syn::parse_quote!(
                #gen_allow
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
        let gen_allow = generated_converter_attr();
        syn::parse_quote!(
            #gen_allow
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
    /// * Input: `OwnedObject::from_raw(*v as *const T)` (borrow only),
    ///   after rejecting null and tag-bit-set values — bit 0 is the
    ///   Kotlin-side closed tag (see `NativeHandle`), so an odd `jlong`
    ///   is a handle that was closed after the wrapper's pre-lock guard;
    ///   it must never be dereferenced.
    /// * Niche: `0i64` / `*v == 0` — `Box::into_raw` never returns 0,
    ///   so `Option<T>` automatically synthesises `0` = `None`,
    ///   matching the legacy "null pointer" ABI for nullable handles.
    ///   A *tagged* (closed-but-present) value is an error, not `None`.
    pub fn opaque_handle_input(&self, ty: &syn::Type) -> ConverterImpl<KotlinMeta> {
        let wire: syn::Type = syn::parse_quote!(jni::sys::jlong);
        let name = input_name(ty, &wire);
        let gen_allow = generated_converter_attr();
        let function: syn::ItemFn = syn::parse_quote!(
            #gen_allow
            pub(crate) unsafe fn #name<'env, 'v>(
                env: &mut jni::JNIEnv<'env>,
                v: &jni::sys::jlong,
            ) -> ::core::result::Result<OwnedObject<#ty>, __JniErr> {
                // Null or tag-bit-set (closed handle raced past the Kotlin
                // pre-lock guard) — reject before any dereference.
                if *v == 0 || (*v & 1) == 1 {
                    return ::core::result::Result::Err(
                        <__JniErr as ::core::convert::From<String>>::from(
                            "Operation on a closed native handle.".to_string(),
                        ),
                    );
                }
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
                leaf_key: TypeKey::from_type(ty),
                owned: true,
                strategy: FoldStrategy::Base,
                kind: ProjectionKind::Handle,
                niche_sentinels: Vec::new(),
            }),
            ..self.framework_meta(Some(kt::KtType::cls("Long")))
        }
    }

    /// Leaf metadata for Rust `u64`: the JNI value-context stays `Long`, while
    /// projection-aware Kotlin emitters surface `ULong` and insert the
    /// bit-preserving `toLong()` / `toULong()` bridge.
    fn unsigned64_leaf_meta(&self) -> KotlinMeta {
        KotlinMeta {
            projection: Some(Projection {
                leaf_key: TypeKey::parse("u64").expect("builtin type key"),
                owned: false,
                strategy: FoldStrategy::Base,
                kind: ProjectionKind::Unsigned64,
                niche_sentinels: Vec::new(),
            }),
            ..self.framework_meta(Some(kt::KtType::long()))
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
            if !cfg.is_opaque() {
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

    fn emitted_source_type_names(
        &self,
        registry: &Registry<KotlinMeta>,
    ) -> std::collections::HashMap<String, syn::Path> {
        let mut names = std::collections::HashMap::new();
        let mut add = |key: &TypeKey| {
            if let Some(short) = rust_short_name_opt(key) {
                // Per-item origin when the type has an indexed
                // `#[prebindgen]` item; else the default module (a declared
                // type re-exported by the primary source, or a deliberately
                // unmarked type like a convert!-only newtype).
                // Parsed, not constructed: a short name is whatever the source
                // wrote, and `Ident::new` PANICS on a raw one (`r#type`)
                // rather than erroring. Pre-existing; found by the raw-name
                // regression added for the sum encoder's twin of this bug.
                let Ok(ident) = syn::parse_str::<syn::Ident>(&short) else {
                    return;
                };
                let module = registry
                    .origin_module(&ident)
                    .unwrap_or_else(|| self.default_module(registry));
                names.insert(short, module);
            }
        };
        for key in self.types.keys() {
            add(key);
        }
        // Rust-side-only boundary types are absent from the type table but
        // still appear in emitted signatures (e.g. the `E` of a peeled
        // `Result<T, E>`), so they need the same qualification.
        for (key, _) in self.rust_side_only_types().collect::<Vec<_>>() {
            add(&key);
        }
        // `convert!`-declared types likewise have no type-table entry but
        // appear in emitted converter signatures.
        for decl in &self.convert_decls {
            add(&decl.key);
        }
        names
    }

    /// Walk `item` and prefix every bare single-segment type reference
    /// matching a [`Self::emitted_source_type_names`] name with that name's
    /// origin module. Applied once per emitted item at write
    /// time via [`Prebindgen::post_process_item`] so converter bodies,
    /// type ascriptions, and casts all stay in sync without each emit
    /// site having to remember to qualify.
    fn qualify_item(&self, item: &mut syn::Item, registry: &Registry<KotlinMeta>) {
        let source_names = self.emitted_source_type_names(registry);
        // Names reachable from an array LENGTH (`[u8; MAX]`, `[u8; Holder::N]`).
        //
        // Registry-wide, NOT the declared-surface `source_names`: a length's
        // owner is a compile-time namespace, not a boundary type. Requiring it
        // to be declared would force an otherwise-unused Kotlin class into
        // existence just to make the generated Rust compile, and would be
        // asymmetric with consts, which qualify whether or not JniGenBuilder declared
        // them.
        // EVERY named item the registry indexes. A length is an arbitrary const
        // expression, so it can name a const, the type owning an associated
        // const, or a `const fn` — and enumerating item KINDS here missed one
        // of those three twice, so the enumeration lives in core
        // (`named_item_idents`) where a new kind is added once.
        //
        // The NAME SET is independent of origin stamps and the VALUE falls back
        // to the default module: an origin-less hand-built stream holds elements
        // whose location carries no crate name, and those still need qualifying
        // (core documents `crate` as their module).
        let length_names: std::collections::HashMap<String, syn::Path> = registry
            .named_item_idents()
            .map(|ident| {
                let module = registry
                    .origin_module(ident)
                    .unwrap_or_else(|| self.default_module(registry));
                (ident.to_string(), module)
            })
            .collect();
        let mut visitor = QualifyEmittedTypes {
            source_names: &source_names,
            length_names: &length_names,
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

/// The pending-exception guard shared by both signal helpers: if a JVM
/// exception is already pending (a Java upcall threw during a converter), let
/// it propagate untouched — do NOT invoke an error callback over it, and do
/// not clear/describe it (that would swallow the real exception). The extern
/// returns its sentinel and the pending exception surfaces when control
/// returns to the JVM.
pub(crate) fn build_signal_binding_error_item() -> syn::Item {
    syn::parse_quote!(
        #[allow(non_snake_case, dead_code)]
        pub(crate) fn signal_binding_error(
            env: &mut jni::JNIEnv,
            sink: &jni::objects::JObject,
            mid: &::prebindgen::lang::CachedIfaceMethod,
            fqn: &str,
            descr: &str,
            je: &str,
        ) {
            if env.exception_check().unwrap_or(false) {
                return;
            }
            // The binding message crosses as the single `String` param of the
            // `JniErrorHandler.run` (`mid`/`fqn`/`descr` are the per-extern
            // cached interface method).
            let __je: jni::objects::JObject = match env.new_string(je) {
                Ok(s) => s.into(),
                Err(e) => {
                    tracing::error!("signal_binding_error: new_string failed: {}", e);
                    return;
                }
            };
            let __args = [jni::sys::jvalue { l: __je.as_raw() }];
            // On failure leave any pending exception in place (don't describe/
            // clear it) so it propagates rather than being swallowed.
            if let Err(e) = mid.call_object(env, fqn, "run", descr, sink, &__args) {
                tracing::error!("signal_binding_error: error-callback invoke failed: {}", e);
            }
        }
    )
}

/// Invoke a fallible function's typed **domain** error handler
/// (`<Src>Handler.run(ze…)`) with the pre-encoded decomposed-error leaves.
/// There is no leading `je` and no defaults — this is called ONLY on `Err(E)`.
pub(crate) fn build_signal_domain_error_item() -> syn::Item {
    syn::parse_quote!(
        #[allow(non_snake_case, dead_code)]
        pub(crate) fn signal_domain_error(
            env: &mut jni::JNIEnv,
            sink: &jni::objects::JObject,
            mid: &::prebindgen::lang::CachedIfaceMethod,
            fqn: &str,
            descr: &str,
            ze: &[jni::sys::jvalue],
        ) {
            if env.exception_check().unwrap_or(false) {
                return;
            }
            // On failure leave any pending exception in place (don't describe/
            // clear it) so it propagates rather than being swallowed.
            if let Err(e) = mid.call_object(env, fqn, "run", descr, sink, ze) {
                tracing::error!("signal_domain_error: error-callback invoke failed: {}", e);
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
/// `Java_<package_underscores>_<class_short>_<mangled-freePtr>`,
/// where `class_short` is the last segment of the typed-handle FQN
/// (`TypeConfig::kotlin_name`) and the `freePtr` name passes through
/// the package/class-aware method hook — exact symmetry with the Kotlin
/// `external fun <mangled-freePtr>` declaration in
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
    ext: &Declarations,
    registry: &Registry<KotlinMeta>,
) -> Vec<syn::Item> {
    let mut named: Vec<(String, syn::Item)> = Vec::new();
    for (key, cfg) in &ext.types {
        if !cfg.is_opaque() {
            continue;
        }
        // Skip handles the (feature-aware) scan never references — their
        // type may not be in scope in the generated module. Keyed directly:
        // this used to spell the key into tokens purely so `reading_of` could
        // re-key them, twice (#291).
        let Some(reading) = registry.reading(key) else {
            continue;
        };
        if registry.input_entry(&reading).is_none() && registry.output_entry(&reading).is_none() {
            continue;
        }
        let ty = reading.spell().clone();
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
        let class_package = class_fqn.rsplit_once('.').map(|(pkg, _)| pkg).unwrap_or("");
        let free_ptr = ext.mangle_method(class_package, class_short, "freePtr");
        let symbol = super::symbol::native_symbol(class_package, class_short, &free_ptr);
        let ident = syn::Ident::new(&symbol, Span::call_site());
        // Bit 0 of the jlong is the Kotlin-side closed tag, so every handle
        // type must leave it free: `Box` pointers to `T` are `align_of::<T>()`
        // aligned, hence the compile-time floor of 2. Spelled as an `if` +
        // `panic!` (not `assert!`) so the type reference is real AST — the
        // `qualify_item` pass does not descend into macro token streams.
        let item: syn::Item = syn::parse_quote!(
            const _: () = {
                if ::core::mem::align_of::<#ty>() < 2 {
                    panic!(
                        "opaque handle types must have alignment >= 2 (bit 0 is the closed tag)"
                    );
                }
            };
        );
        named.push((format!("{symbol}__align_assert"), item));
        let item: syn::Item = syn::parse_quote!(
            #[no_mangle]
            #[allow(non_snake_case, unused_variables)]
            pub(crate) unsafe extern "C" fn #ident(
                _env: jni::JNIEnv,
                _class: jni::objects::JClass,
                ptr: jni::sys::jlong,
            ) {
                if ptr != 0 && (ptr & 1) == 0 {
                    drop(Box::from_raw(ptr as *mut #ty));
                }
            }
        );
        named.push((symbol, item));
    }
    named.sort_by(|a, b| a.0.cmp(&b.0));
    named.into_iter().map(|(_, item)| item).collect()
}

/// Which built-in wrapper a converter is being built for — **the model's
/// answer, not a guess from the spelling**.
///
/// This used to be a `&syn::Type` wildcard pattern (`Option<_>`, `& mut _`)
/// rebuilt from the type's tokens and compared as a *string*. That made the
/// dispatch depend on how Rust happened to spell the type: `Box<Option<T>>`
/// reconstructed as `Box<_>`, matched no pattern, and got no converter at all
/// (#270) — even though the model classifies it `Optional` and says so.
///
/// So the shape comes from [`TypeKind`](crate::api::core::flat::TypeKind) and
/// the spelling comes from `spell()`, which is the same split the rest of
/// the pipeline follows: classify off `kind`, spell with `spell()`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum WrapperShape {
    /// `Ref` — a borrow of its inner.
    Borrow { mutable: bool },
    /// `Optional` whose inner is a `Ref` — the deep handle-borrow form, tried
    /// before [`Self::Optional`].
    OptionRef { mutable: bool },
    /// `Sequence` — a run of its element.
    Sequence,
    /// `Optional` — its inner, or absent.
    Optional,
}

/// What generated Rust can do with one wrapper the model
/// [erases](crate::api::core::flat::TRANSPARENT_WRAPPERS).
///
/// Erasure and reconstruction are different questions, and only the first is the
/// model's. `Box<T>` *is* `T` to every destination language — but undoing it in
/// Rust is `*b`, undoing a `Cow` is `into_owned()`, and undoing an `Rc` is not
/// possible at all. There is no trait spanning those, so the operations live
/// here, one row per wrapper, instead of as a special case per converter.
///
/// **Adding a wrapper is adding a row.** Put its name in
/// `TRANSPARENT_WRAPPERS` (the model decides what it erases) and a row here
/// (the adapter decides what it can rebuild); `every_erased_wrapper_has_ops`
/// fails if the two disagree, so a wrapper cannot become transparent without
/// this file having an answer for it.
struct WrapperOps {
    /// Its last path segment, as `TRANSPARENT_WRAPPERS` spells it.
    name: &'static str,
    /// Move the inner value **out**. `None` when the representation does not
    /// permit it — a `Cow` payload cannot be moved through `Deref` (`E0507`),
    /// and neither can an `Rc`'s.
    ///
    /// Emitted **unparenthesized**: every consumer splices the result into a
    /// `let` initializer, where a wrapping paren is `unused_parens` — and
    /// generated code runs through the consumer's own lints, where that is a
    /// denial. A consumer that splices into a tighter position (a method
    /// receiver, a field base) parenthesizes at its own site.
    read: Option<fn(TokenStream) -> TokenStream>,
    /// Build it **from** the inner value. `None` when not supported.
    build: Option<fn(TokenStream) -> TokenStream>,
}

/// The operations table. One row per wrapper the model erases.
const WRAPPER_OPS: &[WrapperOps] = &[
    WrapperOps {
        name: "Box",
        // `*b` moves out of a box, and `Box::new` puts it back.
        read: Some(|e| quote!(*#e)),
        build: Some(|e| quote!(::std::boxed::Box::new(#e))),
    },
    WrapperOps {
        name: "Cow",
        // Reading would be `into_owned()`, which needs `B: ToOwned` — not
        // implied by anything the model knows about the payload. Refused until
        // something needs it; that is one row, not a redesign.
        read: None,
        // Building is a DIFFERENT question, and it is refused for a different
        // reason — the two `None`s here are not one fact repeated.
        //
        // `Cow::Owned(v)` is well-typed: an input rebuild owns its value, and
        // `Cow<'_, [T]>` takes a `Vec<T>` while `Cow<'_, str>` takes a
        // `String`. So this is not "cannot", it is **should not**. A source
        // spells `Cow` to accept borrowed data without copying; a binding that
        // can only ever hand it `Owned` pays that copy on every call and
        // silently removes the borrow path — and the callee can see the
        // difference (`matches!(c, Cow::Borrowed(_))`), so it is observable
        // rather than merely wasteful.
        //
        // **Deliberate, not deferred.** If a binding decides the copy is
        // acceptable for its own source, this is one line —
        // `Some(|e| quote!(::std::borrow::Cow::Owned(#e)))` — and nothing else
        // moves. The refusal is here so that decision is made on purpose.
        build: None,
    },
];

fn wrapper_ops(name: &str) -> Option<&'static WrapperOps> {
    WRAPPER_OPS.iter().find(|w| w.name == name)
}

/// The chain of wrappers standing between a **spelling** and the canonical
/// shape its `kind` names, outermost first — empty when the source already
/// wrote the canonical form.
///
/// `None` means the spelling is not a wrapping of the canonical one at all, so
/// no converter should claim it.
fn bridge_layers(spelling: &syn::Type, canonical: &syn::Type) -> Option<Vec<&'static WrapperOps>> {
    if spelling.to_token_stream().to_string() == canonical.to_token_stream().to_string() {
        return Some(Vec::new());
    }
    let (name, inner) = crate::api::core::flat::peel_transparent(spelling)?;
    let ops = wrapper_ops(name)?;
    let mut rest = bridge_layers(&inner, canonical)?;
    rest.insert(0, ops);
    Some(rest)
}

/// Read the converter's `v` as the canonical shape, undoing each layer
/// outside-in. `None` when any layer cannot be read through — the crossing then
/// stays **unresolved**, naming the type, rather than resolving and emitting
/// Rust the consumer cannot build (#270 review).
fn read_as_canonical(produced: &syn::Type, canonical: &syn::Type) -> Option<TokenStream> {
    let layers = bridge_layers(produced, canonical)?;
    let mut e = quote!(v);
    for w in layers {
        e = (w.read?)(e);
    }
    Some(e)
}

/// Build the spelling from a canonical value — the input-side peer, applying
/// each layer inside-out.
fn build_from_canonical(
    produced: &syn::Type,
    canonical: &syn::Type,
    value: TokenStream,
) -> Option<TokenStream> {
    let layers = bridge_layers(produced, canonical)?;
    let mut e = value;
    for w in layers.into_iter().rev() {
        e = (w.build?)(e);
    }
    Some(e)
}

/// Move a value the **source** produced out of the transparent wrappers its
/// spelling adds over its classification, so an emitter that binds it holds the
/// canonical shape — `Box<Option<T>>` → `(*e)`, an unwrapped spelling → `e`
/// unchanged.
///
/// The counterpart of [`bind_as_option`](super::emit::bind_as_option) for an
/// **owned** position. A type-ascribed `let` is a coercion site and serves any
/// representation, but coercion applies to *references*: a value whose payload
/// downstream moves has to be moved out of the wrapper instead, which is what
/// [`WrapperOps::read`] does and what only some wrappers permit.
///
/// `None` when a layer cannot be read through (`Cow`, whose payload cannot be
/// moved out by `Deref`) — the caller then has an unrepresentable crossing to
/// report, and must not emit the match anyway.
///
/// **This answers for one layer's spelling.** It undoes the wrappers standing
/// over `ty`'s own classification; a wrapper *inside* — the `Box` of
/// `Option<Box<Vec<T>>>` — belongs to the inner reading and is that layer's
/// question, per [`TypeRef::erased_wrappers`](crate::api::core::flat::TypeRef::erased_wrappers).
pub(crate) fn read_through_erased_wrappers(
    ty: &crate::api::core::flat::TypeRef,
    e: TokenStream,
) -> Option<TokenStream> {
    let mut out = e;
    // Outermost first, which is the order they have to come off in.
    for name in ty.erased_wrappers() {
        out = (wrapper_ops(name)?.read?)(out);
    }
    Some(out)
}

/// Put back the transparent wrappers a **rebuild** dropped, so a value the
/// emitter constructed from the classification has the type the source spelled
/// — `Box<Option<S>>` ← `Box::new(v)`, an unwrapped spelling ← `v` unchanged.
///
/// The input-side dual of [`read_through_erased_wrappers`], and the reason both
/// live here rather than at the sites that need them: the specialized input
/// lowerings do not *decode* their parameter, they **rebuild** it — a literal
/// `S { .. }`, an `Option::Some(v)`, a `Vec<T>` pushed element by element — and
/// a rebuild from the classification alone produces the *stripped* type. Handing
/// that to a parameter spelled `Box<..>` is an `E0308` in the generated crate,
/// which is why this is one rule in one place instead of three selection sites
/// each remembering it.
///
/// Applied **innermost-out**, the reverse of reading: the value in hand is the
/// canonical shape, and each layer wraps what the previous one produced.
///
/// `None` when any layer has no [`WrapperOps::build`] — `Cow`, by policy rather
/// than by impossibility; see its row. A caller that gets `None` has a crossing
/// it cannot serve and must decline or report it, never emit the bare value.
///
/// **This answers for one layer's spelling.** It restores the wrappers standing
/// over `ty`'s own classification; a wrapper *inside* — the `Box` of
/// `Option<Box<S>>` — belongs to the inner reading, is applied when that layer
/// is built, and is invisible here. An erasure sits **outside** the layer it
/// wraps, so a rebuild collects wrappers as it descends and applies them as it
/// comes back out.
pub(crate) fn build_through_erased_wrappers(
    ty: &crate::api::core::flat::TypeRef,
    value: TokenStream,
) -> Option<TokenStream> {
    build_through_wrappers(&ty.erased_wrappers(), value)
}

/// [`build_through_erased_wrappers`] over a wrapper list already taken off a
/// reading — for a plan that recorded *what to put back* rather than keeping the
/// whole `TypeRef` to ask again.
///
/// The list is the only part of the reading a rebuild uses, and it is two
/// pointers instead of a `TypeRef`'s ~264 bytes. That matters because these
/// plans live in `InputKind`, whose size every variant pays.
pub(crate) fn build_through_wrappers(
    names: &[&'static str],
    value: TokenStream,
) -> Option<TokenStream> {
    let mut out = value;
    for name in names.iter().rev() {
        out = (wrapper_ops(name)?.build?)(out);
    }
    Some(out)
}

/// Whether the source wrote the canonical spelling itself — no wrapper to undo.
///
/// Required by the converters that do **not** produce the spelled type by
/// construction: the borrow shapes hand back the inner type's own converter (or
/// an `OwnedObject`) and let the call site add `&` / `.as_deref()`. There is no
/// value in hand to wrap or unwrap, so a wrapped spelling cannot be served
/// here at all and must not resolve.
fn is_canonical_spelling(produced: &syn::Type, canonical: &syn::Type) -> bool {
    bridge_layers(produced, canonical).is_some_and(|l| l.is_empty())
}

/// Per-shape **input** wrapper converter builders (`&`/`Option<&>`/`Vec`/
/// `Option`). Each returns `Some(ConverterImpl)` only for the [`WrapperShape`]
/// it claims; [`Declarations::input_wrapper_shape`] chains them in priority
/// order. The shapes are disjoint — except the two `Optional` sub-cases
/// (direct-handle-by-value vs general), which share one and so live together in
/// [`Declarations::input_option`] to keep their original fall-through.
///
/// Each takes `produced`: the Rust type the converter's function **yields**.
/// Normally that is the crossing's own spelling, so a `Box<Option<T>>` crossing
/// produces a `Box<Option<T>>` rather than silently declaring `Option<T>` and
/// mismatching its call site. The one deliberate exception is a `&[T]`
/// parameter, which decodes to an owned `Vec<T>` the call site borrows — see
/// [`Declarations::select_input_type`].
impl Declarations {
    /// `& _` / `& mut _` borrow: share T's resolved converter — `&T`'s entry
    /// points at the same `ItemFn` (the fn returns owned `T`; the call site in
    /// `emit_jni_function_wrapper` adds `&decoded`). Exists so the
    /// wildcard-substitution machinery marks T required transitively from `&T`.
    fn input_borrow(
        &self,
        shape: WrapperShape,
        produced: &syn::Type,
        t1: &crate::api::core::flat::TypeRef,
        registry: &impl Conversions<KotlinMeta>,
    ) -> Option<ConverterImpl<KotlinMeta>> {
        // `t1`'s spelling, for the parts that ask spelling questions; the
        // READING stays in `t1` for the lookups (#284).
        let t1_ty = t1.as_syn();
        let WrapperShape::Borrow { mutable } = shape else {
            return None;
        };
        // This converter does NOT produce the spelled type: it hands back the
        // inner type's own entry, and the call site adds the `&`. So there is no
        // value in hand to unwrap a representation from, and a wrapped spelling
        // — `Box<&T>` — must not resolve here (it would pass an owned `T` where
        // `Box<&T>` is expected).
        let canonical: syn::Type = if mutable {
            syn::parse_quote!(&mut #t1_ty)
        } else {
            syn::parse_quote!(&#t1_ty)
        };
        if !is_canonical_spelling(produced, &canonical) {
            return None;
        }
        let inner = registry.input_entry(t1)?;
        let outer_ty = produced.clone();
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
                value_rust_type: None,
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
        shape: WrapperShape,
        produced: &syn::Type,
        t1: &crate::api::core::flat::TypeRef,
        registry: &impl Conversions<KotlinMeta>,
    ) -> Option<ConverterImpl<KotlinMeta>> {
        // `t1`'s spelling, for the parts that ask spelling questions; the
        // READING stays in `t1` for the lookups (#284).
        let t1_ty = t1.as_syn();
        let WrapperShape::OptionRef { mutable } = shape else {
            return None;
        };
        // Produces `Option<OwnedObject<T>>`, which the call site adapts with
        // `.as_deref()` — again not the spelled type, so a wrapped spelling has
        // nothing to bridge and must not resolve. See `input_borrow`.
        let canonical: syn::Type = if mutable {
            syn::parse_quote!(Option<&mut #t1_ty>)
        } else {
            syn::parse_quote!(Option<&#t1_ty>)
        };
        if !is_canonical_spelling(produced, &canonical) {
            return None;
        }
        let inner = registry.input_entry(t1)?;
        if !inner.metadata.is_direct_handle() {
            // Non-opaque: let the general `Option<_>` handler take it.
            return None;
        }
        let inner_wire = inner.destination.clone();
        let inner_conv = inner.function.sig.ident.clone();
        let outer_ty = produced.clone();
        let name = input_name(&outer_ty, &inner_wire);
        let gen_allow = generated_converter_attr();
        let function: syn::ItemFn = syn::parse_quote!(
            #gen_allow
            pub(crate) unsafe fn #name<'env, 'v>(
                env: &mut jni::JNIEnv<'env>,
                v: &#inner_wire,
            ) -> ::core::result::Result<Option<OwnedObject<#t1_ty>>, __JniErr> {
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
                value_rust_type: None,
                projection,
            },
        })
    }

    /// `Vec<T>` (input side): wire is `JObject` carrying a Java
    /// `List<InnerWire>`; iterate, decode each element via the inner converter,
    /// collect into a `Vec`. (`Vec<u8>` is special-cased at rank-0.)
    fn input_vec(
        &self,
        shape: WrapperShape,
        produced: &syn::Type,
        t1: &crate::api::core::flat::TypeRef,
        registry: &impl Conversions<KotlinMeta>,
    ) -> Option<ConverterImpl<KotlinMeta>> {
        // `t1`'s spelling, for the parts that ask spelling questions; the
        // READING stays in `t1` for the lookups (#284).
        let t1_ty = t1.as_syn();
        if shape != WrapperShape::Sequence {
            return None;
        }
        let inner = registry.input_entry(t1)?;
        reject_vec_of_handle(&inner.metadata.projection, t1_ty);
        let inner_wire = inner.destination.clone();
        if !is_jobject_shaped_wire(&inner_wire) {
            return None;
        }
        // The element's COMPLETE wire -> Rust chain: a `convert!` element
        // (`Label` -> `String`) reaches its value through the rust-side stages,
        // not through the wire-facing converter alone.
        let inner_conv = crate::api::lang::jnigen::jni::emit::composed_inner_input(
            inner,
            quote::quote!(&__elem_wire),
        );
        let outer_ty = produced.clone();
        let canonical: syn::Type = syn::parse_quote!(Vec<#t1_ty>);
        // Bridgeable first — see `box_layers_to`.
        let build = build_from_canonical(produced, &canonical, quote::quote!(__out))?;
        let wire: syn::Type = syn::parse_quote!(jni::objects::JObject);
        let body: syn::Expr = syn::parse_quote!({
            let __list = jni::objects::JList::from_env(env, v)
                .map_err(|e| <__JniErr as ::core::convert::From<String>>::from(format!("Vec<_>: list-from-env: {}", e)))?;
            let mut __it = __list.iter(env)
                .map_err(|e| <__JniErr as ::core::convert::From<String>>::from(format!("Vec<_>: list-iter: {}", e)))?;
            let mut __out: Vec<#t1_ty> = Vec::new();
            while let Some(__obj) = __it.next(env)
                .map_err(|e| <__JniErr as ::core::convert::From<String>>::from(format!("Vec<_>: list-next: {}", e)))?
            {
                let __elem_wire: #inner_wire = __obj.into();
                let __elem: #t1_ty = #inner_conv;
                __out.push(__elem);
            }
            #build
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
                value_rust_type: None,
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
        shape: WrapperShape,
        produced: &syn::Type,
        t1: &crate::api::core::flat::TypeRef,
        registry: &impl Conversions<KotlinMeta>,
    ) -> Option<ConverterImpl<KotlinMeta>> {
        // `t1`'s spelling, for the parts that ask spelling questions; the
        // READING stays in `t1` for the lookups (#284).
        let t1_ty = t1.as_syn();
        if shape == WrapperShape::Optional {
            let inner = registry.input_entry(t1)?;
            if inner.metadata.is_direct_handle() {
                let inner_wire = inner.destination.clone();
                let outer_ty = produced.clone();
                let canonical: syn::Type = syn::parse_quote!(Option<#t1_ty>);
                let build = build_from_canonical(produced, &canonical, quote::quote!(__v))?;
                let name = input_name(&outer_ty, &inner_wire);
                let gen_allow = generated_converter_attr();
                let function: syn::ItemFn = syn::parse_quote!(
                    #gen_allow
                    pub(crate) unsafe fn #name<'env, 'v>(
                        env: &mut jni::JNIEnv<'env>,
                        v: &#inner_wire,
                    ) -> ::core::result::Result<#outer_ty, __JniErr> {
                        Ok({
                            let __v: ::core::option::Option<#t1_ty> = if *v == 0 {
                                None
                            } else if (*v & 1) == 1 {
                                // Tagged (closed) handle raced past the Kotlin
                                // pre-lock guard — present-but-closed is an
                                // error, absent is None.
                                return ::core::result::Result::Err(
                                    <__JniErr as ::core::convert::From<String>>::from(
                                        "Operation on a closed native handle.".to_string(),
                                    ),
                                );
                            } else {
                                Some(*std::boxed::Box::from_raw(*v as *mut #t1_ty))
                            };
                            #build
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
                        value_rust_type: None,
                        projection,
                    },
                });
            }
            // Non-opaque inner: fall through to the general Option handler.
        }
        if shape == WrapperShape::Optional {
            let outer_ty = produced.clone();
            let canonical: syn::Type = syn::parse_quote!(Option<#t1_ty>);
            let build = build_from_canonical(produced, &canonical, quote::quote!(__v))?;
            let (wire, inner_body, niches) = option_input(t1_ty, registry)?;
            // `option_input` yields the canonical `Option<T>`; the converter
            // yields the spelling.
            let body: syn::Expr = syn::parse_quote!({
                let __v: ::core::option::Option<#t1_ty> = #inner_body;
                #build
            });
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
            let nullable_kind = nullable_kind_for(&wire, t1_ty, registry);
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
    /// side can reassemble from one wire value — a declared opaque handle
    /// (→ a `jlong` pointer), the `String` builtin (→ `JString`), or the `u64`
    /// scalar projection (→ a raw `jlong` the folder wraps into `ULong`).
    /// Multi-field `data_class` elements (whose output is a `fromParts`
    /// object) and enums are excluded. Drives
    /// [`Self::leaf_vec_fold_elements`].
    ///
    /// Classified from the adapter's declared [`TypeConfig`] table (and the
    /// `String` builtin), not the resolver's output converters — this runs
    /// **before** type resolution, exactly like [`Self::value_struct_decons`].
    fn is_leaf_vec_element(&self, elem: &syn::Type) -> bool {
        match self.types.get(&TypeKey::from_type(elem)) {
            // A declared opaque handle crosses as a single `jlong` (pointer)
            // leaf that the Kotlin folder wraps into its typed handle class.
            // Enums and multi-field data classes are not leaf-folded — data
            // classes go through `value_struct_decons`.
            Some(cfg) => cfg.is_opaque(),
            // Undeclared: `String` is JObject-shaped; `u64` is the built-in
            // scalar projection whose raw jlong leaf the Kotlin folder wraps
            // into `ULong`. Other primitive collections retain their existing
            // unsupported status (`Vec<u8>` is the rank-0 ByteArray special).
            None => is_string_type(elem) || TypeKey::from_type(elem).as_str() == "u64",
        }
    }
}

// ──────────────────────────────────────────────────────────────────────
// Prebindgen impl
// ──────────────────────────────────────────────────────────────────────

impl JniGenBuilder {
    /// State this binding into `registry`: what it exports, what crosses, and
    /// what it defines itself.
    ///
    /// **Push, not pull.** The registry does not call back to ask — the build
    /// script calls this, and the registry stays a passive recorder. That is
    /// what makes "a converter reads a half-built registry" unrepresentable
    /// rather than merely avoided.
    ///
    /// Order-independent, and idempotent apart from the local-fn collision
    /// check: every method it calls records rather than derives.
    /// State this binding into `registry`, then resolve it.
    ///
    /// The pair is always used together, and the generator is what knows both
    /// halves — so it drives, and the registry never calls back. Becomes the
    /// body of `generate(..)` once emission moves here too (#251 phase E).
    /// Read the source, resolve every crossing, and hand back the binding.
    ///
    /// Runs the whole pipeline a build script used to run by hand: parse the
    /// declared sources into a model, describe this binding over it, answer
    /// each crossing in dependency order, and check the set is complete. A
    /// `Flat` and a `Registry` exist inside — they are simply not this caller's
    /// problem.
    pub fn build(self) -> Result<JniGen, crate::core::WriteRustError> {
        let flat = self
            .sources
            .clone()
            .build()
            .map_err(crate::core::ScanError::from)?;
        let registry = crate::core::Registry::builder(flat)?;
        self.build_with(registry)
    }

    /// [`Self::build`] over a registry that was described elsewhere.
    ///
    /// The seam tests use to feed synthetic items without a source directory;
    /// `build` is this with the model read from [`Self::source`].
    ///
    /// **This is the phase change.** The declarations are taken out of the
    /// builder here and never put back: everything below runs against
    /// `&decls`, and what it produces is stored in a [`JniGen`], which has no
    /// route to a `JniGenBuilder` at all.
    pub(crate) fn build_with(
        self,
        registry: crate::api::core::registry::RegistryBuilder<KotlinMeta>,
    ) -> Result<JniGen, crate::core::WriteRustError> {
        let decls = self.decls;
        let registry = decls
            .declare_into(registry)?
            .validate_with(&decls)?
            .convert_with(|crossing, built| decls.convert_crossing(crossing, built))?
            .build()?;
        // Post-resolve invariants, run once here so the writers are pure reads
        // and a `JniGen` is valid by construction.
        decls
            .validate_resolved(&registry)
            .map_err(|message| crate::core::ScanError::AdapterInvariant { message })?;
        Ok(JniGen { decls, registry })
    }
}

impl Declarations {
    /// Build the conversion for one crossing, against what is already built.
    ///
    /// `None` is *cannot*, never *not yet*: `crossings` hands them out
    /// inner-first, so everything this could compose from is already in `built`.
    fn convert_crossing(
        &self,
        crossing: &Crossing,
        built: &Building<'_, KotlinMeta>,
    ) -> Option<ConverterImpl<KotlinMeta>> {
        let (dir, key) = crossing;
        // The reading the scan already took for this crossing, fetched by the
        // key the crossing IS. This used to go `key -> to_type() -> reading`,
        // and its own comment called that "the same door, one layer out" as the
        // round trip #263 removed from `api/core`. The door is now keyed, so
        // there is no spelling to rebuild (#284).
        let reading = built.reading(key)?;
        match dir {
            Direction::Input => self.select_input_type(&reading, built).or_else(|| {
                // `impl Fn(args)` that nothing else claimed. Callback args cross
                // in the OPPOSITE direction, which is why their required-ness
                // rides `immediate_edges` rather than this converter's `subs`.
                // The arguments are `TypeRef`s on the classification, so nothing
                // is re-extracted from the signature's syntax.
                let crate::api::core::flat::TypeKind::Callback { args } =
                    reading.unwrapped().kind()
                else {
                    return None;
                };
                self.dispatch_fn_input(args, built)
            }),
            Direction::Output => self.select_output_type(&reading, built),
        }
    }

    pub fn declare_into(
        &self,
        mut registry: RegistryBuilder<KotlinMeta>,
    ) -> Result<RegistryBuilder<KotlinMeta>, crate::core::ScanError> {
        // Binding-local fns first: they become model, and everything below may
        // name one.
        for (item_fn, origin) in self.collect_local_functions() {
            registry = registry.local_function(item_fn, origin)?;
        }

        for ident in self.declared_functions() {
            registry = registry.export(&ident);
        }
        for ident in self.helper_functions() {
            registry = registry.reference(&ident);
        }
        // JniGenBuilder HAS a const mechanism, so const emission is declared-only even
        // when nothing is declared.
        registry = registry.declares_consts();
        for ident in self.declared_consts().into_iter().flatten() {
            registry = registry.export_const(&ident);
        }
        for ty in self.declared_types().into_values() {
            registry = registry.export_type(ty);
        }
        for ident in self.accessor_functions() {
            registry = registry.accessor(&ident);
        }
        for (ident, receiver) in self.method_receivers() {
            registry = registry.method_receiver(&ident, receiver);
        }

        // An expression constant's value type has no captured item to scan.
        for ty in self.required_output_types() {
            registry = registry.cross(Direction::Output, &ty);
        }
        // The other-side type of every `convert!` conversion, in the
        // conversion's direction: an input fn's parameter type needs its own
        // input converter for the composed body to chain through; an output
        // fn's return type needs the output twin.
        let mut convert_edges: Vec<(Crossing, Crossing)> = Vec::new();
        for decl in &self.convert_decls {
            if let Some((ty, _, _)) = self.convert_input_body(&decl.key, &registry) {
                registry = registry.cross(Direction::Input, &ty);
                // The target's conversion chains through this one, and nothing
                // about the target type says so.
                convert_edges.push((
                    (Direction::Input, decl.key.clone()),
                    (Direction::Input, TypeKey::from_type(&ty)),
                ));
            }
            if let Some((ty, _, _)) = self.convert_output_body(&decl.key, &registry) {
                registry = registry.cross(Direction::Output, &ty);
                convert_edges.push((
                    (Direction::Output, decl.key.clone()),
                    (Direction::Output, TypeKey::from_type(&ty)),
                ));
            }
        }
        for (from, on) in convert_edges {
            registry = registry.depends(from, on);
        }
        // How composites cross in pieces. Every one of these reads only the
        // model, which is what lets them be stated here rather than asked for
        // mid-resolve.
        let decompositions = crate::core::Decompositions {
            expansions: Some(self.build_expansions()),
            deconstructors: Some(self.build_deconstructors(&registry)),
            value_structs: self.build_value_struct_decons(&registry),
            sums: self.build_sum_decons(&registry),
            leaf_vec_elements: self.build_leaf_vec_fold_elements(&registry),
            replaces: self.boundary_only_types(),
        };
        registry = registry.decompose(decompositions);
        Ok(registry)
    }
}

impl Declarations {
    pub(crate) fn build_value_struct_decons(
        &self,
        registry: &impl Conversions<KotlinMeta>,
    ) -> Vec<crate::api::core::unfold::ValueDecon> {
        let mut out = Vec::new();
        for (ident, item_struct) in registry.flat().types().filter_map(|t| match t {
            crate::api::core::flat::Type::Struct(s) => Some((&s.name, s)),
            _ => None,
        }) {
            let source: syn::Type = syn::parse_quote!(#ident);
            let key = TypeKey::from_type(&source);
            // A `data_class` is a registered type that is neither an opaque
            // handle nor an enum.
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

    pub(crate) fn build_sum_decons(
        &self,
        registry: &impl Conversions<KotlinMeta>,
    ) -> Vec<crate::api::core::unfold::SumDecon> {
        let mut keys: Vec<&TypeKey> = self.types.keys().collect();
        keys.sort_by(|a, b| a.as_str().cmp(b.as_str()));
        let mut out = Vec::new();
        for key in keys {
            let Some(sum_cfg) = self.types[key].sum() else {
                continue;
            };
            // The `sealed_class!` declaration's own spelling. This runs during
            // the declare phase, where a `reading()` would legitimately answer
            // `None` for a type nothing has interned yet — the declaration is
            // the only thing that can say (#291).
            let source = self.types[key].rust_type.as_syn().clone();
            let Some(ident) = bare_path_ident(&source) else {
                continue;
            };
            let Some(crate::api::core::flat::Type::Variant(sum)) =
                registry.flat().declared_type(&ident)
            else {
                continue;
            };
            out.push(crate::api::core::unfold::SumDecon {
                key: key.clone(),
                source,
                leaves: crate::api::lang::jnigen::jni::synth_sum_leaves(self, sum_cfg, sum),
            });
        }
        out
    }

    pub(crate) fn build_leaf_vec_fold_elements(
        &self,
        registry: &impl Conversions<KotlinMeta>,
    ) -> Vec<syn::Type> {
        let mut seen = std::collections::HashSet::new();
        let mut out = Vec::new();
        let mut consider = |bare: syn::Type| {
            if seen.insert(TypeKey::from_type(&bare)) && self.is_leaf_vec_element(&bare) {
                out.push(bare);
            }
        };
        for f in registry.flat().functions() {
            // `Vec<T>` / `Option<Vec<T>>` return. The model's `ret` already
            // normalizes an elided return to `()`, so there is no arm for it.
            {
                let ret = f.ret.as_syn();
                let after_opt =
                    crate::api::core::types_util::option_inner_type(ret).unwrap_or(ret.clone());
                if let Some(elem) = crate::api::core::types_util::vec_inner_type(&after_opt) {
                    consider(peel_leading_ref(&elem));
                }
            }
            // `impl Fn(&[T])` / `impl Fn([T])` callback arg. Over the model's
            // params, whose readings already say which ones ARE callbacks —
            // walking `sig.inputs` re-extracted that from the bounds.
            for p in &f.params {
                let Some(args) = p.ty.callback_args() else {
                    continue;
                };
                for arg in args {
                    if let syn::Type::Slice(s) = &peel_leading_ref(arg.as_syn()) {
                        consider(peel_leading_ref(&s.elem));
                    }
                }
            }
        }
        out
    }
}

impl Declarations {
    fn dispatch_fn_input(
        &self,
        args: &[crate::api::core::flat::TypeRef],
        registry: &impl Conversions<KotlinMeta>,
    ) -> Option<ConverterImpl<KotlinMeta>> {
        let spellings: Vec<syn::Type> = args.iter().map(|a| a.as_syn().clone()).collect();
        let outer_ty = build_fn_type(&spellings);
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

impl Prebindgen for Declarations {
    /// Cross-language extras every JNI converter carries — currently
    /// the Kotlin value-context type name. Filled by the rank-N
    /// handlers at the same point they build the wire/body; the
    /// resolver propagates it into [`crate::api::core::registry::TypeEntry::metadata`];
    /// the Kotlin emitter reads it back to drive every wrapper /
    /// typed-handle / `JNIWrappers` signature.
    type Metadata = KotlinMeta;

    // ── Structural type resolution ──────────────────────────────────────
    // Try the terminal categories, then the `Result` peel, then the built-in
    // wrapper shapes — peel
    // `ty`'s outermost layer and dispatch to `{input,output}_wrapper_shape` with
    // the reconstructed canonical pattern. `subs` = the captured inner(s).

    /// Member-shape invariants (N5), checked against registry signatures —
    /// the earliest possible moment. Without this, a receiver-less `.method()`
    /// member would silently emit a method that ignores `this`, and a
    /// wrong-return `.constructor()` a factory of the wrong type.
    fn validate(&self, binding: &Building<'_, Self::Metadata>) -> Result<(), String> {
        // Report what this binding left unclaimed. Here because it is the
        // earliest generator-owned hook that sees the model, and it runs
        // exactly where the binding used to print these itself. Moves into
        // `JniGenBuilder::generate` once that exists (prebindgen#251 phase E).
        crate::core::warn_unclaimed(binding.flat(), &self.claimed());

        for (key, members) in &self.class_members {
            for m in members {
                // A binding-absent fn already hard-errored in the scan.
                let Some(item_fn) = binding
                    .flat()
                    .function(&m.rust_ident)
                    .map(|func| func.origin.as_syn())
                else {
                    continue;
                };
                match m.kind {
                    MemberKind::Method => {
                        let has_receiver = item_fn.sig.inputs.iter().any(|input| {
                            matches!(input, syn::FnArg::Typed(pt)
                                if &peel_receiver_key(&pt.ty) == key)
                        });
                        if !has_receiver {
                            let took: Vec<String> = item_fn
                                .sig
                                .inputs
                                .iter()
                                .filter_map(|i| match i {
                                    syn::FnArg::Typed(pt) => {
                                        Some(pt.ty.to_token_stream().to_string())
                                    }
                                    _ => None,
                                })
                                .collect();
                            return Err(format!(
                                "class `{}` method `{}`: no parameter of type `{}` — an \
                                 instance method's receiver must appear in the signature \
                                 (took: {})",
                                key.as_str(),
                                m.rust_ident,
                                key.as_str(),
                                if took.is_empty() {
                                    "no parameters".to_string()
                                } else {
                                    took.join(", ")
                                }
                            ));
                        }
                    }
                    MemberKind::Constructor => {
                        let ret = match &item_fn.sig.output {
                            syn::ReturnType::Type(_, ty) => (**ty).clone(),
                            syn::ReturnType::Default => syn::parse_quote!(()),
                        };
                        // Allowed factory shapes: `Self` and `Result<Self, E>`.
                        let core = crate::api::core::types_util::result_ok_type(&ret)
                            .unwrap_or_else(|| ret.clone());
                        if &peel_receiver_key(&core) != key {
                            return Err(format!(
                                "class `{}` constructor `{}`: must return `{}` or \
                                 `Result<{}, E>` — it returns `{}`",
                                key.as_str(),
                                m.rust_ident,
                                key.as_str(),
                                key.as_str(),
                                ret.to_token_stream()
                            ));
                        }
                    }
                }
            }
        }
        // Three sum positions in a declared signature are wrong. Two have no
        // lowering at all and would otherwise fail as
        // "`E` has no output converter", which names the sum rather than the
        // position — actively misleading, because a sum has no whole-value
        // converter BY DESIGN, so that message sends the reader looking for
        // something that must not exist. Reject them here, where the message
        // can say what is actually unsupported and what to write instead.
        for ident in self.declared_functions() {
            // The ELEMENT, not just its syntax: check (3) below asks its params
            // which are callbacks, which is the model's answer, not the tokens'.
            let Some(func) = binding.flat().function(&ident) else {
                continue;
            };
            let item_fn = func.origin.as_syn();
            // (1) A sum in the `Ok` position of a fallible return. A sum is
            // delivered DECOMPOSED through a builder callback, and the
            // `Result` lane has no builder: a `Result` return deliberately
            // keeps its whole-value converter so a fallible factory still
            // yields a handle (see `unfold::returns_type`).
            if let syn::ReturnType::Type(_, ret) = &item_fn.sig.output {
                if let Some(ok) = crate::api::core::types_util::result_ok_type(ret) {
                    let core = crate::api::core::types_util::peel_ref_option_vec(&ok);
                    if matches!(self.type_kind(binding, &core), TypeKind::Sum) {
                        return Err(format!(
                            "fn `{ident}`: `Result<{}, _>` — a sealed_class value is not \
                             supported in the success position of a fallible return. A sum \
                             crosses decomposed (a tag plus one leaf group per variant) \
                             through a builder callback, which the `Result` lane does not \
                             have — a fallible return keeps its whole-value converter so a \
                             factory can still hand back a handle. Return `{}` directly and \
                             report failure through the error channel, or model the failure \
                             as one of the sum's own variants",
                            ok.to_token_stream(),
                            ok.to_token_stream(),
                        ));
                    }
                }
            }
            // (2) A sum in the **error** position of a `Result` with no
            // deconstructor declared for it. Unlike the other two this one
            // RESOLVES — it takes the generic undecomposed-`E` path, where the
            // `Err` is routed to the plain binding-error channel as
            // `e.to_string()`. So the author declares a sealed hierarchy and
            // Kotlin silently receives a `String`, and the generated crate
            // quietly acquires an `E: Display` bound that fails downstream in
            // generated code rather than at the declaration. Emitting
            // something misleading is worse than not emitting: say so here.
            //
            // An error plan can only come from a TYPE-level `expand_return!`
            // (auto-applied to every fn with that `E`); a per-fn
            // `.expand_return(...)` always targets the Output position. So the
            // declaration set is the whole story, and this stays a pre-resolve
            // check.
            if let syn::ReturnType::Type(_, ret) = &item_fn.sig.output {
                if let Some(err_ty) = crate::api::core::types_util::result_err_type(ret) {
                    let core = crate::api::core::types_util::peel_ref_option_vec(&err_ty);
                    let declared = self
                        .return_expand_decls
                        .iter()
                        .any(|d| d.key == TypeKey::from_type(&err_ty));
                    if !declared && matches!(self.type_kind(binding, &core), TypeKind::Sum) {
                        return Err(format!(
                            "fn `{ident}`: `Result<_, {}>` — `{}` is declared `sealed_class!`, \
                             but nothing decomposes it in the error position, so it would be \
                             delivered as `e.to_string()` on the plain binding-error channel \
                             rather than as the sealed hierarchy (and would silently require \
                             `{}: Display`). Declare `expand_return!({})` with the fields to \
                             deliver, so the error crosses through the typed domain-handler \
                             channel; or, if a text message really is what you want, drop the \
                             `sealed_class!` declaration for this type",
                            // `Result<_, E>` and the `expand_return!` key are
                            // the WHOLE error type: the `Display` bound falls
                            // on it (the generated code calls `__e.to_string()`
                            // on the `Err` value), and the auto-apply matches a
                            // deconstructor by that same whole type. Only the
                            // "is declared `sealed_class!`" clause names the
                            // peeled sum, since that is what carries the
                            // declaration. Identical for a bare `E`; they
                            // diverge once it is wrapped.
                            err_ty.to_token_stream(),
                            core.to_token_stream(),
                            err_ty.to_token_stream(),
                            err_ty.to_token_stream(),
                        ));
                    }
                }
            }
            // (3) A **slice of sums** delivered to a callback
            // (`impl Fn(&[E])`): the element fold would need the sum's
            // folder-appender singleton, which is emitted per `Vec<E>` RETURN
            // position, so the shape resolves to nothing.
            for p in &func.params {
                let Some(args) = p.ty.callback_args() else {
                    continue;
                };
                for arg in args {
                    let after_ref = match arg.as_syn() {
                        syn::Type::Reference(r) => (*r.elem).clone(),
                        other => other.clone(),
                    };
                    let syn::Type::Slice(s) = &after_ref else {
                        continue;
                    };
                    let elem = match &*s.elem {
                        syn::Type::Reference(r) => (*r.elem).clone(),
                        other => other.clone(),
                    };
                    if matches!(self.type_kind(binding, &elem), TypeKind::Sum) {
                        return Err(format!(
                            "fn `{ident}`: `impl Fn(&[{}])` — a slice of a sealed_class value \
                             is not supported as a callback argument. A sum crosses as a tag \
                             plus one leaf group per variant, and folding a *sequence* of \
                             those into the foreign list needs the element folder a `Vec<{}>` \
                             RETURN provides; declare the callback over one value \
                             (`impl Fn({})`) or return `Vec<{}>` instead",
                            elem.to_token_stream(),
                            elem.to_token_stream(),
                            elem.to_token_stream(),
                            elem.to_token_stream(),
                        ));
                    }
                }
            }
        }
        Ok(())
    }

    /// The post-resolve validation boundary (issue #90): every bound
    /// function's lowered plan must build, and the split declarations must
    /// be unambiguous, before ANY artifact writer touches disk — see
    /// [`validate_bindings`].
    fn validate_resolved(&self, registry: &Registry<KotlinMeta>) -> Result<(), String> {
        validate_bindings(self, registry)
    }

    /// The other-side type of every `convert!` conversion, in the
    /// conversion's direction: an input fn's parameter type (peeled of `&`)
    /// must have its own **input** converter for the composed rank-0 body to
    /// chain through; an output fn's return type needs the **output** twin.
    /// Signatures are read from the registry (missing fns are reported by
    /// the scan's helper-function warning; the body derivation later
    /// hard-errors with the precise decl).
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
        // The two error-channel fns the extern bodies call: `signal_binding_error`
        // (binding/system failure → `JniErrorHandler`) and `signal_domain_error`
        // (a fallible fn's `Err(E)` → the typed `<Src>Handler`). Emitted above the
        // converters so wrapper code references them by bare name; the binding
        // crate reaches them as `<include_module>::signal_*` from outside the file.
        items.push(build_signal_binding_error_item());
        items.push(build_signal_domain_error_item());
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
        // Expression constants — one nullary JNI getter extern per
        // `PackageDecl::constant_expr`, its value the binding-defined
        // expression evaluated with a glob import of every source module (so
        // it composes the source crate's items without qualification). The
        // getter reuses the whole function-wrapper pipeline via the
        // synthetic signature, exactly like a const-backed getter.
        let mut glob_modules = registry.all_source_modules();
        if glob_modules.is_empty() {
            glob_modules.push(self.default_module(registry));
        }
        for decl in self.packages.values().flat_map(|p| &p.constant_exprs) {
            validate_constant_expr(self, &decl.kotlin_name, &decl.ty);
            let getter = const_expr_getter_fn(&decl.kotlin_name, &decl.ty, registry);
            let expr = &decl.expr;
            let callee: syn::Expr = syn::parse_quote!({
                #(
                    #[allow(unused_imports)]
                    use #glob_modules::*;
                )*
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

    fn post_process_item(&self, item: &mut syn::Item, registry: &Registry<KotlinMeta>) {
        self.qualify_item(item, registry);
    }

    // ── Item methods ─────────────────────────────────────────────────

    fn on_function(
        &self,
        f: &crate::api::core::flat::Function,
        registry: &Registry<KotlinMeta>,
    ) -> TokenStream {
        emit_jni_function_wrapper(self, f, registry)
    }

    fn on_struct(
        &self,
        _s: &crate::api::core::flat::Struct,
        _registry: &Registry<KotlinMeta>,
    ) -> TokenStream {
        // Struct converter bodies are emitted by the resolver via
        // input_terminal / output_terminal below; no separate
        // per-struct item is needed.
        TokenStream::new()
    }

    fn on_variant(
        &self,
        _v: &crate::api::core::flat::Variant,
        _registry: &Registry<KotlinMeta>,
    ) -> TokenStream {
        TokenStream::new()
    }

    fn on_enum(
        &self,
        _e: &crate::api::core::flat::Enum,
        _registry: &Registry<KotlinMeta>,
    ) -> TokenStream {
        TokenStream::new()
    }

    /// Declared consts only reach here (write gating via
    /// [`Prebindgen::declared_consts`]): re-emit the const as a path-alias
    /// to its source-of-truth (initializer tokens are never copied — they
    /// may reference source-crate internals) AND emit its nullary JNI getter
    /// extern. The getter reuses the whole function-wrapper pipeline (so the
    /// const's type flows through the ordinary output-converter machinery);
    /// only the callee expression differs — a path to the const, not a call.
    fn on_const(
        &self,
        c: &crate::api::core::flat::Constant,
        registry: &Registry<KotlinMeta>,
    ) -> TokenStream {
        reject_handle_const(self, c.origin.as_syn());
        let getter = const_getter_fn(c);
        let const_ident = &c.name;
        let source_module = self.fn_module(registry, const_ident);
        let callee: syn::Expr = syn::parse_quote!(#source_module::#const_ident);
        let wrapper = emit_jni_function_wrapper_with_callee(self, &getter, registry, Some(callee));
        let alias = crate::api::core::const_path_alias(c.origin.as_syn(), &source_module);
        quote! {
            #alias
            #wrapper
        }
    }
}

/// Structural converter builders — the rank-0 terminal chains and the rank-1
/// wrapper-shape handlers, now inherent helpers called by the structural
/// [`Prebindgen::on_input_type`] / [`Prebindgen::on_output_type`].
impl Declarations {
    // ── Input converters ─────────────────────────────────────────────

    /// Whole-type **input** terminal categories (opaque handle, enum,
    /// `convert!`, `str`, primitive, struct) — depends on nothing, `subs`
    /// empty.
    pub(crate) fn input_terminal(
        &self,
        reading: &crate::api::core::flat::TypeRef,
        registry: &impl Conversions<KotlinMeta>,
    ) -> Option<ConverterImpl<KotlinMeta>> {
        // Classify off `kind`, spell with `spell()`: the arms below that ask what
        // a type IS use `reading`, and everything that has to name it in
        // generated Rust uses this.
        let ty = reading.as_syn();
        // Structured-config overrides first (opaque handles, then user-
        // registered rank-0 wrappers, then built-ins).
        let key = TypeKey::from_type(ty);
        if let Some(cfg) = self.types.get(&key) {
            if cfg.is_opaque() {
                return Some(self.opaque_handle_input(ty));
            }
        }
        // Fixed-size array of JNI primitives — dual of the output branch.
        // The `try_into` IS the length check: a JVM array of the wrong size
        // becomes a binding error naming the type, never a panic.
        if let Some(spec) = crate::api::lang::jnigen::jni::prim_array::prim_array_of(ty) {
            let body = crate::api::lang::jnigen::jni::prim_array::input_body(ty, &spec);
            let wire = spec.wire.clone();
            let kotlin_name = self.override_kotlin_name(ty, Some(spec.kotlin.clone()));
            let niches = default_niches_for_wire(&wire);
            return Some(ConverterImpl {
                subs: vec![],
                pre_stages: vec![],
                function: self.build_input_fn(ty, &wire, &body, None),
                destination: wire,
                niches,
                metadata: self.framework_meta(kotlin_name),
            });
        }
        // `enum_class`-declared enums: jint wire, `TryFrom<i32>` decode.
        // Registered before the user-wrapper lookup so a stray
        // `input_wrapper` registration on the same key would have to be
        // intentional. The rank-0 enum arm produces a terminal converter
        // (jint → Rust enum) with the configured Kotlin FQN in metadata.
        if let Some(cfg) = self.types.get(&key) {
            if cfg.is_enum_class() {
                if let Some(name) = bare_path_ident(ty) {
                    if let Some(e) = registry.flat().enum_item(&name) {
                        let (wire, body) = enum_input_body(self, registry, e);
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
        if let Some(conv) = self.lookup_input(ty, registry) {
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
        // Any OWNED string, however Rust spells it — `String`, `Box<String>`,
        // `Cow<'_, str>`. The model classifies each of them `Str`; the spelling is
        // the source's business, and `.into()` constructs it from the decoded
        // `String`. This used to be one hardcoded `TypeKey == "Box < String >"`
        // arm, which is what a spelling-keyed converter table costs: one
        // hand-written case per representation anyone happened to write (#270).
        //
        // `str` is handled above, separately and deliberately: it is unsized,
        // so its converter yields an owned `String` the call site borrows —
        // a different contract, not a different spelling.
        if matches!(
            reading.unwrapped().kind(),
            crate::api::core::flat::TypeKind::Str | crate::api::core::flat::TypeKind::String
        ) {
            let wire: syn::Type = syn::parse_quote!(jni::objects::JString);
            let body: syn::Expr = syn::parse_quote!({
                let s = env.get_string(v).map_err(|e| {
                    <__JniErr as ::core::convert::From<String>>::from(format!(
                        "decode_string: {}",
                        e
                    ))
                })?;
                // The canonical value, then the spelling.
                ::std::string::String::from(s).into()
            });
            let rust_ty = ty.clone();
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
            let metadata = if TypeKey::from_type(ty).as_str() == "u64" {
                self.unsigned64_leaf_meta()
            } else {
                self.framework_meta(kotlin_name)
            };
            return Some(ConverterImpl {
                subs: vec![],
                pre_stages: vec![],
                function: self.build_input_fn(ty, &wire, &body, None),
                destination: wire,
                niches,
                metadata,
            });
        }
        if let Some(name) = bare_path_ident(ty) {
            // A `sealed_class` sum reached as a whole `JObject` — a field of a
            // data class, or any position where the parent is already an
            // object. Its own converter, so the parent's generic field branch
            // delegates exactly as it does for a nested data class. (The
            // OUTPUT direction has no counterpart: a sum crosses Rust →
            // Kotlin flattened, always.)
            if self.types.get(&key).is_some_and(|c| c.sum().is_some()) {
                if let Some(crate::api::core::flat::Type::Variant(v)) =
                    registry.flat().declared_type(&name)
                {
                    let (wire, body) = sum_input_body(self, v, registry)?;
                    // The wire's own null niche, exactly as a data class gets
                    // — that is what lets `Option<sum>` fold with JVM null as
                    // `None` instead of needing a boxed wrapper.
                    let niches = default_niches_for_wire(&wire);
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
            }
            if let Some(s) = registry.flat().struct_type(&name) {
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

    /// The **outbound** half of [`Self::input_transparent_bridge`], and the same
    /// last resort: a spelling whose only difference from something this adapter
    /// can already convert is the transparent wrappers over it.
    ///
    /// It had no twin, so an erased wrapper resolved inbound and not outbound —
    /// `Box<Priority>` was a parameter this binding could take and a return it
    /// could not give, for a wrapper the model exists to make invisible (#309).
    /// The one arm covers `Box<Handle>`, `Box<enum>` and `Box<DataClass>` alike,
    /// because [`Self::output_terminal`] misses all three the same way: it keys
    /// on the SPELLING, and no config sits under `Box < Priority >`.
    ///
    /// The wrappers come **off** here rather than going on, which is the whole
    /// difference:
    ///
    /// ```text
    /// input : let __inner = <inner>(env, v)?;      build_through_erased_wrappers(__inner)
    /// output: let __inner = read_through(v);       <inner>(env, __inner)
    /// ```
    ///
    /// Everything else is direction-independent — `subs`, `destination`,
    /// `niches`, `metadata` all mean the same thing either way, and inheriting
    /// the inner's metadata is what keeps `Box<Priority>` presenting as the
    /// Kotlin enum class instead of losing it behind the wrapper.
    pub(crate) fn output_transparent_bridge(
        &self,
        reading: &crate::api::core::flat::TypeRef,
        registry: &impl Conversions<KotlinMeta>,
    ) -> Option<ConverterImpl<KotlinMeta>> {
        if reading.erased_wrappers().is_empty() {
            return None;
        }
        let produced = reading.as_syn();
        let stripped = reading.stripped_syntax();
        // A wrapper over a **borrow** is refused here too, and outbound the
        // reason is its own: a borrow's output route is the clone-into-a-fresh-
        // handle arm, which hands back a wire built from a reference — there is
        // no owned value to read the wrapper off. Inbound the same guard is
        // about `E0106`; the shapes coincide, the reasons do not.
        //
        // Asked of the MODEL: an erasure is transparent, so `Box<&T>` already
        // classifies as `Ref` and nothing here matches a `syn` variant.
        if reading.borrow_target().is_some() {
            return None;
        }
        // It has to be a type this binding already crosses; if it is not, the
        // ordinary "unresolved" diagnostic names it, which is the better error.
        let inner = registry.reading_of(&stripped)?;
        let entry = registry.output_entry(&inner)?;
        let wire = entry.destination.clone();
        // Take the wrappers off what the caller handed us. `None` is `Cow`'s
        // policy refusal — the crossing then stays unresolved and names the
        // type, rather than resolving and emitting Rust the consumer cannot
        // build.
        let read = read_through_erased_wrappers(reading, quote!(v))?;
        // The inner's COMPLETE chain, stages included: a `convert!` type reaches
        // its wire through them.
        let inner_call =
            crate::api::lang::jnigen::jni::emit::composed_inner_output(entry, quote!(__inner));
        let body: syn::Expr = syn::parse_quote!({
            let __inner = #read;
            #inner_call
        });
        Some(ConverterImpl {
            subs: vec![stripped],
            pre_stages: vec![],
            function: self.build_output_fn(produced, &wire, &body, None),
            destination: wire,
            niches: entry.niches.clone(),
            // The surface is the inner type's — a wrapper is invisible to the
            // destination language, which is why the model erases it.
            metadata: entry.metadata.clone(),
        })
    }

    /// **Last resort**: a spelling whose only difference from something this
    /// adapter can already convert is the transparent wrappers over it.
    ///
    /// The layer arms each handle one *classification* layer — `Optional`,
    /// `Sequence`, `Ref` — and bridge a wrapper as part of doing so. What none of
    /// them covers is a wrapper over a **terminal**: `Box<Payload>` classifies as
    /// `Named`, so no layer arm claims it, and `input_terminal` keys on the whole
    /// spelling and finds no `Payload` config under `Box < Payload >`. Before
    /// this it resolved to nothing at all — the crossing was refused for a
    /// wrapper the model exists to make invisible.
    ///
    /// So this delegates to the **stripped** spelling's own converter and puts
    /// the wrappers back on what it produced. The inner type is declared as a
    /// `sub`, exactly as a layer arm declares its inner, so it is required and
    /// resolved through the ordinary machinery rather than being resolved here.
    ///
    /// Deliberately tried **after** every layer arm, so nothing that resolves
    /// today changes route: `Box<Option<T>>` keeps the `Optional` arm (which
    /// bridges via `build_from_canonical`), and only the shapes that previously
    /// reached `None` arrive here.
    pub(crate) fn input_transparent_bridge(
        &self,
        reading: &crate::api::core::flat::TypeRef,
        registry: &impl Conversions<KotlinMeta>,
    ) -> Option<ConverterImpl<KotlinMeta>> {
        if reading.erased_wrappers().is_empty() {
            return None;
        }
        let produced = reading.as_syn();
        // The spelling under every wrapper — by the model's own definition, the
        // one whose lowering yields this `kind`.
        let stripped = reading.stripped_syntax();
        // A wrapper over a **borrow** is not bridgeable here, and the reason is
        // the converter's own shape rather than the wrapper's: this produces an
        // owned value, and there is nothing for a `Box<&T>` to borrow *from* —
        // the returned reference would have to outlive the call that made it
        // (`E0106` on the generated signature). The borrow arms own that case,
        // and they serve the canonical spelling only.
        //
        // Asked of the MODEL, not of `stripped`: an erasure is transparent, so
        // `Box<&T>` already classifies as `Ref` and `kind` answers this without
        // anything here matching a `syn` variant.
        if reading.borrow_target().is_some() {
            return None;
        }
        // It has to be a type this binding already crosses; if it is not, the
        // ordinary "unresolved" diagnostic names it, which is the better error.
        let inner = registry.reading_of(&stripped)?;
        let entry = registry.input_entry(&inner)?;
        let wire = entry.destination.clone();
        // Wrap what the inner converter produced. `None` here is `Cow`'s policy
        // refusal — the crossing then stays unresolved and names the type,
        // rather than resolving and emitting Rust the consumer cannot build.
        let built = build_through_erased_wrappers(reading, quote!(__inner))?;
        // The inner's COMPLETE chain, stages included. This called
        // `entry.function` directly and left `pre_stages` empty, which SKIPPED
        // them: a `convert!`-declared type reaches its Rust value through those
        // stages (`jlong -> u64 -> Duration`), so a `Box` over one arrived
        // un-staged. Every other composing arm goes through this helper for
        // exactly that reason (#309).
        let inner_call =
            crate::api::lang::jnigen::jni::emit::composed_inner_input(entry, quote!(v));
        let body: syn::Expr = syn::parse_quote!({
            let __inner = #inner_call;
            #built
        });
        Some(ConverterImpl {
            subs: vec![stripped],
            pre_stages: vec![],
            function: self.build_input_fn(produced, &wire, &body, None),
            destination: wire,
            niches: entry.niches.clone(),
            // The surface is the inner type's: a wrapper is invisible to the
            // destination language, which is the whole reason the model erases
            // it. Inheriting rather than recomputing also keeps a projection's
            // Kotlin class from being lost behind the wrapper.
            metadata: entry.metadata.clone(),
        })
    }

    /// **Input** wrapper shape (`pat` = the reconstructed canonical pattern,
    /// `t1` = its captured inner): the built-in `&`/`Option<&>`/`Vec`/`Option`
    /// handlers. The dual of [`Self::output_wrapper_shape`], whose own doc has
    /// said so all along — this had been stranded above a different function
    /// since the transparent bridge was inserted between them (#294), and
    /// adding the outbound bridge moved it onto an OUTPUT converter, where it
    /// read as an outright contradiction.
    pub(crate) fn input_wrapper_shape(
        &self,
        shape: WrapperShape,
        produced: &syn::Type,
        t1: &crate::api::core::flat::TypeRef,
        registry: &impl Conversions<KotlinMeta>,
    ) -> Option<ConverterImpl<KotlinMeta>> {
        // Disjoint shapes (see [`WrapperShape`]), tried in priority order. The
        // borrow/option-ref/vec shapes are mutually exclusive; the two
        // `Optional` sub-cases share a method.
        self.input_borrow(shape, produced, t1, registry)
            .or_else(|| self.input_option_ref(shape, produced, t1, registry))
            .or_else(|| self.input_vec(shape, produced, t1, registry))
            .or_else(|| self.input_option(shape, produced, t1, registry))
    }

    // ── Output converters ────────────────────────────────────────────

    /// Whole-type **output** terminal categories (the dual of
    /// [`Self::input_terminal`]: opaque handle, enum, user table,
    /// `str`, `Cow<[u8]>`, unit, primitive, struct) — `subs` empty.
    pub(crate) fn output_terminal(
        &self,
        reading: &crate::api::core::flat::TypeRef,
        registry: &impl Conversions<KotlinMeta>,
    ) -> Option<ConverterImpl<KotlinMeta>> {
        // Classify off `kind`, spell with `spell()` — see `input_terminal`.
        let ty = reading.as_syn();
        // Structured-config overrides first (opaque handles, then built-ins).
        let key = TypeKey::from_type(ty);
        if let Some(cfg) = self.types.get(&key) {
            if cfg.is_opaque() {
                return Some(self.opaque_handle_output(ty));
            }
        }
        // Fixed-size array of JNI primitives: `[u8; N]` -> `ByteArray`,
        // `[i64; N]` -> `LongArray`, ... Bulk-copied, nothing boxed. See
        // [`prim_array`]; this replaced the raw-memory value blob.
        if let Some(spec) = crate::api::lang::jnigen::jni::prim_array::prim_array_of(ty) {
            let body = crate::api::lang::jnigen::jni::prim_array::output_body(&spec);
            let wire = spec.wire.clone();
            let kotlin_name = self.override_kotlin_name(ty, Some(spec.kotlin.clone()));
            let niches = default_niches_for_wire(&wire);
            return Some(ConverterImpl {
                subs: vec![],
                pre_stages: vec![],
                function: self.build_output_fn(ty, &wire, &body, None),
                destination: wire,
                niches,
                metadata: self.framework_meta(kotlin_name),
            });
        }
        // `enum_class`-declared enums: jint wire, `as jni::sys::jint`
        // encode. Symmetric to the input arm above; relies on
        // `#[repr(i32)]` (or any repr that supports the cast) on the
        // declared enum so the discriminant value round-trips identically.
        if let Some(cfg) = self.types.get(&key) {
            if cfg.is_enum_class() {
                if let Some(name) = bare_path_ident(ty) {
                    if let Some(e) = registry.flat().enum_item(&name) {
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
        if let Some(conv) = self.lookup_output(ty, registry) {
            return Some(conv);
        }
        // `str` is unsized, so it has no by-value output converter — but it is
        // reached as the sub of a `&str` reference accessor leaf. Resolve it to
        // the same `&str → jstring` fn the rank-1 `&str` arm uses (deduped by
        // name) so required-propagation doesn't flag it unresolved.
        if TypeKey::from_type(ty).as_str() == "str" {
            return Some(self.str_ref_output());
        }
        // An owned string in any representation the model erases — `Box<String>`,
        // `Cow<'_, str>`. It classifies each of them `Str`, and the body was
        // already representation-agnostic: `v.as_str()` reaches through any of
        // them by `Deref`. Only the *dispatch* was spelling-keyed, as one
        // hardcoded `TypeKey == "Box < String >"` arm (#270).
        //
        // Plain `String` keeps its own earlier arm in `primitive_output`, whose
        // body this matches exactly; this one is reached for the wrapped
        // spellings that arm's key cannot name.
        if matches!(
            reading.unwrapped().kind(),
            crate::api::core::flat::TypeKind::Str | crate::api::core::flat::TypeKind::String
        ) {
            let wire: syn::Type = syn::parse_quote!(jni::objects::JString);
            let body: syn::Expr = syn::parse_quote!({
                env.new_string(v.as_str()).map_err(|e| {
                    <__JniErr as ::core::convert::From<String>>::from(format!("encode_str: {}", e))
                })?
            });
            let rust_ty = ty.clone();
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
        if matches!(
            reading.unwrapped().kind(),
            crate::api::core::flat::TypeKind::Unit
        ) {
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
            let metadata = if TypeKey::from_type(ty).as_str() == "u64" {
                self.unsigned64_leaf_meta()
            } else {
                self.framework_meta(kotlin_name)
            };
            return Some(ConverterImpl {
                subs: vec![],
                pre_stages: vec![],
                function: self.build_output_fn(ty, &wire, &body, None),
                destination: wire,
                niches,
                metadata,
            });
        }
        if let Some(name) = bare_path_ident(ty) {
            if let Some(s) = registry.flat().struct_type(&name) {
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
    /// the built-in `&Handle`/`&str`/`Option`/`Vec` handlers. An
    /// `Option<&Handle>` resolves via the shallow `Option<_>`.
    pub(crate) fn output_wrapper_shape(
        &self,
        shape: WrapperShape,
        produced: &syn::Type,
        t1: &crate::api::core::flat::TypeRef,
        registry: &impl Conversions<KotlinMeta>,
    ) -> Option<ConverterImpl<KotlinMeta>> {
        // `t1`'s spelling, for the parts that ask spelling questions; the
        // READING stays in `t1` for the lookups (#284).
        let t1_ty = t1.as_syn();
        // Borrowed opaque-handle output (`&T` / `&'static T` where `T` is a
        // declared opaque handle). Canonical zenoh-flat's `z_*` accessors
        // return *borrowed* handles for the C tier's zero-copy borrows, but
        // the JVM keeps its handle past the call — so the only sound lowering
        // is to clone the referent into a fresh owned `Box`-handle (every such
        // zenoh handle type is `Clone`). This mirrors `opaque_handle_output`
        // with a `.clone()`; `Option<&T>` then composes through the `Option`
        // arm below (it looks up this `&T` entry as its inner). Matched
        // structurally so the lifetime variant `&'static _` is covered too.
        if let syn::Type::Reference(r) = produced {
            if r.mutability.is_none()
                && self
                    .types
                    .get(&TypeKey::from_type(t1_ty))
                    .is_some_and(|c| c.is_opaque())
            {
                let mut ref_ty = r.clone();
                *ref_ty.elem = t1_ty.clone();
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
                    metadata: self.opaque_leaf_meta(t1_ty),
                });
            }
        }
        // Borrowed string slice output (`&str` / `&'a str`): the converter used
        // for a zero-copy reference accessor return (`f(&T) -> &str`, output
        // expansion). The single copy into the JVM is `&str → jstring` (no
        // intermediate owned `String`). The unsized `str` sub resolves via the
        // rank-0 arm to the same fn (see [`Self::str_ref_output`]).
        if let syn::Type::Reference(r) = produced {
            if r.mutability.is_none() && TypeKey::from_type(t1_ty).as_str() == "str" {
                return Some(self.str_ref_output());
            }
        }
        // `Result<T, E>` is peeled by the selector, off the model's
        // `TypeKind::Fallible`. Bindings declare the `Err` type via
        // `.throwable()`.
        if shape == WrapperShape::Optional {
            let outer_ty = produced.clone();
            let canonical: syn::Type = syn::parse_quote!(Option<#t1_ty>);
            // Bridgeable first: an unsupported representation must not resolve
            // and then emit code the consumer cannot compile.
            let read = read_as_canonical(produced, &canonical)?;
            let (wire, inner_body, niches) = option_output(t1_ty, registry)?;
            let body: syn::Expr = syn::parse_quote!({
                let v: #canonical = #read;
                #inner_body
            });
            let inherited = registry
                .output_entry(t1)
                .and_then(|e| e.metadata.kotlin_name.clone());
            let kotlin_name = self.override_kotlin_name(&outer_ty, inherited);
            // Fold a Nullable layer over the inner projection (if any). The
            // kind reflects which path `option_output` took (see
            // [`nullable_kind_for`]): niche-fulfilled keeps the inner wire
            // and treats the slot value as `None`; boxed widens to `JObject`
            // and uses JVM null.
            let nullable_kind = nullable_kind_for_output(&wire, t1_ty, registry);
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
        if shape == WrapperShape::Sequence {
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
            // The element's COMPLETE Rust -> wire chain (see the input peer).
            let inner_conv = crate::api::lang::jnigen::jni::emit::composed_inner_output(
                inner,
                quote::quote!(__elem),
            );
            let outer_ty = produced.clone();
            let canonical: syn::Type = syn::parse_quote!(Vec<#t1_ty>);
            let read = read_as_canonical(produced, &canonical)?;
            let wire: syn::Type = syn::parse_quote!(jni::objects::JObject);
            let body: syn::Expr = syn::parse_quote!({
                let v: #canonical = #read;
                let __list_obj = env
                    .new_object("java/util/ArrayList", "()V", &[])
                    .map_err(|e| <__JniErr as ::core::convert::From<String>>::from(format!("Vec<_>: new ArrayList: {}", e)))?;
                let __list = jni::objects::JList::from_env(env, &__list_obj)
                    .map_err(|e| <__JniErr as ::core::convert::From<String>>::from(format!("Vec<_>: list-from-env: {}", e)))?;
                for __elem in v.into_iter() {
                    let __elem_wire = #inner_conv;
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
            // `Vec<Handle>` carries the full strategy.
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
                    value_rust_type: None,
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
        registry: &impl Conversions<KotlinMeta>,
    ) -> Option<ConverterImpl<KotlinMeta>> {
        let inner = registry
            .reading_of(elem)
            .and_then(|tr| registry.output_entry(&tr))?;
        // A `&[opaque-handle]` callback arg is delivered by the Kotlin-side leaf
        // fold (typed-handle wrap), bypassing this whole-`ArrayList` converter; a
        // handle's `jlong` wire isn't JObject-shaped, so it returns `None` here.
        let inner_wire = inner.destination.clone();
        if !is_jobject_shaped_wire(&inner_wire) {
            return None;
        }
        // The element's COMPLETE Rust -> wire chain (see the `Vec<_>` peer).
        let inner_conv = crate::api::lang::jnigen::jni::emit::composed_inner_output(
            inner,
            quote::quote!(::core::clone::Clone::clone(__elem)),
        );
        let outer_ty: syn::Type = syn::parse_quote!(&[#elem]);
        let wire: syn::Type = syn::parse_quote!(jni::objects::JObject);
        let body: syn::Expr = syn::parse_quote!({
            let __list_obj = env
                .new_object("java/util/ArrayList", "()V", &[])
                .map_err(|e| <__JniErr as ::core::convert::From<String>>::from(format!("&[_]: new ArrayList: {}", e)))?;
            let __list = jni::objects::JList::from_env(env, &__list_obj)
                .map_err(|e| <__JniErr as ::core::convert::From<String>>::from(format!("&[_]: list-from-env: {}", e)))?;
            for __elem in v.iter() {
                let __elem_wire = #inner_conv;
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
                value_rust_type: None,
                projection,
            },
        })
    }
}

/// The declaration surface, stated once.
///
/// These were trait methods the registry called back into the adapter from
/// inside `resolve`. They are the adapter's own business now, gathered into the
/// one value the registry is constructed from.
impl Declarations {
    /// Union of every `.fun(...)` list across all
    /// [`Self::package`] subpackage contexts. Each entry is a
    /// `#[prebindgen]` fn ident the user explicitly hooked into the
    /// binding; functions not in this set are skipped by the registry's
    /// signature scan and by the per-item emitter.
    pub(crate) fn declared_functions(&self) -> std::collections::HashSet<syn::Ident> {
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
    /// Functions ever referenced as a named `.field(fun!(...))` in any
    /// `expand_return!` decl, type-level or per-fn — see
    /// [`JniGenBuilder::field_accessor_fns`]. Usage-derived, not tied to `.method()`
    /// class-member declarations: a function need not also be exposed as an
    /// instance method to be referenced this way.
    pub(crate) fn accessor_functions(&self) -> std::collections::HashSet<syn::Ident> {
        self.field_accessor_fns()
    }
    /// Methods (`.method`) — their fn ident mapped to the owning class's
    /// `TypeKey`, so input-flattening can skip the receiver parameter.
    pub(crate) fn method_receivers(&self) -> std::collections::HashMap<syn::Ident, TypeKey> {
        self.class_members
            .iter()
            .flat_map(|(key, ms)| {
                ms.iter()
                    .filter(|m| m.kind == MemberKind::Method)
                    .map(move |m| (m.rust_ident.clone(), key.clone()))
            })
            .collect()
    }
    /// Fns acknowledged-but-unbound via [`JniGenBuilder::ignore`] — suppresses
    /// the registry's "skipping undeclared" warning, emits nothing.
    pub(crate) fn ignored_functions(&self) -> std::collections::HashSet<syn::Ident> {
        self.ignored_fns.clone()
    }
    /// Bulk name-family ignores from [`JniGenBuilder::ignore`] +
    /// [`matching`](crate::lang::matching).
    pub(crate) fn ignored_name_predicates(
        &self,
    ) -> Vec<crate::api::core::prebindgen::NamePredicate> {
        self.ignored_name_predicates.clone()
    }
    /// Framework-called fns that get no extern of their own: `convert!`
    /// conversion fns (called by generated converter bodies) and fns
    /// referenced only inside boundary decls (`expand_return!` accessors /
    /// `expand_param!` ctors, called by the generated fold/unfold code).
    /// Routing both through the *helper* channel — not the ignore channel —
    /// makes a typo'd `fun!(…)` inside a decl a hard scan error
    /// (`ScanError::DeclaredNotFound`) instead of a stale-ignore
    /// warning.
    /// Declared functions are subtracted: a fn that is also a real
    /// member/package fn keeps its extern. Type requirements come through
    /// [`Self::extra_required_types`], not a signature scan.
    pub(crate) fn helper_functions(&self) -> std::collections::HashSet<syn::Ident> {
        let declared = self.declared_functions();
        self.convert_fns()
            .chain(self.boundary_referenced_fns())
            .filter(|f| !declared.contains(f))
            .collect()
    }
    pub(crate) fn declared_consts(&self) -> Option<std::collections::HashSet<syn::Ident>> {
        let mut out = std::collections::HashSet::new();
        for pkg in self.packages.values() {
            for c in &pkg.constants {
                out.insert(c.rust_ident.clone());
            }
        }
        Some(out)
    }
    /// Consts acknowledged-but-unexposed via [`JniGenBuilder::ignore`].
    pub(crate) fn ignored_consts(&self) -> std::collections::HashSet<syn::Ident> {
        self.ignored_const_idents.clone()
    }
    /// Union of every `.constant(...)` list across all
    /// [`Self::package`] subpackage contexts. `Some` even when empty — JniGenBuilder
    /// HAS a const declaration mechanism, so const emission is declared-only
    /// and undeclared consts get the skip warning (see
    /// [`Prebindgen::declared_consts`]).
    /// The declared value types of every expression constant
    /// (`ConstDecl::expr`) — they have no `#[prebindgen]` item to
    /// scan, so the resolver is told directly to produce their output
    /// converters.
    pub(crate) fn required_output_types(&self) -> Vec<syn::Type> {
        self.packages
            .values()
            .flat_map(|p| p.constant_exprs.iter().map(|e| e.ty.clone()))
            .collect()
    }
    /// Every type registered via one of the **class declarators**
    /// (`ptr_class!` / `enum_class!` / `sealed_class!` / `data_class!`)
    /// — i.e. every entry in the type table, whose only
    /// writer is `JniGenBuilder::register_class`. These are the only structs/enums
    /// the per-item emitter walks, and the scan requires them in BOTH
    /// directions (their converters always resolve both ways). Wrapper
    /// registrations live in their own tables and are deliberately excluded: a
    /// wrapper type is required per **usage** direction, so an output-only
    /// wrapper needs no input twin.
    ///
    /// Each with the spelling its declarator was written with — the scan needs
    /// real tokens to intern a type that is in no table yet (#291).
    pub(crate) fn declared_types(&self) -> std::collections::HashMap<TypeKey, Origin<syn::Type>> {
        self.types
            .iter()
            .map(|(k, c)| (k.clone(), c.rust_type.clone()))
            .collect()
    }
    /// Types acknowledged-but-undeclared via [`JniGenBuilder::ignore`].
    pub(crate) fn ignored_types(&self) -> std::collections::HashSet<TypeKey> {
        self.ignored_class_types.clone()
    }
    /// What this binding claimed, for the unclaimed-item report. A helper is
    /// claimed even though it is never emitted, and a boundary-only type even
    /// though it never crosses whole: both are deliberate, so neither is a
    /// skip worth reporting.
    pub(crate) fn claimed(&self) -> crate::core::Claimed {
        let mut functions = self.declared_functions();
        functions.extend(self.helper_functions());
        // The report asks what was *claimed*, which is a set of identities —
        // the declarations' spellings are the scan's business, not this one's.
        let mut types: std::collections::HashSet<TypeKey> =
            self.declared_types().into_keys().collect();
        types.extend(self.boundary_only_types().into_keys());
        crate::core::Claimed {
            functions,
            types,
            consts: self.declared_consts(),
            ignored_functions: self.ignored_functions(),
            ignored_types: self.ignored_types(),
            ignored_consts: self.ignored_consts(),
            ignored_name_predicates: self.ignored_name_predicates(),
        }
    }
    /// **Rust-side-only** types: boundary decls (`expand_param!` /
    /// `expand_return!`) whose type has no class declaration. They never
    /// materialize in Kotlin — only their ingredients (fold) and fields
    /// (unfold / error channel) cross the boundary — so the registry
    /// acknowledges them and drops their direct converter requirements once
    /// the plans are in place.
    pub(crate) fn boundary_only_types(
        &self,
    ) -> std::collections::HashMap<TypeKey, Origin<syn::Type>> {
        // A `sealed_class!`-declared sum has no single wire: it crosses as a
        // tag plus one leaf group per variant, so a direct converter for the
        // value itself is genuinely not needed. Declaring it boundary-only
        // drops that requirement while keeping the type scanned (its payload
        // types register and resolve, which is what the Kotlin surface reads
        // its field types from).
        self.rust_side_only_types()
            .chain(
                self.types
                    .iter()
                    .filter(|(_, c)| c.sum().is_some())
                    .map(|(k, c)| (k.clone(), c.rust_type.clone())),
            )
            .collect()
    }
}

#[cfg(test)]
mod wrapper_ops_tests {
    use super::*;

    /// Every wrapper the model erases has a row here.
    ///
    /// The two lists answer different questions — the model's is "what do I
    /// erase", this file's is "what can I rebuild" — and they are allowed to
    /// disagree about *capability* (`Cow` is erased and cannot be read through).
    /// They are not allowed to disagree about *membership*: a wrapper that
    /// becomes transparent without a row here would be silently unbridgeable
    /// everywhere, which looks exactly like a type the binding got wrong.
    ///
    /// So adding `Rc` is: one entry in `TRANSPARENT_WRAPPERS`, one row in
    /// `WRAPPER_OPS` (`read: None` — an `Rc`'s payload cannot be moved out —
    /// and `build: Some(Rc::new)`). This test is what says so out loud instead
    /// of leaving the second step to be discovered.
    #[test]
    fn every_erased_wrapper_has_ops() {
        let missing: Vec<&str> = crate::api::core::flat::TRANSPARENT_WRAPPERS
            .iter()
            .copied()
            .filter(|w| wrapper_ops(w).is_none())
            .collect();
        assert!(
            missing.is_empty(),
            "the model erases {missing:?}, and this adapter has no `WrapperOps` row for them — \
             add one (`read`/`build` may be `None` when the representation does not allow it, \
             which refuses the shape instead of mis-generating it)"
        );
    }

    /// …and nothing here claims a wrapper the model does not erase, which would
    /// be an operation that can never run.
    #[test]
    fn no_ops_for_a_wrapper_the_model_keeps() {
        let stray: Vec<&str> = WRAPPER_OPS
            .iter()
            .map(|w| w.name)
            .filter(|n| !crate::api::core::flat::TRANSPARENT_WRAPPERS.contains(n))
            .collect();
        assert!(
            stray.is_empty(),
            "`WRAPPER_OPS` rows for non-erased {stray:?}"
        );
    }

    /// A rebuild puts the wrappers back **innermost-out**, the reverse of the
    /// order a read takes them off.
    ///
    /// Asserted on a `Box<Box<_>>` rather than a single layer, because a single
    /// layer cannot tell the two orders apart — which is exactly how a
    /// composition bug survives. And asserted against `read` on the same type,
    /// so the two are pinned as duals rather than as two independent claims.
    #[test]
    fn a_rebuild_puts_the_wrappers_back_inside_out() {
        let ty = crate::api::test_util::reading(syn::parse_quote!(Box<Box<Option<String>>>));
        assert_eq!(ty.erased_wrappers(), ["Box", "Box"]);

        let built = build_through_erased_wrappers(&ty, quote!(v)).expect("Box builds");
        assert_eq!(
            built.to_string().replace(' ', ""),
            ":: std :: boxed :: Box :: new (:: std :: boxed :: Box :: new (v))".replace(' ', ""),
        );
        // The dual, on the same type: reading takes them off outermost-first.
        let read = read_through_erased_wrappers(&ty, quote!(v)).expect("Box reads");
        assert_eq!(read.to_string().replace(' ', ""), "**v");

        // The control: nothing erased, so both are the identity and neither
        // test above can be passing on an unconditional wrap.
        let plain = crate::api::test_util::reading(syn::parse_quote!(Option<String>));
        assert!(plain.erased_wrappers().is_empty());
        for e in [
            build_through_erased_wrappers(&plain, quote!(v)),
            read_through_erased_wrappers(&plain, quote!(v)),
        ] {
            assert_eq!(e.expect("identity").to_string(), "v");
        }
    }

    /// `Cow` declines a rebuild, and the two directions decline for **different
    /// reasons** — which is why the row carries two `None`s rather than one
    /// capability flag.
    ///
    /// Reading is impossible (`E0507`: a `Cow` payload cannot be moved through
    /// `Deref`). Building is *possible* — `Cow::Owned(v)` is well-typed for an
    /// owned payload — and refused on purpose, because a binding that can only
    /// ever hand a `Cow` parameter `Owned` pays a copy per call and removes the
    /// borrow path the source asked for. If that policy is ever revisited, this
    /// test is the thing that has to change with it.
    #[test]
    fn a_cow_declines_a_rebuild_by_policy() {
        let ty = crate::api::test_util::reading(syn::parse_quote!(Cow<'_, str>));
        assert_eq!(ty.erased_wrappers(), ["Cow"]);
        assert!(build_through_erased_wrappers(&ty, quote!(v)).is_none());
        assert!(read_through_erased_wrappers(&ty, quote!(v)).is_none());

        // A `Cow` under a `Box` declines too: one unbuildable layer refuses the
        // whole chain, rather than the `Box` half quietly succeeding.
        let nested = crate::api::test_util::reading(syn::parse_quote!(Box<Cow<'_, str>>));
        assert_eq!(nested.erased_wrappers(), ["Box", "Cow"]);
        assert!(build_through_erased_wrappers(&nested, quote!(v)).is_none());
    }
}
