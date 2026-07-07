//! Global settings for a whole binding — the source module, target package,
//! name-mangling rules and native-init hook — collected into one value and
//! handed to [`JniGen::new`] before any declaration is made.

use super::*;

/// Global configuration for a [`JniGen`] instance: the Rust module path,
/// JVM/Kotlin package, per-kind name-mangling hooks, the `JNINative` static
/// initializer, and the handle-lock scaffold toggle. Build one and hand it to
/// [`JniGen::new`].
pub struct JniGenConfig {
    pub(crate) source_module: syn::Path,
    pub(crate) package_prefix: String,
    pub(crate) jni_native_init: Option<String>,
    pub(crate) kotlin_harness_name_mangle: Option<NameMangle>,
    pub(crate) kotlin_fun_name_mangle: Option<NameMangle>,
    pub(crate) kotlin_ptr_class_name_mangle: Option<NameMangle>,
    pub(crate) kotlin_data_class_name_mangle: Option<NameMangle>,
    pub(crate) kotlin_enum_name_mangle: Option<NameMangle>,
    pub(crate) emit_handle_locks: bool,
}

impl JniGenConfig {
    /// Defaults: `source_module = crate`, empty base package, no
    /// `JNINative` init block, identity name-mangling, handle locks enabled.
    pub fn new() -> Self {
        Self {
            source_module: syn::parse_str("crate").unwrap(),
            package_prefix: String::new(),
            jni_native_init: None,
            kotlin_harness_name_mangle: None,
            kotlin_fun_name_mangle: None,
            kotlin_ptr_class_name_mangle: None,
            kotlin_data_class_name_mangle: None,
            kotlin_enum_name_mangle: None,
            emit_handle_locks: true,
        }
    }

    /// Set the Rust module path that contains the original `#[prebindgen]`
    /// items. Generated Rust wrappers call functions as
    /// `<source_module>::<function>(...)`; defaults to `crate`.
    pub fn source_module(mut self, p: syn::Path) -> Self {
        self.source_module = p;
        self
    }

    /// Set the JVM/Kotlin **base** package (dot-separated, e.g.
    /// `"io.zenoh.jni"`). All derived forms (slash-separated `FindClass`
    /// paths, `_`-mangled JNI extern idents, Kotlin `package` declarations)
    /// are computed from this. Empty = no prefix.
    pub fn package_prefix(mut self, p: impl Into<String>) -> Self {
        self.package_prefix = p.into().trim_matches('.').trim_matches('/').to_string();
        self
    }

    /// Emit `code` inside an `init { … }` block of the generated centralized
    /// externs object (`JNINative`). Because every generated native call
    /// routes through that object, its static initializer is the single
    /// point at which the consumer can trigger native-library loading
    /// transparently — e.g.
    /// `.jni_native_init("io.zenoh.jni.NativeLibrary.ensureLoaded()")` so any
    /// call into the generated bindings loads the library first. The
    /// referenced loader is the consumer's own (hand-written) code; this
    /// keeps the generator free of any concrete loading logic. Unset = no
    /// init block.
    pub fn jni_native_init(mut self, code: impl Into<String>) -> Self {
        self.jni_native_init = Some(code.into());
        self
    }

    /// Set the closure that mangles the framework "harness" class name
    /// `"Native"` (the centralized extern holder). Default = prepend
    /// `"JNI"` (yielding `JNINative`). Affects the generated Kotlin class
    /// name and the derived JNI extern symbol path on the Rust side.
    pub fn kotlin_harness_name_mangle<F>(mut self, f: F) -> Self
    where
        F: Fn(&str) -> String + Send + Sync + 'static,
    {
        self.kotlin_harness_name_mangle = Some(Arc::new(f));
        self
    }

    /// Set the closure that mangles function names. Called for every scanned
    /// `#[prebindgen]` free function and the synthetic `freePtr` destructor;
    /// receives the camelCased Kotlin-side name and returns the final form
    /// (e.g. `"putPublisher"` → `"putPublisherViaJNI"`). Default = identity.
    pub fn kotlin_fun_name_mangle<F>(mut self, f: F) -> Self
    where
        F: Fn(&str) -> String + Send + Sync + 'static,
    {
        self.kotlin_fun_name_mangle = Some(Arc::new(f));
        self
    }

    /// Set the closure that mangles Kotlin ptr-class names declared via a
    /// `PtrClassDecl`. Receives the Rust short name. Default = identity.
    pub fn kotlin_ptr_class_name_mangle<F>(mut self, f: F) -> Self
    where
        F: Fn(&str) -> String + Send + Sync + 'static,
    {
        self.kotlin_ptr_class_name_mangle = Some(Arc::new(f));
        self
    }

    /// Set the closure that mangles Kotlin data-class names declared via a
    /// `DataClassDecl`. Receives the Rust short name. Default = identity.
    pub fn kotlin_data_class_name_mangle<F>(mut self, f: F) -> Self
    where
        F: Fn(&str) -> String + Send + Sync + 'static,
    {
        self.kotlin_data_class_name_mangle = Some(Arc::new(f));
        self
    }

    /// Set the closure that mangles enum-class names declared via an
    /// `EnumClassDecl`. Receives the Rust short name. Default = identity.
    pub fn kotlin_enum_name_mangle<F>(mut self, f: F) -> Self
    where
        F: Fn(&str) -> String + Send + Sync + 'static,
    {
        self.kotlin_enum_name_mangle = Some(Arc::new(f));
        self
    }

    /// Turn off the per-call locking the generator normally wraps around
    /// every handle a wrapper touches (which guards against a handle being
    /// `close()`d on another thread mid-call). Leave it on unless you know
    /// your handles are never used concurrently and want the overhead gone.
    pub fn disable_handle_locks(mut self) -> Self {
        self.emit_handle_locks = false;
        self
    }
}

impl Default for JniGenConfig {
    fn default() -> Self {
        Self::new()
    }
}
