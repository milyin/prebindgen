//! Global settings of a [`JniGen`] instance — the source module, target
//! package, name-mangling rules, native-init hook and handle-lock toggle.
//!
//! Every setter carries the `set_` prefix: unlike the declaration methods
//! ([`JniGen::package`], [`JniGen::scalar_type_wrapper`], …) which each add
//! one item to the binding surface, a `set_` method changes how *all* other
//! declarations are interpreted. Setters are **order-independent by
//! construction**: the builder stores only raw inputs — settings here,
//! declaration name specs ([`NameSpec`]) in the type table — and every
//! setting-derived value (class FQN, `FindClass` path, JNI extern symbol
//! path) is computed at the point of use, so there is no stored derived
//! state that could go stale.

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

impl JniGen {
    /// Set the Rust module path that contains the original `#[prebindgen]`
    /// items. Generated Rust wrappers call functions as
    /// `<source_module>::<function>(...)`; defaults to `crate`.
    pub fn set_source_module(mut self, p: syn::Path) -> Self {
        self.source_module = p;
        self
    }

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

    /// Set the closure that mangles the framework "harness" class name
    /// `"Native"` (the centralized extern holder). Default = prepend
    /// `"JNI"` (yielding `JNINative`). Affects the generated Kotlin class
    /// name and the derived JNI extern symbol path on the Rust side.
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
    /// (e.g. `"putPublisher"` → `"putPublisherViaJNI"`). Default = identity.
    pub fn set_fun_name_mangle<F>(mut self, f: F) -> Self
    where
        F: Fn(&str) -> String + Send + Sync + 'static,
    {
        self.fun_name_mangle = Some(Arc::new(f));
        self
    }

    /// Set the closure that mangles Kotlin ptr-class names declared via a
    /// `PtrClassDecl`. Receives the Rust short name. Default = identity.
    pub fn set_ptr_class_name_mangle<F>(mut self, f: F) -> Self
    where
        F: Fn(&str) -> String + Send + Sync + 'static,
    {
        self.ptr_class_name_mangle = Some(Arc::new(f));
        self
    }

    /// Set the closure that mangles Kotlin data-class names declared via a
    /// `DataClassDecl` (and value classes, which reuse this hook). Receives
    /// the Rust short name. Default = identity.
    pub fn set_data_class_name_mangle<F>(mut self, f: F) -> Self
    where
        F: Fn(&str) -> String + Send + Sync + 'static,
    {
        self.data_class_name_mangle = Some(Arc::new(f));
        self
    }

    /// Set the closure that mangles enum-class names declared via an
    /// `EnumClassDecl`. Receives the Rust short name. Default = identity.
    pub fn set_enum_name_mangle<F>(mut self, f: F) -> Self
    where
        F: Fn(&str) -> String + Send + Sync + 'static,
    {
        self.enum_name_mangle = Some(Arc::new(f));
        self
    }

    /// Toggle the per-call locking the generator wraps around every handle a
    /// wrapper touches (which guards against a handle being `close()`d on
    /// another thread mid-call). Defaults to `true`; pass `false` only if
    /// you know your handles are never used concurrently and want the
    /// overhead gone.
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
    /// mangle_harness("Native")` — the JNI extern symbol path of the
    /// centralized Native object every emitted wrapper hangs off.
    pub(crate) fn jni_class_path(&self) -> String {
        let native_class = self.mangle_harness("Native");
        if self.package.is_empty() {
            format!("Java_{}", native_class)
        } else {
            format!("Java_{}_{}", self.package.replace('.', "_"), native_class)
        }
    }
}
