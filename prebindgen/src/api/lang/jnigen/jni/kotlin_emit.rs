//! `KotlinExt` impl for [`Declarations`].
//!
//! [`Declarations::write_kotlin`] is the single entry point for every Kotlin
//! file the JNI back-end emits. Each per-kind emitter builds in-memory
//! [`kt::KtFile`] *model fragments* (declarations, not strings — the
//! generator module `api::gen::kotlin` owns formatting and imports):
//!   * the shared `NativeHandle` base + lock helpers (root package, e.g.
//!     `io.zenoh.jni`).
//!   * one typed-handle class per `ptr_class` entry.
//!   * one enum / data class per declaration.
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
//! of that package's classes, enums and free functions.
//!
//! Every `#[prebindgen]` function must be assigned a Kotlin home — as a
//! class member (`.method`/`.constructor` on a class decl) or a free function
//! (`PackageDecl::fun`). Undeclared functions are skipped with a build
//! warning (the generator's unclaimed-item report); there is no "orphan" bucket.

use super::*;
use crate::api::{
    core::registry::Conversions,
    gen::{
        kotlin as kt,
        kotlin::{ClassKind, Code, KtClass, KtCtorParam, KtFun, KtParam, KtProperty, KtType, Vis},
    },
};

/// Declaration of one auto-generated typed `NativeHandle` subclass.
///
/// Consumed by [`Declarations::write_typed_handles`] (and forwarded to
/// [`Declarations::write_jni_wrappers`] so the same promotion list can carve
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

impl super::JniGen {
    /// Unified Kotlin emission — the JNI adapter's second artifact,
    /// alongside [`write_rust`](Self::write_rust). Each per-kind emitter
    /// builds in-memory [`kt::KtFile`] model fragments; they are merged
    /// into one file per package, rendered, and written under
    /// `kotlin_root`. The initial write accepts a missing or empty directory
    /// and marks it generator-owned; later writes replace only marked output
    /// (point it at a dedicated directory like `kotlin/generated/`, never at
    /// hand-written sources). Pure emission
    /// over the resolved registry — order-free with respect to
    /// `write_rust`. Returns every path written (one per non-empty
    /// package).
    pub fn write_kotlin(&self, kotlin_root: &Path) -> Result<Vec<PathBuf>, WriteKotlinError> {
        self.declarations()
            .write_kotlin(self.registry(), kotlin_root)
    }
}

impl Declarations {
    /// Kotlin emission body — the public entry point is
    /// `JniGen::write_kotlin`, which guarantees the registry
    /// was resolved first.
    pub(crate) fn write_kotlin(
        &self,
        registry: &Registry<KotlinMeta>,
        kotlin_root: &Path,
    ) -> Result<Vec<PathBuf>, WriteKotlinError> {
        // Validation already ran once in `RegistryBuilder::build` — this emitter
        // is a pure consumer of the resolved, validated registry.
        let mut fragments: Vec<kt::KtFile> = Vec::new();
        fragments.push(self.write_native_handle());
        fragments.extend(self.write_enum_classes(registry)?);
        fragments.extend(self.write_sealed_classes(registry)?);
        fragments.extend(self.write_data_classes(registry));

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

        let mut file = kt::KtFile::new(&self.package)
            .import("java.lang.ref.Cleaner")
            .import("java.util.concurrent.atomic.AtomicLong")
            .decl(
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
                         monitor.\n\
                         \n\
                         A `gc_managed` class extends [GcNativeHandle] instead, which redirects\n\
                         `ptr` into a separate atomic cell so a GC [Cleaner] action can settle\n\
                         the release after the handle object itself is unreachable.",
                    )
                    .ctor_param(KtCtorParam::new("initialPtr", KtType::long()))
                    .supertype(KtType::cls("AutoCloseable"), None)
                    .member(
                        KtProperty::var("ptr")
                            .ty(KtType::long())
                            .initializer("initialPtr")
                            .vis(Vis::Internal)
                            .modifier("open")
                            .annotation("Volatile"),
                    )
                    .member(
                        KtFun::new("markConsumed")
                            .vis(Vis::Internal)
                            .modifier("open")
                            .kdoc(
                                "Mark this handle consumed by value — the native side now owns\n\
                                 (and frees) the box; only the closed tag is recorded here. A\n\
                                 GC-managed handle also settles its release ticket.",
                            )
                            .body(Code::new().line("ptr = ptr or 1L")),
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
            )
            .decl(
                KtClass::new(ClassKind::Abstract, "GcNativeHandle")
                    .vis(Vis::Public)
                    .kdoc(
                        "Storage variant for `gc_managed` handle classes: the pointer (tag bit\n\
                         and all) lives in a separate [cell], so the [Cleaner] action the\n\
                         concrete class registers (see [registerGcHandle]) can settle the\n\
                         release after this handle object is unreachable — the action must\n\
                         never reference the handle itself, or it would keep it alive forever.\n\
                         \n\
                         The untagged→tagged CAS transition ([releaseCell]) is the once-only\n\
                         free ticket: explicit `close()` frees eagerly, `take()`/by-value\n\
                         consumption void the ticket (ownership moved), and the GC action\n\
                         frees only if it wins. Address bits still never change, so the\n\
                         lock-ordering key stays immutable; `isClosed()` and the Rust-side\n\
                         tagged-pointer guards behave exactly as for a plain handle. The GC\n\
                         action needs no monitor: it can only fire when the handle is\n\
                         unreachable, and an in-flight native call holds the handle on its\n\
                         stack.",
                    )
                    .ctor_param(KtCtorParam::new("initialPtr", KtType::long()))
                    .supertype(KtType::cls("NativeHandle"), Some("initialPtr"))
                    .member(
                        KtProperty::val("cell")
                            .ty(KtType::cls("AtomicLong"))
                            .initializer("AtomicLong(initialPtr)")
                            .vis(Vis::Internal),
                    )
                    .member(
                        KtProperty::var("ptr")
                            .ty(KtType::long())
                            .vis(Vis::Internal)
                            .modifier("final override")
                            .accessors(
                                Code::new()
                                    .line("get() = cell.get()")
                                    .line("set(v) { cell.set(v) }"),
                            ),
                    )
                    .member(
                        KtFun::new("markConsumed")
                            .vis(Vis::Internal)
                            .modifier("final override")
                            .body(Code::new().line("releaseCell(cell)")),
                    ),
            )
            .decl(
                KtFun::new("releaseCell")
                    .vis(Vis::Internal)
                    .kdoc(
                        "Win the untagged→tagged release transition of a gc_managed handle's\n\
                         cell: returns the untagged address if the caller now owns the\n\
                         release, else `0` (empty or already tagged — someone else settled\n\
                         it).",
                    )
                    .param(KtParam::new("cell", KtType::cls("AtomicLong")))
                    .returns(KtType::long())
                    .body(
                        Code::new()
                            .line("while (true) {")
                            .line("    val v = cell.get()")
                            .line("    if (v == 0L || (v and 1L) != 0L) return 0L")
                            .line("    if (cell.compareAndSet(v, v or 1L)) return v")
                            .line("}"),
                    ),
            )
            .decl(
                KtClass::object_("NativeCleaner")
                    .vis(Vis::Internal)
                    .kdoc("Shared [Cleaner] settling gc_managed handles' release tickets.")
                    .member(
                        KtProperty::val("CLEANER")
                            .ty(KtType::cls("Cleaner"))
                            .initializer("Cleaner.create()")
                            .annotation("JvmField"),
                    ),
            )
            .decl(
                KtFun::new("registerGcHandle")
                    .vis(Vis::Internal)
                    .kdoc(
                        "Register [handle]'s GC release action, capturing only its cell and\n\
                         the class's `freePtr` (never the handle — that would keep it\n\
                         reachable forever). Returns `null` for a handle born closed.",
                    )
                    .param(KtParam::new("handle", KtType::cls("GcNativeHandle")))
                    .param(KtParam::new(
                        "free",
                        KtType::lambda([("raw".to_string(), KtType::long())], KtType::unit()),
                    ))
                    .returns(KtType::cls("java.lang.ref.Cleaner.Cleanable").nullable())
                    .body(
                        Code::new()
                            .line("if (handle.isClosed()) return null")
                            .line("val c = handle.cell")
                            .line(
                                "return NativeCleaner.CLEANER.register(handle) { val p = releaseCell(c); if (p != 0L) free(p) }",
                            ),
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
            if !cfg.is_opaque() {
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

impl Declarations {
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
            if !cfg.is_enum_class() {
                continue;
            }
            let Some(kotlin_fqn) = cfg.name_spec.as_ref().map(|s| self.fqn_of(s)) else {
                continue;
            };
            // Look up the syn::ItemEnum by the type-key's own short name.
            let Some(name) = key.short_name() else {
                continue;
            };
            // The element: a fieldless `Enum`, which is what an `enum_class!`
            // declares. A sum under the same name is a `Variant` and is not one
            // of these — the model's own distinction, made at parse time.
            let Some(crate::api::core::flat::Type::Enum(item_enum)) =
                registry.flat().declared_type(&name)
            else {
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

    /// Emit one Kotlin `sealed interface` per `sealed_class`-declared type —
    /// the surface of a sum where the target language has sums natively.
    ///
    /// The shape follows the model's own
    /// [`Variant`](crate::api::core::flat::Variant) directly: an alternative
    /// with an empty leaf group becomes a `data object`, one with a payload a
    /// `data class`, both **nested inside** the interface so variant names
    /// cannot collide package-wide. The `fromParts(tag, …slots)` companion is
    /// the reassembly point: it takes the tag plus every variant's slots side
    /// by side and picks the live group, throwing `IllegalArgumentException`
    /// on a tag outside `0..N-1` rather than letting an invalid tag become a
    /// variant.
    ///
    /// A unit-only enum never reaches here — that is `enum_class!`'s shape,
    /// and each declarator rejects the other's.
    pub(crate) fn write_sealed_classes(
        &self,
        registry: &Registry<KotlinMeta>,
    ) -> Result<Vec<kt::KtFile>, WriteKotlinError> {
        let mut written = Vec::new();
        // Deterministic order by canonical Rust type-key.
        let mut keys: Vec<&TypeKey> = self.types.keys().collect();
        keys.sort_by(|a, b| a.as_str().cmp(b.as_str()));
        for key in keys {
            let cfg = &self.types[key];
            let Some(sum_cfg) = cfg.sum() else {
                continue;
            };
            let Some(kotlin_fqn) = cfg.name_spec.as_ref().map(|s| self.fqn_of(s)) else {
                continue;
            };
            let Some(ident) = key.ident() else {
                continue;
            };
            // The sum as the MODEL holds it: its alternatives' payloads are
            // `TypeRef`s, so the Kotlin type of one asks nothing and cannot be
            // asked about a type the model never saw.
            //
            // A FIELDLESS enum is `Type::Enum`, not `Type::Variant` — the model
            // already draws the distinction this arm used to re-derive with
            // `enum_shape`. It is a declaration error rather than a skip, so it
            // keeps its diagnosis; only the source of the answer changed.
            let declared = registry.flat().declared_type(&ident);
            assert!(
                matches!(declared, Some(crate::api::core::flat::Type::Variant(_))),
                "`{}` has no payload variants: declare it with `enum_class!({})`, not \
                 `sealed_class!({})` — a fieldless enum crosses as a bare discriminant and \
                 needs no sealed hierarchy",
                ident,
                ident,
                ident
            );
            let Some(crate::api::core::flat::Type::Variant(sum)) = declared else {
                unreachable!("asserted just above")
            };

            // Every declared `.variant(...)` must name a real variant —
            // a typo would otherwise silently do nothing.
            for declared in sum_cfg.variant_names.keys() {
                assert!(
                    sum.alternatives.iter().any(|a| a.name == *declared),
                    "sealed_class!({ident}): variant!({declared}) does not name a variant of \
                     `{ident}`"
                );
            }

            let (package, class_name) = match kotlin_fqn.rsplit_once('.') {
                Some((p, c)) => (p.to_string(), c.to_string()),
                None => (String::new(), kotlin_fqn.clone()),
            };
            let mut class = self.build_sealed_class(registry, &class_name, sum, sum_cfg);
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

    /// The `sealed interface` model for one sum: nested variant classes plus
    /// the `fromParts` companion. Payload Kotlin types come from the same
    /// resolved converter metadata a data-class field reads, so a sum's
    /// payload and a struct's field of the same Rust type surface
    /// identically.
    fn build_sealed_class(
        &self,
        registry: &Registry<KotlinMeta>,
        class_name: &str,
        sum: &crate::api::core::flat::Variant,
        sum_cfg: &SumConfig,
    ) -> KtClass {
        // Everything below comes off the element: `alternatives` for the
        // classes, `Field::member()` for the property names, `docs()` for the
        // prose the source wrote.
        let framework_line = format!(
            "JVM-side surface for the native Rust `{}` sum: exactly one alternative is live.",
            sum.name
        );
        let kdoc = sum
            .docs()
            .map(|d| format!("{d}\n\n{framework_line}"))
            .unwrap_or(framework_line);

        let mut class = KtClass::new(ClassKind::SealedInterface, class_name)
            .vis(Vis::Public)
            .kdoc(kdoc);

        // Nested variant classes, in declaration (tag) order.
        for alt in &sum.alternatives {
            let vname = self.sum_variant_class_name(sum_cfg, &alt.name);
            let mut vclass = if alt.is_empty() {
                KtClass::new(ClassKind::DataObject, &vname)
            } else {
                KtClass::new(ClassKind::Data, &vname)
            }
            .vis(Vis::Public)
            .supertype(KtType::cls(class_name), None);
            if let Some(doc) = alt.docs() {
                vclass = vclass.kdoc(doc);
            }
            let mut vprops: Vec<(String, KtType)> = Vec::new();
            for field in &alt.fields {
                let prop = sum_field_prop_name(&field.member());
                let ty = self.sum_payload_kt_type(registry, &sum.name, &alt.name, &prop, field);
                vprops.push((prop.clone(), ty.clone()));
                vclass = vclass.ctor_param(KtCtorParam::new(&prop, ty).val().vis(Vis::Public));
            }
            // An array-backed payload (a `Vec<u8>` variant field) compares by
            // identity otherwise — same rule as a data-class property.
            for m in
                crate::api::lang::jnigen::jni::equality::content_equality_members(&vname, &vprops)
                    .into_iter()
                    .flatten()
            {
                vclass = vclass.member(m);
            }
            class = class.member(vclass);
        }

        // `fromParts(tag, …)` — the tag slot plus every variant's slots side
        // by side, in the same order both sides enumerate them.
        let mut factory = KtFun::new("fromParts")
            .vis(Vis::Public)
            .annotation("JvmStatic")
            .param(KtParam::new("tag", KtType::int()))
            .returns(KtType::cls(class_name));
        for alt in &sum.alternatives {
            let vname = self.sum_variant_class_name(sum_cfg, &alt.name);
            for field in &alt.fields {
                let prop = sum_field_prop_name(&field.member());
                let ty = self.sum_payload_kt_type(registry, &sum.name, &alt.name, &prop, field);
                factory = factory.param(KtParam::new(sum_slot_fragment(&vname, &prop), ty));
            }
        }
        let mut body = Code::new();
        body = body.blk("when (tag) {", |mut w| {
            for alt in &sum.alternatives {
                let vname = self.sum_variant_class_name(sum_cfg, &alt.name);
                let args: Vec<String> = alt
                    .fields
                    .iter()
                    .map(|f| sum_slot_fragment(&vname, &sum_field_prop_name(&f.member())))
                    .collect();
                let ctor = if alt.is_empty() {
                    vname
                } else {
                    format!("{vname}({})", args.join(", "))
                };
                // The same tag the selector leaf carries — a `when` arm that
                // disagreed with the wire value would simply never match.
                w = w.line(format!("{} -> {ctor}", sum_tag(alt)));
            }
            w.line(format!(
                "else -> throw IllegalArgumentException(\"{class_name}: invalid tag $tag\")"
            ))
        });
        factory = factory.expr_body(body);

        // The companion is named only when it has to be (a variant took
        // `Companion`), so ordinary emission keeps the anonymous form.
        let mut companion = KtClass::companion_object().vis(Vis::Public).member(factory);
        let companion_name = self.sum_companion_name(sum_cfg, sum);
        if companion_name != "Companion" {
            companion.name = companion_name;
        }
        class.companion(companion)
    }

    /// Kotlin name of the companion object holding a sum's `fromParts`.
    ///
    /// Normally the implicit `Companion`. A variant class of that name is a
    /// "Conflicting declarations" error in Kotlin — but the colliding name is
    /// **ours**, an artifact of emitting a companion at all, not something
    /// the language reserves. So the generator moves rather than making the
    /// source crate rename a legitimate variant: the companion takes a
    /// trailing `_` until it is free, the same escape
    /// [`mangle_kotlin_ident`] uses for a taken name.
    ///
    /// Renaming it is invisible on the wire: `@JvmStatic` on a *named*
    /// companion still emits `fromParts` as a real static method on the
    /// interface class, which is what `call_static_method` /
    /// `GetStaticMethodID` resolve.
    pub(crate) fn sum_companion_name(
        &self,
        sum_cfg: &SumConfig,
        sum: &crate::api::core::flat::Variant,
    ) -> String {
        // The alternatives, not the item's `variants`: the same list, already
        // classified, and named the way the model names them.
        let taken: std::collections::HashSet<String> = sum
            .alternatives
            .iter()
            .map(|a| self.sum_variant_class_name(sum_cfg, &a.name))
            .collect();
        let mut name = "Companion".to_string();
        while taken.contains(&name) {
            name.push('_');
        }
        name
    }

    /// The Kotlin class name of one variant: its `.variant(variant!(V).name())`
    /// override, else the Rust variant ident (already PascalCase by
    /// convention), sanitized for Kotlin.
    pub(crate) fn sum_variant_class_name(
        &self,
        sum_cfg: &SumConfig,
        variant: &syn::Ident,
    ) -> String {
        let rust = variant.to_string();
        match sum_cfg.variant_names.get(&rust) {
            Some(n) => n.clone(),
            None => mangle_kotlin_ident(&rust),
        }
    }

    /// Kotlin type of one payload field, derived from the **output**
    /// converter entry — one direction, authoritatively.
    ///
    /// A sealed class is a bidirectional contract: the Rust output encoder
    /// builds it through `fromParts`, and the Kotlin input destructure reads
    /// the same properties back out. The property's Kotlin type, its
    /// nullability and its wire slot are therefore **one** decision, and
    /// every fact behind it has to come from the same entry — deriving the
    /// type from whichever direction happens to have resolved while reading
    /// the wire from the other is how the two flatten paths drift apart.
    /// Output is the authoritative side because this emitter declares the
    /// `fromParts` slots the output encoder fills.
    ///
    /// So there is deliberately **no** output-then-input fallback here:
    ///
    /// * a missing output entry is a generation error naming the payload,
    ///   rather than a Kotlin surface quietly emitted for a direction that
    ///   never resolved;
    /// * an input entry that disagrees on the Kotlin type is a generation
    ///   error too — that disagreement is exactly the drift the shared plans
    ///   exist to prevent, and it must surface at the declaration, not as an
    ///   ABI mismatch at runtime.
    fn sum_payload_kt_type(
        &self,
        registry: &Registry<KotlinMeta>,
        sum_name: &syn::Ident,
        variant: &syn::Ident,
        prop: &str,
        field: &crate::api::core::flat::Field,
    ) -> KtType {
        // The field's own reading: the nullability question below is answered
        // from `kind`, so a wrapped spelling answers as the bare one does and
        // nothing is looked up (#275).
        let field_ty = field.ty.spell();
        let where_ = || format!("sealed_class!({}) payload `{variant}.{prop}`", sum_name);
        let out = registry.output_entry(&field.ty).unwrap_or_else(|| {
            panic!(
                "{}: `{}` has no resolved OUTPUT converter, so the Kotlin surface for it \
                 cannot be derived — register converters for the payload type before \
                 declaring the sealed class",
                where_(),
                field_ty.to_token_stream(),
            )
        });

        if let Some(h) = out.metadata.projection.clone() {
            let leaf = projection_leaf_kt(self, &h).unwrap_or_else(|| {
                panic!(
                    "{}: leaf `{}` has no Kotlin FQN registered (ptr_class)",
                    where_(),
                    h.leaf_key
                )
            });
            return handle_kt_type(&h.strategy, &leaf);
        }

        let ty = out.metadata.kotlin_name.clone().unwrap_or_else(|| {
            panic!(
                "{}: `{}` has no Kotlin type mapping on its output converter",
                where_(),
                field_ty.to_token_stream(),
            )
        });
        // The input side must agree on WHICH TYPE the property is — Kotlin
        // reads these very properties back when the value crosses the other
        // way, so a genuine disagreement (`String` in, `Long` out) is drift
        // that belongs at the declaration.
        //
        // Nullability is deliberately excluded from the comparison: the two
        // directions record it by different conventions. For `Option<T>` the
        // output converter's name carries the `?` while the input converter's
        // does not, because the input side expresses absence in the wire (a
        // boxed value, a present flag, a niche) rather than in the type name.
        // Comparing the rendered types would reject that legitimate shape —
        // which is what an `Option<enum>` payload does.
        if let Some(inp) = registry.input_entry(&field.ty) {
            if let (Some(in_ty), (Some(a), Some(b))) = (
                inp.metadata.kotlin_name.clone(),
                (
                    inp.metadata
                        .kotlin_name
                        .as_ref()
                        .and_then(|t| t.leaf_name())
                        .map(str::to_string),
                    ty.leaf_name().map(str::to_string),
                ),
            ) {
                assert!(
                    a == b,
                    "{}: the input and output converters for `{}` disagree on its Kotlin \
                     type (`{}` in, `{}` out) — a sealed class's properties are read by both \
                     directions, so they must map to one type",
                    where_(),
                    field_ty.to_token_stream(),
                    in_ty,
                    ty,
                );
            }
        }
        // An `Option<T>` payload whose wire is a JNI primitive is encoded as
        // the bare primitive with a sentinel, exactly as for a data-class
        // field — the Kotlin type must match that slot. Read from the same
        // entry the type came from.
        let primitive_wire = crate::api::lang::jnigen::jni::is_jni_primitive(&out.destination);
        if field.ty.optional_inner().is_some() && !primitive_wire {
            ty.nullable()
        } else {
            ty
        }
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
            // Opaque handles, enums and sealed classes each have their own
            // emitter; only plain structs become data classes here.
            if cfg.special_decl() {
                continue;
            }
            let Some(kotlin_fqn) = cfg.name_spec.as_ref().map(|s| self.fqn_of(s)) else {
                continue;
            };

            let Some(name) = key.short_name() else {
                continue;
            };
            let Some(item_struct) = registry.flat().struct_type(&name) else {
                continue;
            };

            let (package, class_name) = match kotlin_fqn.rsplit_once('.') {
                Some((p, c)) => (p.to_string(), c.to_string()),
                None => (String::new(), kotlin_fqn.clone()),
            };
            if item_struct.name != class_name {
                aliases.push((item_struct.name.to_string(), class_name.clone()));
            }
            let mut class = build_data_class(self, &class_name, item_struct, registry);
            // The data class is self-contained (property/factory types +
            // factory-body imports ride the AST/`Code`); this file-level set
            // is only for the `JNINative` harness the promoted members call.
            let mut imports: BTreeSet<String> = BTreeSet::new();
            // Members: the instance
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
            for m in members.iter().filter(|m| m.kind == MemberKind::Method) {
                if let Some(item_fn) = registry.flat().function(&m.rust_ident) {
                    if let Some(f) = crate::api::lang::jnigen::jni::render_wrapper_fn(
                        self,
                        item_fn,
                        registry,
                        Some(self.effective_method_name(key, m).as_str()),
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
                    if let Some(item_fn) = registry.flat().function(&m.rust_ident) {
                        if let Some(f) = crate::api::lang::jnigen::jni::render_wrapper_fn(
                            self,
                            item_fn,
                            registry,
                            Some(self.effective_method_name(key, m).as_str()),
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
    /// One top-level safe wrapper per `FunctionEntry` in `pkg_cfg.functions`.
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

        // Distinct interface identities in use — [`SpecKey`] (`Ord`, so
        // emission is deterministic). The memo derives each spec from the
        // key alone, so no side context is carried.
        let mut uses: BTreeSet<SpecKey> = BTreeSet::new();

        /// A hoisted-singleton request emitted alongside an interface: the
        /// `fromParts` builder / folder for a synthesized `data_class`, or the
        /// single-leaf appender for a whole-element leaf fold. The wrapper
        /// references the singleton instead of taking a caller `build`/`fold`.
        enum FixedSingleton {
            StructBuilder(DeconId),
            StructFolder(DeconId),
            /// A decomposed **sum**: the reassembly is a `when` over the tag
            /// picking the live group, not a `fromParts` over a fixed product.
            SumBuilder(DeconId),
            SumFolder(DeconId),
            LeafFolder,
        }
        // A decomposition is a sum's when it carries the synthesized selector.
        let is_sum = |d: &DeconId| {
            registry
                .decon_plans()
                .get(d)
                .is_some_and(|p| is_sum_leaves(&p.leaves))
        };

        // Fixedness sets shared with the memo derivation (`iface.rs`): a
        // fixed DeconId gets a hoisted `__<Name>Builder` / folder-appender
        // singleton emitted alongside its interface; a fixed whole-element
        // key gets the `__<Elem>FolderRaw` appender, the leaf dual.
        let fixed_decons = fixed_decon_ids(registry);
        let fixed_leaf_elements = fixed_leaf_element_keys(registry);

        // Walk every declared function — free `.fun`s AND class methods/factories
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
                // The ELEMENT, so the callback params come off the model's own
                // classification rather than a second walk of the bounds.
                let Some(func) = registry.flat().function(&ident) else {
                    continue;
                };
                let item_fn = func.origin.as_syn();
                for p in &func.params {
                    if let Some(cb_args) = p.ty.callback_args() {
                        uses.insert(SpecKey::callback(cb_args));
                    }
                }
                if let Some(plan) = registry
                    .unfold_plans()
                    .get(&item_fn.sig.ident)
                    .filter(|p| p.delivery == Delivery::Callback)
                {
                    let iterable = is_iterable_fold(&plan.shape);
                    match (iterable, &plan.element, &plan.decon) {
                        (true, Some(el), _) => {
                            uses.insert(SpecKey::whole_folder(el));
                        }
                        (true, None, Some(d)) => {
                            uses.insert(SpecKey::Folder(d.clone()));
                        }
                        (false, _, Some(d)) => {
                            uses.insert(SpecKey::Builder(d.clone()));
                        }
                        _ => {}
                    }
                }
                match registry.error_plans().get(&item_fn.sig.ident) {
                    Some(ep) => {
                        let d = ep
                            .decon
                            .clone()
                            .expect("error plans are always record-built (decon is Some)");
                        uses.insert(SpecKey::Handler(d));
                    }
                    None => {
                        uses.insert(SpecKey::JniErrorHandler);
                    }
                }
            }
        }

        uses.into_iter()
            .filter_map(|u| {
                // Every spec comes from the SAME memo the wrappers and the
                // resolve-time trampoline read ([`Declarations::iface_spec`]) —
                // this site only classifies the extras: `is_error` ⇒ also
                // emit the zero-alloc capture holder used by the generated
                // wrappers' error channel; `fixed` carries a
                // hoisted-singleton request (the `fromParts` builder /
                // folder-appender for a synthesized `data_class`, or the
                // single-leaf appender for a fixed whole-element fold).
                let (is_error, fixed) = match &u {
                    SpecKey::Callback(_) => (false, None),
                    SpecKey::Builder(d) => (
                        false,
                        fixed_decons.contains(d).then(|| {
                            if is_sum(d) {
                                FixedSingleton::SumBuilder(d.clone())
                            } else {
                                FixedSingleton::StructBuilder(d.clone())
                            }
                        }),
                    ),
                    SpecKey::Folder(d) => (
                        false,
                        fixed_decons.contains(d).then(|| {
                            if is_sum(d) {
                                FixedSingleton::SumFolder(d.clone())
                            } else {
                                FixedSingleton::StructFolder(d.clone())
                            }
                        }),
                    ),
                    SpecKey::WholeFolder(el_key) => (
                        false,
                        fixed_leaf_elements
                            .contains(el_key)
                            .then_some(FixedSingleton::LeafFolder),
                    ),
                    SpecKey::Handler(_) | SpecKey::JniErrorHandler => (true, None),
                };
                self.iface_spec(registry, &u).map(|s| (s, is_error, fixed))
            })
            .map(|(s, is_error, fixed)| {
                // A **fixed** builder/folder has no user-facing side: the only
                // implementation is the hoisted singleton emitted below, which
                // implements the RAW twin, and the wrapper passes that singleton
                // to the native call. So the typed interface and its `asRaw`
                // proxy would be dead public API — and for a sum actively
                // misleading, since `asRaw` wraps every group's slots as if all
                // were live when exactly one ever is (issue #160). Emit neither.
                //
                // Only when there IS a twin: with no twin, `raw_name() == name`,
                // so `to_decl()` *is* the interface the singleton implements and
                // JNI calls.
                let typed_is_dead = fixed.is_some() && s.needs_raw();
                let mut file = kt::KtFile::new(s.package.clone());
                if !typed_is_dead {
                    file = file.decl(s.to_decl());
                }
                // Typed (user-facing) interface; when any leaf's raw view
                // differs, also the JNI-called raw twin and the `asRaw()`
                // proxy adapter that wraps raw leaves into typed objects.
                if s.needs_raw() {
                    file = file.decl(s.to_raw_decl());
                    if !typed_is_dead {
                        file = file.decl(s.to_as_raw_fun());
                    }
                    // Kept even when the proxy is suppressed: a hoisted
                    // singleton names the same classes by short name from its
                    // own raw text (`Class.fromParts(…)`, a `wrap` expression).
                    for p in &s.params {
                        if let Some(fqn) = p.wrap.class_fqn() {
                            file = file.import(fqn.to_string());
                        }
                    }
                    // A group's reassembly is raw text (`Reading.Exact(…)`,
                    // `Priority.fromInt(…)`), so the classes it names by short
                    // name need importing here.
                    for g in &s.typed_groups {
                        for fqn in &g.imports {
                            file = file.import(fqn.clone());
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
                        FixedSingleton::SumBuilder(decon) => {
                            self.sum_builder_singleton(registry, &s, &decon)
                        }
                        FixedSingleton::SumFolder(decon) => {
                            self.sum_folder_singleton(registry, &s, &decon)
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
        let source = &registry.decon_plans()[decon].source;
        let class_fqn = self
            .kotlin_fqn(&source.key())
            .unwrap_or_else(|| panic!("value-struct builder: no Kotlin FQN for {}", source.key()));
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
        let source = &registry.decon_plans()[decon].source;
        let class_fqn = self
            .kotlin_fqn(&source.key())
            .unwrap_or_else(|| panic!("value-struct folder: no Kotlin FQN for {}", source.key()));
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

    /// The hoisted **fixed builder** singleton for a decomposed **sum**: the
    /// sum's dual of [`Self::value_struct_builder_singleton`], with a `when`
    /// over the tag in place of a `fromParts` over a fixed product.
    ///
    /// The sealed interface's own `fromParts` is deliberately NOT the target
    /// here. That factory is the Kotlin-facing convenience stage C emits — its
    /// parameters are the variants' **property** types, not the wire, and its
    /// object slots are non-null, which an inert group's `JObject::null()`
    /// would trip on before any code runs. The reassembly the wire needs is the
    /// same inlined `when` a sum-typed struct field already gets from
    /// `factory_field`, so it is emitted here directly.
    fn sum_builder_singleton(
        &self,
        registry: &Registry<KotlinMeta>,
        spec: &crate::api::lang::jnigen::jni::IfaceSpec,
        decon: &crate::api::core::unfold::DeconId,
    ) -> kt::KtDecl {
        let plan = &registry.decon_plans()[decon];
        let mut imports: BTreeSet<String> = BTreeSet::new();
        let names: Vec<String> = spec.params.iter().map(|p| p.name.clone()).collect();
        let (iface_short, when) = self.sum_reconstruct(
            registry,
            &plan.source.key(),
            &plan.leaves,
            &spec.params,
            &names,
            &mut imports,
        );
        let builder = spec.raw_name();
        let val_name = format!("__{builder}");
        let code = format!(
            "internal val {val_name}: {builder}<{iface_short}> =\n    \
             {builder} {{ {} ->\n    {when}\n}}",
            names.join(", "),
        );
        let mut body = kt::Code::raw_reindent(&code);
        for fqn in imports {
            body = body.import(fqn);
        }
        kt::KtDecl::Raw {
            name: val_name,
            code: body,
        }
    }

    /// The hoisted **folder-appender** singleton for a `Vec<sum>` element fold:
    /// per element, pick the live alternative by tag and append it to the
    /// accumulator `ArrayList`. The sum's dual of
    /// [`Self::value_struct_folder_singleton`]; the folder's `run` params are
    /// `[acc, tag, group-slots…]`, so the reassembly reads all but `acc`.
    fn sum_folder_singleton(
        &self,
        registry: &Registry<KotlinMeta>,
        spec: &crate::api::lang::jnigen::jni::IfaceSpec,
        decon: &crate::api::core::unfold::DeconId,
    ) -> kt::KtDecl {
        let plan = &registry.decon_plans()[decon];
        let mut imports: BTreeSet<String> = BTreeSet::new();
        let names: Vec<String> = spec.params.iter().map(|p| p.name.clone()).collect();
        let (iface_short, when) = self.sum_reconstruct(
            registry,
            &plan.source.key(),
            &plan.leaves,
            &spec.params[1..],
            &names[1..],
            &mut imports,
        );
        let folder = spec.raw_name();
        let holder = spec.singleton_holder_name();
        let field = crate::api::lang::jnigen::jni::SINGLETON_FIELD;
        let acc = &names[0];
        let acc_ty = format!("ArrayList<{iface_short}>");
        let code = format!(
            "internal object {holder} {{\n    \
             @JvmField\n    \
             val {field}: {folder}<{acc_ty}> =\n        \
             {folder} {{ {} -> {acc}.add({when}); {acc} }}\n\
             }}",
            names.join(", "),
        );
        let mut body = kt::Code::raw_reindent(&code);
        for fqn in imports {
            body = body.import(fqn);
        }
        kt::KtDecl::Raw {
            name: holder,
            code: body,
        }
    }

    /// The wire-shaped reassembly of one decomposed sum: `(interface short
    /// name, when-expression)`.
    ///
    /// `params` are the interface's `run` parameters **aligned with the plan's
    /// leaves** (the selector first, then the groups) — so the caller strips a
    /// folder's leading `acc`. Each group leaf's parameter is unwrapped into
    /// its variant-constructor argument by [`Self::sum_ctor_arg`].
    pub(crate) fn sum_reconstruct(
        &self,
        registry: &impl Conversions<KotlinMeta>,
        // The sum's **identity**: every use of `source` here was
        // `TypeKey::from_type` or `bare_path_ident`, and a key that is one
        // identifier IS the ident — the same reduction `type_kind` made.
        key: &TypeKey,
        leaves: &[crate::api::core::unfold::UnfoldLeaf],
        params: &[crate::api::lang::jnigen::jni::IfaceParam],
        names: &[String],
        imports: &mut BTreeSet<String>,
    ) -> (String, String) {
        let iface_fqn = self
            .kotlin_fqn(key)
            .unwrap_or_else(|| panic!("sum builder: no Kotlin FQN for {key}"));
        let iface_short = register_fqn(&iface_fqn, imports);
        let ident = key
            .ident()
            .unwrap_or_else(|| panic!("sum builder: `{key}` is not a path type"));
        let Some(crate::api::core::flat::Type::Variant(sum)) =
            registry.flat().declared_type(&ident)
        else {
            panic!("sum builder: `{ident}` is not an indexed sum")
        };
        let sum_cfg = self.types[key]
            .sum()
            .unwrap_or_else(|| panic!("sum builder: `{ident}` is not a sealed class"));
        let tag = &names[0];

        let mut arms: Vec<String> = Vec::new();
        for alt in &sum.alternatives {
            let group = sum_tag(alt);
            let vname = self.sum_variant_class_name(sum_cfg, &alt.name);
            let args: Vec<String> = leaves
                .iter()
                .zip(params)
                .zip(names)
                .filter(|((l, _), _)| l.group == Some(group))
                .map(|((l, p), n)| self.sum_ctor_arg(registry, l, p, n, imports))
                .collect();
            // Kotlin has no `B()` / `B {}` distinction to keep: a payload-less
            // alternative is a `data object`, named bare. The Rust side is where
            // the delimiters matter, and `Alternative::spell` owns them there.
            let ctor = if args.is_empty() {
                format!("{iface_short}.{vname}")
            } else {
                format!("{iface_short}.{vname}({})", args.join(", "))
            };
            arms.push(format!("{group} -> {ctor}"));
        }
        // A NULLABLE selector carries the absent case of a conditional value
        // form: null in means null out. Without this arm the `when` would fall
        // through to the invalid-tag throw, and boxing the absence as tag 0
        // would alias whichever variant that really is.
        if leaves[0].nullable {
            arms.insert(0, "null -> null".to_string());
        }
        let when = format!(
            "when ({tag}) {{ {}; else -> throw IllegalArgumentException(\"{iface_short}: invalid \
             tag ${tag}\") }}",
            arms.join("; "),
        );
        (iface_short, when)
    }

    /// The variant-constructor argument for one group leaf: its `run`
    /// parameter, un-inerted and wrapped back into the property type.
    ///
    /// Two independent transforms, in this order:
    ///
    /// 1. **Un-inert** — an object-shaped group slot is declared nullable
    ///    (an inert group arrives as JVM null), so inside its own live arm it is
    ///    re-asserted with `!!`. A payload that is *itself* optional
    ///    (`Option<T>`) keeps its null: there the JVM null means `None`, and
    ///    `!!` would turn a legitimately absent value into an exception.
    /// 2. **Wrap** — the raw wire becomes the property: an enum discriminant
    ///    through `fromInt`, a handle/blob/`ULong` through the interface's own
    ///    [`WrapKind`](crate::api::lang::jnigen::jni::WrapKind), everything else
    ///    verbatim.
    fn sum_ctor_arg(
        &self,
        registry: &impl Conversions<KotlinMeta>,
        leaf: &crate::api::core::unfold::UnfoldLeaf,
        param: &crate::api::lang::jnigen::jni::IfaceParam,
        name: &str,
        imports: &mut BTreeSet<String>,
    ) -> String {
        // Off the leaf's own reading — no lookup, and a wrapped spelling
        // answers as the bare one does.
        let optional = leaf.out_ty.optional_inner().is_some();
        let arg = if param.raw.is_nullable() && !optional {
            format!("{name}!!")
        } else {
            name.to_string()
        };
        // An enum payload rides its `jint` discriminant, so the interface types
        // it `Int` and the wrap has to name the enum class itself — read off the
        // same output-converter metadata `factory_field` reads for an enum
        // struct field.
        if self.is_kotlin_enum_reading(&leaf.out_ty) {
            // The `Option` layer peeled off the model, so the entry lookup takes
            // the layer's own reading instead of a spelling to look back up.
            let inner = leaf.out_ty.optional_inner().unwrap_or(&leaf.out_ty);
            let name = registry
                .output_entry(inner)
                .and_then(|e| e.metadata.kotlin_name.clone())
                .and_then(|t| t.leaf_name().map(str::to_string))
                .unwrap_or_else(|| {
                    panic!(
                        "sum builder: enum payload `{}` has no Kotlin type on its output converter",
                        leaf.name
                    )
                });
            let short = register_fqn(&name, imports);
            return if optional {
                format!("{arg}?.let {{ {short}.fromInt(it) }}")
            } else {
                format!("{short}.fromInt({arg})")
            };
        }
        param.wrap.wrap_expr(&arg, false)
    }

    /// The hoisted **folder-appender** singleton for a **whole single-leaf
    /// element** fold (`Vec<String>` / `Vec<handle>` return, or the matching
    /// slice callback): an instance of the folder's raw twin (`__<Elem>FolderRaw`)
    /// that, per element, wraps the raw leaf into its typed Kotlin value and
    /// appends it to the accumulator `ArrayList`, returning the same list. The
    /// single-leaf analog of [`Self::value_struct_folder_singleton`] — there is no
    /// `fromParts`; reassembly is just `acc.add(<wrap>(element))`, where `<wrap>`
    /// is the handle ctor for a handle, `toULong()` for a `u64`, or
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
        let package = self.package_name(subpackage);
        let mut file = kt::KtFile::new(&package);
        let mut imports: BTreeSet<String> = BTreeSet::new();
        for entry in &pkg_cfg.functions {
            let item_fn = &registry
                .flat()
                .function(&entry.rust_ident)
                .unwrap_or_else(|| {
                    panic!(
                        "write_jni_package: function `{}` registered via .function(...) is \
                         not in the prebindgen registry — check the spelling against the \
                         matching `#[prebindgen]` Rust fn name.",
                        entry.rust_ident,
                    )
                });
            let kotlin_name = self.effective_function_name(subpackage, entry);
            if let Some(f) = render_wrapper_fn(self, item_fn, registry, Some(&kotlin_name), None) {
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
            let item_const = registry
                .flat()
                .constant(&entry.rust_ident)
                .unwrap_or_else(|| {
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
                &package,
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
            let item_fn = &registry
                .flat()
                .function(&entry.rust_ident)
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
                &package,
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
            if let Some((helper, prop)) =
                render_const_expr_val(self, &package, decl, registry, &mut imports)
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
    /// (class name from [`Declarations::jni_native_class_name`]). Holds one
    /// `external fun` per `#[prebindgen]` function — names mangled as methods
    /// via [`JniGenBuilder::set_method_name_mangle`], parameter and return types rendered at
    /// the JNI **wire** level so the declarations match the Rust extern
    /// symbols generated under the spec-escaped
    /// `Java_<package>_<jni_native_class>_<name>` (see `symbol`, #86). Every generated native
    /// call routes through this object, so its static initializer is the
    /// single point at which native-library loading can be triggered: when
    /// [`Declarations::jni_native_init`] is set, its Kotlin statement(s) are emitted
    /// inside an `init { … }` block here (e.g. a reference to the consumer's
    /// own loader object). Unset, the holder stays free of any loading logic
    /// and the wrapper layer is responsible for loading.
    pub(crate) fn write_jni_native(&self, registry: &Registry<KotlinMeta>) -> kt::KtFile {
        let class_name = self.jni_native_class_name();
        let declared = self.declared_functions();

        // Each extern is a `KtFun` member of the object; the AST renderer
        // shortens types, collects imports, and wraps long signatures (no
        // derivation-time import set).
        let mut externs: Vec<kt::KtFun> = Vec::new();
        let mut fns: Vec<&crate::api::core::flat::Function> = registry.flat().functions().collect();
        fns.sort_by(|a, b| a.name.cmp(&b.name));
        for f in fns {
            if !declared.contains(&f.name) {
                continue;
            }
            if let Some(fun) = render_extern_decl(self, f, registry) {
                externs.push(fun);
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
            let Some(item_const) = registry.flat().constant(&ident) else {
                continue; // missing decl already warned by the scan
            };
            let getter = crate::api::lang::jnigen::jni::const_getter_fn(item_const);
            if let Some(fun) = render_extern_decl(self, &getter, registry) {
                externs.push(fun);
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
            let getter = const_expr_getter_fn(&decl.kotlin_name, &decl.ty, registry);
            if let Some(fun) = render_extern_decl(self, &getter, registry) {
                externs.push(fun);
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
            // `New(cap: Int): Long`, `Push(handle: Long, <leaves…>)`,
            // `Free(handle: Long)`.
            externs.push(
                kt::KtFun::new(new_m)
                    .modifier("external")
                    .param(kt::KtParam::new("cap", kt::KtType::int()))
                    .returns(kt::KtType::long()),
            );
            let mut push = kt::KtFun::new(push_m)
                .modifier("external")
                .param(kt::KtParam::new("handle", kt::KtType::long()));
            for leaf in h.plan.leaves.iter().filter(|l| !l.is_present_flag) {
                push = push.param(kt::KtParam::new(
                    leaf.kt_name.clone(),
                    kt::KtType::cls(leaf.kt_wire_ty.clone()),
                ));
            }
            externs.push(push);
            externs.push(
                kt::KtFun::new(free_m)
                    .modifier("external")
                    .param(kt::KtParam::new("handle", kt::KtType::long())),
            );
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
        // Each `external fun` is an object member; the AST renderer collects
        // their imports from the (full-FQN) parameter/return types.
        for fun in externs {
            obj = obj.member(fun);
        }
        kt::KtFile::new(&self.package).decl(obj)
    }

    /// Emit one Kotlin file per entry in `handles` — each becomes a
    /// `public class <ClassName>(initialPtr: Long) : NativeHandle(initialPtr)`
    /// with the standard `free()` + package/class-aware mangled
    /// `private external fun freePtr(ptr: Long)`
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
    /// [`Declarations::kotlin_fqn`] so the generator can map it back to its
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
        package: &str,
        class_short: &str,
        override_: Option<&str>,
    ) -> String {
        let name = self.interface_short_name_unchecked(package, class_short, override_);
        // The class-vs-interface collision is reported as a collected error
        // by `validate_symbols` (issue #89) before any file is written, so
        // this assert is an unreachable backstop for the emission path.
        assert!(
            name != class_short,
            "the generated interface name `{name}` must differ from the class name \
             `{class_short}` (a class and its interface cannot share a name in one package)"
        );
        name
    }

    /// The interface short name without the class-collision assert — the
    /// non-panicking core, used by `validate_symbols` to build the
    /// per-package name table (where the collision surfaces as a collected
    /// error naming both origins).
    pub(crate) fn interface_short_name_unchecked(
        &self,
        package: &str,
        class_short: &str,
        override_: Option<&str>,
    ) -> String {
        match override_ {
            Some(n) => n.to_string(),
            None => match &self.interface_name_mangle {
                Some(f) => f(package, class_short),
                None => format!("{class_short}Api"),
            },
        }
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

        let class_fqn = self
            .kotlin_fqn(key)
            .unwrap_or_else(|| class_short.to_string());
        let package = class_fqn.rsplit_once('.').map(|(p, _)| p).unwrap_or("");
        let iface_name = self.interface_short_name(package, class_short, name_override.as_deref());
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
