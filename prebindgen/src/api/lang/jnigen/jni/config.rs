//! Global settings of a [`JniGen`] instance — the source module, target
//! package, name-mangling rules, native-init hook and handle-lock toggle.
//!
//! Every setter carries the `set_` prefix: unlike the declaration methods
//! ([`JniGen::package`], [`JniGen::scalar_type_wrapper`], …) which each add
//! one item to the binding surface, a `set_` method changes how *all* other
//! declarations are interpreted. Setters are **order-insensitive**: a
//! setting-derived name (class FQN, extern symbol path) is re-derived from
//! the retained raw declaration inputs whenever a relevant setter runs, so
//! `set_package_prefix` after `.package(...)` yields the same output as
//! before it.

use super::*;

/// Which per-kind mangle hook applies when deriving a declared class's
/// Kotlin name. Value classes reuse the data-class hook.
#[derive(Clone, Copy)]
pub(crate) enum NameKind {
    Ptr,
    Enum,
    DataOrValue,
}

/// Raw naming inputs of one accepted class declaration, retained so the
/// derived Kotlin FQN can be recomputed when a naming-relevant setter
/// (`set_package_prefix`, a class mangle) runs after the declaration.
#[derive(Clone)]
pub(crate) struct DeclaredClassName {
    pub(crate) key: TypeKey,
    pub(crate) subpackage: String,
    pub(crate) short: String,
    /// Per-decl `.name()` — resolved against package + subpackage, bypasses
    /// the mangle hook.
    pub(crate) name_override: Option<String>,
    /// Data/value `kotlin_type` expression — a verbatim Kotlin type,
    /// bypassing package, subpackage and mangle entirely. Wins over
    /// `name_override`.
    pub(crate) explicit_fqn: Option<String>,
    pub(crate) kind: NameKind,
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
        self.refresh_class_names();
        self.recompute_derived();
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
    pub fn set_kotlin_harness_name_mangle<F>(mut self, f: F) -> Self
    where
        F: Fn(&str) -> String + Send + Sync + 'static,
    {
        self.kotlin_harness_name_mangle = Some(Arc::new(f));
        self.recompute_derived();
        self
    }

    /// Set the closure that mangles function names. Called for every scanned
    /// `#[prebindgen]` free function and the synthetic `freePtr` destructor;
    /// receives the camelCased Kotlin-side name and returns the final form
    /// (e.g. `"putPublisher"` → `"putPublisherViaJNI"`). Default = identity.
    pub fn set_kotlin_fun_name_mangle<F>(mut self, f: F) -> Self
    where
        F: Fn(&str) -> String + Send + Sync + 'static,
    {
        self.kotlin_fun_name_mangle = Some(Arc::new(f));
        self
    }

    /// Set the closure that mangles Kotlin ptr-class names declared via a
    /// `PtrClassDecl`. Receives the Rust short name. Default = identity.
    pub fn set_kotlin_ptr_class_name_mangle<F>(mut self, f: F) -> Self
    where
        F: Fn(&str) -> String + Send + Sync + 'static,
    {
        self.kotlin_ptr_class_name_mangle = Some(Arc::new(f));
        self.refresh_class_names();
        self
    }

    /// Set the closure that mangles Kotlin data-class names declared via a
    /// `DataClassDecl` (and value classes, which reuse this hook). Receives
    /// the Rust short name. Default = identity.
    pub fn set_kotlin_data_class_name_mangle<F>(mut self, f: F) -> Self
    where
        F: Fn(&str) -> String + Send + Sync + 'static,
    {
        self.kotlin_data_class_name_mangle = Some(Arc::new(f));
        self.refresh_class_names();
        self
    }

    /// Set the closure that mangles enum-class names declared via an
    /// `EnumClassDecl`. Receives the Rust short name. Default = identity.
    pub fn set_kotlin_enum_name_mangle<F>(mut self, f: F) -> Self
    where
        F: Fn(&str) -> String + Send + Sync + 'static,
    {
        self.kotlin_enum_name_mangle = Some(Arc::new(f));
        self.refresh_class_names();
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
    /// Derive the Kotlin FQN of one declared class from its retained raw
    /// inputs and the *current* settings. Precedence: verbatim
    /// `explicit_fqn` (data/value `kotlin_type`), then per-decl
    /// `name_override` (package-resolved, mangle-bypassed), then the
    /// mangle hook for the class kind.
    pub(crate) fn derive_class_fqn(&self, d: &DeclaredClassName) -> String {
        if let Some(fqn) = &d.explicit_fqn {
            return fqn.clone();
        }
        if let Some(n) = &d.name_override {
            return self.resolve_class_fqn(&d.subpackage, n);
        }
        let mangled = match d.kind {
            NameKind::Ptr => self.mangle_ptr_class(&d.short),
            NameKind::Enum => self.mangle_enum(&d.short),
            NameKind::DataOrValue => self.mangle_data_class(&d.short),
        };
        self.resolve_class_fqn(&d.subpackage, &mangled)
    }

    /// Re-derive every declared class's FQN from the current settings,
    /// updating both the structured [`Self::types`] entry and the flat
    /// [`Self::kotlin_type_fqns`] view. Rows registered by
    /// [`JniGen::scalar_type_wrapper`] carry a verbatim FQN independent of
    /// any setting and are left untouched (they have no
    /// [`DeclaredClassName`]).
    pub(crate) fn refresh_class_names(&mut self) {
        let declared = std::mem::take(&mut self.declared_class_names);
        for d in &declared {
            let fqn = self.derive_class_fqn(d);
            if let Some(entry) = self.types.get_mut(&d.key) {
                entry.kotlin_name = Some(fqn.clone());
            }
            let key_str = d.key.as_str();
            for row in self
                .kotlin_type_fqns
                .iter_mut()
                .filter(|(k, _)| k.as_str() == key_str)
            {
                row.1 = fqn.clone();
            }
        }
        self.declared_class_names = declared;
    }
}
