//! Global settings of a [`JniGen`] instance — the target
//! package, name-mangling rules, native-init hook and handle-lock toggle.
//!
//! Every setter carries the `set_` prefix: unlike the declaration methods
//! ([`JniGen::package`], [`JniGen::convert`], …) which each add
//! one item to the binding surface, a `set_` method changes how *all* other
//! declarations are interpreted. Setters are **order-independent by
//! construction**: the builder stores only raw inputs — settings here,
//! declaration name specs ([`NameSpec`]) in the type table — and every
//! setting-derived value (class FQN, `FindClass` path, JNI extern symbol
//! path) is computed at the point of use, so there is no stored derived
//! state that could go stale.
//!
//! # Name-mangle hooks
//!
//! Placement-aware hooks receive the fully-qualified Kotlin **package where
//! the named object is emitted**, followed by the derived default name for its
//! tier. The method hook additionally receives the final containing class
//! short name. The harness is the exception: it has one fixed placement and
//! receives only its default short name. Hooks return the final short name;
//! unset hooks are identity.
//!
//! | hook | names | input (the derived default) | default |
//! |---|---|---|---|
//! | [`set_harness_name_mangle`](JniGen::set_harness_name_mangle) | the centralized externs object | `"JNINative"` | identity |
//! | [`set_fun_name_mangle`](JniGen::set_fun_name_mangle) | top-level package functions | package, camelCased Rust fn name (`put_publisher` → `"putPublisher"`) | identity |
//! | [`set_ptr_class_name_mangle`](JniGen::set_ptr_class_name_mangle) | `ptr_class` Kotlin classes | package, Rust type short name (`"KeyExpr"`) | identity |
//! | [`set_data_class_name_mangle`](JniGen::set_data_class_name_mangle) | `data_class` + `value_class` Kotlin classes | package, Rust type short name | identity |
//! | [`set_enum_name_mangle`](JniGen::set_enum_name_mangle) | `enum_class` Kotlin classes | package, Rust type short name | identity |
//! | [`set_method_name_mangle`](JniGen::set_method_name_mangle) | class methods/factories and JNI extern methods | package, final class name, full camelCase Rust fn name | identity |
//!
//! One further hook does NOT follow the identity rule, because its input and
//! output are two names that must differ:
//!
//! | hook | names | input | default |
//! |---|---|---|---|
//! | [`set_interface_name_mangle`](JniGen::set_interface_name_mangle) | the generated `.interface()` interface | package, final **class** name (`"Storage"`) | append `"Api"` (`"StorageApi"`); identity is a hard error |
//!
//! A per-decl `.name()` / `.interface_name()` override is always verbatim and
//! **bypasses** the hooks entirely.

use super::*;

/// Which per-kind mangle hook applies when deriving a declared class's
/// Kotlin name. Value classes reuse the data-class hook.
#[derive(Clone, Copy)]
pub(crate) enum NameKind {
    Ptr,
    Enum,
    DataOrValue,
}

/// Raw naming spec of one declared class type, stored in [`TypeConfig`] as
/// declared and turned into a concrete Kotlin FQN only when read
/// ([`JniGen::fqn_of`]), against whatever the settings are at that moment.
#[derive(Clone)]
pub(crate) struct NameSpec {
    pub(crate) subpackage: String,
    /// `rust_short_name(&key)` — the name part of the mangle hook's input;
    /// `subpackage` supplies its package part.
    pub(crate) short: String,
    /// Per-decl `.name()` — resolved against package + subpackage,
    /// bypasses the mangle hook.
    pub(crate) name_override: Option<String>,
    pub(crate) kind: NameKind,
}

impl JniGen {
    /// Set the JVM/Kotlin **base** package (dot-separated, e.g.
    /// `"io.zenoh.jni"`). All derived forms (slash-separated `FindClass`
    /// paths, `_`-mangled JNI extern idents, Kotlin `package` declarations)
    /// are computed from this. Empty = no prefix.
    pub fn set_package_prefix(mut self, p: impl Into<String>) -> Self {
        self.package = p.into().trim_matches('.').trim_matches('/').to_string();
        self
    }

    /// Emit `code` inside an `init { … }` block of the generated centralized
    /// externs object (`JNINative`). Because every generated native call
    /// routes through that object, its static initializer is the single
    /// point at which the consumer can trigger native-library loading
    /// transparently — e.g.
    /// `.set_jni_native_init("io.zenoh.jni.NativeLibrary.ensureLoaded()")` so
    /// any call into the generated bindings loads the library first. The
    /// referenced loader is the consumer's own (hand-written) code; this
    /// keeps the generator free of any concrete loading logic. Unset = no
    /// init block.
    pub fn set_jni_native_init(mut self, code: impl Into<String>) -> Self {
        self.jni_native_init = Some(code.into());
        self
    }

    /// Set the closure that renames the framework "harness" class (the
    /// centralized extern holder). Receives the derived default `"JNINative"`
    /// and replaces it wholesale (`|_| "MyNative".to_string()`); default = identity (see the
    /// module-level table). Affects the generated Kotlin class name and
    /// the derived JNI extern symbol path on the Rust side.
    pub fn set_harness_name_mangle<F>(mut self, f: F) -> Self
    where
        F: Fn(&str) -> String + Send + Sync + 'static,
    {
        self.harness_name_mangle = Some(Arc::new(f));
        self
    }

    /// Set the closure that mangles top-level package function names. It
    /// receives the fully-qualified target package and camelCased Rust name.
    /// JNI externs are methods of the generated harness and therefore use
    /// [`set_method_name_mangle`](Self::set_method_name_mangle) instead.
    pub fn set_fun_name_mangle<F>(mut self, f: F) -> Self
    where
        F: Fn(&str, &str) -> String + Send + Sync + 'static,
    {
        self.fun_name_mangle = Some(Arc::new(f));
        self
    }

    /// Set the closure that turns a class name into the name of its generated
    /// `.interface()` interface. Receives its package and the **final class
    /// name**, and must return a DIFFERENT name — identity is a hard error (a class and its
    /// interface cannot share a name in one package), the one deviation from
    /// the uniform hook contract in the module-level table. Default when
    /// unset = append `"Api"` (`Storage` → `StorageApi`). A per-decl
    /// [`interface_name`](crate::lang::PtrClassDecl::interface_name) wins over
    /// the hook.
    pub fn set_interface_name_mangle<F>(mut self, f: F) -> Self
    where
        F: Fn(&str, &str) -> String + Send + Sync + 'static,
    {
        self.interface_name_mangle = Some(Arc::new(f));
        self
    }

    /// Set the closure that mangles Kotlin ptr-class names declared via a
    /// `PtrClassDecl`. Receives the target package and Rust short name.
    /// Default = identity (see the module-level table).
    pub fn set_ptr_class_name_mangle<F>(mut self, f: F) -> Self
    where
        F: Fn(&str, &str) -> String + Send + Sync + 'static,
    {
        self.ptr_class_name_mangle = Some(Arc::new(f));
        self
    }

    /// Set the closure that mangles Kotlin data-class names declared via a
    /// `DataClassDecl` (and value classes, which reuse this hook). Receives
    /// the target package and Rust short name. Default = identity (see the
    /// module-level table).
    pub fn set_data_class_name_mangle<F>(mut self, f: F) -> Self
    where
        F: Fn(&str, &str) -> String + Send + Sync + 'static,
    {
        self.data_class_name_mangle = Some(Arc::new(f));
        self
    }

    /// Set the closure that mangles enum-class names declared via an
    /// `EnumClassDecl`. Receives the target package and Rust short name.
    /// Default = identity (see the module-level table).
    pub fn set_enum_name_mangle<F>(mut self, f: F) -> Self
    where
        F: Fn(&str, &str) -> String + Send + Sync + 'static,
    {
        self.enum_name_mangle = Some(Arc::new(f));
        self
    }

    /// Set the closure that mangles methods and companion factories. Receives
    /// the fully-qualified target package, final containing class short name,
    /// and the full camelCase Rust function name. It is also used for methods
    /// of generated infrastructure classes (`JNINative`, handle `freePtr`,
    /// vector helpers). Per-method `.name()` remains verbatim.
    pub fn set_method_name_mangle<F>(mut self, f: F) -> Self
    where
        F: Fn(&str, &str, &str) -> String + Send + Sync + 'static,
    {
        self.method_name_mangle = Some(Arc::new(f));
        self
    }

    /// Toggle the per-call locking the generator wraps around every handle a
    /// wrapper touches (which guards against a handle being `close()`d on
    /// another thread mid-call). Defaults to `true`.
    ///
    /// The locks are **not** a performance trade-off: benchmarks (perftest,
    /// N = 5M per op, JDK 21 / macOS arm64, 2026-07-15) show locks-on vs
    /// locks-off deltas within run-to-run noise on every op that does real
    /// work (±3%, mixed sign); the cheapest op (`put` with no string field,
    /// ~33 ns/call) bounds the whole uncontended lock pair at about one
    /// nanosecond. This setting exists so that claim can be independently
    /// re-verified on your own workload — generate once with `false`,
    /// benchmark both, and keep the default — not as an optimization knob.
    pub fn set_emit_handle_locks(mut self, emit: bool) -> Self {
        self.emit_handle_locks = emit;
        self
    }
}

impl JniGen {
    /// Materialize a [`NameSpec`] into a concrete Kotlin FQN under the
    /// current settings. Precedence for a declared class: per-decl
    /// `name_override` (package-resolved, mangle-bypassed), then the mangle
    /// hook for the class kind + package resolution.
    pub(crate) fn fqn_of(&self, spec: &NameSpec) -> String {
        let NameSpec {
            subpackage,
            short,
            name_override,
            kind,
        } = spec;
        if let Some(n) = name_override {
            return self.resolve_class_fqn(subpackage, n);
        }
        let package = self.package_name(subpackage);
        let mangled = match kind {
            NameKind::Ptr => self.mangle_ptr_class(&package, short),
            NameKind::Enum => self.mangle_enum(&package, short),
            NameKind::DataOrValue => self.mangle_data_class(&package, short),
        };
        self.resolve_class_fqn(subpackage, &mangled)
    }

    /// The registered Kotlin FQN for a canonical Rust type key, derived on
    /// demand from the type's stored [`NameSpec`]. A typed direct lookup —
    /// declaration keys and probe keys share the [`TypeKey`] constructor, so
    /// the former linear string scan is gone (issue #95).
    pub(crate) fn kotlin_fqn(&self, key: &TypeKey) -> Option<String> {
        self.types
            .get(key)
            .and_then(|cfg| cfg.name_spec.as_ref())
            .map(|spec| self.fqn_of(spec))
    }

    /// Derived on demand: `package.replace('.', "/")` — the slash-separated
    /// prefix `FindClass` strings are built from.
    pub(crate) fn java_class_prefix(&self) -> String {
        self.package.replace('.', "/")
    }

    /// Derived on demand: the spec-escaped JNI export symbol
    /// `Java_<package>_<jni_native_class_name()>_<method>` for a method on
    /// the centralized Native object every emitted wrapper hangs off
    /// (see [`super::symbol`], #86).
    pub(crate) fn native_method_symbol(&self, method: &str) -> String {
        super::symbol::native_symbol(&self.package, &self.jni_native_class_name(), method)
    }
}
