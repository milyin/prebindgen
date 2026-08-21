//! JNI back-end for the Registry pipeline.
//!
//! [`JniGenBuilder`] implements [`prebindgen_registry::Prebindgen`]
//! (Rust-side conversion bodies) and provides an inherent
//! `JniGenBuilder::write_kotlin` for emitting all Kotlin output
//! (`NativeHandle.kt`, typed-handle classes, `JNIWrappers.kt`).

// The implementation is split across sibling submodules, all sharing this
// `jni` module's namespace via the `pub(crate) use …::*` glob re-exports
// below (each sibling needs only `use super::*;`):
//   * this file — type / metadata definitions (JniGenBuilder, KotlinMeta,
//     Projection, FoldStrategy, the config structs) + the shared imports;
//   * `builder` — the JniGenBuilder builder API;
//   * `trait_impl` — the Prebindgen impl + its converter-selector helpers;
//   * `emit` — Rust-side `extern "C"` wrapper / converter-body emission;
//   * `prim` — JNI primitive (un)boxing tables;
//   * `kotlin_emit` / `render` / `fold` — the Kotlin source emitters.

mod metadata;
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

pub(crate) use kotlin_codegen as kt;
use kotlin_codegen::WriteKotlinError;
pub(crate) use metadata::{FoldStrategy, KotlinMeta, NullableKind, Projection, ProjectionKind};
pub use prebindgen_jni_runtime::{
    box_jboolean, box_jbyte, box_jchar, box_jdouble, box_jfloat, box_jint, box_jlong, box_jshort,
    decode_byte_array, decode_string, encode_byte_array, encode_string, null_byte_array,
    null_string, CachedIfaceMethod, JniBindingError,
};
// Kotlin-emission shared imports (used by `kotlin_emit` / `render` / `fold`).
pub(crate) use prebindgen_registry::{
    flat::Origin,
    types_util::{option_inner_type, vec_inner_type},
    ConverterImpl, Direction, NicheSlot, Niches, Prebindgen, Registry, ScalarValue, Stage, TypeKey,
};
pub(crate) use proc_macro2::{Span, TokenStream};
pub(crate) use quote::{format_ident, quote, ToTokens};

pub(crate) use crate::{
    jni::wire_access::{box_descriptor_for_primitive, box_helper_for_wire, jni_field_access},
    util::snake_to_camel,
};

/// Short name of the generated `@RequiresOptIn` marker annotation class.
///
/// The Kotlin half of the raw-pointer guard: it marks every generated entry
/// point that hands a raw native pointer to, or takes one from, safe Kotlin
/// and cannot be hidden outright — `NativeHandle.peek()` and the `fromParts`
/// factories, both reached from Rust by JNI reflection, so renaming them is
/// not an option. [`JVM_SYNTHETIC`] is the other half, for Java.
///
/// See [`Declarations::unsafe_marker_fqn`] for the qualified form.
pub(crate) const UNSAFE_MARKER: &str = "UnsafeNativeApi";

/// `kotlin.jvm.JvmSynthetic` — the JVM half of the raw-pointer guard.
///
/// `internal` and `@RequiresOptIn` are Kotlin-source constructs and stop at
/// the Kotlin compiler: `internal` members are emitted as **public** JVM
/// methods under a mangled name, and javac neither knows nor enforces an
/// opt-in marker. `@JvmSynthetic` sets `ACC_SYNTHETIC`, which javac skips when
/// resolving a call, so a Java consumer cannot name the member at all.
///
/// It leaves the name and the JVM signature alone, so JNI's own lookup
/// (`GetMethodID` / `GetStaticMethodID`, which ignore the flag) still finds
/// `peek`, `fromParts`, and the `external fun` externs. That is exactly what
/// `internal` could not offer. It is **not** applicable to a constructor
/// (Kotlin rejects the target), which is why a handle's constructor is instead
/// `private` behind a synthetic factory — see [`HANDLE_FACTORY`].
pub(crate) const JVM_SYNTHETIC: &str = "JvmSynthetic";

/// Name of the generated per-handle factory that replaces the raw-pointer
/// constructor: `Storage.fromRawPtr(p)` rather than `Storage(p)`.
///
/// A handle's constructor is `private` — `internal` would still be a public
/// JVM constructor and `@JvmSynthetic` cannot be applied to one, so `new
/// Storage(0xdeadbeefL)` compiled fine from Java. The factory is `internal` +
/// [`JVM_SYNTHETIC`], reachable from generated Kotlin and from nowhere else.
pub(crate) const HANDLE_FACTORY: &str = "fromRawPtr";

/// Spell the construction of a typed handle from a raw pointer expression.
///
/// Every generated site that mints a handle goes through here, so the choice
/// of [`HANDLE_FACTORY`] over a constructor is made once.
pub(crate) fn handle_from_raw(short: &str, raw: &str) -> String {
    format!("{short}.{HANDLE_FACTORY}({raw})")
}

/// An `internal` member of the generated surface, hidden from Java too.
///
/// Always pair the two: `internal` alone leaves a public JVM method behind a
/// mangled name, and `handle.setPtr$mymodule(0xdeadbeefL)` from Java would
/// then repoint a live handle at an address of the caller's choosing, which
/// the next generated call happily frees. See [`JVM_SYNTHETIC`].
pub(crate) fn internal_fun(name: &str) -> kotlin_codegen::KtFun {
    kotlin_codegen::KtFun::new(name)
        .vis(kotlin_codegen::KtVis::Internal)
        .annotation(JVM_SYNTHETIC)
}

/// Use-site targets for [`JVM_SYNTHETIC`] on an `internal` property: a bare
/// annotation would land on the backing field and leave the accessors — the
/// part javac actually resolves — visible.
pub(crate) fn internal_prop(p: kotlin_codegen::KtProperty) -> kotlin_codegen::KtProperty {
    p.vis(kotlin_codegen::KtVis::Internal)
        .annotation(format!("get:{JVM_SYNTHETIC}"))
        .annotation(format!("set:{JVM_SYNTHETIC}"))
}

/// [`internal_prop`] for a `val`, which has no setter to hide.
pub(crate) fn internal_val(p: kotlin_codegen::KtProperty) -> kotlin_codegen::KtProperty {
    p.vis(kotlin_codegen::KtVis::Internal)
        .annotation(format!("get:{JVM_SYNTHETIC}"))
}

// ──────────────────────────────────────────────────────────────────────
// Structured type-conversion configuration
// ──────────────────────────────────────────────────────────────────────

/// Options of a type declared with `ptr_class!` — the payload of
/// [`DeclaredKind::Ptr`], which is what marks the type as an opaque handle.
/// The unified Kotlin emitter writes a typed-handle `.kt` file (and the Rust
/// side its `freePtr` destructor) for every such type.
///
/// The typed-handle Kotlin FQN (e.g. `"io.zenoh.jni.JNISession"`) lives
/// in the surrounding [`TypeConfig::name_spec`] slot — FQN-consumers
/// (typed-handle class emission, `instanceof` dispatch,
/// return-value constructor wrap) read it from there. The
/// value-context Kotlin name for the same type (`"Long"`) is produced
/// independently by the rank-0 opaque handler in [`KotlinMeta`], so
/// the two roles don't collide despite sharing the `TypeConfig`.
#[derive(Clone, Default)]
pub(crate) struct OpaqueConfig {
    /// `ptr_class!(X).gc_managed()`: the typed handle stores its pointer in
    /// a separate atomic cell and registers a `Cleaner` action that frees the
    /// native box if no other release path (close/take/consumption) won the
    /// untagged→tagged CAS ticket first.
    pub gc_managed: bool,
}

/// Options of a type declared with `enum_class!` — the payload of
/// [`DeclaredKind::Enum`], which is what marks a `#[prebindgen]`-scanned
/// `enum` as a Kotlin enum class. There are none yet: the declaration itself
/// carries all the information, so this is the empty slot future enum options
/// go in.
///
/// The rank-0 converter arms emit `jint ↔ Rust enum` bodies (via
/// `TryFrom<i32>` for input and `as jni::sys::jint` for output), and the
/// Kotlin emitter writes an `enum class` file with SCREAMING_SNAKE_CASE
/// variants and a discriminant-keyed `fromInt(...)` companion. The Kotlin FQN
/// lives in the surrounding [`TypeConfig::name_spec`] slot, same as
/// [`OpaqueConfig`].
#[derive(Clone, Default)]
pub(crate) struct EnumConfig {}

/// Per-variant naming of a type declared with `sealed_class!` — the payload
/// of [`DeclaredKind::Sealed`], which is what marks a `#[prebindgen]`
/// **data-carrying** enum as mirrored by a
/// Kotlin `sealed interface`. The tag/leaf-group structure itself is read
/// from the model's [`Variant`](prebindgen_registry::flat::Variant) — its
/// `alternatives` in declaration order, indexed as they are tagged — and only
/// what the declaration adds lives here.
#[derive(Clone, Default)]
pub(crate) struct SumConfig {
    /// Per-variant Kotlin class-name overrides, keyed by the Rust variant
    /// ident. Undeclared variants keep their Rust ident.
    pub variant_names: HashMap<String, String>,
}

/// One registered package-level `.fun(...)` entry. The Rust identifier is captured
/// at build-script time via `syn::parse_quote` (i.e. `pq!(rust_fn_name)`); the
/// optional override sets the Kotlin-side name when the default
/// `snake_to_camel(rust_ident)` derivation isn't what the user wants.
#[derive(Clone, Debug)]
pub struct FunctionEntry {
    /// Rust function ident — must match a `#[prebindgen]`-marked free
    /// function in the registered source module. Looked up by
    /// `registry.flat().function(ident)`.
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

/// Which of the five class declarators a type is registered under, **with
/// that declarator's own options**. A type gets exactly one — this is a sum,
/// not a set of independent markers: two declarators would emit two Kotlin
/// declarations for the same FQN, so the second is a hard error (see
/// [`DeclaredKind::merge`]). Reopening the *same* declarator stays legal —
/// `merge` folds the incoming payload into the stored one.
///
/// Adding a sixth class kind is one variant here plus its emitter: there is
/// no flag to add and no precedence chain to extend, because every consumer
/// reads this one field (via [`TypeConfig`]'s accessors, [`JniGenBuilder::type_kind`],
/// or a direct match).
#[derive(Clone)]
pub(crate) enum DeclaredKind {
    /// `ptr_class!` — an opaque-handle type: jlong wire,
    /// `Box::into_raw`/`Box::from_raw` conventions, instanceof dispatch, and
    /// Kotlin typed-handle class emission.
    Ptr(OpaqueConfig),
    /// `enum_class!` — a `#[prebindgen]` fieldless enum mirrored as a Kotlin
    /// `enum class`: jint wire (input + output via `TryFrom<i32>` / `as
    /// jint`) and a generated `.kt` file.
    Enum(EnumConfig),
    /// `sealed_class!` — a `#[prebindgen]` data-carrying enum mirrored as a
    /// Kotlin `sealed interface`: a tag plus one leaf group per variant.
    Sealed(SumConfig),
    /// `data_class!` — a `#[prebindgen]` struct mirrored as a Kotlin `data
    /// class`, flattened field-by-field at the boundary. The kind with no
    /// options of its own.
    Data,
}

/// What compiling a site needs beyond the model: which rows exist, and which
/// row each site takes.
pub(crate) struct Tables {
    pub(crate) recipes: prebindgen_registry::recipe::Recipes,
    pub(crate) bindings: prebindgen_registry::recipe::Bindings,
}

/// All configuration the structured builder accumulates for one
/// canonical Rust type key. The declared kind is mandatory (an entry exists
/// only because some declarator created it); the remaining, cross-kind
/// options default to unset and are populated by the decl that carries them.
#[derive(Clone)]
pub(crate) struct TypeConfig {
    /// The class declarator this type is registered under, carrying that
    /// kind's own options. Every entry in [`JniGenBuilder::types`] has one: entries
    /// are created only by a class declarator (see
    /// `JniGenBuilder::register_class`), which is why presence in the table *is*
    /// "declared as a class" — declared classes are required in **both**
    /// directions at scan (their converters always resolve both ways),
    /// unlike a wrapper registration, which is required per **usage**
    /// direction.
    pub kind: DeclaredKind,
    /// The type this declaration was **written with**, e.g. the `Foo` in
    /// `ptr_class!(Foo)`.
    ///
    /// A class declarator receives a real `syn::Type` and used to keep only the
    /// key derived from it, so every later site that needed the tokens back had
    /// to ask the key for them. That is the wrong direction: the declaration is
    /// where the type came from, and this is where it stays (#291).
    pub rust_type: Origin<syn::Type>,
    /// Raw naming spec of the type as declared — verbatim Kotlin type or
    /// settings-derived class name. Required for any type emitted in
    /// Kotlin; the concrete FQN (`Sample` → `"io.zenoh.jni.Sample"`,
    /// `Vec<u8>` → `"ByteArray"`) is materialized only at read time via
    /// [`JniGenBuilder::fqn_of`], which is what makes the `set_*` settings
    /// order-independent w.r.t. declarations.
    pub name_spec: Option<NameSpec>,
    /// Explicit opt-in for a `data_class` to cross Kotlin → Rust as one
    /// `JObject`. Unmarked data classes are required to flatten completely;
    /// this flag is sticky across reopened declarations.
    pub jobject_input: bool,
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

impl TypeConfig {
    /// A freshly declared type: the declarator's kind and naming spec, every
    /// cross-kind option unset. Reopening the same declarator goes through
    /// [`DeclaredKind::merge`] instead.
    pub(crate) fn new(
        kind: DeclaredKind,
        name_spec: NameSpec,
        rust_type: Origin<syn::Type>,
    ) -> Self {
        Self {
            kind,
            rust_type,
            name_spec: Some(name_spec),
            jobject_input: false,
            interface_enabled: false,
            interface_name_override: None,
            interfaces: Vec::new(),
        }
    }

    /// The opaque-handle options if this type is a `ptr_class`, else `None`.
    pub(crate) fn opaque(&self) -> Option<&OpaqueConfig> {
        match &self.kind {
            DeclaredKind::Ptr(o) => Some(o),
            _ => None,
        }
    }

    /// `true` if this type is a `ptr_class`-declared opaque handle.
    pub(crate) fn is_opaque(&self) -> bool {
        matches!(self.kind, DeclaredKind::Ptr(_))
    }

    /// `true` if this type is an `enum_class`-declared Kotlin `enum class`.
    pub(crate) fn is_enum_class(&self) -> bool {
        matches!(self.kind, DeclaredKind::Enum(_))
    }

    /// The sum options if this type is a `sealed_class`, else `None`.
    pub(crate) fn sum(&self) -> Option<&SumConfig> {
        match &self.kind {
            DeclaredKind::Sealed(s) => Some(s),
            _ => None,
        }
    }
}

/// Free-standing functions emitted into a synthetic package-level wrapper
/// object. One entry per `.package(subpackage)` context that
/// received `.fun(...)` calls.
#[derive(Clone, Default)]
pub(crate) struct PackageConfig {
    /// `#[prebindgen]` fns declared as free-standing wrappers under this
    /// subpackage via [`JniGenBuilder::fun`].
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
/// `data_class`) via a declaration's `.method(...)` /
/// `.constructor(...)`. Methods become **instance methods** (receiver
/// dropped→`this`); constructors become **companion factory** methods. Each
/// is also a real `#[prebindgen]` wrapper (Rust extern + `JNINative` extern +
/// JSONL).
#[derive(Clone, Debug)]
pub(crate) struct ClassMember {
    /// Rust function ident (`registry.flat().function(ident)`).
    pub rust_ident: syn::Ident,
    /// Per-member `.name()` override, stored RAW — the effective Kotlin
    /// name is derived at point of use by [`JniGenBuilder::class_method_kotlin_name`]
    /// (override, else the package/class-aware method hook over the full
    /// camelCase ident), keeping `set_method_name_mangle` order-independent. An
    /// `expand_return!` `.field` referencing the same underlying function
    /// inherits the effective name unless it sets its own `.name()`;
    /// `expand_param!` variants reference the fn by ident only.
    pub kotlin_name_override: Option<String>,
    /// Member kind (method / constructor).
    pub kind: MemberKind,
}
/// Closure that transforms a Kotlin short name with the fully-qualified
/// package in which the named object is emitted. Installed via [`JniGenBuilder`]'s
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
/// independently of `JniGenBuilder` itself; there is no fluent typestate cursor.
///
/// ```
/// use prebindgen_jni::JniGenBuilder;
///
/// let jni = JniGenBuilder::new()
///     .set_package_prefix("io.test.jni")
///     .package(
///         prebindgen_jni::package!("keyexpr")
///             .class(prebindgen_jni::ptr_class!(KeyExpr)
///                 .method(prebindgen_registry::fun!(keyexpr_get_str).name("getStr"))
///                 .constructor(prebindgen_registry::fun!(keyexpr_new_try_from).name("tryFrom"))),
///     )
///     // A KeyExpr param accepts EITHER a String (built via tryFrom) OR an
///     // existing handle; a returned KeyExpr decomposes into its string form.
///     .expand(
///         prebindgen_registry::expand_param!(KeyExpr)
///             .variant(prebindgen_registry::fun!(keyexpr_new_try_from))
///             .variant_self(),
///     )
///     .expand(prebindgen_registry::expand_return!(KeyExpr).field(prebindgen_registry::fun!(keyexpr_get_str)));
/// ```
/// A resolved JNI binding: every crossing has a conversion, and the artifacts
/// can be written.
///
/// Built by [`JniGenBuilder::build`]. Read-only — the registry inside it is
/// complete, which is what lets every `write_*` be a pure emission that can run
/// in any order, or not at all.
pub struct JniGen {
    /// What the binding declared. The emitters read it for names, classes and
    /// decompositions.
    ///
    /// A [`Declarations`], not the [`JniGenBuilder`] it came from: the builder's
    /// mutators would otherwise ride into the finished object, and "finished"
    /// would be a comment rather than a type.
    decls: Declarations,
    /// Every crossing this binding needs, each with its conversion.
    registry: prebindgen_registry::Registry,
}

/// One JNI parameter as a signature writes it: name, Kotlin type, the Kotlin
/// expression that fills it, the conversion it crosses through, where Kotlin
/// finds the object to lock and whether that access can be null (both for a
/// nested owned handle only), whether the conversion carries Rust-side stages,
/// and the struct field it fills.
#[cfg(test)]
pub(crate) type NamedWire = (
    String,
    String,
    String,
    Option<String>,
    Option<String>,
    bool,
    bool,
    Option<String>,
);

#[cfg(test)]
impl JniGen {
    /// What one crossing occupies on the wire, named the way a parameter
    /// names it.
    ///
    /// Test support: the composition is what the emitters read, and this is it
    /// in the form the three coordinated sites see — the JNI parameter name,
    /// its Kotlin type, the Kotlin expression that fills it, the conversion it
    /// crosses through, where the lock scaffold finds a nested handle and
    /// whether that can be null, whether the conversion carries Rust-side
    /// stages, and the struct field it fills.
    pub(crate) fn named_wires_for_test(
        &self,
        spelling: &str,
        param: &str,
    ) -> Option<Vec<NamedWire>> {
        Some(
            self.parts_wires_for_test(spelling)?
                .iter()
                .map(|w| {
                    (
                        crate::util::snake_to_camel(&format!(
                            "{param}_{}",
                            w.path.replace('.', "_")
                        )),
                        w.kt_ty.clone(),
                        w.access.render(param),
                        w.conv().map(|c| c.to_string()),
                        w.handle_target
                            .as_ref()
                            .map(|t| crate::jni::compile::reached(param, t)),
                        w.handle_nullable,
                        w.staged(),
                        w.field().map(str::to_string),
                    )
                })
                .collect(),
        )
    }

    /// What a declared type hands out, as the row states it: one line per
    /// value, naming it, the alternative it belongs to, and how the Rust side
    /// reaches it.
    ///
    /// Test support. The same list `synth_sum_leaves` produces, in the form
    /// that compares — a `TypeRef` has no equality and a `syn::Member` prints
    /// differently by variant, so both sides go through the same rendering.
    pub(crate) fn out_lines_for_test(&self, short: &str) -> Option<Vec<String>> {
        use prebindgen_registry::recipe::Assembly;
        let ident = syn::Ident::new(short, proc_macro2::Span::call_site());
        let ty: syn::Type = syn::parse_quote!(#ident);
        let reading = prebindgen_registry::Conversions::reading_of(&self.registry, &ty)?;
        let compiled = self.decls.compiled.borrow();
        let wires = compiled
            .row_fragment(
                &reading.key(),
                Assembly::Deconstruct,
                &crate::jni::rows::parts(),
            )?
            .out_wires
            .clone()?;
        Some(
            wires
                .iter()
                .map(|w| {
                    let from = match &w.from {
                        crate::jni::compile::OutFrom::Tag => "tag".to_string(),
                        crate::jni::compile::OutFrom::Field { path } => path
                            .iter()
                            .map(|p| p.to_string())
                            .collect::<Vec<_>>()
                            .join("."),
                        crate::jni::compile::OutFrom::Payload { variant, member } => format!(
                            "{}.{}",
                            variant
                                .as_ref()
                                .map(|v| v.to_string())
                                .unwrap_or_else(|| "?".to_string()),
                            crate::jni::struct_plan::sum_field_prop_name(member)
                        ),
                    };
                    format!("{}: {} <- {from} @{:?}", w.name, w.out_ty.key(), w.group)
                })
                .collect(),
        )
    }

    /// The same list, taken from the leaf synthesis the row is meant to
    /// replace — `synth_sum_leaves` for a `sealed_class`,
    /// `synth_value_struct_leaves` for a `data_class`.
    pub(crate) fn walk_lines_for_test(&self, short: &str) -> Option<Vec<String>> {
        use prebindgen_registry::unfold::{LeafSource, PathStep};
        let ident = syn::Ident::new(short, proc_macro2::Span::call_site());
        let leaves = match self.registry.flat().declared_type(&ident)? {
            prebindgen_registry::flat::Type::Variant(sum) => {
                crate::jni::synth_sum_leaves(&self.decls, &self.registry, &ident, sum)
            }
            prebindgen_registry::flat::Type::Struct(s) => {
                crate::jni::synth_value_struct_leaves(&self.decls, &self.registry, s, &[], "", 0)?
            }
            _ => return None,
        };
        Some(
            leaves
                .iter()
                .map(|l| {
                    let from = match &l.source {
                        LeafSource::SumTag => "tag".to_string(),
                        LeafSource::VariantField { variant, member } => format!(
                            "{variant}.{}",
                            crate::jni::struct_plan::sum_field_prop_name(member)
                        ),
                        LeafSource::Field => l
                            .path
                            .iter()
                            .map(|step| match step {
                                PathStep::Field { ident, .. } => ident.to_string(),
                                other => format!("{other:?}"),
                            })
                            .collect::<Vec<_>>()
                            .join("."),
                        other => format!("{other:?}"),
                    };
                    format!("{}: {} <- {from} @{:?}", l.name, l.out_ty.key(), l.group)
                })
                .collect(),
        )
    }

    /// The leaves the registry's own expansion plans for one function, in the
    /// form [`Self::out_lines_for_test`] renders a row in.
    ///
    /// The walk side of a value form: its leaves come from `unfold::flatten`
    /// rather than from a JniGen-side synthesis, so the comparison goes through
    /// a plan rather than through a leaf list.
    pub(crate) fn plan_lines_for_test(&self, func: &str) -> Option<Vec<String>> {
        use prebindgen_registry::unfold::PathStep;
        let ident = syn::Ident::new(func, proc_macro2::Span::call_site());
        let plan = prebindgen_registry::Conversions::unfold_plans(&self.registry).get(&ident)?;
        Some(
            plan.leaves
                .iter()
                .map(|l| {
                    // The accessor call every leaf hangs off is the site's, not
                    // the wire's, so it is dropped to line the two up.
                    let from = l
                        .path
                        .iter()
                        .filter_map(|step| match step {
                            PathStep::Field { ident, .. } => Some(ident.to_string()),
                            PathStep::Call { .. } => None,
                        })
                        .collect::<Vec<_>>()
                        .join(".");
                    format!("{}: {} <- {from} @{:?}", l.name, l.out_ty.key(), l.group)
                })
                .collect(),
        )
    }

    /// The wires a crossing composes into, by spelling.
    pub(crate) fn parts_wires_for_test(
        &self,
        spelling: &str,
    ) -> Option<Vec<crate::jni::compile::Wire>> {
        use prebindgen_registry::recipe::Assembly;
        let ty: syn::Type = syn::parse_str(spelling).ok()?;
        let reading = prebindgen_registry::Conversions::reading_of(&self.registry, &ty)?;
        let key = reading.key();
        let compiled = self.decls.compiled.borrow();
        // A declared class states its composition under `parts`; an optional
        // over one has no row of its own and composes on the row the registry
        // derived, which is the crossing's default.
        compiled
            .row_fragment(&key, Assembly::Construct, &crate::jni::rows::parts())
            .or_else(|| compiled.fragment(&key, Assembly::Construct))?
            .wires
            .clone()
    }
}

// Opaque — exists so `Result<JniGen, _>::expect_err` works in tests.
impl std::fmt::Debug for JniGen {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("JniGen(..)")
    }
}

impl JniGen {
    /// Describe a JNI binding.
    ///
    /// The entry point: everything a binding states — its Kotlin surface, its
    /// decompositions, and where its `#[prebindgen]` source lives — goes on the
    /// builder, and [`JniGenBuilder::build`] turns it into a [`JniGen`].
    pub fn builder() -> JniGenBuilder {
        JniGenBuilder::new()
    }

    /// Write the generated Rust file — the JNI externs and the converters they
    /// call. `out_path` may be relative (resolved against `OUT_DIR`) or
    /// absolute; returns the path actually written.
    pub fn write_rust(
        &self,
        out_path: impl AsRef<std::path::Path>,
    ) -> Result<std::path::PathBuf, prebindgen_registry::WriteRustError> {
        Ok(prebindgen_registry::write::write_rust(
            &self.registry,
            &self.decls,
            &self.decls.compiled_fns,
            out_path,
        )?)
    }

    /// The resolved registry — conversions, decompositions, and the model.
    pub fn registry(&self) -> &prebindgen_registry::Registry {
        &self.registry
    }

    /// What the binding declared.
    pub fn declarations(&self) -> &Declarations {
        &self.decls
    }
}

/// Everything a binding **declared**, once it is done declaring.
///
/// The read-only half of what used to be one `JniGenBuilder`. Splitting it is what
/// keeps the phase separation now that a generator owns its own registry: before
/// [#253](https://github.com/milyin/prebindgen/pull/253) a build script held a
/// `RegistryBuilder` and then a `Registry`, and that type split *was* the
/// enforcement. Once `JniGen::builder().source(..).build()` moved both inside the
/// generator, the built object was left holding the builder — mutators and all.
///
/// So the mutators live on [`JniGenBuilder`] and nothing else, and this type — the
/// one a [`JniGen`] keeps and every emitter reads — **has no `&mut self` method at
/// all**. Not "the obvious ones were removed": none, which
/// `a_built_jnigen_exposes_no_mutation` checks by reading the source, because a
/// one-line grep once missed a multi-line signature and let `Registry::supply`
/// survive two commits that claimed it was gone.
#[derive(Clone)]
pub struct Declarations {
    /// Single source of truth for the JVM/Kotlin namespace this binding
    /// targets, dot-separated (e.g. `io.zenoh.jni`). Empty = no prefix.
    /// Every derived form — slash-separated for `FindClass`
    /// ([`Declarations::java_class_prefix`]), `_`-mangled for JNI extern idents,
    /// dot-separated for Kotlin `package` declarations — is computed from this at
    /// the point of use.
    /// `pub(crate)`: consumers go through [`JniGenBuilder::set_package_prefix`],
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
    /// Method-name mangle hook ([`JniGenBuilder::set_method_name_mangle`]) — applied
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
    /// One entry per declared class; populated when accepting a `ClassDecl`,
    /// through the table's single writer `JniGenBuilder::register_class` — so
    /// presence here *is* "declared as a class", and each entry's
    /// [`TypeConfig::kind`] is the one representation of which declarator it
    /// came from. Also holds the raw [`NameSpec`] (Kotlin FQNs are
    /// derived from it on read via [`JniGenBuilder::kotlin_fqn`] /
    /// [`JniGenBuilder::fqn_of`]). Terminal dispatch order is opaque → enum →
    /// `convert!` → primitive → struct; see
    /// [`JniGenBuilder::select_input_type`](crate::JniGenBuilder)'s selector.
    pub(crate) types: HashMap<TypeKey, TypeConfig>,

    /// Free-standing package-level wrappers, keyed by subpackage path
    /// (relative to [`Self::package`], dot-separated; the empty key is the
    /// base package itself). Populated by [`JniGenBuilder::package`], merging into
    /// whatever the named subpackage already holds.
    pub(crate) packages: BTreeMap<String, PackageConfig>,

    /// Canonical single-value conversions ([`ConvertDecl`], accepted by
    /// [`JniGenBuilder::convert`]), stored raw — the rank-0 converter bodies derive
    /// from the conversion fns' registry signatures at lookup time
    /// ([`JniGenBuilder::convert_input_body`] / [`JniGenBuilder::convert_output_body`]),
    /// keeping declarations order-independent and origin-qualified.
    pub(crate) convert_decls: Vec<ConvertDecl>,
    /// Every conversion this binding compiled.
    ///
    /// Filled once by `JniGenBuilder::build_with` and handed to `write_rust`
    /// directly. It is what reaches the generated file, so a fragment no longer
    /// has to be expressible as one `ConverterImpl` to be emitted — only to be
    /// looked up. The writer sorts and de-duplicates by function name, so the
    /// order here decides which of two same-named functions wins and not where
    /// any of them lands.
    pub(crate) compiled_fns: Vec<syn::ItemFn>,
    /// Every conversion this binding has compiled so far, keyed by crossing.
    ///
    /// What the emitters ask instead of the converter table. A table entry
    /// carries one `destination`, the single wire type a conversion produces,
    /// so it can answer only while a crossing occupies one wire value — and the
    /// answer an emitter wants is this adapter's own fragment.
    ///
    /// Shared and interior-mutable because JniGen reads it **while** it is
    /// being filled: unlike `prebindgen-c`, this adapter's emitters are also
    /// its conversion builders, and `JniGen::build_with` calls them from inside
    /// `convert_with`. A conversion for one type is built out of the
    /// conversions for its inners, which the resolver compiles first — the same
    /// order that filled the converter table, so a fragment is there exactly
    /// when a table entry would have been.
    pub(crate) compiled: std::rc::Rc<
        std::cell::RefCell<prebindgen_registry::recipe::Compiled<crate::jni::compile::JFrag>>,
    >,
    /// The row table and the site bindings this binding was built against.
    ///
    /// Kept beside the fragment store for the same reason it is: a plan is
    /// compiled per **site**, and the sites are `fn_plan`'s to enumerate — a
    /// constructor expansion contributes leaves no signature names. So the
    /// compiler has to be resumable after `build_with` has returned, and these
    /// are the two things `Compiler::resume` needs that the model does not
    /// already carry.
    ///
    /// `None` only before the build fills them, which is before anything can
    /// ask.
    pub(crate) tables: Option<std::rc::Rc<Tables>>,

    /// When `true` (default), generated wrappers wrap each call that
    /// touches an opaque handle in the per-call `withSortedHandleLocks`
    /// scaffold (deadlock-safe N-ary monitor acquisition + atomic
    /// consume). When `false`, the scaffold is omitted — wrappers emit
    /// only the raw `ptr` read + closed-handle null-check + native call.
    /// Toggled via [`JniGenBuilder::set_emit_handle_locks`].
    pub(crate) emit_handle_locks: bool,

    /// Optional Kotlin statement(s) to place inside an `init { … }` block of
    /// the generated centralized externs object (`JNINative`). Set via
    /// [`JniGenBuilder::set_jni_native_init`]. Every generated native call routes
    /// through that object, so its `<clinit>` is the single point at which a
    /// consumer can trigger native-library loading (e.g.
    /// `"io.zenoh.jni.NativeLibrary.ensureLoaded()"`). `None` (default) emits no
    /// init block — loading stays the consumer's responsibility.
    pub(crate) jni_native_init: Option<String>,

    /// Type-level default input boundaries ([`ExpandParamDecl`], accepted by
    /// [`JniGenBuilder::expand`]), stored raw — merged into the expansion set
    /// at the point of use so declarations stay order-independent.
    pub(crate) param_expand_decls: Vec<ExpandParamDecl>,

    /// Type-level default output boundaries ([`ExpandReturnDecl`], accepted
    /// by [`JniGenBuilder::expand`]), stored raw — field names (member
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
    /// via [`JniGenBuilder::ignore`]. Backs [`Prebindgen::ignored_functions`]:
    /// suppresses the registry's per-item "skipping undeclared" warning
    /// without emitting anything.
    pub(crate) ignored_fns: std::collections::HashSet<syn::Ident>,

    /// Bulk name-family ignore predicates, declared via [`JniGenBuilder::ignore`] +
    /// [`matching`](crate::matching). Backs
    /// [`Prebindgen::ignored_name_predicates`]: every undeclared item
    /// (fn/type/const) whose name matches is an acknowledged skip.
    pub(crate) ignored_name_predicates: Vec<prebindgen_registry::NamePredicate>,

    /// `#[prebindgen]` types the binding deliberately does NOT declare,
    /// via [`JniGenBuilder::ignore`]. Backs [`Prebindgen::ignored_types`].
    pub(crate) ignored_class_types: std::collections::HashSet<TypeKey>,

    /// `#[prebindgen]` consts the binding deliberately does NOT declare,
    /// via [`JniGenBuilder::ignore_const`]. Backs [`Prebindgen::ignored_consts`].
    pub(crate) ignored_const_idents: std::collections::HashSet<syn::Ident>,
    /// Binding-local fns declared via path-built [`fun!`](prebindgen_registry::fun) +
    /// [`FunctionDecl::sig`]: `(fn ident = path last segment, declared path,
    /// stated signature)`. Synthesized into registry entries by the
    /// [`Prebindgen::local_functions`] pre-pass.
    pub(crate) local_fns: Vec<(syn::Ident, syn::Path, syn::Signature)>,

    /// Memoized callback-interface specs, one per [`SpecKey`] identity —
    /// populated lazily via [`JniGenBuilder::iface_spec`] (first touch may be the
    /// resolve-time trampoline, which runs before any function plan exists)
    /// and shared by every later consumer, so the FQN/descriptor pair cannot
    /// drift between the Rust, Kotlin-wrapper, and interface-declaration
    /// tiers (issue #107). Not a setting: derived state, keyed entirely by
    /// `(self, registry)` — cloning the builder just clones the cache.
    pub(crate) iface_specs:
        std::cell::RefCell<std::collections::BTreeMap<SpecKey, std::sync::Arc<IfaceSpec>>>,

    /// Memoized per-function lowered plans, one [`JniFunctionPlan`] per bound
    /// function/const-getter ident — the single lowering shared by validation
    /// and every emitter (Rust extern, JNINative extern, Kotlin wrapper,
    /// report), so a function's binding is derived ONCE per generation
    /// instead of ~8× (issue #90's deferred "build the plan once and store
    /// it" — `fn_plan.rs`). Populated eagerly at `resolve` by
    /// [`validate_bindings`], which builds every declared function's plan for
    /// the collision tables; the writers then read it. Same interior-mutable
    /// "derived state, keyed by `(self, registry)`" contract as
    /// [`Self::iface_specs`].
    pub(crate) fn_plans: std::cell::RefCell<HashMap<syn::Ident, std::rc::Rc<JniFunctionPlan>>>,
}

/// Describe a JNI binding: state the Kotlin surface, then [`build`](Self::build).
///
/// Holds the [`Declarations`] being filled and the sources to parse, and it is the
/// only type with mutators. `build` consumes it, hands the declarations to the
/// registry, and stores them in a [`JniGen`] — from where nothing can declare
/// anything again, because this type is gone by then.
#[derive(Clone, Default)]
pub struct JniGenBuilder {
    /// What has been declared so far. Moved out whole by [`Self::build`].
    pub(crate) decls: Declarations,

    /// Where the `#[prebindgen]` items come from.
    ///
    /// A [`FlatBuilder`](prebindgen_registry::flat::FlatBuilder), stated with the same
    /// three feeders it has — so a build script says where the source is in the
    /// vocabulary the model already uses, and never names a `Flat` or a
    /// `Registry` itself.
    ///
    /// Not a declaration, which is why it stays here rather than moving across:
    /// it is *input to* building, and keeping it out is what makes
    /// [`Declarations`] mean one thing.
    pub(crate) sources: prebindgen_registry::flat::FlatBuilder,
}

// ── Sibling submodules (carved from the former monolithic file) ─────────
mod builder;
mod classify;
mod compile;
mod config;
mod decl;
mod emit;
mod equality;
mod iface;
mod prim;
mod prim_array;
mod rows;
mod selector;
#[cfg(test)]
mod tests;
pub(crate) mod trait_impl;

mod fn_plan;
mod fold;
mod kotlin_emit;
mod overloads;
mod render;
mod report;
mod struct_plan;
mod symbol;
mod symbols;

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
pub(crate) use symbols::*;
