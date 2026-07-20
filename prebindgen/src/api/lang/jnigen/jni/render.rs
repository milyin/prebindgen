//! Kotlin source-file renderers (enum / data-class / typed-handle /
//! package / native / wrapper-fn) for the JNI back-end.
//!
//! Carved from the former `jni_kotlin_ext.rs`; shares the `jni` namespace
//! via `use super::*`.

use super::*;

// ── Safe-wrapper emitters ──────────────────────────────────────────────

/// One generated Kotlin `enum class` source — variants in
/// SCREAMING_SNAKE_CASE, each carrying the Rust discriminant as a
/// `val value: Int`, plus a `fromInt(value: Int)` companion. Mirrors
/// the hand-written `io.zenoh.qos.Priority` shape so adapter code that
/// already speaks the `.value` / `.fromInt(...)` idiom keeps working.
pub(crate) fn build_enum_class(class_name: &str, item_enum: &syn::ItemEnum) -> kt::KtClass {
    // Same discriminant source of truth the Rust `jint → variant` decode
    // uses, so Kotlin `value(N)` and the generated decode agree.
    let entries: Vec<kt::KtEnumEntry> =
        crate::api::lang::jnigen::util::enum_discriminant_values(item_enum)
            .into_iter()
            .map(|(ident, value)| kt::KtEnumEntry {
                name: mangle_kotlin_ident(
                    &crate::api::lang::jnigen::util::camel_to_screaming_snake(&ident.to_string()),
                ),
                args: Some(value.to_string()),
            })
            .collect();

    let framework_line = format!(
        "JVM-side surface for the native Rust `{}` enum.",
        item_enum.ident
    );
    let enum_kdoc = crate::api::lang::jnigen::util::doc_string(&item_enum.attrs)
        .map(|d| format!("{d}\n\n{framework_line}"))
        .unwrap_or(framework_line);
    kt::KtClass::new(kt::ClassKind::Enum(entries), class_name)
        .vis(kt::Vis::Public)
        .kdoc(enum_kdoc)
        .ctor_param(
            kt::KtCtorParam::new("value", kt::KtType::int())
                .val()
                .vis(kt::Vis::Public),
        )
        // `@JvmStatic` exposes `fromInt` as a real static method on the enum
        // class itself (rather than only on the `Companion` nested class). The
        // generated struct-encoder calls it via `env.call_static_method`,
        // which wouldn't find a companion-only method.
        .companion(
            kt::KtClass::companion_object().vis(kt::Vis::Public).member(
                kt::KtFun::new("fromInt")
                    .vis(kt::Vis::Public)
                    .annotation("JvmStatic")
                    .param(kt::KtParam::new("value", kt::KtType::int()))
                    .returns(kt::KtType::cls(class_name))
                    .expr_body(kt::Code::new().line("entries.first { it.value == value }")),
            ),
        )
}

/// Build the Kotlin `data class` declaration for a `data_class`-declared
/// Rust struct. Returns the class plus the FQN imports its (pre-shortened)
/// field/factory type strings reference.
pub(crate) fn build_data_class(
    ext: &JniGen,
    class_name: &str,
    item_struct: &syn::ItemStruct,
    registry: &Registry<KotlinMeta>,
) -> (kt::KtClass, BTreeSet<String>) {
    let fields_named = match &item_struct.fields {
        syn::Fields::Named(n) => &n.named,
        _ => {
            panic!(
                "render_data_class_source: struct `{}` must use named fields to map onto Kotlin data class properties",
                item_struct.ident
            )
        }
    };

    let mut imports: BTreeSet<String> = BTreeSet::new();
    let mut ctor_params: Vec<kt::KtCtorParam> = Vec::new();
    // Track per-field destructible (name, folded close strategy) so the
    // bottom emitter can produce a matching `close()` body for each.
    let mut destructible_fields: Vec<(String, crate::api::lang::jnigen::jni::FoldStrategy)> =
        Vec::new();
    for field in fields_named {
        let field_ident = field.ident.as_ref().unwrap_or_else(|| {
            panic!(
                "render_data_class_source: struct `{}` has an unnamed field in named-fields context",
                item_struct.ident
            )
        });
        let kotlin_field_name = mangle_kotlin_ident(&kt_snake_to_camel(&field_ident.to_string()));

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
                .kotlin_fqn(&h.leaf_key)
                .map(|v| v.to_string())
                .unwrap_or_else(|| {
                    panic!(
                        "render_data_class_source: projection field `{}.{}` leaf `{}` has no \
                         Kotlin FQN registered (ptr_class / value_class)",
                        item_struct.ident, field_ident, h.leaf_key
                    )
                });
            let short = register_fqn(&fqn, &mut imports);
            ctor_params.push(
                kt::KtCtorParam::new(
                    &kotlin_field_name,
                    handle_kt_type(&h.strategy, &kt::KtType::cls(short)),
                )
                .val(),
            );
            if matches!(
                h.kind,
                crate::api::lang::jnigen::jni::ProjectionKind::Handle
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
        let ty = register_kt_type(&kotlin_ty, &mut imports);
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
            .map(crate::api::lang::jnigen::jni::is_jni_primitive)
            .unwrap_or(false);
        let ty = if is_option_type(&field.ty) && !primitive_wire {
            ty.nullable()
        } else {
            ty
        };
        ctor_params.push(kt::KtCtorParam::new(&kotlin_field_name, ty).val());
    }

    // `fromParts` companion factory — recursively flattened the same way as the
    // native `flatten_struct_encode`: nested data-class fields are inlined as
    // their leaf wires, so native builds the whole object graph with ONE
    // `call_static_method`. Any nested child FQN it references is registered
    // into `imports`.
    let (factory_params, factory_reconstruct) =
        flatten_struct_factory(ext, registry, item_struct, "", class_name, &mut imports, 0)
            .unwrap_or_else(|| {
                panic!(
                    "render_data_class_source: could not build fromParts factory for `{class_name}`"
                )
            });

    let mut class = kt::KtClass::new(kt::ClassKind::Data, class_name).vis(kt::Vis::Public);
    if let Some(doc) = crate::api::lang::jnigen::util::doc_string(&item_struct.attrs) {
        class = class.kdoc(doc);
    }
    for p in ctor_params {
        class = class.ctor_param(p);
    }
    // Supertype clause: a data class with a destructible native-handle field
    // implements `AutoCloseable`; otherwise no supertype.
    if !destructible_fields.is_empty() {
        class = class.supertype(kt::KtType::cls("AutoCloseable"), None);
        // `close()` walks every destructible field via its folded close
        // strategy. `JNINativeHandle.close()` is idempotent
        // (Cleaner.Cleanable.clean() invokes exactly once), so calling
        // this multiple times — or alongside the cleaner's own firing on
        // GC — is safe. NOTE: `data class` copy() shares the handle
        // reference between copies; if you intend to close independently,
        // don't copy this class.
        let mut body = kt::Code::new();
        for (fname, strategy) in &destructible_fields {
            body = body.line(render_handle_close(strategy, fname));
        }
        class = class.member(kt::KtFun::new("close").modifier("override").body(body));
    }
    // `fromParts` factory: native (`struct_output_body`) makes ONE
    // `call_static_method` passing the whole graph's flattened leaf wires;
    // this factory reassembles it (incl. nested `Child.fromParts(...)`) in
    // JVM bytecode. `public`, not `internal`: an `internal` fun is mangled
    // to `fromParts$<module>`, unresolvable by native (`NoSuchMethodError`).
    let mut factory = kt::KtFun::new("fromParts")
        .vis(kt::Vis::Public)
        .annotation("JvmStatic")
        .returns(kt::KtType::cls(class_name))
        .expr_body(kt::Code::new().line(factory_reconstruct));
    for (name, ty) in &factory_params {
        factory = factory.param(kt::KtParam::new(name, ty.clone()));
    }
    class = class.companion(
        kt::KtClass::companion_object()
            .vis(kt::Vis::Public)
            .member(factory),
    );
    (class, imports)
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
    ext: &JniGen,
    registry: &Registry<KotlinMeta>,
    class_name: &str,
    rust_doc_name: &str,
    key: &TypeKey,
    imports: &mut BTreeSet<String>,
) -> kt::KtClass {
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
        .and_then(|cfg| cfg.opaque.as_ref())
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
    let mut companion = kt::KtClass::companion_object().vis(kt::Vis::Public).member(
        kt::KtFun::new(free_extern.clone())
            .annotation("JvmStatic")
            .modifier("external")
            .param(kt::KtParam::new("ptr", kt::KtType::long())),
    );
    for m in members.iter().filter(|m| m.kind == MemberKind::Constructor) {
        if let Some((item_fn, _)) = registry.functions.get(&m.rust_ident) {
            if let Some(f) = render_wrapper_fn(
                ext,
                item_fn,
                registry,
                imports,
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
    let mut class = kt::KtClass::new(kt::ClassKind::Plain, class_name)
        .vis(kt::Vis::Public)
        .kdoc(class_kdoc)
        .ctor_param(kt::KtCtorParam::new("initialPtr", kt::KtType::long()));
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
            .supertype(kt::KtType::cls(base_fqn), Some("initialPtr"))
            .member(
                kt::KtProperty::val("__cleanable")
                    .vis(kt::Vis::Private)
                    .initializer(format!("registerGcHandle(this) {{ {free_extern}(it) }}")),
            )
            .member(
                kt::KtFun::new("close")
                    .annotation("Synchronized")
                    .modifier("override")
                    .body(
                        kt::Code::new()
                            .line("val p = releaseCell(cell)")
                            .line(format!("if (p != 0L) {free_extern}(p)"))
                            .line("__cleanable?.clean()"),
                    ),
            )
            .member(
                kt::KtFun::new("take")
                    .vis(kt::Vis::Public)
                    .annotation("Synchronized")
                    .returns(kt::KtType::cls(class_name))
                    .body(
                        kt::Code::new()
                            .line("val p = releaseCell(cell)")
                            .line("__cleanable?.clean()")
                            .line(format!(
                                "return {class_name}(if (p != 0L) p else cell.get())"
                            )),
                    ),
            )
    } else {
        class
            .supertype(kt::KtType::cls(base_fqn), Some("initialPtr"))
            .member(
                kt::KtFun::new("close")
                    .annotation("Synchronized")
                    .modifier("override")
                    .body(
                        kt::Code::new()
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
                kt::KtFun::new("take")
                    .vis(kt::Vis::Public)
                    .annotation("Synchronized")
                    .returns(kt::KtType::cls(class_name))
                    .body(
                        kt::Code::new()
                            .line("val p = ptr")
                            .line("ptr = p or 1L")
                            .line(format!("return {class_name}(p)")),
                    ),
            )
    };
    let mut class = class.companion(companion);

    // Promoted instance methods: each `.method(f)` becomes an instance method
    // (receiver bound to `this`), delegating to the same centralized
    // `JNINative` extern as a free wrapper would.
    for m in members.iter().filter(|m| m.kind == MemberKind::Method) {
        if let Some((item_fn, _)) = registry.functions.get(&m.rust_ident) {
            if let Some(f) = render_wrapper_fn(
                ext,
                item_fn,
                registry,
                imports,
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
pub(crate) fn is_iterable_fold(shape: &crate::api::core::unfold::UnfoldShape) -> bool {
    shape.has_iterable_layer()
}

/// Render one `external fun <mangle_method(package, JNINative, name)>(…): <wire-return>` line
/// at the JNI **wire** level (matches what the Rust extern receives):
///   * opaque-handle (Borrow/Consume) → jlong → `Long`
///   * `enum_class`                  → jint  → `Int` (call passes `.value`)
///   * `Any` (impl-Into Dispatch)     → JObject → `Any`
///   * everything else                → entry's high-level Kotlin name
///
/// Opaque returns become `Long`; every other return uses
/// [`classify_return`]'s `kt_return` (Unit is empty string).
/// Returns `None` if any parameter's input converter isn't resolved.
pub(crate) fn render_extern_decl(
    ext: &JniGen,
    f: &syn::ItemFn,
    registry: &Registry<KotlinMeta>,
    imports: &mut BTreeSet<String>,
) -> Option<kt::Code> {
    // The name and wire params come straight off the lowered plan — the
    // same classification the Rust extern and the Kotlin call site consume,
    // so the three sites agree on arity, types, and symbol by construction.
    let fplan = JniFunctionPlan::build(ext, registry, f).ok()?;
    let jni_call = &fplan.jni_method;
    let mut params: Vec<(String, String)> = Vec::new();
    for leaf in fplan.leaves() {
        let name = leaf.kt_name.clone();
        match &leaf.kind {
            // Flattenable data_class param → its leaf wire params.
            InputKind::FlattenStruct(plan) => {
                for l in &plan.leaves {
                    let short = register_fqn(&l.kt_wire_ty, imports);
                    params.push((l.kt_name.clone(), short));
                }
            }
            // Bare `Option<primitive>` / `Option<enum>` param → a `(present:
            // Boolean, value: <Prim>)` pair (no boxed `java.lang.*` wire).
            InputKind::OptionScalar(sp) => {
                let pshort = register_fqn("Boolean", imports);
                params.push((sp.present_kt.clone(), pshort));
                let vshort = register_fqn(&sp.value_kt_type, imports);
                params.push((sp.value_kt.clone(), vshort));
            }
            // Slice/Vec of a flattenable data_class → a single `jlong`
            // Vec-handle param (the Rust extern decodes the boxed `Vec<T>`).
            // Elements cross through the synthetic `…VecPush` extern, not
            // this one.
            InputKind::VecBuild { .. } => {
                let short = register_kt_type(&kt::KtType::long(), imports).to_string();
                params.push((name, short));
            }
            // An opaque-**handle** projection (direct `&T`/`T`, `Option<&T>`,
            // or by-value `Option<T>`) crosses the JNI wire as a primitive
            // `jlong` with `0` encoding `None` — so the extern param is a
            // non-null `Long`, and the `?` lives only on the typed-wrapper
            // surface. (`value_blob` projections are NOT handles; they keep
            // their erased wire and can be nullable.)
            InputKind::Handle { .. } => {
                let short = register_kt_type(&kt::KtType::long(), imports).to_string();
                params.push((name, short));
            }
            InputKind::Callback { .. } | InputKind::ValueUnwrap { .. } | InputKind::Plain => {
                let kt_type_raw = if leaf.as_enum_value {
                    // Enum (incl. `Option<enum>`) crosses as jint → Kotlin
                    // `Int`; the wrapper passes `.value` / `?.value`. The Rust
                    // converter unboxes a `java.lang.Integer`, so the extern
                    // must declare `Int`/`Int?`, never the enum object.
                    kt::KtType::int()
                } else {
                    leaf.kt_meta.clone()?
                };
                // The extern block is raw text — render the shortened type.
                let short = register_kt_type(&kt_type_raw, imports).to_string();
                let suffix = if leaf.optional { "?" } else { "" };
                params.push((name, format!("{short}{suffix}")));
            }
        }
    }
    // Output (data) expansion: a **callback** delivery (`deconstruct_output`)
    // appends the lambda(s) before the error sink and returns the erased result
    // (`Any?`). A **return** delivery (`convert_output`) appends nothing and
    // returns the real converted wire (handled in `wire_return` below, keyed on
    // the plan's `Value` classification over `convert_out_ty`).
    if let FnOutputPlan::Unfold(u) = &fplan.output {
        if u.iterable_fold {
            // `acc` is the unbounded accumulator `A` (may be nullable) → `Any?`;
            // `fold` is the non-null adapter callback.
            params.push(("acc".to_string(), "Any?".to_string()));
            params.push(("fold".to_string(), "Any".to_string()));
        } else {
            params.push(("build".to_string(), "Any".to_string()));
        }
    }
    // Trailing error-sink callback — every extern accepts one (see
    // `signal_error` / the wrapper's default sink). `Any` at the wire level
    // (JObject); the wrapper passes an `ErrorSink` instance.
    params.push(("errorSink".to_string(), "Any".to_string()));

    let wire_return = match &fplan.output {
        FnOutputPlan::Unfold(_) => "Any?".to_string(),
        FnOutputPlan::Value(v) => {
            // The plan classified the declared surface once — `convert_out_ty`
            // for a `convert_output` (Return), else the function's own return.
            let (kt_return, projection) = render_return_surface(&v.surface, imports)?;
            // JNI extern's wire return: handle projections wire as `Long` (the
            // boxed jlong gets wrapped); value-class projections wire as their
            // inner converter's Kotlin type folded through the projection's
            // strategy (the value class is erased to that inner). Enums wire as
            // `Int` (`Int?` under `Option`); everything else is the declared
            // return.
            match &projection {
                Some(p) => projection_wire_return(p),
                None if v.is_enum => "Int".to_string(),
                None if v.is_option_enum => "Int?".to_string(),
                None => kt_return.map(|t| t.to_string()).unwrap_or_default(),
            }
        }
    };

    let formals: Vec<String> = params.iter().map(|(n, t)| format!("{n}: {t}")).collect();
    let ret_suffix = if wire_return.is_empty() {
        String::new()
    } else {
        format!(": {wire_return}")
    };
    let head = format!("external fun {jni_call}");
    let single = format!("{head}({}){ret_suffix}", formals.join(", "));
    // Externs render as members of `object JNINative` at one indent level
    // (4 columns). Past the shared signature-width budget, wrap to one
    // parameter per line — the same treatment the model renderer gives the
    // public wrappers — so long native declarations stay readable.
    const EXTERN_INDENT_COLS: usize = 4;
    if !formals.is_empty() && EXTERN_INDENT_COLS + single.len() > kt::render::MAX_SIGNATURE_WIDTH {
        let opener = format!("{head}(");
        let closer = format!("){ret_suffix}");
        Some(kt::Code::new().blk_with(opener, closer, move |mut c| {
            for f in &formals {
                c = c.line(format!("{f},"));
            }
            c
        }))
    } else {
        Some(kt::Code::new().line(single))
    }
}

struct Param {
    kt_name: String,
    kt_type: kt::KtType,
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
    /// Value-projection param (`value_blob`): a Kotlin `@JvmInline value class`
    /// that is **not** a lockable handle. The Kotlin param type is the
    /// value-class FQN; the call site passes the unwrapped inline-class field
    /// (`<name>.<field>`) so the `JNINative` extern receives the erased inner
    /// wire (e.g. `ByteArray`). No lock.
    ValueUnwrap {
        field: String,
    },
    /// Flattenable `data_class` param: the high-level Kotlin signature keeps the
    /// typed object, but the `JNINative` call destructures it into the leaf
    /// access expressions (no `JObject` crosses, so the Rust side skips
    /// `env.get_field(...)`). The strings are the per-leaf call-site
    /// expressions in plan order.
    FlattenStruct {
        accesses: Vec<String>,
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
    /// lambdas. `call_arg` is the call-site expression: the param itself, or a
    /// value-blob rebuilding adapter.
    Callback {
        call_arg: String,
    },
    /// Bare `Option<primitive>` / `Option<enum>` param decomposed into a
    /// `(present: Boolean, value: <prim>)` pair so no boxed `java.lang.*`
    /// crosses (and the Rust side does no `intValue()` unboxing). The public
    /// Kotlin signature keeps `T?`; the call site passes `present_expr`
    /// (`<name> != null`) then `value_expr` (`<name> ?: 0` / `<name>?.value ?:
    /// 0`). See [`crate::api::lang::jnigen::jni::OptionScalarInputPlan`].
    OptionScalar {
        present_expr: String,
        value_expr: String,
    },
}

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
}

/// Peel `&` / `Option<…>` / `Option<&…>` layers and return the inner type's
/// [`TypeKey`] — used to match an accessor's receiver parameter against its
/// owning class key in [`render_wrapper_fn`].
pub(crate) fn peel_receiver_key(ty: &syn::Type) -> TypeKey {
    let core = match ty {
        syn::Type::Reference(r) => &*r.elem,
        other => other,
    };
    if let syn::Type::Path(tp) = core {
        if let Some(seg) = tp.path.segments.last() {
            if seg.ident == "Option" {
                if let syn::PathArguments::AngleBracketed(ab) = &seg.arguments {
                    if let Some(syn::GenericArgument::Type(inner)) = ab.args.first() {
                        let inner_core = match inner {
                            syn::Type::Reference(r) => &*r.elem,
                            other => other,
                        };
                        return TypeKey::from_type(inner_core);
                    }
                }
            }
        }
    }
    TypeKey::from_type(core)
}

/// Build a single top-level (free-function) wrapper as a [`kt::KtFun`].
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
/// (the inherited `NativeHandle` scope for a `ptr_class` — `this.ptr` + lock —
/// or `this.bytes` for a `value_class` blob). The JNINative extern/call is
/// unchanged (keyed on the Rust ident), so only the Kotlin wrapper relocates.
/// The Kotlin surface of a wrapper: the assembled `KtFun` with every
/// parameter/return type in place but **no body**, plus the emission
/// internals [`render_wrapper_fn`] needs to fill that body. One derivation of
/// the overload surface, shared by emission (which adds the body) and
/// [`validate_symbols`](crate::api::lang::jnigen::jni::validate_symbols)
/// (which erases `fun` to a JVM signature), so the emitted overload and the
/// validated one cannot drift (issue #89).
pub(crate) struct WrapperSurface {
    /// The wrapper with its full signature and an empty body. The validator
    /// reads this; [`render_wrapper_fn`] fills the body and adds the KDoc.
    pub fun: kt::KtFun,
    // Emission-only internals — computed while assembling the signature,
    // consumed by `render_wrapper_fn`; opaque to the validator.
    params: Vec<Param>,
    out: OutputPlan,
    sink: ErrorSink,
    jni_call: String,
}

/// Build the [`WrapperSurface`]: everything [`render_wrapper_fn`] does up to
/// (but not including) the body render — the single surface-signature
/// derivation. Validation calls this directly and skips the body work
/// (`build_native_call` / `render_body` / KDoc / opaque-lock collection).
pub(crate) fn build_wrapper_surface(
    ext: &JniGen,
    f: &syn::ItemFn,
    registry: &Registry<KotlinMeta>,
    imports: &mut BTreeSet<String>,
    kotlin_name_override: Option<&str>,
    receiver_key: Option<&TypeKey>,
) -> Option<WrapperSurface> {
    let fplan = JniFunctionPlan::build(ext, registry, f).ok()?;
    // The Kotlin extern in `JNINative` is keyed on the Rust ident (the
    // plan's `jni_method`). The per-entry `.name("...")` override only
    // changes the *user-facing* Kotlin wrapper name; the JNI call still has
    // to hit the one extern that the Rust extern actually emits.
    let kt_name = match kotlin_name_override {
        Some(n) => n.to_string(),
        None => kt_snake_to_camel(&f.sig.ident.to_string()),
    };
    let jni_call = fplan.jni_method.clone();
    let (params, receiver_idx) = classify_params(ext, &fplan, registry, imports, receiver_key)?;
    let out = classify_output(ext, f, &fplan, registry, imports)?;
    let r_ty = out.kt_return.clone().unwrap_or_else(kt::KtType::unit);
    let sink = error_sink_parts(ext, f, &fplan, registry, imports, &r_ty)?;

    let mut fun = kt::KtFun::new(&kt_name).vis(kt::Vis::Public);
    if let Some(g) = &out.generic {
        fun = fun.generic(g);
    }
    for (i, p) in params.iter().enumerate() {
        // The receiver param is bound to `this` — not a rendered parameter.
        if Some(i) == receiver_idx {
            continue;
        }
        fun = fun.param(kt::KtParam::new(&p.kt_name, p.kt_type.clone()));
    }
    // The error callback — **required**: the generated code never throws; the
    // consumer decides how a failure surfaces (e.g. by throwing its own type).
    // When an output-expansion builder/fold lambda exists, it must remain the
    // **trailing** lambda (Kotlin trailing-lambda call syntax), so `onError` is
    // placed *before* it — but *after* any non-lambda `builder_lead` (`acc: A`),
    // which is passed positionally. Without a builder lambda, `onError` is the
    // last param.
    let onerr = kt::KtParam::new("onError", sink.onerr_type.clone());
    if let Some((bp_name, bp_ty)) = &out.builder_param {
        if let Some((lead_name, lead_ty)) = &out.builder_lead {
            fun = fun.param(kt::KtParam::new(lead_name, lead_ty.clone()));
        }
        fun = fun
            .param(onerr)
            .param(kt::KtParam::new(bp_name, bp_ty.clone()))
            .annotation("Suppress(\"UNCHECKED_CAST\")");
    } else {
        fun = fun.param(onerr);
    }
    if let Some(rt) = &out.kt_return {
        fun = fun.returns(rt.clone());
    }
    Some(WrapperSurface {
        fun,
        params,
        out,
        sink,
        jni_call,
    })
}

pub(crate) fn render_wrapper_fn(
    ext: &JniGen,
    f: &syn::ItemFn,
    registry: &Registry<KotlinMeta>,
    imports: &mut BTreeSet<String>,
    kotlin_name_override: Option<&str>,
    receiver_key: Option<&TypeKey>,
) -> Option<kt::KtFun> {
    let surface = build_wrapper_surface(
        ext,
        f,
        registry,
        imports,
        kotlin_name_override,
        receiver_key,
    )?;
    let WrapperSurface {
        mut fun,
        params,
        out,
        sink,
        jni_call,
    } = surface;
    // KDoc: the Rust fn's `///` prose first, then generated notes for every
    // position an expansion reshaped away from the Rust signature (N1).
    // Emission-only — the validator skips it.
    if let Some(doc) = wrapper_kdoc(f, registry) {
        fun = fun.kdoc(doc);
    }
    // Collect the opaque-handle params so we can scaffold pointer-ordered
    // synchronized blocks around them.
    let opaques = collect_opaques(&params);
    let is_unit = fun.ret.is_none();
    let body_expr = build_native_call(ext, &jni_call, &params, &out);
    let body = render_body(ext, &params, &opaques, &sink, &body_expr, is_unit, imports);
    Some(fun.body(body))
}

/// Render one declared const (see `ConstDecl`): a **private** nullary helper
/// — the standard wrapper fn over the synthetic getter signature
/// ([`const_getter_fn`]), reused verbatim so the const's type crosses through
/// the ordinary output machinery — plus the public lazily-initialized `val`
/// that calls it once, on first use (see [`render_val_over_helper`]).
pub(crate) fn render_const_val(
    ext: &JniGen,
    package: &str,
    c: &syn::ItemConst,
    registry: &Registry<KotlinMeta>,
    imports: &mut BTreeSet<String>,
    kotlin_name_override: Option<&str>,
) -> Option<(kt::KtFun, kt::KtProperty)> {
    let getter = const_getter_fn(c);
    let default = kt_snake_to_camel(&getter.sig.ident.to_string());
    let helper_name = ext.mangle_fun(package, &default);
    let helper = render_wrapper_fn(ext, &getter, registry, imports, Some(&helper_name), None)?;
    let val_name = kotlin_name_override
        .map(str::to_string)
        .unwrap_or_else(|| c.ident.to_string());
    let framework_line = format!(
        "Mirrors the Rust `#[prebindgen]` const `{}` (read lazily, once, through \
         the generated JNI getter on first use).",
        c.ident
    );
    let kdoc = crate::api::lang::jnigen::util::doc_string(&c.attrs)
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
    ext: &JniGen,
    package: &str,
    f: &syn::ItemFn,
    registry: &Registry<KotlinMeta>,
    imports: &mut BTreeSet<String>,
    kotlin_name_override: Option<&str>,
) -> Option<(kt::KtFun, kt::KtProperty)> {
    let default = kt_snake_to_camel(&f.sig.ident.to_string());
    let helper_name = ext.mangle_fun(package, &default);
    let helper = render_wrapper_fn(ext, f, registry, imports, Some(&helper_name), None)?;
    let val_name = kotlin_name_override
        .map(str::to_string)
        .unwrap_or_else(|| f.sig.ident.to_string());
    let framework_line = format!(
        "Mirrors the Rust `#[prebindgen]` fn `{}()` (evaluated lazily, once, \
         through the generated JNI wrapper on first use).",
        f.sig.ident
    );
    let kdoc = crate::api::lang::jnigen::util::doc_string(&f.attrs)
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
    ext: &JniGen,
    package: &str,
    decl: &crate::api::lang::jnigen::jni::decl::ConstExprDecl,
    registry: &Registry<KotlinMeta>,
    imports: &mut BTreeSet<String>,
) -> Option<(kt::KtFun, kt::KtProperty)> {
    let getter = const_expr_getter_fn(&decl.kotlin_name, &decl.ty);
    let default = kt_snake_to_camel(&getter.sig.ident.to_string());
    let helper_name = ext.mangle_fun(package, &default);
    let helper = render_wrapper_fn(ext, &getter, registry, imports, Some(&helper_name), None)?;
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
    ext: &JniGen,
    registry: &Registry<KotlinMeta>,
    mut helper: kt::KtFun,
    val_name: String,
    kdoc: String,
    imports: &mut BTreeSet<String>,
) -> Option<(kt::KtFun, kt::KtProperty)> {
    helper.vis = kt::Vis::Private;
    let helper_name = helper.name.clone();
    // A constant always carries a value type; a helper with no return would
    // mean the type never resolved — skip like an unresolvable fn.
    let val_ty = helper.ret.clone()?;
    let spec = ext.iface_spec(registry, &SpecKey::JniErrorHandler)?;
    imports.insert(spec.fqn());
    let init = format!(
        "{helper_name}(JniErrorHandler {{ je -> error(je ?: \"const {val_name}: JNI getter failed\") }})"
    );
    let prop = kt::KtProperty::val(&val_name)
        .ty(val_ty)
        .vis(kt::Vis::Public)
        .delegate(format!("lazy {{ {init} }}"))
        .kdoc(kdoc);
    Some((helper, prop))
}

/// The classified output side of a wrapper: return type, projection wrap,
/// output-expansion (builder/fold) params, and the extra call-site args —
/// everything the call-expression builder and the signature assembly must
/// agree on.
struct OutputPlan {
    kt_return: Option<kt::KtType>,
    /// Kotlin-newtype return (opaque handle / value class) — the wrap the
    /// call expression folds around the extern result.
    projection: Option<Projection>,
    /// Trailing **lambda** param (`build` / `fold`) of an output expansion.
    builder_param: Option<(String, kt::KtType)>,
    /// Non-lambda lead param (`acc: A`) — precedes `onError` positionally.
    builder_lead: Option<(String, kt::KtType)>,
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

/// The `onError` wiring: the handler's Kotlin type, the capture holder, and
/// the two prebuilt argument lists (post-call redispatch / pre-lock guard).
struct ErrorSink {
    onerr_type: kt::KtType,
    /// Short name of the generated per-thread raw capture holder.
    capture_short: String,
    /// je/ze args for the post-call `onError.run(...)` redispatch.
    call_args: String,
    /// Wrapped-default args for the pre-lock closed-handle guard.
    guard_args: String,
}

/// Type every effective input of the lowered [`JniFunctionPlan`] into a
/// [`Param`] (Kotlin name/type + call-site [`ParamMode`]). The crossing-form
/// classification comes from the plan — the same decision the Rust extern and
/// `external fun` renderers consume — this site only maps each [`InputKind`]
/// to its Kotlin surface. Returns the params plus the index of the
/// instance-method receiver (the first param whose peeled type matches
/// `receiver_key`), which is bound to `this` and dropped from the signature.
fn classify_params(
    ext: &JniGen,
    fplan: &JniFunctionPlan,
    registry: &Registry<KotlinMeta>,
    imports: &mut BTreeSet<String>,
    receiver_key: Option<&TypeKey>,
) -> Option<(Vec<Param>, Option<usize>)> {
    let mut receiver_idx: Option<usize> = None;
    let mut params: Vec<Param> = Vec::new();
    for leaf in fplan.leaves() {
        let mut name = leaf.kt_name.clone();
        let arg_ty = &leaf.ty;

        // Instance-method receiver: the first parameter whose peeled Rust type
        // is the owning class binds to `this` (so `this_ptr`/`this.ptr`/lock or
        // `this.bytes` fall out of the normal param handling) and is dropped
        // from the rendered signature.
        if receiver_idx.is_none() {
            if let Some(rk) = receiver_key {
                if &peel_receiver_key(arg_ty) == rk {
                    receiver_idx = Some(params.len());
                    name = "this".to_string();
                }
            }
        }

        // `impl Fn(args)` param: a generated typed `fun interface`
        // (`<ArgShorts>Callback`) whose `run` parameters are the flattened
        // leaves of each arg's callback plan (the arg whole when plan-less).
        // The extern receives it erased (`Any`) and the native trampoline
        // calls the typed `run` — value-blob leaves surface as their raw
        // `ByteArray` wire (the SDK wraps), so no call-site adapter exists.
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
            InputKind::VecBuild { elem, .. } => {
                // Slice/Vec of a flattenable data_class: build the Rust-side
                // Vec by pushing each element's leaves, pass the handle (see
                // the body assembly + `build_vec_build_helper_items`). The
                // high-level signature stays `List<T>` (registered below).
                let h = crate::api::lang::jnigen::jni::vec_build_helpers(ext, registry, elem)
                    .expect("vec_build_elem Some ⇒ vec_build_helpers Some");
                let elem_accesses = h
                    .plan
                    .leaves
                    .iter()
                    .filter(|l| !l.is_present_flag)
                    .map(|l| l.kt_access("__e"))
                    .collect();
                ParamMode::VecBuild {
                    base: h.base,
                    elem_accesses,
                }
            }
            InputKind::OptionScalar(sp) => {
                // Bare `Option<primitive>` / `Option<enum>`: cross as a
                // `(present, value)` pair (no boxed object). The high-level
                // signature keeps `T?`; only the call-site args split in two.
                let present_expr = format!("{name} != null");
                let value_expr = if sp.is_enum {
                    format!("{name}?.value ?: {}", sp.value_kt_zero)
                } else {
                    format!("{name} ?: {}", sp.value_kt_zero)
                };
                ParamMode::OptionScalar {
                    present_expr,
                    value_expr,
                }
            }
            InputKind::FlattenStruct(plan) => ParamMode::FlattenStruct {
                accesses: plan.leaves.iter().map(|l| l.kt_access(&name)).collect(),
            },
            InputKind::Handle { .. } => {
                // Handle → Borrow/Consume by Rust syntactic shape (locked);
                // `Option<&T>` / by-value `Option<T>` mark the param nullable
                // and the wrapper body branches on null before lock selection.
                if is_option_ref(arg_ty) {
                    ParamMode::BorrowNullable
                } else if is_option_type(arg_ty) {
                    // by-value `Option<T>` opaque → nullable consume
                    ParamMode::ConsumeNullable
                } else if matches!(arg_ty, syn::Type::Reference(_)) {
                    ParamMode::Borrow
                } else {
                    ParamMode::Consume
                }
            }
            // `@JvmInline value class` (value_blob) param: pass the erased
            // inner field (`<name>.bytes`, or `<name>?.bytes` when Option) to
            // the extern, no lock.
            InputKind::ValueUnwrap { field } => ParamMode::ValueUnwrap {
                field: field.clone(),
            },
            InputKind::Plain => ParamMode::PassThrough,
            InputKind::Callback { .. } => unreachable!("callback params handled above"),
        };

        let ty = register_kt_type(&kt_type_raw, imports);
        let kt_type = if leaf.optional { ty.nullable() } else { ty };
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
/// Each leaf is delivered with its final Kotlin type; a **value_blob** leaf
/// (`@JvmInline value class`) can't be constructed Rust-side, so the wrapper
/// installs an **adapter** that applies the Kotlin-side projection wrap
/// (`ZZenohId(raw)`) before the user callback. Leaves with no value_blob ⇒
/// the callback is passed directly (M1–M4 unchanged).
fn classify_output(
    ext: &JniGen,
    f: &syn::ItemFn,
    fplan: &JniFunctionPlan,
    registry: &Registry<KotlinMeta>,
    imports: &mut BTreeSet<String>,
) -> Option<OutputPlan> {
    let unfold = registry.unfold_plans.get(&f.sig.ident);
    // `builder_param` is the trailing **lambda** param (build / fold) as a
    // `(name, function-type)` pair. For the `Iterable` shape, the non-lambda
    // accumulator (`acc: A`) goes in `builder_lead` — it must precede
    // `onError` (a defaulted param) so the positional-`acc` call stays valid;
    // the trailing `fold` lambda follows.
    let mut builder_param: Option<(String, kt::KtType)> = None;
    let mut builder_lead: Option<(String, kt::KtType)> = None;
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
        // projection wrap (value_blob/handle) below.
        render_return_surface(&v.surface, imports)?
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
        // a **whole-element leaf** fold (`plan.element` set — String / value blob
        // / handle) `plan.source` (e.g. `String`) has no class FQN, so take the
        // element's typed view from the folder interface's element param instead.
        let class_ty = if u.whole_element {
            let spec = u.iface.as_deref()?;
            register_kt_type(&spec.params[1].typed, imports)
        } else {
            let class_fqn = ext
                .kotlin_fqn(&TypeKey::from_type(&plan.source))
                .map(|s| s.to_string())?;
            register_kt_type(&kt::KtType::cls(class_fqn), imports)
        };
        if u.iterable_fold {
            // `Vec<data_class>` fold: allocate an `ArrayList<Class>` accumulator,
            // pass the hoisted **folder-appender** singleton as `fold` (it
            // rebuilds each element via `fromParts` and appends it), and return
            // the threaded accumulator as `List<Class>` (`?`-nullable for an
            // `Option<Vec<…>>` return — `None` yields a null list). Per element
            // only the raw leaves cross — no Java object is built on the Rust side.
            let spec = u.iface.as_deref()?;
            let holder = spec.singleton_holder_name();
            let field = crate::api::lang::jnigen::jni::SINGLETON_FIELD;
            imports.insert(spec.singleton_holder_fqn());
            unfold_call_args.push(format!("ArrayList<{class_ty}>()"));
            unfold_call_args.push(format!("{holder}.{field}"));
            let list_ty = kt::KtType::generic("List", [class_ty]);
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
        // calls their typed `run` with raw jvalues (value-blob leaves surface
        // as `ByteArray` — no call-site adapter). Lambda-literal call sites
        // SAM-convert unchanged.
        generic = u.generic.map(str::to_string);
        // An `Iterable` fold — bare or `Optional`-wrapped — folds with `<A>`
        // (`acc` lead + `fold` lambda). The wrapped form returns `A?`: `None`
        // skips the fold and delivers null (matching the scalar `R?` and the
        // fixed path's null `List`), `Some(empty)` returns `acc` unchanged.
        if u.generic == Some("A") {
            let spec = u.iface.as_deref()?;
            builder_lead = Some(("acc".to_string(), kt::KtType::var_("A")));
            builder_param = Some(("fold".to_string(), spec.kt_ref(vec![kt::KtType::var_("A")])));
            unfold_call_args.push("acc".to_string());
            if spec.needs_raw() {
                imports.insert(format!("{}.asRaw", spec.package));
                unfold_call_args.push("fold.asRaw()".to_string());
            } else {
                unfold_call_args.push("fold".to_string());
            }
            let kt = if u.optional {
                kt::KtType::var_("A").nullable()
            } else {
                kt::KtType::var_("A")
            };
            (Some(kt), None)
        } else {
            let spec = u.iface.as_deref()?;
            builder_param = Some(("build".to_string(), spec.kt_ref(vec![kt::KtType::var_r()])));
            if spec.needs_raw() {
                imports.insert(format!("{}.asRaw", spec.package));
                unfold_call_args.push("build.asRaw()".to_string());
            } else {
                unfold_call_args.push("build".to_string());
            }
            let kt = if u.optional {
                kt::KtType::var_r().nullable()
            } else {
                kt::KtType::var_r()
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

/// Build the JNINative call expression. Every param maps to exactly one call
/// arg (or several, for a flattened data_class); the output plan's extra args
/// and the trailing `__cap` follow; the result is wrapped per the return
/// classification (projection / enum / erased-`Any` cast).
fn build_native_call(ext: &JniGen, jni_call: &str, params: &[Param], out: &OutputPlan) -> String {
    let mut args: Vec<String> = Vec::with_capacity(params.len());
    for p in params.iter() {
        // Flattened data_class param expands into multiple call args
        // (the leaf destructure expressions, in plan order).
        if let ParamMode::FlattenStruct { accesses } = &p.mode {
            args.extend(accesses.iter().cloned());
            continue;
        }
        // VecBuild param: the extern receives the `jlong` Vec handle the
        // wrapper body allocated and filled (`__vec_<name>`), not the `List`.
        if let ParamMode::VecBuild { .. } = &p.mode {
            args.push(format!("__vec_{}", p.kt_name));
            continue;
        }
        // OptionScalar param expands into two call args: the present flag
        // and the value-or-zero expression (in that order).
        if let ParamMode::OptionScalar {
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
            ParamMode::ValueUnwrap { field } => {
                // Inline value class → pass its erased inner field to the
                // extern (e.g. `z.bytes`: a `ByteArray`). A nullable value
                // class (`ZBytes?`) safe-navigates so it stays `ByteArray?`.
                if p.kt_type.is_nullable() {
                    format!("{}?.{}", p.kt_name, field)
                } else {
                    format!("{}.{}", p.kt_name, field)
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
            // erased `Any`), or its value-blob rebuilding adapter.
            ParamMode::Callback { call_arg } => call_arg.clone(),
            ParamMode::FlattenStruct { .. } => {
                unreachable!("FlattenStruct expanded before the single-arg match")
            }
            ParamMode::VecBuild { .. } => {
                unreachable!("VecBuild expanded before the single-arg match")
            }
            ParamMode::OptionScalar { .. } => {
                unreachable!("OptionScalar expanded before the single-arg match")
            }
        };
        args.push(arg);
    }
    // Output expansion: the builder / (acc, fold) cross just before the
    // error callback.
    args.extend(out.unfold_call_args.iter().cloned());
    // Every extern takes a trailing error callback. The wrapper passes a
    // **capture** (`__cap`) that records `(je, ze…)` and sets a flag — no
    // throw on the Rust upcall. The wrapper calls the user's `onError` after
    // the native call returns (see the body below).
    args.push("__cap".to_string());
    let mut call = format!(
        "{}.{jni_call}({})",
        ext.jni_native_class_name(),
        args.join(", ")
    );
    if let Some(p) = &out.projection {
        // Fold the wrap through the projection strategy. The wrap class is
        // the projection leaf's typed short name (Handle's typed-handle
        // class or value-class wrapper). The sentinel is the Kotlin
        // null-representation literal for the leaf wire — used only by
        // the `Niche+primitive` arm of `fold_projection_wrap`.
        let leaf_fqn = ext
            .kotlin_fqn(&p.leaf_key)
            .unwrap_or_else(|| p.leaf_key.to_string());
        let short = leaf_fqn.rsplit('.').next().unwrap_or(&leaf_fqn).to_string();
        let sentinel = projection_leaf_sentinel(p);
        call = fold_projection_wrap(&p.strategy, &call, &short, sentinel.as_deref());
    } else if out.is_enum_return {
        let enum_kt = out
            .kt_return
            .as_ref()
            .expect("enum return has a Kotlin type");
        call = format!("{enum_kt}.fromInt({call})");
    } else if out.is_option_enum_return {
        // `kt_return` renders nullable (`Priority?`); the companion lives
        // on the non-null class name.
        let enum_kt = out
            .kt_return
            .as_ref()
            .expect("Option<enum> return has a Kotlin type")
            .to_string();
        let enum_kt = enum_kt.trim_end_matches('?');
        call = format!("{call}?.let {{ {enum_kt}.fromInt(it) }}");
    } else if out.cast_return {
        // Callback delivery: the extern returns the builder's erased `Any?`;
        // cast to the shape type (`R` / `R?` / `List<R>`). The builder always
        // produces `R`, so the unchecked cast is sound — suppressed below.
        let cast_kt = out
            .kt_return
            .as_ref()
            .expect("callback delivery returns R/A");
        call = format!("({call} as {cast_kt})");
    }
    // Return delivery (`convert_output`): the extern returns the real typed
    // wire; the projection wrap above (if any) already produced the value
    // class / handle — no cast needed.
    call
}

/// The opaque-handle params (Borrow/Consume modes) — the set the lock
/// scaffold, pre-lock guards, and consume `try/finally` operate on.
fn collect_opaques(params: &[Param]) -> Vec<Opaque> {
    params
        .iter()
        .filter_map(|p| {
            let (target, consume_null, nullable) = match p.mode {
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
                _ => return None,
            };
            Some(Opaque {
                name: p.kt_name.clone(),
                target,
                consume_null,
                nullable,
            })
        })
        .collect()
}

/// Error callback `onError: <Err>Handler<R>` / `JniErrorHandler<R>` — a
/// generated typed fun interface `run(je: String?, ze…): R` whose ze params
/// are typed EXACTLY like a builder's leaves (the error channel is the
/// output channel with a fixed leading `je`). Contract: `je != null` ⇒
/// binding/system error, the native side fills the ze with defaults;
/// `je == null` ⇒ domain error, the ze carry the decomposed error. The
/// wrapper passes a SAM **capture** to the extern, then — after the native
/// call — calls `onError.run(je, ze…)` and returns its `R` if a failure
/// was recorded (no throw on the Rust upcall).
fn error_sink_parts(
    ext: &JniGen,
    f: &syn::ItemFn,
    fplan: &JniFunctionPlan,
    registry: &Registry<KotlinMeta>,
    imports: &mut BTreeSet<String>,
    r_ty: &kt::KtType,
) -> Option<ErrorSink> {
    let sink_spec = fplan.onerror_iface.as_ref()?;
    let error_plan = registry.error_plans.get(&f.sig.ident);
    // Per ze leaf: (raw capture Kotlin type, raw default literal, raw→typed
    // wrap) — off the handler interface spec (its first param is the fixed
    // `je`), defaults from the matching error-plan leaf (same declaration,
    // same order). The CAPTURE is the raw twin (what the native side calls);
    // the user's handler is the TYPED interface — the redispatch wraps.
    let ze_info: Vec<(kt::KtType, String, crate::api::lang::jnigen::jni::WrapKind)> = sink_spec
        .params[1..]
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let leaf = error_plan
                .map(|pl| &pl.leaves[i])
                .expect("ze params exist only with an error plan");
            let default =
                ze_default_kotlin(&leaf_default(ext, registry, leaf), p.raw.is_nullable());
            if let Some(fqn) = p.wrap.class_fqn() {
                imports.insert(fqn.to_string());
            }
            (p.raw.clone(), default, p.wrap.clone())
        })
        .collect();
    let n_ze = ze_info.len();
    let onerr_type = sink_spec.kt_ref(vec![r_ty.clone()]);
    // The capture is a generated zero-alloc thread-local holder of the RAW
    // twin (no SAM lambda, no `Ref`-boxed captured vars) — its short name in
    // raw body text needs the import registered.
    imports.insert(sink_spec.capture_fqn());
    let capture_short = sink_spec.capture_name();
    // The je/ze argument list to call the user's typed `onError.run`. The
    // native side ALWAYS fills the raw ze (real values or defaults), so the
    // nullable capture slots are non-null whenever `__cap_failed` — assert
    // with `!!` for non-null params, then wrap raw → typed.
    let call_args = std::iter::once("__cap.je".to_string())
        .chain((0..n_ze).map(|i| {
            let (raw, _, wrap) = &ze_info[i];
            if raw.is_nullable() {
                wrap.wrap_expr(&format!("__cap.ze{i}"), true)
            } else {
                wrap.wrap_expr(&format!("__cap.ze{i}!!"), false)
            }
        }))
        .collect::<Vec<_>>()
        .join(", ");
    // Default-ze args for a synchronous (pre-call) closed-handle guard, which
    // calls the typed `onError.run` directly (a binding-class error ⇒ wrapped
    // raw defaults: `ZErr(0L)`, `ZId(ByteArray(0))`, …).
    let guard_args = std::iter::once("\"Operation on a closed native handle.\"".to_string())
        .chain(ze_info.iter().map(|(raw, def, wrap)| {
            if raw.is_nullable() {
                "null".to_string()
            } else {
                wrap.wrap_expr(def, false)
            }
        }))
        .collect::<Vec<_>>()
        .join(", ");
    Some(ErrorSink {
        onerr_type,
        capture_short,
        call_args,
        guard_args,
    })
}

/// Pre-lock closed-handle guards: a racy-but-safe `isClosed()` check before
/// the lock, returning `onError.run(...)` (function-level return; no throw).
/// Racy: a close between this check and the native call is caught by the
/// Rust-side converter guard (the tag bit survives the race), which routes
/// through the same error channel.
fn render_prelock_guards(opaques: &[Opaque], guard_args: &str, is_unit: bool) -> kt::Code {
    let mut guards = kt::Code::new();
    for o in opaques {
        let cond = if o.nullable {
            format!("{n} != null && {t}.isClosed()", n = o.name, t = o.target)
        } else {
            format!("{t}.isClosed()", t = o.target)
        };
        guards = if is_unit {
            guards.wline(format!(
                "if ({cond}) {{ onError.run({guard_args}); return }}"
            ))
        } else {
            guards.wline(format!("if ({cond}) return onError.run({guard_args})"))
        };
    }
    guards
}

/// The call in statement position, `bind`-prefixed (`""` / `"val __ret = "`),
/// wrapped in a consume `try/finally` when any handle is consumed.
fn render_value_stmt(bind: &str, body_expr: &str, opaques: &[Opaque]) -> kt::Code {
    let consume_stmts: Vec<&str> = opaques
        .iter()
        .filter_map(|o| o.consume_null.as_deref())
        .collect();
    if consume_stmts.is_empty() {
        kt::Code::new().wline(format!("{bind}{body_expr}"))
    } else {
        let mut fin = kt::Code::new();
        for s in consume_stmts {
            fin = fin.line(s);
        }
        kt::Code::new().try_finally(bind, kt::Code::new().wline(body_expr), fin)
    }
}

/// The core call statement: `bind` + a single Kotlin **expression**
/// evaluating to the call's result. Handle params contribute
/// pointer-binding statements and a deadlock-safe `withSortedHandleLocks`
/// acquisition; the whole thing is expression-shaped (via `run { … }` where
/// statements are needed) so the caller can bind it to `__ret`, rethrow a
/// captured sink error, then return.
fn render_core_stmt(
    ext: &JniGen,
    opaques: &[Opaque],
    body_expr: &str,
    imports: &mut BTreeSet<String>,
    bind: &str,
) -> kt::Code {
    // Under-lock pointer reads. The closed-handle check is done pre-lock
    // (`prelock_guards`, → `onError`); these just bind the ptr the call
    // passes. A handle closed after the guard carries the tag bit (odd
    // value), which the Rust-side converter guard rejects — never
    // dereferenced.
    let mut ptr_binds = kt::Code::new();
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
        kt::Code::new().blk(format!("{bind}run {{"), |c| {
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
            kt::Code::new().blk(format!("{bind}withSortedHandleLocks({targets}) {{"), |c| {
                c.push(ptr_binds)
                    .push(render_value_stmt("", body_expr, opaques))
            })
        } else {
            let mut adds = kt::Code::new();
            for o in opaques {
                adds = if o.nullable {
                    adds.line(format!("{n}?.let {{ __locks.add(it) }}", n = o.name))
                } else {
                    adds.line(format!("__locks.add({t})", t = o.target))
                };
            }
            kt::Code::new().blk(format!("{bind}run {{"), |c| {
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
fn render_body(
    ext: &JniGen,
    params: &[Param],
    opaques: &[Opaque],
    sink: &ErrorSink,
    body_expr: &str,
    is_unit: bool,
    imports: &mut BTreeSet<String>,
) -> kt::Code {
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
    let mut b = render_prelock_guards(opaques, &sink.guard_args, is_unit)
        .line(format!("val __cap = {}.acquire()", sink.capture_short));
    let failed_check = format!("if (__cap.failed) return onError.run({})", sink.call_args);
    let bind = if is_unit { "" } else { "val __ret = " };
    if vec_build.is_empty() {
        b = b
            .push(render_core_stmt(ext, opaques, body_expr, imports, bind))
            .wline(failed_check);
    } else {
        let native = ext.jni_native_class_name();
        for (name, base, _) in &vec_build {
            let new_m = crate::api::lang::jnigen::jni::vec_helper_method_name(ext, base, "New");
            b = b.wline(format!("val __vec_{name} = {native}.{new_m}({name}.size)"));
        }
        // `try { fill…; <core> } finally { free… }`: Kotlin `try` is an
        // expression, so for a non-unit fn `__ret` binds to the core call
        // (the block's last expression). A push runs no JVM upcall, so the
        // loop needs no per-element failure check.
        let mut fill = kt::Code::new();
        for (name, base, accesses) in &vec_build {
            let push_m = crate::api::lang::jnigen::jni::vec_helper_method_name(ext, base, "Push");
            let args = std::iter::once(format!("__vec_{name}"))
                .chain(accesses.iter().cloned())
                .collect::<Vec<_>>()
                .join(", ");
            fill = fill.blk(format!("for (__e in {name}) {{"), |c| {
                c.wline(format!("{native}.{push_m}({args})"))
            });
        }
        let mut free = kt::Code::new();
        for (name, base, _) in &vec_build {
            let free_m = crate::api::lang::jnigen::jni::vec_helper_method_name(ext, base, "Free");
            free = free.wline(format!("{native}.{free_m}(__vec_{name})"));
        }
        let core = render_core_stmt(ext, opaques, body_expr, imports, "");
        b = b
            .try_finally(bind, fill.push(core), free)
            .wline(failed_check);
    }
    if !is_unit {
        b = b.line("return __ret");
    }
    b
}

/// The Kotlin typing of one delivered lambda leaf: `(builder_kt, wire_kt,
/// wrap, is_value_blob)` — the type the *user's* lambda sees, the type the
/// extern delivers, and the expression rebuilding the former from the latter
/// (`pk` is the adapter's parameter name; passthrough unless the leaf is a
/// `value_blob`, whose `@JvmInline value class` can't be built Rust-side).
/// Shared by the unfold builder/fold lambda and the callback lambda params.
pub(crate) fn unfold_leaf_kt(
    ext: &JniGen,
    registry: &Registry<KotlinMeta>,
    out_ty: &syn::Type,
    nullable: bool,
    pk: &str,
    imports: &mut BTreeSet<String>,
) -> Option<(kt::KtType, String, String, bool)> {
    let proj = registry
        .output_entry(out_ty)
        .and_then(|e| e.metadata.projection.clone());
    let is_vb = proj
        .as_ref()
        .map(|p| p.kind == crate::api::lang::jnigen::jni::ProjectionKind::ValueBlob)
        .unwrap_or(false);
    // builder_kt: enum → Int; otherwise the normal classified type
    // (handle class / value class / String / ByteArray / Long …).
    let builder_kt = if ext.is_kotlin_enum(&enum_probe_type(out_ty)) {
        kt::KtType::int()
    } else {
        let rt: syn::ReturnType = syn::parse_quote!(-> #out_ty);
        classify_return(ext, &rt, registry, imports)?.0?
    };
    let (mut wire_kt, wrap) = if is_vb {
        let p = proj.as_ref().unwrap();
        // Wrap class = the projection leaf's typed short name — NOT
        // `builder_kt` (which is `Short?` for an `Option<…>` leaf and would
        // leak the `?` into the constructor call).
        let leaf_fqn = ext
            .kotlin_fqn(&p.leaf_key)
            .unwrap_or_else(|| p.leaf_key.to_string());
        let short = leaf_fqn.rsplit('.').next().unwrap_or(&leaf_fqn).to_string();
        let sentinel = projection_leaf_sentinel(p);
        let mut wrap = fold_projection_wrap(&p.strategy, pk, &short, sentinel.as_deref());
        // A `nullable` leaf (an `Option` nesting step on its path) makes the
        // wire nullable even when the strategy itself is `Direct` — guard the
        // wrap so a null wire stays null instead of feeding the constructor.
        if nullable
            && matches!(
                p.strategy,
                crate::api::lang::jnigen::jni::FoldStrategy::Base
            )
        {
            wrap = format!("{pk}?.let {{ {short}(it) }}");
        }
        (projection_wire_return(p), wrap)
    } else {
        (builder_kt.to_string(), pk.to_string())
    };
    let builder_kt = if nullable {
        wire_kt.push('?');
        builder_kt.nullable()
    } else {
        builder_kt
    };
    Some((builder_kt, wire_kt, wrap, is_vb))
}

/// Kotlin parameter names for a plan's delivered leaves, in leaf order. The
/// names are the author-supplied [`UnfoldLeaf::name`]s (`handle` for a root
/// identity), emitted **verbatim** — no casing/keyword escaping (the author
/// writes valid Kotlin identifiers) and no dedup (uniqueness is enforced in
/// `core::unfold`).
///
/// [`UnfoldLeaf::name`]: crate::api::core::unfold::UnfoldLeaf::name
pub(crate) fn plan_leaf_names(leaves: &[crate::api::core::unfold::UnfoldLeaf]) -> Vec<String> {
    leaves.iter().map(|leaf| leaf.name.clone()).collect()
}

/// Lambda parameter name for a whole-value (plan-less) callback arg: the
/// decapitalized bare type short (`ZQuery` → `zQuery`), peeling a `&` /
/// `Option<…>` layer; `arg{i}` for non-path shapes.
pub(crate) fn whole_value_name(ty: &syn::Type, i: usize) -> String {
    let mut t = ty.clone();
    if let syn::Type::Reference(r) = &t {
        t = (*r.elem).clone();
    }
    if let syn::Type::Path(tp) = &t {
        if let Some(last) = tp.path.segments.last() {
            if last.ident == "Option" {
                if let syn::PathArguments::AngleBracketed(ab) = &last.arguments {
                    if let Some(syn::GenericArgument::Type(inner)) = ab.args.first() {
                        t = inner.clone();
                    }
                }
            }
        }
    }
    if let syn::Type::Path(tp) = &t {
        if let Some(last) = tp.path.segments.last() {
            let s = last.ident.to_string();
            let mut cs = s.chars();
            if let Some(f) = cs.next() {
                return kt_param_name(&format!("{}{}", f.to_lowercase(), cs.as_str()));
            }
        }
    }
    format!("arg{i}")
}

/// The Kotlin default literal for an error `ze` leaf, used when the
/// **wrapper itself** raises a binding-class error before the native call
/// (the pre-lock closed-handle guard) and must call `onError.run` with
/// builder-typed ze params. Rendered from the shared [`leaf_default`]
/// classification — the same one the native `__ze_defaults` jvalues use —
/// so the two sides cannot drift. The [`LeafDefault::Null`] arm renders
/// `null` (valid for plan-nullable params; an unknown non-null object kind
/// gets a `null!!` assertion — no constructible default exists for it).
fn ze_default_kotlin(d: &LeafDefault, kt_nullable: bool) -> String {
    match d {
        LeafDefault::Null if kt_nullable => "null".to_string(),
        LeafDefault::Null => "null!!".to_string(),
        LeafDefault::Prim(p) => p.kotlin_zero().to_string(),
        LeafDefault::Str => "\"\"".to_string(),
        LeafDefault::Bytes => "ByteArray(0)".to_string(),
        LeafDefault::List => "emptyList()".to_string(),
    }
}

/// Fall-back Kotlin type derived directly from the JNI wire type.
/// Returns the **non-nullable** Kotlin base name — the use site adds
/// a `?` suffix when the entry's Rust type is `Option<…>` (via
/// [`is_option_type`]), so this helper must not double up.
pub(crate) fn kotlin_for_wire(wire: &syn::Type) -> Option<kt::KtType> {
    if let Some(p) = JniPrim::from_wire(wire) {
        return Some(kt::KtType::cls(p.kotlin_type()));
    }
    if let syn::Type::Path(tp) = wire {
        if let Some(last) = tp.path.segments.last() {
            let kt = match last.ident.to_string().as_str() {
                "JString" | "jstring" => "String",
                "JByteArray" | "jbyteArray" => "ByteArray",
                "JObject" | "jobject" | "JClass" => "Any",
                _ => return None,
            };
            return Some(kt::KtType::cls(kt));
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
pub(crate) fn classify_return(
    ext: &JniGen,
    output: &syn::ReturnType,
    registry: &Registry<KotlinMeta>,
    imports: &mut BTreeSet<String>,
) -> Option<(
    Option<kt::KtType>,
    Option<crate::api::lang::jnigen::jni::Projection>,
)> {
    let (surface, _canonical) = ReturnSurface::classify(ext, registry, output);
    render_return_surface(&surface, imports)
}

/// Map a classified [`ReturnSurface`] back to the historical
/// `(kt_return, projection)` pair, registering/shortening the Kotlin names
/// against `imports` at render time (the plan stores unshortened types, so
/// import registration stays identical across all consumers). Panics on an
/// unregistered projection FQN — the same Kotlin-render-time failure
/// `classify_return` always had.
pub(crate) fn render_return_surface(
    surface: &ReturnSurface,
    imports: &mut BTreeSet<String>,
) -> Option<(
    Option<kt::KtType>,
    Option<crate::api::lang::jnigen::jni::Projection>,
)> {
    match surface {
        ReturnSurface::Skip => None,
        ReturnSurface::Unit => Some((None, None)),
        ReturnSurface::Projected {
            projection,
            leaf_fqn,
        } => {
            let fqn = leaf_fqn.clone().unwrap_or_else(|| {
                panic!(
                    "classify_return: projection return type `{}` has no Kotlin FQN registered \
                     — every opaque/value class must be declared via `JniGen::ptr_class(...)` \
                     / `JniGen::value_class(...)`.",
                    projection.leaf_key
                )
            });
            let short = register_fqn(&fqn, imports);
            Some((
                Some(handle_kt_type(
                    &projection.strategy,
                    &kt::KtType::cls(short),
                )),
                Some(projection.clone()),
            ))
        }
        ReturnSurface::Plain { kt } => Some((Some(register_kt_type(kt, imports)), None)),
    }
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
fn wrapper_kdoc(f: &syn::ItemFn, registry: &Registry<KotlinMeta>) -> Option<String> {
    let prose = crate::api::lang::jnigen::util::doc_string(&f.attrs);
    let notes = shape_notes(f, registry);
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
/// (what `onError` receives). Reads the same resolved plan maps the C7
/// report uses.
fn shape_notes(f: &syn::ItemFn, registry: &Registry<KotlinMeta>) -> Option<String> {
    let fn_ident = &f.sig.ident;
    let mut notes: Vec<String> = Vec::new();

    let mut plans: Vec<(&syn::Ident, &crate::api::core::expand::FoldPlan)> = registry
        .expansion_plans
        .iter()
        .filter(|((func, _), _)| func == fn_ident)
        .map(|((_, param), plan)| (param, plan))
        .collect();
    plans.sort_by_key(|(p, _)| p.to_string());
    for (param, plan) in plans {
        let target = plan.target.to_token_stream().to_string();
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

    if let Some(plan) = registry.unfold_plans.get(fn_ident) {
        let source = plan.source.to_token_stream().to_string();
        let leaves: Vec<&str> = plan.leaves.iter().map(|l| l.name.as_str()).collect();
        match plan.delivery {
            crate::api::core::unfold::Delivery::Callback if !leaves.is_empty() => {
                notes.push(format!(
                    "The Rust `{source}` result is delivered decomposed: the builder \
                     callback receives (`{}`).",
                    leaves.join("`, `")
                ));
            }
            crate::api::core::unfold::Delivery::Return => {
                notes.push(format!(
                    "The Rust `{source}` result is converted and returned as a single value."
                ));
            }
            _ => {}
        }
    }

    if let Some(plan) = registry.error_plans.get(fn_ident) {
        let source = plan.source.to_token_stream().to_string();
        let leaves: Vec<&str> = plan.leaves.iter().map(|l| l.name.as_str()).collect();
        notes.push(format!(
            "On failure `onError` receives `je` plus the decomposed Rust `{source}` \
             error (`{}`).",
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
pub(crate) fn source_item_doc<M>(registry: &Registry<M>, key: &TypeKey) -> Option<String> {
    let ident = bare_path_ident(&key.to_type())?;
    let attrs = registry
        .structs
        .get(&ident)
        .map(|(s, _)| s.attrs.as_slice())
        .or_else(|| registry.enums.get(&ident).map(|(e, _)| e.attrs.as_slice()))?;
    crate::api::lang::jnigen::util::doc_string(attrs)
}
