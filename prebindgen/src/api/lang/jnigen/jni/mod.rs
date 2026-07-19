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
pub(crate) struct OpaqueConfig {
    /// `ptr_class!(X).gc_managed()`: the typed handle stores its pointer in
    /// a separate atomic cell and registers a `Cleaner` action that frees the
    /// native box if no other release path (close/take/consumption) won the
    /// untagged→tagged CAS ticket first.
    pub gc_managed: bool,
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
/// Presence marker: a type registered via `JniGen::enum_class`. The unified
/// Kotlin emitter writes an `enum class` `.kt` file for every declared enum.
#[derive(Clone, Default)]
pub(crate) struct EnumConfig {}

/// One registered package-level `.fun(...)` entry. The Rust identifier is captured
/// at build-script time via `syn::parse_quote` (i.e. `pq!(rust_fn_name)`); the
/// optional override sets the Kotlin-side name when the default
/// `snake_to_camel(rust_ident)` derivation isn't what the user wants.
#[derive(Clone, Debug)]
pub struct FunctionEntry {
    /// Rust function ident — must match a `#[prebindgen]`-marked free
    /// function in the registered source module. Looked up by
    /// `registry.functions[ident]`.
    pub rust_ident: syn::Ident,
    /// Kotlin-side name override, set by chaining `.name("...")` after
    /// the entry's registration. `None` = derive from `rust_ident` via
    /// `snake_to_camel`, then apply the target package's function hook.
    pub kotlin_name_override: Option<String>,
}

impl FunctionEntry {
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
    /// Raw naming spec of the type as declared — verbatim Kotlin type or
    /// settings-derived class name. Required for any type emitted in
    /// Kotlin; the concrete FQN (`Sample` → `"io.zenoh.jni.Sample"`,
    /// `Vec<u8>` → `"ByteArray"`) is materialized only at read time via
    /// [`JniGen::fqn_of`], which is what makes the `set_*` settings
    /// order-independent w.r.t. declarations.
    pub name_spec: Option<NameSpec>,
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
    /// Emit the generated interface mirroring the class's public instance
    /// surface, and make the class implement it with `override` on every
    /// class-body member (`.interface()` / implied by `.interface_name()`).
    pub interface_enabled: bool,
    /// Per-decl literal interface name (`.interface_name(...)`, relative, no
    /// dots) — bypasses the `set_interface_name_mangle` hook.
    pub interface_name_override: Option<String>,
    /// Kotlin interfaces added to the generated class's supertype list
    /// (`.implements(...)`, any class kind) — the class implements them
    /// nominally; the class body and lifecycle members are unaffected.
    /// Orthogonal to [`Self::interface_enabled`].
    pub interfaces: Vec<String>,
}

/// Free-standing functions emitted into a synthetic package-level wrapper
/// object. One entry per `.package(subpackage)` context that
/// received `.fun(...)` calls.
#[derive(Clone, Default)]
pub(crate) struct PackageConfig {
    /// `#[prebindgen]` fns declared as free-standing wrappers under this
    /// subpackage via [`JniGen::fun`].
    pub functions: Vec<FunctionEntry>,
    /// `#[prebindgen]` consts declared under this subpackage via
    /// [`PackageDecl::constant`] — each surfaces as a top-level Kotlin `val`
    /// initialized through a generated nullary JNI getter. `FunctionEntry`
    /// is reused as-is (rust ident + Kotlin-name override).
    pub constants: Vec<FunctionEntry>,
    /// Fn-sourced constants declared via [`ConstDecl::fun`]:
    /// nullary `#[prebindgen]` fns whose result surfaces as a top-level
    /// Kotlin `val` (eagerly initialized through the fn's ordinary generated
    /// wrapper) instead of a callable `fun`. Rust-side emission and the
    /// `JNINative` extern are the plain declared-function ones.
    pub constant_functions: Vec<FunctionEntry>,
    /// Expression-backed constants declared via
    /// [`ConstDecl::expr`](super::jni::decl::ConstDecl::expr):
    /// binding-defined expressions evaluated once inside a generated nullary
    /// getter (extern symbol seeded from the val name), surfacing as
    /// top-level Kotlin `val`s. Stored as the full decl — there is no Rust
    /// item behind them.
    pub constant_exprs: Vec<super::jni::decl::ConstExprDecl>,
}

/// What kind of class member a [`ClassMember`] is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum MemberKind {
    /// `f(&T, …) -> R`: promoted to an instance method, receiver bound to
    /// `this` and excluded from input-flatten; any remaining params flatten
    /// normally (a zero-extra-param fn is just the receiver-only case — no
    /// separate arity tracking needed, since there's nothing left to
    /// compose once the receiver is skipped).
    Method,
    /// `f(…) -> T` / `Result<T,E>`: a factory emitted as a companion-object
    /// member returning the class; never output-flattened; referenceable by a
    /// a `expand_param!` `.variant(fun!(...))` arm.
    Constructor,
}

/// One `#[prebindgen]` function attached to a declared class (`ptr_class` /
/// `value_class` / `data_class`) via a declaration's `.method(...)` /
/// `.constructor(...)`. Methods become **instance methods** (receiver
/// dropped→`this`); constructors become **companion factory** methods. Each
/// is also a real `#[prebindgen]` wrapper (Rust extern + `JNINative` extern +
/// JSONL).
#[derive(Clone, Debug)]
pub(crate) struct ClassMember {
    /// Rust function ident (`registry.functions[ident]`).
    pub rust_ident: syn::Ident,
    /// Per-member `.name()` override, stored RAW — the effective Kotlin
    /// name is derived at point of use by [`JniGen::class_method_kotlin_name`]
    /// (override, else the package/class-aware method hook over the full
    /// camelCase ident), keeping `set_method_name_mangle` order-independent. An
    /// `expand_return!` `.field` referencing the same underlying function
    /// inherits the effective name unless it sets its own `.name()`;
    /// `expand_param!` variants reference the fn by ident only.
    pub kotlin_name_override: Option<String>,
    /// Member kind (method / constructor).
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

/// Closure that transforms a Kotlin short name with the fully-qualified
/// package in which the named object is emitted. Installed via [`JniGen`]'s
/// per-kind `set_*_name_mangle` setters. Closure-unset = identity.
pub(crate) type NameMangle = Arc<dyn Fn(&str, &str) -> String + Send + Sync>;

/// Closure that transforms the centralized JNI harness class short name.
/// The harness always lives in the configured base package, so no placement
/// context is needed. Closure-unset = identity.
pub(crate) type HarnessNameMangle = Arc<dyn Fn(&str) -> String + Send + Sync>;

/// Closure that transforms a Kotlin method name with both its containing
/// package and final class short name. This is distinct from [`NameMangle`]
/// because flat Rust APIs conventionally encode the class namespace in the
/// function identifier (`z_session_put`), while a Kotlin method already lives
/// inside that class.
pub(crate) type MethodNameMangle = Arc<dyn Fn(&str, &str, &str) -> String + Send + Sync>;

/// JNI back-end. Global settings are applied with the order-insensitive
/// `set_*` methods; declarations are accepted as pre-built objects
/// (`PackageDecl`, `ExpandParamDecl`, `ExpandReturnDecl`,
/// `ConvertDecl` — see `decl.rs`) built
/// independently of `JniGen` itself; there is no fluent typestate cursor.
///
/// ```
/// use prebindgen::lang::JniGen;
///
/// let jni = JniGen::new()
///     .set_package_prefix("io.test.jni")
///     .package(
///         prebindgen::package!("keyexpr")
///             .class(prebindgen::ptr_class!(KeyExpr)
///                 .method(prebindgen::fun!(keyexpr_get_str).name("getStr"))
///                 .constructor(prebindgen::fun!(keyexpr_new_try_from).name("tryFrom"))),
///     )
///     // A KeyExpr param accepts EITHER a String (built via tryFrom) OR an
///     // existing handle; a returned KeyExpr decomposes into its string form.
///     .expand(
///         prebindgen::expand_param!(KeyExpr)
///             .variant(prebindgen::fun!(keyexpr_new_try_from))
///             .variant_self(),
///     )
///     .expand(prebindgen::expand_return!(KeyExpr).field(prebindgen::fun!(keyexpr_get_str)));
/// ```
#[derive(Clone)]
pub struct JniGen {
    /// Single source of truth for the JVM/Kotlin namespace this binding
    /// targets, dot-separated (e.g. `io.zenoh.jni`). Empty = no prefix.
    /// Every derived form — slash-separated for `FindClass`
    /// (`JniGen::java_class_prefix()`), `_`-mangled for JNI extern idents
    /// (`JniGen::jni_class_path()`), dot-separated for Kotlin `package`
    /// declarations — is computed from this at the point of use.
    /// `pub(crate)`: consumers go through [`JniGen::set_package_prefix`],
    /// whose trimming a direct field write would bypass.
    pub(crate) package: String,

    /// Mangler for top-level package function names. Receives the destination
    /// package and camelCase Rust function name; default = identity.
    pub(crate) fun_name_mangle: Option<NameMangle>,
    /// Mangler for Kotlin ptr-class names declared via a
    /// `PtrClassDecl`. Default = identity.
    pub(crate) ptr_class_name_mangle: Option<NameMangle>,
    /// Mangler for Kotlin data-class names declared via a
    /// `DataClassDecl`. Default = identity.
    pub(crate) data_class_name_mangle: Option<NameMangle>,
    /// Mangler for `EnumClassDecl`-declared C-like enum class
    /// names. Default = identity.
    pub(crate) enum_name_mangle: Option<NameMangle>,
    /// Method-name mangle hook ([`JniGen::set_method_name_mangle`]) — applied
    /// to the camelCase Rust function name of every class method/factory
    /// without a per-method `.name()`, with package and class context.
    pub(crate) method_name_mangle: Option<MethodNameMangle>,
    /// Mangler for the framework `JNINative` harness class name. Receives its
    /// default class name; unset = identity.
    pub(crate) harness_name_mangle: Option<HarnessNameMangle>,
    /// Mangler turning a class name into its generated `.interface()` name.
    /// Receives the target package and final class name; identity is forbidden
    /// (a class and its interface can't share a name). Default when unset =
    /// append `"Api"`.
    pub(crate) interface_name_mangle: Option<NameMangle>,

    /// Structured per-type configuration keyed by canonical Rust type.
    /// One entry per `Rust type ↔ JNI/Kotlin` rule; populated when accepting
    /// a `ClassDecl`. Holds opaque-handle
    /// config, enum config, and the raw [`NameSpec`] (Kotlin FQNs are
    /// derived from it on read via [`JniGen::kotlin_fqn`] /
    /// [`JniGen::fqn_of`]); the converter bodies themselves live in
    /// [`Self::input_wrappers`] / [`Self::output_wrappers`]. The rank-0
    /// dispatch order is opaque → enum → wrapper-table → primitive → struct.
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

    /// Canonical single-value conversions ([`ConvertDecl`], accepted by
    /// [`JniGen::convert`]), stored raw — the rank-0 converter bodies derive
    /// from the conversion fns' registry signatures at lookup time
    /// ([`JniGen::convert_input_body`] / [`JniGen::convert_output_body`]),
    /// keeping declarations order-independent and origin-qualified.
    pub(crate) convert_decls: Vec<ConvertDecl>,

    /// When `true` (default), generated wrappers wrap each call that
    /// touches an opaque handle in the per-call `withSortedHandleLocks`
    /// scaffold (deadlock-safe N-ary monitor acquisition + atomic
    /// consume). When `false`, the scaffold is omitted — wrappers emit
    /// only the raw `ptr` read + closed-handle null-check + native call.
    /// Toggled via [`JniGen::set_emit_handle_locks`].
    pub(crate) emit_handle_locks: bool,

    /// Optional Kotlin statement(s) to place inside an `init { … }` block of
    /// the generated centralized externs object (`JNINative`). Set via
    /// [`JniGen::set_jni_native_init`]. Every generated native call routes
    /// through that object, so its `<clinit>` is the single point at which a
    /// consumer can trigger native-library loading (e.g.
    /// `"io.zenoh.jni.NativeLibrary.ensureLoaded()"`). `None` (default) emits no
    /// init block — loading stays the consumer's responsibility.
    pub(crate) jni_native_init: Option<String>,

    /// Type-level default input boundaries ([`ExpandParamDecl`], accepted by
    /// [`JniGen::expand`]), stored raw — merged into the expansion set
    /// at the point of use so declarations stay order-independent.
    pub(crate) param_expand_decls: Vec<ExpandParamDecl>,

    /// Type-level default output boundaries ([`ExpandReturnDecl`], accepted
    /// by [`JniGen::expand`]), stored raw — field names (member
    /// inheritance) resolve at the point of use so declarations stay
    /// order-independent.
    pub(crate) return_expand_decls: Vec<ExpandReturnDecl>,

    /// Per-fn input overrides ([`FunctionDecl::expand_param`]): the fn ident,
    /// the parameter name, and the decl — stored raw like the type-level
    /// decls; cross-checked and lowered in `core/expand.rs`'s `apply`.
    pub(crate) fn_param_expands: Vec<(syn::Ident, String, ExpandParamDecl)>,

    /// Per-fn output overrides ([`FunctionDecl::expand_return`]): the fn
    /// ident and the decl — stored raw; cross-checked and lowered in
    /// `core/unfold.rs`'s `apply`.
    pub(crate) fn_return_expands: Vec<(syn::Ident, ExpandReturnDecl)>,

    /// Per-fn split requests ([`FunctionDecl::split_on_param`]): the fn ident
    /// and the parameter name whose variants get idiomatic typed overloads
    /// (#52). Consumed by `overloads::render_param_overloads`.
    pub(crate) fn_split_params: Vec<(syn::Ident, String)>,

    /// Class members (funs / constructors) attached to a declared class via
    /// its decl's `.method()`/`.constructor()`, keyed by the class's canonical
    /// Rust type. Supplies the instance-method / companion-factory emission
    /// and the receiver-skip set for input-flattening (see [`ClassMember`]).
    /// Insertion order within a class is preserved (the Vec); class emission
    /// iterates `types` by sorted key, so map order is irrelevant.
    pub(crate) class_members: HashMap<TypeKey, Vec<ClassMember>>,

    /// `#[prebindgen]` fns the binding deliberately does NOT wrap, declared
    /// via [`JniGen::ignore`]. Backs [`Prebindgen::ignored_functions`]:
    /// suppresses the registry's per-item "skipping undeclared" warning
    /// without emitting anything.
    pub(crate) ignored_fns: std::collections::HashSet<syn::Ident>,

    /// Bulk name-family ignore predicates, declared via [`JniGen::ignore`] +
    /// [`matching`](crate::lang::matching). Backs
    /// [`Prebindgen::ignored_name_predicates`]: every undeclared item
    /// (fn/type/const) whose name matches is an acknowledged skip.
    pub(crate) ignored_name_predicates: Vec<crate::api::core::prebindgen::NamePredicate>,

    /// `#[prebindgen]` types the binding deliberately does NOT declare,
    /// via [`JniGen::ignore`]. Backs [`Prebindgen::ignored_types`].
    pub(crate) ignored_class_types: std::collections::HashSet<TypeKey>,

    /// `#[prebindgen]` consts the binding deliberately does NOT declare,
    /// via [`JniGen::ignore_const`]. Backs [`Prebindgen::ignored_consts`].
    pub(crate) ignored_const_idents: std::collections::HashSet<syn::Ident>,
    /// Binding-local fns declared via path-built [`fun!`](crate::fun) +
    /// [`FunctionDecl::sig`]: `(fn ident = path last segment, declared path,
    /// stated signature)`. Synthesized into registry entries by the
    /// [`Prebindgen::local_functions`] pre-pass.
    pub(crate) local_fns: Vec<(syn::Ident, syn::Path, syn::Signature)>,
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

mod fn_plan;
mod fold;
mod kotlin_emit;
mod overloads;
mod render;
mod report;
mod struct_plan;
mod symbol;

pub(crate) use builder::*;
pub(crate) use classify::*;
pub(crate) use config::*;
pub use decl::*;
pub(crate) use emit::*;
pub(crate) use fn_plan::*;
pub(crate) use fold::*;
pub(crate) use iface::*;
pub(crate) use overloads::*;
pub(crate) use prim::*;
pub(crate) use render::*;
pub(crate) use struct_plan::*;
