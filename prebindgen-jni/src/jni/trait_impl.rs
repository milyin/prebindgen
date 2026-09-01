//! [`Prebindgen`] implementation for [`JniGenBuilder`] plus its converter-
//! selector / exception-routing helpers.
//!
//! Carved from the former monolithic JNI module; shares the `jni`
//! namespace via `use super::*`.

use kotlin_codegen::KtType;
use prebindgen_registry::{Building, Conversions, Crossing, RegistryBuilder};

use super::*;

/// Whether Flat classifies this exact crossing as the unsigned 64-bit scalar.
/// Destination policy asks the model once through this helper; no caller reads
/// or compares Rust spelling.
pub(crate) fn is_unsigned64(ty: &prebindgen_registry::flat::TypeRef) -> bool {
    matches!(
        ty.kind(),
        prebindgen_registry::flat::TypeKind::Scalar(prebindgen_registry::flat::ScalarKind::U64)
    )
}

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
pub(crate) fn generated_converter_attr() -> syn::Attribute {
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
    /// Leaf metadata for an opaque handle: value-context name `Long` plus the
    /// projection that folds outward through wrappers. The corresponding
    /// ownership, null-niche, and odd-pointer-tag operations live in the late
    /// `JHandleCodecPlan`; this method describes the Kotlin-facing leaf only.
    pub(crate) fn opaque_leaf_meta(&self, key: TypeKey) -> KotlinMeta {
        KotlinMeta {
            projection: Some(Projection {
                leaf_key: key,
                owned: true,
                strategy: FoldStrategy::Base,
                kind: ProjectionKind::Handle,
                niche_sentinels: Vec::new(),
            }),
            ..self.framework_meta(Some(KtType::cls("Long")))
        }
    }

    /// Leaf metadata for Rust `u64`: the JNI value-context stays `Long`, while
    /// projection-aware Kotlin emitters surface `ULong` and insert the
    /// bit-preserving `toLong()` / `toULong()` bridge.
    pub(crate) fn unsigned64_leaf_meta(&self) -> KotlinMeta {
        KotlinMeta {
            projection: Some(Projection {
                leaf_key: TypeKey::parse("u64").expect("builtin type key"),
                owned: false,
                strategy: FoldStrategy::Base,
                kind: ProjectionKind::Unsigned64,
                niche_sentinels: Vec::new(),
            }),
            ..self.framework_meta(Some(KtType::long()))
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
        key: &TypeKey,
        inherited: Option<KtType>,
    ) -> Option<KtType> {
        if let Some(cfg) = self.types.get(key) {
            // Opaque-handle entries keep their typed FQN in
            // `name_spec` for FQN-consumers, but the value-context
            // name is `"Long"` (set on the rank-0 handler's metadata).
            // Don't let that FQN leak into a wrapper's metadata.
            if !cfg.is_opaque() {
                if let Some(spec) = &cfg.name_spec {
                    return Some(KtType::cls(self.fqn_of(spec)));
                }
            }
        }
        inherited
    }

    /// Canonical input-converter name for `(rust, wire)` — exposed
    /// for plugin wrapper exts that name a converter artifact with a
    /// non-standard return type (e.g.
    /// `impl Into<…>` parameters that can't be expressed via
    /// `input_wrapper_shape`'s fixed signature shape).
    pub fn input_converter_name(&self, rust: &syn::Type, wire: &syn::Type) -> syn::Ident {
        input_name(&rust.to_token_stream(), wire)
    }

    /// Symmetric to [`Self::input_converter_name`].
    pub fn output_converter_name(&self, rust: &syn::Type, wire: &syn::Type) -> syn::Ident {
        output_name(&rust.to_token_stream(), wire)
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
            mid: &::prebindgen_jni_runtime::CachedIfaceMethod,
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
            mid: &::prebindgen_jni_runtime::CachedIfaceMethod,
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

/// The items every extern body reaches by bare name: the framework error
/// alias, the `OwnedObject` carrier borrowed-handle plans return, and the two
/// error-channel functions.
///
/// `__JniErr` is the **framework** error type alias — always the
/// `JniBindingError` String-wrapper. Built-in converter bodies compose their
/// `?` failures into this type via its `From<String>` impl. A `Result<T, E>`
/// return instead binds its own raw `E`; both funnel to the per-call
/// `signal_error` sink (generic over `Display`).
///
/// The two error-channel fns are `signal_binding_error` (binding/system
/// failure → `JniErrorHandler`) and `signal_domain_error` (a fallible fn's
/// `Err(E)` → the typed `<Src>Handler`). They are written above the converters
/// so wrapper code references them by bare name; the binding crate reaches
/// them as `<include_module>::signal_*` from outside the file.
pub(crate) fn render_prelude() -> Vec<syn::Item> {
    let error_type = framework_error_type();
    let mut items: Vec<syn::Item> = vec![syn::parse_quote!(
        #[allow(dead_code)]
        pub(crate) type __JniErr = #error_type;
    )];
    items.extend(owned_object_prerequisite_items());
    items.push(build_signal_binding_error_item());
    items.push(build_signal_domain_error_item());
    items
}

/// One nullary getter extern per binding-defined constant expression.
///
/// The expression is evaluated with a glob import of every source module, so
/// it composes the source crate's items without qualification. The getter
/// reuses the whole extern pipeline through a synthetic nullary signature,
/// exactly like a const-backed getter.
pub(crate) fn plan_constant_expressions(
    ext: &Declarations,
    registry: &Registry,
) -> Vec<crate::jni::emit::JWrapper> {
    let mut glob_modules = registry.all_source_modules();
    if glob_modules.is_empty() {
        glob_modules.push(ext.default_module(registry));
    }
    ext.packages
        .values()
        .flat_map(|package| &package.constant_exprs)
        .map(|decl| {
            validate_constant_expr(ext, &decl.kotlin_name, &decl.ty);
            let getter = {
                ext.freeze_reading_of(registry, &decl.ty);
                const_expr_getter_fn(&decl.kotlin_name, &decl.ty, ext)
            };
            let expr = &decl.expr;
            let callee: syn::Expr = syn::parse_quote!({
                #(
                    #[allow(unused_imports)]
                    use #glob_modules::*;
                )*
                #expr
            });
            crate::jni::emit::JWrapper::new(ext, registry, &getter, Some(callee))
        })
        .collect()
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
/// artifacts are sorted by symbol to keep generated output deterministic.
///
/// Planning is gated on the resolved `registry`: a destructor is only planned
/// for an opaque handle whose type a scanned `#[prebindgen]` fn actually
/// references (as input or output). This mirrors converter emission and keeps
/// feature-gated handles (e.g. `zenoh-ext`-only types whose declare/undeclare
/// fns are `#[cfg]`'d out of the scan) from producing destructors that
/// reference types not in scope.
pub(crate) fn plan_handle_destructors(
    ext: &Declarations,
    registry: &Registry,
) -> Vec<crate::jni::generation::JHandleDestructor> {
    let mut planned: Vec<crate::jni::generation::JHandleDestructor> = Vec::new();
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
        if ext.in_frag(&reading).is_none() && ext.out_frag(&reading).is_none() {
            continue;
        }
        let class_fqn = cfg
            .name_spec
            .as_ref()
            .map(|s| ext.fqn_of(s))
            .unwrap_or_else(|| {
                panic!(
                    "plan_handle_destructors: opaque handle `{}` has no \
                     name spec to derive a destructor symbol from",
                    key.as_str()
                )
            });
        let class_short = class_fqn.rsplit('.').next().unwrap_or(&class_fqn);
        let class_package = class_fqn.rsplit_once('.').map(|(pkg, _)| pkg).unwrap_or("");
        let free_ptr = ext.mangle_method(class_package, class_short, "freePtr");
        planned.push(crate::jni::generation::JHandleDestructor::new(
            reading,
            super::symbol::native_symbol(class_package, class_short, &free_ptr),
        ));
    }
    planned.sort_by(|a, b| a.symbol().cmp(b.symbol()));
    planned
}

/// What generated Rust can do with one wrapper the model
/// [erases](prebindgen_registry::flat::TRANSPARENT_WRAPPERS).
///
/// Erasure and reconstruction are different questions, and only the first is the
/// model's. `Box<T>` *is* `T` to every destination language — but undoing it in
/// Rust is `*b`, undoing a `Cow` is `into_owned()`, and undoing an `Rc` is not
/// possible at all. There is no trait spanning those, so the operations live
/// here, one recipe per wrapper, instead of as a special case per converter.
///
/// **Adding a wrapper is adding a recipe.** Put its name in
/// `TRANSPARENT_WRAPPERS` (the model decides what it erases) and a recipe here
/// (the adapter decides what it can rebuild); `every_erased_wrapper_has_ops`
/// fails if the two disagree, so a wrapper cannot become transparent without
/// this file having an answer for it.
pub(crate) struct WrapperOps {
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
    pub(crate) read: Option<fn(TokenStream) -> TokenStream>,
    /// Build it **from** the inner value. `None` when not supported.
    pub(crate) build: Option<fn(TokenStream) -> TokenStream>,
}

/// The operations table. One recipe per wrapper the model erases.
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
        // something needs it; that is one recipe, not a redesign.
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

pub(crate) fn wrapper_ops(name: &str) -> Option<&'static WrapperOps> {
    WRAPPER_OPS.iter().find(|w| w.name == name)
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
/// question, per [`TypeRef::erased_wrappers`](prebindgen_registry::flat::TypeRef::erased_wrappers).
pub(crate) fn read_through_erased_wrappers(
    ty: &prebindgen_registry::flat::TypeRef,
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
/// than by impossibility; see its recipe. A caller that gets `None` has a crossing
/// it cannot serve and must decline or report it, never emit the bare value.
///
/// **This answers for one layer's spelling.** It restores the wrappers standing
/// over `ty`'s own classification; a wrapper *inside* — the `Box` of
/// `Option<Box<S>>` — belongs to the inner reading, is applied when that layer
/// is built, and is invisible here. An erasure sits **outside** the layer it
/// wraps, so a rebuild collects wrappers as it descends and applies them as it
/// comes back out.
pub(crate) fn build_through_erased_wrappers(
    ty: &prebindgen_registry::flat::TypeRef,
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
/// plans live in `KotlinParamOp`, whose size every variant pays.
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

impl Declarations {
    /// `& _` / `& mut _` borrow: share T's resolved converter — `&T`'s entry
    /// points at the same `ItemFn` (the fn returns owned `T`; the call site in
    /// `emit_jni_function_wrapper` adds `&decoded`). Exists so the
    /// wildcard-substitution machinery marks T required transitively from `&T`.
    pub(crate) fn input_borrow(
        &self,
        produced: &prebindgen_registry::flat::TypeRef,
        t1: &prebindgen_registry::flat::TypeRef,
    ) -> Option<ConverterImpl<KotlinMeta>> {
        // This converter does NOT produce the spelled type: it hands back the
        // inner type's own entry, and the call site adds the `&`. So there is no
        // value in hand to unwrap a representation from, and a wrapped spelling
        // — `Box<&T>` — must not resolve here (it would pass an owned `T` where
        // `Box<&T>` is expected).
        if !produced.erased_wrappers().is_empty() {
            return None;
        }
        let inner = self.in_frag(t1)?;
        let outer_ty = produced.key();
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
            destination: inner.wire.clone(),
            converter: inner.converter.clone(),
            niches: inner.niches.clone(),
            metadata: KotlinMeta {
                kotlin_name,
                value_reading: None,
                projection,
                niche_sentinels: inner.metadata.niche_sentinels.clone(),
            },
        })
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
    fn is_leaf_vec_element(&self, elem: &prebindgen_registry::flat::TypeRef) -> bool {
        match self.types.get(&elem.key()) {
            // A declared opaque handle crosses as a single `jlong` (pointer)
            // leaf that the Kotlin folder wraps into its typed handle class.
            // Enums and multi-field data classes are not leaf-folded — data
            // classes go through `value_struct_decons`.
            Some(cfg) => cfg.is_opaque(),
            // Undeclared: `String` is JObject-shaped; `u64` is the built-in
            // scalar projection whose raw jlong leaf the Kotlin folder wraps
            // into `ULong`. Other primitive collections retain their existing
            // unsupported status (`Vec<u8>` is the rank-0 ByteArray special).
            None => {
                matches!(elem.kind(), prebindgen_registry::flat::TypeKind::String)
                    || is_unsigned64(elem)
            }
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
    pub fn build(self) -> Result<JniGen, prebindgen_registry::WriteRustError> {
        let flat = self
            .sources
            .clone()
            .build()
            .map_err(prebindgen_registry::ScanError::from)?;
        let registry = prebindgen_registry::Registry::builder(flat)?;
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
        registry: prebindgen_registry::RegistryBuilder,
    ) -> Result<JniGen, prebindgen_registry::WriteRustError> {
        let mut decls = self.decls;
        let mut declared = decls.declare_into(registry)?.validate_with(&decls)?;
        // A second holding of the model: `convert_with` consumes the builder,
        // and the table outlives that call.
        let model = declared.flat().clone();
        let expansion_leaves: Vec<_> = declared.expansion_leaf_readings()?.cloned().collect();
        let recipes = decls
            .recipes(&model, &expansion_leaves, &declared)
            .map_err(|errors| prebindgen_registry::ScanError::AdapterInvariant {
                message: errors
                    .iter()
                    .map(|e| e.to_string())
                    .collect::<Vec<_>>()
                    .join("; "),
            })?;
        // A `data_class` field that is itself one takes the `parts` recipe rather
        // than its own default — see `Declarations::bindings`.
        let bindings = decls
            .bindings(&model, &declared, &recipes)
            .map_err(|errors| prebindgen_registry::ScanError::AdapterInvariant {
                message: errors
                    .iter()
                    .map(|e| e.to_string())
                    .collect::<Vec<_>>()
                    .join("; "),
            })?;
        // Both tables go on `decls`, because compiling a **site** happens after
        // this function has returned: the sites are `fn_plan`'s to enumerate,
        // and `Compiler::resume` needs these two beside the model.
        decls.tables = Some(std::rc::Rc::new(crate::jni::Tables { recipes, bindings }));
        // The driver's state lives on `decls` rather than here, because the
        // adapter reads it **while** it compiles: a conversion for one type is
        // built out of the conversions for its inners, which are compiled
        // first. That is the same order the converter table was filled in, so
        // a fragment is there exactly when a table entry would have been.
        // Compositions that refused. See `compile_crossing`: these are adapter
        // invariants, reported together once the walk is done.
        let mut refusals: Vec<String> = Vec::new();
        let registry = declared
            .convert_with(|crossing, built| {
                let mut compiler = prebindgen_registry::recipe::Compiler::resume(
                    &model,
                    decls.recipe_table(),
                    decls.site_bindings(),
                    decls.compiled.borrow().clone(),
                );
                let compiled =
                    decls.compile_crossing(&mut compiler, crossing, built, &mut refusals);
                *decls.compiled.borrow_mut() = compiler.finish();
                let conv = compiled?;
                // The conversion stays here; what the registry gets back is
                // which other crossings this one delegates to, which is what
                // its reachability walk needs.
                Some(prebindgen_registry::Answer::over(conv.subs))
            })?
            .build()?;
        if !refusals.is_empty() {
            return Err(prebindgen_registry::ScanError::AdapterInvariant {
                message: refusals.join("; "),
            }
            .into());
        }
        // Post-resolve invariants, run once here so the writers are pure reads
        // and a `JniGen` is valid by construction.
        decls
            .validate_resolved(&registry)
            .map_err(|message| prebindgen_registry::ScanError::AdapterInvariant { message })?;
        // Validate every declared decomposition. Product plans remain queryable
        // descriptors rather than emission roots: final per-item planning marks only
        // chains used by a concrete input, output or callback and activates nested
        // dependencies transitively. Sealed classes need this pass because they have
        // no deconstructing whole-value crossing.
        for key in decls.declared_decompositions() {
            let Some(ident) = key.ident() else { continue };
            let Ok(ty) = model.classify(&syn::parse_quote!(#ident)) else {
                continue;
            };
            // Only a type every part of which already crosses. One that does
            // not is a gap in the binding, and the writers report it against
            // the part — `Reading.Exact.v0 has no OUTPUT converter` — where
            // this could only say that composing failed.
            let part_types: Vec<&prebindgen_registry::flat::TypeRef> =
                match model.declared_type(&ident) {
                    Some(prebindgen_registry::flat::Type::Variant(sum)) => sum
                        .alternatives
                        .iter()
                        .flat_map(|alt| &alt.fields)
                        .map(|f| &f.ty)
                        .collect(),
                    Some(prebindgen_registry::flat::Type::Struct(s)) => {
                        s.fields.iter().map(|f| &f.ty).collect()
                    }
                    _ if decls.value_form_names(&registry, &ty).is_some() => Vec::new(),
                    _ => continue,
                };
            if part_types.iter().any(|ty| decls.out_frag(ty).is_none()) {
                continue;
            }
            let crossing = prebindgen_registry::recipe::Crossing::new(
                ty,
                prebindgen_registry::recipe::Direction::Deconstruct,
            );
            // A type with nothing to hand out states no `parts` recipe — an empty
            // struct, or an enum with no alternatives. Asking for one by name
            // is refused now rather than answered with the default, so the
            // condition that declared it is the condition asked here.
            if decls
                .recipe_table()
                .key_of(&crossing.key(), &crate::jni::recipes::parts())
                .is_none()
            {
                continue;
            }
            let mut compiler = prebindgen_registry::recipe::Compiler::resume(
                &model,
                decls.recipe_table(),
                decls.site_bindings(),
                decls.compiled.borrow().clone(),
            );
            let mut adapter = crate::jni::compile::JCompile {
                decls: &decls,
                registry: &registry,
                declared_return: None,
                site: None,
            };
            if let Err(e) =
                compiler.recipe_of(&mut adapter, &crossing, &crate::jni::recipes::parts())
            {
                refusals.push(format!(
                    "`{key}` hands out its parts, but composing them failed: {e:?}"
                ));
            }
            *decls.compiled.borrow_mut() = compiler.finish();
        }
        if !refusals.is_empty() {
            return Err(prebindgen_registry::ScanError::AdapterInvariant {
                message: refusals.join("; "),
            }
            .into());
        }
        let generation = crate::jni::generation::JniGenerationPlan::freeze(&mut decls, &registry)?;
        decls.generation = Some(std::rc::Rc::new(generation));
        Ok(JniGen { decls, registry })
    }
}

impl Declarations {
    /// Build the conversion for one crossing by asking the table which recipe it
    /// takes and the driver to compile that recipe.
    ///
    /// `None` is *cannot*, never *not yet*: the crossings arrive inner-first,
    /// so everything this could compose from is already in `built`.
    fn compile_crossing<'v, R: Conversions>(
        &'v self,
        compiler: &mut prebindgen_registry::recipe::Compiler<
            '_,
            crate::jni::compile::JCompile<'v, R>,
        >,
        crossing: &Crossing,
        built: &'v R,
        refusals: &mut Vec<String>,
    ) -> Option<ConverterImpl<KotlinMeta>> {
        let (dir, key) = crossing;
        // The reading the scan already took for this crossing, fetched by the
        // key the crossing IS.
        let reading = built.reading(key)?;
        let direction = *dir;
        let mut adapter = crate::jni::compile::JCompile {
            decls: self,
            registry: built,
            declared_return: None,
            site: None,
        };
        let crossing = prebindgen_registry::recipe::Crossing::new(reading, direction);
        let fragment = compiler.crossing(&mut adapter, &crossing).ok()?;
        // A `data_class` also states a recipe that says what it is made of.
        // Compiling that named recipe equips whole-value input, output and
        // callback paths with the same registry-owned Product descriptor.
        // The **stripped** key, so `Box<Payload>` and `&Payload` compile the
        // recipe too: all three spellings find one recipe and each gets its own
        // fragment, which is what a site taking a wrapped spelling reads.
        // A `data_class` with no fields states no `parts` recipe — there is
        // nothing for it to be made of — so the recipe is asked for by name only
        // where it was declared. `recipe_of` refuses an absent name rather than
        // answering with the default, which is what makes that condition the
        // one that has to match.
        if (direction == prebindgen_registry::recipe::Direction::Construct
            && matches!(
                self.types
                    .get(&crossing.value().stripped_key())
                    .map(|c| &c.kind),
                Some(DeclaredKind::Data)
            )
            || crossing.value().optional_inner().is_some())
            && compiler
                .recipes()
                .key_of(&crossing.key(), &crate::jni::recipes::parts())
                .is_some()
        {
            // A refusal is a bug in the composition, not a gap in the binding:
            // every part of a `data_class` is a crossing that already resolved
            // on its own, so nothing here can legitimately be missing.
            // Returning `None` would report an unresolved crossing and blame
            // the declaration, so the reason is collected and surfaced as an
            // adapter invariant — beside whatever else the walk found, and
            // through the same `Result` every other refusal takes.
            if let Err(e) =
                compiler.recipe_of(&mut adapter, &crossing, &crate::jni::recipes::parts())
            {
                refusals.push(format!(
                    "`{}` crosses as its fields, but composing them failed: {e:?}",
                    crossing.spelled().key()
                ));
            }
        }
        Some((*fragment).clone().conv)
    }

    pub fn declare_into(
        &self,
        mut registry: RegistryBuilder,
    ) -> Result<RegistryBuilder, prebindgen_registry::WriteRustError> {
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
            registry = registry.cross(Direction::Deconstruct, &ty);
        }
        // The other-side type of every `convert!` conversion, in the
        // conversion's direction: an input fn's parameter type needs its own
        // input converter for the composed body to chain through; an output
        // fn's return type needs the output twin.
        let mut convert_edges: Vec<(Crossing, Crossing)> = Vec::new();
        for decl in &self.convert_decls {
            if let Some(ty) = self.convert_target(decl.key(), &registry, Direction::Construct) {
                registry = registry.cross(Direction::Construct, &ty);
                // The target's conversion chains through this one, and nothing
                // about the target type says so.
                convert_edges.push((
                    (Direction::Construct, decl.key().clone()),
                    (Direction::Construct, TypeKey::from_type(&ty)),
                ));
            }
            if let Some(ty) = self.convert_target(decl.key(), &registry, Direction::Deconstruct) {
                registry = registry.cross(Direction::Deconstruct, &ty);
                convert_edges.push((
                    (Direction::Deconstruct, decl.key().clone()),
                    (Direction::Deconstruct, TypeKey::from_type(&ty)),
                ));
            }
        }
        for (from, on) in convert_edges {
            registry = registry.depends(from, on);
        }
        // How composites cross in pieces. Every one of these reads only the
        // model, which is what lets them be stated here rather than asked for
        // mid-resolve.
        //
        // The output side is applied here rather than declared: the plans are
        // this adapter's to keep, and the registry is told only the two things
        // it needs from them — which readings must cross on the output side,
        // and which leaves a callback argument's delivery depends on.
        let mut unfolding = prebindgen_registry::unfold::Unfolding::new(registry.flat());
        let exports = registry.exports().clone();
        let accessors = registry.accessors().clone();
        prebindgen_registry::unfold::apply(
            &mut unfolding,
            &self.build_deconstructors(&registry),
            &exports,
            &accessors,
        )?;
        // Synthesized by-value `data_class` decompositions: the leaves are
        // already built above; this wires them into fixed-builder plans.
        prebindgen_registry::unfold::apply_value_structs(
            &mut unfolding,
            self.build_value_struct_decons(&registry),
            &exports,
        )?;
        // The same wiring for a value whose alternatives are chosen at runtime
        // (a tag plus one leaf group per variant) rather than being a fixed
        // product.
        prebindgen_registry::unfold::apply_sum_returns(
            &mut unfolding,
            self.build_sum_decons(&registry),
            &exports,
        )?;
        // Single-leaf `Vec<T>`/`&[T]` whole-element folds — the dual of the
        // `data_class` folds above, for String / scalar / handle elements, so
        // the list is built on the Kotlin side rather than through a Rust
        // `ArrayList`.
        prebindgen_registry::unfold::apply_leaf_vec_folds(
            &mut unfolding,
            self.build_leaf_vec_fold_elements(&registry),
            &exports,
        )?;
        let decompositions = prebindgen_registry::Decompositions {
            expansions: Some(self.build_expansions()),
            requirements: unfolding.requirements().to_vec(),
            callback_arg_leaves: unfolding.callback_arg_leaves(),
            replaces: self.boundary_only_types(),
        };
        self.unfolded
            .set(unfolding.into_plans())
            .unwrap_or_else(|_| panic!("a binding declares itself into a registry once"));
        registry = registry.decompose(decompositions);
        Ok(registry)
    }
}

impl Declarations {
    pub(crate) fn build_value_struct_decons(
        &self,
        registry: &impl Conversions,
    ) -> Vec<prebindgen_registry::unfold::ValueDecon> {
        let mut out = Vec::new();
        for item_struct in registry.flat().types().filter_map(|t| match t {
            prebindgen_registry::flat::Type::Struct(s) => Some(s),
            _ => None,
        }) {
            // The declaration's own reading, and its key off that — neither
            // composed from the ident, which an adapter cannot do anyway.
            let reading = item_struct.type_ref();
            let key = reading.key();
            // A `data_class` is a registered type that is neither an opaque
            // handle nor an enum.
            let is_data_class = matches!(
                self.type_kind(registry.flat(), &reading.key()),
                TypeKind::DataStruct { cfg: Some(c), .. } if c.name_spec.is_some()
            );
            if !is_data_class {
                continue;
            }
            if let Some(leaves) = crate::jni::synth_value_struct_leaves(self, registry, item_struct)
            {
                if !leaves.is_empty() {
                    out.push(prebindgen_registry::unfold::ValueDecon {
                        key,
                        source: reading.clone(),
                        leaves,
                    });
                }
            }
        }
        out
    }

    pub(crate) fn build_sum_decons(
        &self,
        registry: &impl Conversions,
    ) -> Vec<prebindgen_registry::unfold::SumDecon> {
        let mut keys: Vec<&TypeKey> = self.types.keys().collect();
        keys.sort_by(|a, b| a.as_str().cmp(b.as_str()));
        let mut out = Vec::new();
        for key in keys {
            if self.types[key].sum().is_none() {
                continue;
            }
            // The `sealed_class!` declaration's own IDENTITY. This runs during
            // the declare phase, where a `reading()` would legitimately answer
            // `None` for a type nothing has interned yet — the declaration is
            // the only thing that can say (#291), and `Origin::key` is what it
            // says it with. This took the declaration's node and ran
            // `bare_path_ident` over it to reach the same ident.
            let Some(ident) = self.types[key].rust_type.key().ident() else {
                continue;
            };
            let Some(prebindgen_registry::flat::Type::Variant(sum)) =
                registry.flat().declared_type(&ident)
            else {
                continue;
            };
            out.push(prebindgen_registry::unfold::SumDecon {
                key: key.clone(),
                // The sum's own reading, which the declaration answers with —
                // `Variant::type_ref` exists for exactly this, and it works in
                // the declare phase where a `reading()` lookup could not.
                source: sum.type_ref().clone(),
                leaves: crate::jni::synth_sum_leaves(self, registry, &ident, sum),
            });
        }
        out
    }

    pub(crate) fn build_leaf_vec_fold_elements(&self, registry: &impl Conversions) -> Vec<TypeKey> {
        let mut seen = std::collections::HashSet::new();
        let mut out = Vec::new();
        let mut consider = |bare: &prebindgen_registry::flat::TypeRef| {
            if seen.insert(bare.key()) && self.is_leaf_vec_element(bare) {
                out.push(bare.key());
            }
        };
        for f in registry.flat().functions() {
            // `Vec<T>` / `Option<Vec<T>>` return. The model's `ret` already
            // normalizes an elided return to `()`, so there is no arm for it.
            {
                let after_opt = match f.ret.kind() {
                    prebindgen_registry::flat::TypeKind::Optional(inner) => inner,
                    _ => &f.ret,
                };
                if let prebindgen_registry::flat::TypeKind::Vec(elem) = after_opt.kind() {
                    consider(peel_one_borrow(elem));
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
                    if let prebindgen_registry::flat::TypeKind::Slice(elem) =
                        peel_one_borrow(arg).kind()
                    {
                        consider(peel_one_borrow(elem));
                    }
                }
            }
        }
        out
    }
}

/// One `&` off, and nothing else — the model's own `borrow_target` would also
/// see through a `Box`/`Cow`, which `peel_leading_ref` did not.
fn peel_one_borrow(t: &prebindgen_registry::flat::TypeRef) -> &prebindgen_registry::flat::TypeRef {
    match t.kind() {
        prebindgen_registry::flat::TypeKind::Ref { inner, .. } => inner,
        _ => t,
    }
}

/// The declared **fieldless** enum under `name`, or a panic naming the right
/// declarator when it is a sum.
///
/// `enum_class` crosses the boundary as a bare discriminant and has no room for
/// a payload, which is why this is a hard error rather than a fallthrough. It
/// used to be `assert_only_unit_variants`, running `enum_shape` over a
/// `syn::ItemEnum` to work out which of the two shapes it had — the
/// classification the model makes once, at parse time, and expresses as two
/// different elements.
pub(crate) fn flat_unit_enum<'r>(
    registry: &'r impl Conversions,
    name: &syn::Ident,
    declarator: &str,
) -> Option<&'r prebindgen_registry::flat::Enum> {
    match registry.flat().declared_type(name)? {
        prebindgen_registry::flat::Type::Enum(e) => Some(e),
        prebindgen_registry::flat::Type::Variant(v) => {
            let offender = v
                .alternatives
                .iter()
                .find(|a| !a.is_empty())
                .map(|a| a.name.to_string())
                .unwrap_or_default();
            panic!(
                "`{name}` is a data-carrying enum (variant `{offender}` has fields): declare \
                 it with `sealed_class!({name})`, not `{declarator}!({name})` — \
                 `{declarator}` crosses the boundary as a bare discriminant and has no room \
                 for a payload"
            )
        }
        _ => None,
    }
}

impl Declarations {
    pub(crate) fn dispatch_fn_input(
        &self,
        operation: OperationId,
        source: &prebindgen_registry::flat::TypeRef,
        args: &[prebindgen_registry::flat::TypeRef],
        registry: &impl Conversions,
        arg_fragments: &[&crate::jni::compile::JFrag],
    ) -> Option<(ConverterImpl<KotlinMeta>, crate::jni::chain::JFunction)> {
        let (wire, plan) = callback_input(
            self,
            operation.clone(),
            source,
            args,
            registry,
            arg_fragments,
        )?;
        let niches = default_niches_for_wire(&wire);
        // `impl Fn(...)` crosses the extern tier as the erased lambda object
        // (`Any`) — same as the unfold builder / error-sink params. The typed
        // wrapper-level lambda signature is computed at render time from the
        // arg types' callback plans, not carried in metadata.
        let conv = ConverterImpl {
            subs: vec![],
            converter: operation,
            destination: wire,
            niches,
            metadata: self.framework_meta(Some(KtType::any())),
        };
        Some((conv, crate::jni::chain::JFunction::invoke(plan)))
    }
}

impl Prebindgen for Declarations {
    // ── Structural type resolution ──────────────────────────────────────
    // Try the terminal categories, then the `Result` peel, then the built-in
    // wrapper shapes — peel
    // `ty`'s outermost layer and dispatch to `{input,output}_wrapper_shape` with
    // the reconstructed canonical pattern. `subs` = the captured inner(s).

    /// Member-shape invariants (N5), checked against registry signatures —
    /// the earliest possible moment. Without this, a receiver-less `.method()`
    /// member would silently emit a method that ignores `this`, and a
    /// wrong-return `.constructor()` a factory of the wrong type.
    fn validate(&self, binding: &Building<'_>) -> Result<(), String> {
        // Report what this binding left unclaimed. Here because it is the
        // earliest generator-owned hook that sees the model, and it runs
        // exactly where the binding used to print these itself. Moves into
        // `JniGenBuilder::generate` once that exists (prebindgen#251 phase E).
        prebindgen_registry::warn_unclaimed(binding.flat(), &self.claimed());

        for (key, members) in &self.class_members {
            for m in members {
                // A binding-absent fn already hard-errored in the scan.
                //
                // The ELEMENT, not its item: a signature is a parameter list
                // and a return, both already classified, so neither check
                // below walks `syn::FnArg` / `syn::ReturnType` to re-derive
                // what the model states.
                let Some(func) = binding.flat().function(&m.rust_ident) else {
                    continue;
                };
                match m.kind {
                    MemberKind::Method => {
                        let has_receiver =
                            func.params.iter().any(|p| &peel_receiver_key(&p.ty) == key);
                        if !has_receiver {
                            let took: Vec<String> =
                                func.params.iter().map(|p| p.ty.to_string()).collect();
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
                        // Allowed factory shapes: `Self` and `Result<Self, E>`.
                        // The element normalizes an elided return to `Unit`, so
                        // there is no `ReturnType::Default` arm to write, and
                        // `fallible_parts` is `result_ok_type` asked of the
                        // classification instead of of a path.
                        let core = func.ret.fallible_parts().map_or(&func.ret, |(ok, _)| ok);
                        if &peel_receiver_key(core) != key {
                            return Err(format!(
                                "class `{}` constructor `{}`: must return `{}` or \
                                 `Result<{}, E>` — it returns `{}`",
                                key.as_str(),
                                m.rust_ident,
                                key.as_str(),
                                key.as_str(),
                                func.ret
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
            // (1) A sum in the `Ok` position of a fallible return. A sum is
            // delivered DECOMPOSED through a builder callback, and the
            // `Result` lane has no builder: a `Result` return deliberately
            // keeps its whole-value converter so a fallible factory still
            // yields a handle (see `unfold::returns_type`).
            // Off the model's return, not the item's `sig.output`: the reading
            // says it is fallible and hands over both sides, where re-reading
            // the signature had to find the `Result` in a path first.
            if let Some((ok, _)) = func.ret.fallible_parts() {
                {
                    let core = crate::util::head_type(ok);
                    if matches!(self.type_kind(binding.flat(), &core.key()), TypeKind::Sum) {
                        return Err(format!(
                            "fn `{ident}`: `Result<{}, _>` — a sealed_class value is not \
                             supported in the success position of a fallible return. A sum \
                             crosses decomposed (a tag plus one leaf group per variant) \
                             through a builder callback, which the `Result` lane does not \
                             have — a fallible return keeps its whole-value converter so a \
                             factory can still hand back a handle. Return `{}` directly and \
                             report failure through the error channel, or model the failure \
                             as one of the sum's own variants",
                            ok, ok,
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
            if let Some((_, err_ty)) = func.ret.fallible_parts() {
                {
                    let core = crate::util::head_type(err_ty);
                    let declared = self
                        .return_expand_decls
                        .iter()
                        .any(|d| *d.key() == err_ty.key());
                    if !declared
                        && matches!(self.type_kind(binding.flat(), &core.key()), TypeKind::Sum)
                    {
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
                            err_ty,
                            core,
                            err_ty,
                            err_ty,
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
                    // The same walk `build_leaf_vec_fold_elements` makes over a
                    // callback argument, off the same helper: one borrow, a
                    // slice, one borrow off its element.
                    let prebindgen_registry::flat::TypeKind::Slice(elem) =
                        peel_one_borrow(arg).kind()
                    else {
                        continue;
                    };
                    let elem = peel_one_borrow(elem);
                    if matches!(self.type_kind(binding.flat(), &elem.key()), TypeKind::Sum) {
                        return Err(format!(
                            "fn `{ident}`: `impl Fn(&[{}])` — a slice of a sealed_class value \
                             is not supported as a callback argument. A sum crosses as a tag \
                             plus one leaf group per variant, and folding a *sequence* of \
                             those into the foreign list needs the element folder a `Vec<{}>` \
                             RETURN provides; declare the callback over one value \
                             (`impl Fn({})`) or return `Vec<{}>` instead",
                            elem, elem, elem, elem,
                        ));
                    }
                }
            }
        }
        Ok(())
    }

    /// The post-resolve validation boundary (issue #90): every bound
    /// function's lowered plan must build, and the split declarations must
    /// be unambiguous, before ANY artifact writer touches disk.
    fn validate_resolved(&self, registry: &Registry) -> Result<(), String> {
        validate_bindings(self, registry)
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
    /// [`matching`](crate::matching).
    pub(crate) fn ignored_name_predicates(&self) -> Vec<prebindgen_registry::NamePredicate> {
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
    /// Union of every `.constant(...)` list across all [`Self::package`]
    /// subpackage contexts. `Some` even when empty — [`JniGenBuilder`] HAS a
    /// const declaration mechanism, so const emission is declared-only and
    /// undeclared consts get the skip warning.
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
    /// The recipe table this binding was built against.
    pub(crate) fn recipe_table(&self) -> &prebindgen_registry::recipe::Recipes {
        &self.tables.as_ref().expect("built").recipes
    }

    /// Which recipe each site takes.
    pub(crate) fn site_bindings(&self) -> &prebindgen_registry::recipe::Bindings {
        &self.tables.as_ref().expect("built").bindings
    }

    /// Every type that states what it hands out — `sealed_class!` and
    /// `data_class!` — in a stable order.
    ///
    /// Sorted, because what reads it drives compilation and a refusal has to
    /// name the same type run to run.
    pub(crate) fn declared_decompositions(&self) -> Vec<TypeKey> {
        let mut keys: Vec<TypeKey> = self
            .types
            .iter()
            .filter(|(_, c)| matches!(c.kind, DeclaredKind::Sealed(_) | DeclaredKind::Data))
            .map(|(k, _)| k.clone())
            // And every type whose value form says what it hands out, whatever
            // kind of class it is — `expand_return!` is declared over a
            // `ptr_class` as readily as over anything else.
            .chain(self.return_expand_decls.iter().map(|d| d.key().clone()))
            .collect();
        keys.sort_by(|a, b| a.as_str().cmp(b.as_str()));
        keys.dedup();
        keys
    }

    /// Types acknowledged-but-undeclared via [`JniGenBuilder::ignore`].
    pub(crate) fn ignored_types(&self) -> std::collections::HashSet<TypeKey> {
        self.ignored_class_types.clone()
    }
    /// What this binding claimed, for the unclaimed-item report. A helper is
    /// claimed even though it is never emitted, and a boundary-only type even
    /// though it never crosses whole: both are deliberate, so neither is a
    /// skip worth reporting.
    pub(crate) fn claimed(&self) -> prebindgen_registry::Claimed {
        let mut functions = self.declared_functions();
        functions.extend(self.helper_functions());
        // The report asks what was *claimed*, which is a set of identities —
        // the declarations' spellings are the scan's business, not this one's.
        let mut types: std::collections::HashSet<TypeKey> =
            self.declared_types().into_keys().collect();
        types.extend(self.boundary_only_types().into_keys());
        prebindgen_registry::Claimed {
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

    /// Every wrapper the model erases has a recipe here.
    ///
    /// The two lists answer different questions — the model's is "what do I
    /// erase", this file's is "what can I rebuild" — and they are allowed to
    /// disagree about *capability* (`Cow` is erased and cannot be read through).
    /// They are not allowed to disagree about *membership*: a wrapper that
    /// becomes transparent without a recipe here would be silently unbridgeable
    /// everywhere, which looks exactly like a type the binding got wrong.
    ///
    /// So adding `Rc` is: one entry in `TRANSPARENT_WRAPPERS`, one recipe in
    /// `WRAPPER_OPS` (`read: None` — an `Rc`'s payload cannot be moved out —
    /// and `build: Some(Rc::new)`). This test is what says so out loud instead
    /// of leaving the second step to be discovered.
    #[test]
    fn every_erased_wrapper_has_ops() {
        let missing: Vec<&str> = prebindgen_registry::flat::TRANSPARENT_WRAPPERS
            .iter()
            .copied()
            .filter(|w| wrapper_ops(w).is_none())
            .collect();
        assert!(
            missing.is_empty(),
            "the model erases {missing:?}, and this adapter has no `WrapperOps` recipe for them — \
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
            .filter(|n| !prebindgen_registry::flat::TRANSPARENT_WRAPPERS.contains(n))
            .collect();
        assert!(
            stray.is_empty(),
            "`WRAPPER_OPS` recipes for non-erased {stray:?}"
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
        let ty = crate::test_util::reading(syn::parse_quote!(Box<Box<Option<String>>>));
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
        let plain = crate::test_util::reading(syn::parse_quote!(Option<String>));
        assert!(plain.erased_wrappers().is_empty());
        for e in [
            build_through_erased_wrappers(&plain, quote!(v)),
            read_through_erased_wrappers(&plain, quote!(v)),
        ] {
            assert_eq!(e.expect("identity").to_string(), "v");
        }
    }

    /// `Cow` declines a rebuild, and the two directions decline for **different
    /// reasons** — which is why the recipe carries two `None`s rather than one
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
        let ty = crate::test_util::reading(syn::parse_quote!(Cow<'_, str>));
        assert_eq!(ty.erased_wrappers(), ["Cow"]);
        assert!(build_through_erased_wrappers(&ty, quote!(v)).is_none());
        assert!(read_through_erased_wrappers(&ty, quote!(v)).is_none());

        // A `Cow` under a `Box` declines too: one unbuildable layer refuses the
        // whole chain, rather than the `Box` half quietly succeeding.
        let nested = crate::test_util::reading(syn::parse_quote!(Box<Cow<'_, str>>));
        assert_eq!(nested.erased_wrappers(), ["Box", "Cow"]);
        assert!(build_through_erased_wrappers(&nested, quote!(v)).is_none());
    }
}
