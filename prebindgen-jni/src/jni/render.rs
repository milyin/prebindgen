//! Kotlin source-file renderers (enum / data-class / typed-handle /
//! package / native / wrapper-fn) for the JNI back-end.
//!
//! Carved from the former `jni_kotlin_ext.rs`; shares the `jni` namespace
//! via `use super::*`.

use kotlin_codegen::{
    KtClass, KtCode, KtCompanion, KtCtorParam, KtEnumEntry, KtFun, KtParam, KtProperty, KtType,
    KtVis,
};

use super::*;

// ── Safe-wrapper emitters ──────────────────────────────────────────────

/// One generated Kotlin `enum class` source — variants in
/// SCREAMING_SNAKE_CASE, each carrying the Rust discriminant as a
/// `val value: Int`, plus a `fromInt(value: Int)` companion. Mirrors
/// the hand-written `io.zenoh.qos.Priority` shape so adapter code that
/// already speaks the `.value` / `.fromInt(...)` idiom keeps working.
pub(crate) fn build_enum_class(
    class_name: &str,
    item_enum: &prebindgen_registry::flat::Enum,
) -> KtClass {
    // Same discriminant source of truth the Rust `jint → variant` decode
    // uses, so Kotlin `value(N)` and the generated decode agree — and it is the
    // model's, which is where "same" stops needing to be maintained.
    let entries: Vec<KtEnumEntry> = item_enum
        .discriminant_values()
        .unwrap_or_else(|name| {
            panic!(
                "enum `{}` variant `{name}` has a non-literal discriminant; use a literal \
                 integer value (e.g. `= 1`) or an implicit discriminant",
                item_enum.name
            )
        })
        .into_iter()
        .map(|(ident, value)| {
            KtEnumEntry::with_args(
                mangle_kotlin_ident(&crate::util::camel_to_screaming_snake(&ident.to_string())),
                value.to_string(),
            )
        })
        .collect();

    let framework_line = format!(
        "JVM-side surface for the native Rust `{}` enum.",
        item_enum.name
    );
    let enum_kdoc = item_enum
        .docs()
        .map(|d| format!("{d}\n\n{framework_line}"))
        .unwrap_or(framework_line);
    let mut class = KtClass::enum_(class_name)
        .vis(KtVis::Public)
        .kdoc(enum_kdoc)
        .ctor_param(
            KtCtorParam::new("value", KtType::int())
                .val()
                .vis(KtVis::Public),
        );
    for e in entries {
        class = class.entry(e);
    }
    // `@JvmStatic` exposes `fromInt` as a real static method on the enum
    // class itself (rather than only on the `Companion` nested class). The
    // generated struct-encoder calls it via `env.call_static_method`,
    // which wouldn't find a companion-only method.
    class.companion(
        KtCompanion::new().vis(KtVis::Public).member(
            KtFun::new("fromInt")
                .vis(KtVis::Public)
                .annotation("JvmStatic")
                .param(KtParam::new("value", KtType::int()))
                .returns(KtType::cls(class_name))
                .expr_body(KtCode::new().line("entries.first { it.value == value }")),
        ),
    )
}

/// Build the Kotlin `data class` declaration for a `data_class`-declared
/// Rust struct. The class is **self-contained**: property/factory-param types
/// are full-FQN `KtType`s (the render-time `ImportSet` shortens + imports
/// them), and the `fromParts` factory's raw-text class references carry their
/// imports on the factory body `Code`.
pub(crate) fn build_data_class(
    ext: &Declarations,
    class_name: &str,
    item_struct: &prebindgen_registry::flat::Struct,
    registry: &Registry,
) -> KtClass {
    // A tuple struct is an `Extern` in the model, never a `Struct`, so every
    // field here is named by construction.
    let fields_named = &item_struct.fields;

    // The class declaration is derived from the SAME plan the `fromParts`
    // factory and the Rust encoder walk. Deriving it separately — a third
    // classification with its own rules — is what let a property's type
    // disagree with its own factory parameter (#156).
    let plan = ext
        .struct_plan(registry, item_struct, 0)
        .unwrap_or_else(|| {
            panic!(
                "data class `{}`: could not classify every field for the fromParts bridge. Each \
             field needs a resolved OUTPUT converter (that direction declares the slot the \
             encoder fills) AND the Kotlin metadata that converter carries — a `kotlin_name`, \
             or a registered class for a projection leaf",
                item_struct.name
            )
        });

    let mut ctor_params: Vec<KtCtorParam> = Vec::new();
    // Property (name, type) pairs, for the content-equality members an
    // array-backed property needs — see [`equality::content_equality_members`].
    let mut equality_props: Vec<(String, KtType)> = Vec::new();
    // Track per-field destructible (name, folded close strategy) so the
    // bottom emitter can produce a matching `close()` body for each.
    let mut destructible_fields: Vec<(String, crate::jni::FoldStrategy)> = Vec::new();
    for (field, pf) in fields_named.iter().zip(&plan.fields) {
        let field_ident = field.name.as_ref().unwrap_or_else(|| {
            panic!(
                "render_data_class_source: struct `{}` has an unnamed field in named-fields context",
                item_struct.name
            )
        });
        let kotlin_field_name = kotlin_property_name(field_ident);
        let owner = format!("{}.{}", item_struct.name, field_ident);

        // The declaration reads ONE direction — output — because that is the
        // direction that declares the `fromParts` slots the encoder fills, and
        // it is the direction both plans already use exclusively
        // (`build_struct_plan` output-only, `flat_input.rs` input-only). A
        // property whose type came from whichever direction happened to
        // resolve, while its wire came from the other, is how the declaration
        // drifted from the plans.
        //
        // A projection visible only on the INPUT side is exactly that drift
        // made reachable: the old walk emitted a typed handle property (plus
        // `AutoCloseable`) while the plan classified the same field as an
        // ordinary leaf, so the property and its own factory parameter
        // disagreed. Reject it at the declaration instead.
        if !matches!(pf.kind, PlanFieldKind::Projection { .. }) {
            if let Some(proj) = ext
                .in_frag(&field.ty)
                .and_then(|e| e.metadata.projection.clone())
            {
                panic!(
                    "data class field `{owner}`: the INPUT converter projects this field as `{:?}` \
                     but the OUTPUT converter does not, so the property type and its own \
                     `fromParts` parameter would disagree. The output direction declares the \
                     bridge, so give the field an output converter with the same projection — or \
                     drop the input-side projection",
                    proj.kind
                );
            }
        }

        let property_type = pf.kind.property_type(&owner);
        equality_props.push((kotlin_field_name.clone(), property_type.clone()));
        ctor_params.push(KtCtorParam::new(&kotlin_field_name, property_type).val());
        if let Some(strategy) = pf.kind.destructible() {
            destructible_fields.push((kotlin_field_name, strategy));
        }
    }

    // `fromParts` companion factory — recursively flattened the same way as the
    // native `flatten_struct_encode`: nested data-class fields are inlined as
    // their leaf wires, so native builds the whole object graph with ONE
    // `call_static_method`. Its raw-text class references (`Child.fromParts`,
    // `Enum.fromInt`, projection wraps) use short names; the FQNs they need are
    // collected here and attached to the factory body `Code` below.
    let mut factory_imports: BTreeSet<String> = BTreeSet::new();
    let (factory_params, factory_reconstruct, factory_mints_handle) = flatten_struct_factory(
        ext,
        registry,
        item_struct,
        "",
        class_name,
        &mut factory_imports,
        0,
    )
    .unwrap_or_else(|| {
        panic!("render_data_class_source: could not build fromParts factory for `{class_name}`")
    });

    // Kotlin requires a `data class` to declare at least one constructor
    // property, so the model takes the first one at construction. A declared
    // `data_class` with no fields could never have rendered as valid Kotlin.
    let mut ctor_params = ctor_params.into_iter();
    let first = ctor_params.next().unwrap_or_else(|| {
        panic!(
            "data class `{class_name}`: Rust struct `{}` has no fields — a Kotlin `data class` \
             must declare at least one constructor property",
            item_struct.name
        )
    });
    let mut class = KtClass::data(class_name, first).vis(KtVis::Public);
    if let Some(doc) = item_struct.docs() {
        class = class.kdoc(doc);
    }
    for p in ctor_params {
        class = class.ctor_param(p);
    }
    // Array-backed properties compare by identity in Kotlin, so a class with
    // one gets explicit content-based operators (the Rust type derives `Eq`).
    for m in crate::jni::equality::content_equality_members(class_name, &equality_props)
        .into_iter()
        .flatten()
    {
        class = class.member(m);
    }
    // Supertype clause: a data class with a destructible native-handle field
    // implements `AutoCloseable`; otherwise no supertype.
    if !destructible_fields.is_empty() {
        class = class.implements(KtType::cls("AutoCloseable"));
        // `close()` walks every destructible field via its folded close
        // strategy. `JNINativeHandle.close()` is idempotent
        // (Cleaner.Cleanable.clean() invokes exactly once), so calling
        // this multiple times — or alongside the cleaner's own firing on
        // GC — is safe. NOTE: `data class` copy() shares the handle
        // reference between copies; if you intend to close independently,
        // don't copy this class.
        let mut body = KtCode::new();
        for (fname, strategy) in &destructible_fields {
            body = body.line(render_handle_close(strategy, fname));
        }
        class = class.member(KtFun::new("close").modifier("override").body(body));
    }
    // `fromParts` factory: native (`struct_output_body`) makes ONE
    // `call_static_method` passing the whole graph's flattened leaf wires;
    // this factory reassembles it (incl. nested `Child.fromParts(...)`) in
    // JVM bytecode. `public`, not `internal`: an `internal` fun is mangled
    // to `fromParts$<module>`, unresolvable by native (`NoSuchMethodError`).
    // The factory body's raw-text class references (short names) carry their
    // imports on this `Code`, so the whole `data class` is self-contained.
    let mut factory_body = KtCode::new().line(factory_reconstruct);
    for fqn in factory_imports {
        factory_body = factory_body.import(fqn);
    }
    // Guarded only when a leaf actually is a raw pointer: a `fromParts` over
    // plain scalars and byte arrays cannot forge anything, and marking it
    // would delete a safe factory from Java and make unrelated Kotlin
    // consumers opt into a raw-pointer contract it does not have.
    let factory = KtFun::new("fromParts");
    let mut factory = if factory_mints_handle {
        ext.mark_unsafe(factory)
    } else {
        factory
    }
    .vis(KtVis::Public)
    .annotation("JvmStatic")
    .returns(KtType::cls(class_name))
    .expr_body(factory_body);
    for (name, ty) in &factory_params {
        factory = factory.param(KtParam::new(name, ty.clone()));
    }
    class = class.companion(KtCompanion::new().vis(KtVis::Public).member(factory));
    class
}

/// Render one typed-handle Kotlin source file. Pure-shell form (with a
/// method hook that appends `ViaJNI` to methods of the handle class):
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
/// `<mangle_method(package, class, "freePtr")>`. Kotlin/JVM's JNI name mangler binds it
/// to the matching `Java_<pkg>_<class>_<mangled-freePtr>`
/// extern on the Rust side (the auto-generated destructor).
pub(crate) fn build_typed_handle(
    ext: &Declarations,
    registry: &Registry,
    class_name: &str,
    rust_doc_name: &str,
    key: &TypeKey,
    imports: &mut BTreeSet<String>,
) -> KtClass {
    // The typed handle is a pure shell — `ptr` slot + `close()`/`take()` +
    // the `freePtr` extern. All functions are emitted as flat free functions
    // in their namespace package; nothing is promoted onto the class.
    //
    // Every typed handle extends the shared `NativeHandle` base, which owns
    // the `@Volatile` pointer slot (`ptr`) and its monitor — that common
    // supertype is what lets `render_wrapper_fn` collect a `List<NativeHandle>`
    // and lock it in one pointer-sorted, deadlock-safe pass. The subclass keeps
    // its own type-specific `close()`/`take()`/`freePtr`.
    let class_fqn = ext
        .types
        .get(key)
        .and_then(|cfg| cfg.name_spec.as_ref())
        .map(|spec| ext.fqn_of(spec))
        .unwrap_or_else(|| class_name.to_string());
    let (class_package, final_class_name) = class_fqn
        .rsplit_once('.')
        .unwrap_or(("", class_fqn.as_str()));
    let free_extern = ext.mangle_method(class_package, final_class_name, "freePtr");
    let gc_managed = ext
        .types
        .get(key)
        .and_then(|cfg| cfg.opaque())
        .is_some_and(|o| o.gc_managed);
    let base_short = if gc_managed {
        "GcNativeHandle"
    } else {
        "NativeHandle"
    };
    let base_fqn = if ext.package.is_empty() {
        base_short.to_string()
    } else {
        format!("{}.{base_short}", ext.package)
    };
    let members = ext.class_members.get(key).map(Vec::as_slice).unwrap_or(&[]);
    if !members.is_empty() && !ext.package.is_empty() {
        imports.insert(format!("{}.{}", ext.package, ext.jni_native_class_name()));
    }

    // Companion object: the `@JvmStatic external fun freePtr(ptr: Long)` called
    // by `close()`, plus one **factory** member per `.constructor(f, name)`
    // (a free wrapper — no receiver — returning the class).
    let mut companion = KtCompanion::new()
        .vis(KtVis::Public)
        .member(
            // `@JvmSynthetic`: a `@JvmStatic external fun` is a public static
            // native method, so `Storage.freePtr(0xdeadbeefL)` from Java would
            // free an address of the caller's choosing.
            KtFun::new(free_extern.clone())
                .annotation("JvmStatic")
                .annotation(JVM_SYNTHETIC)
                .external()
                .param(KtParam::new("ptr", KtType::long())),
        )
        // The replacement for the raw-pointer constructor. `internal` +
        // `@JvmSynthetic` is reachable from generated Kotlin and from neither
        // another Kotlin module nor Java; the constructor itself is `private`,
        // because a constructor can be neither hidden by `@JvmSynthetic`
        // (Kotlin rejects the target) nor by `internal` (still public on the
        // JVM). Nothing on the Rust side constructs a handle.
        .member(
            internal_fun(HANDLE_FACTORY)
                .kdoc(
                    "Wrap a pointer a generated native call returned. Passing anything \
                     else — a literal, a stale pointer, one belonging to another \
                     handle — is undefined behaviour, which is why this is not part \
                     of the public API.",
                )
                .param(KtParam::new("initialPtr", KtType::long()))
                .returns(KtType::cls(class_name))
                .expr_body(KtCode::new().line(format!("{class_name}(initialPtr)"))),
        );
    for m in members.iter().filter(|m| m.kind == MemberKind::Constructor) {
        if let Some(item_fn) = registry.flat().function(&m.rust_ident) {
            if let Some(f) = render_wrapper_fn(
                ext,
                item_fn,
                registry,
                Some(ext.effective_method_name(key, m).as_str()),
                None,
            ) {
                for ov in render_param_overloads(ext, item_fn, registry, &f) {
                    companion = companion.member(ov);
                }
                companion = companion.member(f);
            }
        }
    }

    // KDoc: the Rust struct's `///` prose first, framework line after.
    let framework_line = format!("Typed handle for a native Zenoh `{rust_doc_name}`.");
    let class_kdoc = source_item_doc(registry, key)
        .map(|d| format!("{d}\n\n{framework_line}"))
        .unwrap_or(framework_line);
    // Consumer interfaces (`.implements`) and the generated `<Name>Api`
    // interface (`.interface()`) are attached by `apply_class_interface` in
    // `write_typed_handles` after the class body is built.
    let mut class = KtClass::class_(class_name)
        .vis(KtVis::Public)
        // The class is public; minting one from a raw `Long` is not. `private`
        // rather than `internal`, because an internal constructor is still a
        // public JVM constructor — `new Storage(0xdeadbeefL).close()` compiled
        // from Java. Every generated call site goes through the companion's
        // `fromRawPtr` instead (see `handle_from_raw`).
        .ctor_vis(KtVis::Private)
        .kdoc(class_kdoc)
        .ctor_param(KtCtorParam::new("initialPtr", KtType::long()));
    class = if gc_managed {
        // GC-managed lifecycle: the pointer lives in the inherited atomic
        // cell; every release path settles the once-only untagged→tagged
        // ticket via `releaseCell`, and the registered Cleaner action frees
        // the box only if no other path won first. `clean()` on the explicit
        // paths is eager deregistration (the action then no-ops on the
        // already-tagged cell).
        if !ext.package.is_empty() {
            imports.insert(format!("{}.releaseCell", ext.package));
            imports.insert(format!("{}.registerGcHandle", ext.package));
        }
        class
            .extends(KtType::cls(base_fqn), Some("initialPtr"))
            .member(
                KtProperty::val("__cleanable")
                    .vis(KtVis::Private)
                    .initializer(format!("registerGcHandle(this) {{ {free_extern}(it) }}")),
            )
            .member(
                KtFun::new("close")
                    .annotation("Synchronized")
                    .modifier("override")
                    .body(
                        KtCode::new()
                            .line("val p = releaseCell(cell)")
                            .line(format!("if (p != 0L) {free_extern}(p)"))
                            .line("__cleanable?.clean()"),
                    ),
            )
            .member(
                KtFun::new("take")
                    .vis(KtVis::Public)
                    .annotation("Synchronized")
                    .returns(KtType::cls(class_name))
                    .body(
                        KtCode::new()
                            .line("val p = releaseCell(cell)")
                            .line("__cleanable?.clean()")
                            .line(format!(
                                "return {}",
                                handle_from_raw(class_name, "if (p != 0L) p else cell.get()")
                            )),
                    ),
            )
    } else {
        class
            .extends(KtType::cls(base_fqn), Some("initialPtr"))
            .member(
                KtFun::new("close")
                    .annotation("Synchronized")
                    .modifier("override")
                    .body(
                        KtCode::new()
                            .line("val p = ptr")
                            .blk("if (p != 0L && (p and 1L) == 0L) {", |c| {
                                c.line("ptr = p or 1L").line(format!("{free_extern}(p)"))
                            }),
                    ),
            )
            // Transfer ownership of the native pointer into a fresh handle,
            // leaving this one empty. Lets a callback receiver retain a handle
            // that the framework would otherwise `close()` when the callback
            // returns.
            .member(
                KtFun::new("take")
                    .vis(KtVis::Public)
                    .annotation("Synchronized")
                    .returns(KtType::cls(class_name))
                    .body(
                        KtCode::new()
                            .line("val p = ptr")
                            .line("ptr = p or 1L")
                            .line(format!("return {}", handle_from_raw(class_name, "p"))),
                    ),
            )
    };
    let mut class = class.companion(companion);

    // Promoted instance methods: each `.method(f)` becomes an instance method
    // (receiver bound to `this`), delegating to the same centralized
    // `JNINative` extern as a free wrapper would.
    for m in members.iter().filter(|m| m.kind == MemberKind::Method) {
        if let Some(item_fn) = registry.flat().function(&m.rust_ident) {
            if let Some(f) = render_wrapper_fn(
                ext,
                item_fn,
                registry,
                Some(ext.effective_method_name(key, m).as_str()),
                Some(key),
            ) {
                for ov in render_param_overloads(ext, item_fn, registry, &f) {
                    class = class.member(ov);
                }
                class = class.member(f);
            }
        }
    }
    class
}

/// True for an `Iterable` fold delivery, including one wrapped in an
/// `Optional` layer (`Option<Vec<T>>` → a nullable delivery). Selects the fold
/// surface (`acc` + `fold`) over a scalar `Optional`/`Base` builder.
pub(crate) fn is_iterable_fold(shape: &prebindgen_registry::unfold::UnfoldShape) -> bool {
    shape.has_iterable_layer()
}

/// The JNINative `external fun <method>(…): <wire-return>` for one bound
/// function, as a `KtFun` (the AST renderer shortens types + collects imports
/// + wraps long signatures uniformly). Wire level (matches the Rust extern):
///   * opaque-handle (Borrow/Consume) → jlong → `Long`
///   * `enum_class`                  → jint  → `Int` (call passes `.value`)
///   * `Any` (impl-Into Dispatch)     → JObject → `Any`
///   * everything else                → the entry's high-level Kotlin type
///
/// Opaque returns become `Long`; every other return uses [`classify_return`]'s
/// `kt_return` (Unit is no return type). `None` if a param's converter isn't
/// resolved. Full-FQN types throughout — no derivation-time shortening.
pub(crate) fn render_extern_decl(
    ext: &Declarations,
    f: &prebindgen_registry::flat::Function,
    registry: &Registry,
) -> Option<KtFun> {
    // The name and wire params come straight off the lowered plan — the
    // same classification the Rust extern and the Kotlin call site consume,
    // so the three sites agree on arity, types, and symbol by construction.
    let fplan = ext.fn_plan(registry, f).ok()?;
    let jni_call = &fplan.jni_method;
    let mut params: Vec<KtParam> = Vec::new();
    for leaf in fplan.leaves() {
        let name = leaf.kt_name.clone();
        match &leaf.kind {
            // Flattenable data_class param → its leaf wire params.
            InputKind::FlattenStruct(plan) => {
                for l in &plan.leaves {
                    params.push(KtParam::new(
                        l.kt_name.clone(),
                        KtType::cls(l.kt_wire_ty().to_string()),
                    ));
                }
            }
            // Bare `Option<primitive>` / `Option<enum>` param → a `(present:
            // Boolean, value: <Prim>)` pair (no boxed `java.lang.*` wire).
            InputKind::OptionalPair(sp) => {
                params.push(KtParam::new(sp.present_kt.clone(), KtType::boolean()));
                params.push(KtParam::new(
                    sp.value_kt.clone(),
                    KtType::cls(sp.value_kt_type.clone()),
                ));
            }
            // Slice/Vec of a flattenable data_class → a single `jlong`
            // Vec-handle param (the Rust extern decodes the boxed `Vec<T>`).
            // Elements cross through the synthetic `…VecPush` extern.
            InputKind::VecBuild { .. } => {
                params.push(KtParam::new(name, KtType::long()));
            }
            // An opaque-**handle** projection (direct `&T`/`T`, `Option<&T>`,
            // or by-value `Option<T>`) crosses the JNI wire as a primitive
            // `jlong` with `0` encoding `None` — a non-null `Long`; the `?`
            // lives only on the typed-wrapper surface.
            InputKind::Handle { .. } => {
                params.push(KtParam::new(name, KtType::long()));
            }
            InputKind::Callback { .. } | InputKind::Unsigned64 { .. } | InputKind::Plain => {
                let ty = if leaf.as_enum_value {
                    // Enum (incl. `Option<enum>`) crosses as jint → Kotlin
                    // `Int`; the wrapper passes `.value` / `?.value`. The Rust
                    // converter unboxes a `java.lang.Integer`, so the extern
                    // declares `Int`/`Int?`, never the enum object.
                    KtType::int()
                } else {
                    leaf.kt_meta.clone()?
                };
                let niche_primitive =
                    matches!(&leaf.kind, InputKind::Unsigned64 { niche: Some(_) });
                let ty = if leaf.optional && !niche_primitive {
                    ty.nullable()
                } else {
                    ty
                };
                params.push(KtParam::new(name, ty));
            }
        }
    }
    // Output (data) expansion: a **callback** delivery appends the lambda(s)
    // before the error sink and returns the erased `Any?`. A **return**
    // delivery appends nothing and returns the real converted wire below.
    if let FnOutputPlan::Unfold(u) = &fplan.output {
        if u.iterable_fold {
            // `acc` is the unbounded accumulator `A` (may be nullable) → `Any?`;
            // `fold` is the non-null adapter callback.
            params.push(KtParam::new("acc", KtType::any().nullable()));
            params.push(KtParam::new("fold", KtType::any()));
        } else {
            params.push(KtParam::new("build", KtType::any()));
        }
    }
    // Trailing error-sink callbacks — the binding channel (`errorSink`) always,
    // then the typed domain channel (`domainSink`) for a fallible-typed fn. Both
    // erased to `Any` (JObject) on the wire; the wrapper passes a capture for
    // each. A domain plan ⇒ `error_plans` has this fn.
    params.push(KtParam::new("errorSink", KtType::any()));
    if fplan.error.is_some() {
        params.push(KtParam::new("domainSink", KtType::any()));
    }

    let wire_return: Option<KtType> = match &fplan.output {
        FnOutputPlan::Unfold(_) => Some(KtType::any().nullable()),
        FnOutputPlan::Value(v) => {
            // The plan classified the declared surface once — `convert_out_ty`
            // for a `convert_output` (Return), else the function's own return.
            let (kt_return, projection) = render_return_surface(&v.surface)?;
            // JNI extern's wire return: projections wire as `Long` folded
            // through the projection strategy; enums wire as `Int`
            // (`Int?` under `Option`); everything else is the declared return.
            match &projection {
                Some(p) => Some(projection_wire_return(p)),
                None if v.is_enum => Some(KtType::int()),
                None if v.is_option_enum => Some(KtType::int().nullable()),
                None => kt_return,
            }
        }
    };

    let mut fun = KtFun::new(jni_call).external();
    for p in params {
        fun = fun.param(p);
    }
    if let Some(rt) = wire_return {
        fun = fun.returns(rt);
    }
    Some(fun)
}

struct Param {
    kt_name: String,
    kt_type: KtType,
    mode: ParamMode,
    /// `true` when the param's Rust type is a `enum_class`-declared enum: the
    /// high-level Kotlin signature uses the typed enum (`Priority`), but the
    /// underlying JNI `external fun` declares the param as `Int` (jint wire).
    /// The wrapper bridges the two by passing `<name>.value` at the call site.
    as_enum_value: bool,
}

enum ParamMode {
    Borrow,  // &T opaque-handle → withPtr
    Consume, // T  opaque-handle → consume
    /// `Option<&T>` / `Option<&mut T>` opaque-handle → `withPtrOrZero`.
    /// Nullable typed-handle param; the wrapper runs the body under the read
    /// lock when the handle is non-null and with `0L` when null. The Rust
    /// converter materializes `Option<OwnedObject<T>>` and the call site uses
    /// `.as_deref()` / `.as_deref_mut()`.
    BorrowNullable,
    /// By-value `Option<T>` opaque-handle → nullable consume. Same as
    /// [`Self::Consume`] but the typed param is nullable (`T?`): `0L` when
    /// absent, and the handle's `ptr` slot is nulled after the call only when
    /// present. The Rust converter consumes the `Box` to `Option<T>`.
    ConsumeNullable,
    PassThrough,
    /// Kotlin `ULong` projected to its raw JNI `Long` bit pattern.
    Unsigned64 {
        niche: Option<String>,
    },
    /// Flattenable `data_class` param: the high-level Kotlin signature keeps the
    /// typed object, but the `JNINative` call destructures it into the leaf
    /// access expressions (no `JObject` crosses, so the Rust side skips
    /// `env.get_field(...)`). The strings are the per-leaf call-site
    /// expressions in plan order.
    FlattenStruct {
        accesses: Vec<String>,
        /// Owned handles nested anywhere below the flattened root. They join
        /// the wrapper's one sorted lock set and are marked consumed after
        /// the native call, exactly like top-level by-value handles.
        handles: Vec<Opaque>,
    },
    /// `&[T]` / `Vec<T>` of a flattenable data_class `T`: the public Kotlin
    /// signature keeps `List<T>`, but the wrapper allocates a Rust-side `Vec<T>`
    /// (opaque `jlong` handle), pushes each element's decoupled leaves in a loop
    /// via `<base>Push`, passes the handle to the extern, then frees it in a
    /// `finally`. No `JObject` list crosses, so the Rust side skips per-element
    /// `env.get_field(...)`. `base` is the helper method base (`payloadVec`);
    /// `elem_accesses` are the per-element leaf push expressions rooted at the
    /// loop variable (`__e.id`, `__e.seq`, …), in plan order.
    VecBuild {
        base: String,
        elem_accesses: Vec<String>,
    },
    /// `impl Fn(args)` callback param: typed Kotlin lambda over the flattened
    /// leaves of each arg's callback plan (whole arg when plan-less), erased to
    /// `Any` at the extern tier — the same shape as the unfold `build`/`onError`
    /// lambdas. `call_arg` is the call-site expression.
    Callback {
        call_arg: String,
    },
    /// Bare `Option<primitive>` / `Option<enum>` param decomposed into a
    /// `(present: Boolean, value: <prim>)` pair so no boxed `java.lang.*`
    /// crosses (and the Rust side does no `intValue()` unboxing). The public
    /// Kotlin signature keeps `T?`; the call site passes `present_expr`
    /// (`<name> != null`) then `value_expr` (`<name> ?: 0` / `<name>?.value ?:
    /// 0`). See [`crate::jni::compile::OptionalPairPlan`].
    OptionalPair {
        present_expr: String,
        value_expr: String,
    },
}

#[derive(Clone)]
struct Opaque {
    /// Kotlin param name (e.g. `"b"`).
    name: String,
    /// Object to synchronize on and read the pointer from (`<name>`).
    target: String,
    /// Statement that marks the pointer slot closed after consume by setting
    /// the tag bit (`"<target>.ptr = <target>.ptr or 1L"` — the address bits
    /// stay put so the lock-ordering key never changes), or `None` for
    /// borrow modes.
    consume_null: Option<String>,
    /// `true` for `Option<&T>` — nullable param, branches before lock.
    nullable: bool,
    /// The **resource domain**: the handle's Kotlin type name with nullability
    /// dropped, so `T` and `T?` land in one domain and two unrelated handle
    /// classes do not. Used only by [`render_alias_preflight`], to keep it from
    /// comparing a `Storage` against a `Summary` — pointers that can never be
    /// equal. `None` ⇒ unknown, so compare against everything (fail-safe).
    domain: Option<String>,
}

/// Peel `&` / `Option<…>` / `Option<&…>` layers and return the inner type's
/// [`TypeKey`] — used to match an accessor's receiver parameter against its
/// owning class key in [`render_wrapper_fn`].
///
/// Off the model. This walked a node four levels deep — a `Type::Reference`,
/// a `Type::Path`'s last segment compared against the *name* `"Option"`, its
/// `AngleBracketed` arguments, and a second `Type::Reference` — to reach a
/// question `borrow_target` / `optional_inner` / `key` answer directly.
pub(crate) fn peel_receiver_key(ty: &prebindgen_registry::flat::TypeRef) -> TypeKey {
    let core = ty.borrow_target().unwrap_or(ty);
    match core.optional_inner() {
        Some(inner) => inner.borrow_target().unwrap_or(inner).key(),
        None => core.key(),
    }
}

/// Build a single top-level (free-function) wrapper as a [`KtFun`].
/// Returns `None` if the function has a parameter whose Kotlin type isn't
/// registered (in that case we skip the function rather than panicking — the
/// legacy `JNINative.kt` retains the unwrapped external fun so callers still
/// have an escape hatch).
///
/// Every `#[prebindgen]` function is emitted as a flat namespaced free function
/// — opaque-handle parameters are ordinary `NativeHandle` params, locked via the
/// per-call `withSortedHandleLocks` scaffold.
///
/// When `receiver_key` is `Some(class_key)` the function is emitted as an
/// **instance method** of that class: the first parameter whose (peeled) Rust
/// type equals `class_key` is dropped from the signature and bound to `this`
/// (the inherited `NativeHandle` scope for a `ptr_class` — `this.ptr` + lock).
/// The JNINative extern/call is
/// unchanged (keyed on the Rust ident), so only the Kotlin wrapper relocates.
/// The Kotlin surface of a wrapper: the assembled `KtFun` with every
/// parameter/return type in place but **no body**, plus the emission
/// internals [`render_wrapper_fn`] needs to fill that body. One derivation of
/// the overload surface, shared by emission (which adds the body) and
/// [`validate_symbols`](crate::jni::validate_symbols)
/// (which erases `fun` to a JVM signature), so the emitted overload and the
/// validated one cannot drift (issue #89).
pub(crate) struct WrapperSurface {
    /// The wrapper with its full signature and an empty body. The validator
    /// reads this; [`render_wrapper_fn`] fills the body and adds the KDoc.
    pub fun: KtFun,
    // Emission-only internals — computed while assembling the signature,
    // consumed by `render_wrapper_fn`; opaque to the validator.
    params: Vec<Param>,
    out: OutputPlan,
    sink: ErrorSink,
    jni_call: String,
    /// FQNs the wrapper **body** references by short name (extension `asRaw`,
    /// hoisted singletons, the error-capture holder). Signature-type imports
    /// are NOT here — those are full-FQN `KtType`s in `fun`, collected by the
    /// render-time `ImportSet`. `render_wrapper_fn` attaches these to the
    /// body `Code`, so the emitted `KtFun` is self-contained; the validator
    /// ignores them.
    body_imports: BTreeSet<String>,
}

/// Whether an error handler may return null when it cannot manufacture the
/// value a failed call was meant to produce.
#[derive(Clone, Copy)]
enum RecoveryReturn {
    /// Public wrappers: reference-shaped returns become nullable. Types whose
    /// nullable form has a different JVM representation (primitives and
    /// ULong) keep their declared type.
    NullableReferences,
    /// Constant helpers always install a throwing handler themselves, so they
    /// retain the constant's declared type instead of leaking nullability into
    /// the public property.
    Declared,
}

/// The type returned by both error handlers and, consequently, by the wrapper
/// itself. A handler may decline to fabricate a reference result by returning
/// null; primitive-shaped results keep their existing unboxed contract.
fn recovery_return_type(out: &OutputPlan, policy: RecoveryReturn) -> KtType {
    let declared = out.kt_return.clone().unwrap_or_else(KtType::unit);
    if matches!(policy, RecoveryReturn::Declared) {
        return declared;
    }
    let generics: Vec<String> = out.generic.iter().cloned().collect();
    nullable_recovery_type(declared, &generics)
}

fn nullable_recovery_type(declared: KtType, generics: &[String]) -> KtType {
    if declared.is_nullable() || declared == KtType::unit() {
        return declared;
    }
    let nullable = declared.clone().nullable();
    if crate::jni::symbols::erase_kt_type(generics, &declared)
        == crate::jni::symbols::erase_kt_type(generics, &nullable)
    {
        nullable
    } else {
        declared
    }
}

/// Build the [`WrapperSurface`]: everything [`render_wrapper_fn`] does up to
/// (but not including) the body render — the single surface-signature
/// derivation. **Pure** over `(ext, f, registry, name, receiver)`: signature
/// types are full-FQN `KtType`s and any body-import FQNs are returned in
/// [`WrapperSurface::body_imports`], so nothing is registered into a caller's
/// import set. Validation calls this directly and skips the body work
/// (`build_native_call` / `render_body` / KDoc / opaque-lock collection).
pub(crate) fn build_wrapper_surface(
    ext: &Declarations,
    f: &prebindgen_registry::flat::Function,
    registry: &Registry,
    kotlin_name_override: Option<&str>,
    receiver_key: Option<&TypeKey>,
) -> Option<WrapperSurface> {
    build_wrapper_surface_with_recovery(
        ext,
        f,
        registry,
        kotlin_name_override,
        receiver_key,
        RecoveryReturn::NullableReferences,
    )
}

fn build_wrapper_surface_with_recovery(
    ext: &Declarations,
    f: &prebindgen_registry::flat::Function,
    registry: &Registry,
    kotlin_name_override: Option<&str>,
    receiver_key: Option<&TypeKey>,
    recovery: RecoveryReturn,
) -> Option<WrapperSurface> {
    let mut body_imports = BTreeSet::new();
    let fplan = ext.fn_plan(registry, f).ok()?;
    // The Kotlin extern in `JNINative` is keyed on the Rust ident (the
    // plan's `jni_method`). The per-entry `.name("...")` override only
    // changes the *user-facing* Kotlin wrapper name; the JNI call still has
    // to hit the one extern that the Rust extern actually emits.
    let kt_name = match kotlin_name_override {
        Some(n) => n.to_string(),
        None => kt_snake_to_camel(&f.name.to_string()),
    };
    let jni_call = fplan.jni_method.clone();
    let (params, receiver_idx) = classify_params(&fplan, &mut body_imports, receiver_key)?;
    let out = classify_output(ext, &fplan, &mut body_imports)?;
    let r_ty = recovery_return_type(&out, recovery);
    let sink = error_sink_parts(&fplan, &mut body_imports, &r_ty)?;

    let mut fun = KtFun::new(&kt_name).vis(KtVis::Public);
    if let Some(g) = &out.generic {
        fun = fun.generic(g);
    }
    for (i, p) in params.iter().enumerate() {
        // The receiver param is bound to `this` — not a rendered parameter.
        if Some(i) == receiver_idx {
            continue;
        }
        fun = fun.param(KtParam::new(&p.kt_name, p.kt_type.clone()));
    }
    // The error callbacks — **required**: the generated code never throws; the
    // consumer decides how a failure surfaces (e.g. by throwing its own type).
    // Every wrapper takes the binding handler; a fallible-typed fn additionally
    // takes the domain handler, ordered `onBindingError, onError` so the DOMAIN
    // `onError` is the natural trailing lambda. When an output-expansion
    // builder/fold lambda exists it must stay the **trailing** lambda, so the
    // error params go *before* it — but *after* any non-lambda `builder_lead`
    // (`acc: A`), which is passed positionally.
    let mut err_params = vec![KtParam::new(&sink.binding_param, sink.binding_type.clone())];
    if let Some(d) = &sink.domain {
        err_params.push(KtParam::new("onError", d.onerr_type.clone()));
    }
    if let Some((bp_name, bp_ty)) = &out.builder_param {
        if let Some((lead_name, lead_ty)) = &out.builder_lead {
            fun = fun.param(KtParam::new(lead_name, lead_ty.clone()));
        }
        for ep in err_params {
            fun = fun.param(ep);
        }
        fun = fun.param(KtParam::new(bp_name, bp_ty.clone()));
    } else {
        for ep in err_params {
            fun = fun.param(ep);
        }
    }
    if out.cast_return {
        fun = fun.annotation("Suppress(\"UNCHECKED_CAST\")");
    }
    if out.kt_return.is_some() {
        fun = fun.returns(r_ty);
    }
    Some(WrapperSurface {
        fun,
        params,
        out,
        sink,
        jni_call,
        body_imports,
    })
}

pub(crate) fn render_wrapper_fn(
    ext: &Declarations,
    f: &prebindgen_registry::flat::Function,
    registry: &Registry,
    kotlin_name_override: Option<&str>,
    receiver_key: Option<&TypeKey>,
) -> Option<KtFun> {
    render_wrapper_fn_with_recovery(
        ext,
        f,
        registry,
        kotlin_name_override,
        receiver_key,
        RecoveryReturn::NullableReferences,
    )
}

fn render_wrapper_fn_with_recovery(
    ext: &Declarations,
    f: &prebindgen_registry::flat::Function,
    registry: &Registry,
    kotlin_name_override: Option<&str>,
    receiver_key: Option<&TypeKey>,
    recovery: RecoveryReturn,
) -> Option<KtFun> {
    let surface = build_wrapper_surface_with_recovery(
        ext,
        f,
        registry,
        kotlin_name_override,
        receiver_key,
        recovery,
    )?;
    let WrapperSurface {
        mut fun,
        params,
        out,
        sink,
        jni_call,
        mut body_imports,
    } = surface;
    // KDoc: the Rust fn's `///` prose first, then generated notes for every
    // position an expansion reshaped away from the Rust signature (N1).
    // Emission-only — the validator skips it.
    let fplan = ext.fn_plan(registry, f).ok()?;
    if let Some(doc) = wrapper_kdoc(f, &fplan) {
        fun = fun.kdoc(doc);
    }
    // Collect the opaque-handle params so we can scaffold pointer-ordered
    // synchronized blocks around them.
    let opaques = collect_opaques(&params);
    let is_unit = fun.ret.is_none();
    let body_expr = build_native_call(ext, &jni_call, &params, &out, &sink);
    let return_mode = if is_unit {
        BodyReturn::Unit
    } else {
        BodyReturn::Value(build_success_return(ext, &out, "__ret"))
    };
    // `render_body` extends `body_imports` with the body's own references; all
    // of them are attached to the body `Code`, so the emitted `KtFun` carries
    // its own imports (signature imports ride the FQN types via the render-time
    // `ImportSet`). The wrapper is thus a self-contained decl — callers no
    // longer thread a per-file import set through it.
    let mut body = render_body(
        ext,
        &params,
        &opaques,
        &sink,
        &body_expr,
        &return_mode,
        &mut body_imports,
    );
    for fqn in body_imports {
        body = body.import(fqn);
    }
    Some(fun.body(body))
}

/// Render one declared const (see `ConstDecl`): a **private** nullary helper
/// — the standard wrapper fn over the synthetic getter signature
/// ([`const_getter_fn`]), reused verbatim so the const's type crosses through
/// the ordinary output machinery — plus the public lazily-initialized `val`
/// that calls it once, on first use (see [`render_val_over_helper`]).
pub(crate) fn render_const_val(
    ext: &Declarations,
    package: &str,
    c: &prebindgen_registry::flat::Constant,
    registry: &Registry,
    imports: &mut BTreeSet<String>,
    kotlin_name_override: Option<&str>,
) -> Option<(KtFun, KtProperty)> {
    let getter = const_getter_fn(c);
    let default = kt_snake_to_camel(&getter.name.to_string());
    let helper_name = ext.mangle_fun(package, &default);
    let helper = render_wrapper_fn_with_recovery(
        ext,
        &getter,
        registry,
        Some(&helper_name),
        None,
        RecoveryReturn::Declared,
    )?;
    let val_name = kotlin_name_override
        .map(str::to_string)
        .unwrap_or_else(|| c.name.to_string());
    let framework_line = format!(
        "Mirrors the Rust `#[prebindgen]` const `{}` (read lazily, once, through \
         the generated JNI getter on first use).",
        c.name
    );
    let kdoc = c
        .docs()
        .map(|d| format!("{d}\n\n{framework_line}"))
        .unwrap_or(framework_line);
    render_val_over_helper(ext, registry, helper, val_name, kdoc, imports)
}

/// Render one fn-sourced constant (see `ConstDecl::fun`):
/// the declared nullary fn's ordinary wrapper demoted to a **private**
/// helper, plus the public lazily-initialized `val` holding its result —
/// computed once, on first use, through the ordinary generated wrapper
/// (one JNI call, exactly like a const getter).
pub(crate) fn render_constant_fn_val(
    ext: &Declarations,
    package: &str,
    f: &prebindgen_registry::flat::Function,
    registry: &Registry,
    imports: &mut BTreeSet<String>,
    kotlin_name_override: Option<&str>,
) -> Option<(KtFun, KtProperty)> {
    let default = kt_snake_to_camel(&f.name.to_string());
    let helper_name = ext.mangle_fun(package, &default);
    let helper = render_wrapper_fn_with_recovery(
        ext,
        f,
        registry,
        Some(&helper_name),
        None,
        RecoveryReturn::Declared,
    )?;
    let val_name = kotlin_name_override
        .map(str::to_string)
        .unwrap_or_else(|| f.name.to_string());
    let framework_line = format!(
        "Mirrors the Rust `#[prebindgen]` fn `{}()` (evaluated lazily, once, \
         through the generated JNI wrapper on first use).",
        f.name
    );
    let kdoc = f
        .docs()
        .map(|d| format!("{d}\n\n{framework_line}"))
        .unwrap_or(framework_line);
    render_val_over_helper(ext, registry, helper, val_name, kdoc, imports)
}

/// Render one expression-backed constant (see `ConstDecl::expr`):
/// a private nullary helper over the synthetic `const_get_*` getter (seeded
/// from the val name), plus the public lazily-initialized `val` — the value
/// is the binding-defined expression, evaluated once, on first use, through
/// the generated getter.
pub(crate) fn render_const_expr_val(
    ext: &Declarations,
    package: &str,
    decl: &crate::jni::decl::ConstExprDecl,
    registry: &Registry,
    imports: &mut BTreeSet<String>,
) -> Option<(KtFun, KtProperty)> {
    let getter = const_expr_getter_fn(&decl.kotlin_name, &decl.ty, registry);
    let default = kt_snake_to_camel(&getter.name.to_string());
    let helper_name = ext.mangle_fun(package, &default);
    let helper = render_wrapper_fn_with_recovery(
        ext,
        &getter,
        registry,
        Some(&helper_name),
        None,
        RecoveryReturn::Declared,
    )?;
    let expr = decl.expr.to_token_stream();
    let kdoc = format!(
        "Binding-defined constant: `{expr}` (evaluated lazily, once, through \
         the generated JNI getter on first use)."
    );
    render_val_over_helper(
        ext,
        registry,
        helper,
        decl.kotlin_name.clone(),
        kdoc,
        imports,
    )
}

/// Shared val-rendering core for both constant kinds (`ConstDecl` /
/// `ConstDecl::fun`): demote the rendered wrapper to a private
/// helper and emit the public `val X: T by lazy { … }` that calls it once,
/// on first use, with a throwing `JniErrorHandler` (dead code for infallible
/// converts; a binding-layer failure surfaces as `IllegalStateException` via
/// `error(...)` at first use). Lazy, not eager: a consts-heavy package must
/// not fire one JNI call per `val` at class-load (issue #58).
fn render_val_over_helper(
    ext: &Declarations,
    registry: &Registry,
    mut helper: KtFun,
    val_name: String,
    kdoc: String,
    imports: &mut BTreeSet<String>,
) -> Option<(KtFun, KtProperty)> {
    helper.vis = KtVis::Private;
    let helper_name = helper.name.clone();
    // A constant always carries a value type; a helper with no return would
    // mean the type never resolved — skip like an unresolvable fn.
    let val_ty = helper.ret.clone()?;
    let spec = ext.iface_spec(registry, &SpecKey::JniErrorHandler)?;
    imports.insert(spec.fqn());
    let init = format!(
        "{helper_name}(JniErrorHandler {{ je -> error(je ?: \"const {val_name}: JNI getter failed\") }})"
    );
    let prop = KtProperty::val(&val_name)
        .ty(val_ty)
        .vis(KtVis::Public)
        .delegate(format!("lazy {{ {init} }}"))
        .kdoc(kdoc);
    Some((helper, prop))
}

/// The classified output side of a wrapper: return type, projection wrap,
/// output-expansion (builder/fold) params, and the extra call-site args —
/// everything the call-expression builder and the signature direction must
/// agree on.
struct OutputPlan {
    kt_return: Option<KtType>,
    /// Kotlin-newtype return (opaque handle / `ULong`) — the wrap the
    /// call expression folds around the extern result.
    projection: Option<Projection>,
    /// Trailing **lambda** param (`build` / `fold`) of an output expansion.
    builder_param: Option<(String, KtType)>,
    /// Non-lambda lead param (`acc: A`) — precedes `onError` positionally.
    builder_lead: Option<(String, KtType)>,
    /// Type variable (`R` / `A`) when the wrapper is generic.
    generic: Option<String>,
    /// Extra call-site args injected before `__cap` (builder/adapter, or
    /// `acc` + fold callback for `Iterable`).
    unfold_call_args: Vec<String>,
    /// Callback delivery: cast the extern's erased `Any?` to `R`/`A`.
    cast_return: bool,
    /// enum_class return crossing as jint — wrap with `fromInt`.
    is_enum_return: bool,
    /// `Option<enum>` return crossing boxed — `?.let { fromInt(it) }`.
    is_option_enum_return: bool,
}

/// The error-callback wiring for a wrapper. Every wrapper has a **binding**
/// channel (`JniErrorHandler`, any marshalling/closed-handle failure); a
/// fallible function with a declared error type additionally has a **domain**
/// channel (the typed `<Src>Handler`, the decomposed `Err(E)`). Each is a
/// separate handler param + per-thread capture, so neither carries a `je`
/// discriminator or fabricated defaults.
struct ErrorSink {
    /// Kotlin param name of the binding handler: `"onBindingError"` when a
    /// domain channel is also present, else `"onError"` (the sole channel).
    binding_param: String,
    /// The binding handler's Kotlin type — always `JniErrorHandler<R>`.
    binding_type: KtType,
    /// Short name of the base per-thread capture (`JniErrorHandlerCapture`).
    binding_capture_short: String,
    /// The single redispatch arg for `<binding_param>.run(...)` — the captured
    /// message slot (`__bcap.ze0`).
    binding_call_arg: String,
    /// The domain channel, present only for a fallible fn with a typed error.
    domain: Option<DomainSink>,
}

/// The typed domain-error channel: the `onError` handler, its raw capture, and
/// the wrapped leaf args for the post-call redispatch (no `je`).
struct DomainSink {
    onerr_type: KtType,
    /// Short name of the generated per-thread raw capture holder.
    capture_short: String,
    /// Wrapped ze-leaf args for the post-call `onError.run(...)` redispatch.
    call_args: String,
}

/// Type every effective input of the lowered [`JniFunctionPlan`] into a
/// [`Param`] (Kotlin name/type + call-site [`ParamMode`]). The crossing-form
/// classification comes from the plan — the same decision the Rust extern and
/// `external fun` renderers consume — this site only maps each [`InputKind`]
/// to its Kotlin surface. Returns the params plus the index of the
/// instance-method receiver (the first param whose peeled type matches
/// `receiver_key`), which is bound to `this` and dropped from the signature.
fn classify_params(
    fplan: &JniFunctionPlan,
    imports: &mut BTreeSet<String>,
    receiver_key: Option<&TypeKey>,
) -> Option<(Vec<Param>, Option<usize>)> {
    let mut receiver_idx: Option<usize> = None;
    let mut params: Vec<Param> = Vec::new();
    for leaf in fplan.leaves() {
        let mut name = leaf.kt_name.clone();

        // Instance-method receiver: the first parameter whose peeled Rust type
        // is the owning class binds to `this` (so `this_ptr`/`this.ptr`/lock or
        // `this.bytes` fall out of the normal param handling) and is dropped
        // from the rendered signature.
        if receiver_idx.is_none() {
            if let Some(rk) = receiver_key {
                if &peel_receiver_key(&leaf.reading) == rk {
                    receiver_idx = Some(params.len());
                    name = "this".to_string();
                }
            }
        }

        // `impl Fn(args)` param: a generated typed `fun interface`
        // (`<ArgShorts>Callback`) whose `run` parameters are the flattened
        // leaves of each arg's callback plan (the arg whole when plan-less).
        // The extern receives it erased (`Any`) and the native trampoline
        // calls the typed `run`, so no call-site adapter exists.
        // Lambda-literal call sites SAM-convert unchanged.
        if let InputKind::Callback { iface, .. } = &leaf.kind {
            let spec = iface.as_deref()?;
            let kt_type = spec.kt_ref(vec![]);
            // The extern receives the RAW twin: the generated `asRaw()`
            // proxy (built once per registration) wraps raw leaves into the
            // typed objects the user's interface declares.
            let call_arg = if spec.needs_raw() {
                imports.insert(format!("{}.asRaw", spec.package));
                format!("{name}.asRaw()")
            } else {
                name.clone()
            };
            params.push(Param {
                kt_name: name.clone(),
                kt_type,
                mode: ParamMode::Callback { call_arg },
                as_enum_value: false,
            });
            continue;
        }

        // Typed surface: the projection's Kotlin FQN for opaque handles /
        // value projections (any `Option<_>` layer is nullable purely on the
        // typed-wrapper surface — the handle wire stays `jlong` with `0` =
        // absent), else the resolved entry's Kotlin name. `None` (unresolved
        // name) skips the wrapper — the escape-hatch path.
        let kt_type_raw = leaf.kt_public.clone()?;

        // Map the plan's crossing form to the Kotlin call-site mode.
        let mode = match &leaf.kind {
            InputKind::VecBuild { helpers, .. } => {
                // Slice/Vec of a flattenable data_class: build the Rust-side
                // Vec by pushing each element's leaves, pass the handle (see
                // the body direction + `build_vec_build_helper_items`). The
                // high-level signature stays `List<T>` (registered below).
                let elem_accesses = helpers
                    .plan
                    .leaves
                    .iter()
                    .filter(|l| !l.is_present_flag())
                    .map(|l| l.kt_access("__e"))
                    .collect();
                ParamMode::VecBuild {
                    base: helpers.base.clone(),
                    elem_accesses,
                }
            }
            InputKind::OptionalPair(sp) => {
                // Bare `Option<primitive>` / `Option<enum>`: cross as a
                // `(present, value)` pair (no boxed object). The high-level
                // signature keeps `T?`; only the call-site args split in two.
                let present_expr = format!("{name} != null");
                let value_expr = if sp.is_enum {
                    format!("{name}?.value ?: {}", sp.value_kt_zero)
                } else {
                    format!("{name} ?: {}", sp.value_kt_zero)
                };
                ParamMode::OptionalPair {
                    present_expr,
                    value_expr,
                }
            }
            InputKind::FlattenStruct(plan) => {
                let handles = plan
                    .leaves
                    .iter()
                    .filter_map(|leaf| {
                        // Through the leaf's own access template rather than a
                        // reconstructed `<name>.<field>`, so the lock target is
                        // whatever expression the leaf actually reads from.
                        // No live case needs it yet — `build_flat_sum_field`
                        // returns `None` for a projection payload, so a
                        // tag-gated handle leaf cannot reach here — but the
                        // template is the leaf's own answer to "where did this
                        // come from", and deriving it twice is how the two
                        // drift apart.
                        let target = leaf.kt_handle_target(&name)?;
                        let consume_null = if leaf.handle_nullable() {
                            format!("{target}?.markConsumed()")
                        } else {
                            format!("{target}.markConsumed()")
                        };
                        Some(Opaque {
                            name: leaf.kt_name.clone(),
                            target,
                            consume_null: Some(consume_null),
                            nullable: leaf.handle_nullable(),
                            // A flattened leaf carries no Kotlin class name of
                            // its own, so its domain is unknown and it compares
                            // against every other handle — the fail-safe
                            // direction (an extra `ptr` comparison, never a
                            // missed alias).
                            domain: None,
                        })
                    })
                    .collect();
                ParamMode::FlattenStruct {
                    accesses: plan.leaves.iter().map(|l| l.kt_call_arg(&name)).collect(),
                    handles,
                }
            }
            InputKind::Handle { mode, .. } => match mode {
                HandleMode::Borrow => ParamMode::Borrow,
                HandleMode::Consume => ParamMode::Consume,
                HandleMode::BorrowNullable => ParamMode::BorrowNullable,
                HandleMode::ConsumeNullable => ParamMode::ConsumeNullable,
            },
            InputKind::Unsigned64 { niche } => ParamMode::Unsigned64 {
                niche: niche.clone(),
            },
            InputKind::Plain => ParamMode::PassThrough,
            InputKind::Callback { .. } => unreachable!("callback params handled above"),
        };

        // Full-FQN surface type — the render-time `ImportSet` shortens it and
        // collects the import from the AST (issue #89 follow-up: decouple the
        // signature from the derivation-time import side-effect). No manual
        // registration here.
        let kt_type = if leaf.optional {
            kt_type_raw.nullable()
        } else {
            kt_type_raw
        };
        params.push(Param {
            kt_name: name,
            kt_type,
            mode,
            as_enum_value: leaf.as_enum_value,
        });
    }
    Some((params, receiver_idx))
}

/// Classify the output side into an [`OutputPlan`].
///
/// Output (data) expansion: the return value is delivered to a caller
/// callback per shape:
///   * `Decompose`/`Optional` (M1–M3): decompose into leaves → `build:
///     (L0, …) -> R` once; `<R>`, returns `R` / `R?`.
///   * `Iterable` (M4 whole / M5 decomposed): per element, fold
///     `(acc, leaves…) -> acc`; `<A>`, returns `A`, threads the accumulator.
///
/// Each leaf is delivered with its final Kotlin type; a leaf whose typed form
/// can't be constructed Rust-side makes the wrapper install an **adapter** that
/// applies the Kotlin-side projection wrap before the user callback. Leaves
/// with no such projection ⇒ the callback is passed directly (M1–M4
/// unchanged).
fn classify_output(
    ext: &Declarations,
    fplan: &JniFunctionPlan,
    imports: &mut BTreeSet<String>,
) -> Option<OutputPlan> {
    let unfold = fplan.unfold.as_ref();
    // `builder_param` is the trailing **lambda** param (build / fold) as a
    // `(name, function-type)` pair. For the `Iterable` shape, the non-lambda
    // accumulator (`acc: A`) goes in `builder_lead` — it must precede
    // `onError` (a defaulted param) so the positional-`acc` call stays valid;
    // the trailing `fold` lambda follows.
    let mut builder_param: Option<(String, KtType)> = None;
    let mut builder_lead: Option<(String, KtType)> = None;
    let mut generic: Option<String> = None;
    // Extra call-site args injected before `__sink` (e.g. `build`/adapter, or
    // `acc` + the fold callback/adapter for `Iterable`).
    let mut unfold_call_args: Vec<String> = Vec::new();

    let (kt_return, projection) = if let FnOutputPlan::Value(v) = &fplan.output {
        // `convert_output` (Return) and plain returns: the wrapper returns
        // the value directly — the plan classified the declared surface once
        // (`convert_out_ty` for a convert, the signature's own output
        // otherwise). No callback param, no generic, no extra call args; the
        // extern returns the real wire and `build_call` applies the
        // projection wrap (handle) below.
        render_return_surface(&v.surface)?
    } else if let (
        FnOutputPlan::Unfold(
            u @ UnfoldOutputPlan {
                fixed_builder: true,
                ..
            },
        ),
        Some(plan),
    ) = (&fplan.output, unfold)
    {
        // Synthesized by-value `data_class` delivery via a **fixed, hoisted
        // singleton** — the wrapper takes no caller `build`/`fold` param and is
        // not generic over `R`/`A`. The native side still receives the singleton
        // as an erased `Any` and calls its cached `run`, so the whole delivery
        // machinery is reused; only the Rust-side object construction is gone.
        // The concrete return is the data class (`T` / `Option<T>`) or, for a
        // `Vec<data_class>` fold, a `List<Class>` composed on the Kotlin side.
        // The concrete element/return Kotlin type. For a decomposed `data_class`
        // builder/fold it is the data class (`plan.source`'s registered FQN); for
        // a **whole-element leaf** fold (`plan.element` set — String / handle)
        // `plan.source` (e.g. `String`) has no class FQN, so take the
        // element's typed view from the folder interface's element param instead.
        // Full-FQN class type: the render-time `ImportSet` shortens it (and
        // handles simple-name collisions) when it renders the return type. The
        // `ArrayList<…>` body arg below uses the short name — the class is
        // imported via that return type, so the short name resolves.
        let class_ty = if u.whole_element {
            let spec = u.iface.as_deref()?;
            spec.params[1].typed.clone()
        } else {
            let class_fqn = ext.kotlin_fqn(&plan.source.key()).map(|s| s.to_string())?;
            KtType::cls(class_fqn)
        };
        let class_short = kt_type_short(&class_ty);
        if u.iterable_fold {
            // `Vec<data_class>` fold: allocate an `ArrayList<Class>` accumulator,
            // pass the hoisted **folder-appender** singleton as `fold` (it
            // rebuilds each element via `fromParts` and appends it), and return
            // the threaded accumulator as `List<Class>` (`?`-nullable for an
            // `Option<Vec<…>>` return — `None` yields a null list). Per element
            // only the raw leaves cross — no Java object is built on the Rust side.
            let spec = u.iface.as_deref()?;
            let holder = spec.singleton_holder_name();
            let field = crate::jni::SINGLETON_FIELD;
            imports.insert(spec.singleton_holder_fqn());
            unfold_call_args.push(format!("ArrayList<{class_short}>()"));
            unfold_call_args.push(format!("{holder}.{field}"));
            let list_ty = KtType::generic("List", [class_ty]);
            let kt = if u.optional {
                list_ty.nullable()
            } else {
                list_ty
            };
            (Some(kt), None)
        } else {
            // Scalar: the hoisted `__<Name>Builder` singleton calls `fromParts`;
            // the wrapper returns the concrete class (`?`-nullable for `Option`).
            let spec = u.iface.as_deref()?;
            let singleton = format!("__{}", spec.raw_name());
            imports.insert(format!("{}.{singleton}", spec.package));
            unfold_call_args.push(singleton);
            let kt = if u.optional {
                class_ty.nullable()
            } else {
                class_ty
            };
            (Some(kt), None)
        }
    } else if let (FnOutputPlan::Unfold(u), Some(_)) = (&fplan.output, unfold) {
        // The builder / fold params are generated typed `fun interface`s
        // (`<Source>Builder<out R>` / `<Element>Folder<A>`); the native side
        // calls their typed `run` with raw jvalues (no call-site adapter).
        // Lambda-literal call sites SAM-convert unchanged.
        generic = u.generic.map(str::to_string);
        // An `Iterable` fold — bare or `Optional`-wrapped — folds with `<A>`
        // (`acc` lead + `fold` lambda). The wrapped form returns `A?`: `None`
        // skips the fold and delivers null (matching the scalar `R?` and the
        // fixed path's null `List`), `Some(empty)` returns `acc` unchanged.
        if u.generic == Some("A") {
            let spec = u.iface.as_deref()?;
            builder_lead = Some(("acc".to_string(), KtType::var_("A")));
            builder_param = Some(("fold".to_string(), spec.kt_ref(vec![KtType::var_("A")])));
            unfold_call_args.push("acc".to_string());
            if spec.needs_raw() {
                imports.insert(format!("{}.asRaw", spec.package));
                unfold_call_args.push("fold.asRaw()".to_string());
            } else {
                unfold_call_args.push("fold".to_string());
            }
            let kt = if u.optional {
                KtType::var_("A").nullable()
            } else {
                KtType::var_("A")
            };
            (Some(kt), None)
        } else {
            let spec = u.iface.as_deref()?;
            builder_param = Some(("build".to_string(), spec.kt_ref(vec![KtType::var_r()])));
            if spec.needs_raw() {
                imports.insert(format!("{}.asRaw", spec.package));
                unfold_call_args.push("build.asRaw()".to_string());
            } else {
                unfold_call_args.push("build".to_string());
            }
            let kt = if u.optional {
                KtType::var_r().nullable()
            } else {
                KtType::var_r()
            };
            (Some(kt), None)
        }
    } else {
        unreachable!("FnOutputPlan is either Value or Unfold-with-plan")
    };
    // enum_class returns cross the JNI wire as jint → Kotlin `Int` (`Int?`
    // boxed for `Option<enum>`) — so `build_call` can wrap the result with
    // `fromInt`. The plan's probes run over the convert-peeled declared type;
    // the wrapper surface keeps the historical `unfold.is_none()` mask
    // (`Value` ∧ ¬`is_convert` ⟺ no unfold plan).
    let (is_enum_return, is_option_enum_return) = match &fplan.output {
        FnOutputPlan::Value(v) if !v.is_convert => (v.is_enum, v.is_option_enum),
        _ => (false, false),
    };

    Some(OutputPlan {
        kt_return,
        projection,
        builder_param,
        builder_lead,
        generic,
        unfold_call_args,
        cast_return: matches!(&fplan.output, FnOutputPlan::Unfold(_)),
        is_enum_return,
        is_option_enum_return,
    })
}

/// Build the raw JNINative call expression. Every param maps to exactly one
/// call arg (or several, for a flattened data_class); the output plan's extra
/// args and the trailing `__cap` follow. Kotlin-side result projections are
/// deliberately deferred to [`build_success_return`], after the native error
/// captures have been checked.
fn build_native_call(
    ext: &Declarations,
    jni_call: &str,
    params: &[Param],
    out: &OutputPlan,
    sink: &ErrorSink,
) -> String {
    let mut args: Vec<String> = Vec::with_capacity(params.len());
    for p in params.iter() {
        // Flattened data_class param expands into multiple call args
        // (the leaf destructure expressions, in plan order).
        if let ParamMode::FlattenStruct { accesses, .. } = &p.mode {
            args.extend(accesses.iter().cloned());
            continue;
        }
        // VecBuild param: the extern receives the `jlong` Vec handle the
        // wrapper body allocated and filled (`__vec_<name>`), not the `List`.
        if let ParamMode::VecBuild { .. } = &p.mode {
            args.push(format!("__vec_{}", p.kt_name));
            continue;
        }
        // An Optional pair expands into two call args: the present flag
        // and the value-or-zero expression (in that order).
        if let ParamMode::OptionalPair {
            present_expr,
            value_expr,
        } = &p.mode
        {
            args.push(present_expr.clone());
            args.push(value_expr.clone());
            continue;
        }
        let arg = match &p.mode {
            ParamMode::Borrow
            | ParamMode::Consume
            | ParamMode::BorrowNullable
            | ParamMode::ConsumeNullable => format!("{}_ptr", p.kt_name),
            ParamMode::Unsigned64 { niche } => {
                if p.kt_type.is_nullable() {
                    match niche {
                        Some(niche) => format!("{}?.toLong() ?: {}", p.kt_name, niche),
                        None => format!("{}?.toLong()", p.kt_name),
                    }
                } else {
                    format!("{}.toLong()", p.kt_name)
                }
            }
            ParamMode::PassThrough => {
                if p.as_enum_value {
                    // Enum → its `Int` discriminant for the extern. Nullable
                    // enum (`Enum?`) uses `?.value` so it stays `Int?`.
                    if p.kt_type.is_nullable() {
                        format!("{}?.value", p.kt_name)
                    } else {
                        format!("{}.value", p.kt_name)
                    }
                } else {
                    p.kt_name.clone()
                }
            }
            // Callback lambda → the param itself (the extern takes the
            // erased `Any`).
            ParamMode::Callback { call_arg } => call_arg.clone(),
            ParamMode::FlattenStruct { .. } => {
                unreachable!("FlattenStruct expanded before the single-arg match")
            }
            ParamMode::VecBuild { .. } => {
                unreachable!("VecBuild expanded before the single-arg match")
            }
            ParamMode::OptionalPair { .. } => {
                unreachable!("OptionalPair expanded before the single-arg match")
            }
        };
        args.push(arg);
    }
    // Output expansion: the builder / (acc, fold) cross just before the
    // error callback.
    args.extend(out.unfold_call_args.iter().cloned());
    // Trailing error-sink captures: `__bcap` (binding) always, then `__dcap`
    // (typed domain error) for a fallible-typed fn — each records its channel's
    // args and sets a flag (no throw on the Rust upcall). The wrapper reads them
    // after the native call returns (see the body below).
    args.push("__bcap".to_string());
    if sink.domain.is_some() {
        args.push("__dcap".to_string());
    }
    format!(
        "{}.{jni_call}({})",
        ext.jni_native_class_name(),
        args.join(", ")
    )
}

/// Transform a successful raw native return into the public Kotlin return.
/// This expression is emitted only after binding/domain captures have been
/// checked, so a native failure placeholder can never reach an enum lookup,
/// value projection, or erased-result cast.
fn build_success_return(ext: &Declarations, out: &OutputPlan, raw: &str) -> String {
    if let Some(p) = &out.projection {
        // Fold the wrap through the projection strategy. The wrap class is
        // the projection leaf's typed short name (a Handle's typed-handle
        // class, or `ULong`). The sentinel is the Kotlin
        // null-representation literal for the leaf wire — used only by
        // the `Niche+primitive` arm of `fold_projection_wrap`.
        let leaf_fqn = ext
            .kotlin_fqn(&p.leaf_key)
            .unwrap_or_else(|| p.leaf_key.to_string());
        let short = leaf_fqn.rsplit('.').next().unwrap_or(&leaf_fqn).to_string();
        let sentinel = projection_leaf_sentinel(p);
        fold_projection_wrap(&p.strategy, raw, &p.kind, &short, sentinel.as_deref())
    } else if out.is_enum_return {
        let enum_kt = out
            .kt_return
            .as_ref()
            .expect("enum return has a Kotlin type");
        format!("{enum_kt}.fromInt({raw})")
    } else if out.is_option_enum_return {
        // `kt_return` renders nullable (`Priority?`); the companion lives
        // on the non-null class name.
        let enum_kt = out
            .kt_return
            .as_ref()
            .expect("Option<enum> return has a Kotlin type")
            .to_string();
        let enum_kt = enum_kt.trim_end_matches('?');
        format!("{raw}?.let {{ {enum_kt}.fromInt(it) }}")
    } else if out.cast_return {
        let cast_kt = out
            .kt_return
            .as_ref()
            .expect("callback delivery returns R/A");
        format!("{raw} as {}", kt_type_short(cast_kt))
    } else {
        raw.to_string()
    }
}

/// The opaque-handle params (Borrow/Consume modes) — the set the lock
/// scaffold, pre-lock guards, and consume `try/finally` operate on.
fn collect_opaques(params: &[Param]) -> Vec<Opaque> {
    params
        .iter()
        .flat_map(|p| {
            let (target, consume_null, nullable) = match &p.mode {
                ParamMode::Borrow => (p.kt_name.clone(), None, false),
                ParamMode::Consume => (
                    p.kt_name.clone(),
                    Some(format!("{n}.markConsumed()", n = p.kt_name)),
                    false,
                ),
                ParamMode::BorrowNullable => (p.kt_name.clone(), None, true),
                // Nullable consume: tag the slot only when present (null-safe).
                ParamMode::ConsumeNullable => (
                    p.kt_name.clone(),
                    Some(format!("{n}?.markConsumed()", n = p.kt_name)),
                    true,
                ),
                ParamMode::FlattenStruct { handles, .. } => return handles.clone(),
                _ => return Vec::new(),
            };
            vec![Opaque {
                name: p.kt_name.clone(),
                target,
                consume_null,
                nullable,
                domain: p.kt_type.simple_name().map(str::to_string),
            }]
        })
        .collect()
}

/// Error-callback wiring — the two independent channels ([`ErrorSink`]). The
/// **binding** handler `JniErrorHandler<R>.run(je: String?)` is always present
/// (its captured message slot is `__bcap.ze0`, since the capture is uniform).
/// A fallible fn with a typed error also gets the **domain** handler
/// `<Src>Handler<R>.run(ze…)` — no `je`, called only on `Err(E)`. Each is a
/// separate SAM param; the wrapper passes a per-thread capture to the extern,
/// then after the native call redispatches to whichever channel fired.
fn error_sink_parts(
    fplan: &JniFunctionPlan,
    imports: &mut BTreeSet<String>,
    r_ty: &KtType,
) -> Option<ErrorSink> {
    let ifaces = fplan.onerror_iface.as_ref()?;
    let binding_spec = &ifaces.binding;
    let binding_type = binding_spec.kt_ref(vec![r_ty.clone()]);
    imports.insert(binding_spec.capture_fqn());
    let binding_capture_short = binding_spec.capture_name();

    // The typed domain channel exists only for a fallible fn with an error
    // plan; when present, both the interface spec and the error plan are.
    let domain = if let Some(domain_spec) = &ifaces.domain {
        let error_plan = fplan
            .error
            .as_ref()
            .expect("domain handler ⇒ frozen error plan");
        // Per ze leaf: (raw capture Kotlin type, raw→typed wrap). The CAPTURE
        // is the raw twin (what the native side calls); the user's handler is
        // the TYPED interface — the redispatch wraps each raw slot.
        let ze_info: Vec<(KtType, crate::jni::WrapKind)> = domain_spec
            .params
            .iter()
            .map(|p| {
                if let Some(fqn) = p.wrap.class_fqn() {
                    imports.insert(fqn.to_string());
                }
                (p.raw.clone(), p.wrap.clone())
            })
            .collect();
        debug_assert_eq!(ze_info.len(), error_plan.leaves.len());
        imports.insert(domain_spec.capture_fqn());
        // The domain redispatch args — the decomposed leaves only (no `je`).
        // The native side always fills the raw ze on `Err`, so non-null slots
        // are asserted `!!` then wrapped raw → typed.
        let call_args = ze_info
            .iter()
            .enumerate()
            .map(|(i, (raw, wrap))| {
                if raw.is_nullable() {
                    wrap.wrap_expr(&format!("__dcap.ze{i}"), true)
                } else {
                    wrap.wrap_expr(&format!("__dcap.ze{i}!!"), false)
                }
            })
            .collect::<Vec<_>>()
            .join(", ");
        Some(DomainSink {
            onerr_type: domain_spec.kt_ref(vec![r_ty.clone()]),
            capture_short: domain_spec.capture_name(),
            call_args,
        })
    } else {
        None
    };

    // When there is a domain channel, the binding handler is named
    // `onBindingError` and the domain one is `onError`; otherwise the binding
    // handler is the sole `onError`.
    let binding_param = if domain.is_some() {
        "onBindingError".to_string()
    } else {
        "onError".to_string()
    };
    Some(ErrorSink {
        binding_param,
        binding_type,
        binding_capture_short,
        binding_call_arg: "__bcap.ze0".to_string(),
        domain,
    })
}

/// Pre-lock **alias preflight**: reject a call whose handle arguments name the
/// same native resource in a combination that would consume or invalidate it
/// twice — before the lock, and before any conversion.
///
/// `zCombine(primary: ZThing, fallback: ZThing)` is a supported declaration;
/// called as `zCombine(x, x)` it hands one allocation to two consuming
/// converters. `zAbsorb(a: ZThing, b: ZThing?)` mixing a consume with a borrow
/// is the same defect: the borrow dangles the moment the consume takes
/// ownership. Emitted whenever the call has **at least one consumed handle**
/// and **any other handle**.
///
/// Comparison is on `ptr`, the resource's own address, not on the declared
/// Kotlin type — which is what makes `T` and `T?` (and a handle nested inside a
/// flattened `data_class`) compare in the same domain without enumerating the
/// spellings. Two distinct live handles cannot share an address, so this has no
/// false positives; `0L` names no resource and is skipped.
///
/// A binding failure, so it routes through the same channel a closed handle
/// does — a function-level return, never a throw.
fn render_alias_preflight(opaques: &[Opaque], binding_param: &str, is_unit: bool) -> KtCode {
    let mut guards = KtCode::new();
    if opaques.len() < 2 || !opaques.iter().any(|o| o.consume_null.is_some()) {
        return guards;
    }
    let ptr_of = |o: &Opaque| {
        if o.nullable {
            format!("({t}?.ptr ?: 0L)", t = o.target)
        } else {
            format!("{t}.ptr", t = o.target)
        }
    };
    for i in 0..opaques.len() {
        for j in (i + 1)..opaques.len() {
            let (a, b) = (&opaques[i], &opaques[j]);
            // Two borrows of one resource are legal; at least one side must be
            // taking it away.
            if a.consume_null.is_none() && b.consume_null.is_none() {
                continue;
            }
            // Two handles of different classes are different allocations, so
            // their pointers can never be equal — checking them would be dead
            // code in every wrapper that mixes handle types. An unknown domain
            // compares against everything.
            if let (Some(da), Some(db)) = (&a.domain, &b.domain) {
                if da != db {
                    continue;
                }
            }
            let (pa, pb) = (ptr_of(a), ptr_of(b));
            let msg = format!(
                "\"Aliasing arguments: '{}' and '{}' are the same native resource; a consumed \
                 handle may not be passed twice in one call.\"",
                a.name, b.name,
            );
            let cond = format!("{pa} != 0L && {pa} == {pb}");
            guards = if is_unit {
                guards.wline(format!(
                    "if ({cond}) {{ {binding_param}.run({msg}); return }}"
                ))
            } else {
                guards.wline(format!("if ({cond}) return {binding_param}.run({msg})"))
            };
        }
    }
    guards
}

/// Pre-lock closed-handle guards: a racy-but-safe `isClosed()` check before
/// the lock. A closed handle is a **binding** failure, so it routes to
/// `<binding_param>.run("Operation on a closed native handle.")` (function-level
/// return; no throw). Racy: a close between this check and the native call is
/// caught by the Rust-side converter guard (the tag bit survives the race),
/// which routes through the same binding channel.
fn render_prelock_guards(opaques: &[Opaque], binding_param: &str, is_unit: bool) -> KtCode {
    const CLOSED_MSG: &str = "\"Operation on a closed native handle.\"";
    let mut guards = KtCode::new();
    for o in opaques {
        let cond = if o.nullable {
            format!("{t}?.isClosed() == true", t = o.target)
        } else {
            format!("{t}.isClosed()", t = o.target)
        };
        guards = if is_unit {
            guards.wline(format!(
                "if ({cond}) {{ {binding_param}.run({CLOSED_MSG}); return }}"
            ))
        } else {
            guards.wline(format!(
                "if ({cond}) return {binding_param}.run({CLOSED_MSG})"
            ))
        };
    }
    guards
}

/// The call in statement position, `bind`-prefixed (`""` / `"val __ret = "`),
/// wrapped in a consume `try/finally` when any handle is consumed.
fn render_value_stmt(bind: &str, body_expr: &str, opaques: &[Opaque]) -> KtCode {
    let consume_stmts: Vec<&str> = opaques
        .iter()
        .filter_map(|o| o.consume_null.as_deref())
        .collect();
    if consume_stmts.is_empty() {
        KtCode::new().wline(format!("{bind}{body_expr}"))
    } else {
        let mut fin = KtCode::new();
        for s in consume_stmts {
            fin = fin.line(s);
        }
        KtCode::new().try_finally(bind, KtCode::new().wline(body_expr), fin)
    }
}

/// The core call statement: `bind` + a single Kotlin **expression**
/// evaluating to the call's result. Handle params contribute
/// pointer-binding statements and a deadlock-safe `withSortedHandleLocks`
/// acquisition; the whole thing is expression-shaped (via `run { … }` where
/// statements are needed) so the caller can bind it to `__ret`, rethrow a
/// captured sink error, then return.
fn render_core_stmt(
    ext: &Declarations,
    opaques: &[Opaque],
    body_expr: &str,
    imports: &mut BTreeSet<String>,
    bind: &str,
) -> KtCode {
    // Under-lock pointer reads. The closed-handle check is done pre-lock
    // (`prelock_guards`, → `onError`); these just bind the ptr the call
    // passes. A handle closed after the guard carries the tag bit (odd
    // value), which the Rust-side converter guard rejects — never
    // dereferenced.
    let mut ptr_binds = KtCode::new();
    for o in opaques {
        ptr_binds = if o.nullable {
            ptr_binds.line(format!(
                "val {n}_ptr = {t}?.ptr ?: 0L",
                n = o.name,
                t = o.target
            ))
        } else {
            ptr_binds.line(format!("val {n}_ptr = {t}.ptr", n = o.name, t = o.target))
        };
    }

    if opaques.is_empty() {
        // No handles — the call expression stands alone.
        render_value_stmt(bind, body_expr, opaques)
    } else if !ext.emit_handle_locks {
        // Lock-free mode: ptr binds then the value, wrapped as an expression.
        KtCode::new().blk(format!("{bind}run {{"), |c| {
            c.push(ptr_binds)
                .push(render_value_stmt("", body_expr, opaques))
        })
    } else {
        // Fast path: a statically-known, small (1–3), all-non-null handle set.
        // Pass the handles positionally to the allocation-free fixed-arity
        // `withSortedHandleLocks` overload. Otherwise build a `List` and use
        // the recursive overload.
        //
        // Deliberate trade-off (#68): ANY nullable handle takes the `List`
        // path, even when the present/absent split could reach a fixed-arity
        // overload (`if (h != null) withSortedHandleLocks(a, h) {…} else …`).
        // The small-list allocation is benchmark-noise, and the branch would
        // duplicate the whole call body per arm — longer generated code for
        // no measured gain. All three overloads stay: each is exercised by
        // all-non-null wrappers (the common case).
        let fixed_arity = !opaques.iter().any(|o| o.nullable) && (1..=3).contains(&opaques.len());
        if !ext.package.is_empty() {
            imports.insert(format!("{}.withSortedHandleLocks", ext.package));
            if !fixed_arity {
                imports.insert(format!("{}.NativeHandle", ext.package));
            }
        }
        if fixed_arity {
            let targets = opaques
                .iter()
                .map(|o| o.target.as_str())
                .collect::<Vec<_>>()
                .join(", ");
            KtCode::new().blk(format!("{bind}withSortedHandleLocks({targets}) {{"), |c| {
                c.push(ptr_binds)
                    .push(render_value_stmt("", body_expr, opaques))
            })
        } else {
            let mut adds = KtCode::new();
            for o in opaques {
                adds = if o.nullable {
                    adds.line(format!("{t}?.let {{ __locks.add(it) }}", t = o.target))
                } else {
                    adds.line(format!("__locks.add({t})", t = o.target))
                };
            }
            KtCode::new().blk(format!("{bind}run {{"), |c| {
                c.line("val __locks = ArrayList<NativeHandle>()")
                    .push(adds)
                    .blk("withSortedHandleLocks(__locks) {", |l| {
                        l.push(ptr_binds)
                            .push(render_value_stmt("", body_expr, opaques))
                    })
            })
        }
    }
}

/// Assemble the wrapper's body text: pre-lock guards, the per-thread error
/// capture, the (possibly Vec-building) core call, the post-call failure
/// redispatch, and the return.
///
/// No throw from the binding: the wrapper installs a **capture** the extern
/// invokes on `Err` (a SAM literal of the same handler interface — the
/// native side calls its typed `run`), then — after the native call —
/// calls the user's `onError.run` and returns its `R` if a failure was
/// recorded. A pre-lock closed-handle guard short-circuits straight to
/// `onError.run` (it can't reach the capture).
/// Slice/Vec params built as Rust-side `Vec` handles: allocate the handle
/// before the lock, fill it by pushing each element's leaves, and free it in
/// a `finally` (always — the target wrapper either borrows the boxed `Vec` or
/// `mem::take`s it, leaving an empty `Vec` to drop). The transient handle is
/// not a `NativeHandle`, so it never joins the lock set.
enum BodyReturn {
    Unit,
    Value(String),
}

fn render_body(
    ext: &Declarations,
    params: &[Param],
    opaques: &[Opaque],
    sink: &ErrorSink,
    body_expr: &str,
    return_mode: &BodyReturn,
    imports: &mut BTreeSet<String>,
) -> KtCode {
    let is_unit = matches!(return_mode, BodyReturn::Unit);
    let vec_build: Vec<(&String, &String, &Vec<String>)> = params
        .iter()
        .filter_map(|p| match &p.mode {
            ParamMode::VecBuild {
                base,
                elem_accesses,
            } => Some((&p.kt_name, base, elem_accesses)),
            _ => None,
        })
        .collect();
    // The capture is a per-thread reusable holder (zero allocation): the
    // extern writes its `@JvmField` slots via `run`, the wrapper reads
    // them after the (synchronous) call. `acquire()` resets the slots.
    let mut b = render_alias_preflight(opaques, &sink.binding_param, is_unit)
        .push(render_prelock_guards(opaques, &sink.binding_param, is_unit))
        .line(format!(
            "val __bcap = {}.acquire()",
            sink.binding_capture_short
        ));
    if let Some(d) = &sink.domain {
        b = b.line(format!("val __dcap = {}.acquire()", d.capture_short));
    }
    // Post-call redispatch: at most one channel fires (the extern signals
    // binding OR domain, then returns), so check binding first, then domain.
    let mut failed_checks: Vec<String> = vec![format!(
        "if (__bcap.failed) return {}.run({})",
        sink.binding_param, sink.binding_call_arg
    )];
    if let Some(d) = &sink.domain {
        failed_checks.push(format!(
            "if (__dcap.failed) return onError.run({})",
            d.call_args
        ));
    }
    let bind = if is_unit { "" } else { "val __ret = " };
    if vec_build.is_empty() {
        b = b.push(render_core_stmt(ext, opaques, body_expr, imports, bind));
        for chk in &failed_checks {
            b = b.wline(chk);
        }
    } else {
        let native = ext.jni_native_class_name();
        for (name, base, _) in &vec_build {
            let new_m = crate::jni::vec_helper_method_name(ext, base, "New");
            b = b.wline(format!("val __vec_{name} = {native}.{new_m}({name}.size)"));
        }
        // `try { fill…; <core> } finally { free… }`: Kotlin `try` is an
        // expression, so for a non-unit fn `__ret` binds to the core call
        // (the block's last expression). A push runs no JVM upcall, so the
        // loop needs no per-element failure check.
        let mut fill = KtCode::new();
        for (name, base, accesses) in &vec_build {
            let push_m = crate::jni::vec_helper_method_name(ext, base, "Push");
            let args = std::iter::once(format!("__vec_{name}"))
                .chain(accesses.iter().cloned())
                .collect::<Vec<_>>()
                .join(", ");
            fill = fill.blk(format!("for (__e in {name}) {{"), |c| {
                c.wline(format!("{native}.{push_m}({args})"))
            });
        }
        let mut free = KtCode::new();
        for (name, base, _) in &vec_build {
            let free_m = crate::jni::vec_helper_method_name(ext, base, "Free");
            free = free.wline(format!("{native}.{free_m}(__vec_{name})"));
        }
        let core = render_core_stmt(ext, opaques, body_expr, imports, "");
        b = b.try_finally(bind, fill.push(core), free);
        for chk in &failed_checks {
            b = b.wline(chk);
        }
    }
    b = match return_mode {
        BodyReturn::Unit => b,
        BodyReturn::Value(expr) => b.line(format!("return {expr}")),
    };
    b
}

/// The Kotlin typing of one delivered lambda leaf: `(builder_kt, wire_kt,
/// wrap, is_value_projection)` — the type the *user's* lambda sees, the type the
/// extern delivers, and the expression rebuilding the former from the latter
/// (`pk` is the adapter's parameter name; passthrough unless the leaf carries a
/// value projection that can't be built Rust-side).
/// Shared by the unfold builder/fold lambda and the callback lambda params.
pub(crate) fn unfold_leaf_kt(
    ext: &Declarations,
    out_ty: &prebindgen_registry::flat::TypeRef,
    nullable: bool,
    pk: &str,
) -> Option<(KtType, String, String, bool)> {
    let proj = ext
        .out_frag(out_ty)
        .and_then(|e| e.metadata.projection.clone());
    let is_value_projection = proj
        .as_ref()
        .map(|p| matches!(p.kind, crate::jni::ProjectionKind::Unsigned64))
        .unwrap_or(false);
    // builder_kt: enum → Int; otherwise the normal classified type
    // (handle class / String / ByteArray / Long …).
    let builder_kt = if ext.is_kotlin_enum_reading(out_ty) {
        KtType::int()
    } else {
        classify_return(ext, out_ty)?.0?
    };
    let (mut wire_kt, wrap) = if is_value_projection {
        let p = proj.as_ref().unwrap();
        // Wrap class = the projection leaf's typed short name — NOT
        // `builder_kt` (which is `Short?` for an `Option<…>` leaf and would
        // leak the `?` into the constructor call).
        let leaf_fqn = ext
            .kotlin_fqn(&p.leaf_key)
            .unwrap_or_else(|| p.leaf_key.to_string());
        let short = leaf_fqn.rsplit('.').next().unwrap_or(&leaf_fqn).to_string();
        // The sentinel is the leaf's OWN `None`, so an absence it inherits from
        // an ancestor grants it none — `wrap_sentinel` is that rule, shared with
        // `leaf_iface_param`, which derives the same wrap (#142).
        let sentinel = wrap_sentinel(p, nullable);
        let mut wrap = fold_projection_wrap(&p.strategy, pk, &p.kind, &short, sentinel.as_deref());
        // A `nullable` leaf (an `Option` nesting step on its path) makes the
        // wire nullable even when the strategy itself is `Direct` — guard the
        // wrap so a null wire stays null instead of feeding the constructor.
        if nullable && matches!(p.strategy, crate::jni::FoldStrategy::Base) {
            let inner = projection_wrap_expr(&p.kind, &short, "it");
            wrap = format!("{pk}?.let {{ {inner} }}");
        }
        // Raw-text wire for the callback/builder descriptor — `KtType`'s
        // Display matches the historical string (`Long`/`ByteArray`/`List<…>`).
        (projection_wire_return(p).to_string(), wrap)
    } else {
        (builder_kt.to_string(), pk.to_string())
    };
    let builder_kt = if nullable {
        wire_kt.push('?');
        builder_kt.nullable()
    } else {
        builder_kt
    };
    Some((builder_kt, wire_kt, wrap, is_value_projection))
}

/// Kotlin parameter names for a plan's delivered leaves, in leaf order. The
/// names are the author-supplied [`UnfoldLeaf::name`]s (`handle` for a root
/// identity), emitted **verbatim** — no casing/keyword escaping (the author
/// writes valid Kotlin identifiers) and no dedup (uniqueness is enforced in
/// `core::unfold`).
///
/// [`UnfoldLeaf::name`]: prebindgen_registry::unfold::UnfoldLeaf::name
pub(crate) fn plan_leaf_names(leaves: &[crate::jni::compile::OutWire]) -> Vec<String> {
    leaves.iter().map(|leaf| leaf.name.clone()).collect()
}

/// Lambda parameter name for a whole-value (plan-less) callback arg: the
/// decapitalized bare type short (`ZQuery` → `zQuery`), peeling a `&` /
/// `Option<…>` layer; `arg{i}` for non-path shapes.
pub(crate) fn whole_value_name(ty: &prebindgen_registry::flat::TypeRef, i: usize) -> String {
    use prebindgen_registry::flat::TypeKind;
    // One borrow, then one `Option` — a fixed depth, not the general peel, and
    // read off `kind()` rather than through `optional_inner` so a `Box` stops
    // it here as it stopped the `syn::Type::Path` match before.
    let t = ty.borrow_target().unwrap_or(ty);
    let t = match t.kind() {
        TypeKind::Optional(inner) => inner,
        _ => t,
    };
    match crate::util::head_name(t) {
        Some(s) => {
            let mut cs = s.chars();
            let f = cs.next().expect("a name is not empty");
            kt_param_name(&format!("{}{}", f.to_lowercase(), cs.as_str()))
        }
        None => format!("arg{i}"),
    }
}

/// Fall-back Kotlin type derived directly from the JNI wire type.
/// Returns the **non-nullable** Kotlin base name — the use site adds
/// a `?` suffix when the entry's Rust type is `Option<…>` (via
/// the model), so this helper must not double up.
pub(crate) fn kotlin_for_wire(wire: &syn::Type) -> Option<KtType> {
    if let Some(p) = JniPrim::from_wire(wire) {
        return Some(KtType::cls(p.kotlin_type()));
    }
    if let syn::Type::Path(tp) = wire {
        if let Some(last) = tp.path.segments.last() {
            let kt = match last.ident.to_string().as_str() {
                "JString" | "jstring" => "String",
                "JByteArray" | "jbyteArray" => "ByteArray",
                "JObject" | "jobject" | "JClass" => "Any",
                _ => return None,
            };
            return Some(KtType::cls(kt));
        }
    }
    None
}

/// Returns `(kt_return, projection)` where:
/// * `kt_return` is the declared Kotlin return type written in the
///   wrapper's signature (empty for `Unit`).
/// * `projection` is `Some(Projection)` when the return is a Kotlin newtype
///   (opaque handle or unsigned scalar) reached through 0+ wrappers. The
///   wrapper body uses it to fold the wrap call (`W(x)` for `Direct`,
///   `?.let { W(it) }` for `Nullable`, `.map { W(it) }` for `Iterable`)
///   and pick the JNI extern's wire return (`Long` for `Handle`). `None` for
///   plain non-projection returns.
pub(crate) fn classify_return(
    ext: &Declarations,
    output: &prebindgen_registry::flat::TypeRef,
) -> Option<(Option<KtType>, Option<crate::jni::Projection>)> {
    let (surface, _canonical) = ReturnSurface::classify(ext, output);
    render_return_surface(&surface)
}

/// Map a classified [`ReturnSurface`] to the `(kt_return, projection)` pair,
/// with **full-FQN** Kotlin types — the render-time `ImportSet` shortens and
/// collects imports from the AST uniformly (issue #89 follow-up: one import
/// mechanism, no derivation-layer shortening). Panics on an unregistered
/// projection FQN — the same Kotlin-render-time failure `classify_return`
/// always had.
pub(crate) fn render_return_surface(
    surface: &ReturnSurface,
) -> Option<(Option<KtType>, Option<crate::jni::Projection>)> {
    match surface {
        ReturnSurface::Skip => None,
        ReturnSurface::Unit => Some((None, None)),
        ReturnSurface::Projected {
            projection,
            leaf_fqn,
        } => {
            let fqn = leaf_fqn.clone().unwrap_or_else(|| {
                panic!(
                    "classify_return: projection return type `{}` has no Kotlin FQN \
                     registered — every opaque class must be declared via `ptr_class!(...)`.",
                    projection.leaf_key
                )
            });
            Some((
                Some(handle_kt_type(&projection.strategy, &KtType::cls(fqn))),
                Some(projection.clone()),
            ))
        }
        ReturnSurface::Plain { kt } => Some((Some(kt.clone()), None)),
    }
}

/// Render a `KtType` to a string with every named type shortened to its
/// simple name — for **raw body text** that references a type already imported
/// elsewhere in the wrapper (the return type). A throwaway `ImportSet`
/// discards the imports (they come from the AST); the shortening matches what
/// the file renderer produces for the same type.
pub(crate) fn kt_type_short(ty: &KtType) -> String {
    ty.render(&mut kt::ImportSet::new(""))
}

/// The Kotlin property name of one struct field — the single derivation, so the
/// site that DECLARES a property and the sites that ACCESS it cannot disagree.
///
/// They did. `render_data_class_source` declared it through `kt_snake_to_camel`,
/// while `flat_input`'s access expression and JVM-slot name went through
/// `crate::util::snake_to_camel`, which additionally lower-cases the first character.
/// The two agree for a conventional lower-snake field and only for that: a field
/// spelled `Xyz` was declared `Xyz` and read as `xyz`, and a JNI `GetFieldID`
/// for a name that is not the declared one fails at runtime.
///
/// `kt_snake_to_camel` is the behaviour kept, because the declaration is what a
/// Kotlin property actually gets called; `snake_to_camel` stays where it names
/// PARAMETERS, which is a different namespace with no declaration to match.
pub(crate) fn kotlin_property_name(field: &syn::Ident) -> String {
    mangle_kotlin_ident(&kt_snake_to_camel(&field.to_string()))
}

pub(crate) fn kt_snake_to_camel(s: &str) -> String {
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

/// Camel-case a Rust param ident into a valid Kotlin parameter name. Param
/// names don't affect JNI linkage (only the function name + JVM signature do),
/// so sanitizing is always safe — this defers to the shared
/// [`mangle_kotlin_ident`], the single identifier sanitizer (issue #89).
pub(crate) fn kt_param_name(rust_ident: &str) -> String {
    mangle_kotlin_ident(&kt_snake_to_camel(rust_ident))
}

/// A wrapper's KDoc (N1): the Rust fn's `///` prose, then generated notes
/// documenting the REAL prototype after all expansions — one note per
/// position a plan reshaped, phrased for the caller. `None` for an
/// undocumented, unshaped fn.
fn wrapper_kdoc(
    f: &prebindgen_registry::flat::Function,
    fplan: &JniFunctionPlan,
) -> Option<String> {
    let prose = f.docs();
    let notes = shape_notes(fplan);
    match (prose, notes) {
        (Some(p), Some(n)) => Some(format!("{p}\n\n{n}")),
        (Some(p), None) => Some(p),
        (None, Some(n)) => Some(n),
        (None, None) => None,
    }
}

/// Caller-facing notes for every boundary position an expansion reshaped:
/// expanded params (what to pass instead of the Rust argument), decomposed
/// returns (what the builder/fold receives), and error decompositions
/// (what `onError` receives). Reads the same frozen function plan the report
/// uses.
fn shape_notes(fplan: &JniFunctionPlan) -> Option<String> {
    let mut notes: Vec<String> = Vec::new();

    let mut plans: Vec<(&syn::Ident, &prebindgen_registry::expand::FoldPlan)> = fplan
        .params
        .iter()
        .filter_map(|param| match &param.form {
            ParamForm::Expanded { plan, .. } => Some((&param.ident, plan.as_ref())),
            ParamForm::Single(_) => None,
        })
        .collect();
    plans.sort_by_key(|(p, _)| p.to_string());
    for (param, plan) in plans {
        let target = plan.target.to_string();
        let arms: Vec<String> = plan
            .variants
            .iter()
            .map(|v| match &v.ctor {
                Some(c) => format!("its `{c}` inputs"),
                None => format!("an existing `{target}`"),
            })
            .collect();
        let leaf_names: Vec<String> = plan
            .leaves
            .iter()
            .map(|l| snake_to_camel(&l.name.to_string()))
            .collect();
        let how = if plan.selector.is_some() {
            if plan.produces_option() {
                format!(
                    "pass EITHER {} — the selector chooses the arm, `-1` = absent",
                    arms.join(" OR ")
                )
            } else {
                format!(
                    "pass EITHER {} — the selector chooses the arm",
                    arms.join(" OR ")
                )
            }
        } else {
            arms.join(" / ").to_string()
        };
        notes.push(format!(
            "Parameter `{param}` is the Rust `{target}` argument, expanded: {how} \
             (crosses as `{}`).",
            leaf_names.join("`, `")
        ));
    }

    if let Some(plan) = &fplan.unfold {
        let source = plan.source.to_string();
        let leaves: Vec<&str> = plan.leaves.iter().map(|l| l.name.as_str()).collect();
        match plan.delivery {
            prebindgen_registry::unfold::Delivery::Callback if !leaves.is_empty() => {
                notes.push(format!(
                    "The Rust `{source}` result is delivered decomposed: the builder \
                     callback receives (`{}`).",
                    leaves.join("`, `")
                ));
            }
            prebindgen_registry::unfold::Delivery::Return => {
                notes.push(format!(
                    "The Rust `{source}` result is converted and returned as a single value."
                ));
            }
            _ => {}
        }
    }

    if let Some(plan) = &fplan.error {
        let source = plan.source.to_string();
        let leaves: Vec<&str> = plan.leaves.iter().map(|l| l.name.as_str()).collect();
        notes.push(format!(
            "On a domain error `onError` receives the decomposed Rust `{source}` error \
             (`{}`); a binding/system failure goes to `onBindingError` instead.",
            leaves.join("`, `")
        ));
    }

    if notes.is_empty() {
        None
    } else {
        Some(notes.join("\n"))
    }
}

/// The `///` doc of the `#[prebindgen]` struct/enum behind a declared type
/// key, when the item is indexed (a re-exported foreign type has none).
pub(crate) fn source_item_doc(registry: &Registry, key: &TypeKey) -> Option<String> {
    // Whichever of the three shapes the name declares — the docs are the
    // element's answer, so there is no `attrs` slice to unify across them.
    match registry.flat().declared_type(&key.ident()?)? {
        prebindgen_registry::flat::Type::Struct(s) => s.docs(),
        prebindgen_registry::flat::Type::Enum(e) => e.docs(),
        prebindgen_registry::flat::Type::Variant(v) => v.docs(),
        prebindgen_registry::flat::Type::Extern(_) => None,
    }
}

#[cfg(test)]
mod recovery_return_tests {
    use kotlin_codegen::KtType;

    use super::nullable_recovery_type;

    fn recover(ty: KtType, generics: &[&str]) -> KtType {
        nullable_recovery_type(
            ty,
            &generics.iter().map(|g| g.to_string()).collect::<Vec<_>>(),
        )
    }

    #[test]
    fn only_reference_shaped_recovery_returns_become_nullable() {
        for primitive in [
            KtType::unit(),
            KtType::int(),
            KtType::long(),
            KtType::boolean(),
            KtType::cls("ULong"),
        ] {
            assert!(!recover(primitive, &[]).is_nullable());
        }
        for reference in [
            KtType::string(),
            KtType::byte_array(),
            KtType::generic("List", [KtType::string()]),
            KtType::cls("io.test.Handle"),
            KtType::var_r(),
        ] {
            assert!(recover(reference, &["R"]).is_nullable());
        }
        assert!(recover(KtType::string().nullable(), &[]).is_nullable());
    }
}
