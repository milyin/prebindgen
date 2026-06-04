//! `KotlinExt` impl for [`JniGen`].
//!
//! [`JniGen::write_kotlin`] is the single entry point for every Kotlin
//! file the JNI back-end emits. Given one `kotlin_root` it writes:
//!   * `NativeHandle.kt` (package `io.zenoh.jni`).
//!   * One typed-handle class per `ptr_class` entry without
//!     `.suppress_kotlin_code()`.
//!   * One package-level wrapper file for `package()` (top-level
//!     safe wrappers for `package_methods` fns).
//!   * `JNINative.kt` — centralized `external fun` holder.
//!   * One Kotlin fun-interface file per `impl Fn(args) + Send + Sync
//!     + 'static` type, named via [`JniGen::kotlin_callback_name_mangle`]
//!     (default = identity over the `"On"`-prefixed auto-derived name;
//!     in zenoh-jni: `JNIOn<Args>`). Callback types overridden via
//!     [`JniGen::callback_input`] are skipped — the override points at
//!     a hand-written interface.
//!
//! Every `#[prebindgen]` function must be assigned a Kotlin home via
//! `.method(...)` on either a typed-handle / data-class / enum config
//! or on `package(...)`. Undeclared functions are skipped (see
//! `Registry::scan_declared` warnings). There is no "orphan" bucket.
//!
//! All emitters route through [`KotlinFile::write`], which translates
//! `package` into a sub-path under `kotlin_root`.

use std::collections::{BTreeSet, HashSet};
use std::path::{Path, PathBuf};

use quote::ToTokens;

use crate::api::core::prebindgen::{IntoSource, IntoSourceMode, Prebindgen};
use crate::api::core::registry::{extract_fn_trait_args, Registry, TypeKey};
use crate::api::lang::jnigen::jni::jni_ext::{
    converter_returns_owned_object, JniGen, KotlinMeta, MethodEntry,
};
use crate::api::lang::jnigen::jni::templates;
use crate::api::lang::jnigen::kotlin::kotlin_ext::{KotlinFile, WriteKotlinError};
use crate::api::lang::jnigen::kotlin::type_map::KotlinTypeMap;

/// Declaration of one auto-generated typed `NativeHandle` subclass.
///
/// Consumed by [`JniGen::write_typed_handles`] (and forwarded to
/// [`JniGen::write_jni_wrappers`] so the same promotion list can carve
/// the matching skip-list). Each entry says "this Kotlin class is the
/// home for the named `#[prebindgen]` functions"; everything else stays
/// in the catch-all `JNIWrappers` object.
#[derive(Clone, Copy)]
pub(crate) struct TypedHandle<'a> {
    /// Short Rust name shown in the class doc comment (e.g. `"Publisher"`).
    /// Pure documentation, doesn't have to match anything in the Registry.
    pub rust_doc: &'a str,
    /// Package-qualified Kotlin class name (e.g.
    /// `"io.zenoh.jni.JNIPublisher"`). The Rust type-key registered for
    /// this FQN via [`JniGen::kotlin_type_fqn`] identifies which
    /// parameter of each promoted function becomes `this`.
    pub kotlin_fqn: &'a str,
    /// `#[prebindgen]` fns declared as **instance methods** via
    /// [`JniGen::method`]. The matched first parameter is dropped from
    /// the Kotlin signature and substituted by inherited `withPtr` /
    /// `consume` scope. Mismatch (no param matches the class type) is a
    /// build-time error.
    pub instance_methods: &'a [MethodEntry],
    /// `#[prebindgen]` fns declared as **companion-object methods** via
    /// [`JniGen::companion_method`]. Rendered inside `companion object`
    /// using the same shape as a package-level wrapper.
    pub companion_methods: &'a [MethodEntry],
}

/// Reverse-lookup the Rust type-key registered for a given Kotlin FQN
/// in [`JniGen::kotlin_type_fqns`]. Used by [`JniGen::write_typed_handles`]
/// to determine which parameter of each promoted function should be
/// dropped (becomes `this`).
fn rust_key_for_fqn<'a>(ext: &'a JniGen, fqn: &str) -> Option<&'a str> {
    ext.kotlin_type_fqns
        .iter()
        .find_map(|(rust, k)| (k == fqn).then_some(rust.as_str()))
}

impl JniGen {
    /// Unified Kotlin emission — single public entry point that fans out
    /// to per-callback fun-interface files, `NativeHandle.kt`, typed-handle
    /// classes (one per `ptr_class` registration), and
    /// `JNIWrappers.kt`. Reads all configuration (typed-handle methods,
    /// callback FQN overrides, Kotlin type names) from internal state set
    /// during the builder phase. Returns every path written.
    pub fn write_kotlin(
        &self,
        registry: &Registry<KotlinMeta>,
        kotlin_root: &Path,
    ) -> Result<Vec<PathBuf>, WriteKotlinError> {
        let mut written = Vec::new();
        written.push(self.write_native_handle(kotlin_root)?);
        written.extend(self.emit_callback_files(registry, kotlin_root)?);
        written.extend(self.write_exception_classes(kotlin_root)?);
        written.extend(self.write_enum_classes(registry, kotlin_root)?);
        written.extend(self.write_data_classes(registry, kotlin_root)?);
        written.extend(self.write_value_blobs(kotlin_root)?);

        // Build the borrowed `TypedHandle<'_>` view from internal config.
        let owned = self.collect_typed_handles();
        let typed_handles: Vec<TypedHandle<'_>> = owned
            .iter()
            .map(|h| TypedHandle {
                rust_doc: &h.rust_doc,
                kotlin_fqn: &h.kotlin_fqn,
                instance_methods: h.instance_methods.as_slice(),
                companion_methods: h.companion_methods.as_slice(),
            })
            .collect();
        let kotlin_types = self.build_kotlin_type_map();
        written.extend(self.write_typed_handles(
            &typed_handles,
            registry,
            &kotlin_types,
            kotlin_root,
        )?);
        for (subpackage, pkg_cfg) in &self.packages {
            if pkg_cfg.functions.is_empty() {
                continue;
            }
            written.push(self.write_jni_package(
                registry,
                &kotlin_types,
                kotlin_root,
                subpackage,
                pkg_cfg,
            )?);
        }
        written.push(self.write_jni_native(registry, &kotlin_types, kotlin_root)?);
        Ok(written)
    }

    /// Emit `NativeHandle.kt` — the shared base class every typed handle
    /// extends, plus the `withSortedHandleLocks` helper that the generated
    /// wrappers use to acquire any number of handle monitors in one
    /// pointer-sorted, deadlock-safe pass.
    pub(crate) fn write_native_handle(
        &self,
        output_dir: &Path,
    ) -> Result<PathBuf, WriteKotlinError> {
        let mut s = String::new();
        s.push_str("// Auto-generated by JniGen — do not edit by hand.\n");
        if !self.package.is_empty() {
            s.push_str(&format!("package {}\n\n", self.package));
        }
        s.push_str(
            "/** Base class for every typed native handle: owns the raw `Box<T>` pointer\n\
            \x20*  slot and its monitor. Subclasses add their type-specific `close()` /\n\
            \x20*  `take()` / `freePtr`. */\n\
            public abstract class NativeHandle(initialPtr: Long) : AutoCloseable {\n\
            \x20   @Volatile internal var ptr: Long = initialPtr\n\
            \x20   public fun peek(): Long = ptr\n\
            \x20   public fun isClosed(): Boolean = ptr == 0L\n\
            }\n\n",
        );
        s.push_str(
            "/** Acquire every handle's monitor in one global order (sorted by raw\n\
            \x20*  pointer) so concurrent calls touching the same handles can't deadlock,\n\
            \x20*  then run [body]. Closed handles (`ptr == 0`) are still locked; callers\n\
            \x20*  re-read and null-check each pointer inside [body]. Scales to any arity. */\n\
            internal fun <R> withSortedHandleLocks(handles: List<NativeHandle>, body: () -> R): R {\n\
            \x20   val sorted = handles.sortedBy { it.ptr }\n\
            \x20   fun rec(i: Int): R = if (i == sorted.size) body() else synchronized(sorted[i]) { rec(i + 1) }\n\
            \x20   return rec(0)\n\
            }\n",
        );
        let file = KotlinFile {
            contents: s,
            package: self.package.clone(),
            class_name: "NativeHandle".to_string(),
        };
        Ok(file.write(output_dir)?)
    }

    /// Emit one `@JvmInline value class <Name>(val bytes: ByteArray)` per
    /// declared `value_blob` type. The class is the typed wrapper level; it is
    /// erased to its `ByteArray` field at the JVM/ABI level, so the `JNINative`
    /// extern (and the wire) stays `ByteArray` while wrappers speak the typed
    /// class. The single field name `bytes` matches `value_projection_field`.
    pub(crate) fn write_value_blobs(
        &self,
        output_dir: &Path,
    ) -> Result<Vec<PathBuf>, WriteKotlinError> {
        let mut written = Vec::new();
        for (key, cfg) in &self.types {
            if !cfg.value_blob {
                continue;
            }
            let fqn = cfg.kotlin_name.clone().ok_or_else(|| {
                WriteKotlinError::Other(format!(
                    "value_blob `{}` has no Kotlin FQN",
                    key.as_str()
                ))
            })?;
            let (package, class_name) = match fqn.rsplit_once('.') {
                Some((p, c)) => (p.to_string(), c.to_string()),
                None => (String::new(), fqn.clone()),
            };
            let mut s = String::new();
            s.push_str("// Auto-generated by JniGen — do not edit by hand.\n");
            if !package.is_empty() {
                s.push_str(&format!("package {}\n\n", package));
            }
            s.push_str(&format!(
                "/** Typed by-value wrapper for the native Rust `{}` (a `Copy` blob carried\n\
                \x20*  as its raw bytes; `@JvmInline`-erased to `ByteArray` at the JNI boundary). */\n\
                @JvmInline\n\
                public value class {}(public val bytes: ByteArray)\n",
                key.as_str(),
                class_name,
            ));
            let file = KotlinFile {
                contents: s,
                package,
                class_name,
            };
            written.push(file.write(output_dir)?);
        }
        Ok(written)
    }

    /// Per-callback fun-interface emission (one `<mangle_callback>.kt`
    /// file per `impl Fn(...)` type encountered in the resolved
    /// registry). Skips writes for `impl Fn(...)` keys whose Kotlin
    /// FQN was overridden via [`Self::callback_input`] — the override
    /// already points at a hand-maintained callback interface, so the
    /// auto-stub would be dead code. Each emitted file is placed
    /// under `kotlin_root/<kotlin_callback_package as path>/`.
    pub(crate) fn emit_callback_files(
        &self,
        registry: &Registry<KotlinMeta>,
        kotlin_root: &Path,
    ) -> Result<Vec<PathBuf>, WriteKotlinError> {
        let mut seen: HashSet<TypeKey> = HashSet::new();
        let mut written = Vec::new();
        for buckets in [&registry.input_types, &registry.output_types] {
            for bucket in buckets.iter() {
                for (key, slot) in bucket {
                    if slot.is_none() {
                        continue;
                    }
                    if !seen.insert(key.clone()) {
                        continue;
                    }
                    let ty = key.to_type();
                    if let Some(args) = extract_fn_trait_args(&ty) {
                        // A `callback_input` registration points the
                        // Kotlin signature at a hand-written interface
                        // — skip the auto-stub.
                        if self
                            .types
                            .get(key)
                            .and_then(|c| c.callback_kotlin_fqn.as_ref())
                            .is_some()
                        {
                            continue;
                        }
                        let file = build_callback_kotlin_file(self, &args, registry);
                        written.push(file.write(kotlin_root)?);
                    }
                }
            }
        }
        Ok(written)
    }

    /// Build the `TypedHandle` slice from internal `types` config.
    /// Iterates entries where `opaque.is_some()` and emits one
    /// `TypedHandle` per opaque-handle registration. Stable order by
    /// canonical Rust type-key — keeps generated output deterministic.
    fn collect_typed_handles(&self) -> Vec<OwnedTypedHandle> {
        let mut handles: Vec<OwnedTypedHandle> = Vec::new();
        let mut keys: Vec<&TypeKey> = self.types.keys().collect();
        keys.sort_by(|a, b| a.as_str().cmp(b.as_str()));
        for key in keys {
            let cfg = &self.types[key];
            let Some(opaque) = &cfg.opaque else { continue };
            if opaque.suppress_kotlin_code {
                continue;
            }
            let Some(kotlin_fqn) = &cfg.kotlin_name else {
                continue;
            };
            // rust_doc — short last-segment of the Rust type key (best
            // effort; only used in the generated doc comment).
            let rust_doc = key
                .as_str()
                .split(|c: char| !c.is_ascii_alphanumeric() && c != '_')
                .find(|s| !s.is_empty())
                .unwrap_or(key.as_str())
                .to_string();
            handles.push(OwnedTypedHandle {
                rust_doc,
                kotlin_fqn: kotlin_fqn.clone(),
                instance_methods: cfg.instance_methods.clone(),
                companion_methods: cfg.companion_methods.clone(),
            });
        }
        handles
    }

    /// Build the `KotlinTypeMap` view consumed by the typed-handle and
    /// JNIWrappers emitters. Combines callback FQNs from
    /// [`Self::collect_kotlin_callback_fqns`] (auto-derived or
    /// override) with `kotlin_name` entries from the structured config.
    /// Structured-config entries win on conflict.
    fn build_kotlin_type_map(&self) -> KotlinTypeMap {
        let mut map = KotlinTypeMap::new().with_primitive_builtins();
        for (key, cfg) in &self.types {
            if let Some(name) = &cfg.kotlin_name {
                map = map.add(key.as_str(), name.clone());
            }
        }
        map
    }
}

/// Owned counterpart of [`TypedHandle`] — used internally so the
/// `collect_typed_handles` helper doesn't have to hand out borrows of
/// `self.types`.
pub(crate) struct OwnedTypedHandle {
    pub rust_doc: String,
    pub kotlin_fqn: String,
    pub instance_methods: Vec<MethodEntry>,
    pub companion_methods: Vec<MethodEntry>,
}

impl JniGen {
    /// Emit one Kotlin file per registered
    /// throwable class (via [`crate::api::lang::jnigen::jni::JniGen::throwable`]) — each becomes a
    /// `public class <Name>(message: String? = null) : Exception()`
    /// landing under `<package>/<Name>.kt`. Iterates `self.exceptions`
    /// in declaration order; returns every path written.
    pub(crate) fn write_exception_classes(
        &self,
        output_dir: &Path,
    ) -> Result<Vec<PathBuf>, WriteKotlinError> {
        let mut written = Vec::new();
        for exc in &self.exceptions {
            // Skip exceptions whose Rust type already has a data-class (or
            // ptr/enum) Kotlin emission — those classes carry the `: Exception`
            // extension themselves (via `cfg.throwable` in
            // `render_data_class_source`). The stub-template path only runs
            // for un-registered exception types — in practice that's the
            // framework's `JniBindingError`, declared inside `JniGen::new`
            // without going through `.throwable()`.
            let key = TypeKey::from_type(&exc.rust_type);
            if self
                .types
                .get(&key)
                .map(|cfg| cfg.kotlin_name.is_some())
                .unwrap_or(false)
            {
                continue;
            }
            let (package, class_name) = match exc.kotlin_fqn.rsplit_once('.') {
                Some((p, c)) => (p.to_string(), c.to_string()),
                None => (String::new(), exc.kotlin_fqn.clone()),
            };
            let file = templates::exception::emit_exception(&package, &class_name, &exc.rust_short);
            written.push(file.write(output_dir)?);
        }
        Ok(written)
    }

    /// Emit one Kotlin `enum class` file per `enum_class`-declared type
    /// (skipping any flagged with `.suppress_kotlin_code()`). Variants
    /// render in declaration order using SCREAMING_SNAKE_CASE names; the
    /// constructor stores the Rust discriminant value (or the ordinal as
    /// a fallback when the discriminant isn't a bare integer literal).
    /// A `fromInt(value: Int)` companion mirrors the `Priority.fromInt`
    /// shape that hand-written enums use today, so adapter code stays
    /// uniform.
    pub(crate) fn write_enum_classes(
        &self,
        registry: &Registry<KotlinMeta>,
        output_dir: &Path,
    ) -> Result<Vec<PathBuf>, WriteKotlinError> {
        let mut written = Vec::new();
        let callback_fqns = self.collect_kotlin_callback_fqns(registry);
        let mut kotlin_types = KotlinTypeMap::new();
        for (k, v) in callback_fqns.iter() {
            kotlin_types = kotlin_types.add(k, v.clone());
        }
        let configured_types = self.build_kotlin_type_map();
        for (k, v) in configured_types.iter() {
            kotlin_types = kotlin_types.add(k, v.clone());
        }
        // Deterministic order by canonical Rust type-key.
        let mut keys: Vec<&TypeKey> = self.types.keys().collect();
        keys.sort_by(|a, b| a.as_str().cmp(b.as_str()));
        for key in keys {
            let cfg = &self.types[key];
            let Some(enum_cfg) = &cfg.enum_cfg else {
                continue;
            };
            if enum_cfg.suppress_kotlin_code {
                continue;
            }
            let Some(kotlin_fqn) = &cfg.kotlin_name else {
                continue;
            };
            // Look up the syn::ItemEnum by the type-key's bare ident.
            let ty = key.to_type();
            let Some(ident) = (if let syn::Type::Path(tp) = &ty {
                tp.path.segments.last().map(|s| s.ident.clone())
            } else {
                None
            }) else {
                continue;
            };
            let Some((item_enum, _)) = registry.enums.get(&ident) else {
                continue;
            };
            let (package, class_name) = match kotlin_fqn.rsplit_once('.') {
                Some((p, c)) => (p.to_string(), c.to_string()),
                None => (String::new(), kotlin_fqn.clone()),
            };
            let file = KotlinFile {
                contents: render_enum_source(
                    self,
                    &package,
                    &class_name,
                    item_enum,
                    &cfg.instance_methods,
                    &cfg.companion_methods,
                    registry,
                    &kotlin_types,
                ),
                package,
                class_name,
            };
            written.push(file.write(output_dir)?);
        }
        Ok(written)
    }

    /// Emit one Kotlin `data class` file per `data_class`-declared
    /// struct. Uses resolved converter metadata to derive Kotlin field
    /// types, so wrappers and data-class declarations stay in sync.
    pub(crate) fn write_data_classes(
        &self,
        registry: &Registry<KotlinMeta>,
        output_dir: &Path,
    ) -> Result<Vec<PathBuf>, WriteKotlinError> {
        let mut written = Vec::new();
        let callback_fqns = self.collect_kotlin_callback_fqns(registry);
        let mut kotlin_types = KotlinTypeMap::new();
        for (k, v) in callback_fqns.iter() {
            kotlin_types = kotlin_types.add(k, v.clone());
        }
        let configured_types = self.build_kotlin_type_map();
        for (k, v) in configured_types.iter() {
            kotlin_types = kotlin_types.add(k, v.clone());
        }
        let mut rust_names: Vec<String> = Vec::new();
        let mut aliases: Vec<(String, String)> = Vec::new();
        let mut keys: Vec<&TypeKey> = self.types.keys().collect();
        keys.sort_by(|a, b| a.as_str().cmp(b.as_str()));

        for key in keys {
            let cfg = &self.types[key];
            if cfg.opaque.is_some() || cfg.enum_cfg.is_some() || cfg.callback_kotlin_fqn.is_some() {
                continue;
            }
            let Some(kotlin_fqn) = &cfg.kotlin_name else {
                continue;
            };

            let ty = key.to_type();
            let Some(ident) = (if let syn::Type::Path(tp) = &ty {
                tp.path.segments.last().map(|s| s.ident.clone())
            } else {
                None
            }) else {
                continue;
            };
            let Some((item_struct, _)) = registry.structs.get(&ident) else {
                continue;
            };
            rust_names.push(item_struct.ident.to_string());

            let (package, class_name) = match kotlin_fqn.rsplit_once('.') {
                Some((p, c)) => (p.to_string(), c.to_string()),
                None => (String::new(), kotlin_fqn.clone()),
            };
            if item_struct.ident.to_string() != class_name {
                aliases.push((item_struct.ident.to_string(), class_name.clone()));
            }
            let file = KotlinFile {
                contents: render_data_class_source(
                    self,
                    &package,
                    &class_name,
                    item_struct,
                    registry,
                    &kotlin_types,
                    &cfg.instance_methods,
                    &cfg.companion_methods,
                    cfg.throwable,
                    cfg.value_class,
                    key.as_str(),
                ),
                package: package.clone(),
                class_name,
            };
            written.push(file.write(output_dir)?);

            // If data-class naming changed, remove stale legacy file that
            // may have been generated under the old class name.
            let legacy_path = output_dir
                .join(package.replace('.', "/"))
                .join(format!("{}.kt", item_struct.ident));
            if item_struct.ident.to_string() != file.class_name && legacy_path.exists() {
                let _ = std::fs::remove_file(&legacy_path);
            }
        }

        if !rust_names.is_empty() {
            strip_legacy_jni_native_data_classes(output_dir, &self.package, &rust_names)?;
        }

        if !aliases.is_empty() {
            let alias_file = KotlinFile {
                contents: render_data_class_aliases_source(&self.package, &aliases),
                package: self.package.clone(),
                class_name: "JNIDataClassAliases".to_string(),
            };
            written.push(alias_file.write(output_dir)?);
        }

        Ok(written)
    }

    /// Emit the package-level wrapper file under `output_dir`. One
    /// Emit one package-level wrapper file for the given subpackage.
    /// One top-level safe wrapper per `MethodEntry` in `pkg_cfg.functions`.
    /// Wrappers delegate to the centralized Native object (see
    /// [`Self::write_jni_native`]). Opaque-handle parameters become
    /// `NativeHandle`; the wrapper body nests `withPtr` / `consume` per
    /// the type-conversion rule. Non-opaque parameters pass through with
    /// the Kotlin type from `kotlin_types`. Opaque-handle return values
    /// are wrapped in `NativeHandle(...)` before return.
    pub(crate) fn write_jni_package(
        &self,
        registry: &Registry<KotlinMeta>,
        kotlin_types: &KotlinTypeMap,
        output_dir: &Path,
        subpackage: &str,
        pkg_cfg: &crate::api::lang::jnigen::jni::jni_ext::PackageConfig,
    ) -> Result<PathBuf, WriteKotlinError> {
        let class_name = self.jni_package_class_name(subpackage);
        let package = if self.package.is_empty() {
            subpackage.to_string()
        } else if subpackage.is_empty() {
            self.package.clone()
        } else {
            format!("{}.{}", self.package, subpackage)
        };
        let contents =
            render_jni_package_source(self, registry, kotlin_types, &pkg_cfg.functions, &package);
        let file = KotlinFile {
            package,
            class_name,
            contents,
        };
        Ok(file.write(output_dir)?)
    }

    /// Emit the centralized Native-object Kotlin file under `output_dir`
    /// (class name from [`JniGen::jni_native_class_name`]). Holds one
    /// `external fun` per `#[prebindgen]` function — names mangled via
    /// `kotlin_fun_name_mangle`, parameter and return types rendered at
    /// the JNI **wire** level so the declarations match the Rust extern
    /// symbols generated under
    /// `Java_<package>_<jni_native_class>_<name>`. Loading the native
    /// library is the wrapper layer's responsibility — the auto-generated
    /// holder stays free of any reference to higher-layer types so that
    /// `io.zenoh.jni.*` doesn't depend on `io.zenoh.*`. Trigger
    /// `System.load` / `System.loadLibrary` from wrapper entry points
    /// (e.g. via a `companion object { init { ZenohLoad } }` block) so
    /// the lib is in place before any extern call.
    pub(crate) fn write_jni_native(
        &self,
        registry: &Registry<KotlinMeta>,
        kotlin_types: &KotlinTypeMap,
        output_dir: &Path,
    ) -> Result<PathBuf, WriteKotlinError> {
        let class_name = self.jni_native_class_name();
        let declared = self.declared_functions();
        let contents =
            render_jni_native_source(self, registry, kotlin_types, &declared, &class_name);
        let file = KotlinFile {
            package: self.package.clone(),
            class_name,
            contents,
        };
        Ok(file.write(output_dir)?)
    }

    /// Emit one Kotlin file per entry in `handles` — each becomes a
    /// `public class <ClassName>(initialPtr: Long) : NativeHandle(initialPtr)`
    /// with the standard `free()` + `private external fun <mangle_fun("freePtr")>(ptr: Long)`
    /// destructor pair, plus one instance method per `#[prebindgen]` fn
    /// listed in [`TypedHandle::functions`]. The promoted method's first
    /// opaque parameter matching the handle's Rust type is dropped — the
    /// method uses inherited `withPtr` / `consume` from [`NativeHandle`]
    /// (i.e. `this` scope) for that param, while every remaining
    /// parameter is emitted exactly as it would appear in the
    /// `JNIWrappers` top-level wrapper (including `impl Into<T>`
    /// dispatch arms and opaque-return wrapping).
    ///
    /// Functions listed under any [`TypedHandle::functions`] are skipped
    /// in [`Self::write_jni_wrappers`] — "Not mentioned functions remain
    /// in `JNIWrapper`" is the assignment rule, exposed by passing the
    /// same `handles` slice to both methods.
    ///
    /// Each handle's `kotlin_fqn` must be registered via
    /// [`Self::kotlin_type_fqn`] so the generator can map it back to its
    /// Rust type-key (which identifies the first param to drop in each
    /// promoted method's signature).
    pub(crate) fn write_typed_handles(
        &self,
        handles: &[TypedHandle<'_>],
        registry: &Registry<KotlinMeta>,
        kotlin_types: &KotlinTypeMap,
        output_dir: &Path,
    ) -> Result<Vec<PathBuf>, WriteKotlinError> {
        // Merged Kotlin type map (callback FQNs + caller-supplied).
        // Same merge order as `render_jni_wrappers_source` — kotlin_types
        // entries WIN over the auto-derived callback FQNs.
        let callback_fqns = self.collect_kotlin_callback_fqns(registry);
        let mut merged_types = KotlinTypeMap::new();
        for (k, v) in callback_fqns.iter() {
            merged_types = merged_types.add(k, v.clone());
        }
        for (k, v) in kotlin_types.iter() {
            merged_types = merged_types.add(k, v.clone());
        }

        let mut written = Vec::new();
        for handle in handles {
            let (package, class_name) = match handle.kotlin_fqn.rsplit_once('.') {
                Some((p, c)) => (p.to_string(), c.to_string()),
                None => (String::new(), handle.kotlin_fqn.to_string()),
            };
            // The typed-handle's Rust type-key is always required — it
            // identifies which param of each `.method(...)` entry becomes
            // `this`. Even with no methods declared we resolve it (cheap)
            // so the wrapper API stays uniform.
            let rust_key = rust_key_for_fqn(self, handle.kotlin_fqn)
                .unwrap_or_else(|| {
                    panic!(
                        "write_typed_handles: kotlin_fqn `{}` is not registered via \
                         JniGen::kotlin_type_fqn — required to identify the typed \
                         handle's Rust type-key for promoted-method param matching.",
                        handle.kotlin_fqn
                    )
                })
                .to_string();
            let file = KotlinFile {
                contents: render_typed_handle_source(
                    self,
                    &package,
                    &class_name,
                    handle.rust_doc,
                    handle.instance_methods,
                    handle.companion_methods,
                    &rust_key,
                    registry,
                    &merged_types,
                ),
                package,
                class_name,
            };
            written.push(file.write(output_dir)?);
        }
        Ok(written)
    }

    /// Return the `<rust-type-key> → <kotlin FQN>` map for every
    /// `impl Fn(args)` type the Registry has resolved. Use this to merge
    /// into a `KotlinTypeMap` consumed by the aggregated-interface
    /// generator (so it can refer to callbacks by their Kotlin FQN).
    pub(crate) fn collect_kotlin_callback_fqns(
        &self,
        registry: &Registry<KotlinMeta>,
    ) -> KotlinTypeMap {
        let mut map = KotlinTypeMap::new();
        let mut seen: HashSet<TypeKey> = HashSet::new();
        for buckets in [&registry.input_types, &registry.output_types] {
            for bucket in buckets.iter() {
                for (key, slot) in bucket {
                    if slot.is_none() {
                        continue;
                    }
                    if !seen.insert(key.clone()) {
                        continue;
                    }
                    let ty = key.to_type();
                    if let Some(args) = extract_fn_trait_args(&ty) {
                        // Re-use the single source of truth for callback
                        // FQN derivation — same closure-mangled name the
                        // converter dispatcher stamps into metadata.
                        let fqn = self.auto_callback_fqn(&args);
                        map = map.add(key.as_str(), fqn);
                    }
                }
            }
        }
        // Merge in plugin-supplied extra mappings (e.g. data-class FQNs
        // that aren't reachable from impl-Fn types).
        for (rust_canon, fqn) in &self.kotlin_type_fqns {
            map = map.add(rust_canon.as_str(), fqn.clone());
        }
        map
    }
}

fn build_callback_kotlin_file(
    ext: &JniGen,
    args: &[syn::Type],
    registry: &Registry<KotlinMeta>,
) -> KotlinFile {
    let name = derive_callback_name(args);
    let class_name = ext.mangle_callback(&name);
    let package = ext.kotlin_callback_package.clone();

    // Resolve each arg's Kotlin type by reading the output-direction
    // entry's metadata — callback args flow inverse to the callback
    // (Rust produces them, Java consumes them). Fall back to the bare
    // last-segment ident when the metadata is missing (matches today's
    // behavior; preserves the dead-stub compile path).
    let mut params: Vec<String> = Vec::new();
    let mut used_fqns: BTreeSet<String> = BTreeSet::new();
    for (i, arg) in args.iter().enumerate() {
        let entry = registry.output_entry(arg);
        // Opaque-handle args: the converter's value-name is `"Long"`, but the
        // callback delivers the typed handle class. Prefer its registered FQN
        // so the param type — and the file's import — resolve to e.g.
        // `io.zenoh.jni.scouting.ZHello`.
        let kotlin_ty = entry
            .and_then(|e| e.metadata.projection.as_ref())
            .map(|h| crate::api::lang::jnigen::jni::jni_ext::handle_field_fqn(ext, h))
            .or_else(|| entry.and_then(|e| e.metadata.kotlin_name.clone()))
            .or_else(|| {
                if let syn::Type::Path(tp) = arg {
                    if let Some(last) = tp.path.segments.last() {
                        return Some(last.ident.to_string());
                    }
                }
                None
            })
            .unwrap_or_else(|| "Any".to_string());
        let short = register_fqn(&kotlin_ty, &mut used_fqns);
        let optional_suffix = if is_option_type(arg) { "?" } else { "" };
        params.push(format!("        p{i}: {short}{optional_suffix},"));
    }

    let contents =
        templates::callback::render_kotlin_interface(&package, &class_name, &params, &used_fqns);
    KotlinFile {
        package,
        class_name,
        contents,
    }
}

/// Derive the auto-callback short Kotlin name for an `impl Fn(args)`
/// signature. Always starts with the hardcoded `"On"` and appends each
/// concatenated parameter type Rust short idents + `"Callback"` suffix
/// (`Fn(Query)` → `"QueryCallback"`, `Fn(Reply)` → `"ReplyCallback"`,
/// `Fn(K, V)` → `"KVCallback"`, `Fn()` → `"Callback"`). The result
/// feeds [`JniGen::mangle_callback`] before the FQN is qualified
/// against [`JniGen::kotlin_callback_package`].
pub(crate) fn derive_callback_name(args: &[syn::Type]) -> String {
    let mut s = String::new();
    for a in args {
        s.push_str(&type_short_ident(a));
    }
    s.push_str("Callback");
    s
}

fn type_short_ident(ty: &syn::Type) -> String {
    if let syn::Type::Path(tp) = ty {
        if let Some(last) = tp.path.segments.last() {
            return last.ident.to_string();
        }
    }
    "Unknown".into()
}

fn is_option_type(ty: &syn::Type) -> bool {
    if let syn::Type::Path(tp) = ty {
        if let Some(last) = tp.path.segments.last() {
            return last.ident == "Option";
        }
    }
    false
}

/// Peel a leading `&`/`&mut` and an `Option<…>` layer to expose the inner type
/// used for enum detection. So `&Priority`, `Priority`, and `Option<Priority>`
/// all probe as `Priority` — letting nullable enum params (`Option<enum>`) wire
/// as `Int?` + `?.value` just like a non-null enum wires as `Int` + `.value`,
/// instead of leaking the enum object to the (boxed-int-expecting) Rust converter.
fn enum_probe_type(ty: &syn::Type) -> syn::Type {
    let stripped = match ty {
        syn::Type::Reference(r) => (*r.elem).clone(),
        other => other.clone(),
    };
    match crate::api::lang::jnigen::jni::jni_ext::option_inner_type(&stripped) {
        Some(inner) => match inner {
            syn::Type::Reference(r) => (*r.elem).clone(),
            other => other,
        },
        None => stripped,
    }
}

/// `true` if `ty` is `Option<&T>` or `Option<&mut T>` (any inner T).
/// Mirrors `option_inner_ref_mutability` in `jni_ext.rs` — kept here too
/// to avoid a cross-module helper just for one call site.
fn is_option_ref(ty: &syn::Type) -> bool {
    let syn::Type::Path(tp) = ty else {
        return false;
    };
    let Some(seg) = tp.path.segments.last() else {
        return false;
    };
    if seg.ident != "Option" {
        return false;
    }
    let syn::PathArguments::AngleBracketed(ab) = &seg.arguments else {
        return false;
    };
    let Some(syn::GenericArgument::Type(inner)) = ab.args.first() else {
        return false;
    };
    matches!(inner, syn::Type::Reference(_))
}

/// Render the Kotlin type for a closeable handle reached through the
/// folded [`FoldStrategy`] layers, given the leaf typed-handle short
/// name (e.g. `"ZKeyExpr"`): `Direct → "ZKeyExpr"`,
/// `Nullable(inner) → "<inner>?"`, `Iterable(inner) → "List<<inner>>"`.
fn render_handle_type(
    strategy: &crate::api::lang::jnigen::jni::jni_ext::FoldStrategy,
    leaf: &str,
) -> String {
    use crate::api::lang::jnigen::jni::jni_ext::FoldStrategy::*;
    match strategy {
        Direct => leaf.to_string(),
        // The declared Kotlin projection type is `T?` regardless of how null
        // is represented over the wire — the wrap fold and the wire-return
        // helper read the kind to handle the wire shape separately.
        Nullable { inner, .. } => format!("{}?", render_handle_type(inner, leaf)),
        Iterable(inner) => format!("List<{}>", render_handle_type(inner, leaf)),
    }
}

/// Render the Kotlin `close()` expression for a handle `receiver` through
/// the folded [`FoldStrategy`] layers. Fresh lambda variable per nesting
/// level avoids `it` shadowing; the common single-layer cases are
/// special-cased for readable output (`x?.close()`, `x.forEach { it.close() }`).
fn render_handle_close(
    strategy: &crate::api::lang::jnigen::jni::jni_ext::FoldStrategy,
    receiver: &str,
) -> String {
    use crate::api::lang::jnigen::jni::jni_ext::FoldStrategy::*;
    fn go(
        strategy: &crate::api::lang::jnigen::jni::jni_ext::FoldStrategy,
        receiver: &str,
        depth: usize,
    ) -> String {
        match strategy {
            Direct => format!("{receiver}.close()"),
            // The Kotlin-side receiver is already nullable (`render_handle_type`
            // emits `T?` for both niche and boxed kinds), so `?.close()` covers
            // both wire representations.
            Nullable { inner, .. } => match &**inner {
                Direct => format!("{receiver}?.close()"),
                _ => {
                    let v = format!("e{depth}");
                    format!("{receiver}?.let {{ {v} -> {} }}", go(inner, &v, depth + 1))
                }
            },
            Iterable(inner) => {
                let v = format!("e{depth}");
                format!(
                    "{receiver}.forEach {{ {v} -> {} }}",
                    go(inner, &v, depth + 1)
                )
            }
        }
    }
    go(strategy, receiver, 0)
}

/// Fold the projection wrap call `W(receiver)` through the
/// [`FoldStrategy`] layers:
/// * `Direct`         → `W(x)`
/// * `Nullable{Boxed}` → `x?.let { W(it) }` (JVM-null at the wire)
/// * `Nullable{Niche}` over a primitive wire (e.g. `jlong`) →
///   `x.let { if (it == <sentinel>) null else W(it) }`
/// * `Nullable{Niche}` over an object wire (e.g. `JByteArray`) →
///   `x?.let { W(it) }` (the wire is already a nullable reference)
/// * `Iterable`       → `x.map { W(it) }`
///
/// `niche_sentinel` is the Kotlin literal to compare against for the
/// `Niche+primitive` arm (e.g. `"0L"` for `jlong`-wired handles). When the
/// wire is object-shaped the sentinel is unused — `null` is the wire-level
/// representation and `?.let` is a no-cost null check.
fn fold_projection_wrap(
    strategy: &crate::api::lang::jnigen::jni::jni_ext::FoldStrategy,
    receiver: &str,
    wrap_class: &str,
    niche_sentinel: Option<&str>,
) -> String {
    use crate::api::lang::jnigen::jni::jni_ext::{FoldStrategy::*, NullableKind};
    fn go(
        s: &crate::api::lang::jnigen::jni::jni_ext::FoldStrategy,
        r: &str,
        w: &str,
        sentinel: Option<&str>,
        depth: usize,
    ) -> String {
        match s {
            Direct => format!("{w}({r})"),
            Nullable { kind, inner } => match (kind, &**inner) {
                // Primitive-wired niche → can't carry null on the wire, so
                // compare against the sentinel and synthesize null on the
                // Kotlin side.
                (NullableKind::Niche, Direct) if sentinel.is_some() => {
                    let s = sentinel.unwrap();
                    format!("{r}.let {{ if (it == {s}) null else {w}(it) }}")
                }
                // Object-wired niche or fully boxed Nullable → `?.let { W(it) }`.
                (_, Direct) => format!("{r}?.let {{ {w}(it) }}"),
                // Deeper nesting. The niche/boxed distinction is only
                // observable at the outermost layer covering a `Direct`
                // leaf; intermediate layers (nullable-of-iterable etc.)
                // can keep the simple form because Kotlin's `?.` chain
                // already represents the layered null.
                _ => {
                    let v = format!("e{depth}");
                    format!(
                        "{r}?.let {{ {v} -> {} }}",
                        go(inner, &v, w, sentinel, depth + 1)
                    )
                }
            },
            Iterable(inner) => match &**inner {
                Direct => format!("{r}.map {{ {w}(it) }}"),
                _ => {
                    let v = format!("e{depth}");
                    format!(
                        "{r}.map {{ {v} -> {} }}",
                        go(inner, &v, w, sentinel, depth + 1)
                    )
                }
            },
        }
    }
    go(strategy, receiver, wrap_class, niche_sentinel, 0)
}

/// JNI extern's declared Kotlin wire-return for a projection. The leaf wire
/// is the inner converter's destination Kotlin name: `Long` for handles
/// (boxed jlong), the inner field's converter result for value classes (e.g.
/// `ByteArray` for `ZenohId`/`ZBytes`). The fold honours
/// [`NullableKind`] so the declared wire matches the runtime ABI:
/// `Niche+primitive` keeps the layer non-nullable on the wire (the sentinel
/// represents null); `Niche+object` and `Boxed` add `?`.
fn projection_wire_return(
    ext: &JniGen,
    registry: &Registry<KotlinMeta>,
    proj: &crate::api::lang::jnigen::jni::jni_ext::Projection,
    imports: &mut BTreeSet<String>,
) -> String {
    use crate::api::lang::jnigen::jni::jni_ext::{FoldStrategy, NullableKind, ProjectionKind};
    let (inner_wire_name, inner_is_primitive) = match proj.kind {
        ProjectionKind::Handle => ("Long".to_string(), true),
        ProjectionKind::ValueClass => {
            let vc_ty: syn::Type = syn::parse_str(&proj.leaf_key).unwrap_or_else(|_| {
                panic!("projection_wire_return: bad leaf_key `{}`", proj.leaf_key)
            });
            let inner_ty = crate::api::lang::jnigen::jni::jni_ext::value_class_inner_type_for(
                ext, registry, &vc_ty,
            )
            .unwrap_or_else(|| {
                panic!(
                    "projection_wire_return: `{}` is not a registered value class",
                    proj.leaf_key
                )
            });
            let inner_entry = registry.output_entry(&inner_ty).unwrap_or_else(|| {
                panic!(
                    "projection_wire_return: inner of `{}` has no output converter",
                    proj.leaf_key
                )
            });
            let n = inner_entry.metadata.kotlin_name.clone().unwrap_or_else(|| {
                panic!(
                    "projection_wire_return: inner of `{}` has no Kotlin name",
                    proj.leaf_key
                )
            });
            let is_prim = matches!(
                crate::api::lang::jnigen::jni::wire_access::jni_field_access(
                    &inner_entry.destination
                ),
                Some((_, _, false))
            );
            (register_fqn(&n, imports), is_prim)
        }
        // Value-blob's inner wire is always `ByteArray` (object-shaped).
        ProjectionKind::ValueBlob => ("ByteArray".to_string(), false),
    };
    fn fold(s: &FoldStrategy, leaf: &str, leaf_is_primitive: bool) -> String {
        match s {
            FoldStrategy::Direct => leaf.to_string(),
            FoldStrategy::Nullable { kind, inner } => {
                let inner_str = fold(inner, leaf, leaf_is_primitive);
                // A niche layer over a primitive wire keeps the wire
                // non-nullable — the sentinel value is the null
                // representation. Object-wired niches and full-boxed
                // Nullables both add `?` (JVM null on the reference).
                match (kind, &**inner) {
                    (NullableKind::Niche, FoldStrategy::Direct) if leaf_is_primitive => inner_str,
                    _ => format!("{}?", inner_str),
                }
            }
            FoldStrategy::Iterable(inner) => {
                format!("List<{}>", fold(inner, leaf, leaf_is_primitive))
            }
        }
    }
    fold(&proj.strategy, &inner_wire_name, inner_is_primitive)
}

/// Kotlin null-sentinel literal for the *leaf wire* of a projection. Read
/// at the wrapper-body call site and forwarded to [`fold_projection_wrap`];
/// `None` for object-wired leaves (e.g. value classes over `ByteArray`),
/// where `?.let { }` covers the JVM-null case directly.
fn projection_leaf_sentinel(
    ext: &JniGen,
    registry: &Registry<KotlinMeta>,
    proj: &crate::api::lang::jnigen::jni::jni_ext::Projection,
) -> Option<String> {
    use crate::api::lang::jnigen::jni::jni_ext::ProjectionKind;
    let leaf_wire: syn::Type = match proj.kind {
        ProjectionKind::Handle => syn::parse_quote!(jni::sys::jlong),
        ProjectionKind::ValueClass => {
            let vc_ty: syn::Type = syn::parse_str(&proj.leaf_key).ok()?;
            let inner_ty = crate::api::lang::jnigen::jni::jni_ext::value_class_inner_type_for(
                ext, registry, &vc_ty,
            )?;
            registry.output_entry(&inner_ty)?.destination.clone()
        }
        // Value-blob leaf wire is always `JByteArray` (object-shaped) — no
        // primitive sentinel; JVM `null` represents the absent value, so
        // `?.let` covers nullability.
        ProjectionKind::ValueBlob => syn::parse_quote!(jni::objects::JByteArray),
    };
    kotlin_null_sentinel(&leaf_wire).map(|s| s.to_string())
}

/// Kotlin literal for the null-sentinel of a primitive wire — used by
/// [`fold_projection_wrap`] when a `Niche` layer covers a primitive wire and
/// can't carry JVM null. Mirrors `jni_field_access`'s primitive descriptors.
/// Returns `None` for object-shaped wires (where JVM null *is* the null
/// representation and `?.let` is the right pattern).
fn kotlin_null_sentinel(wire: &syn::Type) -> Option<&'static str> {
    let (_, _, is_object) = crate::api::lang::jnigen::jni::wire_access::jni_field_access(wire)?;
    if is_object {
        return None;
    }
    let syn::Type::Path(tp) = wire else {
        return None;
    };
    let last = tp.path.segments.last()?;
    Some(match last.ident.to_string().as_str() {
        "jlong" => "0L",
        "jint" | "jshort" | "jbyte" | "jchar" => "0",
        "jfloat" => "0.0f",
        "jdouble" => "0.0",
        "jboolean" => "false",
        _ => return None,
    })
}

fn register_fqn(fqn: &str, used: &mut BTreeSet<String>) -> String {
    if fqn.contains('.') {
        used.insert(fqn.to_string());
        fqn.rsplit('.').next().unwrap_or(fqn).to_string()
    } else {
        fqn.to_string()
    }
}

// ── Safe-wrapper emitters ──────────────────────────────────────────────

/// One generated Kotlin `enum class` source — variants in
/// SCREAMING_SNAKE_CASE, each carrying the Rust discriminant as a
/// `val value: Int`, plus a `fromInt(value: Int)` companion. Mirrors
/// the hand-written `io.zenoh.qos.Priority` shape so adapter code that
/// already speaks the `.value` / `.fromInt(...)` idiom keeps working.
fn render_enum_source(
    ext: &JniGen,
    package: &str,
    class_name: &str,
    item_enum: &syn::ItemEnum,
    instance_methods: &[MethodEntry],
    companion_methods_in: &[MethodEntry],
    registry: &Registry<KotlinMeta>,
    kotlin_types: &KotlinTypeMap,
) -> String {
    assert!(
        instance_methods.is_empty(),
        "render_enum_source: `{class_name}` has `.method(...)` entries but instance \
         methods on `enum_class`-declared types are not supported yet — declare them \
         as `.companion_method(...)` for now",
    );
    // Same discriminant source of truth the Rust `jint → variant` decode
    // uses, so Kotlin `value(N)` and the generated decode agree.
    let variants: Vec<(String, i64)> =
        crate::api::lang::jnigen::util::enum_discriminant_values(item_enum)
            .into_iter()
            .map(|(ident, value)| {
                (
                    crate::api::lang::jnigen::util::camel_to_screaming_snake(&ident.to_string()),
                    value,
                )
            })
            .collect();

    let mut imports: BTreeSet<String> = BTreeSet::new();
    let mut companion_methods = String::new();
    for entry in companion_methods_in {
        let (item_fn, _loc) = registry
            .functions
            .get(&entry.rust_ident)
            .unwrap_or_else(|| {
                panic!(
                    "render_enum_source: `{class_name}` promotes function `{}` \
                 which is not present in `registry.functions` — check the spelling against \
                 the matching `#[prebindgen]` Rust fn name.",
                    entry.rust_ident,
                )
            });
        let (block, _kind) = render_wrapper_fn(
            ext,
            item_fn,
            registry,
            kotlin_types,
            &mut imports,
            None,
            entry.kotlin_name_override.as_deref(),
        )
        .unwrap_or_else(|| {
            panic!(
                "render_enum_source: `{class_name}` promotes function `{}` \
                 but its parameter types couldn't be Kotlin-resolved — verify that all \
                 non-opaque parameter types are registered in `kotlin_types`.",
                entry.rust_ident,
            )
        });
        if !companion_methods.is_empty() {
            companion_methods.push('\n');
        }
        companion_methods.push_str(&block);
        companion_methods.push('\n');
    }

    let mut import_list: Vec<String> = imports
        .iter()
        .filter(|fqn| {
            let pkg = fqn.rsplit_once('.').map(|(p, _)| p).unwrap_or("");
            !pkg.is_empty() && pkg != package
        })
        .cloned()
        .collect();
    import_list.sort();
    import_list.dedup();

    let mut s = String::new();
    s.push_str("// Auto-generated by JniGen — do not edit by hand.\n");
    if !package.is_empty() {
        s.push_str(&format!("package {}\n\n", package));
    }
    for imp in &import_list {
        s.push_str(&format!("import {}\n", imp));
    }
    if !import_list.is_empty() {
        s.push('\n');
    }
    s.push_str(&format!(
        "/** JVM-side surface for the native Rust `{}` enum. */\n",
        item_enum.ident
    ));
    s.push_str(&format!(
        "public enum class {}(public val value: Int) {{\n",
        class_name
    ));
    for (i, (name, value)) in variants.iter().enumerate() {
        let sep = if i + 1 == variants.len() { ";" } else { "," };
        s.push_str(&format!("    {}({}){}\n", name, value, sep));
    }
    s.push('\n');
    s.push_str("    public companion object {\n");
    // `@JvmStatic` exposes `fromInt` as a real static method on the enum
    // class itself (rather than only on the `Companion` nested class). The
    // generated struct-encoder calls it via `env.call_static_method`, which
    // wouldn't find a companion-only method.
    s.push_str(&format!(
        "        @JvmStatic\n        public fun fromInt(value: Int): {} = entries.first {{ it.value == value }}\n",
        class_name
    ));
    if !companion_methods.is_empty() {
        s.push('\n');
        for line in companion_methods.lines() {
            if line.is_empty() {
                s.push('\n');
            } else {
                s.push_str("        ");
                s.push_str(line);
                s.push('\n');
            }
        }
    }
    s.push_str("    }\n");
    s.push_str("}\n");
    s
}

/// One generated Kotlin `data class` (or `@JvmInline value class` when
/// `value_class` is set) source for a `data_class` /
/// `value_class`-declared Rust struct.
fn render_data_class_source(
    ext: &JniGen,
    package: &str,
    class_name: &str,
    item_struct: &syn::ItemStruct,
    registry: &Registry<KotlinMeta>,
    kotlin_types: &KotlinTypeMap,
    instance_methods: &[MethodEntry],
    companion_methods_in: &[MethodEntry],
    throwable: bool,
    value_class: bool,
    rust_key: &str,
) -> String {
    assert!(
        !(value_class && !instance_methods.is_empty()),
        "render_data_class_source: `{class_name}` is a `value_class` and has \
         `.method(...)` entries; instance methods on value classes aren't supported yet \
         — declare them as `.companion_method(...)` for now",
    );
    let fields_named = match &item_struct.fields {
        syn::Fields::Named(n) => &n.named,
        _ => {
            panic!(
                "render_data_class_source: struct `{}` must use named fields to map onto Kotlin data class properties",
                item_struct.ident
            )
        }
    };
    if value_class {
        assert!(
            !throwable,
            "render_data_class_source: `{}` is registered as both \
             `value_class` and `throwable` — @JvmInline value \
             classes cannot extend `Exception`. Drop `.throwable()` or \
             switch to `data_class`.",
            item_struct.ident
        );
        assert!(
            fields_named.len() == 1,
            "render_data_class_source: `value_class` requires \
             struct `{}` to have exactly one field; found {}. Use \
             `data_class` for multi-field structs.",
            item_struct.ident,
            fields_named.len()
        );
    }

    let mut imports: BTreeSet<String> = BTreeSet::new();
    let mut field_lines: Vec<String> = Vec::new();
    // Track per-field destructible (name, folded close strategy) so the
    // bottom emitter can produce a matching `close()` body for each.
    let mut destructible_fields: Vec<(
        String,
        crate::api::lang::jnigen::jni::jni_ext::FoldStrategy,
    )> = Vec::new();
    for field in fields_named {
        let field_ident = field.ident.as_ref().unwrap_or_else(|| {
            panic!(
                "render_data_class_source: struct `{}` has an unnamed field in named-fields context",
                item_struct.ident
            )
        });
        let kotlin_field_name = snake_to_camel(&field_ident.to_string());
        // When the class extends Exception (throwable), the `message`
        // field shadows `Exception.message` — Kotlin requires `override`.
        let override_prefix = if throwable && kotlin_field_name == "message" {
            "override "
        } else {
            ""
        };

        // Projection field (opaque handle or value class): the typed Kotlin
        // type (`ZKeyExpr?`, `List<ZKeyExpr>`, `ZenohId`, `List<ZenohId>`, …)
        // is derived from the folded `Projection` the type-unfolding mechanism
        // propagated onto this field's converter metadata, instead of a
        // syntactic `Option<T>` peel. Closeable `close()` emission applies
        // only to `Handle` kind (and only when owned); value classes are
        // erased to their inner wire, so they never close.
        let field_projection = registry
            .output_entry(&field.ty)
            .and_then(|e| e.metadata.projection.clone())
            .or_else(|| {
                registry
                    .input_entry(&field.ty)
                    .and_then(|e| e.metadata.projection.clone())
            });
        if let Some(h) = field_projection {
            let fqn = ext
                .kotlin_type_fqns
                .iter()
                .find(|(k, _)| k == &h.leaf_key)
                .map(|(_, v)| v.clone())
                .unwrap_or_else(|| {
                    panic!(
                        "render_data_class_source: projection field `{}.{}` leaf `{}` has no \
                         Kotlin FQN registered (ptr_class / value_class)",
                        item_struct.ident, field_ident, h.leaf_key
                    )
                });
            let short = register_fqn(&fqn, &mut imports);
            field_lines.push(format!(
                "    {override_prefix}val {kotlin_field_name}: {},",
                render_handle_type(&h.strategy, &short)
            ));
            if matches!(
                h.kind,
                crate::api::lang::jnigen::jni::jni_ext::ProjectionKind::Handle
            ) && h.owned
            {
                destructible_fields.push((kotlin_field_name, h.strategy));
            }
            continue;
        }

        let kotlin_ty = registry
            .output_entry(&field.ty)
            .and_then(|e| e.metadata.kotlin_name.clone())
            .or_else(|| registry.input_entry(&field.ty).and_then(|e| e.metadata.kotlin_name.clone()))
            .unwrap_or_else(|| {
                panic!(
                    "render_data_class_source: field `{}.{}` has no Kotlin type mapping; register converters before declaring data_class",
                    item_struct.ident,
                    field_ident
                )
            });
        let short = register_fqn(&kotlin_ty, &mut imports);
        // `Option<T>` whose wire is a JNI primitive (jlong/jint/jboolean/…)
        // and that *isn't* an opaque handle (handled above) is encoded by
        // the struct emitter as the bare primitive with a sentinel for
        // `None` (0 / 0.0 / false). The Kotlin field must match that JVM
        // slot: declare it non-nullable so the constructor signature
        // stays primitive (`J` vs `Ljava/lang/Long;`). Nullable boxing
        // for non-handle primitives would require generator-side changes
        // in `struct_output_body` to `Long.valueOf(...)`.
        let wire = registry
            .output_entry(&field.ty)
            .map(|e| e.destination.clone());
        let primitive_wire = wire
            .as_ref()
            .map(|w| crate::api::lang::jnigen::jni::jni_ext::is_jni_primitive(w))
            .unwrap_or(false);
        let optional_suffix = if is_option_type(&field.ty) && !primitive_wire {
            "?"
        } else {
            ""
        };
        field_lines.push(format!(
            "    {override_prefix}val {kotlin_field_name}: {short}{optional_suffix},"
        ));
    }

    let mut instance_body = String::new();
    for entry in instance_methods {
        let (item_fn, _loc) = registry
            .functions
            .get(&entry.rust_ident)
            .unwrap_or_else(|| {
                panic!(
                    "render_data_class_source: `{class_name}` promotes function `{}` \
                 which is not present in `registry.functions` — check the spelling against \
                 the matching `#[prebindgen]` Rust fn name.",
                    entry.rust_ident,
                )
            });
        let (block, kind) = render_wrapper_fn(
            ext,
            item_fn,
            registry,
            kotlin_types,
            &mut imports,
            Some(rust_key),
            entry.kotlin_name_override.as_deref(),
        )
        .unwrap_or_else(|| {
            panic!(
                "render_data_class_source: `{class_name}` promotes function `{}` \
                 but its parameter types couldn't be Kotlin-resolved — verify that all \
                 non-opaque parameter types are registered in `kotlin_types`.",
                entry.rust_ident,
            )
        });
        if kind != MethodKind::Instance {
            panic!(
                ".method({}) on `{class_name}`: the function's first parameter doesn't match \
                 the class's Rust type ({rust_key}) — declare it as `.companion_method(...)` \
                 if it isn't an instance method.",
                entry.rust_ident,
            );
        }
        if !instance_body.is_empty() {
            instance_body.push('\n');
        }
        instance_body.push_str(&block);
        instance_body.push('\n');
    }

    let mut companion_methods = String::new();
    for entry in companion_methods_in {
        let (item_fn, _loc) = registry
            .functions
            .get(&entry.rust_ident)
            .unwrap_or_else(|| {
                panic!(
                    "render_data_class_source: `{class_name}` promotes function `{}` \
                 which is not present in `registry.functions` — check the spelling against \
                 the matching `#[prebindgen]` Rust fn name.",
                    entry.rust_ident,
                )
            });
        let (block, _kind) = render_wrapper_fn(
            ext,
            item_fn,
            registry,
            kotlin_types,
            &mut imports,
            None,
            entry.kotlin_name_override.as_deref(),
        )
        .unwrap_or_else(|| {
            panic!(
                "render_data_class_source: `{class_name}` promotes function `{}` \
                 but its parameter types couldn't be Kotlin-resolved — verify that all \
                 non-opaque parameter types are registered in `kotlin_types`.",
                entry.rust_ident,
            )
        });
        if !companion_methods.is_empty() {
            companion_methods.push('\n');
        }
        companion_methods.push_str(&block);
        companion_methods.push('\n');
    }

    // Wrapper methods emitted into subpackages still call the centralized
    // Native object anchored at the base package.
    if package != ext.package && (!instance_body.is_empty() || !companion_methods.is_empty()) {
        imports.insert(format!("{}.{}", ext.package, ext.jni_native_class_name()));
    }

    let mut import_list: Vec<String> = imports
        .iter()
        .filter(|fqn| {
            let pkg = fqn.rsplit_once('.').map(|(p, _)| p).unwrap_or("");
            !pkg.is_empty() && pkg != package
        })
        .cloned()
        .collect();
    import_list.sort();
    import_list.dedup();

    let mut s = String::new();
    s.push_str("// Auto-generated by JniGen — do not edit by hand.\n");
    if !package.is_empty() {
        s.push_str(&format!("package {}\n\n", package));
    }
    for imp in &import_list {
        s.push_str(&format!("import {}\n", imp));
    }
    if !import_list.is_empty() {
        s.push('\n');
    }
    if value_class {
        assert!(
            destructible_fields.is_empty(),
            "render_data_class_source: `value_class` struct `{}` \
             has a destructible native-handle field — value classes can \
             only express one inline-erased payload, not the \
             `AutoCloseable` + `close()` contract a handle field needs. \
             Use `data_class` for handle-bearing wrappers.",
            item_struct.ident
        );
        // Single line is enforced by the upstream `fields_named.len() == 1`
        // assertion; strip the data-class formatting (leading indent and
        // trailing comma) so the primary constructor reads cleanly.
        let only = field_lines[0]
            .trim_start()
            .trim_end_matches(',')
            .to_string();
        s.push_str("@JvmInline\n");
        s.push_str(&format!("public value class {}({})", class_name, only));
        if companion_methods.is_empty() {
            s.push('\n');
        } else {
            s.push_str(" {\n");
            s.push_str("    public companion object {\n");
            for line in companion_methods.lines() {
                if line.is_empty() {
                    s.push('\n');
                } else {
                    s.push_str("        ");
                    s.push_str(line);
                    s.push('\n');
                }
            }
            s.push_str("    }\n");
            s.push_str("}\n");
        }
    } else {
        s.push_str(&format!("public data class {}(\n", class_name));
        for line in &field_lines {
            s.push_str(line);
            s.push('\n');
        }
        // Supertype clause. `Exception(...)` (a class) and `AutoCloseable`
        // (an interface) stack — Kotlin allows at most one class super + any
        // interfaces. `: Exception(message)` picks the field literally named
        // `message` to forward to Exception's message slot; falls back to
        // `: Exception()` when no such field exists (data-class auto-toString
        // still surfaces the structured fields).
        let exception_clause: Option<String> = if throwable {
            let has_message = fields_named.iter().any(|f| {
                f.ident
                    .as_ref()
                    .map(|i| i.to_string() == "message")
                    .unwrap_or(false)
            });
            Some(if has_message {
                "Exception(message)".to_string()
            } else {
                "Exception()".to_string()
            })
        } else {
            None
        };
        let supertypes: Vec<String> = match (&exception_clause, !destructible_fields.is_empty()) {
            (Some(e), true) => vec![e.clone(), "AutoCloseable".to_string()],
            (Some(e), false) => vec![e.clone()],
            (None, true) => vec!["AutoCloseable".to_string()],
            (None, false) => vec![],
        };
        if supertypes.is_empty() {
            s.push_str(") {\n");
        } else {
            s.push_str(&format!(") : {} {{\n", supertypes.join(", ")));
        }
        if !destructible_fields.is_empty() {
            // `close()` walks every destructible field via its folded close
            // strategy. `JNINativeHandle.close()` is idempotent
            // (Cleaner.Cleanable.clean() invokes exactly once), so calling
            // this multiple times — or alongside the cleaner's own firing on
            // GC — is safe. NOTE: `data class` copy() shares the handle
            // reference between copies; if you intend to close independently,
            // don't copy this class.
            s.push_str("    override fun close() {\n");
            for (fname, strategy) in &destructible_fields {
                s.push_str(&format!(
                    "        {}\n",
                    render_handle_close(strategy, fname)
                ));
            }
            s.push_str("    }\n\n");
        }
        if !instance_body.is_empty() {
            for line in instance_body.lines() {
                if line.is_empty() {
                    s.push('\n');
                } else {
                    s.push_str("    ");
                    s.push_str(line);
                    s.push('\n');
                }
            }
            s.push('\n');
        }
        s.push_str("    public companion object {\n");
        if !companion_methods.is_empty() {
            for line in companion_methods.lines() {
                if line.is_empty() {
                    s.push('\n');
                } else {
                    s.push_str("        ");
                    s.push_str(line);
                    s.push('\n');
                }
            }
        }
        s.push_str("    }\n");
        s.push_str("}\n");
    }
    s
}

fn render_data_class_aliases_source(package: &str, aliases: &[(String, String)]) -> String {
    let mut pairs = aliases.to_vec();
    pairs.sort_by(|a, b| a.0.cmp(&b.0));
    pairs.dedup_by(|a, b| a.0 == b.0 && a.1 == b.1);

    let mut s = String::new();
    s.push_str("// Auto-generated by JniGen — do not edit by hand.\n");
    if !package.is_empty() {
        s.push_str(&format!("package {}\n\n", package));
    }
    s.push_str("// Compatibility aliases for legacy un-mangled data-class references.\n");
    for (legacy, current) in pairs {
        s.push_str(&format!("public typealias {} = {}\n", legacy, current));
    }
    s
}

fn strip_legacy_jni_native_data_classes(
    output_dir: &Path,
    package: &str,
    _rust_names: &[String],
) -> Result<(), WriteKotlinError> {
    let jni_native_path = output_dir
        .join(package.replace('.', "/"))
        .join("JNINative.kt");
    if !jni_native_path.exists() {
        return Ok(());
    }

    let source = std::fs::read_to_string(&jni_native_path)?;
    let lines: Vec<&str> = source.lines().collect();
    let Some(object_start) = lines
        .iter()
        .position(|line| line.trim_start().starts_with("internal object JNINative {"))
    else {
        return Ok(());
    };

    let mut filtered: Vec<String> = Vec::new();
    for line in &lines[..object_start] {
        let trimmed = line.trim_start();
        if trimmed.starts_with("package ")
            || trimmed.starts_with("import ")
            || trimmed.starts_with("//")
            || trimmed.is_empty()
        {
            filtered.push((*line).to_string());
        }
    }
    for line in &lines[object_start..] {
        filtered.push((*line).to_string());
    }

    let mut out = filtered.join("\n");
    out.push('\n');
    if out != source {
        std::fs::write(jni_native_path, out)?;
    }
    Ok(())
}

/// Render one typed-handle Kotlin source file. Pure-shell form (with
/// the closure `|n| format!("{n}ViaJNI")` installed via
/// [`JniGen::kotlin_fun_name_mangle`]):
///
/// ```kotlin
/// public class JNIFoo(initialPtr: Long) : NativeHandle(initialPtr) {
///     public fun free() = free { freePtrViaJNI(it) }
///     private external fun freePtrViaJNI(ptr: Long)
/// }
/// ```
///
/// When `promoted_functions` is non-empty, one extra instance method is
/// appended per `#[prebindgen]` fn — the matching opaque first param
/// (Rust type-key = `promoted_rust_key`) is dropped from the Kotlin
/// signature, and its `withPtr` / `consume` wrapper uses the
/// inherited [`NativeHandle`] scope.
///
/// The free-pointer extern name is built as
/// `<mangle_fun("freePtr")>`. Kotlin/JVM's JNI name mangler binds it
/// to the matching `Java_<pkg>_<class>_<mangle_fun("freePtr")>`
/// extern on the Rust side (the auto-generated destructor).
fn render_typed_handle_source(
    ext: &JniGen,
    package: &str,
    class_name: &str,
    rust_doc_name: &str,
    instance_methods: &[MethodEntry],
    companion_methods: &[MethodEntry],
    promoted_rust_key: &str,
    registry: &Registry<KotlinMeta>,
    kotlin_types: &KotlinTypeMap,
) -> String {
    // Build method bodies first so we can collect imports up front.
    // Two buckets — instance methods land in the class body; companion
    // methods are wrapped in a `companion object { ... }` block. All
    // promoted wrappers dispatch into the centralized Native object;
    // no per-class `external fun` declarations are emitted here.
    let mut imports: BTreeSet<String> = BTreeSet::new();
    // Every typed handle extends the shared `NativeHandle` base (emitted in
    // `ext.package`); pull it in unless we *are* that package.
    if !ext.package.is_empty() {
        imports.insert(format!("{}.NativeHandle", ext.package));
    }
    let mut instance_body = String::new();
    let mut companion_body = String::new();
    for entry in instance_methods {
        let (item_fn, _loc) = registry
            .functions
            .get(&entry.rust_ident)
            .unwrap_or_else(|| {
                panic!(
                    "render_typed_handle_source: `{class_name}` promotes function `{}` \
                 which is not present in `registry.functions` — check the spelling against \
                 the matching `#[prebindgen]` Rust fn name.",
                    entry.rust_ident,
                )
            });
        let (block, kind) = render_wrapper_fn(
            ext,
            item_fn,
            registry,
            kotlin_types,
            &mut imports,
            Some(promoted_rust_key),
            entry.kotlin_name_override.as_deref(),
        )
        .unwrap_or_else(|| {
            panic!(
                "render_typed_handle_source: `{class_name}` promotes function `{}` \
                 but its parameter types couldn't be Kotlin-resolved — verify that all \
                 non-opaque parameter types are registered in `kotlin_types`.",
                entry.rust_ident,
            )
        });
        if kind != MethodKind::Instance {
            panic!(
                ".method({}) on `{class_name}`: the function's first parameter doesn't match \
                 the class's Rust type ({promoted_rust_key}) — declare it as `.companion_method(...)` \
                 if it isn't an instance method.",
                entry.rust_ident,
            );
        }
        if !instance_body.is_empty() {
            instance_body.push('\n');
        }
        for line in block.lines() {
            if line.is_empty() {
                instance_body.push('\n');
            } else {
                instance_body.push_str(line);
                instance_body.push('\n');
            }
        }
    }
    for entry in companion_methods {
        let (item_fn, _loc) = registry
            .functions
            .get(&entry.rust_ident)
            .unwrap_or_else(|| {
                panic!(
                    "render_typed_handle_source: `{class_name}` promotes function `{}` \
                 which is not present in `registry.functions` — check the spelling against \
                 the matching `#[prebindgen]` Rust fn name.",
                    entry.rust_ident,
                )
            });
        let (block, _kind) = render_wrapper_fn(
            ext,
            item_fn,
            registry,
            kotlin_types,
            &mut imports,
            None,
            entry.kotlin_name_override.as_deref(),
        )
        .unwrap_or_else(|| {
            panic!(
                "render_typed_handle_source: `{class_name}` promotes function `{}` \
                 but its parameter types couldn't be Kotlin-resolved — verify that all \
                 non-opaque parameter types are registered in `kotlin_types`.",
                entry.rust_ident,
            )
        });
        if !companion_body.is_empty() {
            companion_body.push('\n');
        }
        for line in block.lines() {
            if line.is_empty() {
                companion_body.push('\n');
            } else {
                companion_body.push_str(line);
                companion_body.push('\n');
            }
        }
    }

    // Typed-handle classes emitted into subpackages still need to import
    // the centralized JNINative object for their promoted method bodies.
    if package != ext.package && (!instance_methods.is_empty() || !companion_methods.is_empty()) {
        imports.insert(format!("{}.{}", ext.package, ext.jni_native_class_name()));
    }

    // Imports filtered the same way as render_kotlin_interface — drop
    // entries whose package matches our own (no need to import locals).
    let mut import_list: Vec<String> = imports
        .iter()
        .filter(|fqn| {
            let pkg = fqn.rsplit_once('.').map(|(p, _)| p).unwrap_or("");
            !pkg.is_empty() && pkg != package
        })
        .cloned()
        .collect();
    import_list.sort();
    import_list.dedup();

    let mut s = String::new();
    s.push_str("// Auto-generated by JniGen — do not edit by hand.\n");
    if !package.is_empty() {
        s.push_str(&format!("package {}\n\n", package));
    }
    if !import_list.is_empty() {
        for imp in &import_list {
            s.push_str(&format!("import {}\n", imp));
        }
        s.push('\n');
    }
    let free_extern = ext.mangle_fun("freePtr");
    s.push_str(&format!(
        "/** Typed handle for a native Zenoh `{}`. */\n",
        rust_doc_name
    ));
    // Every typed handle extends the shared `NativeHandle` base, which owns
    // the `@Volatile` pointer slot (`ptr`) and its monitor — that common
    // supertype is what lets `render_wrapper_fn` collect a `List<NativeHandle>`
    // and lock it in one pointer-sorted, deadlock-safe pass. The subclass keeps
    // its own type-specific `close()`/`take()`/`freePtr`.
    s.push_str(&format!(
        "public class {class_name}(initialPtr: Long) : NativeHandle(initialPtr) {{\n",
    ));
    s.push_str("    @Synchronized\n");
    s.push_str("    override fun close() {\n");
    s.push_str("        val p = ptr\n");
    s.push_str("        if (p != 0L) {\n");
    s.push_str("            ptr = 0L\n");
    s.push_str(&format!("            {free_extern}(p)\n"));
    s.push_str("        }\n");
    s.push_str("    }\n\n");
    // Transfer ownership of the native pointer into a fresh handle, leaving
    // this one empty. Lets a callback receiver retain a handle that the
    // framework would otherwise `close()` when the callback returns.
    s.push_str("    @Synchronized\n");
    s.push_str(&format!("    public fun take(): {class_name} {{\n"));
    s.push_str("        val p = ptr\n");
    s.push_str("        ptr = 0L\n");
    s.push_str(&format!("        return {class_name}(p)\n"));
    s.push_str("    }\n");
    if !instance_body.is_empty() {
        s.push('\n');
        for line in instance_body.lines() {
            if line.is_empty() {
                s.push('\n');
            } else {
                s.push_str("    ");
                s.push_str(line);
                s.push('\n');
            }
        }
    }
    // Companion object always exists — at minimum it carries the
    // `@JvmStatic external fun freePtr(ptr: Long)` called by `close()`
    // above. Promoted-method bodies (e.g. typed factory functions) follow.
    s.push('\n');
    s.push_str("    public companion object {\n");
    s.push_str(&format!(
        "        @JvmStatic\n        external fun {free_extern}(ptr: Long)\n",
    ));
    if !companion_body.is_empty() {
        s.push('\n');
        for line in companion_body.lines() {
            if line.is_empty() {
                s.push('\n');
            } else {
                s.push_str("        ");
                s.push_str(line);
                s.push('\n');
            }
        }
    }
    s.push_str("    }\n");
    s.push_str("}\n");
    s
}

/// Emit the package-level wrapper file: one safe top-level wrapper per
/// `#[prebindgen]` fn whose name is in `promoted` (i.e. listed in
/// `package_methods.methods`). Each wrapper delegates to the centralized
/// Native object's matching `external fun`. Opaque-handle parameters
/// (detected via the input converter returning `OwnedObject<T>`) become
/// `NativeHandle`; the wrapper body nests `withPtr` / `consume` per the
/// syntactic shape. Non-opaque parameters pass through with the Kotlin
/// type from `kotlin_types`.
fn render_jni_package_source(
    ext: &JniGen,
    registry: &Registry<KotlinMeta>,
    kotlin_types: &KotlinTypeMap,
    functions: &[MethodEntry],
    package: &str,
) -> String {
    // Start with the auto-derived callback FQNs and let user-provided
    // entries WIN — the user (build.rs) may need to override e.g.
    // `impl Fn(Query)` to point at a hand-written
    // `JNIQueryCallback` instead of the auto-derived default.
    let callback_fqns = ext.collect_kotlin_callback_fqns(registry);
    let mut merged_types = KotlinTypeMap::new();
    for (k, v) in callback_fqns.iter() {
        merged_types = merged_types.add(k, v.clone());
    }
    for (k, v) in kotlin_types.iter() {
        merged_types = merged_types.add(k, v.clone());
    }

    let mut imports: BTreeSet<String> = BTreeSet::new();
    let mut body = String::new();

    for entry in functions {
        let (item_fn, _loc) = registry
            .functions
            .get(&entry.rust_ident)
            .unwrap_or_else(|| {
                panic!(
                    "render_jni_package_source: function `{}` registered via .function(...) is \
                 not in the prebindgen registry — check the spelling against the matching \
                 `#[prebindgen]` Rust fn name.",
                    entry.rust_ident,
                )
            });
        // Top-level wrappers never carry a `promoted_handle`, so the
        // returned [`MethodKind`] is always `Instance` and can be
        // discarded — there is no companion-object emission here.
        if let Some((block, _kind)) = render_wrapper_fn(
            ext,
            item_fn,
            registry,
            &merged_types,
            &mut imports,
            None,
            entry.kotlin_name_override.as_deref(),
        ) {
            body.push_str(&block);
            body.push('\n');
        }
    }

    let mut out = String::new();
    out.push_str("// Auto-generated by JniGen — do not edit by hand.\n");
    if !package.is_empty() {
        out.push_str(&format!("package {}\n\n", package));
    }
    // Exception imports (if any) are added to `imports` by the per-wrapper
    // `@Throws` emission, so no error class is hardcoded here.
    for imp in &imports {
        out.push_str(&format!("import {}\n", imp));
    }
    if !ext.package.is_empty() {
        out.push_str(&format!(
            "import {}.{}\n",
            ext.package,
            ext.jni_native_class_name()
        ));
    }
    out.push('\n');
    out.push_str(&body);
    out
}

/// Render the centralized `internal object <jni_native_class>` holder:
/// one `external fun` per `#[prebindgen]` function, at the JNI **wire**
/// level. Parameter and return types match what the Rust extern
/// receives:
///   * opaque-handle (Borrow/Consume) → jlong → `Long`
///   * `enum_class`                  → jint  → `Int` (call passes `.value`)
///   * `Any` (impl-Into Dispatch)     → JObject → `Any`
///   * everything else                → entry's high-level Kotlin name
/// Opaque returns become `Long`; every other return uses
/// [`classify_return`]'s `kt_return` (Unit is empty string). No `init`
/// block is emitted — the holder stays free of any wrapper-layer
/// reference; the wrapper-layer call sites are responsible for
/// triggering `System.load` before invoking any extern.
fn render_jni_native_source(
    ext: &JniGen,
    registry: &Registry<KotlinMeta>,
    kotlin_types: &KotlinTypeMap,
    declared: &HashSet<syn::Ident>,
    class_name: &str,
) -> String {
    let callback_fqns = ext.collect_kotlin_callback_fqns(registry);
    let mut merged_types = KotlinTypeMap::new();
    for (k, v) in callback_fqns.iter() {
        merged_types = merged_types.add(k, v.clone());
    }
    for (k, v) in kotlin_types.iter() {
        merged_types = merged_types.add(k, v.clone());
    }

    let mut imports: BTreeSet<String> = BTreeSet::new();
    let mut body = String::new();

    let mut idents: Vec<&syn::Ident> = registry.functions.keys().collect();
    idents.sort();
    for ident in idents {
        if !declared.contains(ident) {
            continue;
        }
        let (item_fn, _loc) = &registry.functions[ident];
        if let Some(line) = render_extern_decl(ext, item_fn, registry, &mut imports) {
            body.push_str(&line);
            body.push('\n');
        }
    }

    let mut out = String::new();
    out.push_str("// Auto-generated by JniGen — do not edit by hand.\n");
    if !ext.package.is_empty() {
        out.push_str(&format!("package {}\n\n", ext.package));
    }
    for imp in &imports {
        out.push_str(&format!("import {}\n", imp));
    }
    out.push('\n');
    out.push_str(&format!("internal object {} {{\n", class_name));
    for line in body.lines() {
        if line.is_empty() {
            out.push('\n');
        } else {
            out.push_str("    ");
            out.push_str(line);
            out.push('\n');
        }
    }
    out.push_str("}\n");
    out
}

/// Render one `external fun <mangle_fun(name)>(…): <wire-return>` line
/// at the JNI **wire** level (matches what the Rust extern receives):
///   * opaque-handle (Borrow/Consume) → jlong → `Long`
///   * `enum_class`                  → jint  → `Int` (call passes `.value`)
///   * `Any` (impl-Into Dispatch)     → JObject → `Any`
///   * everything else                → entry's high-level Kotlin name
/// Opaque returns become `Long`; every other return uses
/// [`classify_return`]'s `kt_return` (Unit is empty string).
/// Returns `None` if any parameter's input converter isn't resolved.
pub(crate) fn render_extern_decl(
    ext: &JniGen,
    f: &syn::ItemFn,
    registry: &Registry<KotlinMeta>,
    imports: &mut BTreeSet<String>,
) -> Option<String> {
    use std::fmt::Write;

    let rust_name = f.sig.ident.to_string();
    let kt_name = snake_to_camel(&rust_name);
    let jni_call = ext.mangle_fun(&kt_name);

    let mut params: Vec<(String, String)> = Vec::new();
    for input in &f.sig.inputs {
        let syn::FnArg::Typed(pt) = input else {
            continue;
        };
        let syn::Pat::Ident(pid) = &*pt.pat else {
            continue;
        };
        let name = snake_to_camel(&pid.ident.to_string());
        let arg_ty = &*pt.ty;

        let entry = registry.input_entry(arg_ty)?;

        let is_opaque = converter_returns_owned_object(&entry.function.sig.output);
        // `Option<&Opaque>` crosses the JNI wire as a primitive `jlong`
        // with `0` encoding `None`; nullability lives in the safe wrapper
        // (`withPtrOrZero`) not the JNI extern. Strip the `?` here so the
        // extern signature matches what the JVM will look up. Detection
        // uses `metadata.projection.is_some()` because the `Option<OwnedObject<T>>`
        // converter doesn't return `OwnedObject` directly so the local
        // `is_opaque` flag (which checks the return shape) misses it.
        let is_opt_ref_opaque = entry.metadata.projection.is_some() && is_option_ref(arg_ty);
        let optional = is_option_type(arg_ty) && !is_opt_ref_opaque;

        let kt_type_raw = if is_opaque || is_opt_ref_opaque {
            "Long".to_string()
        } else if ext.is_kotlin_enum(&enum_probe_type(arg_ty)) {
            // Enum (incl. `Option<enum>`) crosses as jint → Kotlin `Int`; the
            // wrapper passes `.value` / `?.value`. The Rust converter unboxes a
            // `java.lang.Integer`, so the extern must declare `Int`/`Int?`, never
            // the enum object.
            "Int".to_string()
        } else {
            entry.metadata.kotlin_name.clone()?
        };
        let short = register_fqn(&kt_type_raw, imports);
        let suffix = if optional { "?" } else { "" };
        params.push((name, format!("{short}{suffix}")));
    }

    let (kt_return, projection) = classify_return(ext, &f.sig.output, registry, imports)?;
    // enum_class returns cross the JNI wire as jint → Kotlin `Int`.
    // The public wrapper converts back using `EnumType.fromInt(Int)`.
    let is_enum_return = return_is_kotlin_enum(ext, &f.sig.output, registry);
    // JNI extern's wire return: handle projections wire as `Long` (the boxed
    // jlong gets wrapped); value-class projections wire as their inner
    // converter's Kotlin type folded through the projection's strategy (the
    // value class is erased to that inner). Enums wire as `Int`; everything
    // else is the declared return.
    let wire_return = match &projection {
        Some(p) => projection_wire_return(ext, registry, p, imports),
        None if is_enum_return => "Int".to_string(),
        None => kt_return,
    };

    let formals = params
        .iter()
        .map(|(n, t)| format!("{n}: {t}"))
        .collect::<Vec<_>>()
        .join(", ");

    let mut line = String::new();
    if wire_return.is_empty() {
        write!(&mut line, "external fun {jni_call}({formals})").ok()?;
    } else {
        write!(
            &mut line,
            "external fun {jni_call}({formals}): {wire_return}"
        )
        .ok()?;
    }
    Some(line)
}

/// Whether a typed-handle-promoted wrapper is emitted as an instance
/// method on the handle class (the first parameter matched the promoted
/// Rust type-key as a literal `&T` / `T`), or inside the class's
/// `companion object` (no param matched, or the candidate was an
/// `impl Into<T>` Dispatch param — those are not eligible for instance
/// promotion even when the inner `T` matches).
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum MethodKind {
    Instance,
    Companion,
}

/// Emit a single wrapper function. Returns `None` if the function has
/// a parameter whose Kotlin type isn't registered (in that case we
/// skip the function rather than panicking — the legacy `JNINative.kt`
/// retains the unwrapped external fun so callers still have an
/// escape hatch).
///
/// When `promoted_handle` is `Some(rust_key)`, the wrapper is emitted
/// as either an instance method or a companion-object method, depending
/// on whether any parameter matches `rust_key`:
///
/// * **Instance** — the first parameter whose Rust type matches
///   `rust_key` (modulo `&T` borrow) is dropped from the signature, and
///   its `withPtr` / `consume` wrapper uses the inherited
///   [`NativeHandle`] scope (no `<name>.` prefix) so the captured
///   `<name>_ptr` is bound in `this`. Every other parameter is emitted
///   exactly as the `JNIWrappers` top-level form.
/// * **Companion** — no parameter matched (e.g. the fn takes no opaque
///   handle of this type, or it takes an `impl Into<T>` Dispatch param
///   whose inner `T` matches the key — those are intentionally **not**
///   promoted to instance methods). The body is emitted exactly as the
///   `JNIWrappers` top-level form (all params, full Dispatch arm tree,
///   no `this` rewrite); the caller is expected to wrap it inside a
///   `companion object { ... }` block on the typed-handle class.
///
/// When `promoted_handle` is `None` (top-level `JNIWrappers` emission),
/// the returned kind is always [`MethodKind::Instance`] (no
/// promotion-shape decision is made) and the caller can ignore it.
fn render_wrapper_fn(
    ext: &JniGen,
    f: &syn::ItemFn,
    registry: &Registry<KotlinMeta>,
    kotlin_types: &KotlinTypeMap,
    imports: &mut BTreeSet<String>,
    promoted_handle: Option<&str>,
    kotlin_name_override: Option<&str>,
) -> Option<(String, MethodKind)> {
    use std::fmt::Write;

    let rust_name = f.sig.ident.to_string();
    // The Kotlin extern in `JNINative` is keyed on the Rust ident
    // (`snake_to_camel(rust_name)` → `ext.mangle_fun`). The per-entry
    // `.name("...")` override only changes the *user-facing* Kotlin
    // wrapper name; the JNI call still has to hit the one extern that
    // the Rust extern actually emits.
    let default_kt_name = snake_to_camel(&rust_name);
    let kt_name = match kotlin_name_override {
        Some(n) => n.to_string(),
        None => default_kt_name.clone(),
    };
    let jni_call = ext.mangle_fun(&default_kt_name);

    // Pre-parse the promoted Rust type-key (if any) so per-param matching
    // is whitespace-normalised against the canonical form.
    let promoted_key: Option<TypeKey> = promoted_handle.map(|s| TypeKey::parse(s));

    // Classify each parameter.
    struct Param {
        kt_name: String,
        kt_type: String,
        mode: ParamMode,
        /// `true` when the param's Rust type is a `enum_class`-declared
        /// enum: the high-level Kotlin signature uses the typed enum
        /// (`Priority`), but the underlying JNI `external fun` declares
        /// the param as `Int` (jint wire). The wrapper bridges the two
        /// by passing `<name>.value` at the call site.
        as_enum_value: bool,
    }
    enum ParamMode {
        Borrow,  // &T opaque-handle → withPtr
        Consume, // T  opaque-handle → consume
        /// `Option<&T>` / `Option<&mut T>` opaque-handle → `withPtrOrZero`.
        /// Nullable typed-handle param; the wrapper runs the body under
        /// the read lock when the handle is non-null and with `0L` when
        /// null. The Rust converter materializes `Option<OwnedObject<T>>`
        /// and the call site uses `.as_deref()` / `.as_deref_mut()`.
        BorrowNullable,
        /// `impl Into<T>` (Kotlin `Any`). At runtime the parameter
        /// fans out into one arm per declared
        /// [`IntoSource`] in `arms`. See
        /// [`DispatchArm`] for the arm shape.
        Dispatch {
            arms: Vec<DispatchArm>,
        },
        PassThrough,
        /// Promoted opaque param: identical lock semantics to
        /// `Borrow` / `Consume` (the inner bool flag chooses), but the
        /// wrapper uses inherited [`NativeHandle`] scope (no
        /// `<name>.` prefix) and the param is omitted from the
        /// Kotlin signature. Set when `promoted_handle` matches.
        PromotedBorrow,
        PromotedConsume,
        /// Value-projection param (`value_class` / `value_blob`): a Kotlin
        /// `@JvmInline value class` that is **not** a lockable handle. The
        /// Kotlin param type is the value-class FQN; the call site passes the
        /// unwrapped inline-class field (`<name>.<field>`) so the `JNINative`
        /// extern receives the erased inner wire (e.g. `ByteArray`). No lock.
        ValueUnwrap {
            field: String,
        },
        /// Promoted non-opaque param (e.g. `&Hello` on a `data_class`
        /// instance method). The Kotlin call site substitutes `this` for
        /// the param name — no lock wrapping needed, and the param is
        /// dropped from the wrapper signature. Set when `promoted_handle`
        /// matches a non-opaque type.
        PromotedPassThrough,
    }

    // Tracks whether we've already consumed the promoted-handle slot —
    // only the first matching param is promoted; any later param of the
    // same Rust type stays as a normal Borrow/Consume.
    let mut promoted_taken = false;

    let mut params: Vec<Param> = Vec::new();
    for input in &f.sig.inputs {
        let syn::FnArg::Typed(pt) = input else {
            continue;
        };
        let syn::Pat::Ident(pid) = &*pt.pat else {
            continue;
        };
        let name = snake_to_camel(&pid.ident.to_string());
        let arg_ty = &*pt.ty;

        // Strip leading reference for the type-map lookup; the registry's
        // input entry is keyed by the param as-written.
        let entry = registry.input_entry(arg_ty)?;
        // Opaque-handle params surface as their typed-handle FQN — every
        // handle is self-contained (its own ptr slot + monitor), so the
        // wrapper body inlines synchronized blocks directly on the typed
        // receiver. Detection flows from the folded `Projection` — present
        // for both `&T` and by-value `T` (the `owned` flag is orthogonal
        // to presence) — so it's the same source of truth the typed-surface
        // emitters use.
        let is_opaque = entry.metadata.projection.is_some();

        // `Option<&T>` / `Option<&mut T>` for opaque T marks the param
        // nullable; the wrapper body branches on null before lock selection.
        let is_opt_ref_opaque = is_opaque && is_option_ref(arg_ty);
        let (kt_type_raw, optional) = if is_opaque {
            let h = entry.metadata.projection.as_ref()?;
            let fqn = ext
                .kotlin_type_fqns
                .iter()
                .find(|(k, _)| k == &h.leaf_key)
                .map(|(_, v)| v.clone())?;
            (fqn, is_opt_ref_opaque)
        } else {
            // Read the Kotlin name straight off the resolved entry's
            // metadata — the rank-N handler that built this converter
            // is also the one that derived the Kotlin name (primitives
            // from `kotlin_for_wire`, wrappers inherit from inner,
            // user-declared decoders from `with_kotlin_name`).
            let kt = entry.metadata.kotlin_name.clone()?;
            let opt = is_option_type(arg_ty);
            (kt, opt)
        };

        // Does this param match the promoted handle's Rust type?
        // Strip a leading `&` before comparing; the registered type-key
        // is the bare-name form (e.g. `Publisher < 'static >`).
        let matches_promoted = if !promoted_taken {
            if let Some(pk) = &promoted_key {
                let arg_no_ref: syn::Type = match arg_ty {
                    syn::Type::Reference(r) => (*r.elem).clone(),
                    _ => arg_ty.clone(),
                };
                TypeKey::from_type(&arg_no_ref) == *pk
            } else {
                false
            }
        } else {
            false
        };

        // A projection can be a lockable opaque **handle** or a non-lockable
        // **value projection** (`value_class` / `value_blob` — an inline value
        // class). Only handles participate in the lock scaffold and pass a
        // `_ptr`; value projections pass their unwrapped inner field.
        let proj_kind = entry
            .metadata
            .projection
            .as_ref()
            .map(|p| p.kind.clone());
        let is_handle = matches!(
            proj_kind,
            Some(crate::api::lang::jnigen::jni::jni_ext::ProjectionKind::Handle)
        );
        let is_value_proj = matches!(
            proj_kind,
            Some(crate::api::lang::jnigen::jni::jni_ext::ProjectionKind::ValueClass)
                | Some(crate::api::lang::jnigen::jni::jni_ext::ProjectionKind::ValueBlob)
        );

        // Mode: handle → Borrow/Consume by Rust syntactic shape (locked).
        // Value projection → ValueUnwrap (inline-class field, no lock).
        // Non-projection + `Any` triggers Dispatch — one arm per declared
        // `IntoSource`. Everything else (primitives, callbacks, data
        // classes) passes through. Promoted variants kick in when this
        // param is the matched-and-not-yet-consumed handle slot.
        let mode = if is_handle {
            let borrow = matches!(arg_ty, syn::Type::Reference(_));
            if is_opt_ref_opaque {
                // Nullable borrow — promoted form not supported (the
                // receiver can't be null).
                ParamMode::BorrowNullable
            } else if matches_promoted {
                promoted_taken = true;
                if borrow {
                    ParamMode::PromotedBorrow
                } else {
                    ParamMode::PromotedConsume
                }
            } else if borrow {
                ParamMode::Borrow
            } else {
                ParamMode::Consume
            }
        } else if is_value_proj {
            // `@JvmInline value class` param: pass `<name>.<field>` (the erased
            // inner) to the extern, no lock. Only the Direct shape (`T` / `&T`,
            // non-`Option`, non-`Vec`) is supported for now — the only shape
            // any value-projection param currently takes.
            if is_option_type(arg_ty) {
                panic!(
                    "render_wrapper_fn: value-class/value-blob `Option<_>` / `Vec<_>` params \
                     aren't supported yet (param `{name}`); only a direct `T` / `&T` value \
                     projection param is handled."
                );
            }
            let field = crate::api::lang::jnigen::jni::jni_ext::value_projection_field(
                ext, registry, arg_ty,
            )
            .unwrap_or_else(|| {
                panic!(
                    "render_wrapper_fn: cannot determine inline-class field for value \
                     projection param `{name}`"
                )
            });
            ParamMode::ValueUnwrap { field }
        } else if matches_promoted {
            // Non-opaque (data/value/enum class) instance-method param:
            // drop from the Kotlin signature, substitute `this` at the
            // JNI call site. No lock semantics — the JNI side decodes the
            // Kotlin instance directly (struct decoder via jobject field
            // reflection, value-class projection, enum `.value` etc.).
            promoted_taken = true;
            ParamMode::PromotedPassThrough
        } else if kt_type_raw == "Any" {
            let sources = entry.into_sources.as_deref().unwrap_or(&[]);
            ParamMode::Dispatch {
                arms: build_dispatch_arms(sources, registry, kotlin_types, imports),
            }
        } else {
            ParamMode::PassThrough
        };

        let short = register_fqn(&kt_type_raw, imports);
        let suffix = if optional { "?" } else { "" };
        // Strip a leading `&` before the enum check — the `&Priority`
        // input converter shares Priority's converter (see the rank-1
        // `& _` arm), and the same `.value` projection applies either
        // way at the call site.
        // Detect enums through a leading `&` and through `Option<…>`, so a
        // nullable enum param passes `?.value` to the (Int?-typed) extern.
        let as_enum_value = ext.is_kotlin_enum(&enum_probe_type(arg_ty));
        params.push(Param {
            kt_name: name,
            kt_type: format!("{short}{suffix}"),
            mode,
            as_enum_value,
        });
    }

    // A promoted-handle was requested but never matched any param —
    // emit as a companion-object method instead of panicking. `.method(...)`
    // is a namespace declaration ("this fn lives on the typed-handle
    // class"), and the generator chooses between an instance method and
    // a companion-object method based on whether any param matched.
    let kind = if promoted_handle.is_some() && !promoted_taken {
        MethodKind::Companion
    } else {
        MethodKind::Instance
    };

    // Return type: peel ZResult<...>; detect projection return (opaque
    // handle or value class). `projection` carries the folded fold
    // strategy + kind the wrap emission and JNINative wire-return code
    // branch on.
    let (kt_return, projection) = classify_return(ext, &f.sig.output, registry, imports)?;
    // enum_class returns cross the JNI wire as jint → Kotlin `Int`.
    // Detect this so `build_call` can wrap the result with `fromInt`.
    let is_enum_return = return_is_kotlin_enum(ext, &f.sig.output, registry);

    // Indices of Dispatch-mode params.
    let dispatch_indices: Vec<usize> = params
        .iter()
        .enumerate()
        .filter_map(|(i, p)| matches!(p.mode, ParamMode::Dispatch { .. }).then_some(i))
        .collect();

    // Build the JNINative call for a given per-Dispatch arm selection.
    // `arm_choice[k]` is the index into the arms list for
    // `dispatch_indices[k]`; values are interpreted by the arm itself
    // — `Unwrap` arms pass `<name>_ptr`, every other arm passes the
    // raw `<name>` (typed handle or non-handle value, untouched).
    let build_call = |arm_choice: &[usize]| -> String {
        let mut args: Vec<String> = Vec::with_capacity(params.len());
        for (i, p) in params.iter().enumerate() {
            let arg = match &p.mode {
                ParamMode::Borrow
                | ParamMode::Consume
                | ParamMode::BorrowNullable
                | ParamMode::PromotedBorrow
                | ParamMode::PromotedConsume => format!("{}_ptr", p.kt_name),
                ParamMode::PromotedPassThrough => {
                    if p.as_enum_value {
                        "this.value".to_string()
                    } else {
                        "this".to_string()
                    }
                }
                ParamMode::Dispatch { arms } => {
                    let pos = dispatch_indices.iter().position(|&di| di == i).unwrap();
                    let arm = &arms[arm_choice[pos]];
                    if arm.unwrap_to_ptr {
                        format!("{}_ptr", p.kt_name)
                    } else {
                        p.kt_name.clone()
                    }
                }
                ParamMode::ValueUnwrap { field } => {
                    // Inline value class → pass its erased inner field to the
                    // extern (e.g. `z.bytes`: a `ByteArray`).
                    format!("{}.{}", p.kt_name, field)
                }
                ParamMode::PassThrough => {
                    if p.as_enum_value {
                        // Enum → its `Int` discriminant for the extern. Nullable
                        // enum (`Enum?`) uses `?.value` so it stays `Int?`.
                        if p.kt_type.ends_with('?') {
                            format!("{}?.value", p.kt_name)
                        } else {
                            format!("{}.value", p.kt_name)
                        }
                    } else {
                        p.kt_name.clone()
                    }
                }
            };
            args.push(arg);
        }
        let mut call = format!(
            "{}.{jni_call}({})",
            ext.jni_native_class_name(),
            args.join(", ")
        );
        if let Some(p) = &projection {
            // Fold the wrap through the projection strategy. The wrap class is
            // the projection leaf's typed short name (Handle's typed-handle
            // class or value-class wrapper). The sentinel is the Kotlin
            // null-representation literal for the leaf wire — used only by
            // the `Niche+primitive` arm of `fold_projection_wrap`.
            let leaf_fqn = ext
                .kotlin_type_fqns
                .iter()
                .find(|(k, _)| k == &p.leaf_key)
                .map(|(_, v)| v.as_str())
                .unwrap_or(&p.leaf_key);
            let short = leaf_fqn.rsplit('.').next().unwrap_or(leaf_fqn).to_string();
            let sentinel = projection_leaf_sentinel(ext, registry, p);
            call = fold_projection_wrap(&p.strategy, &call, &short, sentinel.as_deref());
        } else if is_enum_return {
            call = format!("{kt_return}.fromInt({call})");
        }
        call
    };

    // Recurse over Dispatch params; at each level enumerate arms.
    fn build_tree(
        level: usize,
        choice: &mut Vec<usize>,
        dispatch_indices: &[usize],
        params: &[Param],
        build_call: &dyn Fn(&[usize]) -> String,
    ) -> String {
        if level == dispatch_indices.len() {
            return build_call(choice);
        }
        let pi = dispatch_indices[level];
        let arms = match &params[pi].mode {
            ParamMode::Dispatch { arms } => arms,
            _ => unreachable!("dispatch_indices points only at Dispatch params"),
        };
        let name = &params[pi].kt_name;

        // Emit the if/else-if chain over arms, with the final
        // `else` carrying the unconditional-pass-through branch.
        let mut out = String::new();
        for (k, arm) in arms.iter().enumerate() {
            choice.push(k);
            let inner = build_tree(level + 1, choice, dispatch_indices, params, build_call);
            choice.pop();
            // Locking is no longer done here — every handle this call touches
            // (including a dispatch value that turns out to be a handle) is
            // locked once, in pointer-sorted order, by the outer
            // `withSortedHandleLocks` (see the scaffold below). These arms only
            // select the per-source marshalling; an `unwrap_to_ptr` arm still
            // reads `<name>_ptr` here, which is safe because the handle's
            // monitor is already held by the enclosing scaffold. Catch-all
            // (non-opaque) arms just inline.
            let arm_body = match &arm.runtime_check {
                Some(check) => {
                    let ptr_bind = if arm.unwrap_to_ptr {
                        format!(
                            "val {name}_ptr = {name}.ptr\n    if ({name}_ptr == 0L) throw JniBindingError(\"Operation on a closed native handle.\")\n    "
                        )
                    } else {
                        String::new()
                    };
                    format!(
                        "{prefix} ({name} is {check}) {{\n    {ptr_bind}{inner}\n}}",
                        prefix = if k == 0 { "if" } else { " else if" },
                    )
                }
                None => {
                    // Catch-all else branch (no runtime check).
                    if k == 0 {
                        // Single unconditional arm (no opaque sources
                        // declared) — skip the if/else scaffolding.
                        inner.clone()
                    } else {
                        format!(" else {{\n    {inner}\n}}")
                    }
                }
            };
            out.push_str(&arm_body);
        }
        out
    }

    // The per-`impl Into` dispatch `is`-tree now only matters when an arm
    // marshals differently from the others (an `unwrap_to_ptr` arm passing a
    // `<name>_ptr`); locking moved to the unified scaffold. When no arm
    // unwraps, every branch makes the identical call, so collapse the tree to
    // a single call instead of emitting 2ⁿ copies.
    let any_unwrap_arm = params.iter().any(|p| {
        matches!(&p.mode, ParamMode::Dispatch { arms }
            if arms.iter().any(|a| a.unwrap_to_ptr))
    });
    let body_expr = if !dispatch_indices.is_empty() && any_unwrap_arm {
        let mut choice: Vec<usize> = Vec::with_capacity(dispatch_indices.len());
        build_tree(0, &mut choice, &dispatch_indices, &params, &build_call)
    } else {
        build_call(&vec![0; dispatch_indices.len()])
    };

    // Collect the opaque-handle params so we can scaffold pointer-ordered
    // synchronized blocks around them. Dispatch params handle their own
    // lock acquisition inside `build_tree` and are excluded here.
    struct Opaque {
        /// Kotlin param name (e.g. `"b"`). For promoted params this is
        /// the receiver param's name (matches `<name>_ptr` references in
        /// `body_expr`), but the `target` below is `"this"`.
        name: String,
        /// Object to synchronize on and read the pointer from
        /// (`"this"` for promoted, `<name>` otherwise).
        target: String,
        /// Statement that nulls the pointer slot after consume
        /// (`"<target>.ptr = 0L"`), or `None` for borrow modes.
        consume_null: Option<String>,
        /// `true` for `Option<&T>` — nullable param, branches before lock.
        nullable: bool,
    }
    let opaques: Vec<Opaque> = params
        .iter()
        .filter_map(|p| {
            let (target, consume_null, nullable) = match p.mode {
                ParamMode::Borrow => (p.kt_name.clone(), None, false),
                ParamMode::Consume => (
                    p.kt_name.clone(),
                    Some(format!("{}.ptr = 0L", p.kt_name)),
                    false,
                ),
                ParamMode::BorrowNullable => (p.kt_name.clone(), None, true),
                ParamMode::PromotedBorrow => ("this".to_string(), None, false),
                ParamMode::PromotedConsume => {
                    ("this".to_string(), Some("ptr = 0L".to_string()), false)
                }
                _ => return None,
            };
            Some(Opaque {
                name: p.kt_name.clone(),
                target,
                consume_null,
                nullable,
            })
        })
        .collect();

    let is_unit = kt_return.is_empty();
    let throw_stmt = "throw JniBindingError(\"Operation on a closed native handle.\")";

    // Dispatch (`impl Into<T>`) params that can resolve to an opaque handle at
    // runtime (at least one `IntoSource` arm is an opaque-handle source). When
    // such a param's runtime value is a handle it joins the same sorted lock
    // set as the statically-typed handle params; `as? NativeHandle` captures
    // exactly those arms (handle sources are `NativeHandle`s, non-opaque
    // sources like `String`/`ByteArray` are not).
    let dispatch_lockables: Vec<&Param> = params
        .iter()
        .filter(|p| {
            matches!(&p.mode, ParamMode::Dispatch { arms }
                if arms.iter().any(|a| a.lock_qual.is_some()))
        })
        .collect();

    // Unified, deadlock-safe N-ary handle locking. Gather every live handle
    // monitor the call touches — concrete `&T`/`T`/promoted params, non-null
    // `Option<&T>`, and dispatch values that turn out to be handles — then
    // acquire them all in one pointer-sorted pass via `withSortedHandleLocks`
    // before reading any `ptr`. Replaces both the old per-arity (0/1/2, with a
    // `>2` panic) pointer-ordered scaffold and the separate per-dispatch-arm
    // `synchronized` nesting with a single mechanism that scales to any arity
    // and puts the dispatch handles under the same global lock order.
    let scaffold_body: Option<String> = if opaques.is_empty() && dispatch_lockables.is_empty() {
        None
    } else {
        // Base class + helper live in `ext.package` (the import is filtered
        // out when this file already is that package).
        if !ext.package.is_empty() {
            imports.insert(format!("{}.NativeHandle", ext.package));
            imports.insert(format!("{}.withSortedHandleLocks", ext.package));
        }
        let mut adds = String::new();
        let mut ptr_binds = String::new();
        for o in &opaques {
            if o.nullable {
                // `Option<&T>`: lock + read only when present; null → ptr 0.
                adds.push_str(&format!("{n}?.let {{ __locks.add(it) }}\n", n = o.name));
                ptr_binds.push_str(&format!(
                    "val {n}_ptr = {t}?.ptr ?: 0L\nif ({n} != null && {n}_ptr == 0L) {throw_stmt}\n",
                    n = o.name,
                    t = o.target,
                ));
            } else {
                adds.push_str(&format!("__locks.add({t})\n", t = o.target));
                ptr_binds.push_str(&format!(
                    "val {n}_ptr = {t}.ptr\nif ({n}_ptr == 0L) {throw_stmt}\n",
                    n = o.name,
                    t = o.target,
                ));
            }
        }
        for p in &dispatch_lockables {
            adds.push_str(&format!(
                "({n} as? NativeHandle)?.let {{ __locks.add(it) }}\n",
                n = p.kt_name,
            ));
        }
        // Consume-mode handles null their pointer slot after the call, in a
        // `finally` so the slot is invalidated even when the JNI call throws.
        let consume_stmts: Vec<&str> = opaques
            .iter()
            .filter_map(|o| o.consume_null.as_deref())
            .collect();
        let value_expr = if consume_stmts.is_empty() {
            body_expr.clone()
        } else {
            format!(
                "try {{\n{body_expr}\n}} finally {{\n{}\n}}",
                consume_stmts.join("\n")
            )
        };
        // The lambda's last expression is the call's value; the method
        // `return`s the helper's result (or ignores it for Unit).
        // `withSortedHandleLocks` is a plain (recursive, non-inline) fn, so the
        // body is value-returning rather than using a non-local `return`.
        let ret = if is_unit { "" } else { "return " };
        Some(format!(
            "val __locks = ArrayList<NativeHandle>()\n{adds}{ret}withSortedHandleLocks(__locks) {{\n{ptr_binds}{value_expr}\n}}"
        ))
    };

    let _ = ext; // ext no longer needed here — throws comes from registry metadata
    let mut out = String::new();
    // `@Throws` is the UNION of every stage every converter the wrapper
    // drives can raise:
    //   * each input parameter's wire-facing converter (its `?` failure
    //     raises the metadata `throws` exception — framework
    //     `JniBindingError` by default, or a custom one bound via
    //     `Some(parse_quote!(<full path>))` in the input wrapper's
    //     closure);
    //   * each pre_stage on that input's chain (value-inspecting throw
    //     stages — an `input_wrapper` / `output_wrapper` whose closure
    //     returns a rust type with `Some(parse_quote!(<full path>))`
    //     and gets composed onto that type's converter);
    //   * the return type's output converter and its pre_stages
    //     (likewise).
    // Collected into a `BTreeSet` so the emitted annotation is sorted and
    // deterministic; stages/converters with no `throws` metadata
    // contribute nothing.
    let mut throws_fqns: BTreeSet<String> = BTreeSet::new();
    for input in &f.sig.inputs {
        let syn::FnArg::Typed(pt) = input else {
            continue;
        };
        let arg_ty = &*pt.ty;
        if let Some(entry) = registry.input_entry(arg_ty) {
            if let Some(fqn) = entry.metadata.throws.clone() {
                throws_fqns.insert(fqn);
            }
            for stage in &entry.pre_stages {
                if let Some(fqn) = stage.metadata.throws.clone() {
                    throws_fqns.insert(fqn);
                }
            }
        }
    }
    let return_ty: syn::Type = match &f.sig.output {
        syn::ReturnType::Default => syn::parse_quote!(()),
        syn::ReturnType::Type(_, ty) => (**ty).clone(),
    };
    if let Some(entry) = registry.output_entry(&return_ty) {
        if let Some(fqn) = entry.metadata.throws.clone() {
            throws_fqns.insert(fqn);
        }
        for stage in &entry.pre_stages {
            if let Some(fqn) = stage.metadata.throws.clone() {
                throws_fqns.insert(fqn);
            }
        }
    }
    if !throws_fqns.is_empty() {
        let parts: Vec<String> = throws_fqns
            .iter()
            .map(|fqn| format!("{}::class", register_fqn(fqn, imports)))
            .collect();
        let _ = writeln!(out, "@Throws({})", parts.join(", "));
    }
    let param_list: Vec<String> = params
        .iter()
        .filter(|p| {
            !matches!(
                p.mode,
                ParamMode::PromotedBorrow
                    | ParamMode::PromotedConsume
                    | ParamMode::PromotedPassThrough
            )
        })
        .map(|p| format!("{}: {}", p.kt_name, p.kt_type))
        .collect();
    let _ = write!(out, "public fun {kt_name}({})", param_list.join(", "));
    if !kt_return.is_empty() {
        let _ = write!(out, ": {kt_return}");
    }
    match &scaffold_body {
        None => {
            // Pure expression body (no opaque handles to lock).
            let _ = writeln!(out, " =");
            let _ = writeln!(out, "    {body_expr}");
        }
        Some(body) => {
            // Block body — indent every line of the scaffold by four spaces.
            let _ = writeln!(out, " {{");
            for line in body.lines() {
                if line.is_empty() {
                    out.push('\n');
                } else {
                    let _ = writeln!(out, "    {line}");
                }
            }
            let _ = writeln!(out, "}}");
        }
    }
    Some((out, kind))
}

/// One arm of an `impl Into<T>` parameter's Java-side dispatch tree.
/// Produced by [`build_dispatch_arms`] from the
/// [`IntoSource`] list the resolver stored on
/// `TypeEntry::into_sources`.
struct DispatchArm {
    /// `is <KotlinShortName>` check, or `None` for the unconditional
    /// catch-all arm placed last. Examples:
    /// * `Some("JNISession")` — typed-FQN arm; the JNI dispatcher's
    ///   matching arm does `instanceof io/zenoh/jni/JNISession` and
    ///   reads the pointer via `.peek()`. Kotlin holds the lock via
    ///   `.<lock_qual>` and passes the typed handle to JNI unchanged.
    /// * `Some("NativeHandle")` — generic opaque catch-all for
    ///   sources whose Kotlin class isn't registered as a typed FQN.
    ///   The JNI dispatcher's matching arm does `instanceof
    ///   java.lang.Long` and reads the autoboxed long via
    ///   `longValue()`; Kotlin unwraps to `Long` via
    ///   `.<lock_qual> { ptr -> ... }` and passes `ptr` (autoboxed).
    /// * `None` — final else; emits the JNI call unconditionally on
    ///   the raw `Any` parameter. Covers non-opaque source kinds
    ///   (e.g. `String`, `Int`) whose JNI side does its own
    ///   per-class `instanceof` checks downstream of the wire.
    runtime_check: Option<String>,
    /// `withPtr` / `consume` — scope qualifier on the typed handle
    /// (`is NativeHandle` arms only). `None` for the non-handle
    /// catch-all (no lock to acquire).
    lock_qual: Option<&'static str>,
    /// `true` → JNI receives `<name>_ptr` (the `Long` extracted by
    /// `.withPtr`/`.consume`, autoboxed to `java.lang.Long`).
    /// `false` → JNI receives the parameter as-is (typed handle for
    /// typed-FQN arms, raw value for the catch-all). The two cases
    /// pair with the JNI-side `instanceof` shape — `java.lang.Long`
    /// vs typed FQN vs whatever non-opaque source class.
    unwrap_to_ptr: bool,
}

/// Translate the resolver-recorded `IntoSource` list into the Kotlin
/// emit's per-arm dispatch shape. Arm ordering matters: typed-FQN
/// opaque arms come first (so they aren't swallowed by the catch-all
/// else), then the final unconditional `else` branch handling every
/// non-opaque source class (`String`, primitives, etc.) — the JNI
/// dispatcher does its own `instanceof` chain on the wire side for
/// those.
///
/// Opaque sources without a typed FQN are a build-time error in
/// `jobject_to_wire_adapter` (it panics with a registration hint),
/// so this helper never has to emit a generic `is NativeHandle`
/// fallback arm — every opaque source is either typed or rejected.
fn build_dispatch_arms(
    sources: &[IntoSource],
    registry: &Registry<KotlinMeta>,
    kotlin_types: &KotlinTypeMap,
    imports: &mut BTreeSet<String>,
) -> Vec<DispatchArm> {
    let mut typed: Vec<DispatchArm> = Vec::new();
    for src in sources {
        let canon = TypeKey::from_type(&src.source_type).as_str().to_string();
        // Only opaque sources need an `is <KotlinClass>` arm; the rest
        // (String, primitives) fall through to the catch-all else
        // where the JNI dispatcher's own per-class `instanceof` chain
        // takes over.
        let is_opaque = registry
            .input_entry(&src.source_type)
            .map(|e| converter_returns_owned_object(&e.function.sig.output))
            .unwrap_or(false);
        if !is_opaque {
            continue;
        }
        let qual: &'static str = match src.mode {
            IntoSourceMode::Borrow => "withPtr",
            IntoSourceMode::Consume => "consume",
        };
        let fqn = kotlin_types.lookup(&canon).unwrap_or_else(|| {
            panic!(
                "build_dispatch_arms: opaque source `{}` has no Kotlin FQN registered \
                 — register one via `JniGen::kotlin_type_fqn(...)` and ensure the \
                 corresponding Kotlin class exists.",
                canon
            )
        });
        let short = register_fqn(fqn, imports);
        typed.push(DispatchArm {
            runtime_check: Some(short),
            lock_qual: Some(qual),
            unwrap_to_ptr: false,
        });
    }

    let mut arms = typed;
    // Final unconditional else — JNI dispatcher's own `instanceof`
    // chain handles non-opaque source classes (String, etc.).
    arms.push(DispatchArm {
        runtime_check: None,
        lock_qual: None,
        unwrap_to_ptr: false,
    });
    arms
}

/// Fall-back Kotlin type derived directly from the JNI wire type.
/// Returns the **non-nullable** Kotlin base name — the use site adds
/// a `?` suffix when the entry's Rust type is `Option<…>` (via
/// [`is_option_type`]), so this helper must not double up.
pub(crate) fn kotlin_for_wire(wire: &syn::Type) -> Option<String> {
    if let syn::Type::Path(tp) = wire {
        if let Some(last) = tp.path.segments.last() {
            let name = last.ident.to_string();
            let kt = match name.as_str() {
                "jboolean" => "Boolean",
                "jbyte" => "Byte",
                "jchar" => "Char",
                "jshort" => "Short",
                "jint" => "Int",
                "jlong" => "Long",
                "jfloat" => "Float",
                "jdouble" => "Double",
                "JString" | "jstring" => "String",
                "JByteArray" | "jbyteArray" => "ByteArray",
                "JObject" | "jobject" => "Any",
                "JClass" => "Any",
                _ => return None,
            };
            return Some(kt.to_string());
        }
    }
    None
}

/// Returns `(kt_return, projection)` where:
/// * `kt_return` is the declared Kotlin return type written in the
///   wrapper's signature (empty for `Unit`).
/// * `projection` is `Some(Projection)` when the return is a Kotlin newtype
///   (opaque handle or value class) reached through 0+ wrappers. The
///   wrapper body uses it to fold the wrap call (`W(x)` for `Direct`,
///   `?.let { W(it) }` for `Nullable`, `.map { W(it) }` for `Iterable`)
///   and pick the JNI extern's wire return (`Long` for `Handle`,
///   the inner wire's Kotlin name for `ValueClass`). `None` for plain
///   non-projection returns.
fn classify_return(
    ext: &JniGen,
    output: &syn::ReturnType,
    registry: &Registry<KotlinMeta>,
    imports: &mut BTreeSet<String>,
) -> Option<(
    String,
    Option<crate::api::lang::jnigen::jni::jni_ext::Projection>,
)> {
    let ty = match output {
        syn::ReturnType::Default => return Some((String::new(), None)),
        syn::ReturnType::Type(_, t) => &**t,
    };
    let outer_meta = registry.output_entry(ty).map(|e| e.metadata.clone());
    // Unit returns (incl. `ZResult<()>`, whose inner identity rides
    // `value_rust_key`) declare no Kotlin return type.
    let inner_canon = outer_meta
        .as_ref()
        .and_then(|m| m.value_rust_key.clone())
        .unwrap_or_else(|| ty.to_token_stream().to_string());
    let inner: syn::Type = syn::parse_str(&inner_canon).unwrap_or_else(|_| ty.clone());
    if crate::api::lang::jnigen::util::is_unit(&inner) {
        return Some((String::new(), None));
    }
    // Projection return (opaque handle or value class): read the folded
    // `Projection` the type-unfolding mechanism propagated onto this return
    // type's converter metadata — one source of truth, no shape-specific
    // peeling. The declared return type is the concrete projection class
    // folded through `Nullable`/`Iterable`; callers fold the wrap and pick
    // the wire return based on `kind`.
    if let Some(h) = outer_meta.as_ref().and_then(|m| m.projection.clone()) {
        let fqn = ext
            .kotlin_type_fqns
            .iter()
            .find(|(k, _)| k == &h.leaf_key)
            .map(|(_, v)| v.clone())
            .unwrap_or_else(|| {
                panic!(
                    "classify_return: projection return type `{}` has no Kotlin FQN registered \
                     — every opaque/value class must be declared via `JniGen::ptr_class(...)` \
                     / `JniGen::value_class(...)`.",
                    h.leaf_key
                )
            });
        let short = register_fqn(&fqn, imports);
        return Some((render_handle_type(&h.strategy, &short), Some(h)));
    }
    // Non-opaque: read the Kotlin name straight off the resolved
    // output entry's metadata — the rank-N handler propagates
    // `ZResult<T>` / `Option<T>` / `Vec<T>` derivations alongside the
    // wire, so no peel-and-fallback chain is needed at the use site.
    if let Some(out_entry) = registry.output_entry(ty) {
        if let Some(kt) = out_entry.metadata.kotlin_name.clone() {
            return Some((register_fqn(&kt, imports), None));
        }
    }
    None
}

/// Returns `true` when the function's return type resolves to a type registered
/// via [`JniGen::enum_class`]. Enum returns cross the JNI wire as `jint` (Kotlin
/// `Int`); the public wrapper must call `EnumType.fromInt(Int)` to convert back.
fn return_is_kotlin_enum(
    ext: &JniGen,
    output: &syn::ReturnType,
    registry: &Registry<KotlinMeta>,
) -> bool {
    let ty = match output {
        syn::ReturnType::Default => return false,
        syn::ReturnType::Type(_, t) => &**t,
    };
    let outer_meta = registry.output_entry(ty).map(|e| e.metadata.clone());
    let inner_canon = outer_meta
        .as_ref()
        .and_then(|m| m.value_rust_key.clone())
        .unwrap_or_else(|| ty.to_token_stream().to_string());
    let inner: syn::Type = syn::parse_str(&inner_canon).unwrap_or_else(|_| ty.clone());
    ext.is_kotlin_enum(&inner)
}

fn snake_to_camel(s: &str) -> String {
    let mut out = String::new();
    let mut upper = false;
    for c in s.chars() {
        if c == '_' {
            upper = true;
        } else if upper {
            out.push(c.to_ascii_uppercase());
            upper = false;
        } else {
            out.push(c);
        }
    }
    out
}
