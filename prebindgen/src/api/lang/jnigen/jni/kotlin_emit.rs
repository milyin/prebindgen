//! `KotlinExt` impl for [`JniGen`].
//!
//! [`JniGen::write_kotlin`] is the single entry point for every Kotlin
//! file the JNI back-end emits. Each per-kind emitter builds in-memory
//! [`kt::KtFile`] *model fragments* (declarations, not strings — the
//! generator module `api::gen::kotlin` owns formatting and imports):
//!   * the shared `NativeHandle` base + lock helpers (root package, e.g.
//!     `io.zenoh.jni`).
//!   * one typed-handle class per `ptr_class` entry.
//!   * one enum / data / `@JvmInline value` class per declaration.
//!   * one top-level free-function bucket per `package()` context.
//!   * the centralized `external fun` holder (`JNINative`). (`impl Fn(...)`
//!     params surface as typed Kotlin lambdas on the wrapper tier and erased
//!     `Any` here — no fun-interface files are generated.)
//!
//! The fragments are merged by [`kt::merge_files`] so every Java/Kotlin
//! package collapses to a SINGLE `.kt` file, written by [`kt::write_files`]
//! at the FLATTENED path `<root>/<package as dirs>.kt` (`io.zenoh.jni.config`
//! → `io/zenoh/jni/config.kt`) — i.e. the file is named after the package's
//! last segment and lives in the directory of its parent package, holding all
//! of that package's classes, enums, value-classes and free functions.
//!
//! Every `#[prebindgen]` function must be assigned a Kotlin home — as a
//! class member (`.fun`/`.constructor` on a class decl) or a free function
//! (`PackageDecl::fun`). Undeclared functions are skipped with a build
//! warning (`Registry::scan_declared`); there is no "orphan" bucket.

use super::*;
use crate::api::gen::{
    kotlin as kt,
    kotlin::{ClassKind, Code, KtClass, KtCtorParam, KtFun, KtParam, KtProperty, KtType, Vis},
};

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
    /// `"io.zenoh.jni.JNIPublisher"`).
    pub kotlin_fqn: &'a str,
    /// Canonical Rust type key of the handle — used to look up the class's
    /// [`crate::api::lang::jnigen::jni::ClassMember`]s (promoted methods).
    pub key: &'a TypeKey,
}

impl crate::api::core::Generation<JniGen> {
    /// Unified Kotlin emission — the JNI adapter's second artifact,
    /// alongside [`write_rust`](Self::write_rust). Each per-kind emitter
    /// builds in-memory [`kt::KtFile`] model fragments; they are merged
    /// into one file per package, rendered, and written under
    /// `kotlin_root` — which is **generator-owned**: deleted and recreated
    /// on every run (point it at a dedicated directory like
    /// `kotlin/generated/`, never at hand-written sources). Pure emission
    /// over the resolved registry — order-free with respect to
    /// `write_rust`. Returns every path written (one per non-empty
    /// package).
    pub fn write_kotlin(&self, kotlin_root: &Path) -> Result<Vec<PathBuf>, WriteKotlinError> {
        self.adapter().write_kotlin(self.registry(), kotlin_root)
    }
}

impl JniGen {
    /// Kotlin emission body — the public entry point is
    /// `Generation::<JniGen>::write_kotlin`, which guarantees the registry
    /// was resolved first.
    pub(crate) fn write_kotlin(
        &self,
        registry: &Registry<KotlinMeta>,
        kotlin_root: &Path,
    ) -> Result<Vec<PathBuf>, WriteKotlinError> {
        // #52: verify every multi-variant expand_param! is splittable up front,
        // before any per-function `.split_on_param` emission.
        self.validate_split_declarations(registry);
        let mut fragments: Vec<kt::KtFile> = Vec::new();
        fragments.push(self.write_native_handle());
        fragments.extend(self.write_enum_classes(registry)?);
        fragments.extend(self.write_data_classes(registry));
        fragments.extend(self.write_value_blobs(registry)?);

        // Build the borrowed `TypedHandle<'_>` view from internal config.
        let owned = self.collect_typed_handles();
        let typed_handles: Vec<TypedHandle<'_>> = owned
            .iter()
            .map(|h| TypedHandle {
                rust_doc: &h.rust_doc,
                kotlin_fqn: &h.kotlin_fqn,
                key: &h.key,
            })
            .collect();
        fragments.extend(self.write_typed_handles(registry, &typed_handles));
        fragments.extend(self.write_callback_ifaces(registry));
        for (subpackage, pkg_cfg) in &self.packages {
            if pkg_cfg.functions.is_empty()
                && pkg_cfg.constants.is_empty()
                && pkg_cfg.constant_functions.is_empty()
                && pkg_cfg.constant_exprs.is_empty()
            {
                continue;
            }
            fragments.push(self.write_jni_package(registry, subpackage, pkg_cfg));
        }
        fragments.push(self.write_jni_native(registry));

        kt::write_files(&kt::merge_files(fragments)?, kotlin_root)
    }

    /// Emit the shared-base fragment — the `NativeHandle` class every typed
    /// handle extends, plus the `withSortedHandleLocks` helper that the
    /// generated wrappers use to acquire any number of handle monitors in
    /// one pointer-sorted, deadlock-safe pass.
    pub(crate) fn write_native_handle(&self) -> kt::KtFile {
        let handle_ty = KtType::cls("NativeHandle");
        // `body: () -> R` — a zero-param function type.
        let body_param = || KtParam::new("body", KtType::lambda([], KtType::var_r()));

        let mut file = kt::KtFile::new(&self.package).decl(
            KtClass::new(ClassKind::Abstract, "NativeHandle")
                .vis(Vis::Public)
                .kdoc(
                    "Base class for every typed native handle: owns the raw `Box<T>` pointer\n\
                     slot and its monitor. Subclasses add their type-specific `close()` /\n\
                     `take()` / `freePtr`.\n\
                     \n\
                     Lifecycle is a tag bit: `Box` pointers are at least 2-aligned (asserted\n\
                     on the Rust side), so bit 0 is free — closing/consuming sets `ptr = p or 1`\n\
                     instead of zeroing. The address bits (`ptr and -2`) are therefore\n\
                     write-once for the object's whole lifetime, which is what makes them a\n\
                     sound lock-ordering key (a mutable key could reorder concurrent lock\n\
                     acquisition and deadlock). All `ptr` writes happen under this handle's\n\
                     monitor.",
                )
                .ctor_param(KtCtorParam::new("initialPtr", KtType::long()))
                .supertype(KtType::cls("AutoCloseable"), None)
                .member(
                    KtProperty::var("ptr")
                        .ty(KtType::long())
                        .initializer("initialPtr")
                        .vis(Vis::Internal)
                        .annotation("Volatile"),
                )
                .member(
                    KtFun::new("peek")
                        .vis(Vis::Public)
                        .kdoc("The live pointer, or `0` if this handle is closed.")
                        .returns(KtType::long())
                        .body(
                            Code::new()
                                .line("val p = ptr")
                                .line("return if (p == 0L || (p and 1L) != 0L) 0L else p"),
                        ),
                )
                .member(
                    KtFun::new("isClosed")
                        .vis(Vis::Public)
                        .returns(KtType::boolean())
                        .expr_body(Code::new().line("ptr == 0L || (ptr and 1L) != 0L")),
                ),
        );

        // The N-ary locking helper is only referenced when wrappers are
        // emitted with locking on; skip it under `set_emit_handle_locks(false)`
        // so it doesn't surface as an unused-`internal fun` warning.
        if self.emit_handle_locks {
            file = file.decl(
                KtFun::new("withSortedHandleLocks")
                    .vis(Vis::Internal)
                    .kdoc(
                        "Acquire every handle's monitor in one global order — sorted by the\n\
                         immutable address bits (`ptr and -2`; bit 0 is the closed tag and\n\
                         never participates) — so concurrent calls touching the same handles\n\
                         can't deadlock, then run [body]. The key never changes after\n\
                         construction: closing only sets bit 0, so a concurrent `close()`\n\
                         can't reorder anyone's acquisition. Closed handles are still locked;\n\
                         their tagged pointers are rejected by the Rust-side converter guard\n\
                         inside the native call. Scales to any arity.",
                    )
                    .generic("R")
                    .param(KtParam::new(
                        "handles",
                        KtType::generic("List", [handle_ty.clone()]),
                    ))
                    .param(body_param())
                    .returns(KtType::var_r())
                    .body(
                        Code::new()
                            .line("val sorted = handles.sortedBy { it.ptr and -2L }")
                            .line("fun rec(i: Int): R = if (i == sorted.size) body() else synchronized(sorted[i]) { rec(i + 1) }")
                            .line("return rec(0)"),
                    ),
            );
            // Allocation-free fixed-arity overloads for the common cases (1–3
            // statically-known, non-null handles). `inline` folds both the
            // helper and [body] into the call site — no `ArrayList`, no
            // `sortedBy`, no recursion, no lambda object. The ordering key is
            // the masked address bits (`ptr and -2L`) ascending, IDENTICAL to
            // the `List` overload above, so the global lock order is
            // consistent whichever overload a wrapper uses — deadlock-freedom
            // is preserved even across paths.
            file = file
                .decl(
                    KtFun::new("withSortedHandleLocks")
                        .vis(Vis::Internal)
                        .modifier("inline")
                        .kdoc("Allocation-free single-handle lock (one monitor, nothing to order).")
                        .generic("R")
                        .param(KtParam::new("a", handle_ty.clone()))
                        .param(body_param())
                        .returns(KtType::var_r())
                        .expr_body(Code::new().line("synchronized(a) { body() }")),
                )
                .decl(
                    KtFun::new("withSortedHandleLocks")
                        .vis(Vis::Internal)
                        .modifier("inline")
                        .kdoc("Allocation-free two-handle lock: order by masked address then nest monitors.")
                        .generic("R")
                        .param(KtParam::new("a", handle_ty.clone()))
                        .param(KtParam::new("b", handle_ty.clone()))
                        .param(body_param())
                        .returns(KtType::var_r())
                        .body(
                            Code::new()
                                .line("val first: NativeHandle")
                                .line("val second: NativeHandle")
                                .line("if ((a.ptr and -2L) <= (b.ptr and -2L)) { first = a; second = b } else { first = b; second = a }")
                                .line("return synchronized(first) { synchronized(second) { body() } }"),
                        ),
                )
                .decl(
                    KtFun::new("withSortedHandleLocks")
                        .vis(Vis::Internal)
                        .modifier("inline")
                        .kdoc("Allocation-free three-handle lock: 3-compare sorting network, then nest.")
                        .generic("R")
                        .param(KtParam::new("a", handle_ty.clone()))
                        .param(KtParam::new("b", handle_ty.clone()))
                        .param(KtParam::new("c", handle_ty))
                        .param(body_param())
                        .returns(KtType::var_r())
                        .body(
                            Code::new()
                                .line("var x = a")
                                .line("var y = b")
                                .line("var z = c")
                                .line("if ((x.ptr and -2L) > (y.ptr and -2L)) { val t = x; x = y; y = t }")
                                .line("if ((y.ptr and -2L) > (z.ptr and -2L)) { val t = y; y = z; z = t }")
                                .line("if ((x.ptr and -2L) > (y.ptr and -2L)) { val t = x; x = y; y = t }")
                                .line("return synchronized(x) { synchronized(y) { synchronized(z) { body() } } }"),
                        ),
                );
        }
        // Error channel: every generated wrapper takes a **required** trailing
        // error callback `onError: (je: String?, ze…) -> R`. On a native error
        // the Rust side invokes a capture (no JVM throw on the Rust side); the
        // wrapper calls `onError` after the native call returns. The generated
        // code itself never throws — the consumer decides how a failure
        // surfaces (e.g. building a domain object, or throwing its own type).
        file
    }

    /// Emit one `@JvmInline value class <Name>(val bytes: ByteArray)` per
    /// declared `value_blob` type. The class is the typed wrapper level; it is
    /// erased to its `ByteArray` field at the JVM/ABI level, so the `JNINative`
    /// extern (and the wire) stays `ByteArray` while wrappers speak the typed
    /// class. The single field name `bytes` matches `value_projection_field`.
    pub(crate) fn write_value_blobs(
        &self,
        registry: &Registry<KotlinMeta>,
    ) -> Result<Vec<kt::KtFile>, WriteKotlinError> {
        let mut written = Vec::new();
        // Deterministic order by canonical Rust type-key (the `types` map is a
        // HashMap, so iterate sorted keys rather than raw map order).
        let mut keys: Vec<&TypeKey> = self.types.keys().collect();
        keys.sort_by(|a, b| a.as_str().cmp(b.as_str()));
        for key in keys {
            let cfg = &self.types[key];
            if !cfg.value_blob {
                continue;
            }
            let fqn = cfg
                .name_spec
                .as_ref()
                .map(|s| self.fqn_of(s))
                .ok_or_else(|| {
                    WriteKotlinError::Other(format!(
                        "value_blob `{}` has no Kotlin FQN",
                        key.as_str()
                    ))
                })?;
            let (package, class_name) = match fqn.rsplit_once('.') {
                Some((p, c)) => (p.to_string(), c.to_string()),
                None => (String::new(), fqn.clone()),
            };
            let framework_line = format!(
                "Typed by-value wrapper for the native Rust `{}` (a `Copy` blob carried\n\
                 as its raw bytes; `@JvmInline`-erased to `ByteArray` at the JNI boundary).",
                key.as_str()
            );
            let class_kdoc = crate::api::lang::jnigen::jni::source_item_doc(registry, key)
                .map(|d| format!("{d}\n\n{framework_line}"))
                .unwrap_or(framework_line);
            let mut class = KtClass::new(ClassKind::ValueInline, &class_name)
                .vis(Vis::Public)
                .kdoc(class_kdoc)
                .ctor_param(
                    KtCtorParam::new("bytes", KtType::byte_array())
                        .val()
                        .vis(Vis::Public),
                );
            let mut imports: BTreeSet<String> = BTreeSet::new();
            let members = self
                .class_members
                .get(key)
                .map(Vec::as_slice)
                .unwrap_or(&[]);
            if !members.is_empty() && !self.package.is_empty() {
                imports.insert(format!("{}.{}", self.package, self.jni_native_class_name()));
            }
            // Promoted instance methods (`.fun`): receiver bound to `this`,
            // passing `this.bytes` to the extern.
            for m in members.iter().filter(|m| m.kind == MemberKind::Fun) {
                if let Some((item_fn, _)) = registry.functions.get(&m.rust_ident) {
                    if let Some(f) = crate::api::lang::jnigen::jni::render_wrapper_fn(
                        self,
                        item_fn,
                        registry,
                        &mut imports,
                        Some(self.effective_member_name(key, m).as_str()),
                        Some(key),
                    ) {
                        for ov in crate::api::lang::jnigen::jni::render_param_overloads(
                            self, item_fn, registry, &f,
                        ) {
                            class = class.member(ov);
                        }
                        class = class.member(f);
                    }
                }
            }
            // Companion-object factory members (`.constructor`).
            let ctors: Vec<_> = members
                .iter()
                .filter(|m| m.kind == MemberKind::Constructor)
                .collect();
            if !ctors.is_empty() {
                let mut companion = KtClass::companion_object().vis(Vis::Public);
                for m in ctors {
                    if let Some((item_fn, _)) = registry.functions.get(&m.rust_ident) {
                        if let Some(f) = crate::api::lang::jnigen::jni::render_wrapper_fn(
                            self,
                            item_fn,
                            registry,
                            &mut imports,
                            Some(self.effective_member_name(key, m).as_str()),
                            None,
                        ) {
                            for ov in crate::api::lang::jnigen::jni::render_param_overloads(
                                self, item_fn, registry, &f,
                            ) {
                                companion = companion.member(ov);
                            }
                            companion = companion.member(f);
                        }
                    }
                }
                class = class.companion(companion);
            }
            let mut file = kt::KtFile::new(package);
            if let Some(iface) =
                self.apply_class_interface(key, &mut class, &class_name, &[], Vec::new(), true)
            {
                file = file.decl(iface);
            }
            written.push(file.decl(class).imports(imports));
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
            if cfg.opaque.is_none() {
                continue;
            }
            let Some(kotlin_fqn) = cfg.name_spec.as_ref().map(|s| self.fqn_of(s)) else {
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
                key: key.clone(),
            });
        }
        handles
    }
}

/// Owned counterpart of [`TypedHandle`] — used internally so the
/// `collect_typed_handles` helper doesn't have to hand out borrows of
/// `self.types`.
pub(crate) struct OwnedTypedHandle {
    pub rust_doc: String,
    pub kotlin_fqn: String,
    pub key: TypeKey,
}

impl JniGen {
    /// Emit one Kotlin `enum class` file per `enum_class`-declared type.
    /// Variants render in declaration order using SCREAMING_SNAKE_CASE names; the
    /// constructor stores the Rust discriminant value (or the ordinal as
    /// a fallback when the discriminant isn't a bare integer literal).
    /// A `fromInt(value: Int)` companion mirrors the `Priority.fromInt`
    /// shape that hand-written enums use today, so adapter code stays
    /// uniform.
    pub(crate) fn write_enum_classes(
        &self,
        registry: &Registry<KotlinMeta>,
    ) -> Result<Vec<kt::KtFile>, WriteKotlinError> {
        let mut written = Vec::new();
        // Deterministic order by canonical Rust type-key.
        let mut keys: Vec<&TypeKey> = self.types.keys().collect();
        keys.sort_by(|a, b| a.as_str().cmp(b.as_str()));
        for key in keys {
            let cfg = &self.types[key];
            if cfg.enum_cfg.is_none() {
                continue;
            }
            let Some(kotlin_fqn) = cfg.name_spec.as_ref().map(|s| self.fqn_of(s)) else {
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
            let mut class = build_enum_class(&class_name, item_enum);
            let mut file = kt::KtFile::new(package);
            if let Some(iface) =
                self.apply_class_interface(key, &mut class, &class_name, &[], Vec::new(), true)
            {
                file = file.decl(iface);
            }
            written.push(file.decl(class));
        }
        Ok(written)
    }

    /// Build one Kotlin `data class` fragment per `data_class`-declared
    /// struct. Uses resolved converter metadata to derive Kotlin field
    /// types, so wrappers and data-class declarations stay in sync. A
    /// compatibility-alias fragment is appended when any data class is
    /// renamed relative to its Rust ident.
    pub(crate) fn write_data_classes(&self, registry: &Registry<KotlinMeta>) -> Vec<kt::KtFile> {
        let mut written = Vec::new();
        let mut aliases: Vec<(String, String)> = Vec::new();
        let mut keys: Vec<&TypeKey> = self.types.keys().collect();
        keys.sort_by(|a, b| a.as_str().cmp(b.as_str()));

        for key in keys {
            let cfg = &self.types[key];
            // Opaque handles, enums and `value_blob` (`@JvmInline value`)
            // types each have their own emitter; only plain structs become
            // data classes here.
            if cfg.special_decl() {
                continue;
            }
            let Some(kotlin_fqn) = cfg.name_spec.as_ref().map(|s| self.fqn_of(s)) else {
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

            let (package, class_name) = match kotlin_fqn.rsplit_once('.') {
                Some((p, c)) => (p.to_string(), c.to_string()),
                None => (String::new(), kotlin_fqn.clone()),
            };
            if item_struct.ident != class_name {
                aliases.push((item_struct.ident.to_string(), class_name.clone()));
            }
            let (mut class, mut imports) =
                build_data_class(self, &class_name, item_struct, registry);
            // Members: same shape as the value-blob path — the instance
            // method's receiver re-enters Rust as `this`'s field leaves
            // (the data-class param destructuring, rebased to `this`).
            let members = self
                .class_members
                .get(key)
                .map(Vec::as_slice)
                .unwrap_or(&[]);
            if !members.is_empty() && !self.package.is_empty() {
                imports.insert(format!("{}.{}", self.package, self.jni_native_class_name()));
            }
            for m in members.iter().filter(|m| m.kind == MemberKind::Fun) {
                if let Some((item_fn, _)) = registry.functions.get(&m.rust_ident) {
                    if let Some(f) = crate::api::lang::jnigen::jni::render_wrapper_fn(
                        self,
                        item_fn,
                        registry,
                        &mut imports,
                        Some(self.effective_member_name(key, m).as_str()),
                        Some(key),
                    ) {
                        for ov in crate::api::lang::jnigen::jni::render_param_overloads(
                            self, item_fn, registry, &f,
                        ) {
                            class = class.member(ov);
                        }
                        class = class.member(f);
                    }
                }
            }
            let ctors: Vec<_> = members
                .iter()
                .filter(|m| m.kind == MemberKind::Constructor)
                .collect();
            if !ctors.is_empty() {
                // `build_data_class` already installed the `fromParts`
                // companion — factories join it rather than replacing it.
                let mut companion = class
                    .companion
                    .take()
                    .map(|c| *c)
                    .unwrap_or_else(|| KtClass::companion_object().vis(Vis::Public));
                for m in ctors {
                    if let Some((item_fn, _)) = registry.functions.get(&m.rust_ident) {
                        if let Some(f) = crate::api::lang::jnigen::jni::render_wrapper_fn(
                            self,
                            item_fn,
                            registry,
                            &mut imports,
                            Some(self.effective_member_name(key, m).as_str()),
                            None,
                        ) {
                            for ov in crate::api::lang::jnigen::jni::render_param_overloads(
                                self, item_fn, registry, &f,
                            ) {
                                companion = companion.member(ov);
                            }
                            companion = companion.member(f);
                        }
                    }
                }
                class = class.companion(companion);
            }
            let mut file = kt::KtFile::new(package);
            if let Some(iface) =
                self.apply_class_interface(key, &mut class, &class_name, &[], Vec::new(), true)
            {
                file = file.decl(iface);
            }
            written.push(file.decl(class).imports(imports));
        }

        if !aliases.is_empty() {
            // Compatibility aliases for legacy un-mangled data-class references.
            aliases.sort_by(|a, b| a.0.cmp(&b.0));
            aliases.dedup_by(|a, b| a.0 == b.0 && a.1 == b.1);
            let mut file = kt::KtFile::new(&self.package);
            for (legacy, current) in aliases {
                file = file.decl(kt::KtDecl::TypeAlias {
                    vis: Vis::Public,
                    name: legacy,
                    target: KtType::cls(current),
                });
            }
            written.push(file);
        }

        written
    }

    /// Build the package-level wrapper fragment for the given subpackage.
    /// One top-level safe wrapper per `MethodEntry` in `pkg_cfg.functions`.
    /// Wrappers delegate to the centralized Native object (see
    /// [`Self::write_jni_native`]). Opaque-handle parameters become
    /// `NativeHandle`; the wrapper body nests `withPtr` / `consume` per
    /// the type-conversion rule. Non-opaque parameters pass through with
    /// their mapped Kotlin type. Opaque-handle return values
    /// are wrapped in `NativeHandle(...)` before return.
    /// Emit every typed callback `fun interface` the declared functions
    /// reference — impl-`Fn` delivery callbacks, output-expansion builders
    /// and folders, and onError handlers (plus the shared `JniErrorHandler`
    /// for infallible functions). The function walk only **collects which
    /// identities are used** (emission stays opt-in: an unused declaration
    /// emits nothing); each spec is then derived exactly once per identity
    /// from the declaration's representative plan (`registry.decon_plans`) —
    /// the same source the native emitters read, so all sites agree by
    /// construction (no dedup, no signature reconciliation).
    pub(crate) fn write_callback_ifaces(&self, registry: &Registry<KotlinMeta>) -> Vec<kt::KtFile> {
        use crate::api::core::unfold::{DeconId, Delivery};

        /// One distinct interface identity in use. Ordered so emission is
        /// deterministic.
        #[derive(PartialEq, Eq, PartialOrd, Ord)]
        enum Use {
            /// impl-Fn delivery — identified by the args' canonical type keys
            /// (each arg uses its type's canonical decomposition or crosses
            /// whole; the spec carries the arg types).
            Callback(Vec<String>),
            Builder(DeconId),
            Folder(DeconId),
            /// Whole-element fold — no declaration; keyed by element type.
            WholeFolder(String),
            Handler(DeconId),
            JniErrorHandler,
        }
        // Identity → the syn-typed context the spec constructor needs (arg
        // types for Callback, the element type for WholeFolder).
        let mut uses: BTreeMap<Use, Vec<syn::Type>> = BTreeMap::new();

        /// A hoisted-singleton request emitted alongside an interface: the
        /// `fromParts` builder / folder for a synthesized `data_class`, or the
        /// single-leaf appender for a whole-element leaf fold. The wrapper
        /// references the singleton instead of taking a caller `build`/`fold`.
        enum FixedSingleton {
            StructBuilder(DeconId),
            StructFolder(DeconId),
            LeafFolder,
        }

        // DeconIds whose builder is a synthesized by-value `data_class`
        // (`fixed_builder`): these get a hoisted `__<Name>Builder` singleton
        // (the `fromParts` factory) emitted alongside the interface, so the
        // wrapper references it instead of taking a caller `build` param.
        let fixed_decons: std::collections::HashSet<DeconId> = registry
            .unfold_plans
            .values()
            .chain(registry.callback_arg_plans.values())
            .filter(|p| p.fixed_builder)
            .filter_map(|p| p.decon.clone())
            .collect();
        // Element type keys whose whole-element fold is fixed (a synthesized
        // single-leaf `Vec<T>` fold): these get a hoisted `__<Elem>FolderRaw`
        // appender singleton, the leaf dual of `fixed_decons`.
        let fixed_leaf_elements: std::collections::HashSet<String> = registry
            .unfold_plans
            .values()
            .chain(registry.callback_arg_plans.values())
            .filter(|p| p.fixed_builder)
            .filter_map(|p| p.element.as_ref())
            .map(|el| TypeKey::from_type(el).to_string())
            .collect();

        // Walk every declared function — free `.fun`s AND class members
        // (`.method`/`.accessor`/`.constructor`): a method can also need a
        // generated interface (e.g. a `Vec<T>` whole-element folder). The `uses`
        // map dedups, so an identity shared across positions emits once.
        let declared_idents: std::collections::BTreeSet<syn::Ident> = self
            .packages
            .values()
            .flat_map(|p| p.functions.iter().map(|e| e.rust_ident.clone()))
            .chain(
                self.class_members
                    .values()
                    .flatten()
                    .map(|m| m.rust_ident.clone()),
            )
            .collect();
        for ident in &declared_idents {
            {
                let Some((item_fn, _loc)) = registry.functions.get(ident) else {
                    continue;
                };
                for input in &item_fn.sig.inputs {
                    let syn::FnArg::Typed(pt) = input else {
                        continue;
                    };
                    if let Some(cb_args) = extract_fn_trait_args(&pt.ty) {
                        let key = cb_args
                            .iter()
                            .map(|t| TypeKey::from_type(t).to_string())
                            .collect();
                        uses.insert(Use::Callback(key), cb_args);
                    }
                }
                if let Some(plan) = registry
                    .unfold_plans
                    .get(&item_fn.sig.ident)
                    .filter(|p| p.delivery == Delivery::Callback)
                {
                    let iterable = is_iterable_fold(&plan.shape);
                    match (iterable, &plan.element, &plan.decon) {
                        (true, Some(el), _) => {
                            uses.insert(
                                Use::WholeFolder(TypeKey::from_type(el).to_string()),
                                vec![el.clone()],
                            );
                        }
                        (true, None, Some(d)) => {
                            uses.insert(Use::Folder(d.clone()), vec![]);
                        }
                        (false, _, Some(d)) => {
                            uses.insert(Use::Builder(d.clone()), vec![]);
                        }
                        _ => {}
                    }
                }
                match registry.error_plans.get(&item_fn.sig.ident) {
                    Some(ep) => {
                        let d = ep
                            .decon
                            .clone()
                            .expect("error plans are always record-built (decon is Some)");
                        uses.insert(Use::Handler(d), vec![]);
                    }
                    None => {
                        uses.insert(Use::JniErrorHandler, vec![]);
                    }
                }
            }
        }

        uses.into_iter()
            .filter_map(|(u, tys)| {
                // `is_error` ⇒ also emit the zero-alloc capture holder used by
                // the generated wrappers' error channel. `fixed` carries the
                // builder's DeconId when it is a synthesized `data_class`, so a
                // hoisted `__<Name>Builder` singleton is emitted with it.
                // `fixed` carries a hoisted-singleton request: `(decon, is_folder)`.
                // `is_folder` picks the folder-appender singleton (`Vec<data_class>`
                // fold) over the scalar `fromParts` builder.
                let (spec, is_error, fixed) = match u {
                    Use::Callback(_) => (callback_iface_spec(self, registry, &tys), false, None),
                    Use::Builder(d) => {
                        let fixed = fixed_decons
                            .contains(&d)
                            .then(|| FixedSingleton::StructBuilder(d.clone()));
                        (builder_iface_spec(self, registry, &d), false, fixed)
                    }
                    Use::Folder(d) => {
                        // A fixed-builder fold groups the leaves into a typed
                        // `(acc, element)` view (raw twin keeps the leaves) so the
                        // emitted interface matches the wrapper's
                        // `folder_iface_for_plan`; an explicit-accessor fold keeps
                        // its 1:1 leaf view unchanged.
                        let is_fixed = fixed_decons.contains(&d);
                        let spec = folder_iface_spec(self, registry, &d).map(|mut s| {
                            if is_fixed {
                                s.typed_groups = fixed_folder_typed_groups(self, registry, &d)
                                    .unwrap_or_default();
                            }
                            s
                        });
                        (
                            spec,
                            false,
                            is_fixed.then(|| FixedSingleton::StructFolder(d.clone())),
                        )
                    }
                    Use::WholeFolder(_) => {
                        // A synthesized single-leaf `Vec<T>` fold gets a hoisted
                        // appender singleton; an explicit caller-fold whole-element
                        // deconstruction (not `fixed_builder`) does not.
                        let fixed = fixed_leaf_elements
                            .contains(&TypeKey::from_type(&tys[0]).to_string())
                            .then_some(FixedSingleton::LeafFolder);
                        (
                            whole_folder_iface_spec(self, registry, &tys[0]),
                            false,
                            fixed,
                        )
                    }
                    Use::Handler(d) => (error_handler_iface_spec(self, registry, &d), true, None),
                    Use::JniErrorHandler => (Some(jni_error_handler_iface_spec(self)), true, None),
                };
                spec.map(|s| (s, is_error, fixed))
            })
            .map(|(s, is_error, fixed)| {
                // Typed (user-facing) interface; when any leaf's raw view
                // differs, also the JNI-called raw twin and the `asRaw()`
                // proxy adapter that wraps raw leaves into typed objects.
                let mut file = kt::KtFile::new(s.package.clone()).decl(s.to_decl());
                if s.needs_raw() {
                    file = file.decl(s.to_raw_decl()).decl(s.to_as_raw_fun());
                    for p in &s.params {
                        if let Some(fqn) = p.wrap.class_fqn() {
                            file = file.import(fqn.to_string());
                        }
                    }
                }
                if is_error {
                    file = file.decl(s.to_capture_decl());
                }
                if let Some(fixed) = fixed {
                    let decl = match fixed {
                        FixedSingleton::StructBuilder(decon) => {
                            self.value_struct_builder_singleton(registry, &s, &decon)
                        }
                        FixedSingleton::StructFolder(decon) => {
                            self.value_struct_folder_singleton(registry, &s, &decon)
                        }
                        FixedSingleton::LeafFolder => self.whole_value_folder_singleton(&s),
                    };
                    file = file.decl(decl);
                }
                file
            })
            .collect()
    }

    /// The hoisted **fixed builder** singleton for a synthesized by-value
    /// `data_class` decomposition: `internal val __<Name>Builder:
    /// <Name>Builder<Class> = <Name>Builder { leaves… -> Class.fromParts(leaves…) }`.
    /// One instance per process (a Kotlin SAM singleton — no per-call alloc);
    /// the wrapper passes it to the native call instead of taking a caller
    /// `build` param, so the object is reconstructed on the Kotlin side via the
    /// existing `fromParts` factory and never built on the Rust side. The leaf
    /// names/order come straight from the builder interface, so they line up
    /// positionally with `fromParts`.
    fn value_struct_builder_singleton(
        &self,
        registry: &Registry<KotlinMeta>,
        spec: &crate::api::lang::jnigen::jni::IfaceSpec,
        decon: &crate::api::core::unfold::DeconId,
    ) -> kt::KtDecl {
        let source = &registry.decon_plans[decon].source;
        let class_fqn = self
            .kotlin_fqn(&TypeKey::from_type(source).to_string())
            .unwrap_or_else(|| {
                panic!(
                    "value-struct builder: no Kotlin FQN for {}",
                    TypeKey::from_type(source)
                )
            });
        let class_short = class_fqn.rsplit('.').next().unwrap_or(&class_fqn);
        // The native side calls the raw twin's `run` (== the typed interface
        // when the builder needs no twin — synthesized data classes are
        // all-simple-leaf today). `fromParts` takes the raw wire types and
        // applies any projection/enum wrap itself.
        let builder = spec.raw_name();
        let val_name = format!("__{builder}");
        let names: Vec<String> = spec.params.iter().map(|p| p.name.clone()).collect();
        let joined = names.join(", ");
        let code = format!(
            "internal val {val_name}: {builder}<{class_short}> =\n    \
             {builder} {{ {joined} -> {class_short}.fromParts({joined}) }}"
        );
        kt::KtDecl::Raw {
            name: val_name,
            code: kt::Code::raw_reindent(&code),
        }
    }

    /// The hoisted **folder-appender** singleton for a synthesized by-value
    /// `data_class` element fold (`Vec<data_class>` return): an instance of the
    /// folder's raw twin (`__<Name>FolderRaw`) that, per element, rebuilds the
    /// value via `fromParts` and appends it to the accumulator `ArrayList`,
    /// returning the same list. The wrapper allocates the `ArrayList`, passes this
    /// singleton as the `fold`, and returns the threaded accumulator as a
    /// `List<Class>` — so the list is composed on the Kotlin side and no Java
    /// object is built on the Rust side. The folder's `run` params are
    /// `[acc, leaf0, …]`; `fromParts` takes the element leaves (all but `acc`).
    fn value_struct_folder_singleton(
        &self,
        registry: &Registry<KotlinMeta>,
        spec: &crate::api::lang::jnigen::jni::IfaceSpec,
        decon: &crate::api::core::unfold::DeconId,
    ) -> kt::KtDecl {
        let source = &registry.decon_plans[decon].source;
        let class_fqn = self
            .kotlin_fqn(&TypeKey::from_type(source).to_string())
            .unwrap_or_else(|| {
                panic!(
                    "value-struct folder: no Kotlin FQN for {}",
                    TypeKey::from_type(source)
                )
            });
        let class_short = class_fqn.rsplit('.').next().unwrap_or(&class_fqn);
        // The native side calls the raw twin's `run(acc, leaves…)`; `acc` is the
        // accumulator list and the remaining params are the element leaves.
        let folder = spec.raw_name();
        let holder = spec.singleton_holder_name();
        let field = crate::api::lang::jnigen::jni::SINGLETON_FIELD;
        let names: Vec<String> = spec.params.iter().map(|p| p.name.clone()).collect();
        let lambda_params = names.join(", ");
        let acc = &names[0];
        let leaf_args = names[1..].join(", ");
        let acc_ty = format!("ArrayList<{class_short}>");
        // The folder appender lives as a `@JvmField` in a holder `object` (not a
        // top-level `val`) so it has a stable JVM class + static field that the
        // callback trampoline can fetch via `FindClass` + `GetStaticField`; the
        // output `Vec` wrapper references it as `{holder}.{field}`.
        let code = format!(
            "internal object {holder} {{\n    \
             @JvmField\n    \
             val {field}: {folder}<{acc_ty}> =\n        \
             {folder} {{ {lambda_params} -> \
             {acc}.add({class_short}.fromParts({leaf_args})); {acc} }}\n\
             }}"
        );
        kt::KtDecl::Raw {
            name: holder,
            code: kt::Code::raw_reindent(&code),
        }
    }

    /// The hoisted **folder-appender** singleton for a **whole single-leaf
    /// element** fold (`Vec<String>` / `Vec<value-blob>` return, or the matching
    /// slice callback): an instance of the folder's raw twin (`__<Elem>FolderRaw`)
    /// that, per element, wraps the raw leaf into its typed Kotlin value and
    /// appends it to the accumulator `ArrayList`, returning the same list. The
    /// single-leaf analog of [`Self::value_struct_folder_singleton`] — there is no
    /// `fromParts`; reassembly is just `acc.add(<wrap>(element))`, where `<wrap>`
    /// is the value-class ctor for a value blob, the handle ctor for a handle, or
    /// identity for a String. So the list is composed on the Kotlin side and no
    /// Java object is built on the Rust side. The folder's `run` params are
    /// `[acc, element]`.
    fn whole_value_folder_singleton(
        &self,
        spec: &crate::api::lang::jnigen::jni::IfaceSpec,
    ) -> kt::KtDecl {
        let folder = spec.raw_name();
        let holder = spec.singleton_holder_name();
        let field = crate::api::lang::jnigen::jni::SINGLETON_FIELD;
        // params[0] is the accumulator `acc`; params[1] is the single element leaf.
        let acc = &spec.params[0].name;
        let elem = &spec.params[1];
        let elem_short = elem.typed.simple_name().unwrap_or("Any");
        let wrap = elem.wrap.wrap_expr(&elem.name, false);
        let acc_ty = format!("ArrayList<{elem_short}>");
        let code = format!(
            "internal object {holder} {{\n    \
             @JvmField\n    \
             val {field}: {folder}<{acc_ty}> =\n        \
             {folder} {{ {acc}, {elem} -> {acc}.add({wrap}); {acc} }}\n\
             }}",
            elem = elem.name,
        );
        kt::KtDecl::Raw {
            name: holder,
            code: kt::Code::raw_reindent(&code),
        }
    }

    pub(crate) fn write_jni_package(
        &self,
        registry: &Registry<KotlinMeta>,
        subpackage: &str,
        pkg_cfg: &crate::api::lang::jnigen::jni::PackageConfig,
    ) -> kt::KtFile {
        let package = if self.package.is_empty() {
            subpackage.to_string()
        } else if subpackage.is_empty() {
            self.package.clone()
        } else {
            format!("{}.{}", self.package, subpackage)
        };
        let mut file = kt::KtFile::new(&package);
        let mut imports: BTreeSet<String> = BTreeSet::new();
        for entry in &pkg_cfg.functions {
            let (item_fn, _loc) = registry
                .functions
                .get(&entry.rust_ident)
                .unwrap_or_else(|| {
                    panic!(
                        "write_jni_package: function `{}` registered via .function(...) is \
                         not in the prebindgen registry — check the spelling against the \
                         matching `#[prebindgen]` Rust fn name.",
                        entry.rust_ident,
                    )
                });
            if let Some(f) = render_wrapper_fn(
                self,
                item_fn,
                registry,
                &mut imports,
                entry.kotlin_name_override.as_deref(),
                None,
            ) {
                // #52: idiomatic typed overloads for `.split_on_param`
                // parameters, delegating to this selector wrapper.
                for ov in render_param_overloads(self, item_fn, registry, &f) {
                    file = file.decl(ov);
                }
                file = file.decl(f);
            }
        }
        // Declared consts: a private nullary helper + the public
        // lazily-initialized `val` (see `render_const_val`).
        for entry in &pkg_cfg.constants {
            let (item_const, _loc) = registry.consts.get(&entry.rust_ident).unwrap_or_else(|| {
                panic!(
                    "write_jni_package: const `{}` registered via .constant(...) is \
                     not in the prebindgen registry — check the spelling against the \
                     matching `#[prebindgen]` Rust const name.",
                    entry.rust_ident,
                )
            });
            reject_handle_const(self, item_const);
            if let Some((helper, prop)) = render_const_val(
                self,
                item_const,
                registry,
                &mut imports,
                entry.kotlin_name_override.as_deref(),
            ) {
                file = file.decl(helper).decl(prop);
            }
        }
        // Function-backed constants: the declared nullary fn's ordinary
        // wrapper demoted to a private helper + the public lazily-initialized
        // `val` (see `render_constant_fn_val`). The JNINative extern and the
        // Rust wrapper are the plain declared-function ones.
        for entry in &pkg_cfg.constant_functions {
            let (item_fn, _loc) = registry
                .functions
                .get(&entry.rust_ident)
                .unwrap_or_else(|| {
                    panic!(
                        "write_jni_package: constant fn `{}` registered via .constant_fun(...) \
                         is not in the prebindgen registry — check the spelling against the \
                         matching `#[prebindgen]` Rust fn name.",
                        entry.rust_ident,
                    )
                });
            validate_constant_fn(self, item_fn);
            if let Some((helper, prop)) = render_constant_fn_val(
                self,
                item_fn,
                registry,
                &mut imports,
                entry.kotlin_name_override.as_deref(),
            ) {
                file = file.decl(helper).decl(prop);
            }
        }
        // Expression constants: a private nullary helper over the synthetic
        // getter + the public lazily-initialized `val` (see
        // `render_const_expr_val`). The value is a binding-defined expression
        // evaluated Rust-side (`prerequisites`).
        for decl in &pkg_cfg.constant_exprs {
            validate_constant_expr(self, &decl.kotlin_name, &decl.ty);
            if let Some((helper, prop)) = render_const_expr_val(self, decl, registry, &mut imports)
            {
                file = file.decl(helper).decl(prop);
            }
        }
        // The wrapper bodies call the centralized Native object.
        if !self.package.is_empty() {
            imports.insert(format!("{}.{}", self.package, self.jni_native_class_name()));
        }
        file.imports(imports)
    }

    /// Emit the centralized Native-object Kotlin file under `output_dir`
    /// (class name from [`JniGen::jni_native_class_name`]). Holds one
    /// `external fun` per `#[prebindgen]` function — names mangled via
    /// [`JniGen::set_fun_name_mangle`], parameter and return types rendered at
    /// the JNI **wire** level so the declarations match the Rust extern
    /// symbols generated under
    /// `Java_<package>_<jni_native_class>_<name>`. Every generated native
    /// call routes through this object, so its static initializer is the
    /// single point at which native-library loading can be triggered: when
    /// [`JniGen::jni_native_init`] is set, its Kotlin statement(s) are emitted
    /// inside an `init { … }` block here (e.g. a reference to the consumer's
    /// own loader object). Unset, the holder stays free of any loading logic
    /// and the wrapper layer is responsible for loading.
    pub(crate) fn write_jni_native(&self, registry: &Registry<KotlinMeta>) -> kt::KtFile {
        let class_name = self.jni_native_class_name();
        let declared = self.declared_functions();

        let mut imports: BTreeSet<String> = BTreeSet::new();
        let mut externs = Code::new();
        let mut idents: Vec<&syn::Ident> = registry.functions.keys().collect();
        idents.sort();
        for ident in idents {
            if !declared.contains(ident) {
                continue;
            }
            let (item_fn, _loc) = &registry.functions[ident];
            if let Some(code) = render_extern_decl(self, item_fn, registry, &mut imports) {
                externs = externs.push(code);
            }
        }

        // Declared consts: one `external fun` per generated nullary getter,
        // derived from the same synthetic signature (`const_getter_fn`) the
        // Rust extern is emitted from — both sides stay in sync by
        // construction.
        let mut const_idents: Vec<&syn::Ident> = self
            .packages
            .values()
            .flat_map(|p| p.constants.iter().map(|e| &e.rust_ident))
            .collect();
        const_idents.sort_by_key(|i| i.to_string());
        for ident in const_idents {
            let Some((item_const, _loc)) = registry.consts.get(ident) else {
                continue; // missing decl already warned by the scan
            };
            let getter = crate::api::lang::jnigen::jni::const_getter_fn(item_const);
            if let Some(code) = render_extern_decl(self, &getter, registry, &mut imports) {
                externs = externs.push(code);
            }
        }

        // Expression constants: same synthetic const_get_* getter shape,
        // seeded from the val name (no Rust item behind them).
        let mut expr_decls: Vec<_> = self
            .packages
            .values()
            .flat_map(|p| &p.constant_exprs)
            .collect();
        expr_decls.sort_by(|a, b| a.kotlin_name.cmp(&b.kotlin_name));
        for decl in expr_decls {
            let getter = const_expr_getter_fn(&decl.kotlin_name, &decl.ty);
            if let Some(code) = render_extern_decl(self, &getter, registry, &mut imports) {
                externs = externs.push(code);
            }
        }

        // Synthetic slice/Vec-input helpers: a `…VecNew/Push/Free` trio per
        // flattenable element type a scanned `&[T]`/`Vec<T>` param takes — the
        // `external fun` halves of `build_vec_build_helper_items`. Kotlin builds
        // the Rust-side `Vec` by pushing each element's leaves (decoupled raw
        // params), then passes the handle (see `ParamMode::VecBuild`).
        for elem in crate::api::lang::jnigen::jni::collect_vec_build_elem_types(self, registry) {
            let Some(h) = crate::api::lang::jnigen::jni::vec_build_helpers(self, registry, &elem)
            else {
                continue;
            };
            let new_m = crate::api::lang::jnigen::jni::vec_helper_method_name(self, &h.base, "New");
            let push_m =
                crate::api::lang::jnigen::jni::vec_helper_method_name(self, &h.base, "Push");
            let free_m =
                crate::api::lang::jnigen::jni::vec_helper_method_name(self, &h.base, "Free");
            let mut push_params = vec!["handle: Long".to_string()];
            for leaf in h.plan.leaves.iter().filter(|l| !l.is_present_flag) {
                let short = register_fqn(&leaf.kt_wire_ty, &mut imports);
                push_params.push(format!("{}: {}", leaf.kt_name, short));
            }
            externs = externs
                .line(format!("external fun {new_m}(cap: Int): Long"))
                .line(format!("external fun {push_m}({})", push_params.join(", ")))
                .line(format!("external fun {free_m}(handle: Long)"));
        }

        let mut obj = KtClass::object_(class_name).vis(Vis::Internal);
        // Optional native-load trigger: emitted FIRST so the object's static
        // initializer runs the consumer's loader before any extern resolves.
        if let Some(code) = &self.jni_native_init {
            obj = obj.member(kt::KtDecl::Raw {
                name: "native_init".to_string(),
                code: Code::new()
                    .line("init {")
                    .line(format!("    {code}"))
                    .line("}"),
            });
        }
        // One compact run of `external fun` lines (no blank lines between
        // them), kept as a single raw member.
        obj = obj.member(kt::KtDecl::Raw {
            name: "externs".to_string(),
            code: externs,
        });
        kt::KtFile::new(&self.package).decl(obj).imports(imports)
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
    /// [`JniGen::kotlin_fqn`] so the generator can map it back to its
    /// Rust type-key (which identifies the first param to drop in each
    /// promoted method's signature).
    pub(crate) fn write_typed_handles(
        &self,
        registry: &Registry<KotlinMeta>,
        handles: &[TypedHandle<'_>],
    ) -> Vec<kt::KtFile> {
        let mut written = Vec::new();
        for handle in handles {
            let (package, class_name) = match handle.kotlin_fqn.rsplit_once('.') {
                Some((p, c)) => (p.to_string(), c.to_string()),
                None => (String::new(), handle.kotlin_fqn.to_string()),
            };
            let mut imports: BTreeSet<String> = BTreeSet::new();
            let mut class = build_typed_handle(
                self,
                registry,
                &class_name,
                handle.rust_doc,
                handle.key,
                &mut imports,
            );
            // A ptr class's own surface for the generated interface: peek() /
            // isClosed() are inherited from NativeHandle (declared abstract in
            // the interface, satisfied without an `override`); take() and the
            // declared members are class-body (marked override by the helper);
            // close() is covered by AutoCloseable. The interface extends
            // AutoCloseable so consumers get `close()` too.
            let base = vec![
                kt::KtFun::new("peek")
                    .vis(kt::Vis::Default)
                    .returns(kt::KtType::long()),
                kt::KtFun::new("isClosed")
                    .vis(kt::Vis::Default)
                    .returns(kt::KtType::boolean()),
            ];
            let mut file = kt::KtFile::new(package);
            if let Some(iface) = self.apply_class_interface(
                handle.key,
                &mut class,
                &class_name,
                &["AutoCloseable"],
                base,
                false,
            ) {
                file = file.decl(iface);
            }
            written.push(file.decl(class).imports(imports));
        }
        written
    }

    /// The generated-interface short name for a class whose final Kotlin name
    /// is `class_short`: the per-decl `.interface_name(...)` override, else
    /// the `set_interface_name_mangle` hook over the class name (unset
    /// default: append `"Api"`). Asserted to differ from the class name.
    pub(crate) fn interface_short_name(
        &self,
        class_short: &str,
        override_: Option<&str>,
    ) -> String {
        let name = match override_ {
            Some(n) => n.to_string(),
            None => match &self.interface_name_mangle {
                Some(f) => f(class_short),
                None => format!("{class_short}Api"),
            },
        };
        assert!(
            name != class_short,
            "the generated interface name `{name}` must differ from the class name \
             `{class_short}` (a class and its interface cannot share a name in one package)"
        );
        name
    }

    /// Attach interface information to a just-built class. The `.implements`
    /// list is added to the class supertypes unconditionally (nominal
    /// implementation). When `.interface()` is enabled, ALSO build the
    /// generated `<Name>Api` interface mirroring the class's public surface,
    /// add it as a supertype, mark every class-body member (and, when
    /// `include_ctor_props`, every ctor `val`) `override`, and return the
    /// interface decl to emit alongside. `base_abstracts` are signatures the
    /// interface declares that are satisfied by an inherited base member (no
    /// `override` on the class — e.g. a ptr class's `peek()`/`isClosed()`).
    pub(crate) fn apply_class_interface(
        &self,
        key: &TypeKey,
        class: &mut kt::KtClass,
        class_short: &str,
        extra_supers: &[&str],
        base_abstracts: Vec<kt::KtFun>,
        include_ctor_props: bool,
    ) -> Option<kt::KtClass> {
        let cfg = self.types.get(key)?;
        let interfaces = cfg.interfaces.clone();
        let enabled = cfg.interface_enabled;
        let name_override = cfg.interface_name_override.clone();

        if !enabled {
            for iface in &interfaces {
                class.supertypes.push((kt::KtType::cls(iface), None));
            }
            return None;
        }

        let iface_name = self.interface_short_name(class_short, name_override.as_deref());
        let mut iface =
            kt::KtClass::new(kt::ClassKind::Interface, &iface_name).vis(kt::Vis::Public);
        for s in extra_supers {
            iface = iface.supertype(kt::KtType::cls(*s), None);
        }
        // Signatures satisfied by an inherited base member.
        for f in base_abstracts {
            iface = iface.member(f);
        }
        // Ctor `val`s become interface properties (data/value/enum).
        if include_ctor_props {
            for p in &mut class.ctor_params {
                if p.prop.is_some() {
                    iface = iface.member(
                        kt::KtProperty::val(&p.name)
                            .ty(p.ty.clone())
                            .vis(kt::Vis::Default),
                    );
                    p.overrides = true;
                }
            }
        }
        // Class-body instance methods become interface abstracts + `override`
        // on the class. A member already marked `override` (a ptr class's
        // `close()`, via AutoCloseable) is skipped — already covered.
        for m in &mut class.members {
            if let kt::KtDecl::Fun(f) = m {
                if f.modifiers.iter().any(|s| s == "override") {
                    continue;
                }
                iface = iface.member(abstract_fun_sig(f));
                f.modifiers.push("override".to_string());
            }
        }
        // The generated interface first, then the user `.implements` list.
        class.supertypes.push((kt::KtType::cls(&iface_name), None));
        for iface_fqn in &interfaces {
            class.supertypes.push((kt::KtType::cls(iface_fqn), None));
        }
        Some(iface)
    }
}

/// A concrete class member's signature as an abstract interface member:
/// same name / generics / params / return, no body, no modifiers, no vis
/// keyword (interface members are public-abstract by default).
fn abstract_fun_sig(f: &kt::KtFun) -> kt::KtFun {
    let mut a = kt::KtFun::new(&f.name).vis(kt::Vis::Default);
    for g in &f.generics {
        a = a.generic(g.clone());
    }
    for p in &f.params {
        a = a.param(p.clone());
    }
    if let Some(r) = &f.ret {
        a = a.returns(r.clone());
    }
    a
}
