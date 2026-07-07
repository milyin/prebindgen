//! JNI back-end for the Registry pipeline.
//!
//! [`JniGen`] implements [`crate::api::core::prebindgen::Prebindgen`]
//! (Rust-side conversion bodies) and provides an inherent
//! [`JniGen::write_kotlin`] for emitting all Kotlin output
//! (`NativeHandle.kt`, typed-handle classes, `JNIWrappers.kt`).
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

pub mod box_helpers;
pub mod byte_array_helpers;
pub mod iface_method;
pub mod jni_binding_error;
mod metadata;
pub mod string_helpers;
pub(crate) mod wire_access;

// Shared imports for this module and all its sibling submodules
// (`builder`, `trait_impl`, `emit`, `prim`, `kotlin_emit`, `render`, `fold`,
// `tests`). They are re-exported `pub(crate)` so each sibling only needs
// `use super::*;`.
pub(crate) use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    path::{Path, PathBuf},
    sync::Arc,
};

pub use box_helpers::{
    box_jboolean, box_jbyte, box_jchar, box_jdouble, box_jfloat, box_jint, box_jlong, box_jshort,
};
pub use byte_array_helpers::{decode_byte_array, encode_byte_array, null_byte_array};
pub use iface_method::CachedIfaceMethod;
pub use jni_binding_error::JniBindingError;
pub(crate) use metadata::{FoldStrategy, KotlinMeta, NullableKind, Projection, ProjectionKind};
pub(crate) use proc_macro2::{Span, TokenStream};
pub(crate) use quote::{format_ident, quote, ToTokens};
pub use string_helpers::{decode_string, encode_string, null_string};

// Kotlin-emission shared imports (used by `kotlin_emit` / `render` / `fold`).
pub(crate) use crate::api::gen::kotlin as kt;
pub(crate) use crate::api::{
    core::{
        niches::Niches,
        prebindgen::{ConverterImpl, Prebindgen, Stage},
        registry::{extract_fn_trait_args, Registry, TypeKey},
        types_util::{
            bare_path_ident, is_option_ref, is_option_type, option_inner_type, vec_inner_type,
        },
    },
    gen::kotlin::WriteKotlinError,
    lang::jnigen::{
        jni::wire_access::{box_descriptor_for_primitive, box_helper_for_wire, jni_field_access},
        util::snake_to_camel,
    },
};

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
/// Presence marker: a type registered via `JniGen::ptr_class`. The unified
/// Kotlin emitter writes a typed-handle `.kt` file (and the Rust side its
/// `freePtr` destructor) for every opaque type.
#[derive(Clone, Default)]
pub(crate) struct OpaqueConfig {}

/// Per-enum configuration (driven by `JniGen::enum_class`).
///
/// Marks a `#[prebindgen]`-scanned `enum` as a Kotlin enum class — the
/// rank-0 converter arms emit `jint ↔ Rust enum` bodies (via `TryFrom<i32>`
/// for input and `as jni::sys::jint` for output), and the Kotlin emitter
/// writes an `enum class` file with SCREAMING_SNAKE_CASE variants and a
/// discriminant-keyed `fromInt(...)` companion. The Kotlin FQN lives in
/// the surrounding [`TypeConfig::kotlin_name`] slot, same as
/// [`OpaqueConfig`].
/// Presence marker: a type registered via `JniGen::enum_class`. The unified
/// Kotlin emitter writes an `enum class` `.kt` file for every declared enum.
#[derive(Clone, Default)]
pub(crate) struct EnumConfig {}

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
}

impl MethodEntry {
    pub fn new(rust_ident: syn::Ident) -> Self {
        Self {
            rust_ident,
            kotlin_name_override: None,
        }
    }
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
    /// Set by [`JniGen::value_class`]: this is a `Copy` Rust type passed
    /// **by value as its raw memory blob** in a `JByteArray` (wire), the
    /// value-level peer of an opaque handle's `jlong`. No Kotlin class, no
    /// projection — it surfaces as `ByteArray`. Mutually exclusive with
    /// `opaque` / `enum_cfg`.
    pub value_blob: bool,
    /// Set by the four class declarators (`ptr_class` / `enum_class` /
    /// `data_class` / `value_class`), NOT by wrapper registration. Declared
    /// classes are required in **both** directions at scan (their converters
    /// always resolve both ways); a wrapper-only entry is required per
    /// **usage** direction, so an output-only wrapper needs no input twin.
    pub class_decl: bool,
}

/// Free-standing functions emitted into a synthetic package-level wrapper
/// object. One entry per `.package(subpackage)` context that
/// received `.fun(...)` calls.
#[derive(Clone, Default)]
pub(crate) struct PackageConfig {
    /// `#[prebindgen]` fns declared as free-standing wrappers under this
    /// subpackage via [`JniGen::fun`].
    pub functions: Vec<MethodEntry>,
}

/// What kind of class member a [`ClassMember`] is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum MemberKind {
    /// `f(&T, …) -> R`: promoted to an instance method, receiver bound to
    /// `this` and excluded from input-flatten; any remaining params flatten
    /// normally (a zero-extra-param fn is just the receiver-only case — no
    /// separate arity tracking needed, since there's nothing left to
    /// compose once the receiver is skipped).
    Fun,
    /// `f(…) -> T` / `Result<T,E>`: a factory emitted as a companion-object
    /// member returning the class; never output-flattened; referenceable by a
    /// `.default_param_variant(fun!(...))`.
    Constructor,
}

/// One `#[prebindgen]` function attached to a declared class (`ptr_class` /
/// `enum_class` / `value_class` / `data_class`) via [`JniGen::fun`] /
/// [`JniGen::constructor`]. Funs become **instance methods** (receiver
/// dropped→`this`); constructors become **companion factory** members. Each
/// is also a real `#[prebindgen]` wrapper (Rust extern + `JNINative` extern +
/// JSONL).
#[derive(Clone, Debug)]
pub(crate) struct ClassMember {
    /// Rust function ident (`registry.functions[ident]`).
    pub rust_ident: syn::Ident,
    /// Kotlin-visible name of this instance method / companion factory
    /// (derived from `FunctionDecl.name()`, defaulting to
    /// `snake_to_camel(rust_ident)`). Independent of any `.default_param_variant`/
    /// `.default_return_field` reference to the same underlying function — those
    /// take a fresh `FunctionDecl` directly and don't consult this list.
    pub kotlin_name: String,
    /// Member kind (fun / constructor).
    pub kind: MemberKind,
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
/// * `exc` — the bound domain error **as a Rust type**: the `E` peeled
///   from a source `Result<T, E>`, matched by exact canonical-form
///   equality (use the same full path the source signature uses, e.g.
///   `parse_quote!(zenoh_flat::errors::ZError)` — no short-name
///   matching). `Some(...)` ⇒ domain-fallible: the body evaluates to
///   `Result<ty, exc>` and is emitted as-is; a failure routes to the
///   wrapper's error sink (never a JVM throw). `None` ⇒ binding-fallible
///   only: the body evaluates to a bare `ty` and the framework wraps it
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

/// Closure that transforms a Kotlin short name. Installed via
/// [`JniGenConfig`]'s per-kind setters; the framework calls the matching
/// closure wherever it needs to derive a Kotlin/JNI short name for a
/// generated element. Closure-unset = identity.
pub(crate) type NameMangle = Arc<dyn Fn(&str) -> String + Send + Sync>;

/// JNI back-end. Accepts pre-built declaration objects (`PackageDecl`,
/// `ScalarTypeWrapperDecl`, `GenericTypeWrapperDecl` — see `decl.rs`) built
/// independently of `JniGen` itself; there is no fluent typestate cursor.
///
/// ```
/// use prebindgen::lang::{JniGen, JniGenConfig};
///
/// let jni = JniGen::new(JniGenConfig::new().package_prefix("io.test.jni"))
///     .package(
///         prebindgen::package!("session")
///             .class(prebindgen::ptr_class!(ZKeyExpr)
///                 .fun(prebindgen::fun!(z_keyexpr_as_str).name("getStr"))
///                 .default_return_field_self()),
///     );
/// ```
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
    /// Derived: `package.replace('.', '/')`. Read by
    /// [`struct_output_body`] when building `FindClass` strings.
    pub(crate) java_class_prefix: String,
    /// Derived: `"Java_" + package.replace('.', '_') + "_" +
    /// mangle_harness("Native")`. Read by [`mangle_jni_name`] when
    /// building the JNI extern symbol path for every emitted wrapper.
    pub(crate) jni_class_path: String,

    /// Mangler for function names (scanned `#[prebindgen]` free fns and
    /// the synthetic `freePtr` destructor). Default = identity; in
    /// zenoh-jni the closure returns `format!("{name}ViaJNI")` so the
    /// generated JNI extern symbols and matching Kotlin `external fun`s
    /// both pick up the `ViaJNI` suffix.
    pub(crate) kotlin_fun_name_mangle: Option<NameMangle>,
    /// Mangler for Kotlin ptr-class names declared via a
    /// `PtrClassDecl`. Default = identity.
    pub(crate) kotlin_ptr_class_name_mangle: Option<NameMangle>,
    /// Mangler for Kotlin data-class names declared via a
    /// `DataClassDecl`. Default = identity.
    pub(crate) kotlin_data_class_name_mangle: Option<NameMangle>,
    /// Mangler for `EnumClassDecl`-declared C-like enum class
    /// names. Default = identity.
    pub(crate) kotlin_enum_name_mangle: Option<NameMangle>,
    /// Mangler for the framework "harness" class name —
    /// `"Native"` (the centralized JNI extern holder). Default when
    /// unset = prepend `"JNI"`, so you get `JNINative`. Override to
    /// plug in a different convention.
    pub(crate) kotlin_harness_name_mangle: Option<NameMangle>,
    /// Derived `<rust-type-canonical-string> → <kotlin FQN>` view —
    /// populated alongside [`Self::types`] when accepting a `ClassDecl`
    /// (ptr/enum/data/value). Internal readers (`emit_into_dispatcher`)
    /// consume this flat list directly; the structured `types` map is the
    /// source of truth.
    pub(crate) kotlin_type_fqns: Vec<(String, String)>,

    /// Structured per-type configuration keyed by canonical Rust type.
    /// One entry per `Rust type ↔ JNI/Kotlin` rule; populated when accepting
    /// a `ClassDecl` or a `ScalarTypeWrapperDecl`. Holds opaque-handle
    /// config, enum config, and Kotlin names; the converter bodies
    /// themselves live in [`Self::input_wrappers`] /
    /// [`Self::output_wrappers`]. The rank-0 dispatch order is opaque →
    /// enum → wrapper-table → primitive → struct.
    pub(crate) types: HashMap<TypeKey, TypeConfig>,

    /// Free-standing package-level wrappers, keyed by subpackage path
    /// (relative to [`Self::package`], dot-separated; the empty key is the
    /// base package itself). Populated by [`JniGen::package`], merging into
    /// whatever the named subpackage already holds.
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

    /// When `true` (default), generated wrappers wrap each call that
    /// touches an opaque handle in the per-call `withSortedHandleLocks`
    /// scaffold (deadlock-safe N-ary monitor acquisition + atomic
    /// consume). When `false`, the scaffold is omitted — wrappers emit
    /// only the raw `ptr` read + closed-handle null-check + native call.
    /// Toggled via [`JniGenConfig::disable_handle_locks`].
    pub(crate) emit_handle_locks: bool,

    /// Optional Kotlin statement(s) to place inside an `init { … }` block of
    /// the generated centralized externs object (`JNINative`). Set via
    /// [`JniGenConfig::jni_native_init`]. Every generated native call routes
    /// through that object, so its `<clinit>` is the single point at which a
    /// consumer can trigger native-library loading (e.g.
    /// `"io.zenoh.jni.NativeLibrary.ensureLoaded()"`). `None` (default) emits no
    /// init block — loading stays the consumer's responsibility.
    pub(crate) jni_native_init: Option<String>,

    /// Constructor-expansion declarations (`.default_param_variant()` /    /// `.param_variant()` on a class/function decl). Resolved into
    /// [`crate::api::core::expand::FoldPlan`]s on the registry during
    /// `write_rust` and consumed at the parameter-emission site.
    pub(crate) expansions: crate::api::core::expand::Expansions,

    /// Output-expansion declarations (`.default_return_field()` /
    /// `.return_field()` on a class/function decl). Resolved into
    /// [`crate::api::core::unfold::UnfoldPlan`]s on the registry during
    /// `write_rust` and consumed at the return-emission site.
    pub(crate) deconstructors: crate::api::core::unfold::Deconstructors,

    /// Class members (funs / constructors) attached to a declared class via
    /// its decl's `.fun()`/`.constructor()`, keyed by the class's canonical
    /// Rust type. Supplies the instance-method / companion-factory emission
    /// and the receiver-skip set for input-flattening (see [`ClassMember`]).
    /// Insertion order within a class is preserved (the Vec); class emission
    /// iterates `types` by sorted key, so map order is irrelevant.
    pub(crate) class_members: HashMap<TypeKey, Vec<ClassMember>>,

    /// `#[prebindgen]` fns the binding deliberately does NOT wrap, declared
    /// via [`JniGen::ignore_fun`]. Backs [`Prebindgen::ignored_functions`]:
    /// suppresses the registry's per-item "skipping undeclared" warning
    /// without emitting anything.
    pub(crate) ignored_fns: std::collections::HashSet<syn::Ident>,

    /// `#[prebindgen]` types the binding deliberately does NOT declare,
    /// via [`JniGen::ignore_class`]. Backs [`Prebindgen::ignored_types`].
    pub(crate) ignored_class_types: std::collections::HashSet<TypeKey>,

    /// Every function ever referenced as a named leaf in a `.default_return_field(fun!(...))`/
    /// `.return_field(...)` record (class- or
    /// function-scoped) — populated as `builder.rs` accepts each decl.
    /// Backs [`Prebindgen::accessor_functions`]: `core/unfold.rs`'s
    /// deconstructor gate requires every named record's function to be in
    /// this set (`RecordNotAccessor` otherwise), and `core/expand.rs` excludes
    /// them from parameter composition. Derived from *usage*, not from any
    /// separate declaration — a function need not also be a `.fun()` class
    /// member to be referenced this way.
    pub(crate) accessor_record_fns: std::collections::HashSet<syn::Ident>,
}

// ── Sibling submodules (carved from the former monolithic file) ─────────
mod builder;
mod classify;
mod config;
mod decl;
mod emit;
mod iface;
mod prim;
mod selector;
#[cfg(test)]
mod tests;
mod trait_impl;

mod fold;
mod kotlin_emit;
mod render;
mod struct_plan;

pub(crate) use builder::*;
pub(crate) use classify::*;
pub use config::*;
pub use decl::*;
pub(crate) use emit::*;
pub(crate) use fold::*;
pub(crate) use iface::*;
pub(crate) use prim::*;
pub(crate) use render::*;
pub(crate) use struct_plan::*;
