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
//! One uniform contract for all six hooks: **the hook receives the derived
//! default name for its tier** — exactly what the generator would use if no
//! hook were set — **and returns the final name; every default is the
//! identity.** The input *domain* differs per tier (a fn name is camelCase,
//! a class name is a Rust short name); the table states each:
//!
//! | hook | names | input (the derived default) | default |
//! |---|---|---|---|
//! | [`set_harness_name_mangle`](JniGen::set_harness_name_mangle) | the centralized externs object | `"JNINative"` | identity |
//! | [`set_fun_name_mangle`](JniGen::set_fun_name_mangle) | JNI extern short names (scanned fns, synthetic `freePtr`) | camelCased Rust fn name (`put_publisher` → `"putPublisher"`) | identity |
//! | [`set_ptr_class_name_mangle`](JniGen::set_ptr_class_name_mangle) | `ptr_class` Kotlin classes | Rust type short name (`"KeyExpr"`) | identity |
//! | [`set_data_class_name_mangle`](JniGen::set_data_class_name_mangle) | `data_class` + `value_class` Kotlin classes | Rust type short name | identity |
//! | [`set_enum_name_mangle`](JniGen::set_enum_name_mangle) | `enum_class` Kotlin classes | Rust type short name | identity |
//! | [`set_member_name_mangle`](JniGen::set_member_name_mangle) | class members without a per-member `.name()` | namespace-stripped camelCase (`storage_len` on `Storage` → `"len"`) | identity |
//!
//! A per-decl `.name()` override is always verbatim and **bypasses** the
//! hooks entirely.

use super::*;

/// Which per-kind mangle hook applies when deriving a declared class's
/// Kotlin name. Value classes reuse the data-class hook.
#[derive(Clone, Copy)]
pub(crate) enum NameKind {
    Ptr,
    Enum,
    DataOrValue,
}

/// Raw naming spec of one type, stored in [`TypeConfig`] as declared and
/// turned into a concrete Kotlin FQN only when read ([`JniGen::fqn_of`]),
/// against whatever the settings are at that moment.
#[derive(Clone)]
pub(crate) enum NameSpec {
    /// Verbatim Kotlin type/FQN — a scalar wrapper's `kotlin_type`, or a
    /// data/value class's `kotlin_type` expression. Settings-independent.
    Verbatim(String),
    /// A declared class whose FQN derives from the current settings.
    Class {
        subpackage: String,
        /// `rust_short_name(&key)` — the mangle hook's input.
        short: String,
        /// Per-decl `.name()` — resolved against package + subpackage,
        /// bypasses the mangle hook.
        name_override: Option<String>,
        kind: NameKind,
    },
}

impl NameSpec {
    /// True for a `kotlin_type`-mapped declaration: the type surfaces as an
    /// EXISTING Kotlin type — emitters generate no file for it (the FQN is
    /// used verbatim at reference sites only).
    pub(crate) fn is_verbatim(&self) -> bool {
        matches!(self, NameSpec::Verbatim(_))
    }
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
    /// centralized extern holder). Receives the derived default
    /// `"JNINative"` and replaces it wholesale
    /// (`|_| "MyNative".to_string()`); default = identity (see the
    /// module-level table). Affects the generated Kotlin class name and
    /// the derived JNI extern symbol path on the Rust side.
    pub fn set_harness_name_mangle<F>(mut self, f: F) -> Self
    where
        F: Fn(&str) -> String + Send + Sync + 'static,
    {
        self.harness_name_mangle = Some(Arc::new(f));
        self
    }

    /// Set the closure that mangles function names. Called for every scanned
    /// `#[prebindgen]` free function and the synthetic `freePtr` destructor;
    /// receives the camelCased Kotlin-side name and returns the final form
    /// (e.g. `"putPublisher"` → `"putPublisherViaJNI"`). Default = identity
    /// (see the module-level table).
    pub fn set_fun_name_mangle<F>(mut self, f: F) -> Self
    where
        F: Fn(&str) -> String + Send + Sync + 'static,
    {
        self.fun_name_mangle = Some(Arc::new(f));
        self
    }

    /// Set the closure that mangles Kotlin ptr-class names declared via a
    /// `PtrClassDecl`. Receives the Rust short name. Default = identity
    /// (see the module-level table).
    pub fn set_ptr_class_name_mangle<F>(mut self, f: F) -> Self
    where
        F: Fn(&str) -> String + Send + Sync + 'static,
    {
        self.ptr_class_name_mangle = Some(Arc::new(f));
        self
    }

    /// Set the closure that mangles Kotlin data-class names declared via a
    /// `DataClassDecl` (and value classes, which reuse this hook). Receives
    /// the Rust short name. Default = identity (see the module-level table).
    pub fn set_data_class_name_mangle<F>(mut self, f: F) -> Self
    where
        F: Fn(&str) -> String + Send + Sync + 'static,
    {
        self.data_class_name_mangle = Some(Arc::new(f));
        self
    }

    /// Set the closure that mangles enum-class names declared via an
    /// `EnumClassDecl`. Receives the Rust short name. Default = identity
    /// (see the module-level table).
    pub fn set_enum_name_mangle<F>(mut self, f: F) -> Self
    where
        F: Fn(&str) -> String + Send + Sync + 'static,
    {
        self.enum_name_mangle = Some(Arc::new(f));
        self
    }

    /// Set the closure that mangles **class member** names (instance
    /// methods and companion factories). Receives the derived camelCase
    /// name AFTER the class-namespace prefix was stripped from the fn
    /// ident (`storage_len` on class `Storage` arrives as `"len"`); runs
    /// only for members without a per-member `.name()` (which is always
    /// verbatim). Default = identity (see the module-level table). The
    /// sixth hook of the name-mangle family, covering the one name tier
    /// the other five don't.
    pub fn set_member_name_mangle<F>(mut self, f: F) -> Self
    where
        F: Fn(&str) -> String + Send + Sync + 'static,
    {
        self.member_name_mangle = Some(Arc::new(f));
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
        match spec {
            NameSpec::Verbatim(fqn) => fqn.clone(),
            NameSpec::Class {
                subpackage,
                short,
                name_override,
                kind,
            } => {
                if let Some(n) = name_override {
                    return self.resolve_class_fqn(subpackage, n);
                }
                let mangled = match kind {
                    NameKind::Ptr => self.mangle_ptr_class(short),
                    NameKind::Enum => self.mangle_enum(short),
                    NameKind::DataOrValue => self.mangle_data_class(short),
                };
                self.resolve_class_fqn(subpackage, &mangled)
            }
        }
    }

    /// The registered Kotlin FQN for a canonical Rust type key, derived on
    /// demand from the type's stored [`NameSpec`].
    pub(crate) fn kotlin_fqn(&self, rust_canon: &str) -> Option<String> {
        self.types
            .iter()
            .find(|(k, _)| k.as_str() == rust_canon)
            .and_then(|(_, cfg)| cfg.name_spec.as_ref())
            .map(|spec| self.fqn_of(spec))
    }

    /// Derived on demand: `package.replace('.', "/")` — the slash-separated
    /// prefix `FindClass` strings are built from.
    pub(crate) fn java_class_prefix(&self) -> String {
        self.package.replace('.', "/")
    }

    /// Derived on demand: `"Java_" + package.replace('.', "_") + "_" +
    /// jni_native_class_name()` — the JNI extern symbol path of the
    /// centralized Native object every emitted wrapper hangs off.
    pub(crate) fn jni_class_path(&self) -> String {
        let native_class = self.jni_native_class_name();
        if self.package.is_empty() {
            format!("Java_{}", native_class)
        } else {
            format!("Java_{}_{}", self.package.replace('.', "_"), native_class)
        }
    }
}
