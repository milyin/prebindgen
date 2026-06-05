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

use super::*;

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
pub(crate) fn rust_key_for_fqn<'a>(ext: &'a JniGen, fqn: &str) -> Option<&'a str> {
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
        written.push(self.write_jni_native(registry, kotlin_root)?);
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
        // The N-ary locking helper is only referenced when wrappers are
        // emitted with locking on; skip it under `handle_locks(false)` so it
        // doesn't surface as an unused-`internal fun` warning.
        if self.emit_handle_locks {
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
            // Allocation-free fixed-arity overloads for the common cases (1–3
            // statically-known, non-null handles). `inline` folds both the
            // helper and [body] into the call site — no `ArrayList`, no
            // `sortedBy`, no recursion, no lambda object. The ordering key is
            // `ptr` ascending, IDENTICAL to the `List` overload above, so the
            // global lock order is consistent whichever overload a wrapper
            // uses — deadlock-freedom is preserved even across paths.
            s.push_str(
                "/** Allocation-free single-handle lock (one monitor, nothing to order). */\n\
                internal inline fun <R> withSortedHandleLocks(a: NativeHandle, body: () -> R): R =\n\
                \x20   synchronized(a) { body() }\n\
                /** Allocation-free two-handle lock: order by `ptr` then nest monitors. */\n\
                internal inline fun <R> withSortedHandleLocks(\n\
                \x20   a: NativeHandle,\n\
                \x20   b: NativeHandle,\n\
                \x20   body: () -> R,\n\
                ): R {\n\
                \x20   val first: NativeHandle\n\
                \x20   val second: NativeHandle\n\
                \x20   if (a.ptr <= b.ptr) { first = a; second = b } else { first = b; second = a }\n\
                \x20   return synchronized(first) { synchronized(second) { body() } }\n\
                }\n\
                /** Allocation-free three-handle lock: 3-compare sorting network, then nest. */\n\
                internal inline fun <R> withSortedHandleLocks(\n\
                \x20   a: NativeHandle,\n\
                \x20   b: NativeHandle,\n\
                \x20   c: NativeHandle,\n\
                \x20   body: () -> R,\n\
                ): R {\n\
                \x20   var x = a\n\
                \x20   var y = b\n\
                \x20   var z = c\n\
                \x20   if (x.ptr > y.ptr) { val t = x; x = y; y = t }\n\
                \x20   if (y.ptr > z.ptr) { val t = y; y = z; z = t }\n\
                \x20   if (x.ptr > y.ptr) { val t = x; x = y; y = t }\n\
                \x20   return synchronized(x) { synchronized(y) { synchronized(z) { body() } } }\n\
                }\n",
            );
        }
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

    /// Build a `KotlinTypeMap` with the registry's callback FQNs prepended
    /// onto `base` (callback entries first, `base` entries win on conflict).
    /// Single home for the "merge callbacks onto a type map" step the Kotlin
    /// emitters used to open-code at five call sites.
    pub(crate) fn merged_kotlin_type_map(
        &self,
        registry: &Registry<KotlinMeta>,
        base: &KotlinTypeMap,
    ) -> KotlinTypeMap {
        let callback_fqns = self.collect_kotlin_callback_fqns(registry);
        let mut map = KotlinTypeMap::new();
        for (k, v) in callback_fqns.iter() {
            map = map.add(k, v.clone());
        }
        for (k, v) in base.iter() {
            map = map.add(k, v.clone());
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
        let kotlin_types = self.merged_kotlin_type_map(registry, &self.build_kotlin_type_map());
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
        let kotlin_types = self.merged_kotlin_type_map(registry, &self.build_kotlin_type_map());
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
        pkg_cfg: &crate::api::lang::jnigen::jni::PackageConfig,
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
        output_dir: &Path,
    ) -> Result<PathBuf, WriteKotlinError> {
        let class_name = self.jni_native_class_name();
        let declared = self.declared_functions();
        let contents = render_jni_native_source(self, registry, &declared, &class_name);
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
        let merged_types = self.merged_kotlin_type_map(registry, kotlin_types);

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

pub(crate) fn build_callback_kotlin_file(
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
        // Data-class arg: flatten into leaf params (mirror of the native
        // `flatten_struct_encode` that fills the `run` call), so the callback
        // receives the struct's fields directly — no built `jni.<Struct>`
        // object crosses the boundary, and the consumer constructs whatever it
        // wants from the flat params. Prefix `p{i}` matches the native order.
        if let Some(st) =
            crate::api::lang::jnigen::jni::callback_arg_data_class(ext, registry, arg)
        {
            let prefix = format!("p{i}");
            if let Some((flat_params, _reconstruct)) =
                flatten_struct_factory(ext, registry, &st, &prefix, "", &mut used_fqns, 0)
            {
                for (name, ty) in &flat_params {
                    params.push(format!("        {name}: {ty},"));
                }
                continue;
            }
        }
        let entry = registry.output_entry(arg);
        // Opaque-handle args: the converter's value-name is `"Long"`, but the
        // callback delivers the typed handle class. Prefer its registered FQN
        // so the param type — and the file's import — resolve to e.g.
        // `io.zenoh.jni.scouting.ZHello`.
        let kotlin_ty = entry
            .and_then(|e| e.metadata.projection.as_ref())
            .map(|h| crate::api::lang::jnigen::jni::handle_field_fqn(ext, h))
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
