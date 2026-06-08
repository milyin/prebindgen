//! JNI back-end for the Registry pipeline.
//!
//! [`JniGen`] implements [`crate::api::core::prebindgen::Prebindgen`]
//! (Rust-side conversion bodies) and provides an inherent
//! [`JniGen::write_kotlin`] for emitting all Kotlin output (per-callback
//! fun-interface files, `NativeHandle.kt`, typed-handle classes,
//! `JNIWrappers.kt`).
//!
//! The implementation is split across sibling submodules, all sharing this
//! `jni` module's namespace via the `pub(crate) use …::*` glob re-exports
//! below (each sibling needs only `use super::*;`):
//!   * this file — type / metadata definitions ([`JniGen`], [`KotlinMeta`],
//!     [`Projection`], [`FoldStrategy`], the config structs) + the shared imports;
//!   * `builder` — the [`JniGen`] builder API;
//!   * `trait_impl` — the [`Prebindgen`] impl + its converter-selector helpers;
//!   * `emit` — Rust-side `extern "C"` wrapper / converter-body emission;
//!   * `prim` — JNI primitive (un)boxing tables;
//!   * `kotlin_emit` / `render` / `fold` — the Kotlin source emitters.

pub mod byte_array_helpers;
pub mod jni_binding_error;
pub mod string_helpers;
pub(crate) mod templates;
pub(crate) mod wire_access;

pub use byte_array_helpers::{decode_byte_array, encode_byte_array, null_byte_array};
pub use jni_binding_error::JniBindingError;
pub use string_helpers::{decode_string, encode_string, null_string};

// Shared imports for this module and all its sibling submodules
// (`builder`, `trait_impl`, `emit`, `prim`, `kotlin_emit`, `render`, `fold`,
// `tests`). They are re-exported `pub(crate)` so each sibling only needs
// `use super::*;`.
pub(crate) use std::collections::{BTreeMap, HashMap};
pub(crate) use std::sync::Arc;

pub(crate) use proc_macro2::{Span, TokenStream};
pub(crate) use quote::{format_ident, quote, ToTokens};

pub(crate) use crate::api::core::niches::Niches;
pub(crate) use crate::api::core::prebindgen::{ConverterImpl, Prebindgen, Stage};
pub(crate) use crate::api::core::registry::{extract_fn_trait_args, Registry, TypeKey};
pub(crate) use crate::api::lang::jnigen::jni::wire_access::{box_class_for_wire, jni_field_access};
pub(crate) use crate::api::lang::jnigen::util::snake_to_camel;

// Kotlin-emission shared imports (used by `kotlin_emit` / `render` / `fold`).
pub(crate) use std::collections::{BTreeSet, HashSet};
pub(crate) use std::path::{Path, PathBuf};
pub(crate) use crate::api::lang::jnigen::kotlin::file::{KotlinFile, WriteKotlinError};

// ──────────────────────────────────────────────────────────────────────
// Language metadata (Prebindgen::Metadata for JniGen)
// ──────────────────────────────────────────────────────────────────────

/// Folded nullability / collection layers wrapping a closeable native
/// projection, outermost first. Mirrors how the type folds: the opaque-handle
/// leaf is [`FoldStrategy::Direct`]; an `Option<_>` wrapper adds
/// [`FoldStrategy::Nullable`]; a collection wrapper would add
/// [`FoldStrategy::Iterable`]. Drives both the typed Kotlin rendering of
/// a handle-bearing field/return and the generated `close()` expression,
/// uniformly across whatever wrappers compose.
/// How a `Nullable` fold layer represents `None` over the JNI wire.
///
/// The choice is made at the point the `Option<_>` rank-1 handler folds the
/// layer onto a projection's `FoldStrategy`, and only depends on whether
/// `option_output` rode the inner converter's niche (wire stayed identical to
/// the inner's wire) or boxed the primitive into `java.lang.<Box>` (wire
/// widened to `JObject`). The renderer reads this to pick the matching
/// Kotlin shape — without it, a primitive-wired `Option<Handle>` would be
/// declared as nullable `Long?` even though the wire is a non-nullable
/// `jlong` whose `0L` *is* the null.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NullableKind {
    /// Wire kept the inner converter's encoding; `None` is the carved niche
    /// slot's `value` (e.g. `0L` for a handle's `jlong`). On Kotlin the
    /// declared wire is non-nullable; the wrapper body converts the sentinel
    /// to `null` explicitly via an `if (it == <sentinel>) null else W(it)`
    /// pattern.
    Niche,
    /// Wire widened to `JObject`; `None` is JVM `null`. On Kotlin the
    /// declared wire is nullable and `?.let { W(it) }` works directly. This
    /// is also the rendering object-shaped niches (`JByteArray::null` /
    /// `JString::null`) collapse onto — Kotlin's `T?` already maps to JVM
    /// reference-null at no extra cost.
    Boxed,
}

#[derive(Clone, Debug)]
pub enum FoldStrategy {
    /// The receiver *is* the handle.
    Direct,
    /// `T?` — receiver may be null. `kind` records how null is represented
    /// over the wire (see [`NullableKind`]).
    Nullable {
        kind: NullableKind,
        inner: Box<FoldStrategy>,
    },
    /// `List<T>` — receiver is a collection. EXTENSION POINT: no
    /// `Vec<Handle>` shape exists today, so the emitters guard this arm
    /// loudly rather than silently mis-generating.
    Iterable(Box<FoldStrategy>),
}

/// Which flavor of Kotlin newtype a [`Projection`] surfaces. Both share the
/// same "wire ≠ declared Kotlin type, wrap as `W(wire)`, fold through
/// `Option`/`Vec`" shape; they differ only in how a struct field stores them
/// and whether they own a closeable resource.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProjectionKind {
    /// Opaque native handle (`ptr_class`). Wire is `jlong`; a struct field
    /// stores the **boxed** handle object (`L<fqn>;`); closeable when owned.
    Handle,
    /// Kotlin `@JvmInline value class` wrapping a **`Copy` value-blob**
    /// (`value_blob`). Its inner is always a raw `ByteArray` (`[B`) — there is
    /// no Rust struct field to resolve. The typed class has a single
    /// `bytes: ByteArray` field; the wire is `JByteArray`. Never closeable.
    ValueBlob,
}

/// Folded description of a Kotlin newtype projection (opaque handle or value
/// class) reached through zero or more wrapper layers. Set at the leaf,
/// transformed by each wrapper as the type folds (see [`FoldStrategy`]), and
/// read by every typed-surface emitter (data-class fields, struct
/// encode/decode, `classify_return`, param classification) so "what Kotlin
/// class does this surface, how do I wrap/fold it, do I close it" has one
/// source of truth instead of a parallel ad-hoc decision tree.
#[derive(Clone, Debug)]
pub struct Projection {
    /// Canonical [`TypeKey`] string of the leaf type (e.g. `"ZKeyExpr"`,
    /// `"ZenohId"`); look up [`JniGen::kotlin_type_fqns`] for the typed
    /// Kotlin FQN.
    pub leaf_key: String,
    /// `false` for `&T` borrows of a handle — still a projection (param
    /// classification needs this), but not the holder's to close, so
    /// `close()` emission skips it. Always `false` for [`ProjectionKind::ValueBlob`].
    pub owned: bool,
    /// Nullability / collection layers.
    pub strategy: FoldStrategy,
    /// Handle vs value class — see [`ProjectionKind`].
    pub kind: ProjectionKind,
}

/// Per-converter language-specific extras carried by every
/// [`ConverterImpl`] this back-end produces. Filled by the same handler
/// that builds the wire/body, propagated by the resolver into
/// [`crate::api::core::registry::TypeEntry::metadata`], and read directly by
/// the Kotlin emitter — so cross-language facts flow through the
/// existing wrapper machinery rather than a parallel side channel.
#[derive(Clone, Debug, Default)]
pub struct KotlinMeta {
    /// Value-context Kotlin type name. `Some("Long")` for opaque
    /// handles (jlong wire mention), `Some("io.zenoh.jni.JNIEncoding")`
    /// for user-declared decoder types whose wire isn't primitive,
    /// `Some("List<ByteArray>")` when a wrapper composes a primitive
    /// inner. `None` only for entries that must not appear in any
    /// Kotlin signature — the emitter treats that as a hard error.
    pub kotlin_name: Option<String>,
    /// For wrapper converters whose Kotlin projection is the *inner*
    /// type's projection (e.g. `ZResult<Publisher>` → `Publisher`),
    /// this carries the inner Rust type's canonical key so downstream
    /// emitters (typed-handle constructor lookup in
    /// `classify_return`) can find the
    /// wrapped value's identity without baking in any specific shape
    /// (no `peel_zresult` / `peel_result`-style framework hardcoding).
    /// Populated with `args[0]`'s canonical key for arity-1 wrappers, and
    /// inherited by the built-in `Option<_>` / `Vec<_>` / `&_` rank-1
    /// handlers from their inner type's metadata. `None` for plain values
    /// and arity-0 converters.
    pub value_rust_key: Option<String>,
    /// Present iff this (possibly wrapped) value is an opaque native
    /// handle. Set at the opaque-handle leaf and folded outward by the
    /// rank-1 `&_` / `Option<_>` handlers and the `lookup_*` composed
    /// branches. The single source of truth for typed-handle rendering
    /// and `close()` generation — see [`Projection`].
    pub projection: Option<Projection>,
}

impl KotlinMeta {
    pub fn from_name(name: impl Into<String>) -> Self {
        Self {
            kotlin_name: Some(name.into()),
            value_rust_key: None,
            projection: None,
        }
    }

    /// True iff this (input-direction) converter decodes a directly-
    /// consumable owned opaque handle — i.e. its projection is a bare
    /// `Handle` leaf with no `Option`/`Vec` fold. Replaces the former
    /// `converter_returns_owned_object` return-type AST sniff; the two are
    /// equivalent for every input converter this back-end produces (the
    /// only input converters returning `Result<OwnedObject<_>, _>` are the
    /// opaque-handle leaf and the `&_`/`&mut _` arm that shares its
    /// function, both carrying a `Direct` `Handle` projection).
    pub(crate) fn is_direct_handle(&self) -> bool {
        self.projection.as_ref().is_some_and(|p| {
            p.kind == ProjectionKind::Handle && matches!(p.strategy, FoldStrategy::Direct)
        })
    }
}

// ──────────────────────────────────────────────────────────────────────
// Structured type-conversion configuration
// ──────────────────────────────────────────────────────────────────────

/// Per-opaque-handle configuration (driven by `JniGen::ptr_class`).
///
/// The typed-handle Kotlin FQN (e.g. `"io.zenoh.jni.JNISession"`) lives
/// in the surrounding [`TypeConfig::kotlin_name`] slot — FQN-consumers
/// (typed-handle class emission, `instanceof` dispatch,
/// return-value constructor wrap) read it from there. The
/// value-context Kotlin name for the same type (`"Long"`) is produced
/// independently by the rank-0 opaque handler in [`KotlinMeta`], so
/// the two roles don't collide despite sharing the `TypeConfig`.
#[derive(Clone, Default)]
pub(crate) struct OpaqueConfig {
    /// When `false` (default), the unified Kotlin emitter writes a
    /// typed-handle `.kt` file for this opaque type. Set to `true` by
    /// [`JniGen::suppress_kotlin_code`] to indicate the Kotlin file is
    /// hand-maintained — only the Rust-side converter and `instanceof`
    /// dispatch wire up.
    pub suppress_kotlin_code: bool,
}

/// Per-enum configuration (driven by `JniGen::enum_class`).
///
/// Marks a `#[prebindgen]`-scanned `enum` as a Kotlin enum class — the
/// rank-0 converter arms emit `jint ↔ Rust enum` bodies (via `TryFrom<i32>`
/// for input and `as jni::sys::jint` for output), and the Kotlin emitter
/// writes an `enum class` file with SCREAMING_SNAKE_CASE variants and a
/// discriminant-keyed `fromInt(...)` companion. The Kotlin FQN lives in
/// the surrounding [`TypeConfig::kotlin_name`] slot, same as
/// [`OpaqueConfig`].
#[derive(Clone, Default)]
pub(crate) struct EnumConfig {
    /// When `false` (default), the unified Kotlin emitter writes an
    /// `enum class` `.kt` file for this enum. Set to `true` by
    /// [`JniGen::suppress_kotlin_code`] when the Kotlin source is
    /// hand-maintained — only the Rust-side converter wires up.
    pub suppress_kotlin_code: bool,
}

/// One registered `.fun(...)` entry. The Rust identifier is captured
/// at build-script time via `syn::parse_quote` (i.e. `pq!(rust_fn_name)`); the
/// optional override sets the Kotlin-side name when the default
/// `snake_to_camel(rust_ident)` derivation isn't what the user wants.
#[derive(Clone, Debug)]
pub struct MethodEntry {
    /// Rust function ident — must match a `#[prebindgen]`-marked free
    /// function in the registered source module. Looked up by
    /// `registry.functions[ident]`.
    pub rust_ident: syn::Ident,
    /// Kotlin-side name override, set by chaining `.name("...")` after
    /// the entry's registration. `None` = derive from `rust_ident` via
    /// `snake_to_camel`.
    pub kotlin_name_override: Option<String>,
    /// `true` when declared via `.fun_accessor(...)` (a read accessor): the
    /// parameter composer (constructor `.default()` / explicit `.construct`) is
    /// never applied to it, and it is the only kind of function a decomposer
    /// record (`.deconstructor_record`/`.converter`/`_nested`) may reference.
    pub is_accessor: bool,
}

impl MethodEntry {
    pub fn new(rust_ident: syn::Ident) -> Self {
        Self {
            rust_ident,
            kotlin_name_override: None,
            is_accessor: false,
        }
    }

    /// A `.fun_accessor(...)` entry — see [`Self::is_accessor`].
    pub fn new_accessor(rust_ident: syn::Ident) -> Self {
        Self {
            rust_ident,
            kotlin_name_override: None,
            is_accessor: true,
        }
    }
}

/// Back-pointer to the last entry added via `.fun` / `.fun_accessor`, used by
/// `.name(...)` to find what to mutate. Cleared by every other builder call.
#[derive(Clone, Debug)]
pub(crate) enum NamedEntryRef {
    /// Index into `packages[subpackage].functions`.
    Function(String, usize),
}

/// All configuration the structured builder accumulates for one
/// canonical Rust type key. Every field is `None` by default;
/// builder methods populate the ones they care about.
#[derive(Clone, Default)]
pub(crate) struct TypeConfig {
    /// Short Kotlin name or FQN. Required for any type emitted in
    /// Kotlin (`Sample` → `"io.zenoh.jni.Sample"`,
    /// `Vec<u8>` → `"ByteArray"`).
    pub kotlin_name: Option<String>,
    /// If `Some`, this is an opaque-handle type — gets jlong wire,
    /// `Box::into_raw`/`Box::from_raw` conventions, instanceof
    /// dispatch, and Kotlin typed-handle class emission.
    pub opaque: Option<OpaqueConfig>,
    /// If `Some`, this is a `#[prebindgen]` enum mirrored as a Kotlin
    /// `enum class` — gets jint wire (input + output via `TryFrom<i32>`
    /// / `as jint`) and a generated `.kt` file. Mutually exclusive with
    /// [`Self::opaque`]; builder enforces it.
    pub enum_cfg: Option<EnumConfig>,
    /// Kotlin FQN override for `impl Fn(...)` keys (the
    /// closure-mangled callback name, e.g. zenoh-jni stamps `JNIOn...`
    /// here via [`JniGen::auto_callback_fqn`]).
    pub callback_kotlin_fqn: Option<String>,
    /// Set by [`JniGen::value_blob`]: this is a `Copy` Rust type passed
    /// **by value as its raw memory blob** in a `JByteArray` (wire), the
    /// value-level peer of an opaque handle's `jlong`. No Kotlin class, no
    /// projection — it surfaces as `ByteArray`. Mutually exclusive with
    /// `opaque` / `enum_cfg`.
    pub value_blob: bool,
}

/// Free-standing functions emitted into a synthetic package-level wrapper
/// object. One entry per `.package(subpackage)` context that
/// received `.function(...)` calls.
#[derive(Clone, Default)]
pub(crate) struct PackageConfig {
    /// `#[prebindgen]` fns declared as free-standing wrappers under this
    /// subpackage via [`JniGen::function`].
    pub functions: Vec<MethodEntry>,
}

/// Boxed closure that builds a converter when applied to the wildcard
/// substitutions. Returns `None` to defer (an inner converter the
/// builder depends on isn't yet resolved; the resolver retries on the
/// next phase), or `Some((ty, exc, body))` where:
///
/// * `ty` — the type the body produces. Auto-classified at lookup:
///   a wire shape (or the self-converter case) ⇒ terminal converter
///   with `destination = ty`; a rust type with its own converter ⇒
///   composed as a value-inspecting stage onto that converter's chain.
/// * `exc` — the bound exception **as a Rust type**, matched by exact
///   canonical-form equality against a [`JniGen::throwable`]
///   registration's `rust_type` (use the same full path the
///   registration was declared with, e.g.
///   `parse_quote!(zenoh_flat::errors::ZError)` — no short-name
///   matching). `Some(...)` ⇒ throwing: the body evaluates to
///   `Result<ty, exc>` and is emitted as-is. `None` ⇒ non-throwing:
///   the body evaluates to a bare `ty` and the framework wraps it
///   `Ok(body)` with `Result<ty, __JniErr>` (= `JniBindingError`).
/// * `body` — the closure body. The decision between Ok-wrap vs
///   verbatim is keyed on `exc` (see [`JniGen::build_input_fn`] /
///   [`JniGen::build_output_fn`]).
///
/// Receives `&Registry<KotlinMeta>` so the closure can look up
/// inner-type entries (`registry.output_entry(t)`).
pub(crate) type WrapperFn = Arc<
    dyn Fn(&[syn::Type], &Registry<KotlinMeta>) -> Option<(syn::Type, Option<syn::Type>, syn::Expr)>
        + Send
        + Sync,
>;

/// Closure that transforms a Kotlin short name. Installed via the
/// per-kind setters ([`JniGen::kotlin_fun_name_mangle`],
/// [`JniGen::kotlin_data_class_name_mangle`], etc.); the framework calls
/// the matching closure wherever it needs to derive a Kotlin/JNI
/// short name for a generated element. Closure-unset = identity.
pub(crate) type NameMangle = Arc<dyn Fn(&str) -> String + Send + Sync>;

/// Trait selecting the arity-appropriate impl of
/// [`JniGen::input_wrapper`] / [`JniGen::output_wrapper`]. The phantom
/// type parameter discriminates closures of arity 0..3 so a single
/// public method name accepts any of them. Closures take the wildcard
/// substitutions plus the registry, and return `Some((ty, exc, body))`
/// or `None` (defer to a later resolver phase). See [`WrapperFn`] for
/// the triple's semantics.
pub trait WrapperBuilder<Arity>: Send + Sync + 'static {
    fn into_wrapper_fn(self) -> WrapperFn;
    fn rank() -> usize;
}

/// Arity-discriminating marker types. `Arity0` is for non-wildcard
/// patterns (e.g. `"i32"`); `Arity1`/`2`/`3` carry that many `_` slots.
pub struct Arity0;
pub struct Arity1;
pub struct Arity2;
pub struct Arity3;

impl<F> WrapperBuilder<Arity0> for F
where
    F: Fn(&Registry<KotlinMeta>) -> Option<(syn::Type, Option<syn::Type>, syn::Expr)>
        + Send
        + Sync
        + 'static,
{
    fn into_wrapper_fn(self) -> WrapperFn {
        Arc::new(move |_args: &[syn::Type], reg: &Registry<KotlinMeta>| self(reg))
    }
    fn rank() -> usize {
        0
    }
}

impl<F> WrapperBuilder<Arity1> for F
where
    F: Fn(&syn::Type, &Registry<KotlinMeta>) -> Option<(syn::Type, Option<syn::Type>, syn::Expr)>
        + Send
        + Sync
        + 'static,
{
    fn into_wrapper_fn(self) -> WrapperFn {
        Arc::new(move |args: &[syn::Type], reg: &Registry<KotlinMeta>| self(&args[0], reg))
    }
    fn rank() -> usize {
        1
    }
}

impl<F> WrapperBuilder<Arity2> for F
where
    F: Fn(
            &syn::Type,
            &syn::Type,
            &Registry<KotlinMeta>,
        ) -> Option<(syn::Type, Option<syn::Type>, syn::Expr)>
        + Send
        + Sync
        + 'static,
{
    fn into_wrapper_fn(self) -> WrapperFn {
        Arc::new(move |args: &[syn::Type], reg: &Registry<KotlinMeta>| {
            self(&args[0], &args[1], reg)
        })
    }
    fn rank() -> usize {
        2
    }
}

impl<F> WrapperBuilder<Arity3> for F
where
    F: Fn(
            &syn::Type,
            &syn::Type,
            &syn::Type,
            &Registry<KotlinMeta>,
        ) -> Option<(syn::Type, Option<syn::Type>, syn::Expr)>
        + Send
        + Sync
        + 'static,
{
    fn into_wrapper_fn(self) -> WrapperFn {
        Arc::new(move |args: &[syn::Type], reg: &Registry<KotlinMeta>| {
            self(&args[0], &args[1], &args[2], reg)
        })
    }
    fn rank() -> usize {
        3
    }
}

/// JNI back-end. Configure paths in the Rust crate (zresult, throw macro,
/// source module the original fns live in) and the JNI/Kotlin classpath
/// (java class prefix, callback Kotlin package + output dir).
#[derive(Clone)]
pub struct JniGen {
    /// Module path the original `#[prebindgen]` fns live under (e.g.
    /// the host crate of `#[prebindgen]` items). The wrapper body calls
    /// `<source_module>::<fn>(args)`.
    pub source_module: syn::Path,
    /// Single source of truth for the JVM/Kotlin namespace this binding
    /// targets, dot-separated (e.g. `io.zenoh.jni`). Empty = no prefix.
    /// Drives every derived form: slash-separated for `FindClass`,
    /// `_`-mangled for JNI extern idents, and dot-separated for Kotlin
    /// `package` declarations.
    pub package: String,
    /// Sub-package leaf appended to [`Self::package`] for the auto-emitted
    /// callback fun-interface files. Combined as
    /// `<package>.<callback_subpackage>`; empty = same package as
    /// [`Self::package`].
    pub callback_subpackage: String,
    /// Derived: `package.replace('.', '/')`. Read by
    /// [`struct_output_body`] when building `FindClass` strings.
    pub(crate) java_class_prefix: String,
    /// Derived: `"Java_" + package.replace('.', '_') + "_" +
    /// mangle_harness("Native")`. Read by [`mangle_jni_name`] when
    /// building the JNI extern symbol path for every emitted wrapper.
    pub(crate) jni_class_path: String,
    /// Derived: `package + "." + callback_subpackage` (or just `package`
    /// when the subpackage is empty). Also drives the on-disk subdirectory
    /// under the `kotlin_root` passed to [`Self::write_kotlin`]
    /// (`a.b.c` → `a/b/c/`).
    pub(crate) kotlin_callback_package: String,

    /// Mangler for function names (scanned `#[prebindgen]` free fns and
    /// the synthetic `freePtr` destructor). Default = identity; in
    /// zenoh-jni the closure returns `format!("{name}ViaJNI")` so the
    /// generated JNI extern symbols and matching Kotlin `external fun`s
    /// both pick up the `ViaJNI` suffix.
    pub(crate) kotlin_fun_name_mangle: Option<NameMangle>,
    /// Mangler for Kotlin ptr-class names declared via
    /// [`Self::ptr_class`]. Default = identity.
    pub(crate) kotlin_ptr_class_name_mangle: Option<NameMangle>,
    /// Mangler for Kotlin data-class names declared via
    /// [`Self::data_class`]. Default = identity.
    pub(crate) kotlin_data_class_name_mangle: Option<NameMangle>,
    /// Mangler for [`Self::enum_class`]-declared C-like enum class
    /// names. Default = identity.
    pub(crate) kotlin_enum_name_mangle: Option<NameMangle>,
    /// Mangler for the package-level wrapper object created by
    /// [`Self::package`]. Default = identity.
    pub(crate) kotlin_package_name_mangle: Option<NameMangle>,
    /// Mangler for `impl Fn(...)` callback Kotlin class names. The
    /// closure receives the auto-derived callback name
    /// (`derive_callback_name`, always
    /// concatenated parameter type shorts + `"Callback"` suffix — e.g.
    /// `"QueryCallback"`, `"ReplyCallback"`, `"Callback"` for `Fn()`);
    /// the return value is qualified against
    /// [`Self::kotlin_callback_package`]. Default = identity.
    pub(crate) kotlin_callback_name_mangle: Option<NameMangle>,
    /// Mangler for rank-0 user-registered
    /// [`Self::input_wrapper`] / [`Self::output_wrapper`] pattern names
    /// — the Rust short name of the pattern (`Encoding`,
    /// `SetIntersectionLevel`, …). Rank-N wrappers (`Option<_>`,
    /// `ZResult<_>`, `Vec<_>`, `&_`) are NOT routed through any
    /// mangler — they inherit from the inner type's metadata via the
    /// existing rank-N handlers. Default = identity.
    pub(crate) kotlin_wrapper_name_mangle: Option<NameMangle>,
    /// Mangler for the framework "harness" class name —
    /// `"Native"` (the centralized JNI extern holder). Default when
    /// unset = prepend `"JNI"`, so you get `JNINative`. Override to
    /// plug in a different convention.
    pub(crate) kotlin_harness_name_mangle: Option<NameMangle>,
    /// Derived `<rust-type-canonical-string> → <kotlin FQN>` view —
    /// populated alongside [`Self::types`] by the structured builders
    /// ([`Self::ptr_class`], [`Self::data_class`],
    /// [`Self::callback_input`], [`Self::input_wrapper`] /
    /// [`Self::output_wrapper`]). Internal readers
    /// (`emit_into_dispatcher`, callback FQN merging) consume this flat
    /// list directly; the structured `types` map is the source of
    /// truth.
    pub(crate) kotlin_type_fqns: Vec<(String, String)>,

    /// Structured per-type configuration keyed by canonical Rust type.
    /// One entry per `Rust type ↔ JNI/Kotlin` rule; populated by the
    /// structured builders (`ptr_class`, `enum_class`,
    /// `data_class`, `input_wrapper`, `output_wrapper`,
    /// `callback_input`). Holds opaque-handle config, enum config,
    /// Kotlin names, and callback FQNs; the converter bodies themselves
    /// live in [`Self::input_wrappers`] / [`Self::output_wrappers`].
    /// The rank-0 dispatch order is opaque → enum → wrapper-table →
    /// primitive → struct.
    pub(crate) types: HashMap<TypeKey, TypeConfig>,

    /// Free-standing package-level wrappers, keyed by subpackage path
    /// (relative to [`Self::package`], dot-separated; never empty for an
    /// entry to be emitted). Populated by [`Self::function`] under the
    /// currently-active [`Self::active_subpackage`].
    pub(crate) packages: BTreeMap<String, PackageConfig>,

    /// Per-rank input converters — index `n` holds rank-`n` registrations
    /// keyed by the pattern's `TypeKey`. Rank 0 is non-wildcard (e.g.
    /// `"i32"`); ranks 1..3 carry that many `_` slots (e.g. `"Vec < _ >"`).
    /// Each [`WrapperFn`] closure carries the builder body AND the bound
    /// exception (the closure returns `(ty, exc, body)`); terminal vs
    /// composed is derived at lookup time, throwing vs non-throwing
    /// from the closure's `Option<String>` middle slot.
    pub(crate) input_wrappers: [HashMap<TypeKey, WrapperFn>; 4],

    /// Per-rank output converters. Same shape as [`Self::input_wrappers`].
    pub(crate) output_wrappers: [HashMap<TypeKey, WrapperFn>; 4],

    /// Tracks the last [`Self::ptr_class`] key registered so
    /// [`Self::method`] / [`Self::suppress_kotlin_code`] know which
    /// entry to extend. Cleared after each unrelated builder call.
    last_opaque_key: Option<TypeKey>,

    /// Tracks the last rank-0 wrapper registration so chained per-type
    /// builders ([`Self::suppress_kotlin_code`], [`Self::with_kotlin_type`])
    /// know which entry to extend. Set by `input_wrapper` /
    /// `output_wrapper` (rank 0 only), `enum_class`, `callback_input`,
    /// and `data_class`; cleared by other unrelated builders.
    last_meta_key: Option<TypeKey>,

    /// The currently-active subpackage set by [`Self::package`].
    /// Drives where [`Self::function`] entries land and is folded into
    /// the FQN of any class declared while it's `Some(_)`. Package
    /// inheritance via chaining is **not** supported — each
    /// `package` call overwrites the previous; nest via dotted
    /// paths (`package("a.b")`) instead.
    pub(crate) active_subpackage: Option<String>,

    /// Back-pointer to the entry the next [`Self::name`] call should
    /// mutate (the most recent `.method` / `.companion_method` /
    /// `.function`). Cleared by every other builder call so `.name(...)`
    /// only applies right after a fn-entry registration.
    last_entry_ref: Option<NamedEntryRef>,

    /// When `true` (default), generated wrappers wrap each call that
    /// touches an opaque handle in the per-call `withSortedHandleLocks`
    /// scaffold (deadlock-safe N-ary monitor acquisition + atomic
    /// consume). When `false`, the scaffold is omitted — wrappers emit
    /// only the raw `ptr` read + closed-handle null-check + native call.
    /// Toggled via [`Self::handle_locks`].
    pub(crate) emit_handle_locks: bool,

    /// Constructor-expansion declarations (`.constructor`,
    /// `.constructor`, `.expand`, …). Resolved into
    /// [`crate::api::core::expand::FoldPlan`]s on the registry during
    /// `write_rust` and consumed at the parameter-emission site.
    pub(crate) expansions: crate::api::core::expand::Expansions,

    /// Output-expansion declarations (`.deconstructor`,
    /// `.deconstructor_record*`, `.converter`, `.deconstruct_output`,
    /// `.convert_output`, …). Resolved into
    /// [`crate::api::core::unfold::UnfoldPlan`]s on the registry during
    /// `write_rust` and consumed at the return-emission site.
    pub(crate) deconstructors: crate::api::core::unfold::Deconstructors,
}


// ── Sibling submodules (carved from the former monolithic file) ─────────
mod builder;
mod emit;
mod prim;
mod trait_impl;
#[cfg(test)]
mod tests;

mod fold;
mod kotlin_emit;
mod render;

pub(crate) use builder::*;
pub(crate) use emit::*;
pub(crate) use fold::*;
pub(crate) use prim::*;
pub(crate) use render::*;
